// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 B612S <sem.alimov@inbox.ru>

#include "util/Loc.hpp"

#include <QCoreApplication>

namespace Loc {

QString t(const char *key)
{
    return QCoreApplication::translate("aion", key);
}

} // namespace Loc
