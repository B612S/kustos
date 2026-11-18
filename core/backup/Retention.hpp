// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 B612S <sem.alimov@inbox.ru>

#pragma once

#include "backup/Snapshot.hpp"

#include <QDateTime>
#include <QList>
#include <QStringList>

// Time Machine-like thinning: keep everything from the last hours, then one
// snapshot per day, then one per week, and drop the rest. Pure logic, no I/O,
// so it is covered by unit tests.
namespace Retention {

struct Policy {
    int hourlyHours = 24;
    int dailyDays = 30;
    int weeklyWeeks = 12;
};

// Returns the ids that should be deleted. The newest snapshot is never
// returned. `snapshots` may be in any order.
QStringList toRemove(const QList<SnapshotInfo> &snapshots, const Policy &policy,
                     const QDateTime &now);

QStringList toKeep(const QList<SnapshotInfo> &snapshots, const Policy &policy,
                   const QDateTime &now);

} // namespace Retention
