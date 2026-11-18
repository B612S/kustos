// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 B612S <sem.alimov@inbox.ru>

#include "schedule/Autostart.hpp"

#include "app/ConfigManager.hpp"

#include <QCoreApplication>
#include <QDir>
#include <QFile>
#include <QFileInfo>
#include <QProcess>
#include <QStandardPaths>
#include <QTextStream>
#include <QTimer>

namespace {

const char *kServiceName = "aion-backup.service";
const char *kTimerName = "aion-backup.timer";

} // namespace

Autostart::Autostart(ConfigManager *config, QObject *parent)
    : QObject(parent)
    , m_config(config)
{
    connect(m_config, &ConfigManager::scheduleChanged, this, &Autostart::sync);
    refresh();
    // The timer must match the configuration at startup too: a config edited
    // by hand or a unit removed outside the app is repaired here.
    QTimer::singleShot(0, this, [this]() {
        if (m_config->scheduleEnabled())
            sync();
    });
}

QString Autostart::unitsDir() const
{
    return QStandardPaths::writableLocation(QStandardPaths::GenericConfigLocation)
           + QStringLiteral("/systemd/user");
}

bool Autostart::installed() const
{
    return QFileInfo::exists(unitsDir() + QLatin1Char('/') + QLatin1String(kServiceName))
           && QFileInfo::exists(unitsDir() + QLatin1Char('/') + QLatin1String(kTimerName));
}

bool Autostart::runSystemctl(const QStringList &arguments, QString *output) const
{
    if (qEnvironmentVariableIsEmpty("DBUS_SESSION_BUS_ADDRESS")
        && qEnvironmentVariableIsEmpty("XDG_RUNTIME_DIR")) {
        if (output)
            *output = QStringLiteral("no user session bus");
        return false;
    }

    QProcess process;
    process.start(QStringLiteral("systemctl"), QStringList{QStringLiteral("--user")} + arguments);
    if (!process.waitForFinished(20000)) {
        process.kill();
        if (output)
            *output = QStringLiteral("systemctl timed out");
        return false;
    }
    if (output)
        *output = QString::fromLocal8Bit(process.readAllStandardError()).trimmed();
    return process.exitCode() == 0;
}

bool Autostart::writeUnits(int minutes)
{
    QDir().mkpath(unitsDir());

    const QString executable = QCoreApplication::applicationFilePath();
    const QString servicePath = unitsDir() + QLatin1Char('/') + QLatin1String(kServiceName);
    const QString timerPath = unitsDir() + QLatin1Char('/') + QLatin1String(kTimerName);

    QFile service(servicePath);
    if (!service.open(QIODevice::WriteOnly | QIODevice::Truncate | QIODevice::Text)) {
        setError(QStringLiteral("cannot write %1").arg(servicePath));
        return false;
    }
    QTextStream serviceStream(&service);
    serviceStream << "[Unit]\n"
                  << "Description=Aion automatic backup\n"
                  << "Documentation=man:aion(1)\n"
                  << "\n"
                  << "[Service]\n"
                  << "Type=oneshot\n"
                  << "ExecStart=\"" << executable << "\" --backup --quiet\n"
                  << "Nice=10\n"
                  << "IOSchedulingClass=idle\n";
    service.close();

    QFile timer(timerPath);
    if (!timer.open(QIODevice::WriteOnly | QIODevice::Truncate | QIODevice::Text)) {
        setError(QStringLiteral("cannot write %1").arg(timerPath));
        return false;
    }
    QTextStream timerStream(&timer);
    timerStream << "[Unit]\n"
                << "Description=Aion backup every " << minutes << " minutes\n"
                << "\n"
                << "[Timer]\n"
                << "OnActiveSec=5min\n"
                << "OnUnitActiveSec=" << minutes << "min\n"
                << "AccuracySec=1min\n"
                << "Persistent=true\n"
                << "\n"
                << "[Install]\n"
                << "WantedBy=timers.target\n";
    timer.close();
    return true;
}

void Autostart::removeUnits()
{
    QFile::remove(unitsDir() + QLatin1Char('/') + QLatin1String(kServiceName));
    QFile::remove(unitsDir() + QLatin1Char('/') + QLatin1String(kTimerName));
}

void Autostart::setError(const QString &error)
{
    if (m_lastError == error)
        return;
    m_lastError = error;
    emit changed();
}

void Autostart::sync()
{
    if (m_busy)
        return;
    m_busy = true;
    emit changed();

    const bool wanted = m_config->scheduleEnabled();
    const int minutes = m_config->scheduleMinutes();

    QString output;
    if (wanted) {
        if (!writeUnits(minutes) || !runSystemctl({QStringLiteral("daemon-reload")}, &output)) {
            setError(output.isEmpty() ? QStringLiteral("failed to install the timer") : output);
        } else if (!runSystemctl({QStringLiteral("enable"), QStringLiteral("--now"),
                                  QStringLiteral("aion-backup.timer")},
                                 &output)) {
            setError(output.isEmpty() ? QStringLiteral("failed to enable the timer") : output);
        } else {
            m_lastError.clear();
        }
    } else {
        runSystemctl({QStringLiteral("disable"), QStringLiteral("--now"),
                      QStringLiteral("aion-backup.timer")},
                     &output);
        removeUnits();
        runSystemctl({QStringLiteral("daemon-reload")}, &output);
        m_lastError.clear();
    }

    m_busy = false;
    emit changed();
    refresh();
}

void Autostart::refresh()
{
    QString output;
    QString state = QStringLiteral("absent");
    if (installed()) {
        state = QStringLiteral("inactive");
        if (runSystemctl({QStringLiteral("is-active"), QStringLiteral("aion-backup.timer")}, &output))
            state = QStringLiteral("active");
    }
    if (state != m_timerState) {
        m_timerState = state;
        emit changed();
    }
}
