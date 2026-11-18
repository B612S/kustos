// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 B612S <sem.alimov@inbox.ru>

#pragma once

#include <QObject>

class ConfigManager;

// Small facade for everything QML needs from the application shell: version
// strings, opening directories, the clipboard and quitting.
class AppController : public QObject {
    Q_OBJECT
    Q_PROPERTY(QString version READ version CONSTANT)
    Q_PROPERTY(QString revision READ revision CONSTANT)
    Q_PROPERTY(QString configPath READ configPath NOTIFY pathsChanged)
    Q_PROPERTY(QString logPath READ logPath NOTIFY pathsChanged)
    Q_PROPERTY(QString homePath READ homePath CONSTANT)

public:
    explicit AppController(ConfigManager *config, QObject *parent = nullptr);

    QString version() const;
    QString revision() const;
    QString configPath() const;
    QString logPath() const;
    QString homePath() const;

    Q_INVOKABLE void openPath(const QString &path) const;
    Q_INVOKABLE void copyText(const QString &text) const;
    Q_INVOKABLE QString formatBytes(qint64 value) const;
    Q_INVOKABLE void quit();

signals:
    void pathsChanged();

private:
    ConfigManager *m_config = nullptr;
};
