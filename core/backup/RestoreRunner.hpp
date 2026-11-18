// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 B612S <sem.alimov@inbox.ru>

#pragma once

#include <QObject>
#include <QProcess>
#include <QStringList>

class SnapshotStore;
class Notifier;

// Copies selected items out of a snapshot back to the filesystem. Items keep
// their path relative to the snapshot root, so restoring into "/" puts every
// file back where it came from, and restoring into a folder produces the same
// tree under that folder (a safe place to inspect before overwriting).
class RestoreRunner : public QObject {
    Q_OBJECT
    Q_PROPERTY(bool running READ running NOTIFY runningChanged)
    Q_PROPERTY(int progress READ progress NOTIFY progressChanged)
    Q_PROPERTY(QString currentItem READ currentItem NOTIFY progressChanged)
    Q_PROPERTY(QString state READ state NOTIFY stateChanged)
    Q_PROPERTY(int restoredCount READ restoredCount NOTIFY progressChanged)
    Q_PROPERTY(QString errorMessage READ errorMessage NOTIFY errorMessageChanged)
    Q_PROPERTY(QString destination READ destination NOTIFY progressChanged)

public:
    RestoreRunner(SnapshotStore *store, Notifier *notifier, QObject *parent = nullptr);

    // policy: "overwrite" | "skip" | "keep_both"
    Q_INVOKABLE bool start(const QString &snapshotId, const QStringList &relativePaths,
                           const QString &destinationDir, const QString &policy);
    Q_INVOKABLE void cancel();

    bool running() const { return m_running; }
    int progress() const { return m_progress; }
    QString currentItem() const { return m_currentItem; }
    QString state() const { return m_state; }
    int restoredCount() const { return m_restoredCount; }
    QString errorMessage() const { return m_error; }
    QString destination() const { return m_destination; }

signals:
    void runningChanged();
    void progressChanged();
    void stateChanged();
    void errorMessageChanged();
    void finished(bool ok, int restoredCount);

private:
    void runNext();
    void handleItemFinished(int exitCode, QProcess::ExitStatus status);
    void finishOk();
    void fail(const QString &message);
    void cleanup();
    void setState(const QString &state);
    QString uniqueTarget(const QString &target) const;

    SnapshotStore *m_store = nullptr;
    Notifier *m_notifier = nullptr;
    QProcess *m_process = nullptr;

    QString m_snapshotId;
    QString m_destination;
    QString m_policy;
    QStringList m_items;
    int m_index = 0;
    int m_restoredCount = 0;
    int m_progress = 0;
    int m_itemPercent = 0;
    QString m_currentItem;
    QString m_state = QStringLiteral("idle");
    QString m_error;
    QString m_itemError;
    bool m_running = false;
    bool m_cancelled = false;
};
