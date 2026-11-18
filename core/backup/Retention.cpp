// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 B612S <sem.alimov@inbox.ru>

#include "backup/Retention.hpp"

#include <QSet>

#include <algorithm>

namespace Retention {
namespace {

// "2026-11" for the ISO week of the date: the key of the weekly bucket.
QString weekKey(const QDateTime &when)
{
    const QDate date = when.date();
    int year = date.year();
    const int week = date.weekNumber(&year);
    return QStringLiteral("%1-%2").arg(year).arg(week, 2, 10, QLatin1Char('0'));
}

QString dayKey(const QDateTime &when)
{
    return when.date().toString(Qt::ISODate);
}

QList<SnapshotInfo> sortedNewestFirst(QList<SnapshotInfo> snapshots)
{
    std::sort(snapshots.begin(), snapshots.end(), [](const SnapshotInfo &a, const SnapshotInfo &b) {
        if (a.startedAt == b.startedAt)
            return a.id > b.id;
        return a.startedAt > b.startedAt;
    });
    return snapshots;
}

} // namespace

QStringList toKeep(const QList<SnapshotInfo> &snapshots, const Policy &policy, const QDateTime &now)
{
    const QList<SnapshotInfo> ordered = sortedNewestFirst(snapshots);
    if (ordered.isEmpty())
        return {};

    QStringList keep;
    keep.reserve(ordered.size());

    const qint64 hourlySeconds = static_cast<qint64>(policy.hourlyHours) * 3600;
    const qint64 dailySeconds = static_cast<qint64>(policy.dailyDays) * 86400;
    const qint64 weeklySeconds = static_cast<qint64>(policy.weeklyWeeks) * 7 * 86400;

    QSet<QString> seenDays;
    QSet<QString> seenWeeks;

    for (int i = 0; i < ordered.size(); ++i) {
        const SnapshotInfo &snapshot = ordered.at(i);
        // The newest snapshot is always kept: never leave the user without one.
        if (i == 0) {
            keep.append(snapshot.id);
            seenDays.insert(dayKey(snapshot.startedAt));
            seenWeeks.insert(weekKey(snapshot.startedAt));
            continue;
        }

        const qint64 age = snapshot.startedAt.secsTo(now);
        if (age <= hourlySeconds) {
            keep.append(snapshot.id);
            continue;
        }
        if (age <= dailySeconds) {
            const QString day = dayKey(snapshot.startedAt);
            if (!seenDays.contains(day)) {
                seenDays.insert(day);
                keep.append(snapshot.id);
            }
            continue;
        }
        if (age <= dailySeconds + weeklySeconds) {
            const QString week = weekKey(snapshot.startedAt);
            if (!seenWeeks.contains(week)) {
                seenWeeks.insert(week);
                keep.append(snapshot.id);
            }
            continue;
        }
        // Older than every configured window: dropped.
    }

    return keep;
}

QStringList toRemove(const QList<SnapshotInfo> &snapshots, const Policy &policy,
                     const QDateTime &now)
{
    const QStringList keep = toKeep(snapshots, policy, now);
    QStringList remove;
    for (const SnapshotInfo &snapshot : snapshots) {
        if (!keep.contains(snapshot.id))
            remove.append(snapshot.id);
    }
    std::sort(remove.begin(), remove.end());
    return remove;
}

} // namespace Retention
