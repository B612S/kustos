// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 B612S <sem.alimov@inbox.ru>

#include "backup/SnapshotStore.hpp"
#include "app/ConfigManager.hpp"

#include <QDir>
#include <QFile>
#include <QJsonDocument>
#include <QJsonObject>
#include <QStandardPaths>
#include <QTemporaryDir>
#include <QTest>

class TestSnapshotStore : public QObject {
    Q_OBJECT

private slots:
    void initTestCase();
    void readsSnapshotsNewestFirst();
    void readsLatestLink();
    void removesASnapshot();
    void prunesWithTheConfiguredPolicy();

private:
    void createSnapshot(const QString &id);
    static QDateTime timeOf(const QString &id);
    QTemporaryDir m_root;
    QTemporaryDir m_configHome;
};

QDateTime TestSnapshotStore::timeOf(const QString &id)
{
    return QDateTime::fromString(id, QStringLiteral("yyyy-MM-dd_HH-mm-ss"));
}

void TestSnapshotStore::initTestCase()
{
    QVERIFY(m_root.isValid());
    QVERIFY(m_configHome.isValid());
    qputenv("XDG_CONFIG_HOME", m_configHome.path().toLocal8Bit());
}

void TestSnapshotStore::createSnapshot(const QString &id)
{
    const QString path = m_root.path() + QStringLiteral("/snapshots/") + id;
    QDir().mkpath(path + QStringLiteral("/home/tester"));
    QFile marker(path + QStringLiteral("/home/tester/file.txt"));
    QVERIFY(marker.open(QIODevice::WriteOnly));
    marker.write("payload");
    marker.close();

    SnapshotInfo info;
    info.id = id;
    info.startedAt = timeOf(id);
    info.finishedAt = info.startedAt.addSecs(5);
    info.status = QStringLiteral("ok");
    info.totalBytes = 1234;
    info.totalFiles = 3;
    info.transferredBytes = 7;
    info.transferredFiles = 1;

    QFile meta(path + QStringLiteral("/meta.json"));
    QVERIFY(meta.open(QIODevice::WriteOnly));
    meta.write(QJsonDocument(snapshotToJson(info)).toJson());
    meta.close();
}

void TestSnapshotStore::readsSnapshotsNewestFirst()
{
    ConfigManager config;
    config.setTargetDir(m_root.path());
    SnapshotStore store(&config);

    createSnapshot(QStringLiteral("2026-11-17_10-00-00"));
    createSnapshot(QStringLiteral("2026-11-18_10-00-00"));
    createSnapshot(QStringLiteral("2026-11-16_10-00-00"));
    store.refresh();

    const QStringList ids = store.ids();
    QCOMPARE(ids.size(), 3);
    QCOMPARE(ids.at(0), QStringLiteral("2026-11-18_10-00-00"));
    QCOMPARE(ids.at(2), QStringLiteral("2026-11-16_10-00-00"));

    const SnapshotInfo info = store.snapshot(ids.at(0));
    QCOMPARE(info.totalBytes, 1234);
    QCOMPARE(info.status, QStringLiteral("ok"));
    QCOMPARE(store.homeRoot(), QStringLiteral("home/") + QDir::home().dirName());
}

void TestSnapshotStore::readsLatestLink()
{
    ConfigManager config;
    config.setTargetDir(m_root.path());
    SnapshotStore store(&config);
    store.refresh();

    const QString latest = QStringLiteral("2026-11-18_10-00-00");
    QVERIFY(store.setLatest(latest));
    store.refresh();
    QCOMPARE(store.latestId(), latest);
    QCOMPARE(store.latestPath(), m_root.path() + QStringLiteral("/snapshots/latest"));
}

void TestSnapshotStore::removesASnapshot()
{
    ConfigManager config;
    config.setTargetDir(m_root.path());
    SnapshotStore store(&config);
    store.refresh();

    QVERIFY(store.removeSnapshot(QStringLiteral("2026-11-17_10-00-00")));
    store.refresh();
    QVERIFY(!store.ids().contains(QStringLiteral("2026-11-17_10-00-00")));
    QVERIFY(!store.removeSnapshot(QStringLiteral("../evil")));
}

void TestSnapshotStore::prunesWithTheConfiguredPolicy()
{
    ConfigManager config;
    config.setTargetDir(m_root.path());
    config.setKeepHourlyHours(1);
    config.setKeepDailyDays(0);
    config.setKeepWeeklyWeeks(0);
    SnapshotStore store(&config);

    // A month-old snapshot is outside every window and must go; the newest
    // snapshot always survives.
    createSnapshot(QStringLiteral("2026-10-01_10-00-00"));
    store.refresh();
    QVERIFY(store.ids().contains(QStringLiteral("2026-10-01_10-00-00")));

    const QStringList removed = store.prune();
    store.refresh();
    QVERIFY(!store.ids().contains(QStringLiteral("2026-10-01_10-00-00")));
    QVERIFY(removed.contains(QStringLiteral("2026-10-01_10-00-00")));
    QVERIFY(store.ids().size() >= 1);
}

QTEST_GUILESS_MAIN(TestSnapshotStore)
#include "tst_snapshotstore.moc"
