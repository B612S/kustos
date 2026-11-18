// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 B612S <sem.alimov@inbox.ru>

#include "app/ConfigManager.hpp"

#include <QDir>
#include <QSettings>
#include <QStandardPaths>
#include <QTemporaryDir>
#include <QTest>

class TestConfigManager : public QObject {
    Q_OBJECT

private slots:
    void initTestCase();
    void resolvesLanguage();
    void defaultsAreUsable();
    void persistsValues();
    void resetRestoresDefaults();

private:
    QTemporaryDir m_configHome;
};

void TestConfigManager::initTestCase()
{
    QVERIFY(m_configHome.isValid());
    qputenv("XDG_CONFIG_HOME", m_configHome.path().toLocal8Bit());
}

void TestConfigManager::resolvesLanguage()
{
    QCOMPARE(ConfigManager::resolveLanguage(QStringLiteral("auto"), QStringLiteral("ru_RU.UTF-8")),
             QStringLiteral("ru"));
    QCOMPARE(ConfigManager::resolveLanguage(QStringLiteral("auto"), QStringLiteral("de_DE")),
             QStringLiteral("en"));
    QCOMPARE(ConfigManager::resolveLanguage(QStringLiteral("ru"), QStringLiteral("en_US")),
             QStringLiteral("ru"));
    QCOMPARE(ConfigManager::resolveLanguage(QStringLiteral("ja"), QStringLiteral("en_US")),
             QStringLiteral("en"));
}

void TestConfigManager::defaultsAreUsable()
{
    ConfigManager config;
    QVERIFY(!config.targetDir().isEmpty());
    QVERIFY(config.targetDir().startsWith(QLatin1Char('/')));
    QVERIFY(!config.sources().isEmpty());
    QVERIFY(!config.excludes().isEmpty());

    bool hasSteamExclude = false;
    for (const QString &exclude : config.excludes()) {
        if (exclude.contains(QStringLiteral("Steam")))
            hasSteamExclude = true;
    }
    QVERIFY(hasSteamExclude);

    QCOMPARE(config.keepHourlyHours(), 24);
    QCOMPARE(config.keepDailyDays(), 30);
    QCOMPARE(config.scheduleMinutes(), 60);
}

void TestConfigManager::persistsValues()
{
    {
        ConfigManager config;
        config.setTargetDir(QStringLiteral("/tmp/aion-test-target"));
        config.setLanguage(QStringLiteral("ru"));
        config.setKeepDailyDays(7);
    }

    ConfigManager reopened;
    QCOMPARE(reopened.targetDir(), QStringLiteral("/tmp/aion-test-target"));
    QCOMPARE(reopened.language(), QStringLiteral("ru"));
    QCOMPARE(reopened.keepDailyDays(), 7);
}

void TestConfigManager::resetRestoresDefaults()
{
    ConfigManager config;
    config.setLanguage(QStringLiteral("en"));
    config.resetToDefaults();
    QCOMPARE(config.language(), QStringLiteral("auto"));
    QCOMPARE(config.keepDailyDays(), 30);
}

QTEST_GUILESS_MAIN(TestConfigManager)
#include "tst_configmanager.moc"
