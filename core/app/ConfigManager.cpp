// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 B612S <sem.alimov@inbox.ru>

#include "app/ConfigManager.hpp"

#include <QDir>
#include <QFileInfo>
#include <QSettings>
#include <QStandardPaths>
#include <QStringList>

namespace {

QString settingsPath()
{
    const QString dir = QStandardPaths::writableLocation(QStandardPaths::GenericConfigLocation)
                        + QStringLiteral("/aion");
    QDir().mkpath(dir);
    return dir + QStringLiteral("/aion.conf");
}

QString normalizedPath(const QString &path)
{
    if (path.isEmpty())
        return path;
    QString cleaned = QDir::cleanPath(path);
    if (cleaned.size() > 1 && cleaned.endsWith(QLatin1Char('/')))
        cleaned.chop(1);
    return cleaned;
}

QStringList normalizedList(const QStringList &paths)
{
    QStringList out;
    out.reserve(paths.size());
    for (const QString &path : paths) {
        const QString cleaned = normalizedPath(path.trimmed());
        if (!cleaned.isEmpty() && !out.contains(cleaned))
            out.append(cleaned);
    }
    return out;
}

} // namespace

ConfigManager::ConfigManager(QObject *parent)
    : QObject(parent)
    , m_settings(std::make_unique<QSettings>(settingsPath(), QSettings::IniFormat))
{
    m_settings->setFallbacksEnabled(false);
}

ConfigManager::~ConfigManager() = default;

QString ConfigManager::language() const
{
    return m_settings->value(QStringLiteral("ui/language"), QStringLiteral("auto")).toString();
}

void ConfigManager::setLanguage(const QString &lang)
{
    if (language() == lang)
        return;
    m_settings->setValue(QStringLiteral("ui/language"), lang);
    m_settings->sync();
    emit languageChanged();
}

QString ConfigManager::resolveLanguage(const QString &requested, const QString &systemLocale)
{
    static const QStringList supported = {QStringLiteral("en"), QStringLiteral("ru")};
    if (requested != QLatin1String("auto") && supported.contains(requested))
        return requested;
    const QString code = systemLocale.left(2).toLower();
    if (supported.contains(code))
        return code;
    return QStringLiteral("en");
}

QString ConfigManager::targetDir() const
{
    const QString stored = m_settings->value(QStringLiteral("backup/targetDir")).toString();
    return stored.isEmpty() ? defaultTargetDir() : normalizedPath(stored);
}

void ConfigManager::setTargetDir(const QString &path)
{
    const QString cleaned = normalizedPath(path);
    if (targetDir() == cleaned)
        return;
    m_settings->setValue(QStringLiteral("backup/targetDir"), cleaned);
    m_settings->sync();
    emit targetDirChanged();
}

QStringList ConfigManager::sources() const
{
    const QStringList stored = m_settings->value(QStringLiteral("backup/sources")).toStringList();
    if (stored.isEmpty())
        return defaultSources();
    return normalizedList(stored);
}

void ConfigManager::setSources(const QStringList &paths)
{
    const QStringList cleaned = normalizedList(paths);
    if (sources() == cleaned)
        return;
    m_settings->setValue(QStringLiteral("backup/sources"), cleaned);
    m_settings->sync();
    emit sourcesChanged();
}

QStringList ConfigManager::excludes() const
{
    const QStringList stored = m_settings->value(QStringLiteral("backup/excludes")).toStringList();
    if (stored.isEmpty() && !m_settings->contains(QStringLiteral("backup/excludes")))
        return defaultExcludes();
    return normalizedList(stored);
}

void ConfigManager::setExcludes(const QStringList &patterns)
{
    const QStringList cleaned = normalizedList(patterns);
    if (excludes() == cleaned)
        return;
    m_settings->setValue(QStringLiteral("backup/excludes"), cleaned);
    m_settings->sync();
    emit excludesChanged();
}

int ConfigManager::keepHourlyHours() const
{
    return m_settings->value(QStringLiteral("retention/hourlyHours"), 24).toInt();
}

void ConfigManager::setKeepHourlyHours(int hours)
{
    hours = qBound(1, hours, 24 * 30);
    if (keepHourlyHours() == hours)
        return;
    m_settings->setValue(QStringLiteral("retention/hourlyHours"), hours);
    m_settings->sync();
    emit retentionChanged();
}

int ConfigManager::keepDailyDays() const
{
    return m_settings->value(QStringLiteral("retention/dailyDays"), 30).toInt();
}

void ConfigManager::setKeepDailyDays(int days)
{
    days = qBound(0, days, 3650);
    if (keepDailyDays() == days)
        return;
    m_settings->setValue(QStringLiteral("retention/dailyDays"), days);
    m_settings->sync();
    emit retentionChanged();
}

int ConfigManager::keepWeeklyWeeks() const
{
    return m_settings->value(QStringLiteral("retention/weeklyWeeks"), 12).toInt();
}

void ConfigManager::setKeepWeeklyWeeks(int weeks)
{
    weeks = qBound(0, weeks, 520);
    if (keepWeeklyWeeks() == weeks)
        return;
    m_settings->setValue(QStringLiteral("retention/weeklyWeeks"), weeks);
    m_settings->sync();
    emit retentionChanged();
}

bool ConfigManager::notifications() const
{
    return m_settings->value(QStringLiteral("ui/notifications"), true).toBool();
}

void ConfigManager::setNotifications(bool enabled)
{
    if (notifications() == enabled)
        return;
    m_settings->setValue(QStringLiteral("ui/notifications"), enabled);
    m_settings->sync();
    emit notificationsChanged();
}

bool ConfigManager::singleInstance() const
{
    return m_settings->value(QStringLiteral("ui/singleInstance"), true).toBool();
}

void ConfigManager::setSingleInstance(bool enabled)
{
    if (singleInstance() == enabled)
        return;
    m_settings->setValue(QStringLiteral("ui/singleInstance"), enabled);
    m_settings->sync();
    emit singleInstanceChanged();
}

bool ConfigManager::scheduleEnabled() const
{
    return m_settings->value(QStringLiteral("schedule/enabled"), false).toBool();
}

void ConfigManager::setScheduleEnabled(bool enabled)
{
    if (scheduleEnabled() == enabled)
        return;
    m_settings->setValue(QStringLiteral("schedule/enabled"), enabled);
    m_settings->sync();
    emit scheduleChanged();
}

int ConfigManager::scheduleMinutes() const
{
    return m_settings->value(QStringLiteral("schedule/minutes"), 60).toInt();
}

void ConfigManager::setScheduleMinutes(int minutes)
{
    minutes = qBound(5, minutes, 7 * 24 * 60);
    if (scheduleMinutes() == minutes)
        return;
    m_settings->setValue(QStringLiteral("schedule/minutes"), minutes);
    m_settings->sync();
    emit scheduleChanged();
}

bool ConfigManager::preserveAcl() const
{
    return m_settings->value(QStringLiteral("backup/preserveAcl"), true).toBool();
}

void ConfigManager::setPreserveAcl(bool enabled)
{
    if (preserveAcl() == enabled)
        return;
    m_settings->setValue(QStringLiteral("backup/preserveAcl"), enabled);
    m_settings->sync();
    emit fidelityChanged();
}

bool ConfigManager::preserveXattr() const
{
    return m_settings->value(QStringLiteral("backup/preserveXattr"), true).toBool();
}

void ConfigManager::setPreserveXattr(bool enabled)
{
    if (preserveXattr() == enabled)
        return;
    m_settings->setValue(QStringLiteral("backup/preserveXattr"), enabled);
    m_settings->sync();
    emit fidelityChanged();
}

QString ConfigManager::configFilePath() const
{
    return m_settings->fileName();
}

QString ConfigManager::defaultTargetDir() const
{
    const QFileInfo dataRoot(QStringLiteral("/data"));
    if (dataRoot.isDir() && dataRoot.isWritable())
        return QStringLiteral("/data/Backup");
    return homePath() + QStringLiteral("/Backup");
}

QString ConfigManager::homePath() const
{
    return QDir::homePath();
}

QStringList ConfigManager::defaultSources() const
{
    const QString home = homePath();
    const QStringList candidates = {
        home + QStringLiteral("/.config"),
        home + QStringLiteral("/.local"),
        home + QStringLiteral("/Music"),
        home + QStringLiteral("/Pictures"),
        home + QStringLiteral("/Videos"),
        home + QStringLiteral("/.bashrc"),
        home + QStringLiteral("/.bash_profile"),
    };
    QStringList existing;
    for (const QString &candidate : candidates) {
        if (QFileInfo::exists(candidate))
            existing.append(candidate);
    }
    return existing;
}

QStringList ConfigManager::defaultExcludes() const
{
    const QString share = homePath() + QStringLiteral("/.local/share");
    return {
        share + QStringLiteral("/Steam"),
        share + QStringLiteral("/twintaillauncher"),
        share + QStringLiteral("/Trash"),
    };
}

void ConfigManager::resetToDefaults()
{
    m_settings->clear();
    m_settings->sync();
    emit languageChanged();
    emit targetDirChanged();
    emit sourcesChanged();
    emit excludesChanged();
    emit retentionChanged();
    emit notificationsChanged();
    emit singleInstanceChanged();
    emit scheduleChanged();
    emit fidelityChanged();
}
