// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 B612S <sem.alimov@inbox.ru>

#pragma once

#include <QString>

// Minimal file logger: every qDebug/qWarning/qCritical line goes to
// $XDG_CACHE_HOME/aion/aion.log, so a failed backup can be diagnosed after
// the fact. The file is rotated at startup once it grows past 5 MB.
class Logger {
public:
    static void install(const QString &directory);
    static QString logFilePath();
    static QString lastError();
};
