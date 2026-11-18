// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 B612S <sem.alimov@inbox.ru>

#pragma once

#include <QObject>
#include <QStringList>

#include <memory>

class QSettings;

// Persistent application settings, stored as an INI file in
// $XDG_CONFIG_HOME/aion/aion.conf (same layout as Luma's luma.conf).
//
// Every property is a thin wrapper over QSettings: the file stays editable by
// hand and there is no second source of truth in memory.
class ConfigManager : public QObject {
    Q_OBJECT
    Q_PROPERTY(QString language READ language WRITE setLanguage NOTIFY languageChanged)
    Q_PROPERTY(QString targetDir READ targetDir WRITE setTargetDir NOTIFY targetDirChanged)
    Q_PROPERTY(QStringList sources READ sources WRITE setSources NOTIFY sourcesChanged)
    Q_PROPERTY(QStringList excludes READ excludes WRITE setExcludes NOTIFY excludesChanged)
    Q_PROPERTY(int keepHourlyHours READ keepHourlyHours WRITE setKeepHourlyHours NOTIFY retentionChanged)
    Q_PROPERTY(int keepDailyDays READ keepDailyDays WRITE setKeepDailyDays NOTIFY retentionChanged)
    Q_PROPERTY(int keepWeeklyWeeks READ keepWeeklyWeeks WRITE setKeepWeeklyWeeks NOTIFY retentionChanged)
    Q_PROPERTY(bool notifications READ notifications WRITE setNotifications NOTIFY notificationsChanged)
    Q_PROPERTY(bool singleInstance READ singleInstance WRITE setSingleInstance NOTIFY singleInstanceChanged)
    Q_PROPERTY(bool scheduleEnabled READ scheduleEnabled WRITE setScheduleEnabled NOTIFY scheduleChanged)
    Q_PROPERTY(int scheduleMinutes READ scheduleMinutes WRITE setScheduleMinutes NOTIFY scheduleChanged)
    Q_PROPERTY(bool preserveAcl READ preserveAcl WRITE setPreserveAcl NOTIFY fidelityChanged)
    Q_PROPERTY(bool preserveXattr READ preserveXattr WRITE setPreserveXattr NOTIFY fidelityChanged)
    Q_PROPERTY(QString configFilePath READ configFilePath CONSTANT)
    Q_PROPERTY(QString defaultTargetDir READ defaultTargetDir CONSTANT)
    Q_PROPERTY(QString homePath READ homePath CONSTANT)

public:
    explicit ConfigManager(QObject *parent = nullptr);
    ~ConfigManager() override;

    QString language() const;
    void setLanguage(const QString &lang);
    // Resolves a stored value ("auto"/"en"/"ru") to a compiled translation.
    // "auto" follows the first two letters of systemLocale; unsupported
    // languages and locales fall back to English.
    static QString resolveLanguage(const QString &requested, const QString &systemLocale);

    QString targetDir() const;
    void setTargetDir(const QString &path);

    QStringList sources() const;
    void setSources(const QStringList &paths);

    QStringList excludes() const;
    void setExcludes(const QStringList &patterns);

    int keepHourlyHours() const;
    void setKeepHourlyHours(int hours);
    int keepDailyDays() const;
    void setKeepDailyDays(int days);
    int keepWeeklyWeeks() const;
    void setKeepWeeklyWeeks(int weeks);

    bool notifications() const;
    void setNotifications(bool enabled);

    bool singleInstance() const;
    void setSingleInstance(bool enabled);

    bool scheduleEnabled() const;
    void setScheduleEnabled(bool enabled);
    int scheduleMinutes() const;
    void setScheduleMinutes(int minutes);

    bool preserveAcl() const;
    void setPreserveAcl(bool enabled);
    bool preserveXattr() const;
    void setPreserveXattr(bool enabled);

    QString configFilePath() const;
    QString defaultTargetDir() const;
    QString homePath() const;

    Q_INVOKABLE QStringList defaultSources() const;
    Q_INVOKABLE QStringList defaultExcludes() const;
    // Wipes aion.conf and falls back to the built-in defaults.
    Q_INVOKABLE void resetToDefaults();

signals:
    void languageChanged();
    void targetDirChanged();
    void sourcesChanged();
    void excludesChanged();
    void retentionChanged();
    void notificationsChanged();
    void singleInstanceChanged();
    void scheduleChanged();
    void fidelityChanged();

private:
    std::unique_ptr<QSettings> m_settings;
};
