// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 B612S <sem.alimov@inbox.ru>

#include "app/ConfigManager.hpp"
#include "app/Logger.hpp"
#include "backup/BackupRunner.hpp"
#include "backup/DirectoryModel.hpp"
#include "backup/RestoreRunner.hpp"
#include "backup/SnapshotModel.hpp"
#include "backup/SnapshotStore.hpp"
#include "schedule/Autostart.hpp"
#include "ui/AppController.hpp"
#include "ui/Notifier.hpp"
#include "util/Format.hpp"
#include "util/Loc.hpp"

#include <QCommandLineOption>
#include <QCommandLineParser>
#include <QCoreApplication>
#include <QDir>
#include <QFileInfo>
#include <QIcon>
#include <QLocalServer>
#include <QLocalSocket>
#include <QLocale>
#include <QQuickWindow>
#include <QQmlApplicationEngine>
#include <QQmlContext>
#include <QStandardPaths>
#include <QTextStream>
#include <QTimer>
#include <QTranslator>
#include <QUrl>

#include <cstdio>
#include <memory>

namespace {

QString cacheDirectory()
{
    const QString base = QStandardPaths::writableLocation(QStandardPaths::GenericCacheLocation);
    return base + QStringLiteral("/aion");
}

QStringList argumentsOf(int argc, char *argv[])
{
    QStringList arguments;
    arguments.reserve(argc);
    for (int i = 0; i < argc; ++i)
        arguments.append(QString::fromLocal8Bit(argv[i]));
    return arguments;
}

QString valueOf(const QStringList &arguments, const QString &key)
{
    const int index = arguments.indexOf(key);
    if (index < 0 || index + 1 >= arguments.size())
        return QString();
    return arguments.at(index + 1);
}

bool wantsCli(const QStringList &arguments)
{
    static const QStringList cliFlags = {
        QStringLiteral("--backup"),   QStringLiteral("--prune"), QStringLiteral("--list"),
        QStringLiteral("--restore"),  QStringLiteral("--help"),  QStringLiteral("-h"),
        QStringLiteral("--version"),  QStringLiteral("-v"),
        QStringLiteral("--print-config"),
    };
    for (const QString &argument : arguments) {
        if (cliFlags.contains(argument))
            return true;
    }
    return false;
}

// Translations are embedded in the binary (qt_add_translations), so the
// resource is tried first: it always matches this build. The same fallback
// chain as Luma.
bool installTranslation(QCoreApplication &app, QTranslator *translator, const QString &requested)
{
    QCoreApplication::removeTranslator(translator);
    const QString language = ConfigManager::resolveLanguage(requested, QLocale::system().name());
    const QStringList candidates = {
        QStringLiteral(":/i18n/") + language + QStringLiteral(".qm"),
        QStringLiteral("/usr/share/aion/translations/") + language + QStringLiteral(".qm"),
        QCoreApplication::applicationDirPath() + QLatin1Char('/') + language + QStringLiteral(".qm"),
    };
    for (const QString &path : candidates) {
        if (translator->load(path))
            return app.installTranslator(translator);
    }
    return false;
}

void printProgressLine(int percent, const QString &speed, const QString &eta, const QString &state)
{
    if (percent < 0) {
        std::fprintf(stderr, "\r[aion] %-12s", qUtf8Printable(state));
        return;
    }
    const QString line = QStringLiteral("%1% %2 %3")
                             .arg(percent, 3)
                             .arg(speed.isEmpty() ? QStringLiteral(" ") : speed)
                             .arg(eta.isEmpty() ? QString() : QStringLiteral("ETA ") + eta);
    std::fprintf(stderr, "\r[aion] %-40s", qUtf8Printable(line));
    std::fflush(stderr);
}

int runCli(QCoreApplication &app, ConfigManager &config, Notifier &notifier)
{
    QCommandLineParser parser;
    parser.setApplicationDescription(QStringLiteral("Aion — Time Machine-like snapshots of your home"));
    parser.addHelpOption();
    parser.addVersionOption();

    const QCommandLineOption backupOption(QStringLiteral("backup"),
                                          QStringLiteral("Create a new snapshot and exit."));
    const QCommandLineOption pruneOption(QStringLiteral("prune"),
                                         QStringLiteral("Apply the retention policy and exit."));
    const QCommandLineOption listOption(QStringLiteral("list"),
                                        QStringLiteral("List snapshots and exit."));
    const QCommandLineOption restoreOption(QStringLiteral("restore"),
                                           QStringLiteral("Restore items from a snapshot."),
                                           QStringLiteral("id"));
    const QCommandLineOption toOption(QStringLiteral("to"),
                                      QStringLiteral("Restore destination (default: /)."),
                                      QStringLiteral("directory"));
    const QCommandLineOption policyOption(
        QStringLiteral("policy"),
        QStringLiteral("Conflict policy: overwrite, skip or keep_both (default: overwrite)."),
        QStringLiteral("policy"), QStringLiteral("overwrite"));
    const QCommandLineOption quietOption(QStringLiteral("quiet"),
                                         QStringLiteral("Do not print progress."));
    const QCommandLineOption printConfigOption(
        QStringLiteral("print-config"),
        QStringLiteral("Print the effective configuration and exit."));
    parser.addOptions({backupOption, pruneOption, listOption, restoreOption, toOption,
                       policyOption, quietOption, printConfigOption});
    parser.addPositionalArgument(QStringLiteral("paths"),
                                 QStringLiteral("Items to restore, relative to the snapshot root."));
    parser.process(app);

    SnapshotStore store(&config);
    const bool quiet = parser.isSet(quietOption);

    if (parser.isSet(printConfigOption)) {
        QTextStream out(stdout);
        out << QStringLiteral("config:   ") << config.configFilePath() << Qt::endl;
        out << QStringLiteral("target:   ") << config.targetDir() << Qt::endl;
        out << QStringLiteral("snapshots:") << Qt::endl;
        for (const QString &source : config.sources())
            out << QStringLiteral("  + ") << source << Qt::endl;
        out << QStringLiteral("excludes:") << Qt::endl;
        for (const QString &exclude : config.excludes())
            out << QStringLiteral("  - ") << exclude << Qt::endl;
        out << QStringLiteral("language: ") << config.language() << Qt::endl;
        out << QStringLiteral("schedule: ")
            << (config.scheduleEnabled() ? QStringLiteral("on") : QStringLiteral("off"))
            << QStringLiteral(" every ") << config.scheduleMinutes() << QStringLiteral(" min")
            << Qt::endl;
        return 0;
    }

    if (parser.isSet(listOption)) {
        QTextStream out(stdout);
        if (store.ids().isEmpty()) {
            out << QStringLiteral("no snapshots in ") << store.snapshotsPath() << Qt::endl;
            return 0;
        }
        out << store.snapshotsPath() << Qt::endl;
        for (const QString &id : store.ids()) {
            const SnapshotInfo info = store.snapshot(id);
            out << id << "  " << info.status << "  " << Format::bytes(info.transferredBytes)
                << QStringLiteral(" added, ") << Format::bytes(info.totalBytes)
                << (id == store.latestId() ? QStringLiteral("  [latest]") : QString()) << Qt::endl;
        }
        return 0;
    }

    if (parser.isSet(pruneOption)) {
        const QStringList removed = store.prune();
        if (!quiet) {
            QTextStream out(stdout);
            if (removed.isEmpty())
                out << QStringLiteral("nothing to prune") << Qt::endl;
            else
                out << QStringLiteral("removed ") << removed.join(QStringLiteral(", "))
                    << Qt::endl;
        }
        return 0;
    }

    if (parser.isSet(backupOption)) {
        BackupRunner backup(&config, &store, &notifier);
        int exitCode = 1;
        bool done = false;
        QObject::connect(&backup, &BackupRunner::finished, &app,
                         [&](bool ok, const QString &id) {
                             exitCode = ok ? 0 : 1;
                             done = true;
                             if (!quiet) {
                                 std::fprintf(stderr, "\n");
                                 QTextStream out(stdout);
                                 if (ok)
                                     out << QStringLiteral("snapshot ") << id << Qt::endl;
                                 else
                                     out << QStringLiteral("backup failed: ")
                                         << backup.errorMessage() << Qt::endl;
                             }
                             QCoreApplication::quit();
                         });
        if (!quiet) {
            QObject::connect(&backup, &BackupRunner::progressChanged, &app, [&]() {
                printProgressLine(backup.progress(), backup.speedText(), backup.etaText(),
                                  backup.state());
            });
            QObject::connect(&backup, &BackupRunner::stateChanged, &app, [&]() {
                printProgressLine(backup.progress(), backup.speedText(), backup.etaText(),
                                  backup.state());
            });
        }
        backup.start();
        if (!done)
            app.exec();
        return exitCode;
    }

    if (parser.isSet(restoreOption)) {
        const QString snapshotId = parser.value(restoreOption);
        const QStringList paths = parser.positionalArguments();
        if (paths.isEmpty()) {
            QTextStream(stderr) << QStringLiteral("aion: --restore needs at least one path\n");
            return 2;
        }

        RestoreRunner restore(&store, &notifier);
        int exitCode = 1;
        bool done = false;
        QObject::connect(&restore, &RestoreRunner::finished, &app,
                         [&](bool ok, int count) {
                             exitCode = ok ? 0 : 1;
                             done = true;
                             if (!quiet) {
                                 QTextStream out(ok ? stdout : stderr);
                                 if (ok)
                                     out << QStringLiteral("restored ") << count
                                         << QStringLiteral(" item(s) to ")
                                         << restore.destination() << Qt::endl;
                                 else
                                     out << QStringLiteral("restore failed: ")
                                         << restore.errorMessage() << Qt::endl;
                             }
                             QCoreApplication::quit();
                         });
        if (!quiet) {
            QObject::connect(&restore, &RestoreRunner::progressChanged, &app, [&]() {
                printProgressLine(restore.progress(), QString(), QString(),
                                  restore.currentItem());
            });
        }
        restore.start(snapshotId, paths, parser.value(toOption), parser.value(policyOption));
        if (!done)
            app.exec();
        return exitCode;
    }

    parser.showHelp(0);
    return 0;
}

} // namespace

int main(int argc, char *argv[])
{
    const QStringList rawArguments = argumentsOf(argc, argv);
    const bool cli = wantsCli(rawArguments);
    const bool smokeTest = rawArguments.contains(QStringLiteral("--smoke-test"));
    // Development-only: render the window to a PNG (offscreen) so the UI can be
    // checked without a display. See AGENTS.md.
    const QString capturePath = valueOf(rawArguments, QStringLiteral("--capture"));
    const QString captureDialog = valueOf(rawArguments, QStringLiteral("--capture-dialog"));

    std::unique_ptr<QCoreApplication> application;
    if (cli) {
        application = std::make_unique<QCoreApplication>(argc, argv);
    } else {
        application = std::make_unique<QGuiApplication>(argc, argv);
        QGuiApplication::setDesktopFileName(QStringLiteral("aion"));
        QGuiApplication::setApplicationDisplayName(QStringLiteral("Aion"));
        QGuiApplication::setWindowIcon(QIcon(QStringLiteral(":/assets/aion.png")));
    }

    QCoreApplication &app = *application;
    QCoreApplication::setApplicationName(QStringLiteral("aion"));
    QCoreApplication::setApplicationVersion(QStringLiteral(AION_VERSION));

    Logger::install(cacheDirectory());

    ConfigManager config;
    auto *translator = new QTranslator(&app);
    installTranslation(app, translator, config.language());
    QObject::connect(&config, &ConfigManager::languageChanged, &app,
                     [&app, translator, &config]() {
                         installTranslation(app, translator, config.language());
                     });

    Notifier notifier;

    if (cli)
        return runCli(app, config, notifier);

    // ---- GUI ----
    const QString serverName = QStringLiteral("aion_single_instance_server");

    if (config.singleInstance()) {
        QLocalSocket socket;
        socket.connectToServer(serverName);
        if (socket.waitForConnected(500)) {
            QTextStream stream(&socket);
            for (const QString &argument : rawArguments)
                stream << argument << '\n';
            stream.flush();
            socket.waitForBytesWritten(1000);
            socket.disconnectFromServer();
            return 0;
        }
    }

    SnapshotStore store(&config);
    SnapshotModel timeline(&store);
    DirectoryModel browser(&store);
    BackupRunner backup(&config, &store, &notifier);
    RestoreRunner restore(&store, &notifier);
    Autostart autostart(&config);
    AppController appController(&config);

    // Time Machine-like thinning happens automatically after a good backup.
    QObject::connect(&backup, &BackupRunner::finished, &store, [&store](bool ok, const QString &) {
        if (!ok)
            return;
        const QStringList removed = store.prune();
        if (!removed.isEmpty())
            qInfo() << "Aion: pruned" << removed.size() << "old snapshot(s)";
    });

    QQmlApplicationEngine engine;
    engine.rootContext()->setContextProperty(QStringLiteral("Config"), &config);
    engine.rootContext()->setContextProperty(QStringLiteral("Store"), &store);
    engine.rootContext()->setContextProperty(QStringLiteral("Timeline"), &timeline);
    engine.rootContext()->setContextProperty(QStringLiteral("Browser"), &browser);
    engine.rootContext()->setContextProperty(QStringLiteral("Backup"), &backup);
    engine.rootContext()->setContextProperty(QStringLiteral("Restore"), &restore);
    engine.rootContext()->setContextProperty(QStringLiteral("Scheduler"), &autostart);
    engine.rootContext()->setContextProperty(QStringLiteral("App"), &appController);

    QLocalServer *server = nullptr;
    if (config.singleInstance()) {
        QLocalServer::removeServer(serverName);
        server = new QLocalServer(&app);
        server->setSocketOptions(QLocalServer::UserAccessOption);
        if (!server->listen(serverName))
            qWarning() << "Aion: cannot start the single-instance server:"
                       << server->errorString();
        QObject::connect(server, &QLocalServer::newConnection, &app, [server, &engine, &backup]() {
            QLocalSocket *connection = server->nextPendingConnection();
            if (!connection)
                return;
            connection->waitForReadyRead(300);
            const QStringList forwarded =
                QString::fromUtf8(connection->readAll()).split(QLatin1Char('\n'), Qt::SkipEmptyParts);
            connection->disconnectFromServer();
            connection->deleteLater();
            if (forwarded.contains(QStringLiteral("--backup")))
                backup.start();
            if (!engine.rootObjects().isEmpty()) {
                if (auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().constFirst())) {
                    window->show();
                    window->raise();
                    window->requestActivate();
                }
            }
        });
    }

    engine.load(QUrl(QStringLiteral("qrc:/qt/qml/aion/qml/Main.qml")));
    if (engine.rootObjects().isEmpty())
        return 1;

    if (smokeTest || !capturePath.isEmpty()) {
        if (!capturePath.isEmpty() && !captureDialog.isEmpty()) {
            QTimer::singleShot(900, &app, [&engine, captureDialog]() {
                if (!engine.rootObjects().isEmpty())
                    QMetaObject::invokeMethod(engine.rootObjects().constFirst(),
                                              "openDialogForCapture",
                                              Q_ARG(QVariant, captureDialog));
            });
        }
        QTimer::singleShot(1500, &app, [&app, &engine, capturePath, smokeTest]() {
            if (!capturePath.isEmpty()) {
                if (auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().constFirst()))
                    window->grabWindow().save(capturePath);
            }
            if (smokeTest)
                std::fprintf(stderr, "AION_SMOKE_OK\n");
            app.quit();
        });
    }

    return app.exec();
}
