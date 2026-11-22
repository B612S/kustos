// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 B612S <sem.alimov@inbox.ru>

mod activity;
mod config;
mod format;
mod i18n;
mod joblock;
mod notify;
mod objects;
mod paths;
mod rustic;
mod schedule;
mod vault;

use qmetaobject::*;
use std::io::Write;

use config::Config;
use joblock::JobLock;
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
        "qml/RulesPanel.qml" as "qml/RulesPanel.qml",
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

/// Keeps the log bounded: once it passes 1 MiB it is rotated to
/// `kustos.log.1` (the previous `.1` is replaced).
fn rotate_log(path: &std::path::Path) {
    let too_large = std::fs::metadata(path)
        .map(|metadata| metadata.len() > 1024 * 1024)
        .unwrap_or(false);
    if too_large {
        let _ = std::fs::rename(path, path.with_extension("log.1"));
    }
}

pub fn logln(message: &str) {
    let path = paths::log_file();
    let _ = paths::ensure_parent(&path);
    rotate_log(&path);
    if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
        let stamp = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S");
        let _ = writeln!(file, "{stamp} {message}");
    }
}

/// The password of a generated systemd unit is passed as a credential named
/// `kustos-password` in `$CREDENTIALS_DIRECTORY`.
fn credential_password() -> Option<String> {
    let directory = std::env::var("CREDENTIALS_DIRECTORY").ok()?;
    let text = std::fs::read_to_string(
        std::path::Path::new(&directory).join("kustos-password"),
    )
    .ok()?;
    let text = text.trim().to_string();
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

fn print_usage() {
    println!(
        "Kustos — {}\n\n\
         Usage:\n\
         \x20 kustos                     open the window\n\
         \x20 kustos --backup            create a snapshot (all enabled rules)\n\
         \x20 kustos --backup --rule ID  create a snapshot for one rule\n\
         \x20 kustos --check             verify the repository structure\n\
         \x20 kustos --check --read-data verify a random 10% of the data too\n\
         \x20 kustos --list              list snapshots\n\
         \x20 kustos --prune             apply the retention policy\n\
         \x20 kustos --prune --dry-run   show what the policy would delete\n\
         \x20 kustos --restore <id> <path>... [--to DIR] [--policy overwrite|skip|keep_both]\n\
         \x20 kustos --print-config      show the effective configuration\n\
         \x20 kustos --version           print the version\n\
         \n\
         The CLI reads the repository password from KUSTOS_PASSWORD,\n\
         KUSTOS_PASSWORD_FILE or the systemd credential directory.",
        env!("CARGO_PKG_VERSION")
    );
}

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let cli = arguments.iter().any(|argument| {
        matches!(
            argument.as_str(),
            "--backup" | "--check" | "--list" | "--prune" | "--restore" | "--print-config"
                | "--help" | "-h" | "--version" | "-v" | "--quiet"
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
        println!(
            "lock after: {}",
            if config.lock_timeout_minutes == 0 {
                "never".to_string()
            } else {
                format!("{} min of inactivity", config.lock_timeout_minutes)
            }
        );
        println!("language:  {}", config.language);
        println!(
            "passwords: {}",
            if config.common_mode() {
                format!(
                    "one common password ({})",
                    if let Some(key) = config.common_key_id() {
                        format!("key {key}")
                    } else {
                        "not set".to_string()
                    }
                )
            } else {
                "one per rule".to_string()
            }
        );
        println!("rules:");
        for rule in &config.rules {
            println!(
                "  [{}] {} ({})",
                rule.id,
                if rule.name.is_empty() { "(untitled)" } else { &rule.name },
                if rule.enabled { "enabled" } else { "paused" }
            );
            println!(
                "    password: {}",
                if rule.has_password() {
                    format!("own key {}", rule.key_id)
                } else {
                    "not set".to_string()
                }
            );
            println!("    sources:");
            for source in &rule.sources {
                println!("      + {source}");
            }
            println!("    excludes:");
            for exclude in &rule.excludes {
                println!("      - {exclude}");
            }
            println!("    schedule: {}", rule.schedule.describe());
            if rule.one_file_system {
                println!("    one filesystem: yes");
            }
            if !rule.exclude_larger.is_empty() {
                println!("    exclude larger than: {}", rule.exclude_larger);
            }
        }
        return 0;
    }

    let quiet = arguments.iter().any(|a| a == "--quiet");
    if let Err(error) = config.verify_repository() {
        eprintln!("kustos: {error}");
        return 1;
    }
    let vault = Vault::default();
    // The GUI keeps passwords in memory only. A command-line run gets one
    // from the environment or from the systemd credential directory that the
    // generated units use.
    if let Ok(password) = std::env::var("KUSTOS_PASSWORD") {
        vault.set_rule_password("__cli__", password.trim().to_string());
    } else if let Ok(file) = std::env::var("KUSTOS_PASSWORD_FILE") {
        match std::fs::read_to_string(&file) {
            Ok(text) => vault.set_rule_password("__cli__", text.trim().to_string()),
            Err(error) => {
                eprintln!("kustos: cannot read {file}: {error}");
                return 1;
            }
        }
    } else if let Some(password) = credential_password() {
        vault.set_rule_password("__cli__", password);
    }
    let repo_path = config.repo.clone();
    let repo = Repo::new(&repo_path, vault.any_password());
    if repo.exists() && vault.is_empty() {
        eprintln!(
            "kustos: the repository is locked; set KUSTOS_PASSWORD or KUSTOS_PASSWORD_FILE"
        );
        return 1;
    }
    if !repo.exists() && arguments.iter().any(|a| a == "--backup") && vault.is_empty() {
        eprintln!(
            "kustos: a password is required to create the repository; use the GUI or set KUSTOS_PASSWORD"
        );
        return 1;
    }

    if arguments.iter().any(|a| a == "--check") {
        let read_data = arguments.iter().any(|a| a == "--read-data");
        match repo.check(read_data) {
            Ok(detail) => {
                config::record_repository_check(true, &detail);
                if !quiet {
                    println!("repository check passed");
                }
                return 0;
            }
            Err(error) => {
                config::record_repository_check(false, &error);
                eprintln!("kustos: repository check failed: {error}");
                return 1;
            }
        }
    }

    if arguments.iter().any(|a| a == "--list") {
        match repo.snapshots() {
            Ok(snapshots) => {
                if snapshots.is_empty() {
                    println!("no snapshots in {}", config.repo);
                }
                for snapshot in snapshots {
                    let rule = snapshot
                        .rule_id()
                        .and_then(|id| config.rules.iter().find(|rule| rule.id == id))
                        .map(|rule| {
                            if rule.name.is_empty() {
                                rule.id.clone()
                            } else {
                                rule.name.clone()
                            }
                        })
                        .unwrap_or_default();
                    println!(
                        "{}  {}  {} files  {} total  {} added ({} on disk){}{}",
                        snapshot.short_id(),
                        snapshot.time,
                        snapshot.files,
                        format::bytes(snapshot.total_bytes),
                        format::bytes(snapshot.added_bytes),
                        format::bytes(snapshot.added_packed_bytes),
                        if rule.is_empty() { String::new() } else { format!("  [{rule}]") },
                        if snapshot.pinned() { "  [pinned]" } else { "" }
                    );
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
        if arguments.iter().any(|a| a == "--dry-run") {
            match repo.forget_preview(
                config.keep_hourly,
                config.keep_daily,
                config.keep_weekly,
            ) {
                Ok(snapshots) => {
                    if snapshots.is_empty() {
                        println!("nothing to prune");
                    }
                    for snapshot in snapshots {
                        println!(
                            "{}  {}  {}",
                            snapshot.short_id(),
                            snapshot.time,
                            snapshot.tags.join(",")
                        );
                    }
                    return 0;
                }
                Err(error) => {
                    eprintln!("kustos: {error}");
                    return 1;
                }
            }
        }
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
        let rule_filter = arguments
            .iter()
            .position(|argument| argument == "--rule")
            .and_then(|index| arguments.get(index + 1))
            .cloned()
            .unwrap_or_default();
        if !rule_filter.is_empty() && config.rule(&rule_filter).is_none() {
            eprintln!("kustos: there is no backup rule '{rule_filter}'");
            return 2;
        }
        let rules: Vec<&config::BackupRule> = if rule_filter.is_empty() {
            config.rules.iter().filter(|rule| rule.enabled).collect()
        } else {
            config.rule(&rule_filter).into_iter().collect()
        };
        if rules.is_empty() {
            eprintln!("kustos: no backup rules are enabled");
            return 1;
        }

        // Scheduled rules can fire at the same moment; wait for the current
        // job instead of letting `rustic` fail on the repository lock.
        let _lock = match JobLock::acquire() {
            Ok(lock) => lock,
            Err(error) => {
                eprintln!("kustos: {error}");
                return 1;
            }
        };

        let mut failures = 0;
        let mut succeeded = 0;
        for rule in rules {
            let (sources, missing) = Config::existing_sources(rule);
            for source in &missing {
                eprintln!("kustos: skipping missing source {source}");
            }
            if sources.is_empty() {
                eprintln!("kustos: rule '{}' has no existing sources", rule.name);
                continue;
            }
            let (excludes, inside_warning) = config::effective_excludes(rule, &config.repo);
            if let Some(warning) = inside_warning {
                eprintln!("kustos: {warning}");
            }
            let cancel = std::sync::atomic::AtomicBool::new(false);
            let started = std::time::Instant::now();
            let mut last_percent = -1.0f64;
            let result = repo.backup(
                &sources,
                &excludes,
                &rule.id,
                rule.one_file_system,
                &rule.exclude_larger,
                &cancel,
                |update| {
                    if quiet {
                        return;
                    }
                    let percent = update.percent * 100.0;
                    if (percent - last_percent).abs() >= 1.0 {
                        last_percent = percent;
                        eprint!(
                            "\r\x1b[K[kustos] {} {percent:5.1}%  {}  {}",
                            if rule.name.is_empty() { &rule.id } else { &rule.name },
                            format::bytes(update.bytes_done),
                            format::duration(started.elapsed().as_secs() as i64)
                        );
                    }
                },
            );
            if !quiet {
                eprintln!();
            }
            match result {
                Ok(summary) => {
                    succeeded += 1;
                    config::remember_repository_fingerprint(&config.repo);
                    if !quiet {
                        println!(
                            "snapshot {}: {} files, {} added ({} on disk), {} (rule {})",
                            summary.snapshot_id.get(..8).unwrap_or(&summary.snapshot_id),
                            summary.total_files,
                            format::bytes(summary.data_added),
                            format::bytes(summary.data_added_packed),
                            format::duration(summary.duration as i64),
                            if rule.name.is_empty() { &rule.id } else { &rule.name }
                        );
                    }
                }
                Err(error) => {
                    failures += 1;
                    eprintln!("kustos: backup failed for rule '{}': {error}", rule.id);
                }
            }
        }
        if succeeded > 0 {
            let _ = repo.forget_policy(config.keep_hourly, config.keep_daily, config.keep_weekly);
        }
        if failures == 0 && succeeded > 0 { 0 } else { 1 }
    } else {
        print_usage();
        2
    }
}

fn run_gui() {
    register_resources();
    let config = Config::load();

    // Keep the per-rule systemd user timers in sync with the configuration at
    // startup (this also removes timers of deleted rules).
    if let Err(error) = schedule::sync(&config.rules) {
        logln(&format!("schedule: {error}"));
    }

    // The engine creates the QGuiApplication; a QTranslator can only be
    // installed afterwards, but before the QML file is loaded.
    let mut engine = QmlEngine::new();
    i18n::install(&config::resolve_language(&config.language));
    activity::install();

    let vault = Vault::default();
    vault.set_timeout_minutes(config.lock_timeout_minutes);
    activity::start_idle_guard(vault.clone());

    // Development-only: screenshots of a real repository need a password, and
    // a capture is not interactive enough to type it into the password dialog.
    if std::env::var_os("KUSTOS_CAPTURE").is_some() {
        if let Ok(password) = std::env::var("KUSTOS_PASSWORD") {
            let password = password.trim().to_string();
            if config.rules.is_empty() {
                vault.set_rule_password("__capture__", password);
            } else {
                for rule in &config.rules {
                    vault.set_rule_password(&rule.id, password.clone());
                }
            }
        }
    }

    let config_object = QObjectBox::new(ConfigObject::new(config.clone(), vault.clone()));
    let store_object = QObjectBox::new(StoreObject::new(config.clone(), vault.clone()));
    let backup_object = QObjectBox::new(BackupObject::new(config.clone(), vault.clone()));
    let restore_object = QObjectBox::new(RestoreObject::new(config.clone(), vault.clone()));
    let browser_object = QObjectBox::new(BrowserObject::new(config.clone(), vault.clone()));
    let scheduler_object = QObjectBox::new(SchedulerObject::new(config.clone(), vault.clone()));
    let maintenance_object = QObjectBox::new(MaintenanceObject::new(config.clone(), vault.clone()));
    let app_object = QObjectBox::new(AppObject::new(config.clone(), vault.clone()));

    engine.set_object_property("Config".into(), config_object.pinned());
    engine.set_object_property("Store".into(), store_object.pinned());
    engine.set_object_property("Backup".into(), backup_object.pinned());
    engine.set_object_property("Restore".into(), restore_object.pinned());
    engine.set_object_property("Browser".into(), browser_object.pinned());
    engine.set_object_property("Scheduler".into(), scheduler_object.pinned());
    engine.set_object_property("Maintenance".into(), maintenance_object.pinned());
    engine.set_object_property("App".into(), app_object.pinned());

    // Development-only screenshot hook: the QML side checks these context
    // properties and saves the window when KUSTOS_CAPTURE is set.
    if let Ok(path) = std::env::var("KUSTOS_CAPTURE") {
        engine.set_property("CapturePath".into(), QVariant::from(QString::from(path)));
    }
    if let Ok(dialog) = std::env::var("KUSTOS_CAPTURE_DIALOG") {
        engine.set_property("CaptureDialog".into(), QVariant::from(QString::from(dialog)));
    }
    if let Ok(action) = std::env::var("KUSTOS_CAPTURE_ACTION") {
        engine.set_property("CaptureAction".into(), QVariant::from(QString::from(action)));
    }

    engine.load_file("qrc:/qml/Main.qml".into());

    if let Ok(path) = std::env::var("KUSTOS_CAPTURE") {
        if let Ok(target) = std::ffi::CString::new(path) {
            unsafe { kustos_capture_screen(target.as_ptr(), 1800) };
        }
    }

    engine.exec();
}

#[cfg(test)]
mod tests {
    use super::rotate_log;

    #[test]
    fn a_large_log_is_rotated() {
        let dir = std::env::temp_dir().join(format!("kustos-log-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("kustos.log");
        std::fs::write(&path, vec![b'x'; 1024 * 1024 + 1]).unwrap();

        rotate_log(&path);

        assert!(!path.exists());
        assert!(dir.join("kustos.log.1").exists());
        let _ = std::fs::remove_dir_all(dir);
    }
}
