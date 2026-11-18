// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 B612S <sem.alimov@inbox.ru>

#include "backup/Snapshot.hpp"

#include <QJsonArray>

namespace {

QJsonArray toArray(const QStringList &values)
{
    QJsonArray array;
    for (const QString &value : values)
        array.append(value);
    return array;
}

QStringList fromArray(const QJsonValue &value)
{
    QStringList out;
    const QJsonArray array = value.toArray();
    out.reserve(array.size());
    for (const QJsonValue &entry : array)
        out.append(entry.toString());
    return out;
}

} // namespace

QJsonObject snapshotToJson(const SnapshotInfo &info)
{
    QJsonObject object;
    object[QStringLiteral("id")] = info.id;
    object[QStringLiteral("started_at")] = info.startedAt.toString(Qt::ISODate);
    object[QStringLiteral("finished_at")] = info.finishedAt.toString(Qt::ISODate);
    object[QStringLiteral("status")] = info.status;
    object[QStringLiteral("total_bytes")] = static_cast<double>(info.totalBytes);
    object[QStringLiteral("total_files")] = static_cast<double>(info.totalFiles);
    object[QStringLiteral("transferred_bytes")] = static_cast<double>(info.transferredBytes);
    object[QStringLiteral("transferred_files")] = static_cast<double>(info.transferredFiles);
    object[QStringLiteral("duration_seconds")] = static_cast<double>(info.durationSeconds);
    object[QStringLiteral("rsync_exit")] = info.rsyncExitCode;
    object[QStringLiteral("error")] = info.error;
    object[QStringLiteral("warnings")] = toArray(info.warnings);
    object[QStringLiteral("hostname")] = info.hostname;
    object[QStringLiteral("app_version")] = info.appVersion;
    object[QStringLiteral("sources")] = toArray(info.sources);
    object[QStringLiteral("excludes")] = toArray(info.excludes);
    return object;
}

SnapshotInfo snapshotFromJson(const QJsonObject &object)
{
    SnapshotInfo info;
    info.id = object.value(QStringLiteral("id")).toString();
    info.startedAt = QDateTime::fromString(object.value(QStringLiteral("started_at")).toString(),
                                           Qt::ISODate);
    info.finishedAt = QDateTime::fromString(object.value(QStringLiteral("finished_at")).toString(),
                                            Qt::ISODate);
    info.status = object.value(QStringLiteral("status")).toString(QStringLiteral("ok"));
    info.totalBytes = static_cast<qint64>(object.value(QStringLiteral("total_bytes")).toDouble());
    info.totalFiles = static_cast<qint64>(object.value(QStringLiteral("total_files")).toDouble());
    info.transferredBytes =
        static_cast<qint64>(object.value(QStringLiteral("transferred_bytes")).toDouble());
    info.transferredFiles =
        static_cast<qint64>(object.value(QStringLiteral("transferred_files")).toDouble());
    info.durationSeconds =
        static_cast<qint64>(object.value(QStringLiteral("duration_seconds")).toDouble());
    info.rsyncExitCode = object.value(QStringLiteral("rsync_exit")).toInt();
    info.error = object.value(QStringLiteral("error")).toString();
    info.warnings = fromArray(object.value(QStringLiteral("warnings")));
    info.hostname = object.value(QStringLiteral("hostname")).toString();
    info.appVersion = object.value(QStringLiteral("app_version")).toString();
    info.sources = fromArray(object.value(QStringLiteral("sources")));
    info.excludes = fromArray(object.value(QStringLiteral("excludes")));
    return info;
}
