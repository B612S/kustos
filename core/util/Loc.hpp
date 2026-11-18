// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 B612S <sem.alimov@inbox.ru>

#pragma once

#include <QString>

// Translations for strings that cannot go through qsTr() because they are
// produced outside QML (notifications, CLI output, error messages).
// The source of truth is locales/*.ts, context "aion".
namespace Loc {

QString t(const char *key);

} // namespace Loc
