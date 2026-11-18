// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 B612S <sem.alimov@inbox.ru>

#include "backup/SnapshotStore.hpp"

#include "app/ConfigManager.hpp"
#include "util/Format.hpp"

#include <QDebug>
#include <QDir>
#include <QFile>
#include <QFileInfo>
#include <QJsonDocument>
#include <QProcess>
#include <QRegularExpression>
#include <QSaveFile>
#include <QStorageInfo>

#include <algorithm>
#include <unistd.h>

namespace {

const QRegularExpression &idPattern()
{
    static const QRegularExpression pattern(
        QStringLiteral("^\\d{4}-\\d{2}-\\d{2}_\\d{2}-\\d{2}-\\d{2}$"));
    return pattern;
}

QDateTime startedAtFromId(const QString &id)
{
    return QDateTime::fromString(id, QStringLiteral("yyyy-MM-dd_HH-mm-ss"));
}

QString latestLinkName()
{
    return QStringLiteral("latest");
}

} // namespace

SnapshotStore::SnapshotStore(ConfigManager *config, QObject *parent)
    : QObject(parent)
    , m_config(config)
{
    refresh();
    connect(m_config, &ConfigManager::targetDirChanged, this, [this]() {
        refresh();
        emit changed();
    });
}

QString SnapshotStore::targetDir() const
{
    return m_config->targetDir();
}

QString SnapshotStore::snapshotsPath() const
{
    return targetDir() + QStringLiteral("/snapshots");
}

QStringList SnapshotStore::ids() const
{
    return m_ids;
}

QString SnapshotStore::latestId() const
{
    return m_latestId;
}

QString SnapshotStore::latestPath() const
{
    return snapshotsPath() + QLatin1Char('/') + latestLinkName();
}

int SnapshotStore::count() const
{
    return m_ids.size();
}

qint64 SnapshotStore::availableBytes() const
{
    return m_availableBytes;
}

QString SnapshotStore::availableText() const
{
    return m_availableBytes < 0 ? QString() : Format::bytes(m_availableBytes);
}

QString SnapshotStore::homeRoot() const
{
    return QStringLiteral("home/") + QDir::home().dirName();
}

QString SnapshotStore::snapshotPath(const QString &id) const
{
    return snapshotsPath() + QLatin1Char('/') + id;
}

bool SnapshotStore::isValidId(const QString &id) const
{
    return idPattern().match(id).hasMatch() && !id.contains(QStringLiteral(".."));
}

SnapshotInfo SnapshotStore::snapshot(const QString &id) const
{
    SnapshotInfo info;
    if (!isValidId(id))
        return info;

    const QString metaPath = snapshotPath(id) + QStringLiteral("/meta.json");
    QFile file(metaPath);
    if (file.open(QIODevice::ReadOnly)) {
        const QJsonDocument document = QJsonDocument::fromJson(file.readAll());
        if (document.isObject())
            info = snapshotFromJson(document.object());
    }

    info.id = id;
    if (!info.startedAt.isValid())
        info.startedAt = startedAtFromId(id);
    if (info.status.isEmpty())
        info.status = QStringLiteral("ok");
    return info;
}

bool SnapshotStore::writeMeta(const SnapshotInfo &info) const
{
    if (!isValidId(info.id))
        return false;
    const QString directory = snapshotPath(info.id);
    if (!QDir().mkpath(directory))
        return false;

    QSaveFile file(directory + QStringLiteral("/meta.json"));
    if (!file.open(QIODevice::WriteOnly))
        return false;
    file.write(QJsonDocument(snapshotToJson(info)).toJson(QJsonDocument::Indented));
    return file.commit();
}

bool SnapshotStore::setLatest(const QString &id)
{
    if (!isValidId(id))
        return false;
    const QString link = latestPath();
    QFile::remove(link);
    // Relative link, so the backup volume can be moved or mounted elsewhere.
    if (::symlink(id.toLocal8Bit().constData(), link.toLocal8Bit().constData()) != 0) {
        m_lastError = QStringLiteral("cannot create symbolic link: %1").arg(link);
        return false;
    }
    m_latestId = id;
    emit changed();
    return true;
}

void SnapshotStore::recomputeLatest()
{
    QString latest;
    const QString link = latestPath();
    const QFileInfo linkInfo(link);
    if (linkInfo.isSymLink()) {
        const QString target = QFileInfo(linkInfo.symLinkTarget()).fileName();
        if (m_ids.contains(target))
            latest = target;
    }
    if (latest.isEmpty() && !m_ids.isEmpty())
        latest = m_ids.first();
    if (latest != m_latestId) {
        m_latestId = latest;
        emit changed();
    }
}

void SnapshotStore::updateAvailableBytes()
{
    QStorageInfo storage(targetDir());
    const qint64 available = storage.isValid() && storage.isReady() ? storage.bytesAvailable() : -1;
    if (available != m_availableBytes) {
        m_availableBytes = available;
        emit changed();
    }
}

void SnapshotStore::refresh()
{
    const QDir directory(snapshotsPath());
    QStringList found;
    if (directory.exists()) {
        const QFileInfoList entries =
            directory.entryInfoList(QDir::Dirs | QDir::NoDotAndDotDot, QDir::Name);
        for (const QFileInfo &entry : entries) {
            if (idPattern().match(entry.fileName()).hasMatch())
                found.append(entry.fileName());
        }
    }
    // Ids sort lexicographically exactly like they sort chronologically.
    std::sort(found.begin(), found.end(), std::greater<QString>());

    if (found != m_ids) {
        m_ids = found;
        m_lastError.clear();
        emit changed();
    }
    recomputeLatest();
    updateAvailableBytes();
}

QVariantMap SnapshotStore::info(const QString &id) const
{
    const SnapshotInfo snapshotInfo = snapshot(id);
    QVariantMap map;
    map.insert(QStringLiteral("id"), snapshotInfo.id);
    map.insert(QStringLiteral("startedAt"), snapshotInfo.startedAt);
    map.insert(QStringLiteral("finishedAt"), snapshotInfo.finishedAt);
    map.insert(QStringLiteral("status"), snapshotInfo.status);
    map.insert(QStringLiteral("totalBytes"), snapshotInfo.totalBytes);
    map.insert(QStringLiteral("totalFiles"), snapshotInfo.totalFiles);
    map.insert(QStringLiteral("transferredBytes"), snapshotInfo.transferredBytes);
    map.insert(QStringLiteral("transferredFiles"), snapshotInfo.transferredFiles);
    map.insert(QStringLiteral("durationSeconds"), snapshotInfo.durationSeconds);
    map.insert(QStringLiteral("error"), snapshotInfo.error);
    map.insert(QStringLiteral("warnings"), snapshotInfo.warnings);
    map.insert(QStringLiteral("sources"), snapshotInfo.sources);
    map.insert(QStringLiteral("path"), snapshotPath(id));
    map.insert(QStringLiteral("totalText"), Format::bytes(snapshotInfo.totalBytes));
    map.insert(QStringLiteral("addedText"), Format::bytes(snapshotInfo.transferredBytes));
    return map;
}

bool SnapshotStore::removeSnapshot(const QString &id, QString *error)
{
    if (!isValidId(id)) {
        if (error)
            *error = QStringLiteral("invalid snapshot id");
        m_lastError = QStringLiteral("invalid snapshot id: %1").arg(id);
        return false;
    }
    const QString path = snapshotPath(id);
    if (!QFileInfo::exists(path)) {
        if (error)
            *error = QStringLiteral("snapshot does not exist");
        return false;
    }

    // Deleting a tree with hard links is just a lot of unlinks; rm -rf is both
    // faster and simpler than walking it in Qt.
    QProcess process;
    process.start(QStringLiteral("rm"), {QStringLiteral("-rf"), QStringLiteral("--"), path});
    if (!process.waitForFinished(300000) || process.exitCode() != 0) {
        const QString message = QString::fromLocal8Bit(process.readAllStandardError()).trimmed();
        if (error)
            *error = message.isEmpty() ? QStringLiteral("rm failed") : message;
        m_lastError = *error;
        return false;
    }

    // Repair the "latest" link: it may have pointed at the deleted snapshot.
    refresh();
    if (m_ids.isEmpty())
        QFile::remove(latestPath());
    else
        setLatest(m_ids.first());
    return true;
}

QStringList SnapshotStore::prune()
{
    Retention::Policy policy;
    policy.hourlyHours = m_config->keepHourlyHours();
    policy.dailyDays = m_config->keepDailyDays();
    policy.weeklyWeeks = m_config->keepWeeklyWeeks();

    QList<SnapshotInfo> snapshots;
    snapshots.reserve(m_ids.size());
    for (const QString &id : m_ids)
        snapshots.append(snapshot(id));

    const QStringList removed = Retention::toRemove(snapshots, policy, QDateTime::currentDateTime());
    for (const QString &id : removed) {
        QString error;
        if (!removeSnapshot(id, &error))
            qWarning() << "Aion: cannot remove snapshot" << id << error;
    }
    return removed;
}
