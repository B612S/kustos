// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 B612S <sem.alimov@inbox.ru>

#pragma once

#include <QObject>

// Desktop notifications over org.freedesktop.Notifications. Failures are
// ignored on purpose: a missing notification daemon must never break a backup.
class Notifier : public QObject {
    Q_OBJECT

public:
    explicit Notifier(QObject *parent = nullptr);

public slots:
    void notify(const QString &title, const QString &body);
};
