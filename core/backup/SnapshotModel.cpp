// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 B612S <sem.alimov@inbox.ru>

#include "backup/SnapshotModel.hpp"

#include "backup/SnapshotStore.hpp"

#include <QLocale>

SnapshotModel::SnapshotModel(SnapshotStore *store, QObject *parent)
    : QAbstractListModel(parent)
    , m_store(store)
{
    connect(m_store, &SnapshotStore::changed, this, &SnapshotModel::reload);
    reload();
}

int SnapshotModel::rowCount(const QModelIndex &parent) const
{
    return parent.isValid() ? 0 : static_cast<int>(m_snapshots.size());
}

QVariant SnapshotModel::data(const QModelIndex &index, int role) const
{
    if (!index.isValid() || index.row() < 0 || index.row() >= m_snapshots.size())
        return {};

    const SnapshotInfo &snapshot = m_snapshots.at(index.row());
    const QLocale locale = QLocale::system();

    switch (role) {
    case IdRole:
        return snapshot.id;
    case StartedAtRole:
        return snapshot.startedAt;
    case DayKeyRole:
        return snapshot.startedAt.date().toString(Qt::ISODate);
    case DateTextRole:
        return locale.toString(snapshot.startedAt.date(), QLocale::ShortFormat);
    case TimeTextRole:
        return locale.toString(snapshot.startedAt.time(), QLocale::ShortFormat);
    case TotalBytesRole:
        return QVariant::fromValue(snapshot.totalBytes);
    case AddedBytesRole:
        return QVariant::fromValue(snapshot.transferredBytes);
    case FileCountRole:
        return QVariant::fromValue(snapshot.totalFiles);
    case StatusRole:
        return snapshot.status;
    case LatestRole:
        return snapshot.id == m_latestId;
    case ErrorRole:
        return snapshot.error;
    case PathRole:
        return m_store->snapshotPath(snapshot.id);
    default:
        return {};
    }
}

QHash<int, QByteArray> SnapshotModel::roleNames() const
{
    return {
        {IdRole, QByteArrayLiteral("id")},
        {StartedAtRole, QByteArrayLiteral("startedAt")},
        {DayKeyRole, QByteArrayLiteral("dayKey")},
        {DateTextRole, QByteArrayLiteral("dateText")},
        {TimeTextRole, QByteArrayLiteral("timeText")},
        {TotalBytesRole, QByteArrayLiteral("totalBytes")},
        {AddedBytesRole, QByteArrayLiteral("addedBytes")},
        {FileCountRole, QByteArrayLiteral("fileCount")},
        {StatusRole, QByteArrayLiteral("status")},
        {LatestRole, QByteArrayLiteral("isLatest")},
        {ErrorRole, QByteArrayLiteral("error")},
        {PathRole, QByteArrayLiteral("path")},
    };
}

int SnapshotModel::indexOfId(const QString &id) const
{
    for (int row = 0; row < m_snapshots.size(); ++row) {
        if (m_snapshots.at(row).id == id)
            return row;
    }
    return -1;
}

QVariantMap SnapshotModel::get(int row) const
{
    if (row < 0 || row >= m_snapshots.size())
        return {};
    return m_store->info(m_snapshots.at(row).id);
}

void SnapshotModel::reload()
{
    beginResetModel();
    m_snapshots.clear();
    const QStringList ids = m_store->ids();
    m_snapshots.reserve(ids.size());
    for (const QString &id : ids)
        m_snapshots.append(m_store->snapshot(id));
    m_latestId = m_store->latestId();
    endResetModel();
    emit countChanged();
}
