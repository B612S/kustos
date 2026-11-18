// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 B612S <sem.alimov@inbox.ru>

#include "ui/AppController.hpp"

#include "app/ConfigManager.hpp"
#include "app/Logger.hpp"
#include "util/Format.hpp"

#include <QClipboard>
#include <QCoreApplication>
#include <QDesktopServices>
#include <QDir>
#include <QFileInfo>
#include <QGuiApplication>
#include <QUrl>

AppController::AppController(ConfigManager *config, QObject *parent)
    : QObject(parent)
    , m_config(config)
{
    connect(m_config, &ConfigManager::targetDirChanged, this, &AppController::pathsChanged);
}

QString AppController::version() const
{
    QString version = QStringLiteral(AION_VERSION);
    const QString revision = QStringLiteral(AION_GIT_HASH);
    if (!revision.isEmpty())
        version += QLatin1Char('-') + revision;
    return version;
}

QString AppController::revision() const
{
    return QStringLiteral(AION_GIT_HASH);
}

QString AppController::configPath() const
{
    return m_config->configFilePath();
}

QString AppController::logPath() const
{
    return Logger::logFilePath();
}

QString AppController::homePath() const
{
    return QDir::homePath();
}

void AppController::openPath(const QString &path) const
{
    QFileInfo info(path);
    const QString directory = info.isDir() ? info.absoluteFilePath() : info.absolutePath();
    QDesktopServices::openUrl(QUrl::fromLocalFile(directory));
}

void AppController::copyText(const QString &text) const
{
    if (auto *clipboard = QGuiApplication::clipboard())
        clipboard->setText(text);
}

QString AppController::formatBytes(qint64 value) const
{
    return Format::bytes(value);
}

void AppController::quit()
{
    QCoreApplication::quit();
}
