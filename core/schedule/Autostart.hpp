// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 B612S <sem.alimov@inbox.ru>

#pragma once

#include <QObject>
#include <QString>
#include <QStringList>

class ConfigManager;

// Installs and removes the systemd user timer that runs
// `aion --backup` in the background, so backups keep happening while the GUI
// is closed. The timer interval follows ConfigManager::scheduleMinutes().
class Autostart : public QObject {
    Q_OBJECT
    Q_PROPERTY(bool installed READ installed NOTIFY changed)
    Q_PROPERTY(QString timerState READ timerState NOTIFY changed)
    Q_PROPERTY(QString lastError READ lastError NOTIFY changed)
    Q_PROPERTY(bool busy READ busy NOTIFY changed)

public:
    explicit Autostart(ConfigManager *config, QObject *parent = nullptr);

    bool installed() const;
    QString timerState() const { return m_timerState; }
    QString lastError() const { return m_lastError; }
    bool busy() const { return m_busy; }

    // Brings the units in line with the configuration; safe to call often.
    Q_INVOKABLE void sync();
    Q_INVOKABLE void refresh();

signals:
    void changed();

private:
    QString unitsDir() const;
    bool writeUnits(int minutes);
    void removeUnits();
    bool runSystemctl(const QStringList &arguments, QString *output) const;
    void setError(const QString &error);

    ConfigManager *m_config = nullptr;
    QString m_timerState = QStringLiteral("unknown");
    QString m_lastError;
    bool m_busy = false;
};
