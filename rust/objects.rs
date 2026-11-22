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
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use crate::config::{self, BackupRule, Config};
use crate::format;
use crate::joblock::JobLock;
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

fn list_of_ints(values: &[u32]) -> QVariant {
    let mut list = QVariantList::default();
    for value in values {
        list.push(QVariant::from(*value as i32));
    }
    QVariant::from(list)
}

fn rule_map(rule: &BackupRule) -> QVariant {
    map_of(vec![
        ("id", QVariant::from(qstr(&rule.id))),
        ("name", QVariant::from(qstr(&rule.name))),
        ("enabled", QVariant::from(rule.enabled)),
        ("sources", list_of_strings(&rule.sources)),
        ("excludes", list_of_strings(&rule.excludes)),
        ("oneFileSystem", QVariant::from(rule.one_file_system)),
        ("excludeLarger", QVariant::from(qstr(&rule.exclude_larger))),
        ("passwordSet", QVariant::from(rule.has_password())),
        (
            "schedule",
            map_of(vec![
                ("enabled", QVariant::from(rule.schedule.enabled)),
                ("mode", QVariant::from(qstr(&rule.schedule.mode))),
                ("minutes", QVariant::from(rule.schedule.minutes as i32)),
                ("time", QVariant::from(qstr(&rule.schedule.time))),
                ("weekdays", list_of_ints(&rule.schedule.weekdays)),
            ]),
        ),
    ])
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
    keep_hourly: qt_property!(i32; NOTIFY retention_changed),
    keep_daily: qt_property!(i32; NOTIFY retention_changed),
    keep_weekly: qt_property!(i32; NOTIFY retention_changed),
    retention_changed: qt_signal!(),
    lockTimeoutMinutes: qt_property!(i32; NOTIFY lock_timeout_changed),
    lock_timeout_changed: qt_signal!(),
    passwordMode: qt_property!(QString; NOTIFY password_mode_changed),
    password_mode_changed: qt_signal!(),
    commonPasswordSet: qt_property!(bool; NOTIFY common_password_changed),
    common_password_changed: qt_signal!(),

    setLanguage: qt_method!(fn setLanguage(&mut self, value: QString) {
        self.apply_language(value.to_string());
    }),
    setRepo: qt_method!(fn setRepo(&mut self, value: QString) {
        let new_repo = value.to_string().trim_end_matches('/').to_string();
        if new_repo == self.config.repo {
            self.repo = qstr(&self.config.repo);
            return;
        }
        let old_rules: Vec<String> = self.config.rules.iter().map(|rule| rule.id.clone()).collect();
        self.config.set_repo(new_repo);
        if self.config.adopt_repository_fingerprint() {
            let _ = self.config.save();
        }
        for rule_id in &old_rules {
            schedule::remove_credential(rule_id);
        }
        // Passwords belong to the old repository.
        self.vault.lock_all();
        let _ = schedule::sync(&self.config.rules);
        self.repo = qstr(&self.config.repo);
        self.commonPasswordSet = false;
        self.common_password_changed();
        self.repo_changed();
        self.rules_changed();
    }),
    setNotifications: qt_method!(fn setNotifications(&mut self, value: bool) {
        self.config.notifications = value;
        let _ = self.config.save();
        self.notifications = value;
        self.notifications_changed();
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
    setLockTimeoutMinutes: qt_method!(fn setLockTimeoutMinutes(&mut self, value: i32) {
        self.config.set_lock_timeout_minutes(value.max(0) as u32);
        self.vault.set_timeout_minutes(self.config.lock_timeout_minutes);
        self.lockTimeoutMinutes = self.config.lock_timeout_minutes as i32;
        self.lock_timeout_changed();
    }),
    setPasswordMode: qt_method!(fn setPasswordMode(&mut self, value: QString) {
        self.config.set_password_mode(&value.to_string());
        self.passwordMode = qstr(&self.config.password_mode);
        self.password_mode_changed();
        self.rules_changed();
    }),
    /// Binds the common password's key to every rule and remembers it as the
    /// canonical common key.
    bindCommonKey: qt_method!(fn bindCommonKey(&mut self, key_id: QString) {
        self.config.bind_common_key(key_id.to_string());
        self.commonPasswordSet = !self.config.common_key_id.is_empty();
        self.passwordMode = qstr(&self.config.password_mode);
        self.common_password_changed();
        self.rules_changed();
    }),

    // ---- backup rules -----------------------------------------------------
    rules: qt_method!(fn rules(&self) -> QVariant {
        let mut list = QVariantList::default();
        for rule in &self.config.rules {
            list.push(rule_map(rule));
        }
        QVariant::from(list)
    }),
    addRule: qt_method!(fn addRule(&mut self, name: QString) -> QString {
        let id = self.config.add_rule(name.to_string());
        self.rules_changed();
        qstr(&id)
    }),
    removeRule: qt_method!(fn removeRule(&mut self, id: QString) {
        let id = id.to_string();
        let key_id = self
            .config
            .rule(&id)
            .map(|rule| rule.key_id.clone())
            .unwrap_or_default();
        // Keep a password for the revocation below: the deleted rule's own
        // password is enough to open the repository and remove its key.
        let password = self.vault.password_for(&id).or_else(|| self.vault.any_password());
        self.config.remove_rule(&id);
        self.vault.forget_rule(&id);
        schedule::remove_credential(&id);
        if !key_id.is_empty() {
            // Best effort: revoke the rule's password. Keep the last key so
            // the repository never becomes inaccessible.
            let repo_path = self.config.repo.clone();
            std::thread::spawn(move || {
                let Some(password) = password else { return };
                let repo = Repo::new(&repo_path, Some(password));
                match repo.key_list() {
                    Ok(keys) if keys.len() > 1 => {
                        if let Err(error) = repo.remove_key(&key_id) {
                            crate::logln(&format!("store: cannot remove key {key_id}: {error}"));
                        }
                    }
                    Ok(_) => {}
                    Err(error) => crate::logln(&format!("store: cannot list keys: {error}")),
                }
            });
        }
        self.rules_changed();
    }),
    setRuleName: qt_method!(fn setRuleName(&mut self, id: QString, name: QString) {
        self.config.set_rule_name(&id.to_string(), name.to_string());
        self.rules_changed();
    }),
    setRuleOneFileSystem: qt_method!(fn setRuleOneFileSystem(&mut self, id: QString, value: bool) {
        self.config.set_rule_one_file_system(&id.to_string(), value);
        self.rules_changed();
    }),
    setRuleExcludeLarger: qt_method!(fn setRuleExcludeLarger(&mut self, id: QString, value: QString) {
        self.config.set_rule_exclude_larger(&id.to_string(), value.to_string());
        self.rules_changed();
    }),
    setRuleEnabled: qt_method!(fn setRuleEnabled(&mut self, id: QString, enabled: bool) {
        self.config.set_rule_enabled(&id.to_string(), enabled);
        self.rules_changed();
    }),
    addRuleSource: qt_method!(fn addRuleSource(&mut self, id: QString, path: QString) {
        self.config.add_rule_source(&id.to_string(), path.to_string());
        self.rules_changed();
    }),
    removeRuleSource: qt_method!(fn removeRuleSource(&mut self, id: QString, index: i32) {
        self.config.remove_rule_source(&id.to_string(), index as usize);
        self.rules_changed();
    }),
    addRuleExclude: qt_method!(fn addRuleExclude(&mut self, id: QString, pattern: QString) {
        self.config.add_rule_exclude(&id.to_string(), pattern.to_string());
        self.rules_changed();
    }),
    addRecommendedExcludes: qt_method!(fn addRecommendedExcludes(&mut self, id: QString) -> i32 {
        let added = self.config.add_recommended_excludes(&id.to_string());
        if added > 0 {
            self.rules_changed();
        }
        added
    }),
    removeRuleExclude: qt_method!(fn removeRuleExclude(&mut self, id: QString, index: i32) {
        self.config.remove_rule_exclude(&id.to_string(), index as usize);
        self.rules_changed();
    }),
    setRuleScheduleEnabled: qt_method!(fn setRuleScheduleEnabled(&mut self, id: QString, enabled: bool) {
        self.config.set_rule_schedule_enabled(&id.to_string(), enabled);
        self.rules_changed();
    }),
    setRuleScheduleMode: qt_method!(fn setRuleScheduleMode(&mut self, id: QString, mode: QString) {
        self.config.set_rule_schedule_mode(&id.to_string(), mode.to_string());
        self.rules_changed();
    }),
    setRuleScheduleMinutes: qt_method!(fn setRuleScheduleMinutes(&mut self, id: QString, minutes: i32) {
        self.config.set_rule_schedule_minutes(&id.to_string(), minutes);
        self.rules_changed();
    }),
    setRuleScheduleTime: qt_method!(fn setRuleScheduleTime(&mut self, id: QString, time: QString) {
        self.config.set_rule_schedule_time(&id.to_string(), time.to_string());
        self.rules_changed();
    }),
    setRuleWeekday: qt_method!(fn setRuleWeekday(&mut self, id: QString, day: i32, enabled: bool) {
        self.config.set_rule_weekday(&id.to_string(), day, enabled);
        self.rules_changed();
    }),
    /// Stores the rustic key ID returned when a rule's password is created or
    /// changed. QML calls this after `Store` reports a successful password
    /// operation.
    bindRuleKey: qt_method!(fn bindRuleKey(&mut self, id: QString, key_id: QString) {
        self.config.set_rule_key_id(&id.to_string(), key_id.to_string());
        self.rules_changed();
    }),
    reset: qt_method!(fn reset(&mut self) {
        for rule in &self.config.rules {
            schedule::remove_credential(&rule.id);
        }
        self.vault.lock_all();
        self.config = Config::default();
        let _ = self.config.save_fresh();
        self.vault.set_timeout_minutes(self.config.lock_timeout_minutes);
        self.reload_properties();
        self.language_changed();
        self.repo_changed();
        self.notifications_changed();
        self.retention_changed();
        self.lock_timeout_changed();
        self.password_mode_changed();
        self.common_password_changed();
        self.rules_changed();
    }),

    rules_changed: qt_signal!(),
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
        self.keep_hourly = self.config.keep_hourly as i32;
        self.keep_daily = self.config.keep_daily as i32;
        self.keep_weekly = self.config.keep_weekly as i32;
        self.lockTimeoutMinutes = self.config.lock_timeout_minutes as i32;
        self.passwordMode = qstr(&self.config.password_mode);
        self.commonPasswordSet = !self.config.common_key_id.is_empty();
        self.vault.set_timeout_minutes(self.config.lock_timeout_minutes);
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

#[derive(Default)]
struct SnapshotCache {
    /// Bumped when the cache is cleared; a listing started before that must
    /// not put its result back after the vault was locked.
    generation: u64,
    loading: bool,
    error: String,
    items: Vec<rustic::Snapshot>,
}

/// Progress of a password dialog: unlocking one backup, creating its first
/// password or replacing an existing one.
#[derive(Default)]
struct PasswordState {
    pending: bool,
    succeeded: bool,
    error: String,
    warning: String,
    rule_id: String,
    /// Key ID returned by rustic; QML stores it on the rule(s).
    key_id: String,
    bind: bool,
    /// The operation changed the common password and QML must bind the key
    /// to every rule.
    bind_all: bool,
}

#[derive(QObject, Default)]
pub struct StoreObject {
    base: qt_base_class!(trait QObject),
    vault: Vault,
    pub config: Config,
    cache: Arc<Mutex<SnapshotCache>>,
    password_state: Arc<Mutex<PasswordState>>,

    snapshots: qt_method!(fn snapshots(&self) -> QVariant { self.snapshot_list() }),
    snapshotsLoading: qt_method!(fn snapshotsLoading(&self) -> bool {
        self.cache.lock().map(|cache| cache.loading).unwrap_or(false)
    }),
    snapshotsError: qt_method!(fn snapshotsError(&self) -> QString {
        self.cache
            .lock()
            .map(|cache| qstr(&cache.error))
            .unwrap_or_default()
    }),
    refreshSnapshots: qt_method!(fn refreshSnapshots(&mut self) { self.begin_refresh(); }),
    reloadConfig: qt_method!(fn reloadConfig(&mut self) { self.config = Config::load(); }),
    clearCache: qt_method!(fn clearCache(&mut self) {
        let mut cache = self.cache.lock().unwrap_or_else(|error| error.into_inner());
        cache.generation = cache.generation.wrapping_add(1);
        cache.loading = false;
        cache.items.clear();
        cache.error.clear();
    }),
    freeBytes: qt_method!(fn freeBytes(&self) -> f64 { self.repo().free_bytes() as f64 }),
    repoPath: qt_method!(fn repoPath(&self) -> QString { qstr(&self.config.repo) }),
    /// True when this configuration has seen a repository at `repo` before,
    /// but the path is gone now (typically an unmounted drive).
    repoMissing: qt_method!(fn repoMissing(&self) -> bool {
        !Repo::new(&self.config.repo, None).exists() && !self.config.repo_fingerprint.is_empty()
    }),
    status: qt_method!(fn status(&self) -> QVariant {
        let has_repo = Repo::new(&self.config.repo, None).exists();
        let unlocked = self.vault.unlocked_rules();
        map_of(vec![
            ("hasRepo", QVariant::from(has_repo)),
            ("locked", QVariant::from(self.vault.is_empty())),
            ("anyUnlocked", QVariant::from(!self.vault.is_empty())),
            ("unlockedRules", list_of_strings(&unlocked)),
            ("generation", QVariant::from(self.vault.generation() as f64)),
            ("error", QVariant::from(qstr(""))),
        ])
    }),
    isRuleUnlocked: qt_method!(fn isRuleUnlocked(&self, rule_id: QString) -> bool {
        self.vault.is_unlocked(&rule_id.to_string())
    }),
    anyUnlocked: qt_method!(fn anyUnlocked(&self) -> bool { !self.vault.is_empty() }),
    lockGeneration: qt_method!(fn lockGeneration(&self) -> f64 { self.vault.generation() as f64 }),
    lockNow: qt_method!(fn lockNow(&mut self) -> bool {
        let locked = self.vault.lock_all();
        if locked {
            self.clear_cache();
        }
        locked
    }),
    unlockRule: qt_method!(fn unlockRule(&mut self, rule_id: QString, password: QString) {
        self.begin_unlock_rule(rule_id.to_string(), password.to_string());
    }),
    setRulePassword: qt_method!(fn setRulePassword(&mut self, rule_id: QString, auth: QString, password: QString) {
        self.begin_set_rule_password(rule_id.to_string(), auth.to_string(), password.to_string());
    }),
    setCommonPassword: qt_method!(fn setCommonPassword(&mut self, auth: QString, password: QString) {
        self.begin_set_common_password(auth.to_string(), password.to_string());
    }),
    passwordPending: qt_method!(fn passwordPending(&self) -> bool {
        self.password_state.lock().map(|state| state.pending).unwrap_or(false)
    }),
    passwordSucceeded: qt_method!(fn passwordSucceeded(&self) -> bool {
        self.password_state.lock().map(|state| state.succeeded).unwrap_or(false)
    }),
    passwordError: qt_method!(fn passwordError(&self) -> QString {
        self.password_state
            .lock()
            .map(|state| qstr(&state.error))
            .unwrap_or_default()
    }),
    passwordWarning: qt_method!(fn passwordWarning(&self) -> QString {
        self.password_state
            .lock()
            .map(|state| qstr(&state.warning))
            .unwrap_or_default()
    }),
    passwordRuleId: qt_method!(fn passwordRuleId(&self) -> QString {
        self.password_state
            .lock()
            .map(|state| qstr(&state.rule_id))
            .unwrap_or_default()
    }),
    passwordKeyId: qt_method!(fn passwordKeyId(&self) -> QString {
        self.password_state
            .lock()
            .map(|state| qstr(&state.key_id))
            .unwrap_or_default()
    }),
    passwordNeedsBind: qt_method!(fn passwordNeedsBind(&self) -> bool {
        self.password_state.lock().map(|state| state.bind).unwrap_or(false)
    }),
    passwordBindsAll: qt_method!(fn passwordBindsAll(&self) -> bool {
        self.password_state.lock().map(|state| state.bind_all).unwrap_or(false)
    }),
    ruleCanRunUnattended: qt_method!(fn ruleCanRunUnattended(&self, rule_id: QString) -> bool {
        schedule::credential_kind(&rule_id.to_string()).is_some()
    }),
    /// True when no typed password is needed to start a backup: the GUI can
    /// use a saved credential (or initialize a new repository with a generated
    /// key). Only an old repository with no saved credential needs one prompt.
    backupNeedsPassword: qt_method!(fn backupNeedsPassword(&self) -> bool {
        if !self.vault.is_empty() {
            return false;
        }
        if !Repo::new(&self.config.repo, None).exists() {
            return false;
        }
        load_any_credential(&self.config, None).is_none()
    }),
    /// Whether an existing saved credential can authorize creating a viewing
    /// password for a rule that does not have one yet.
    canAuthorizeWithCredential: qt_method!(fn canAuthorizeWithCredential(&self) -> bool {
        Repo::new(&self.config.repo, None).exists()
            && load_any_credential(&self.config, None).is_some()
    }),
    /// Returns the repository password saved for unattended backups, so the
    /// user can write it down. Empty when no credential can be decrypted.
    recoveryKey: qt_method!(fn recoveryKey(&self) -> QString {
        load_any_credential(&self.config, None)
            .map(|password| qstr(&password))
            .unwrap_or_default()
    }),
    setPinned: qt_method!(fn setPinned(&mut self, id: QString, pinned: bool) -> bool {
        let repo = self.repo();
        match repo.set_pinned(&id.to_string(), pinned) {
            Ok(()) => {
                self.begin_refresh();
                true
            }
            Err(error) => {
                crate::logln(&format!("store: cannot pin snapshot: {error}"));
                false
            }
        }
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
    /// Dry-run of the retention policy: which snapshots would be deleted.
    prunePreview: qt_method!(fn prunePreview(&self) -> QVariant {
        let repo = self.repo();
        let result = repo.forget_preview(
            self.config.keep_hourly,
            self.config.keep_daily,
            self.config.keep_weekly,
        );
        match result {
            Ok(snapshots) => {
                let mut items = QVariantList::default();
                for snapshot in snapshots {
                    let (date_text, time_text) = chrono::DateTime::parse_from_rfc3339(&snapshot.time)
                        .ok()
                        .map(|time| {
                            let time = time.with_timezone(&chrono::Local);
                            (
                                time.format("%d.%m.%Y").to_string(),
                                time.format("%H:%M").to_string(),
                            )
                        })
                        .unwrap_or_default();
                    let rule_name = snapshot
                        .rule_id()
                        .and_then(|id| self.config.rule(id))
                        .map(|rule| {
                            if rule.name.trim().is_empty() {
                                rule.id.clone()
                            } else {
                                rule.name.clone()
                            }
                        })
                        .unwrap_or_default();
                    items.push(map_of(vec![
                        ("id", QVariant::from(qstr(&snapshot.id))),
                        ("shortId", QVariant::from(qstr(snapshot.short_id()))),
                        ("dateText", QVariant::from(qstr(&date_text))),
                        ("timeText", QVariant::from(qstr(&time_text))),
                        ("ruleName", QVariant::from(qstr(&rule_name))),
                    ]));
                }
                map_of(vec![
                    ("error", QVariant::from(qstr(""))),
                    ("items", QVariant::from(items)),
                ])
            }
            Err(error) => {
                crate::logln(&format!("store: prune preview failed: {error}"));
                map_of(vec![
                    ("error", QVariant::from(qstr(&error))),
                    ("items", QVariant::from(QVariantList::default())),
                ])
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
        Repo::new(&self.config.repo, self.vault.any_password())
    }

    fn clear_cache(&mut self) {
        let mut cache = self.cache.lock().unwrap_or_else(|error| error.into_inner());
        cache.generation = cache.generation.wrapping_add(1);
        cache.loading = false;
        cache.items.clear();
        cache.error.clear();
    }

    fn snapshot_list(&self) -> QVariant {
        let cache = self.cache.lock().unwrap_or_else(|error| error.into_inner());
        let snapshots = &cache.items;
        let latest = snapshots.first().map(|s| s.id.clone()).unwrap_or_default();
        let mut list = QVariantList::default();
        for snapshot in snapshots {
            list.push(self.snapshot_map(snapshot, snapshot.id == latest));
        }
        QVariant::from(list)
    }

    /// Starts `rustic snapshots` on a worker thread. QML polls
    /// `snapshotsLoading()` and then reads the cached list, so the window never
    /// freezes while the repository index is read.
    fn begin_refresh(&mut self) {
        self.config = Config::load();
        let Some(password) = self.vault.any_password() else {
            return;
        };
        let generation = {
            let mut cache = self.cache.lock().unwrap_or_else(|error| error.into_inner());
            if cache.loading {
                return;
            }
            cache.loading = true;
            cache.error.clear();
            cache.generation = cache.generation.wrapping_add(1);
            cache.generation
        };

        let repo_path = self.config.repo.clone();
        let cache = self.cache.clone();
        std::thread::spawn(move || {
            let repo = Repo::new(&repo_path, Some(password));
            let result = repo.snapshots();
            let mut cache = cache.lock().unwrap_or_else(|error| error.into_inner());
            if cache.generation != generation {
                // The vault was locked (or the cache cleared) while the list
                // was being read.
                return;
            }
            cache.loading = false;
            match result {
                Ok(items) => {
                    cache.items = items;
                }
                Err(error) => {
                    crate::logln(&format!("store: cannot list snapshots: {error}"));
                    cache.error = error;
                }
            }
        });
    }

    /// Verifies one rule's password (its rustic key) in the background and
    /// installs it into the vault only on success.
    fn begin_unlock_rule(&mut self, rule_id: String, password: String) {
        if rule_id.is_empty() || password.is_empty() {
            return;
        }
        self.config = Config::load();
        if !self.reset_password_state() {
            return;
        }
        let config = self.config.clone();
        let vault = self.vault.clone();
        let state = self.password_state.clone();
        std::thread::spawn(move || {
            let result = unlock_rule(&config, &vault, &rule_id, &password);
            finish_password_state(&state, &rule_id, result);
        });
    }

    /// Sets or replaces the one password shared by every rule.
    fn begin_set_common_password(&mut self, auth: String, password: String) {
        if password.is_empty() {
            return;
        }
        self.config = Config::load();
        if !self.reset_password_state() {
            return;
        }
        let config = self.config.clone();
        let vault = self.vault.clone();
        let state = self.password_state.clone();
        std::thread::spawn(move || {
            let result = set_common_password(&config, &vault, &auth, &password);
            finish_password_state(&state, "", result);
        });
    }

    /// Creates the first password of a rule or replaces its current one.
    fn begin_set_rule_password(&mut self, rule_id: String, auth: String, password: String) {
        if rule_id.is_empty() || password.is_empty() {
            return;
        }
        self.config = Config::load();
        if !self.reset_password_state() {
            return;
        }
        let config = self.config.clone();
        let vault = self.vault.clone();
        let state = self.password_state.clone();
        std::thread::spawn(move || {
            let result = set_rule_password(&config, &vault, &rule_id, &auth, &password);
            finish_password_state(&state, &rule_id, result);
        });
    }

    /// Clears the previous result and marks a new operation as pending.
    /// Returns false when another operation is still running.
    fn reset_password_state(&mut self) -> bool {
        let mut state = self
            .password_state
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if state.pending {
            return false;
        }
        *state = PasswordState {
            pending: true,
            ..Default::default()
        };
        true
    }

    fn snapshot_map(&self, snapshot: &rustic::Snapshot, latest: bool) -> QVariant {
        let parsed = chrono::DateTime::parse_from_rfc3339(&snapshot.time)
            .ok()
            .map(|time| time.with_timezone(&chrono::Local));
        let epoch = parsed.as_ref().map(|time| time.timestamp() as f64).unwrap_or(0.0);
        let (day_key, date_text, time_text) = match parsed {
            Some(time) => (
                time.format("%Y-%m-%d").to_string(),
                time.format("%d.%m.%Y").to_string(),
                time.format("%H:%M").to_string(),
            ),
            None => (String::new(), String::new(), String::new()),
        };
        let rule_id = snapshot.rule_id().unwrap_or_default().to_string();
        let rule_name = self
            .config
            .rule(&rule_id)
            .map(|rule| rule.name.clone())
            .unwrap_or_default();
        // Older summaries may not carry the packed size; fall back to the raw
        // one so the timeline never shows a misleading zero.
        let added_packed = if snapshot.added_packed_bytes == 0 && snapshot.added_bytes != 0 {
            snapshot.added_bytes
        } else {
            snapshot.added_packed_bytes
        };
        map_of(vec![
            ("id", QVariant::from(qstr(&snapshot.id))),
            ("shortId", QVariant::from(qstr(snapshot.short_id()))),
            ("time", QVariant::from(qstr(&snapshot.time))),
            ("dayKey", QVariant::from(qstr(&day_key))),
            ("dateText", QVariant::from(qstr(&date_text))),
            ("timeText", QVariant::from(qstr(&time_text))),
            ("epoch", QVariant::from(epoch)),
            ("files", QVariant::from(snapshot.files as f64)),
            ("totalBytes", QVariant::from(snapshot.total_bytes as f64)),
            ("addedBytes", QVariant::from(snapshot.added_bytes as f64)),
            ("addedPackedBytes", QVariant::from(added_packed as f64)),
            ("totalText", QVariant::from(qstr(&format::bytes(snapshot.total_bytes)))),
            ("addedText", QVariant::from(qstr(&format::bytes(snapshot.added_bytes)))),
            ("addedPackedText", QVariant::from(qstr(&format::bytes(added_packed)))),
            ("filesNew", QVariant::from(snapshot.files_new as f64)),
            ("filesChanged", QVariant::from(snapshot.files_changed as f64)),
            ("durationText", QVariant::from(qstr(&format::duration(
                snapshot.duration as i64
            )))),
            ("isLatest", QVariant::from(latest)),
            ("pinned", QVariant::from(snapshot.pinned())),
            ("paths", QVariant::from(qstr(&snapshot.paths.join(", ")))),
            ("ruleId", QVariant::from(qstr(&rule_id))),
            ("ruleName", QVariant::from(qstr(&rule_name))),
        ])
    }
}

#[derive(Debug)]
struct PasswordSuccess {
    key_id: String,
    bind: bool,
    bind_all: bool,
    warning: String,
}

/// Finds a saved credential that opens the repository, preferring one rule.
/// Every credential is a repository key, so any of them can start any backup.
fn load_any_credential(config: &Config, preferred: Option<&str>) -> Option<String> {
    let repo_exists = Repo::new(&config.repo, None).exists();
    let mut candidates: Vec<&str> = Vec::new();
    if let Some(id) = preferred {
        candidates.push(id);
    }
    for rule in &config.rules {
        if !candidates.contains(&rule.id.as_str()) {
            candidates.push(&rule.id);
        }
    }
    for id in candidates {
        if let Ok(password) = schedule::load_credential(id) {
            if !repo_exists
                || Repo::new(&config.repo, Some(password.clone()))
                    .key_list()
                    .is_ok()
            {
                return Some(password);
            }
        }
    }
    None
}

fn finish_password_state(
    state: &Arc<Mutex<PasswordState>>,
    rule_id: &str,
    result: Result<PasswordSuccess, String>,
) {
    let mut state = state.lock().unwrap_or_else(|error| error.into_inner());
    state.pending = false;
    state.rule_id = rule_id.to_string();
    match result {
        Ok(success) => {
            state.succeeded = true;
            state.key_id = success.key_id;
            state.bind = success.bind;
            state.bind_all = success.bind_all;
            state.warning = success.warning;
        }
        Err(error) => {
            crate::logln(&format!("store: password operation failed: {error}"));
            state.error = error;
        }
    }
}

/// Opens the repository and unlocks a rule. The password only has to be a
/// valid repository key: passwords may repeat, and `rustic` cannot say which
/// key a duplicate password opened, so the application does not enforce a
/// strict password-to-rule mapping. `rule.key_id` is kept for management
/// (changing/removing keys), not for authentication.
fn unlock_rule(
    config: &Config,
    vault: &Vault,
    rule_id: &str,
    password: &str,
) -> Result<PasswordSuccess, String> {
    let rule = config
        .rule(rule_id)
        .cloned()
        .ok_or_else(|| format!("there is no backup rule '{rule_id}'"))?;
    let repo = Repo::new(&config.repo, Some(password.to_string()));
    if !repo.exists() {
        return Err(
            "this repository does not exist yet; create a password for the backup first".into(),
        );
    }

    // Validates the password. Do not compare the active key with the rule's
    // key ID: duplicate passwords make that identification ambiguous, and any
    // repository key opens the same data anyway.
    repo.key_list()?;
    let mut warning = String::new();

    if config.common_mode() {
        let Some(common) = config.common_key_id() else {
            return Err("the common password is not set yet; set it in Settings".to_string());
        };
        // The common password opens every rule at once. Repair credentials for
        // the rules that will need them for unattended backups.
        for item in &config.rules {
            if schedule::load_credential(&item.id).is_err() {
                if let Err(error) = schedule::store_credential(&item.id, password) {
                    warning = error;
                }
            }
            vault.set_rule_password(&item.id, password.to_string());
        }
        return Ok(PasswordSuccess {
            key_id: common.to_string(),
            bind: false,
            bind_all: true,
            warning,
        });
    }

    if rule.key_id.is_empty() {
        return Err("this backup has no password yet; set one first".to_string());
    }

    // Also repair a credential that exists but cannot be read (an old
    // systemd-creds key, a corrupt file, a changed TPM).
    if schedule::load_credential(&rule.id).is_err() {
        if let Err(error) = schedule::store_credential(&rule.id, password) {
            warning = error;
        }
    }
    vault.set_rule_password(&rule.id, password.to_string());
    Ok(PasswordSuccess {
        key_id: rule.key_id.clone(),
        bind: false,
        bind_all: false,
        warning,
    })
}

/// Initializes a repository with the first rule password, adds a key for a
/// rule that does not have one yet, or replaces a rule's current key.
///
/// Any password is accepted, including one that another backup already uses.
/// A new key is added and the rule is pointed at it; the old key is removed
/// only when no other rule (and not the common password) shares it.
fn set_rule_password(
    config: &Config,
    vault: &Vault,
    rule_id: &str,
    auth: &str,
    password: &str,
) -> Result<PasswordSuccess, String> {
    if password.is_empty() {
        return Err("the password must not be empty".to_string());
    }
    let rule = config
        .rule(rule_id)
        .cloned()
        .ok_or_else(|| format!("there is no backup rule '{rule_id}'"))?;
    config.verify_repository()?;
    let exists = Repo::new(&config.repo, None).exists();
    let mut warning = String::new();

    let key_id = if !exists {
        Repo::new(&config.repo, Some(password.to_string())).init()?;
        config::remember_repository_fingerprint(&config.repo);
        Repo::new(&config.repo, Some(password.to_string()))
            .active_key()?
            .id
    } else if rule.key_id.is_empty() {
        // The viewing password can be created after a passwordless backup:
        // the generated key is already in the credential, so no extra
        // authorization is needed.
        let auth = if !auth.is_empty() {
            auth.to_string()
        } else if let Some(password) = vault.any_password() {
            password
        } else if let Some(password) = load_any_credential(config, Some(&rule.id)) {
            password
        } else {
            return Err("enter an existing backup password first".to_string());
        };
        let repo = Repo::new(&config.repo, Some(auth));
        repo.key_list()?;
        repo.add_key(password)?.id
    } else {
        let current = vault
            .password_for(&rule.id)
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| auth.to_string());
        if current.is_empty() {
            return Err("enter an existing backup password first".to_string());
        }
        let repo = Repo::new(&config.repo, Some(current));
        repo.key_list()?;

        let old_key = rule.key_id.clone();
        let new_key = repo.add_key(password)?.id;
        let shared = config
            .rules
            .iter()
            .any(|other| other.id != rule.id && other.key_id == old_key)
            || config.common_key_id() == Some(old_key.as_str());
        if !shared && old_key != new_key {
            if let Err(error) = Repo::new(&config.repo, Some(password.to_string()))
                .remove_key(&old_key)
            {
                warning = error;
            }
        }
        new_key
    };

    if let Err(error) = schedule::store_credential(&rule.id, password) {
        warning = error;
    }
    vault.set_rule_password(&rule.id, password.to_string());
    Ok(PasswordSuccess {
        key_id,
        bind: true,
        bind_all: false,
        warning,
    })
}

/// Sets one common viewing password for every rule. A new rustic key is
/// added, all rules are pointed at it and every credential is refreshed with
/// the new password; the old common key is removed when another key keeps the
/// repository accessible.
fn set_common_password(
    config: &Config,
    vault: &Vault,
    auth: &str,
    password: &str,
) -> Result<PasswordSuccess, String> {
    if password.is_empty() {
        return Err("the password must not be empty".to_string());
    }
    if !config.common_mode() {
        return Err("enable the common password mode first".to_string());
    }
    if config.rules.is_empty() {
        return Err("add a backup rule first".to_string());
    }

    config.verify_repository()?;
    let exists = Repo::new(&config.repo, None).exists();
    let mut warning = String::new();
    let key_id = if !exists {
        Repo::new(&config.repo, Some(password.to_string())).init()?;
        config::remember_repository_fingerprint(&config.repo);
        Repo::new(&config.repo, Some(password.to_string()))
            .active_key()?
            .id
    } else {
        let auth = if !auth.is_empty() {
            auth.to_string()
        } else if let Some(password) = vault.any_password() {
            password
        } else if let Some(password) = load_any_credential(config, None) {
            password
        } else {
            return Err("enter an existing backup password first".to_string());
        };
        let repo = Repo::new(&config.repo, Some(auth));
        repo.key_list()?;
        let old_common = config.common_key_id().map(str::to_string);
        let new_key = repo.add_key(password)?;
        // Revoke the previous common password; if the removal fails, the new
        // key still works and the old one is reported in the log.
        if let Some(old) = old_common {
            if old != new_key.id {
                if let Err(error) = Repo::new(&config.repo, Some(password.to_string()))
                    .remove_key(&old)
                {
                    warning = error;
                }
            }
        }
        new_key.id
    };

    for rule in &config.rules {
        if let Err(error) = schedule::store_credential(&rule.id, password) {
            warning = error;
        }
        vault.set_rule_password(&rule.id, password.to_string());
    }
    Ok(PasswordSuccess {
        key_id,
        bind: true,
        bind_all: true,
        warning,
    })
}

// ----------------------------------------------------------- Maintenance ----

/// How long a successful or failed check is considered current.
const CHECK_STALE_DAYS: i64 = 30;

#[derive(Clone, Default)]
struct CheckState {
    running: bool,
    ok: bool,
    error: String,
    detail: String,
    last_check: String,
    last_check_ok: bool,
    last_check_detail: String,
    stale: bool,
    has_repo: bool,
}

impl CheckState {
    fn from_config(config: &Config) -> Self {
        let parsed = chrono::DateTime::parse_from_rfc3339(&config.last_check)
            .ok()
            .map(|time| time.with_timezone(&chrono::Local));
        let last_check = parsed
            .map(|time| time.format("%d.%m.%Y %H:%M").to_string())
            .unwrap_or_default();
        let stale = parsed
            .map(|time| {
                chrono::Local::now()
                    .signed_duration_since(time)
                    .num_days()
                    >= CHECK_STALE_DAYS
            })
            .unwrap_or(true);
        Self {
            last_check,
            last_check_ok: config.last_check_ok,
            last_check_detail: config.last_check_detail.clone(),
            stale,
            has_repo: Repo::new(&config.repo, None).exists(),
            ..Default::default()
        }
    }

    fn status_map(&self) -> QVariant {
        map_of(vec![
            ("running", QVariant::from(self.running)),
            ("ok", QVariant::from(self.ok)),
            ("error", QVariant::from(qstr(&self.error))),
            ("detail", QVariant::from(qstr(&self.detail))),
            ("lastCheck", QVariant::from(qstr(&self.last_check))),
            ("lastCheckOk", QVariant::from(self.last_check_ok)),
            ("lastCheckDetail", QVariant::from(qstr(&self.last_check_detail))),
            ("stale", QVariant::from(self.stale)),
            ("hasRepo", QVariant::from(self.has_repo)),
        ])
    }
}

/// Runs `rustic check` in the background and remembers the result in the
/// configuration, so the next start can warn about a stale check.
#[derive(QObject, Default)]
pub struct MaintenanceObject {
    base: qt_base_class!(trait QObject),
    config: Config,
    vault: Vault,
    state: Arc<Mutex<CheckState>>,

    /// `read_data` reads a random 10% sample of the pack files as well.
    start: qt_method!(fn start(&mut self, read_data: bool) {
        self.begin(read_data);
    }),
    status: qt_method!(fn status(&self) -> QVariant {
        let has_repo = Repo::new(&self.config.repo, None).exists();
        self.state
            .lock()
            .map(|mut state| {
                state.has_repo = has_repo;
                state.status_map()
            })
            .unwrap_or_default()
    }),
}

impl MaintenanceObject {
    pub fn new(config: Config, vault: Vault) -> Self {
        let state = Arc::new(Mutex::new(CheckState::from_config(&config)));
        let mut object = MaintenanceObject::default();
        object.config = config;
        object.vault = vault;
        object.state = state;
        object
    }

    fn begin(&mut self, read_data: bool) {
        {
            let Ok(mut state) = self.state.lock() else {
                return;
            };
            if state.running {
                return;
            }
            state.running = true;
            state.ok = false;
            state.error.clear();
            state.detail.clear();
        }

        let config = self.config.clone();
        let vault = self.vault.clone();
        let state = self.state.clone();
        std::thread::spawn(move || {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                check_repository(&config, &vault, read_data)
            }))
            .unwrap_or_else(|_| Err("the check worker stopped unexpectedly".to_string()));

            let ok = result.is_ok();
            let detail = result.unwrap_or_else(|error| error);
            crate::logln(&format!(
                "check: {} ({})",
                if ok { "passed" } else { "failed" },
                detail
            ));
            config::record_repository_check(ok, &detail);

            if let Ok(mut state) = state.lock() {
                state.running = false;
                state.ok = ok;
                state.error = if ok { String::new() } else { detail.clone() };
                state.detail = if ok { detail.clone() } else { String::new() };
                state.last_check = chrono::Local::now()
                    .format("%d.%m.%Y %H:%M")
                    .to_string();
                state.last_check_ok = ok;
                state.last_check_detail = detail;
                state.stale = false;
                state.has_repo = Repo::new(&config.repo, None).exists();
            }
        });
    }
}

fn check_repository(config: &Config, vault: &Vault, read_data: bool) -> Result<String, String> {
    if !Repo::new(&config.repo, None).exists() {
        return Err(format!(
            "the repository was not found at {}",
            config.repo
        ));
    }
    let password = vault
        .any_password()
        .or_else(|| load_any_credential(config, None))
        .ok_or_else(|| {
            "the repository is locked and no saved password is available".to_string()
        })?;
    Repo::new(&config.repo, Some(password)).check(read_data)
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
    pub rule: String,
    pub rule_id: String,
    pub mode: String,
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
        ("rule", QVariant::from(qstr(&status.rule))),
        ("ruleId", QVariant::from(qstr(&status.rule_id))),
        ("mode", QVariant::from(qstr(&status.mode))),
    ])
}

#[derive(QObject, Default)]
pub struct BackupObject {
    base: qt_base_class!(trait QObject),
    vault: Vault,
    pub config: Config,
    state: Arc<Mutex<JobStatus>>,
    cancel_flag: Arc<AtomicBool>,

    /// `rule` is a rule id, or an empty string to run every enabled rule.
    start: qt_method!(fn start(&mut self, rule: QString) { self.begin(rule.to_string()); }),
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

impl BackupObject {
    pub fn new(config: Config, vault: Vault) -> Self {
        let mut object = BackupObject::default();
        object.config = config;
        object.vault = vault;
        object
    }

    fn begin(&mut self, selected_rule: String) {
        // Pick up rule edits and a repository change made in Settings without
        // restarting the application.
        self.config = Config::load();
        {
            let Ok(mut status) = self.state.lock() else { return };
            if status.running {
                return;
            }
            *status = JobStatus {
                running: true,
                percent: 0.0,
                state: "preparing".into(),
                ..Default::default()
            };
        }
        self.cancel_flag.store(false, Ordering::Relaxed);

        let rules = rules_to_run(&self.config, &selected_rule);
        if rules.is_empty() {
            let error = if selected_rule.is_empty() {
                "no enabled backup rules".to_string()
            } else {
                format!("backup rule '{selected_rule}' does not exist")
            };
            set_status(&self.state, |status| {
                status.running = false;
                status.state = "failed".into();
                status.error = error;
            });
            return;
        }

        let config = self.config.clone();
        let vault = self.vault.clone();
        let state = self.state.clone();
        let cancel = self.cancel_flag.clone();
        std::thread::spawn(move || {
            let worker_state = state.clone();
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
                // A backup does not unlock the snapshots for viewing: it uses a
                // saved credential (or a generated key for a new repository)
                // and leaves the vault alone. Resolve it in the worker:
                // decrypting a credential and initializing a repository may
                // involve processes.
                let password = match resolve_backup_password(&config, &vault, &rules) {
                    Ok(password) => password,
                    Err(error) => {
                        set_status(&state, |status| {
                            status.running = false;
                            status.state = "failed".into();
                            status.error = error;
                        });
                        return;
                    }
                };
                run_backup(config, password, rules, state, cancel);
            }));
            if result.is_err() {
                crate::logln("backup: worker panicked");
                set_status(&worker_state, |status| {
                    status.running = false;
                    status.state = "failed".into();
                    status.error = "the backup worker stopped unexpectedly".into();
                });
            }
        });
    }
}

fn set_status(state: &Arc<Mutex<JobStatus>>, update: impl FnOnce(&mut JobStatus)) {
    if let Ok(mut status) = state.lock() {
        update(&mut status);
    }
}

fn rule_display_name(rule: &BackupRule) -> String {
    if rule.name.trim().is_empty() {
        rule.id.clone()
    } else {
        rule.name.clone()
    }
}

fn rules_to_run(config: &Config, selected_rule: &str) -> Vec<BackupRule> {
    if selected_rule.is_empty() {
        config
            .rules
            .iter()
            .filter(|rule| rule.enabled)
            .cloned()
            .collect()
    } else {
        config
            .rule(selected_rule)
            .cloned()
            .into_iter()
            .collect()
    }
}

/// Finds a password for a backup without asking the user: an already
/// unlocked password, a saved systemd credential (any rule's key works) or a
/// freshly generated key for a repository that does not exist yet.
fn resolve_backup_password(
    config: &Config,
    vault: &Vault,
    rules: &[BackupRule],
) -> Result<String, String> {
    config.verify_repository()?;
    if let Some(password) = vault.any_password() {
        return Ok(password);
    }

    let repo_exists = Repo::new(&config.repo, None).exists();
    if let Some(password) = load_any_credential(config, None) {
        return Ok(password);
    }

    if !repo_exists {
        let password = config::generate_password();
        Repo::new(&config.repo, Some(password.clone()))
            .init()
            .map_err(|error| format!("cannot create the repository: {error}"))?;
        config::remember_repository_fingerprint(&config.repo);
        if let Some(rule) = rules.first() {
            if let Err(error) = schedule::store_credential(&rule.id, &password) {
                crate::logln(&format!(
                    "backup: cannot save the generated password as a credential: {error}"
                ));
            }
        }
        return Ok(password);
    }

    Err(
        "the repository is locked and no saved password is available; open a backup once to save it"
            .to_string(),
    )
}

fn run_backup(
    config: Config,
    password: String,
    rules: Vec<BackupRule>,
    state: Arc<Mutex<JobStatus>>,
    cancel: Arc<AtomicBool>,
) {
    crate::logln("backup: start");

    // Make future backups passwordless: every rule that runs gets the same
    // repository key as its systemd credential, unless it already has one.
    for rule in &rules {
        if schedule::load_credential(&rule.id).is_err() {
            if let Err(error) = schedule::store_credential(&rule.id, &password) {
                crate::logln(&format!(
                    "backup: cannot save the credential for {}: {error}",
                    rule.id
                ));
            }
        }
    }

    // Wait for another rule or a CLI backup instead of hitting rustic's
    // repository lock.
    let _lock = match JobLock::acquire() {
        Ok(lock) => lock,
        Err(error) => {
            set_status(&state, |s| {
                s.running = false;
                s.state = "failed".into();
                s.error = error;
            });
            return;
        }
    };
    let repo = Repo::new(&config.repo, Some(password));

    let total = rules.len() as f64;
    let mut files = 0u64;
    let mut added = 0u64;
    let mut added_packed = 0u64;
    let mut last_snapshot = String::new();
    let mut warnings: Vec<String> = Vec::new();
    let mut ran = 0usize;

    for (index, rule) in rules.iter().enumerate() {
        if cancel.load(Ordering::Relaxed) {
            set_status(&state, |s| {
                s.running = false;
                s.state = "cancelled".into();
            });
            if config.notifications {
                crate::notify::send("Backup cancelled", "");
            }
            return;
        }

        let (sources, missing) = Config::existing_sources(rule);
        for path in &missing {
            warnings.push(format!("skipped missing source {path}"));
        }
        if sources.is_empty() {
            warnings.push(format!("rule '{}' has no existing sources", rule_display_name(rule)));
            continue;
        }

        let name = rule_display_name(rule);
        let (excludes, inside_warning) = config::effective_excludes(rule, &config.repo);
        if let Some(warning) = inside_warning {
            crate::logln(&format!("backup: rule {}: {warning}", rule.id));
            warnings.push(warning);
        }
        set_status(&state, |status| {
            status.state = "copying".into();
            status.current.clear();
            status.rule = name.clone();
            status.rule_id = rule.id.clone();
        });

        let started = std::time::Instant::now();
        let result = repo.backup(
            &sources,
            &excludes,
            &rule.id,
            rule.one_file_system,
            &rule.exclude_larger,
            &cancel,
            |update| {
                set_status(&state, |status| {
                    status.state = "copying".into();
                    status.percent = (index as f64 + update.percent) / total * 100.0;
                    let elapsed = started.elapsed().as_secs_f64();
                    status.speed = format::speed(update.bytes_done, elapsed);
                    if update.percent > 0.01 && elapsed > 1.0 {
                        let remaining = elapsed * (1.0 - update.percent) / update.percent;
                        status.eta = format::duration(remaining as i64);
                    }
                });
            },
        );

        match result {
            Ok(summary) => {
                crate::logln(&format!(
                    "backup: rule {} done, {} files, {} added ({} on disk)",
                    rule.id,
                    summary.total_files,
                    format::bytes(summary.data_added),
                    format::bytes(summary.data_added_packed)
                ));
                files += summary.total_files;
                added += summary.data_added;
                added_packed += summary.data_added_packed;
                last_snapshot = summary.snapshot_id;
                ran += 1;
                set_status(&state, |status| {
                    status.percent = (index as f64 + 1.0) / total * 100.0;
                });
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
                return;
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
                return;
            }
        }
    }

    if ran == 0 {
        let error = if warnings.is_empty() {
            "no existing sources in the selected rules".to_string()
        } else {
            warnings.join("; ")
        };
        set_status(&state, |status| {
            status.running = false;
            status.state = "failed".into();
            status.error = error;
        });
        return;
    }

    set_status(&state, |status| {
        status.running = false;
        status.percent = 100.0;
        status.state = "done".into();
        status.files = files;
        status.added = added;
        status.snapshot_id = last_snapshot;
        status.current.clear();
        status.error = warnings.join("; ");
    });
    let _ = repo.forget_policy(config.keep_hourly, config.keep_daily, config.keep_weekly);
    if config.notifications {
        crate::notify::send(
            "Backup finished",
            &format!(
                "{ran} rule(s), {} added ({} on disk)",
                format::bytes(added),
                format::bytes(added_packed)
            ),
        );
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
        self.begin(
            snapshot.to_string(),
            paths.to_string(),
            destination.to_string(),
            policy.to_string(),
            "restore".to_string(),
        );
    }),
    /// Saves the selected items directly into `destination`, without
    /// recreating the original folder tree (a "copy out" instead of a
    /// restore in place). Existing names get a timestamped sibling.
    saveToFolder: qt_method!(fn saveToFolder(&mut self, snapshot: QString, paths: QString, destination: QString) {
        self.begin(
            snapshot.to_string(),
            paths.to_string(),
            destination.to_string(),
            "keep_both".to_string(),
            "extract".to_string(),
        );
    }),
    /// Extracts one snapshot item into a private cache folder and hands it to
    /// the default application, like double-clicking a file in Dolphin.
    openItem: qt_method!(fn openItem(&mut self, snapshot: QString, path: QString) {
        self.begin(
            snapshot.to_string(),
            path.to_string(),
            String::new(),
            "overwrite".to_string(),
            "open".to_string(),
        );
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

    fn begin(&mut self, snapshot: String, paths: String, destination: String, policy: String, mode: String) {
        if self.state.lock().map(|s| s.running).unwrap_or(false) {
            return;
        }
        self.config = Config::load();
        let items: Vec<String> = paths
            .lines()
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty())
            .collect();
        if items.is_empty() {
            return;
        }
        if mode == "extract" && destination.trim().is_empty() {
            return;
        }
        {
            let Ok(mut status) = self.state.lock() else { return };
            *status = JobStatus {
                running: true,
                state: "restoring".into(),
                mode: mode.clone(),
                ..Default::default()
            };
        }
        self.cancel_flag.store(false, Ordering::Relaxed);

        let config = self.config.clone();
        let state = self.state.clone();
        let cancel = self.cancel_flag.clone();
        let vault = self.vault.clone();
        std::thread::spawn(move || {
            let worker_state = state.clone();
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
                run_restore(config, vault, snapshot, items, destination, policy, mode, state, cancel)
            }));
            if result.is_err() {
                crate::logln("restore: worker panicked");
                set_status(&worker_state, |status| {
                    status.running = false;
                    status.state = "failed".into();
                    status.error = "the restore worker stopped unexpectedly".into();
                });
            }
        });
    }
}

#[allow(clippy::too_many_arguments)]
fn run_restore(
    config: Config,
    vault: Vault,
    snapshot: String,
    items: Vec<String>,
    destination: String,
    policy: String,
    mode: String,
    state: Arc<Mutex<JobStatus>>,
    cancel: Arc<AtomicBool>,
) {
    let Some(password) = vault.any_password() else {
        set_status(&state, |s| {
            s.running = false;
            s.state = "failed".into();
            s.error = "repository is locked: enter the key first".into();
        });
        return;
    };
    let repo = Repo::new(&config.repo, Some(password));
    let extracting = mode == "extract";
    let opening = mode == "open";
    if opening {
        // /tmp is shared; keep the extracted copy private to this user.
        let root = paths::open_root();
        if let Err(error) = std::fs::create_dir_all(&root) {
            set_status(&state, |s| {
                s.running = false;
                s.state = "failed".into();
                s.error = error.to_string();
            });
            return;
        }
        let _ = std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o700));
        cleanup_open_cache();
    }
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

        let staging = if opening {
            // Keep this copy: the launched application may still be reading
            // it after `xdg-open` returns. The next open clears old entries.
            paths::open_root().join(format!(
                "{}-{}-{}",
                std::process::id(),
                index,
                chrono::Local::now().format("%Y%m%d-%H%M%S-%f")
            ))
        } else {
            paths::staging_root().join(format!("{}-{}", std::process::id(), index))
        };
        let _ = std::fs::remove_dir_all(&staging);
        if let Err(error) = std::fs::create_dir_all(&staging) {
            set_status(&state, |s| {
                s.running = false;
                s.state = "failed".into();
                s.error = error.to_string();
            });
            return;
        }
        if opening {
            let _ = std::fs::set_permissions(&staging, std::fs::Permissions::from_mode(0o700));
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

        if opening {
            if let Err(error) = open_restored_item(&staging, relative) {
                let _ = std::fs::remove_dir_all(&staging);
                set_status(&state, |s| {
                    s.running = false;
                    s.state = "failed".into();
                    s.error = error;
                });
                return;
            }
            restored += 1;
            set_status(&state, |s| s.percent = (index as f64 + 1.0) / total * 100.0);
            continue;
        }

        let placed = if extracting {
            place_export_item(&staging, relative, &destination)
        } else {
            place_item(&staging, relative, &destination, &policy)
        };
        if let Err(error) = placed {
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
    // Opening a file is an interactive action; a desktop notification would
    // be noise.
    if config.notifications && !opening {
        if extracting {
            crate::notify::send(
                "Saved from backup",
                &format!("{restored} item(s) saved to {destination}"),
            );
        } else {
            crate::notify::send("Restore finished", &format!("{restored} item(s) restored"));
        }
    }
}

/// Hands the extracted item to the desktop. A single restored file lands in
/// `staging/<name>`; a directory is restored directly into `staging`.
fn open_restored_item(staging: &Path, relative: &str) -> Result<(), String> {
    let name = Path::new(relative)
        .file_name()
        .ok_or_else(|| format!("cannot open {relative}: the path has no file name"))?;
    let candidate = staging.join(name);
    let target = if std::fs::symlink_metadata(&candidate).is_ok() {
        candidate
    } else {
        staging.to_path_buf()
    };
    match std::process::Command::new("xdg-open").arg(&target).spawn() {
        Ok(_) => Ok(()),
        Err(error) => {
            // Fall back to gio on systems without xdg-open.
            match std::process::Command::new("gio")
                .arg("open")
                .arg(&target)
                .spawn()
            {
                Ok(_) => Ok(()),
                Err(_) => Err(format!("cannot open {}: {error}", target.display())),
            }
        }
    }
}

/// Drops extracted preview files that are unlikely to be read any more.
fn cleanup_open_cache() {
    let root = paths::open_root();
    let Ok(entries) = std::fs::read_dir(&root) else {
        return;
    };
    let now = std::time::SystemTime::now();
    for entry in entries.flatten() {
        let path = entry.path();
        let old = entry
            .metadata()
            .and_then(|metadata| metadata.modified())
            .ok()
            .and_then(|modified| now.duration_since(modified).ok())
            .map(|age| age.as_secs() > 7 * 24 * 60 * 60)
            .unwrap_or(false);
        if old {
            let _ = std::fs::remove_dir_all(path);
        }
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

/// Puts one restored item directly into `destination` under its own name
/// (no `home/user/...` prefix). Used by "Save to Folder…": the chosen folder
/// is where the user wants the copies, not a new snapshot root.
pub fn place_export_item(staging: &Path, relative: &str, destination: &str) -> Result<(), String> {
    let destination = destination.trim();
    if destination.is_empty() {
        return Err("choose a folder to save the copies to".to_string());
    }
    let staged_name = Path::new(relative)
        .file_name()
        .map(|name| name.to_os_string())
        .ok_or_else(|| format!("cannot save {relative}: the path has no file name"))?;
    let staged_file = staging.join(&staged_name);
    let is_file = std::fs::symlink_metadata(&staged_file)
        .map(|meta| !meta.is_dir())
        .unwrap_or(false);

    let destination = Path::new(destination);
    std::fs::create_dir_all(destination).map_err(|error| error.to_string())?;
    let target = destination.join(&staged_name);
    let final_target = if target.exists() {
        unique_target(&target)
    } else {
        target
    };

    if is_file {
        copy_item(&staged_file, &final_target)
    } else {
        std::fs::create_dir_all(&final_target).map_err(|error| error.to_string())?;
        copy_tree(staging, &final_target)
    }
}

/// Name search over a snapshot listing. Paths are searched too, so a folder
/// name also finds the files below it. Kept separate from the QObject for
/// tests.
fn search_entries<'a>(
    entries: &'a [rustic::Entry],
    query: &str,
    limit: usize,
) -> Vec<&'a rustic::Entry> {
    let needle = query.trim().to_lowercase();
    if needle.len() < 2 {
        return Vec::new();
    }
    let mut matches = Vec::new();
    for entry in entries {
        if entry.path.to_lowercase().contains(&needle) {
            matches.push(entry);
            if matches.len() >= limit {
                break;
            }
        }
    }
    matches
}

// --------------------------------------------------------------- Browser ----

#[derive(Default)]
struct BrowserLoad {
    generation: u64,
    loading: bool,
    ready: bool,
    snapshot_id: String,
    entries: Vec<rustic::Entry>,
    error: String,
}

#[derive(QObject, Default)]
pub struct BrowserObject {
    base: qt_base_class!(trait QObject),
    vault: Vault,
    pub config: Config,
    snapshot_id: String,
    entries_cache: Vec<rustic::Entry>,
    current_path: String,
    load: Arc<Mutex<BrowserLoad>>,

    setSnapshot: qt_method!(fn setSnapshot(&mut self, id: QString) { self.load(id.to_string()); }),
    loading: qt_method!(fn loading(&self) -> bool {
        self.load.lock().map(|state| state.loading).unwrap_or(false)
    }),
    poll: qt_method!(fn poll(&mut self) -> QVariant { self.take_loaded() }),
    entries: qt_method!(fn entries(&self) -> QVariant { self.entry_list() }),
    setPath: qt_method!(fn setPath(&mut self, path: QString) { self.jump(path.to_string()); }),
    goUp: qt_method!(fn goUp(&mut self) { self.up(); }),
    canGoUp: qt_method!(fn canGoUp(&self) -> bool { !self.current_path.is_empty() }),
    path: qt_method!(fn path(&self) -> QString { qstr(&self.current_path) }),
    displayPath: qt_method!(fn displayPath(&self) -> QString { qstr(&self.pretty_path()) }),
    homeRoot: qt_method!(fn homeRoot(&self) -> QString { qstr(&self.home_prefix()) }),
    fileCount: qt_method!(fn fileCount(&self) -> i32 { self.entries_cache.len() as i32 }),
    /// Case-insensitive name search over the whole snapshot. Returns at most
    /// 200 hits; the QML browser turns one into a navigation target.
    search: qt_method!(fn search(&self, query: QString) -> QVariant {
        self.search_list(&query.to_string())
    }),
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
        let current = Config::load();
        if current.repo != self.config.repo {
            self.entries_cache.clear();
            self.snapshot_id.clear();
        }
        self.config = current;
        let same_target = id == self.snapshot_id;
        if same_target && !self.entries_cache.is_empty() {
            return;
        }

        let generation;
        {
            let mut state = self.load.lock().unwrap_or_else(|error| error.into_inner());
            if same_target && state.loading {
                return;
            }
            state.generation = state.generation.wrapping_add(1);
            generation = state.generation;
            state.loading = !id.is_empty();
            state.ready = false;
            state.snapshot_id = id.clone();
            state.entries.clear();
            state.error.clear();
        }

        self.snapshot_id = id.clone();
        self.entries_cache.clear();
        self.current_path.clear();
        if id.is_empty() {
            return;
        }

        let Some(password) = self.vault.any_password() else {
            let mut state = self.load.lock().unwrap_or_else(|error| error.into_inner());
            state.error = "repository is locked".into();
            state.ready = true;
            return;
        };

        // Listing a whole snapshot is the slowest repository operation: run it
        // in the background so the timeline stays responsive, and drop the
        // result when the selection changed while it was running.
        let repo_path = self.config.repo.clone();
        let state = self.load.clone();
        std::thread::spawn(move || {
            let repo = Repo::new(&repo_path, Some(password));
            let result = repo.ls(&id);
            let mut state = state.lock().unwrap_or_else(|error| error.into_inner());
            if state.generation != generation {
                return;
            }
            match result {
                Ok(entries) => state.entries = entries,
                Err(error) => {
                    crate::logln(&format!("browser: cannot list snapshot: {error}"));
                    state.error = error;
                }
            }
            state.ready = true;
        });
    }

    /// Moves a finished background listing into the cache. Returns
    /// `{ done, error }`; `done` stays false until the selected snapshot is
    /// actually available.
    fn take_loaded(&mut self) -> QVariant {
        let mut state = self.load.lock().unwrap_or_else(|error| error.into_inner());
        if !state.ready || state.snapshot_id != self.snapshot_id {
            return map_of(vec![
                ("done", QVariant::from(false)),
                ("error", QVariant::from(qstr(""))),
            ]);
        }
        state.ready = false;
        state.loading = false;
        let entries = std::mem::take(&mut state.entries);
        let error = std::mem::take(&mut state.error);
        drop(state);

        if error.is_empty() {
            self.entries_cache = entries;
            self.current_path = self.home_prefix();
        }
        map_of(vec![
            ("done", QVariant::from(true)),
            ("error", QVariant::from(qstr(&error))),
        ])
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

    fn search_list(&self, query: &str) -> QVariant {
        let matches = search_entries(&self.entries_cache, query, 200);
        let mut list = QVariantList::default();
        for entry in matches {
            let name = entry
                .path
                .rsplit('/')
                .next()
                .unwrap_or(&entry.path)
                .to_string();
            let parent = entry
                .path
                .rsplit_once('/')
                .map(|(parent, _)| parent)
                .unwrap_or("")
                .to_string();
            list.push(map_of(vec![
                ("name", QVariant::from(qstr(&name))),
                ("relativePath", QVariant::from(qstr(&entry.path))),
                ("parent", QVariant::from(qstr(&parent))),
                ("isDir", QVariant::from(entry.is_dir)),
                ("isLink", QVariant::from(entry.is_link)),
                ("sizeText", QVariant::from(qstr(&format::bytes(entry.size)))),
            ]));
        }
        QVariant::from(list)
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
    poll: qt_method!(fn poll(&mut self) -> QVariant {
        self.config = Config::load();
        let error = self.last_error.lock().map(|s| s.clone()).unwrap_or_default();
        let enabled = self
            .config
            .rules
            .iter()
            .filter(|rule| rule.enabled && rule.schedule.enabled)
            .count();
        map_of(vec![
            ("enabledCount", QVariant::from(enabled as i32)),
            ("scheduledCount", QVariant::from(schedule::scheduled_count() as i32)),
            ("error", QVariant::from(qstr(&error))),
        ])
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
        self.config = Config::load();
        let result = schedule::sync(&self.config.rules);
        if let Err(error) = &result {
            crate::logln(&format!("schedule: {error}"));
        }
        let mut last_error = self.last_error.lock().unwrap_or_else(|e| e.into_inner());
        *last_error = result.err().unwrap_or_default();
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn scratch(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!("kustos-test-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("create scratch dir");
        path
    }

    #[test]
    fn search_matches_names_and_paths() {
        let entries = vec![
            rustic::Entry {
                path: "home/user/Documents/invoice.pdf".into(),
                is_dir: false,
                is_link: false,
                size: 10,
                mtime: String::new(),
            },
            rustic::Entry {
                path: "home/user/Documents".into(),
                is_dir: true,
                is_link: false,
                size: 0,
                mtime: String::new(),
            },
            rustic::Entry {
                path: "home/user/notes.txt".into(),
                is_dir: false,
                is_link: false,
                size: 4,
                mtime: String::new(),
            },
        ];
        assert_eq!(search_entries(&entries, "invoice", 10).len(), 1);
        assert_eq!(search_entries(&entries, "DOCUMENTS", 10).len(), 2);
        assert!(search_entries(&entries, "i", 10).is_empty());
        assert!(search_entries(&entries, "missing", 10).is_empty());
    }

    #[test]
    fn check_state_flags_stale_and_fresh_results() {
        let mut config = Config::default();
        config.repo = "/nonexistent/kustos-test-repository".into();

        let mut fresh = config.clone();
        fresh.last_check = chrono::Local::now().to_rfc3339();
        fresh.last_check_ok = true;
        fresh.last_check_detail = "ok".into();
        let state = CheckState::from_config(&fresh);
        assert!(!state.stale);
        assert!(state.last_check_ok);
        assert!(!state.has_repo);
        assert!(!state.last_check.is_empty());

        let mut old = config.clone();
        old.last_check = (chrono::Local::now() - chrono::Duration::days(31)).to_rfc3339();
        assert!(CheckState::from_config(&old).stale);

        let never = CheckState::from_config(&config);
        assert!(never.stale);
        assert!(never.last_check.is_empty());
    }

    #[test]
    fn export_puts_a_file_directly_into_the_chosen_folder() {
        let root = scratch("export-file");
        let staging = root.join("staging");
        let target = root.join("out");
        fs::create_dir_all(&staging).unwrap();
        fs::write(staging.join("notes.txt"), b"hello").unwrap();

        place_export_item(&staging, "home/user/Documents/notes.txt", &target.to_string_lossy())
            .expect("export file");

        assert_eq!(fs::read_to_string(target.join("notes.txt")).unwrap(), "hello");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn export_recreates_only_the_item_folder() {
        let root = scratch("export-dir");
        let staging = root.join("staging");
        let target = root.join("out");
        fs::create_dir_all(staging.join("sub")).unwrap();
        fs::write(staging.join("sub/file.txt"), b"data").unwrap();

        place_export_item(&staging, "home/user/Pictures/holiday", &target.to_string_lossy())
            .expect("export folder");

        assert!(target.join("holiday/sub/file.txt").exists());
        assert!(!target.join("home").exists());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn export_keeps_both_when_the_name_exists() {
        let root = scratch("export-keep-both");
        let staging = root.join("staging");
        let target = root.join("out");
        fs::create_dir_all(&staging).unwrap();
        fs::create_dir_all(&target).unwrap();
        fs::write(staging.join("report.txt"), b"new").unwrap();
        fs::write(target.join("report.txt"), b"old").unwrap();

        place_export_item(&staging, "home/user/report.txt", &target.to_string_lossy())
            .expect("export duplicate");

        assert_eq!(fs::read_to_string(target.join("report.txt")).unwrap(), "old");
        let copies: Vec<_> = fs::read_dir(&target)
            .unwrap()
            .flatten()
            .filter(|entry| entry.file_name() != "report.txt")
            .collect();
        assert_eq!(copies.len(), 1);
        let _ = fs::remove_dir_all(root);
    }

    // ------------------------------------------------------- passwords ----

    static PASSWORD_ENV: Mutex<()> = Mutex::new(());

    /// The credential and key-temporary files use the XDG directories, so the
    /// password tests get their own tree and run one at a time.
    fn password_scratch(name: &str) -> (PathBuf, std::sync::MutexGuard<'static, ()>) {
        let guard = PASSWORD_ENV.lock().unwrap_or_else(|error| error.into_inner());
        let root = scratch(name);
        std::env::set_var("XDG_CONFIG_HOME", root.join("config"));
        std::env::set_var("XDG_CACHE_HOME", root.join("cache"));
        std::env::set_var("KUSTOS_TEST_XDG_READY", "1");
        fs::create_dir_all(root.join("config/kustos")).unwrap();
        fs::create_dir_all(root.join("cache")).unwrap();
        (root, guard)
    }

    fn password_config(repo: &Path) -> Config {
        let mut config = Config::default();
        config.repo = repo.to_string_lossy().into_owned();
        config.rules = vec![BackupRule::new("r1".into(), "Test".into())];
        config
    }

    #[test]
    fn setting_a_password_initializes_and_binds_the_repository() {
        let (root, _guard) = password_scratch("password-create");
        let repo = root.join("repo");
        let config = password_config(&repo);
        let vault = Vault::default();

        let success = set_rule_password(&config, &vault, "r1", "", "first-password")
            .expect("set the first password");
        assert!(success.bind);
        assert!(!success.key_id.is_empty());
        assert_eq!(vault.password_for("r1").as_deref(), Some("first-password"));
        assert!(schedule::credential_kind("r1").is_some());

        let keys = Repo::new(&config.repo, Some("first-password".into()))
            .key_list()
            .expect("list keys");
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0].id, success.key_id);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn unlocking_accepts_any_valid_repository_password() {
        let (root, _guard) = password_scratch("password-unlock");
        let repo = root.join("repo");
        let mut config = password_config(&repo);
        let first = set_rule_password(&config, &Vault::default(), "r1", "", "one-password")
            .expect("first password");
        config.rules[0].key_id = first.key_id.clone();

        let mut second_rule = BackupRule::new("r2".into(), "Second".into());
        second_rule.key_id = String::new();
        config.rules.push(second_rule);

        let second =
            set_rule_password(&config, &Vault::default(), "r2", "one-password", "two-password")
                .expect("second password");
        config.rules[1].key_id = second.key_id.clone();

        assert_ne!(first.key_id, second.key_id);
        // The rule's own password works.
        assert!(unlock_rule(&config, &Vault::default(), "r2", "two-password").is_ok());
        // Any other valid repository key works too: duplicate passwords make
        // key identification ambiguous, and every key opens the same data.
        assert!(unlock_rule(&config, &Vault::default(), "r2", "one-password").is_ok());
        // A password that opens nothing is still rejected.
        assert!(unlock_rule(&config, &Vault::default(), "r2", "wrong-password").is_err());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn duplicate_passwords_are_allowed() {
        let (root, _guard) = password_scratch("password-duplicate");
        let repo = root.join("repo");
        let mut config = password_config(&repo);
        let first = set_rule_password(&config, &Vault::default(), "r1", "", "one-password")
            .expect("first password");
        config.rules[0].key_id = first.key_id.clone();
        config.rules.push(BackupRule::new("r2".into(), "Second".into()));

        let second =
            set_rule_password(&config, &Vault::default(), "r2", "one-password", "one-password")
                .expect("a duplicate password must be accepted");
        config.rules[1].key_id = second.key_id.clone();

        assert_ne!(first.key_id, second.key_id);
        let keys = Repo::new(&config.repo, Some("one-password".into()))
            .key_list()
            .expect("list keys");
        assert_eq!(keys.len(), 2);
        assert!(unlock_rule(&config, &Vault::default(), "r1", "one-password").is_ok());
        assert!(unlock_rule(&config, &Vault::default(), "r2", "one-password").is_ok());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn any_non_empty_password_is_accepted() {
        let (root, _guard) = password_scratch("password-short");
        let repo = root.join("repo");
        let config = password_config(&repo);
        let success = set_rule_password(&config, &Vault::default(), "r1", "", "x")
            .expect("a one-character password must be accepted");
        let mut bound = config.clone();
        bound.rules[0].key_id = success.key_id;
        assert!(unlock_rule(&bound, &Vault::default(), "r1", "x").is_ok());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn setting_the_same_password_again_is_allowed() {
        let (root, _guard) = password_scratch("password-same");
        let repo = root.join("repo");
        let mut config = password_config(&repo);
        let first = set_rule_password(&config, &Vault::default(), "r1", "", "one-password")
            .expect("first password");
        config.rules[0].key_id = first.key_id.clone();

        let vault = Vault::default();
        vault.set_rule_password("r1", "one-password".into());
        let changed = set_rule_password(&config, &vault, "r1", "", "one-password")
            .expect("re-entering the same password must be allowed");
        assert_ne!(changed.key_id, first.key_id);
        config.rules[0].key_id = changed.key_id;
        assert!(unlock_rule(&config, &Vault::default(), "r1", "one-password").is_ok());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn changing_a_password_replaces_the_key() {
        let (root, _guard) = password_scratch("password-change");
        let repo = root.join("repo");
        let mut config = password_config(&repo);
        let first = set_rule_password(&config, &Vault::default(), "r1", "", "one-password")
            .expect("first password");
        config.rules[0].key_id = first.key_id.clone();

        let vault = Vault::default();
        vault.set_rule_password("r1", "one-password".into());
        let changed = set_rule_password(&config, &vault, "r1", "", "new-password")
            .expect("change the password");
        assert_ne!(changed.key_id, first.key_id);
        config.rules[0].key_id = changed.key_id;

        assert!(unlock_rule(&config, &Vault::default(), "r1", "new-password").is_ok());
        assert!(unlock_rule(&config, &Vault::default(), "r1", "one-password").is_err());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn a_new_repository_gets_a_generated_password_and_credential() {
        let (root, _guard) = password_scratch("backup-generate");
        let repo = root.join("repo");
        let config = password_config(&repo);
        let rules = rules_to_run(&config, "");
        assert_eq!(rules.len(), 1);
        assert!(load_any_credential(&config, None).is_none());

        let password = resolve_backup_password(&config, &Vault::default(), &rules)
            .expect("resolve a generated password");
        assert!(password.len() >= 8);
        assert!(Repo::new(&config.repo, Some(password.clone())).exists());
        assert_eq!(
            schedule::load_credential("r1").as_deref(),
            Ok(password.as_str())
        );

        // The saved credential is used again without the vault or the user.
        let again = resolve_backup_password(&config, &Vault::default(), &rules).unwrap();
        assert_eq!(again, password);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn the_saved_credential_is_available_as_a_recovery_key() {
        let (root, _guard) = password_scratch("recovery-key");
        let config = password_config(&root.join("repo"));
        let rules = rules_to_run(&config, "");
        let password = resolve_backup_password(&config, &Vault::default(), &rules)
            .expect("initialize with a generated key");
        assert_eq!(
            load_any_credential(&config, None).as_deref(),
            Some(password.as_str())
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn a_viewing_password_can_be_created_with_the_saved_credential() {
        let (root, _guard) = password_scratch("backup-view-password");
        let repo = root.join("repo");
        let config = password_config(&repo);
        let rules = rules_to_run(&config, "");
        resolve_backup_password(&config, &Vault::default(), &rules)
            .expect("initialize with a generated key");

        // No typed authorization: the generated credential authorizes the new
        // viewing key.
        let success = set_rule_password(&config, &Vault::default(), "r1", "", "viewing-password")
            .expect("create a viewing password");
        assert!(success.bind);
        assert!(!success.key_id.is_empty());
        assert_eq!(
            schedule::load_credential("r1").as_deref(),
            Ok("viewing-password")
        );

        let mut bound = config.clone();
        bound.rules[0].key_id = success.key_id;
        assert!(unlock_rule(&bound, &Vault::default(), "r1", "viewing-password").is_ok());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn a_common_password_unlocks_every_rule_and_replaces_its_key() {
        let (root, _guard) = password_scratch("common-password");
        let repo = root.join("repo");
        let mut config = password_config(&repo);
        config.set_password_mode("common");

        let first = set_common_password(&config, &Vault::default(), "", "common-password")
            .expect("set the common password");
        assert!(first.bind_all);
        assert_eq!(
            schedule::load_credential("r1").as_deref(),
            Ok("common-password")
        );
        config.bind_common_key(first.key_id.clone());

        // A second rule joins the common key.
        let second_id = config.add_rule("Second".into());
        assert_eq!(config.rule(&second_id).unwrap().key_id, first.key_id);

        let vault = Vault::default();
        let unlocked = unlock_rule(&config, &vault, "r1", "common-password")
            .expect("unlock with the common password");
        assert!(unlocked.bind_all);
        assert_eq!(vault.unlocked_rules(), vec!["r1".to_string(), second_id.clone()]);

        // Changing the common password revokes the old key and updates all
        // credentials.
        let changed = set_common_password(&config, &Vault::default(), "", "new-common-password")
            .expect("change the common password");
        assert!(changed.bind_all);
        assert_ne!(changed.key_id, first.key_id);
        assert_eq!(
            schedule::load_credential("r1").as_deref(),
            Ok("new-common-password")
        );
        config.bind_common_key(changed.key_id.clone());
        assert!(Repo::new(&config.repo, Some("common-password".into()))
            .key_list()
            .is_err());
        assert!(unlock_rule(&config, &Vault::default(), &second_id, "new-common-password").is_ok());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn per_rule_mode_still_unlocks_one_rule_at_a_time() {
        let (root, _guard) = password_scratch("per-rule-still");
        let repo = root.join("repo");
        let mut config = password_config(&repo);
        let first = set_rule_password(&config, &Vault::default(), "r1", "", "one-password")
            .expect("first password");
        config.rules[0].key_id = first.key_id;
        config.rules.push(BackupRule::new("r2".into(), "Second".into()));
        let second = set_rule_password(&config, &Vault::default(), "r2", "one-password", "two-password")
            .expect("second password");
        config.rules[1].key_id = second.key_id;

        let vault = Vault::default();
        let unlocked = unlock_rule(&config, &vault, "r1", "one-password").expect("unlock r1");
        assert!(!unlocked.bind_all);
        assert_eq!(vault.unlocked_rules(), vec!["r1".to_string()]);
        assert!(!vault.is_unlocked("r2"));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn a_rule_without_a_password_has_to_set_one_first() {
        let (root, _guard) = password_scratch("password-none");
        let repo = root.join("repo");
        let config = password_config(&repo);
        Repo::new(&config.repo, Some("repo-password".into()))
            .init()
            .expect("init repository");
        let error = unlock_rule(&config, &Vault::default(), "r1", "repo-password")
            .expect_err("a rule without a key must be configured");
        assert!(error.contains("no password"));
        let _ = fs::remove_dir_all(root);
    }
}
