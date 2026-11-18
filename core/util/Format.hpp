// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 B612S <sem.alimov@inbox.ru>

#pragma once

#include <QString>
#include <QtGlobal>

// Human-readable formatting shared by the C++ models and QML (through
// AppController). Sizes use binary units, like Finder and Time Machine.
namespace Format {

QString bytes(qint64 value);
QString speed(qint64 bytesPerSecond);
QString duration(qint64 seconds);

} // namespace Format
