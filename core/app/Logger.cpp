// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 B612S <sem.alimov@inbox.ru>

#include "app/Logger.hpp"

#include <QDateTime>
#include <QDir>
#include <QFile>
#include <QFileInfo>
#include <QMutex>
#include <QTextStream>

#include <cstdio>
#include <cstdlib>

namespace {

constexpr qint64 kMaxLogSize = 5 * 1024 * 1024;

QMutex g_mutex;
QString g_path;
QtMessageHandler g_previousHandler = nullptr;

const char *levelName(QtMsgType type)
{
    switch (type) {
    case QtDebugMsg:
        return "debug";
    case QtInfoMsg:
        return "info";
    case QtWarningMsg:
        return "warning";
    case QtCriticalMsg:
        return "critical";
    case QtFatalMsg:
        return "fatal";
    }
    return "info";
}

void handler(QtMsgType type, const QMessageLogContext &context, const QString &message)
{
    {
        QMutexLocker locker(&g_mutex);
        QFile file(g_path);
        if (file.open(QIODevice::WriteOnly | QIODevice::Append | QIODevice::Text)) {
            QTextStream stream(&file);
            stream << QDateTime::currentDateTime().toString(Qt::ISODate) << " [" << levelName(type)
                   << "] " << message;
            if (context.category && *context.category)
                stream << " (" << context.category << ')';
            stream << '\n';
        }
    }
    // Keep stderr behaviour when a developer runs the binary by hand.
    if (g_previousHandler)
        g_previousHandler(type, context, message);
}

} // namespace

void Logger::install(const QString &directory)
{
    QDir().mkpath(directory);
    g_path = directory + QStringLiteral("/aion.log");

    const QFileInfo info(g_path);
    if (info.exists() && info.size() > kMaxLogSize) {
        QFile::remove(g_path + QStringLiteral(".1"));
        QFile::rename(g_path, g_path + QStringLiteral(".1"));
    }

    g_previousHandler = qInstallMessageHandler(handler);
}

QString Logger::logFilePath()
{
    return g_path;
}

QString Logger::lastError()
{
    return g_path;
}
