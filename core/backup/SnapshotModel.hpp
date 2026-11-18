// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 B612S <sem.alimov@inbox.ru>

#pragma once

#include "backup/Snapshot.hpp"

#include <QAbstractListModel>

class SnapshotStore;

// QML-facing list of snapshots, newest first. Each row is one point on the
// timeline; the UI groups them by day using DayKeyRole.
class SnapshotModel : public QAbstractListModel {
    Q_OBJECT
    Q_PROPERTY(int count READ rowCount NOTIFY countChanged)

public:
    enum Roles {
        IdRole = Qt::UserRole + 1,
        StartedAtRole,
        DayKeyRole,
        DateTextRole,
        TimeTextRole,
        TotalBytesRole,
        AddedBytesRole,
        FileCountRole,
        StatusRole,
        LatestRole,
        ErrorRole,
        PathRole,
    };
    Q_ENUM(Roles)

    explicit SnapshotModel(SnapshotStore *store, QObject *parent = nullptr);

    int rowCount(const QModelIndex &parent = QModelIndex()) const override;
    QVariant data(const QModelIndex &index, int role) const override;
    QHash<int, QByteArray> roleNames() const override;

    Q_INVOKABLE int indexOfId(const QString &id) const;
    Q_INVOKABLE QVariantMap get(int row) const;

signals:
    void countChanged();

private:
    void reload();

    SnapshotStore *m_store = nullptr;
    QList<SnapshotInfo> m_snapshots;
    QString m_latestId;
};
