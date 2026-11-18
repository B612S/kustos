// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 B612S <sem.alimov@inbox.ru>

#pragma once

#include <QDateTime>
#include <QElapsedTimer>
#include <QFile>
#include <QObject>
#include <QProcess>
#include <QStringList>

class ConfigManager;
class SnapshotStore;
class Notifier;
class QLockFile;

// Creates one snapshot, Time Machine style:
//
//   rsync -aHAX --relative --delete --link-dest=<previous snapshot> \
//         --info=progress2 --stats -- <sources...> <snapshots>/<timestamp>/
//
// Files that did not change since the previous snapshot are hard links, so a
// new snapshot costs only the changed bytes. The previous snapshot is never
// modified; a failed run deletes its own partial directory.
class BackupRunner : public QObject {
    Q_OBJECT
    Q_PROPERTY(bool running READ running NOTIFY runningChanged)
    Q_PROPERTY(int progress READ progress NOTIFY progressChanged)
    Q_PROPERTY(QString state READ state NOTIFY stateChanged)
    Q_PROPERTY(QString speedText READ speedText NOTIFY progressChanged)
    Q_PROPERTY(QString etaText READ etaText NOTIFY progressChanged)
    Q_PROPERTY(qint64 bytesDone READ bytesDone NOTIFY progressChanged)
    Q_PROPERTY(qint64 bytesTotal READ bytesTotal NOTIFY progressChanged)
    Q_PROPERTY(QString errorMessage READ errorMessage NOTIFY errorMessageChanged)
    Q_PROPERTY(QString currentSnapshotId READ currentSnapshotId NOTIFY currentSnapshotIdChanged)
    Q_PROPERTY(QString logPath READ logPath NOTIFY logPathChanged)
    Q_PROPERTY(QStringList warnings READ warnings NOTIFY stateChanged)

public:
    BackupRunner(ConfigManager *config, SnapshotStore *store, Notifier *notifier,
                 QObject *parent = nullptr);

    Q_INVOKABLE bool start();
    Q_INVOKABLE void cancel();

    bool running() const { return m_running; }
    int progress() const { return m_progress; }
    QString state() const { return m_state; }
    QString speedText() const { return m_speed; }
    QString etaText() const { return m_eta; }
    qint64 bytesDone() const { return m_bytesDone; }
    qint64 bytesTotal() const { return m_bytesTotal; }
    QString errorMessage() const { return m_error; }
    QString currentSnapshotId() const { return m_snapshotId; }
    QString logPath() const { return m_logPath; }
    QStringList warnings() const { return m_warnings; }

signals:
    void runningChanged();
    void progressChanged();
    void stateChanged();
    void errorMessageChanged();
    void currentSnapshotIdChanged();
    void logPathChanged();
    void finished(bool ok, const QString &snapshotId);

private:
    bool prepare(QString *error);
    void launch();
    void consume(const QByteArray &chunk);
    void consumeErrors(const QByteArray &chunk);
    void handleLine(const QString &line);
    void handleFinished(int exitCode, QProcess::ExitStatus status);
    void fail(const QString &message);
    void finishOk(int exitCode);
    void cleanup(bool removePartial);
    void setState(const QString &state);
    void setError(const QString &message);
    void emitProgress();
    void notify(const QString &title, const QString &body);
    QString uniqueId(const QString &base) const;

    ConfigManager *m_config = nullptr;
    SnapshotStore *m_store = nullptr;
    Notifier *m_notifier = nullptr;

    QProcess *m_process = nullptr;
    QLockFile *m_lock = nullptr;
    QFile m_logFile;
    QString m_logPath;
    QString m_pending;
    QString m_stderrTail;

    bool m_running = false;
    bool m_cancelled = false;
    int m_progress = -1;
    QString m_state = QStringLiteral("idle");
    QString m_speed;
    QString m_eta;
    QString m_error;
    QStringList m_warnings;

    qint64 m_bytesDone = 0;
    qint64 m_bytesTotal = 0;
    qint64 m_totalSize = 0;
    qint64 m_totalFiles = 0;
    qint64 m_transferredSize = 0;
    qint64 m_transferredFiles = 0;

    QString m_snapshotId;
    QString m_snapshotPath;
    QString m_targetDir;
    QStringList m_usedSources;
    QDateTime m_startedAt;
    QElapsedTimer m_clock;
    qint64 m_lastProgressEmit = 0;
};
