// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 B612S <sem.alimov@inbox.ru>

#include "ui/Notifier.hpp"

#include <QDBusConnection>
#include <QDBusPendingCall>
#include <QDBusMessage>
#include <QVariantMap>

Notifier::Notifier(QObject *parent)
    : QObject(parent)
{
}

void Notifier::notify(const QString &title, const QString &body)
{
    if (!QDBusConnection::sessionBus().isConnected())
        return;

    QDBusMessage message = QDBusMessage::createMethodCall(
        QStringLiteral("org.freedesktop.Notifications"),
        QStringLiteral("/org/freedesktop/Notifications"),
        QStringLiteral("org.freedesktop.Notifications"),
        QStringLiteral("Notify"));

    message << QStringLiteral("Aion") << 0u << QStringLiteral("aion") << title << body
            << QStringList() << QVariantMap() << 5000;

    QDBusConnection::sessionBus().asyncCall(message);
}
