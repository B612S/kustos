// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 B612S <sem.alimov@inbox.ru>

#include "backup/Retention.hpp"

#include <QTest>
#include <QTimeZone>

// The retention policy is pure logic: give it snapshots and "now", get the ids
// to delete. These tests pin down the Time Machine buckets.
class TestRetention : public QObject {
    Q_OBJECT

private slots:
    void keepsEverythingInsideTheHourlyWindow();
    void keepsTheNewestSnapshotOfEachDay();
    void keepsTheNewestSnapshotOfEachWeek();
    void dropsOlderThanEveryWindow();
    void neverRemovesTheNewestSnapshot();
    void ignoresEmptyInput();

private:
    static SnapshotInfo snapshot(const QDateTime &when);
    static QDateTime now() { return QDateTime(QDate(2026, 11, 18), QTime(19, 0), QTimeZone::UTC); }
};

SnapshotInfo TestRetention::snapshot(const QDateTime &when)
{
    SnapshotInfo info;
    info.id = when.toString(QStringLiteral("yyyy-MM-dd_HH-mm-ss"));
    info.startedAt = when;
    return info;
}

void TestRetention::keepsEverythingInsideTheHourlyWindow()
{
    Retention::Policy policy;
    policy.hourlyHours = 24;

    QList<SnapshotInfo> snapshots;
    for (int hour = 0; hour < 6; ++hour)
        snapshots.append(snapshot(now().addSecs(-hour * 3600)));

    QCOMPARE(Retention::toRemove(snapshots, policy, now()), QStringList());
}

void TestRetention::keepsTheNewestSnapshotOfEachDay()
{
    Retention::Policy policy;
    policy.hourlyHours = 24;
    policy.dailyDays = 30;

    const QDateTime twoDaysAgo = now().addDays(-2);
    const QList<SnapshotInfo> snapshots = {
        snapshot(now().addSecs(-3600)),                 // inside the hourly window
        snapshot(twoDaysAgo),                           // 09:00, newest of that day
        snapshot(twoDaysAgo.addSecs(-2 * 3600)),        // same day, older
        snapshot(twoDaysAgo.addSecs(-3600)),            // same day, older
    };

    QCOMPARE(snapshots.size(), 4);
    const QStringList removed = Retention::toRemove(snapshots, policy, now());
    QCOMPARE(removed.size(), 2);
    QVERIFY(!removed.contains(snapshots.at(1).id));
}

void TestRetention::keepsTheNewestSnapshotOfEachWeek()
{
    Retention::Policy policy;
    policy.hourlyHours = 24;
    policy.dailyDays = 1;
    policy.weeklyWeeks = 12;

    // ~40 days ago: past the daily window, inside the weekly one.
    const QDateTime base = now().addDays(-40);
    const QList<SnapshotInfo> snapshots = {
        snapshot(now().addSecs(-3600)),      // hourly window
        snapshot(base),                      // newest of its week
        snapshot(base.addDays(-1)),          // same week: dropped
        snapshot(base.addDays(-7)),          // previous week: kept
        snapshot(base.addDays(-8)),          // that week is taken: dropped
    };

    const QStringList removed = Retention::toRemove(snapshots, policy, now());
    // One snapshot per week survives; the duplicates inside each week go.
    QCOMPARE(removed.size(), 2);
    QVERIFY(removed.contains(snapshots.at(2).id));
    QVERIFY(removed.contains(snapshots.at(4).id));
}

void TestRetention::dropsOlderThanEveryWindow()
{
    Retention::Policy policy;
    policy.hourlyHours = 24;
    policy.dailyDays = 30;
    policy.weeklyWeeks = 12;

    const QList<SnapshotInfo> snapshots = {
        snapshot(now().addSecs(-60)),
        snapshot(now().addDays(-400)),
    };

    const QStringList removed = Retention::toRemove(snapshots, policy, now());
    QCOMPARE(removed, QStringList{snapshots.at(1).id});
    QCOMPARE(Retention::toKeep(snapshots, policy, now()).size(), 1);
}

void TestRetention::neverRemovesTheNewestSnapshot()
{
    Retention::Policy policy;
    policy.hourlyHours = 1;
    policy.dailyDays = 0;
    policy.weeklyWeeks = 0;

    // Even a very old snapshot survives when it is the only one.
    const QList<SnapshotInfo> snapshots = {snapshot(now().addDays(-900))};
    QCOMPARE(Retention::toRemove(snapshots, policy, now()), QStringList());
}

void TestRetention::ignoresEmptyInput()
{
    Retention::Policy policy;
    QCOMPARE(Retention::toRemove({}, policy, now()), QStringList());
    QCOMPARE(Retention::toKeep({}, policy, now()), QStringList());
}

QTEST_GUILESS_MAIN(TestRetention)
#include "tst_retention.moc"
