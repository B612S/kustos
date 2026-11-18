// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 B612S <sem.alimov@inbox.ru>

#include "backup/BackupRunner.hpp"

#include "app/ConfigManager.hpp"
#include "backup/SnapshotStore.hpp"
#include "ui/Notifier.hpp"
#include "util/Format.hpp"
#include "util/Loc.hpp"

#include <QDebug>
#include <QDir>
#include <QFileInfo>
#include <QLockFile>
#include <QRegularExpression>
#include <QStandardPaths>
#include <QStorageInfo>
#include <QTimer>
#include <QSysInfo>

#include <algorithm>

namespace {

// "  1,234,567  45%  12.34MB/s  0:00:10"
const QRegularExpression &progressPattern()
{
    static const QRegularExpression pattern(
        QStringLiteral("^\\s*([\\d,]+)\\s+(\\d+)%\\s+(\\S+)\\s+(\\d+:\\d+:\\d+)"));
    return pattern;
}

qint64 parseNumber(const QString &text)
{
    QString digits = text;
    digits.remove(QLatin1Char(','));
    bool ok = false;
    const qint64 value = digits.toLongLong(&ok);
    return ok ? value : 0;
}

QString lastLines(const QString &text, int count)
{
    const QStringList lines = text.split(QLatin1Char('\n'), Qt::SkipEmptyParts);
    const int start = std::max(0, static_cast<int>(lines.size()) - count);
    return lines.mid(start).join(QLatin1Char('\n')).trimmed();
}

} // namespace

BackupRunner::BackupRunner(ConfigManager *config, SnapshotStore *store, Notifier *notifier,
                           QObject *parent)
    : QObject(parent)
    , m_config(config)
    , m_store(store)
    , m_notifier(notifier)
{
}

bool BackupRunner::start()
{
    if (m_running)
        return false;

    m_error.clear();
    m_warnings.clear();
    m_speed.clear();
    m_eta.clear();
    m_progress = -1;
    m_bytesDone = 0;
    m_bytesTotal = 0;
    m_totalSize = 0;
    m_totalFiles = 0;
    m_transferredSize = 0;
    m_transferredFiles = 0;
    m_cancelled = false;
    m_pending.clear();
    m_stderrTail.clear();
    m_snapshotId.clear();
    m_snapshotPath.clear();
    m_usedSources.clear();

    emit progressChanged();
    emit errorMessageChanged();

    setState(QStringLiteral("preparing"));

    QString error;
    if (!prepare(&error)) {
        fail(error);
        return false;
    }

    m_running = true;
    emit runningChanged();
    m_clock.start();
    m_lastProgressEmit = 0;
    emit currentSnapshotIdChanged();

    launch();
    return true;
}

QString BackupRunner::uniqueId(const QString &base) const
{
    QString id = base;
    int suffix = 2;
    while (QDir(m_store->snapshotPath(id)).exists())
        id = base + QStringLiteral("-%1").arg(suffix++);
    return id;
}

bool BackupRunner::prepare(QString *error)
{
    m_targetDir = m_config->targetDir();
    if (m_targetDir.isEmpty()) {
        *error = Loc::t("error_no_target");
        return false;
    }

    QDir target(m_targetDir);
    if (!target.exists() && !QDir().mkpath(m_targetDir)) {
        *error = Loc::t("error_target_missing").arg(m_targetDir);
        return false;
    }
    const QFileInfo targetInfo(m_targetDir);
    if (!targetInfo.isWritable()) {
        *error = Loc::t("error_target_not_writable").arg(m_targetDir);
        return false;
    }

    const QString snapshots = m_store->snapshotsPath();
    if (!QDir().mkpath(snapshots)) {
        *error = Loc::t("error_target_missing").arg(snapshots);
        return false;
    }

    // Sources: silently skip what disappeared (an unmounted drive, a removed
    // folder), but refuse to continue without any of them.
    for (const QString &source : m_config->sources()) {
        if (QFileInfo::exists(source)) {
            m_usedSources.append(source);
        } else {
            m_warnings.append(Loc::t("warning_source_missing").arg(source));
            qWarning() << "Aion: skipping missing source" << source;
        }
    }
    if (m_usedSources.isEmpty()) {
        *error = Loc::t("error_no_sources");
        return false;
    }

    // A destination inside a source would make rsync copy the backup into
    // itself; that is always a configuration mistake.
    const QString canonicalTarget = QFileInfo(m_targetDir).canonicalFilePath();
    for (const QString &source : m_usedSources) {
        const QString canonicalSource = QFileInfo(source).canonicalFilePath();
        if (canonicalSource.isEmpty())
            continue;
        if (canonicalTarget == canonicalSource
            || canonicalTarget.startsWith(canonicalSource + QLatin1Char('/'))) {
            *error = Loc::t("error_target_inside_source").arg(source);
            return false;
        }
    }

    const QStorageInfo storage(m_targetDir);
    if (storage.isValid() && storage.isReady() && storage.bytesAvailable() >= 0
        && storage.bytesAvailable() < 1024LL * 1024 * 1024) {
        *error = Loc::t("error_no_space").arg(Format::bytes(storage.bytesAvailable()));
        return false;
    }

    m_lock = new QLockFile(m_targetDir + QStringLiteral("/.aion.lock"));
    m_lock->setStaleLockTime(30000);
    if (!m_lock->tryLock(0)) {
        delete m_lock;
        m_lock = nullptr;
        *error = Loc::t("error_backup_running");
        return false;
    }

    m_snapshotId = uniqueId(QDateTime::currentDateTime().toString(QStringLiteral("yyyy-MM-dd_HH-mm-ss")));
    m_snapshotPath = m_store->snapshotPath(m_snapshotId);
    if (!QDir().mkpath(m_snapshotPath)) {
        *error = Loc::t("error_target_missing").arg(m_snapshotPath);
        return false;
    }
    m_startedAt = QDateTime::currentDateTime();

    const QString logDir =
        QStandardPaths::writableLocation(QStandardPaths::GenericCacheLocation)
        + QStringLiteral("/aion/logs");
    QDir().mkpath(logDir);
    m_logPath = logDir + QStringLiteral("/backup-") + m_snapshotId + QStringLiteral(".log");
    m_logFile.setFileName(m_logPath);
    if (!m_logFile.open(QIODevice::WriteOnly | QIODevice::Truncate | QIODevice::Text))
        qWarning() << "Aion: cannot open backup log" << m_logPath;
    emit logPathChanged();

    return true;
}

void BackupRunner::launch()
{
    QStringList arguments;
    QString flags = QStringLiteral("-aH");
    if (m_config->preserveAcl())
        flags += QLatin1Char('A');
    if (m_config->preserveXattr())
        flags += QLatin1Char('X');

    arguments << flags
              << QStringLiteral("--numeric-ids")
              << QStringLiteral("--relative")
              << QStringLiteral("--delete")
              << QStringLiteral("--info=progress2")
              // Accurate percentages need the full file list up front; that is
              // the short "preparing" phase rsync spends before copying.
              << QStringLiteral("--no-inc-recursive")
              << QStringLiteral("--stats");

    const QString previous = m_store->latestId();
    if (!previous.isEmpty()) {
        const QString previousPath = m_store->snapshotPath(previous);
        if (QFileInfo::exists(previousPath))
            arguments << QStringLiteral("--link-dest=") + previousPath;
    }

    for (const QString &exclude : m_config->excludes())
        arguments << QStringLiteral("--exclude=") + exclude;

    arguments << QStringLiteral("--");
    arguments << m_usedSources;
    arguments << m_snapshotPath + QLatin1Char('/');

    qInfo() << "Aion: rsync" << arguments;

    m_process = new QProcess(this);
    m_process->setProcessChannelMode(QProcess::SeparateChannels);
    connect(m_process, &QProcess::readyReadStandardOutput, this,
            [this]() { consume(m_process->readAllStandardOutput()); });
    connect(m_process, &QProcess::readyReadStandardError, this,
            [this]() { consumeErrors(m_process->readAllStandardError()); });
    connect(m_process, &QProcess::finished, this, &BackupRunner::handleFinished);
    connect(m_process, &QProcess::errorOccurred, this, [this](QProcess::ProcessError error) {
        if (error == QProcess::FailedToStart)
            fail(Loc::t("error_rsync_missing"));
    });

    setState(QStringLiteral("copying"));
    m_process->start(QStringLiteral("rsync"), arguments);
}

void BackupRunner::consume(const QByteArray &chunk)
{
    if (m_logFile.isOpen()) {
        m_logFile.write(chunk);
        m_logFile.flush();
    }
    m_pending += QString::fromUtf8(chunk);

    int start = 0;
    for (int i = 0; i < m_pending.size(); ++i) {
        const QChar character = m_pending.at(i);
        if (character == QLatin1Char('\n') || character == QLatin1Char('\r')) {
            if (i > start)
                handleLine(m_pending.mid(start, i - start));
            start = i + 1;
        }
    }
    if (start > 0)
        m_pending.remove(0, start);
}

void BackupRunner::consumeErrors(const QByteArray &chunk)
{
    if (m_logFile.isOpen()) {
        m_logFile.write(chunk);
        m_logFile.flush();
    }
    m_stderrTail += QString::fromUtf8(chunk);
    if (m_stderrTail.size() > 8000)
        m_stderrTail = m_stderrTail.right(8000);
}

void BackupRunner::handleLine(const QString &line)
{
    const QString trimmed = line.trimmed();
    if (trimmed.isEmpty())
        return;

    static const QRegularExpression filesPattern(QStringLiteral("^Number of files: ([\\d,]+)"));
    static const QRegularExpression transferredFilesPattern(
        QStringLiteral("^Number of regular files transferred: ([\\d,]+)"));
    static const QRegularExpression totalSizePattern(
        QStringLiteral("^Total file size: ([\\d,]+) bytes"));
    static const QRegularExpression transferredSizePattern(
        QStringLiteral("^Total transferred file size: ([\\d,]+) bytes"));

    const QRegularExpressionMatch match = progressPattern().match(line);
    if (match.hasMatch()) {
        m_bytesDone = parseNumber(match.captured(1));
        m_progress = match.captured(2).toInt();
        m_speed = match.captured(3);
        m_eta = match.captured(4);
        emitProgress();
        return;
    }

    if (const auto files = filesPattern.match(trimmed); files.hasMatch())
        m_totalFiles = parseNumber(files.captured(1));
    else if (const auto transferred = transferredFilesPattern.match(trimmed); transferred.hasMatch())
        m_transferredFiles = parseNumber(transferred.captured(1));
    else if (const auto total = totalSizePattern.match(trimmed); total.hasMatch()) {
        m_totalSize = parseNumber(total.captured(1));
        m_bytesTotal = m_totalSize;
        emit progressChanged();
    } else if (const auto added = transferredSizePattern.match(trimmed); added.hasMatch()) {
        m_transferredSize = parseNumber(added.captured(1));
    }
}

void BackupRunner::emitProgress()
{
    const qint64 now = m_clock.isValid() ? m_clock.elapsed() : 0;
    if (now - m_lastProgressEmit < 100 && m_progress != 100)
        return;
    m_lastProgressEmit = now;
    emit progressChanged();
}

void BackupRunner::setState(const QString &state)
{
    if (m_state == state)
        return;
    m_state = state;
    emit stateChanged();
}

void BackupRunner::setError(const QString &message)
{
    if (m_error == message)
        return;
    m_error = message;
    emit errorMessageChanged();
}

void BackupRunner::fail(const QString &message)
{
    if (m_running) {
        cleanup(true);
    }
    setError(message);
    setState(QStringLiteral("failed"));
    qWarning() << "Aion: backup failed:" << message;
    notify(Loc::t("notify_backup_failed"), message);
    emit finished(false, m_snapshotId);
}

void BackupRunner::finishOk(int exitCode)
{
    const bool warning = exitCode == 23 || exitCode == 24;
    m_progress = 100;
    emit progressChanged();

    SnapshotInfo info;
    info.id = m_snapshotId;
    info.startedAt = m_startedAt;
    info.finishedAt = QDateTime::currentDateTime();
    info.status = warning ? QStringLiteral("warning") : QStringLiteral("ok");
    info.totalBytes = m_totalSize;
    info.totalFiles = m_totalFiles;
    info.transferredBytes = m_transferredSize;
    info.transferredFiles = m_transferredFiles;
    info.durationSeconds = m_clock.isValid() ? m_clock.elapsed() / 1000 : 0;
    info.rsyncExitCode = exitCode;
    info.error = warning ? lastLines(m_stderrTail, 3) : QString();
    info.warnings = m_warnings;
    info.hostname = QSysInfo::machineHostName();
    info.appVersion = QStringLiteral(AION_VERSION);
    info.sources = m_usedSources;
    info.excludes = m_config->excludes();

    m_store->writeMeta(info);
    m_store->setLatest(m_snapshotId);
    m_store->refresh();

    if (warning)
        m_warnings.append(Loc::t("warning_partial_transfer"));
    emit stateChanged();

    cleanup(false);
    setState(warning ? QStringLiteral("warning") : QStringLiteral("ok"));
    m_progress = 100;
    emit progressChanged();

    const QString title = Loc::t("notify_backup_done");
    const QString body = Loc::t("notify_backup_done_body")
                             .arg(Format::bytes(m_transferredSize))
                             .arg(Format::duration(info.durationSeconds));
    notify(title, body);
    emit finished(true, m_snapshotId);
}

void BackupRunner::handleFinished(int exitCode, QProcess::ExitStatus status)
{
    if (!m_running && m_state != QLatin1String("cancelled"))
        return;

    if (m_cancelled) {
        cleanup(true);
        setState(QStringLiteral("cancelled"));
        notify(Loc::t("notify_backup_cancelled"), QString());
        emit finished(false, QString());
        return;
    }

    if (status == QProcess::CrashExit || (exitCode != 0 && exitCode != 23 && exitCode != 24)) {
        QString detail = lastLines(m_stderrTail, 4);
        if (detail.isEmpty())
            detail = QStringLiteral("rsync exit code %1").arg(exitCode);
        cleanup(true);
        setError(detail);
        setState(QStringLiteral("failed"));
        qWarning() << "Aion: backup failed:" << detail;
        notify(Loc::t("notify_backup_failed"), detail);
        emit finished(false, m_snapshotId);
        return;
    }

    finishOk(exitCode);
}

void BackupRunner::cancel()
{
    if (!m_running || !m_process)
        return;
    m_cancelled = true;
    setState(QStringLiteral("cancelling"));
    m_process->terminate();
    QTimer::singleShot(5000, this, [this]() {
        if (m_process && m_process->state() != QProcess::NotRunning)
            m_process->kill();
    });
}

void BackupRunner::cleanup(bool removePartial)
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

    if (m_logFile.isOpen())
        m_logFile.close();

    if (m_lock) {
        m_lock->unlock();
        delete m_lock;
        m_lock = nullptr;
    }

    if (removePartial && !m_snapshotPath.isEmpty() && QDir(m_snapshotPath).exists()) {
        QProcess remover;
        remover.start(QStringLiteral("rm"),
                      {QStringLiteral("-rf"), QStringLiteral("--"), m_snapshotPath});
        remover.waitForFinished(300000);
    }

    if (m_running) {
        m_running = false;
        emit runningChanged();
    }
}

void BackupRunner::notify(const QString &title, const QString &body)
{
    if (!m_config->notifications() || !m_notifier)
        return;
    m_notifier->notify(title, body);
}
