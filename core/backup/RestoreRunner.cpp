// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 B612S <sem.alimov@inbox.ru>

#include "backup/RestoreRunner.hpp"

#include "app/ConfigManager.hpp"
#include "backup/SnapshotStore.hpp"
#include "ui/Notifier.hpp"
#include "util/Format.hpp"
#include "util/Loc.hpp"

#include <QDebug>
#include <QDir>
#include <QFileInfo>
#include <QRegularExpression>
#include <QTimer>

#include <algorithm>

namespace {

const QRegularExpression &progressPattern()
{
    static const QRegularExpression pattern(QStringLiteral("^\\s*[\\d,]+\\s+(\\d+)%"));
    return pattern;
}

bool isSafeRelative(const QString &path)
{
    if (path.isEmpty() || path.startsWith(QLatin1Char('/')))
        return false;
    const QStringList parts = path.split(QLatin1Char('/'), Qt::SkipEmptyParts);
    for (const QString &part : parts) {
        if (part == QLatin1String("..") || part == QLatin1String("."))
            return false;
    }
    return path != QLatin1String("..");
}

} // namespace

RestoreRunner::RestoreRunner(SnapshotStore *store, Notifier *notifier, QObject *parent)
    : QObject(parent)
    , m_store(store)
    , m_notifier(notifier)
{
}

bool RestoreRunner::start(const QString &snapshotId, const QStringList &relativePaths,
                          const QString &destinationDir, const QString &policy)
{
    if (m_running)
        return false;

    m_error.clear();
    m_restoredCount = 0;
    m_progress = 0;
    m_index = 0;
    m_itemPercent = 0;
    m_cancelled = false;
    m_items.clear();

    if (!m_store->isValidId(snapshotId)) {
        fail(Loc::t("error_snapshot_missing"));
        return false;
    }
    if (!QFileInfo::exists(m_store->snapshotPath(snapshotId))) {
        fail(Loc::t("error_snapshot_missing"));
        return false;
    }

    for (const QString &relative : relativePaths) {
        if (!isSafeRelative(relative)) {
            fail(Loc::t("error_invalid_path").arg(relative));
            return false;
        }
        m_items.append(relative);
    }
    if (m_items.isEmpty())
        return false;

    m_snapshotId = snapshotId;
    m_policy = policy.isEmpty() ? QStringLiteral("overwrite") : policy;

    m_destination = destinationDir;
    if (m_destination.isEmpty())
        m_destination = QStringLiteral("/");
    m_destination = QDir::cleanPath(m_destination);
    if (m_destination != QLatin1String("/") && !QDir().mkpath(m_destination)) {
        fail(Loc::t("error_target_missing").arg(m_destination));
        return false;
    }

    m_running = true;
    emit runningChanged();
    setState(QStringLiteral("restoring"));
    emit progressChanged();
    emit errorMessageChanged();

    runNext();
    return true;
}

void RestoreRunner::runNext()
{
    if (m_cancelled) {
        cleanup();
        setState(QStringLiteral("cancelled"));
        emit finished(false, m_restoredCount);
        return;
    }

    if (m_index >= m_items.size()) {
        finishOk();
        return;
    }

    const QString relative = m_items.at(m_index);
    const QString source = m_store->snapshotPath(m_snapshotId) + QLatin1Char('/') + relative;
    QString target = m_destination == QLatin1String("/")
        ? QLatin1Char('/') + relative
        : m_destination + QLatin1Char('/') + relative;
    target = QDir::cleanPath(target);

    m_currentItem = relative;
    m_itemPercent = 0;
    emit progressChanged();

    if (!QFileInfo::exists(source)) {
        qWarning() << "Aion: restore: missing source" << source;
        ++m_index;
        QTimer::singleShot(0, this, &RestoreRunner::runNext);
        return;
    }

    if (m_policy == QLatin1String("skip") && QFileInfo::exists(target)) {
        qInfo() << "Aion: restore: keeping existing" << target;
        ++m_index;
        emit progressChanged();
        QTimer::singleShot(0, this, &RestoreRunner::runNext);
        return;
    }

    if (m_policy == QLatin1String("keep_both") && QFileInfo::exists(target)) {
        target = uniqueTarget(target);
        qInfo() << "Aion: restore: restoring alongside as" << target;
    }

    // rsync creates the target itself but not its parent directories, and a
    // restore into a folder produces a deep path that does not exist yet.
    if (!QDir().mkpath(QFileInfo(target).absolutePath())) {
        fail(Loc::t("error_target_missing").arg(QFileInfo(target).absolutePath()));
        return;
    }
    m_itemError.clear();

    QStringList arguments;
    arguments << QStringLiteral("-aHAX")
              << QStringLiteral("--numeric-ids")
              << QStringLiteral("--info=progress2")
              << QStringLiteral("--stats")
              << QStringLiteral("--")
              << source << target;

    m_process = new QProcess(this);
    m_process->setProcessChannelMode(QProcess::SeparateChannels);
    connect(m_process, &QProcess::readyReadStandardOutput, this, [this]() {
        const QString output = QString::fromUtf8(m_process->readAllStandardOutput());
        for (const QString &line : output.split(QLatin1Char('\r'), Qt::SkipEmptyParts)) {
            if (const auto match = progressPattern().match(line); match.hasMatch()) {
                m_itemPercent = match.captured(1).toInt();
                const double fraction = static_cast<double>(m_index) + m_itemPercent / 100.0;
                m_progress = std::min(99, static_cast<int>(100.0 * fraction / m_items.size()));
                emit progressChanged();
            }
        }
    });
    connect(m_process, &QProcess::readyReadStandardError, this, [this]() {
        m_itemError += QString::fromUtf8(m_process->readAllStandardError());
        if (m_itemError.size() > 4000)
            m_itemError = m_itemError.right(4000);
    });
    connect(m_process, &QProcess::finished, this, &RestoreRunner::handleItemFinished);
    connect(m_process, &QProcess::errorOccurred, this, [this](QProcess::ProcessError error) {
        if (error == QProcess::FailedToStart)
            fail(Loc::t("error_rsync_missing"));
    });

    m_process->start(QStringLiteral("rsync"), arguments);
}

void RestoreRunner::handleItemFinished(int exitCode, QProcess::ExitStatus status)
{
    if (m_process) {
        m_process->deleteLater();
        m_process = nullptr;
    }
    if (m_cancelled) {
        cleanup();
        setState(QStringLiteral("cancelled"));
        emit finished(false, m_restoredCount);
        return;
    }
    if (status == QProcess::CrashExit || exitCode != 0) {
        QString detail = m_itemError.trimmed();
        const QStringList lines = detail.split(QLatin1Char('\n'), Qt::SkipEmptyParts);
        if (!lines.isEmpty())
            detail = lines.last();
        fail(Loc::t("error_restore_failed")
                 .arg(m_currentItem)
                 .arg(detail.isEmpty() ? QStringLiteral("rsync %1").arg(exitCode) : detail));
        return;
    }

    ++m_restoredCount;
    ++m_index;
    m_progress = std::min(99, static_cast<int>(100.0 * m_index / m_items.size()));
    emit progressChanged();
    runNext();
}

void RestoreRunner::finishOk()
{
    m_progress = 100;
    emit progressChanged();
    cleanup();
    setState(QStringLiteral("ok"));
    if (m_notifier) {
        m_notifier->notify(Loc::t("notify_restore_done"),
                           Loc::t("notify_restore_done_body").arg(m_restoredCount));
    }
    emit finished(true, m_restoredCount);
}

void RestoreRunner::fail(const QString &message)
{
    cleanup();
    m_error = message;
    emit errorMessageChanged();
    setState(QStringLiteral("failed"));
    qWarning() << "Aion: restore failed:" << message;
    emit finished(false, m_restoredCount);
}

QString RestoreRunner::uniqueTarget(const QString &target) const
{
    const QFileInfo info(target);
    const QString stamp = QDateTime::currentDateTime().toString(QStringLiteral("yyyy-MM-dd HH-mm-ss"));
    const QString base = info.isDir() ? info.fileName()
                                      : info.completeBaseName();
    const QString extension = info.isDir() ? QString()
                                           : QStringLiteral(".") + info.completeSuffix();
    return info.path() + QLatin1Char('/') + base + QStringLiteral(" (")
           + Loc::t("restore_suffix") + QLatin1Char(' ') + stamp + QLatin1Char(')') + extension;
}

void RestoreRunner::cancel()
{
    if (!m_running)
        return;
    m_cancelled = true;
    setState(QStringLiteral("cancelling"));
    if (m_process) {
        m_process->terminate();
        QTimer::singleShot(5000, this, [this]() {
            if (m_process && m_process->state() != QProcess::NotRunning)
                m_process->kill();
        });
    }
}

void RestoreRunner::cleanup()
{
    if (m_process) {
        m_process->disconnect(this);
        if (m_process->state() != QProcess::NotRunning) {
            m_process->kill();
            m_process->waitForFinished(5000);
        }
        m_process->deleteLater();
        m_process = nullptr;
    }
    if (m_running) {
        m_running = false;
        emit runningChanged();
    }
}

void RestoreRunner::setState(const QString &state)
{
    if (m_state == state)
        return;
    m_state = state;
    emit stateChanged();
}
