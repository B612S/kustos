// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 B612S <sem.alimov@inbox.ru>

//! QObjects exposed to QML as context properties: `Config`, `Store`, `Backup`,
//! `Restore`, `Browser`, `Scheduler`, `App`.

#![allow(non_snake_case)]
//!
//! Long operations (backup, restore) run on a Rust thread and publish their
//! state through an `Arc<Mutex<JobStatus>>`; QML polls it with a `Timer` and a
//! local property, so no cross-thread signal marshalling is needed.

use qmetaobject::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use crate::config::{self, Config};
use crate::format;
use crate::paths;
use crate::rustic::{self, Repo};
use crate::vault::Vault;
use crate::schedule;

fn qstr(value: &str) -> QString {
    QString::from(value)
}

fn map_of(pairs: Vec<(&str, QVariant)>) -> QVariant {
    let map: QVariantMap = pairs
        .into_iter()
        .map(|(key, value)| (key.to_string(), value))
        .collect::<HashMap<String, QVariant>>()
        .into();
    QVariant::from(map)
}

fn list_of_strings(values: &[String]) -> QVariant {
    let mut list = QVariantList::default();
    for value in values {
        list.push(QVariant::from(qstr(value)));
    }
    QVariant::from(list)
}

// ---------------------------------------------------------------- Config ----

#[derive(QObject, Default)]
pub struct ConfigObject {
    base: qt_base_class!(trait QObject),
    vault: Vault,
    pub config: Config,

    language: qt_property!(QString; NOTIFY language_changed),
    language_changed: qt_signal!(),
    repo: qt_property!(QString; NOTIFY repo_changed),
    repo_changed: qt_signal!(),
    notifications: qt_property!(bool; NOTIFY notifications_changed),
    notifications_changed: qt_signal!(),
    schedule_enabled: qt_property!(bool; NOTIFY schedule_changed),
    schedule_minutes: qt_property!(i32; NOTIFY schedule_changed),
    schedule_changed: qt_signal!(),
    keep_hourly: qt_property!(i32; NOTIFY retention_changed),
    keep_daily: qt_property!(i32; NOTIFY retention_changed),
    keep_weekly: qt_property!(i32; NOTIFY retention_changed),
    retention_changed: qt_signal!(),

    setLanguage: qt_method!(fn setLanguage(&mut self, value: QString) {
        self.apply_language(value.to_string());
    }),
    setRepo: qt_method!(fn setRepo(&mut self, value: QString) {
        self.config.repo = value.to_string().trim_end_matches('/').to_string();
        let _ = self.config.save();
        self.repo = qstr(&self.config.repo);
        self.repo_changed();
    }),
    setNotifications: qt_method!(fn setNotifications(&mut self, value: bool) {
        self.config.notifications = value;
        let _ = self.config.save();
        self.notifications = value;
        self.notifications_changed();
    }),
    setScheduleEnabled: qt_method!(fn setScheduleEnabled(&mut self, value: bool) {
        self.config.schedule_enabled = value;
        let _ = self.config.save();
        self.schedule_enabled = value;
        self.schedule_changed();
        let result = if value {
            crate::schedule::install(self.config.schedule_minutes)
        } else {
            crate::schedule::uninstall()
        };
        if let Err(error) = result {
            crate::logln(&format!("schedule: {error}"));
        }
    }),
    setScheduleMinutes: qt_method!(fn setScheduleMinutes(&mut self, value: i32) {
        self.config.schedule_minutes = value.max(5) as u32;
        let _ = self.config.save();
        self.schedule_minutes = value.max(5);
        self.schedule_changed();
        if self.config.schedule_enabled {
            if let Err(error) = crate::schedule::install(self.config.schedule_minutes) {
                crate::logln(&format!("schedule: {error}"));
            }
        }
    }),
    setKeepHourly: qt_method!(fn setKeepHourly(&mut self, value: i32) {
        self.config.keep_hourly = value.clamp(1, 720) as u32;
        let _ = self.config.save();
        self.keep_hourly = self.config.keep_hourly as i32;
        self.retention_changed();
    }),
    setKeepDaily: qt_method!(fn setKeepDaily(&mut self, value: i32) {
        self.config.keep_daily = value.max(0) as u32;
        let _ = self.config.save();
        self.keep_daily = self.config.keep_daily as i32;
        self.retention_changed();
    }),
    setKeepWeekly: qt_method!(fn setKeepWeekly(&mut self, value: i32) {
        self.config.keep_weekly = value.max(0) as u32;
        let _ = self.config.save();
        self.keep_weekly = self.config.keep_weekly as i32;
        self.retention_changed();
    }),

    sources: qt_method!(fn sources(&self) -> QVariant { list_of_strings(&self.config.sources) }),
    excludes: qt_method!(fn excludes(&self) -> QVariant { list_of_strings(&self.config.excludes) }),
    addSource: qt_method!(fn addSource(&mut self, path: QString) {
        self.config.add_source(path.to_string());
        self.sources_changed();
    }),
    removeSource: qt_method!(fn removeSource(&mut self, index: i32) {
        self.config.remove_source(index as usize);
        self.sources_changed();
    }),
    addExclude: qt_method!(fn addExclude(&mut self, path: QString) {
        self.config.add_exclude(path.to_string());
        self.excludes_changed();
    }),
    removeExclude: qt_method!(fn removeExclude(&mut self, index: i32) {
        self.config.remove_exclude(index as usize);
        self.excludes_changed();
    }),
    reset: qt_method!(fn reset(&mut self) {
        self.config = Config::default();
        let _ = self.config.save();
        self.reload_properties();
        self.language_changed();
        self.repo_changed();
        self.notifications_changed();
        self.schedule_changed();
        self.retention_changed();
        self.sources_changed();
        self.excludes_changed();
    }),

    sources_changed: qt_signal!(),
    excludes_changed: qt_signal!(),
    configPath: qt_method!(fn configPath(&self) -> QString {
        qstr(&paths::config_file().to_string_lossy())
    }),
}

impl ConfigObject {
    pub fn new(config: Config, vault: Vault) -> Self {
        let mut object = ConfigObject::default();
        object.vault = vault;
        object.apply_config(config);
        object
    }

    fn apply_config(&mut self, config: Config) {
        self.config = config;
        self.reload_properties();
    }

    fn reload_properties(&mut self) {
        self.language = qstr(&self.config.language);
        self.repo = qstr(&self.config.repo);
        self.notifications = self.config.notifications;
        self.schedule_enabled = self.config.schedule_enabled;
        self.schedule_minutes = self.config.schedule_minutes as i32;
        self.keep_hourly = self.config.keep_hourly as i32;
        self.keep_daily = self.config.keep_daily as i32;
        self.keep_weekly = self.config.keep_weekly as i32;
    }

    fn apply_language(&mut self, value: String) {
        self.config.language = value.clone();
        let _ = self.config.save();
        self.language = qstr(&value);
        self.language_changed();
        crate::i18n::install(&config::resolve_language(&value));
    }
}

// ----------------------------------------------------------------- Store ----

#[derive(QObject, Default)]
pub struct StoreObject {
    base: qt_base_class!(trait QObject),
    vault: Vault,
    pub config: Config,

    snapshots: qt_method!(fn snapshots(&self) -> QVariant { self.snapshot_list() }),
    freeBytes: qt_method!(fn freeBytes(&self) -> f64 { self.repo().free_bytes() as f64 }),
    repoExists: qt_method!(fn repoExists(&self) -> bool { self.repo().exists() }),
    repoPath: qt_method!(fn repoPath(&self) -> QString { qstr(&self.config.repo) }),
    status: qt_method!(fn status(&self) -> QVariant {
        let repo = self.repo();
        map_of(vec![
            ("hasRepo", QVariant::from(repo.exists())),
            ("locked", QVariant::from(repo.exists() && self.vault.is_locked())),
            ("error", QVariant::from(qstr(""))),
        ])
    }),
    unlock: qt_method!(fn unlock(&mut self, key: QString) -> bool {
        let password = key.to_string();
        if password.is_empty() {
            return false;
        }
        let repo = Repo::new(&self.config.repo, Some(password.clone()));
        match repo.snapshots() {
            Ok(_) => {
                self.vault.set_password(password);
                true
            }
            Err(error) => {
                crate::logln(&format!("store: unlock failed: {error}"));
                false
            }
        }
    }),
    initRepo: qt_method!(fn initRepo(&mut self) -> bool {
        let repo = self.repo();
        repo.ensure_repo().is_ok()
    }),
    remove: qt_method!(fn remove(&self, id: QString) -> bool {
        let repo = self.repo();
        match repo.forget_snapshot(&id.to_string()) {
            Ok(()) => true,
            Err(error) => {
                crate::logln(&format!("store: cannot remove snapshot: {error}"));
                false
            }
        }
    }),
    pruneNow: qt_method!(fn pruneNow(&self) -> i32 {
        let repo = self.repo();
        match repo.forget_policy(self.config.keep_hourly, self.config.keep_daily, self.config.keep_weekly) {
            Ok(()) => 1,
            Err(error) => {
                crate::logln(&format!("store: prune failed: {error}"));
                0
            }
        }
    }),
}

impl StoreObject {
    pub fn new(config: Config, vault: Vault) -> Self {
        let mut object = StoreObject::default();
        object.config = config;
        object.vault = vault;
        object
    }

    fn repo(&self) -> Repo {
        Repo::new(&self.config.repo, self.vault.password())
    }


    fn snapshot_list(&self) -> QVariant {
        let repo = self.repo();
        let snapshots = repo.snapshots().unwrap_or_default();
        let latest = snapshots.first().map(|s| s.id.clone()).unwrap_or_default();
        let mut list = QVariantList::default();
        for snapshot in &snapshots {
            list.push(self.snapshot_map(snapshot, snapshot.id == latest));
        }
        QVariant::from(list)
    }

    fn snapshot_map(&self, snapshot: &rustic::Snapshot, latest: bool) -> QVariant {
        let parsed = chrono::DateTime::parse_from_rfc3339(&snapshot.time)
            .ok()
            .map(|time| time.with_timezone(&chrono::Local));
        let (day_key, date_text, time_text) = match parsed {
            Some(time) => (
                time.format("%Y-%m-%d").to_string(),
                time.format("%d.%m.%Y").to_string(),
                time.format("%H:%M").to_string(),
            ),
            None => (String::new(), String::new(), String::new()),
        };
        map_of(vec![
            ("id", QVariant::from(qstr(&snapshot.id))),
            ("shortId", QVariant::from(qstr(snapshot.short_id()))),
            ("time", QVariant::from(qstr(&snapshot.time))),
            ("dayKey", QVariant::from(qstr(&day_key))),
            ("dateText", QVariant::from(qstr(&date_text))),
            ("timeText", QVariant::from(qstr(&time_text))),
            ("files", QVariant::from(snapshot.files as f64)),
            ("totalBytes", QVariant::from(snapshot.total_bytes as f64)),
            ("addedBytes", QVariant::from(snapshot.added_bytes as f64)),
            ("totalText", QVariant::from(qstr(&format::bytes(snapshot.total_bytes)))),
            ("addedText", QVariant::from(qstr(&format::bytes(snapshot.added_bytes)))),
            ("isLatest", QVariant::from(latest)),
            ("paths", QVariant::from(qstr(&snapshot.paths.join(", ")))),
        ])
    }
}

// ---------------------------------------------------------------- Backup ----

#[derive(Clone, Default)]
pub struct JobStatus {
    pub running: bool,
    pub percent: f64,
    pub state: String,
    pub error: String,
    pub speed: String,
    pub eta: String,
    pub files: u64,
    pub added: u64,
    pub snapshot_id: String,
    pub current: String,
    pub started: Option<std::time::Instant>,
}

fn status_map(status: &JobStatus) -> QVariant {
    map_of(vec![
        ("running", QVariant::from(status.running)),
        ("percent", QVariant::from(status.percent)),
        ("state", QVariant::from(qstr(&status.state))),
        ("error", QVariant::from(qstr(&status.error))),
        ("speed", QVariant::from(qstr(&status.speed))),
        ("eta", QVariant::from(qstr(&status.eta))),
        ("files", QVariant::from(status.files as f64)),
        ("added", QVariant::from(status.added as f64)),
        ("snapshotId", QVariant::from(qstr(&status.snapshot_id))),
        ("current", QVariant::from(qstr(&status.current))),
    ])
}

#[derive(QObject, Default)]
pub struct BackupObject {
    base: qt_base_class!(trait QObject),
    vault: Vault,
    pub config: Config,
    state: Arc<Mutex<JobStatus>>,
    cancel_flag: Arc<AtomicBool>,

    start: qt_method!(fn start(&mut self) { self.begin(); }),
    cancel: qt_method!(fn cancel(&mut self) {
        self.cancel_flag.store(true, Ordering::Relaxed);
        if let Ok(mut status) = self.state.lock() {
            status.state = "cancelling".into();
        }
    }),
    poll: qt_method!(fn poll(&self) -> QVariant {
        let status = self.state.lock().map(|s| s.clone()).unwrap_or_default();
        status_map(&status)
    }),
    takeNewKey: qt_method!(fn takeNewKey(&self) -> QString { self.take_new_key() }),
}

impl BackupObject {
    pub fn new(config: Config, vault: Vault) -> Self {
        let mut object = BackupObject::default();
        object.config = config;
        object.vault = vault;
        object
    }

    fn begin(&mut self) {
        {
            let Ok(mut status) = self.state.lock() else { return };
            if status.running {
                return;
            }
            *status = JobStatus {
                running: true,
                percent: 0.0,
                state: "preparing".into(),
                started: Some(std::time::Instant::now()),
                ..Default::default()
            };
        }
        self.cancel_flag.store(false, Ordering::Relaxed);

        // A brand new repository gets a key that is shown exactly once.
        let repo = Repo::new(&self.config.repo, self.vault.password());
        if !repo.exists() && self.vault.is_locked() {
            let key = config::generate_password();
            self.vault.set_password(key.clone());
            self.vault.set_generated(key);
        }

        let config = self.config.clone();
        let state = self.state.clone();
        let cancel = self.cancel_flag.clone();
        let vault = self.vault.clone();
        std::thread::spawn(move || run_backup(config, vault, state, cancel));
    }

    fn take_new_key(&self) -> QString {
        self.vault
            .take_generated()
            .map(|key| qstr(&key))
            .unwrap_or_default()
    }
}

fn set_status(state: &Arc<Mutex<JobStatus>>, update: impl FnOnce(&mut JobStatus)) {
    if let Ok(mut status) = state.lock() {
        update(&mut status);
    }
}

fn run_backup(config: Config, vault: Vault, state: Arc<Mutex<JobStatus>>, cancel: Arc<AtomicBool>) {
    crate::logln("backup: start");
    let Some(password) = vault.password() else {
        set_status(&state, |s| {
            s.running = false;
            s.state = "failed".into();
            s.error = "repository is locked: enter the key first".into();
        });
        return;
    };
    let repo = Repo::new(&config.repo, Some(password));
    let (sources, missing) = config.existing_sources();
    if sources.is_empty() {
        set_status(&state, |s| {
            s.running = false;
            s.state = "failed".into();
            s.error = "none of the configured sources exist".into();
        });
        return;
    }

    let started = std::time::Instant::now();
    let result = repo.backup(&sources, &config.excludes, &cancel, |update| {
        set_status(&state, |status| {
            status.state = "copying".into();
            status.percent = update.percent * 100.0;
            let elapsed = started.elapsed().as_secs_f64();
            status.speed = format::speed(update.bytes_done, elapsed);
            if update.percent > 0.01 && elapsed > 1.0 {
                let remaining = elapsed * (1.0 - update.percent) / update.percent;
                status.eta = format::duration(remaining as i64);
            }
        });
    });

    match result {
        Ok(summary) => {
            crate::logln(&format!(
                "backup: done {} files, {} added",
                summary.total_files,
                format::bytes(summary.data_added)
            ));
            set_status(&state, |status| {
                status.running = false;
                status.percent = 100.0;
                status.state = "done".into();
                status.files = summary.total_files;
                status.added = summary.data_added;
                status.snapshot_id = summary.snapshot_id.clone();
                status.current.clear();
            });
            let _ = repo.forget_policy(config.keep_hourly, config.keep_daily, config.keep_weekly);
            if config.notifications {
                crate::notify::send(
                    "Backup finished",
                    &format!("{} added", format::bytes(summary.data_added)),
                );
            }
            if !missing.is_empty() {
                set_status(&state, |status| {
                    status.error = format!("skipped: {}", missing.join(", "));
                });
            }
        }
        Err(error) if error == "cancelled" => {
            crate::logln("backup: cancelled");
            set_status(&state, |status| {
                status.running = false;
                status.state = "cancelled".into();
            });
            if config.notifications {
                crate::notify::send("Backup cancelled", "");
            }
        }
        Err(error) => {
            crate::logln(&format!("backup: failed: {error}"));
            set_status(&state, |status| {
                status.running = false;
                status.state = "failed".into();
                status.error = error.clone();
            });
            if config.notifications {
                crate::notify::send("Backup failed", &error);
            }
        }
    }
}

// --------------------------------------------------------------- Restore ----

#[derive(QObject, Default)]
pub struct RestoreObject {
    base: qt_base_class!(trait QObject),
    vault: Vault,
    pub config: Config,
    state: Arc<Mutex<JobStatus>>,
    cancel_flag: Arc<AtomicBool>,

    start: qt_method!(fn start(&mut self, snapshot: QString, paths: QString, destination: QString, policy: QString) {
        self.begin(snapshot.to_string(), paths.to_string(), destination.to_string(), policy.to_string());
    }),
    cancel: qt_method!(fn cancel(&mut self) {
        self.cancel_flag.store(true, Ordering::Relaxed);
        if let Ok(mut status) = self.state.lock() {
            status.state = "cancelling".into();
        }
    }),
    poll: qt_method!(fn poll(&self) -> QVariant {
        let status = self.state.lock().map(|s| s.clone()).unwrap_or_default();
        status_map(&status)
    }),
}

impl RestoreObject {
    pub fn new(config: Config, vault: Vault) -> Self {
        let mut object = RestoreObject::default();
        object.config = config;
        object.vault = vault;
        object
    }

    fn begin(&mut self, snapshot: String, paths: String, destination: String, policy: String) {
        if self.state.lock().map(|s| s.running).unwrap_or(false) {
            return;
        }
        let items: Vec<String> = paths
            .lines()
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty())
            .collect();
        if items.is_empty() {
            return;
        }
        {
            let Ok(mut status) = self.state.lock() else { return };
            *status = JobStatus {
                running: true,
                state: "restoring".into(),
                started: Some(std::time::Instant::now()),
                ..Default::default()
            };
        }
        self.cancel_flag.store(false, Ordering::Relaxed);

        let config = self.config.clone();
        let state = self.state.clone();
        let cancel = self.cancel_flag.clone();
        let vault = self.vault.clone();
        std::thread::spawn(move || run_restore(config, vault, snapshot, items, destination, policy, state, cancel));
    }
}

fn run_restore(
    config: Config,
    vault: Vault,
    snapshot: String,
    items: Vec<String>,
    destination: String,
    policy: String,
    state: Arc<Mutex<JobStatus>>,
    cancel: Arc<AtomicBool>,
) {
    let Some(password) = vault.password() else {
        set_status(&state, |s| {
            s.running = false;
            s.state = "failed".into();
            s.error = "repository is locked: enter the key first".into();
        });
        return;
    };
    let repo = Repo::new(&config.repo, Some(password));
    let destination = if destination.is_empty() {
        "/".to_string()
    } else {
        destination
    };
    let total = items.len() as f64;
    let mut restored = 0u64;

    for (index, relative) in items.iter().enumerate() {
        if cancel.load(Ordering::Relaxed) {
            set_status(&state, |s| {
                s.running = false;
                s.state = "cancelled".into();
            });
            return;
        }
        set_status(&state, |s| s.current = relative.clone());

        let staging = paths::staging_root().join(format!("{}-{}", std::process::id(), index));
        let _ = std::fs::remove_dir_all(&staging);
        if let Err(error) = std::fs::create_dir_all(&staging) {
            set_status(&state, |s| {
                s.running = false;
                s.state = "failed".into();
                s.error = error.to_string();
            });
            return;
        }

        let base = index as f64;
        let result = repo.restore(&snapshot, relative, &staging, &cancel, |update| {
            set_status(&state, |s| {
                s.percent = (base + update.percent) / total * 100.0;
            });
        });

        if let Err(error) = result {
            let _ = std::fs::remove_dir_all(&staging);
            if error == "cancelled" {
                set_status(&state, |s| {
                    s.running = false;
                    s.state = "cancelled".into();
                });
            } else {
                set_status(&state, |s| {
                    s.running = false;
                    s.state = "failed".into();
                    s.error = error;
                });
            }
            return;
        }

        if let Err(error) = place_item(&staging, relative, &destination, &policy) {
            let _ = std::fs::remove_dir_all(&staging);
            set_status(&state, |s| {
                s.running = false;
                s.state = "failed".into();
                s.error = error;
            });
            return;
        }
        let _ = std::fs::remove_dir_all(&staging);
        restored += 1;
        set_status(&state, |s| s.percent = (index as f64 + 1.0) / total * 100.0);
    }

    set_status(&state, |s| {
        s.running = false;
        s.state = "done".into();
        s.percent = 100.0;
        s.added = restored;
        s.current.clear();
    });
    if config.notifications {
        crate::notify::send("Restore finished", &format!("{restored} item(s) restored"));
    }
}

fn unique_target(target: &Path) -> PathBuf {
    let stamp = chrono::Local::now().format("%Y-%m-%d %H-%M-%S");
    let parent = target.parent().unwrap_or_else(|| Path::new("/"));
    let stem = target
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "item".to_string());
    let extension = target
        .extension()
        .map(|s| format!(".{}", s.to_string_lossy()))
        .unwrap_or_default();
    parent.join(format!("{stem} (restored {stamp}){extension}"))
}

fn copy_tree(source: &Path, target: &Path) -> Result<(), String> {
    let status = std::process::Command::new("cp")
        .arg("-a")
        .arg(format!("{}/.", source.display()))
        .arg(target)
        .status()
        .map_err(|error| error.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("cannot copy {} to {}", source.display(), target.display()))
    }
}

fn copy_item(source: &Path, target: &Path) -> Result<(), String> {
    let status = std::process::Command::new("cp")
        .arg("-a")
        .arg(source)
        .arg(target)
        .status()
        .map_err(|error| error.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("cannot copy {} to {}", source.display(), target.display()))
    }
}

pub fn place_item(staging: &Path, relative: &str, destination: &str, policy: &str) -> Result<(), String> {
    let target = if destination == "/" {
        PathBuf::from("/").join(relative)
    } else {
        PathBuf::from(destination).join(relative)
    };
    let staged_name = Path::new(relative)
        .file_name()
        .map(|name| name.to_os_string())
        .unwrap_or_default();
    let staged_file = staging.join(&staged_name);
    let is_file = staged_file.is_file();

    let mut final_target = target.clone();
    if final_target.exists() {
        match policy {
            "skip" => return Ok(()),
            "keep_both" => final_target = unique_target(&target),
            _ => {}
        }
    }

    if is_file {
        if let Some(parent) = final_target.parent() {
            std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        copy_item(&staged_file, &final_target)
    } else {
        std::fs::create_dir_all(&final_target).map_err(|error| error.to_string())?;
        copy_tree(staging, &final_target)
    }
}

// --------------------------------------------------------------- Browser ----

#[derive(QObject, Default)]
pub struct BrowserObject {
    base: qt_base_class!(trait QObject),
    vault: Vault,
    pub config: Config,
    snapshot_id: String,
    entries_cache: Vec<rustic::Entry>,
    current_path: String,

    setSnapshot: qt_method!(fn setSnapshot(&mut self, id: QString) { self.load(id.to_string()); }),
    entries: qt_method!(fn entries(&self) -> QVariant { self.entry_list() }),
    setPath: qt_method!(fn setPath(&mut self, path: QString) { self.jump(path.to_string()); }),
    goUp: qt_method!(fn goUp(&mut self) { self.up(); }),
    canGoUp: qt_method!(fn canGoUp(&self) -> bool { !self.current_path.is_empty() }),
    path: qt_method!(fn path(&self) -> QString { qstr(&self.current_path) }),
    displayPath: qt_method!(fn displayPath(&self) -> QString { qstr(&self.pretty_path()) }),
    homeRoot: qt_method!(fn homeRoot(&self) -> QString { qstr(&self.home_prefix()) }),
    fileCount: qt_method!(fn fileCount(&self) -> i32 { self.entries_cache.len() as i32 }),
}

impl BrowserObject {
    pub fn new(config: Config, vault: Vault) -> Self {
        let mut object = BrowserObject::default();
        object.config = config;
        object.vault = vault;
        object
    }

    fn home_prefix(&self) -> String {
        let user = std::env::var("USER")
            .or_else(|_| std::env::var("LOGNAME"))
            .unwrap_or_default();
        if user.is_empty() {
            return String::new();
        }
        let candidate = format!("home/{user}");
        if self.entries_cache.iter().any(|entry| entry.path == candidate) {
            candidate
        } else {
            String::new()
        }
    }

    fn load(&mut self, id: String) {
        if id == self.snapshot_id && !self.entries_cache.is_empty() {
            return;
        }
        self.snapshot_id = id;
        self.entries_cache.clear();
        self.current_path.clear();
        if self.snapshot_id.is_empty() {
            return;
        }
        let repo = Repo::new(&self.config.repo, self.vault.password());
        match repo.ls(&self.snapshot_id) {
            Ok(entries) => {
                self.entries_cache = entries;
                self.current_path = self.home_prefix();
            }
            Err(error) => {
                crate::logln(&format!("browser: cannot list snapshot: {error}"));
            }
        }
    }

    fn jump(&mut self, path: String) {
        let cleaned = path.trim_matches('/').to_string();
        if cleaned.contains("..") {
            return;
        }
        if self.entries_cache.iter().any(|entry| entry.path == cleaned && entry.is_dir) {
            self.current_path = cleaned;
        }
    }

    fn up(&mut self) {
        if self.current_path.is_empty() {
            return;
        }
        match self.current_path.rfind('/') {
            Some(index) => self.current_path.truncate(index),
            None => self.current_path.clear(),
        }
    }

    fn pretty_path(&self) -> String {
        let home = self.home_prefix();
        if self.current_path.is_empty() {
            return "/".to_string();
        }
        if self.current_path == home {
            return "~".to_string();
        }
        if !home.is_empty() && self.current_path.starts_with(&(home.clone() + "/")) {
            return format!("~{}", &self.current_path[home.len()..]);
        }
        self.current_path.clone()
    }

    fn entry_list(&self) -> QVariant {
        let children = rustic::children_of(&self.entries_cache, &self.current_path);
        let mut list = QVariantList::default();
        for entry in children {
            let name = entry
                .path
                .rsplit('/')
                .next()
                .unwrap_or(&entry.path)
                .to_string();
            let modified = chrono::DateTime::parse_from_rfc3339(&entry.mtime)
                .ok()
                .map(|time| time.with_timezone(&chrono::Local).format("%d.%m.%Y %H:%M").to_string())
                .unwrap_or_default();
            list.push(map_of(vec![
                ("name", QVariant::from(qstr(&name))),
                ("relativePath", QVariant::from(qstr(&entry.path))),
                ("isDir", QVariant::from(entry.is_dir)),
                ("isLink", QVariant::from(entry.is_link)),
                ("size", QVariant::from(entry.size as f64)),
                ("sizeText", QVariant::from(qstr(&format::bytes(entry.size)))),
                ("modifiedText", QVariant::from(qstr(&modified))),
            ]));
        }
        QVariant::from(list)
    }
}

// ------------------------------------------------------------- Scheduler ----

#[derive(QObject, Default)]
pub struct SchedulerObject {
    base: qt_base_class!(trait QObject),
    vault: Vault,
    pub config: Config,
    last_error: Arc<Mutex<String>>,

    apply: qt_method!(fn apply(&mut self) { self.sync(); }),
    poll: qt_method!(fn poll(&self) -> QVariant {
        let state = schedule::state();
        let error = self.last_error.lock().map(|s| s.clone()).unwrap_or_default();
        map_of(vec![
            ("enabled", QVariant::from(self.config.schedule_enabled)),
            ("state", QVariant::from(qstr(&state))),
            ("error", QVariant::from(qstr(&error))),
            ("minutes", QVariant::from(self.config.schedule_minutes as i32)),
        ])
    }),
    setEnabled: qt_method!(fn setEnabled(&mut self, value: bool) {
        self.config.schedule_enabled = value;
        let _ = self.config.save();
        self.sync();
    }),
}

impl SchedulerObject {
    pub fn new(config: Config, vault: Vault) -> Self {
        let mut object = SchedulerObject::default();
        object.config = config;
        object.vault = vault;
        object
    }

    fn sync(&mut self) {
        let result = if self.config.schedule_enabled {
            schedule::install(self.config.schedule_minutes)
        } else {
            schedule::uninstall()
        };
        let mut error = self.last_error.lock().unwrap_or_else(|e| e.into_inner());
        *error = result.err().unwrap_or_default();
    }
}

// ------------------------------------------------------------------- App ----

#[derive(QObject, Default)]
pub struct AppObject {
    base: qt_base_class!(trait QObject),
    vault: Vault,
    pub config: Config,

    version: qt_method!(fn version(&self) -> QString {
        qstr(env!("CARGO_PKG_VERSION"))
    }),
    configPath: qt_method!(fn configPath(&self) -> QString {
        qstr(&paths::config_file().to_string_lossy())
    }),
    logPath: qt_method!(fn logPath(&self) -> QString {
        qstr(&paths::log_file().to_string_lossy())
    }),
    homePath: qt_method!(fn homePath(&self) -> QString {
        qstr(&paths::home().to_string_lossy())
    }),
    formatBytes: qt_method!(fn formatBytes(&self, value: f64) -> QString {
        qstr(&format::bytes(value.max(0.0) as u64))
    }),
    openPath: qt_method!(fn openPath(&self, path: QString) {
        let path = path.to_string();
        let target = if Path::new(&path).is_dir() {
            path
        } else {
            Path::new(&path)
                .parent()
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or(path.clone())
        };
        let _ = std::process::Command::new("xdg-open").arg(target).spawn();
    }),
    copyText: qt_method!(fn copyText(&self, text: QString) {
        crate::i18n::set_clipboard(&text.to_string());
    }),
    quit: qt_method!(fn quit(&self) {
        crate::i18n::quit();
    }),
    trace: qt_method!(fn trace(&self, message: QString) {
        crate::logln(&format!("qml: {}", message.to_string()));
    }),
}

impl AppObject {
    pub fn new(config: Config, vault: Vault) -> Self {
        let mut object = AppObject::default();
        object.config = config;
        object.vault = vault;
        object
    }
}
