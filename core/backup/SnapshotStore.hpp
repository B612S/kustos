// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 B612S <sem.alimov@inbox.ru>

#pragma once

#include "backup/Retention.hpp"
#include "backup/Snapshot.hpp"

#include <QObject>
#include <QStringList>
#include <QVariantMap>

class ConfigManager;

// Owns the snapshot directory on the backup volume:
//
//   <target>/snapshots/2026-11-18_19-00-00/meta.json
//   <target>/snapshots/2026-11-18_19-00-00/home/<user>/...
//   <target>/snapshots/latest -> 2026-11-18_19-00-00
//
// It only reads and prunes; creating a snapshot is BackupRunner's job.
class SnapshotStore : public QObject {
    Q_OBJECT
    Q_PROPERTY(QString targetDir READ targetDir NOTIFY changed)
    Q_PROPERTY(QString snapshotsPath READ snapshotsPath NOTIFY changed)
    Q_PROPERTY(QStringList ids READ ids NOTIFY changed)
    Q_PROPERTY(QString latestId READ latestId NOTIFY changed)
    Q_PROPERTY(QString latestPath READ latestPath NOTIFY changed)
    Q_PROPERTY(int count READ count NOTIFY changed)
    Q_PROPERTY(qint64 availableBytes READ availableBytes NOTIFY changed)
    Q_PROPERTY(QString availableText READ availableText NOTIFY changed)
    Q_PROPERTY(QString homeRoot READ homeRoot CONSTANT)

public:
    explicit SnapshotStore(ConfigManager *config, QObject *parent = nullptr);

    QString targetDir() const;
    QString snapshotsPath() const;
    QStringList ids() const;
    QString latestId() const;
    QString latestPath() const;
    int count() const;
    qint64 availableBytes() const;
    QString availableText() const;
    // Relative path of the user's home directory inside a snapshot.
    QString homeRoot() const;

    Q_INVOKABLE QString snapshotPath(const QString &id) const;
    SnapshotInfo snapshot(const QString &id) const;
    bool writeMeta(const SnapshotInfo &info) const;
    bool setLatest(const QString &id);

    Q_INVOKABLE QVariantMap info(const QString &id) const;
    Q_INVOKABLE bool removeSnapshot(const QString &id, QString *error = nullptr);
    // Applies the retention policy from ConfigManager and returns removed ids.
    Q_INVOKABLE QStringList prune();
    Q_INVOKABLE bool isValidId(const QString &id) const;

    QString lastError() const { return m_lastError; }

public slots:
    void refresh();

signals:
    void changed();

private:
    void recomputeLatest();
    void updateAvailableBytes();

    ConfigManager *m_config = nullptr;
    QStringList m_ids;
    QString m_latestId;
    qint64 m_availableBytes = -1;
    QString m_lastError;
};
