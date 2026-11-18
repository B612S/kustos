// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 B612S <sem.alimov@inbox.ru>

#include <QClipboard>
#include <QCoreApplication>
#include <QGuiApplication>
#include <QString>
#include <QTranslator>

namespace {
QTranslator *g_translator = nullptr;
}

extern "C" int kustos_install_translator(const char *path)
{
    if (!g_translator)
        g_translator = new QTranslator(QCoreApplication::instance());
    QCoreApplication::removeTranslator(g_translator);
    if (g_translator->load(QString::fromUtf8(path))) {
        QCoreApplication::installTranslator(g_translator);
        return 1;
    }
    return 0;
}

extern "C" void kustos_set_clipboard(const char *text)
{
    if (auto *clipboard = QGuiApplication::clipboard())
        clipboard->setText(QString::fromUtf8(text));
}

extern "C" void kustos_quit()
{
    QCoreApplication::quit();
}

#include <QQmlApplicationEngine>

extern "C" int kustos_engine_root_count(void *engine)
{
    if (!engine)
        return -1;
    return static_cast<QQmlApplicationEngine *>(engine)->rootObjects().size();
}

#include <QGuiApplication>
#include <QQuickWindow>
#include <QTimer>

extern "C" void kustos_capture_screen(const char *path, int delay_ms)
{
    const QString target = QString::fromUtf8(path);
    QTimer::singleShot(delay_ms, [target]() {
        const auto windows = QGuiApplication::topLevelWindows();
        for (QWindow *window : windows) {
            if (auto *quick = qobject_cast<QQuickWindow *>(window)) {
                quick->grabWindow().save(target);
                break;
            }
        }
        QCoreApplication::quit();
    });
}
