// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 B612S <sem.alimov@inbox.ru>

mod config;
mod format;
mod i18n;
mod notify;
mod objects;
mod paths;
mod rustic;
mod schedule;
mod vault;

use qmetaobject::*;
use std::io::Write;

use config::Config;
use objects::*;

extern "C" {
    fn kustos_capture_screen(path: *const std::os::raw::c_char, delay_ms: i32);
}
use rustic::Repo;
use vault::Vault;

qrc!(register_resources,
    "/" {
        "qml/Main.qml" as "qml/Main.qml",
        "qml/TimelinePanel.qml" as "qml/TimelinePanel.qml",
        "qml/SnapshotBrowser.qml" as "qml/SnapshotBrowser.qml",
        "qml/SettingsDialog.qml" as "qml/SettingsDialog.qml",
        "qml/BackupDialog.qml" as "qml/BackupDialog.qml",
        "qml/UnlockDialog.qml" as "qml/UnlockDialog.qml",
        "qml/RestoreDialog.qml" as "qml/RestoreDialog.qml",
        "qml/AboutDialog.qml" as "qml/AboutDialog.qml",
        "qml/KustosMenu.qml" as "qml/KustosMenu.qml",
        "qml/KustosMenuButton.qml" as "qml/KustosMenuButton.qml",
        "locales/en.qm" as "i18n/en.qm",
        "locales/ru.qm" as "i18n/ru.qm",
        "assets/kustos.png" as "assets/kustos.png",
        "assets/kustos.svg" as "assets/kustos.svg",
    }
);

pub fn logln(message: &str) {
    let path = paths::log_file();
    let _ = paths::ensure_parent(&path);
    if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
        let stamp = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S");
        let _ = writeln!(file, "{stamp} {message}");
    }
}

fn print_usage() {
    println!(
        "Kustos — {}\n\n\
         Usage:\n\
         \x20 kustos                     open the window\n\
         \x20 kustos --backup            create a snapshot\n\
         \x20 kustos --list              list snapshots\n\
         \x20 kustos --prune             apply the retention policy\n\
         \x20 kustos --restore <id> <path>... [--to DIR] [--policy overwrite|skip|keep_both]\n\
         \x20 kustos --print-config      show the effective configuration\n\
         \x20 kustos --version           print the version",
        env!("CARGO_PKG_VERSION")
    );
}

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let cli = arguments.iter().any(|argument| {
        matches!(
            argument.as_str(),
            "--backup" | "--list" | "--prune" | "--restore" | "--print-config" | "--help" | "-h" | "--version" | "-v" | "--quiet"
        )
    });
    if cli {
        std::process::exit(run_cli(&arguments));
    }
    run_gui();
}

fn run_cli(arguments: &[String]) -> i32 {
    let config = Config::load();
    if arguments.iter().any(|a| a == "--help" || a == "-h") {
        print_usage();
        return 0;
    }
    if arguments.iter().any(|a| a == "--version" || a == "-v") {
        println!("kustos {}", env!("CARGO_PKG_VERSION"));
        return 0;
    }
    if arguments.iter().any(|a| a == "--print-config") {
        println!("config:    {}", paths::config_file().display());
        println!("repository: {}", config.repo);
        println!("key:       not stored; asked at startup");
        println!("sources:");
        for source in &config.sources {
            println!("  + {source}");
        }
        println!("excludes:");
        for exclude in &config.excludes {
            println!("  - {exclude}");
        }
        println!("language:  {}", config.language);
        println!(
            "schedule:  {} every {} min",
            if config.schedule_enabled { "on" } else { "off" },
            config.schedule_minutes
        );
        return 0;
    }

    let quiet = arguments.iter().any(|a| a == "--quiet");
    let vault = Vault::default();
    if let Ok(password) = std::env::var("KUSTOS_PASSWORD") {
        vault.set_password(password.trim().to_string());
    } else if let Ok(file) = std::env::var("KUSTOS_PASSWORD_FILE") {
        match std::fs::read_to_string(&file) {
            Ok(text) => vault.set_password(text.trim().to_string()),
            Err(error) => {
                eprintln!("kustos: cannot read {file}: {error}");
                return 1;
            }
        }
    }
    let repo_path = config.repo.clone();
    let mut repo = Repo::new(&repo_path, vault.password());
    if !repo.exists() && arguments.iter().any(|a| a == "--backup") {
        let key = config::generate_password();
        println!("New repository key (shown once, store it safely): {key}");
        vault.set_password(key.clone());
        repo = Repo::new(&repo_path, Some(key));
    } else if repo.exists() && vault.is_locked() {
        eprintln!("kustos: the repository is locked; set KUSTOS_PASSWORD or KUSTOS_PASSWORD_FILE");
        return 1;
    }

    if arguments.iter().any(|a| a == "--list") {
        match repo.snapshots() {
            Ok(snapshots) => {
                if snapshots.is_empty() {
                    println!("no snapshots in {}", config.repo);
                }
                for snapshot in snapshots {
                    println!(
                        "{}  {}  {} files  {} total  {} added",
                        snapshot.short_id(),
                        snapshot.time,
                        snapshot.files,
                        format::bytes(snapshot.total_bytes),
                        format::bytes(snapshot.added_bytes)
                    );
                }
                if let Some(latest) = repo.snapshots().ok().and_then(|s| s.first().cloned()) {
                    let _ = latest;
                }
                return 0;
            }
            Err(error) => {
                eprintln!("kustos: {error}");
                return 1;
            }
        }
    }

    if arguments.iter().any(|a| a == "--prune") {
        match repo.forget_policy(config.keep_hourly, config.keep_daily, config.keep_weekly) {
            Ok(()) => {
                if !quiet {
                    println!("retention policy applied");
                }
                return 0;
            }
            Err(error) => {
                eprintln!("kustos: {error}");
                return 1;
            }
        }
    }

    if let Some(index) = arguments.iter().position(|a| a == "--restore") {
        let snapshot = arguments.get(index + 1).cloned().unwrap_or_default();
        let mut paths: Vec<String> = Vec::new();
        let mut destination = "/".to_string();
        let mut policy = "overwrite".to_string();
        let mut cursor = index + 2;
        while cursor < arguments.len() {
            match arguments[cursor].as_str() {
                "--to" => {
                    cursor += 1;
                    if let Some(value) = arguments.get(cursor) {
                        destination = value.clone();
                    }
                }
                "--policy" => {
                    cursor += 1;
                    if let Some(value) = arguments.get(cursor) {
                        policy = value.clone();
                    }
                }
                value => paths.push(value.to_string()),
            }
            cursor += 1;
        }
        if snapshot.is_empty() || paths.is_empty() {
            eprintln!("kustos: --restore needs a snapshot id and at least one path");
            return 2;
        }
        let mut failures = 0;
        for (position, path) in paths.iter().enumerate() {
            let staging = paths::staging_root().join(format!("cli-{position}"));
            let _ = std::fs::remove_dir_all(&staging);
            if let Err(error) = repo.restore(&snapshot, path, &staging, &std::sync::atomic::AtomicBool::new(false), |_| {}) {
                eprintln!("kustos: {path}: {error}");
                failures += 1;
                continue;
            }
            if let Err(error) = objects::place_item(&staging, path, &destination, &policy) {
                eprintln!("kustos: {path}: {error}");
                failures += 1;
            }
            let _ = std::fs::remove_dir_all(&staging);
        }
        return if failures == 0 { 0 } else { 1 };
    }

    if arguments.iter().any(|a| a == "--backup") {
        let (sources, missing) = config.existing_sources();
        if sources.is_empty() {
            eprintln!("kustos: none of the configured sources exist");
            return 1;
        }
        for source in &missing {
            eprintln!("kustos: skipping missing source {source}");
        }
        let cancel = std::sync::atomic::AtomicBool::new(false);
        let started = std::time::Instant::now();
        let mut last_percent = -1.0f64;
        let result = repo.backup(&sources, &config.excludes, &cancel, |update| {
            if quiet {
                return;
            }
            let percent = update.percent * 100.0;
            if (percent - last_percent).abs() >= 1.0 {
                last_percent = percent;
                eprint!("\r\x1b[K[kustos] {percent:5.1}%  {}  {}", format::bytes(update.bytes_done), format::duration(started.elapsed().as_secs() as i64));
            }
        });
        if !quiet {
            eprintln!();
        }
        match result {
            Ok(summary) => {
                if !quiet {
                    println!(
                        "snapshot {}: {} files, {} added, {}",
                        summary.snapshot_id.get(..8).unwrap_or(&summary.snapshot_id),
                        summary.total_files,
                        format::bytes(summary.data_added),
                        format::duration(summary.duration as i64)
                    );
                }
                let _ = repo.forget_policy(config.keep_hourly, config.keep_daily, config.keep_weekly);
                0
            }
            Err(error) => {
                eprintln!("kustos: backup failed: {error}");
                1
            }
        }
    } else {
        print_usage();
        2
    }
}

fn run_gui() {
    register_resources();
    let config = Config::load();

    // Keep the systemd user timer in sync with the configuration at startup.
    if config.schedule_enabled {
        if let Err(error) = schedule::install(config.schedule_minutes) {
            logln(&format!("schedule: {error}"));
        }
    }

    // The engine creates the QGuiApplication; a QTranslator can only be
    // installed afterwards, but before the QML file is loaded.
    let mut engine = QmlEngine::new();
    i18n::install(&config::resolve_language(&config.language));

    let vault = Vault::default();

    let config_object = QObjectBox::new(ConfigObject::new(config.clone(), vault.clone()));
    let store_object = QObjectBox::new(StoreObject::new(config.clone(), vault.clone()));
    let backup_object = QObjectBox::new(BackupObject::new(config.clone(), vault.clone()));
    let restore_object = QObjectBox::new(RestoreObject::new(config.clone(), vault.clone()));
    let browser_object = QObjectBox::new(BrowserObject::new(config.clone(), vault.clone()));
    let scheduler_object = QObjectBox::new(SchedulerObject::new(config.clone(), vault.clone()));
    let app_object = QObjectBox::new(AppObject::new(config.clone(), vault.clone()));

    engine.set_object_property("Config".into(), config_object.pinned());
    engine.set_object_property("Store".into(), store_object.pinned());
    engine.set_object_property("Backup".into(), backup_object.pinned());
    engine.set_object_property("Restore".into(), restore_object.pinned());
    engine.set_object_property("Browser".into(), browser_object.pinned());
    engine.set_object_property("Scheduler".into(), scheduler_object.pinned());
    engine.set_object_property("App".into(), app_object.pinned());

    // Development-only screenshot hook: the QML side checks these context
    // properties and saves the window when KUSTOS_CAPTURE is set.
    if let Ok(path) = std::env::var("KUSTOS_CAPTURE") {
        engine.set_property("CapturePath".into(), QVariant::from(QString::from(path)));
    }
    if let Ok(dialog) = std::env::var("KUSTOS_CAPTURE_DIALOG") {
        engine.set_property("CaptureDialog".into(), QVariant::from(QString::from(dialog)));
    }

    engine.load_file("qrc:/qml/Main.qml".into());

    if let Ok(path) = std::env::var("KUSTOS_CAPTURE") {
        if let Ok(target) = std::ffi::CString::new(path) {
            unsafe { kustos_capture_screen(target.as_ptr(), 1800) };
        }
    }

    engine.exec();
}
