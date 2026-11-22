// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 B612S <sem.alimov@inbox.ru>

#include <QClipboard>
#include <QCoreApplication>
#include <QDateTime>
#include <QEvent>
#include <QGuiApplication>
#include <QString>
#include <QTranslator>

#include <atomic>

namespace {
QTranslator *g_translator = nullptr;
std::atomic<qint64> g_last_activity{0};

// The vault is locked after a period without any user activity. Watching the
// application-wide event stream keeps the timeout independent of which QML
// item has focus.
class KustosActivityFilter : public QObject
{
public:
    bool eventFilter(QObject *object, QEvent *event) override
    {
        switch (event->type()) {
        case QEvent::MouseButtonPress:
        case QEvent::MouseButtonRelease:
        case QEvent::MouseButtonDblClick:
        case QEvent::MouseMove:
        case QEvent::Wheel:
        case QEvent::KeyPress:
        case QEvent::KeyRelease:
        case QEvent::TouchBegin:
        case QEvent::TouchUpdate:
        case QEvent::TouchEnd:
        case QEvent::FocusIn:
        case QEvent::WindowActivate:
        case QEvent::ApplicationActivate:
            g_last_activity.store(QDateTime::currentMSecsSinceEpoch());
            break;
        default:
            break;
        }
        return QObject::eventFilter(object, event);
    }
};
} // namespace

extern "C" void kustos_install_activity_filter()
{
    static KustosActivityFilter *filter = nullptr;
    if (!filter && QCoreApplication::instance()) {
        filter = new KustosActivityFilter();
        QCoreApplication::instance()->installEventFilter(filter);
        g_last_activity.store(QDateTime::currentMSecsSinceEpoch());
    }
}

extern "C" qint64 kustos_last_activity_ms()
{
    return g_last_activity.load();
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
