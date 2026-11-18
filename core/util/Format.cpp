// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 B612S <sem.alimov@inbox.ru>

#include "util/Format.hpp"

#include <QLocale>

namespace Format {
namespace {

QString number(double value, int decimals)
{
    return QLocale::c().toString(value, 'f', decimals);
}

} // namespace

QString bytes(qint64 value)
{
    static const char *units[] = {"B", "KB", "MB", "GB", "TB", "PB"};
    double size = static_cast<double>(value);
    int unit = 0;
    while (size >= 1024.0 && unit < 5) {
        size /= 1024.0;
        ++unit;
    }
    if (unit == 0)
        return QStringLiteral("%1 B").arg(value);
    return QStringLiteral("%1 %2").arg(number(size, size < 10.0 ? 2 : 1), units[unit]);
}

QString speed(qint64 bytesPerSecond)
{
    if (bytesPerSecond <= 0)
        return QString();
    return bytes(bytesPerSecond) + QStringLiteral("/s");
}

QString duration(qint64 seconds)
{
    if (seconds < 0)
        return QString();
    if (seconds < 60)
        return QStringLiteral("%1 s").arg(seconds);
    const qint64 minutes = seconds / 60;
    if (minutes < 60)
        return QStringLiteral("%1 min %2 s").arg(minutes).arg(seconds % 60);
    const qint64 hours = minutes / 60;
    if (hours < 24)
        return QStringLiteral("%1 h %2 min").arg(hours).arg(minutes % 60);
    return QStringLiteral("%1 d %2 h").arg(hours / 24).arg(hours % 24);
}

} // namespace Format
