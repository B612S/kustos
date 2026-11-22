// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 B612S <sem.alimov@inbox.ru>

use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::paths;

/// Tag prefix that marks which backup rule produced a snapshot. The whole tag
/// is `kustos-rule:<rule id>` and lives next to the generic `kustos` tag.
pub const RULE_TAG_PREFIX: &str = "kustos-rule:";

pub const PASSWORD_MODE_PER_RULE: &str = "per_rule";
pub const PASSWORD_MODE_COMMON: &str = "common";

pub const SCHEDULE_INTERVAL: &str = "interval";
pub const SCHEDULE_DAILY: &str = "daily";
pub const SCHEDULE_WEEKLY: &str = "weekly";

fn default_schedule_mode() -> String {
    SCHEDULE_INTERVAL.to_string()
}

fn default_schedule_minutes() -> u32 {
    60
}

fn default_schedule_time() -> String {
    "03:00".to_string()
}

fn default_weekdays() -> Vec<u32> {
    vec![1, 2, 3, 4, 5, 6, 7]
}

fn default_enabled() -> bool {
    true
}

fn default_password_mode() -> String {
    PASSWORD_MODE_PER_RULE.to_string()
}

fn default_common_key_id() -> String {
    String::new()
}

/// Minutes of inactivity before the unlocked backup passwords are forgotten.
/// `0` means "never lock automatically".
fn default_lock_timeout_minutes() -> u32 {
    15
}

/// When a rule runs automatically. `interval` repeats every `minutes`,
/// `daily` runs at `time`, `weekly` runs at `time` on the listed ISO weekdays
/// (1 = Monday … 7 = Sunday).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuleSchedule {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_schedule_mode")]
    pub mode: String,
    #[serde(default = "default_schedule_minutes")]
    pub minutes: u32,
    #[serde(default = "default_schedule_time")]
    pub time: String,
    #[serde(default = "default_weekdays")]
    pub weekdays: Vec<u32>,
}

impl Default for RuleSchedule {
    fn default() -> Self {
        Self {
            enabled: false,
            mode: default_schedule_mode(),
            minutes: default_schedule_minutes(),
            time: default_schedule_time(),
            weekdays: default_weekdays(),
        }
    }
}

impl RuleSchedule {
    fn normalize(&mut self) {
        if !matches!(
            self.mode.as_str(),
            SCHEDULE_INTERVAL | SCHEDULE_DAILY | SCHEDULE_WEEKLY
        ) {
            self.mode = SCHEDULE_INTERVAL.to_string();
        }
        self.minutes = self.minutes.clamp(5, 7 * 24 * 60);
        self.time = normalize_time(&self.time);
        self.weekdays.retain(|day| (1..=7).contains(day));
        self.weekdays.sort_unstable();
        self.weekdays.dedup();
        if self.mode == SCHEDULE_WEEKLY && self.weekdays.is_empty() {
            self.weekdays = default_weekdays();
        }
    }

    /// Human readable summary, used by the CLI and as a fallback in QML.
    pub fn describe(&self) -> String {
        if !self.enabled {
            return "off".to_string();
        }
        match self.mode.as_str() {
            SCHEDULE_DAILY => format!("daily at {}", self.time),
            SCHEDULE_WEEKLY => {
                let names = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
                let days: Vec<&str> = self
                    .weekdays
                    .iter()
                    .filter_map(|day| names.get((*day as usize).saturating_sub(1)))
                    .copied()
                    .collect();
                format!("weekly on {} at {}", days.join(", "), self.time)
            }
            _ => format!("every {} min", self.minutes),
        }
    }
}

/// One complete backup rule: what to copy, what to skip and when to run.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BackupRule {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub sources: Vec<String>,
    #[serde(default)]
    pub excludes: Vec<String>,
    /// Do not cross filesystem boundaries while scanning a source.
    #[serde(default)]
    pub one_file_system: bool,
    /// Skip files larger than this size (rustic suffix such as `2G`); empty
    /// means no limit.
    #[serde(default)]
    pub exclude_larger: String,
    #[serde(default)]
    pub schedule: RuleSchedule,
    /// ID of the rustic key created from this rule's password. Empty until the
    /// user gives the rule its own password.
    #[serde(default)]
    pub key_id: String,
}

impl Default for BackupRule {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            enabled: true,
            sources: Vec::new(),
            excludes: Vec::new(),
            one_file_system: false,
            exclude_larger: String::new(),
            schedule: RuleSchedule::default(),
            key_id: String::new(),
        }
    }
}

impl BackupRule {
    pub fn new(id: String, name: String) -> Self {
        Self {
            id,
            name,
            ..Default::default()
        }
    }

    fn normalize(&mut self) {
        self.id = sanitize_id(&self.id);
        self.name = self.name.trim().to_string();
        self.sources = clean_paths(&self.sources);
        self.excludes = clean_paths(&self.excludes);
        self.exclude_larger = self.exclude_larger.trim().to_string();
        self.schedule.normalize();
        // Key IDs are hexadecimal; anything else cannot come from rustic.
        self.key_id = self
            .key_id
            .chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .collect();
    }

    /// Whether this rule has its own password (and therefore its own rustic
    /// key). A rule without one can still be opened with a repository key.
    pub fn has_password(&self) -> bool {
        !self.key_id.is_empty()
    }
}

/// Persisted settings, a single JSON document in
/// `$XDG_CONFIG_HOME/kustos/kustos.json`.
///
/// Older versions stored one global scope (`sources`, `excludes`,
/// `schedule_*`). Those fields are still read so an existing installation is
/// migrated into a single default rule, but they are never written back.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub repo: String,
    #[serde(default)]
    pub rules: Vec<BackupRule>,
    pub language: String,
    pub notifications: bool,
    pub keep_hourly: u32,
    pub keep_daily: u32,
    pub keep_weekly: u32,
    /// Minutes without user activity before the in-memory passwords are
    /// dropped; `0` keeps them until the application quits or the user locks.
    #[serde(default = "default_lock_timeout_minutes")]
    pub lock_timeout_minutes: u32,
    /// `per_rule`: every rule has its own viewing password.
    /// `common`: one password protects (and unlocks) all rules.
    #[serde(default = "default_password_mode")]
    pub password_mode: String,
    /// The rustic key created by the common password. Empty until the user
    /// sets one in Settings or in the password dialog.
    #[serde(default = "default_common_key_id")]
    pub common_key_id: String,
    /// Fingerprint of the repository's encrypted `config` file. Once set, a
    /// missing or different repository at `repo` is an error instead of a
    /// fresh `init`.
    #[serde(default)]
    pub repo_fingerprint: String,
    /// Timestamp (RFC 3339) of the last repository integrity check.
    #[serde(default)]
    pub last_check: String,
    #[serde(default)]
    pub last_check_ok: bool,
    /// Short human-readable result or error of that check.
    #[serde(default)]
    pub last_check_detail: String,

    #[serde(default, skip_serializing)]
    sources: Vec<String>,
    #[serde(default, skip_serializing)]
    excludes: Vec<String>,
    #[serde(default, skip_serializing)]
    schedule_enabled: bool,
    #[serde(default, skip_serializing)]
    schedule_minutes: u32,
}

impl Default for Config {
    fn default() -> Self {
        let home = paths::home();
        let data = PathBuf::from("/data");
        let repo = if data.is_dir() {
            data.join("Backup/kustos")
        } else {
            home.join("Backup/kustos")
        };
        Self {
            repo: repo.to_string_lossy().into_owned(),
            rules: vec![default_rule()],
            language: "auto".into(),
            notifications: true,
            keep_hourly: 24,
            keep_daily: 30,
            keep_weekly: 12,
            lock_timeout_minutes: default_lock_timeout_minutes(),
            password_mode: default_password_mode(),
            common_key_id: default_common_key_id(),
            repo_fingerprint: String::new(),
            last_check: String::new(),
            last_check_ok: false,
            last_check_detail: String::new(),
            sources: Vec::new(),
            excludes: Vec::new(),
            schedule_enabled: false,
            schedule_minutes: 60,
        }
    }
}

fn default_rule() -> BackupRule {
    let mut rule = BackupRule::new("default".to_string(), "Home".to_string());
    rule.sources = default_sources();
    rule.excludes = default_excludes();
    rule
}

pub fn default_sources() -> Vec<String> {
    let home = paths::home();
    let candidates = [
        ".config",
        ".local",
        "Music",
        "Pictures",
        "Videos",
        ".bashrc",
        ".bash_profile",
    ];
    candidates
        .iter()
        .map(|c| home.join(c))
        .filter(|p| p.exists())
        .map(|p| p.to_string_lossy().into_owned())
        .collect()
}

pub fn recommended_excludes() -> Vec<String> {
    let home = paths::home();
    let share = home.join(".local/share");
    vec![
        share.join("baloo").to_string_lossy().into_owned(),
        share.join("akonadi").to_string_lossy().into_owned(),
        home.join(".cache").to_string_lossy().into_owned(),
        home.join(".var/app/*/cache").to_string_lossy().into_owned(),
    ]
}

pub fn default_excludes() -> Vec<String> {
    let share = paths::home().join(".local/share");
    let mut excludes = vec![
        share.join("Steam").to_string_lossy().into_owned(),
        share.join("twintaillauncher").to_string_lossy().into_owned(),
        share.join("Trash").to_string_lossy().into_owned(),
    ];
    for path in recommended_excludes() {
        if !excludes.contains(&path) {
            excludes.push(path);
        }
    }
    excludes
}

fn clean_paths(paths: &[String]) -> Vec<String> {
    let mut result = Vec::new();
    for path in paths {
        let path = path.trim();
        if path.is_empty() || result.iter().any(|known| known == path) {
            continue;
        }
        result.push(path.to_string());
    }
    result
}

fn sanitize_id(value: &str) -> String {
    value
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .collect()
}

/// Accepts `HH:MM` (and `H:MM`), returns a normalized `HH:MM` or `03:00`.
fn normalize_time(value: &str) -> String {
    let trimmed = value.trim();
    let Some((hours, minutes)) = trimmed.split_once(':') else {
        return default_schedule_time();
    };
    let (Ok(hours), Ok(minutes)) = (hours.parse::<u32>(), minutes.parse::<u32>()) else {
        return default_schedule_time();
    };
    if hours > 23 || minutes > 59 {
        return default_schedule_time();
    }
    format!("{hours:02}:{minutes:02}")
}

impl Config {
    pub fn load() -> Self {
        let path = paths::config_file();
        let text = match fs::read_to_string(&path) {
            Ok(text) => text,
            Err(_) => {
                // A missing config next to a good backup usually means an
                // interrupted update; prefer the backup over defaults.
                if let Some(config) = read_backup(&path) {
                    crate::logln("config: restored from kustos.json.bak");
                    let _ = fs::copy(backup_path(&path), &path);
                    return config;
                }
                let mut config = Config::default();
                let _ = config.save_fresh();
                return config;
            }
        };
        match serde_json::from_str::<Config>(&text) {
            Ok(mut config) => {
                // Only a file without a `rules` key is an old single-scope
                // configuration. An explicit empty list means the user
                // removed every rule and must stay empty.
                let has_rules = serde_json::from_str::<serde_json::Value>(&text)
                    .map(|value| value.get("rules").is_some())
                    .unwrap_or(false);
                config.normalize(!has_rules);
                let adopted = config.adopt_repository_fingerprint();
                if !has_rules || adopted {
                    let _ = config.save();
                }
                config
            }
            Err(error) => {
                crate::logln(&format!("config: cannot parse {}: {error}", path.display()));
                let corrupt = path.with_extension("json.corrupt");
                let _ = fs::copy(&path, &corrupt);
                if let Some(mut config) = read_backup(&path) {
                    crate::logln("config: restored from kustos.json.bak");
                    let _ = fs::copy(backup_path(&path), &path);
                    config.normalize(false);
                    return config;
                }
                Config::default()
            }
        }
    }

    /// Reads `<repo>/config` and remembers its ID. Returns true when the ID
    /// was not known before.
    pub fn adopt_repository_fingerprint(&mut self) -> bool {
        if !self.repo_fingerprint.is_empty() {
            return false;
        }
        let Some(fingerprint) = repository_fingerprint(Path::new(&self.repo)) else {
            return false;
        };
        self.repo_fingerprint = fingerprint;
        true
    }

    /// The remembered ID, or the one in the on-disk config when this object
    /// predates it (a repository initialized earlier in the same session).
    fn expected_repo_fingerprint(&self) -> String {
        if !self.repo_fingerprint.is_empty() {
            return self.repo_fingerprint.clone();
        }
        let Ok(text) = fs::read_to_string(paths::config_file()) else {
            return String::new();
        };
        let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) else {
            return String::new();
        };
        if value.get("repo").and_then(|value| value.as_str()) != Some(self.repo.as_str()) {
            return String::new();
        }
        value
            .get("repo_fingerprint")
            .and_then(|value| value.as_str())
            .unwrap_or_default()
            .to_string()
    }

    /// Refuses to create or use a repository that does not match the one this
    /// configuration was created with. A first run has no remembered ID and is
    /// therefore allowed to initialize a new repository.
    pub fn verify_repository(&self) -> Result<(), String> {
        let path = Path::new(&self.repo);
        let exists = path.join("config").is_file();
        let expected = self.expected_repo_fingerprint();
        if !exists {
            if !expected.is_empty() {
                return Err(format!(
                    "the repository was not found at {}; is the drive mounted?",
                    self.repo
                ));
            }
            if path.is_dir()
                && fs::read_dir(path)
                    .map(|mut entries| entries.next().is_some())
                    .unwrap_or(false)
            {
                return Err(format!(
                    "{} is not an empty repository folder; choose a new path or remove it",
                    self.repo
                ));
            }
            return Ok(());
        }
        if !expected.is_empty() {
            if let Some(fingerprint) = repository_fingerprint(path) {
                if fingerprint != expected {
                    return Err(format!(
                        "a different repository is mounted at {}",
                        self.repo
                    ));
                }
            }
        }
        Ok(())
    }

    /// Writes atomically and keeps the previous file as `kustos.json.bak`.
    /// Password key bindings that are still on disk are kept if this in-memory
    /// copy has lost them (a stale second window, a slow update, ...).
    pub fn save(&mut self) -> std::io::Result<()> {
        self.write(true)
    }

    /// Writes defaults (or any deliberate reset) without merging old keys.
    pub fn save_fresh(&mut self) -> std::io::Result<()> {
        self.write(false)
    }

    fn write(&mut self, preserve_keys: bool) -> std::io::Result<()> {
        if preserve_keys {
            self.preserve_keys_from_disk();
        }
        let path = paths::config_file();
        paths::ensure_parent(&path)?;
        let text = serde_json::to_string_pretty(self).unwrap_or_default();
        let temporary = path.with_extension("json.tmp");
        {
            let mut file = fs::File::create(&temporary)?;
            file.write_all(text.as_bytes())?;
            file.sync_all()?;
        }
        // Do not replace a good backup with a corrupt primary.
        let primary_is_valid = fs::read_to_string(&path)
            .ok()
            .and_then(|text| serde_json::from_str::<Config>(&text).ok())
            .is_some();
        if primary_is_valid {
            let _ = fs::copy(&path, backup_path(&path));
        }
        fs::rename(&temporary, &path)
    }

    fn preserve_keys_from_disk(&mut self) {
        let path = paths::config_file();
        let Ok(text) = fs::read_to_string(&path) else {
            return;
        };
        let Ok(disk) = serde_json::from_str::<Config>(&text) else {
            return;
        };
        self.merge_keys_from(&disk);
    }

    /// Keeps key bindings that are already on disk when this in-memory copy
    /// has lost them. A different repository means old key IDs are meaningless.
    fn merge_keys_from(&mut self, disk: &Config) {
        if disk.repo != self.repo {
            return;
        }
        for rule in &mut self.rules {
            if !rule.key_id.is_empty() {
                continue;
            }
            if let Some(old) = disk.rules.iter().find(|old| old.id == rule.id) {
                if !old.key_id.is_empty() {
                    rule.key_id = old.key_id.clone();
                }
            }
        }
        if self.common_key_id.is_empty() {
            self.common_key_id = disk.common_key_id.clone();
        }
    }

    fn normalize(&mut self, migrate_legacy: bool) {
        self.repo = self.repo.trim_end_matches('/').to_string();

        // Migration: one global scope becomes one default rule. The legacy
        // fields are consumed here and dropped on the next save.
        if migrate_legacy && self.rules.is_empty() {
            let mut rule = BackupRule::new("default".to_string(), "Home".to_string());
            rule.enabled = true;
            rule.sources = std::mem::take(&mut self.sources);
            rule.excludes = std::mem::take(&mut self.excludes);
            rule.schedule.enabled = self.schedule_enabled;
            rule.schedule.minutes = self.schedule_minutes;
            self.rules.push(rule);
        }
        self.sources.clear();
        self.excludes.clear();

        // Keep only valid, uniquely identified rules.
        let mut seen: Vec<String> = Vec::new();
        let mut next = 1u32;
        for rule in &mut self.rules {
            rule.normalize();
            if rule.id.is_empty() {
                rule.id = format!("r{next}");
                next += 1;
            }
            while rule.id.is_empty() || seen.contains(&rule.id) {
                rule.id = format!("r{next}");
                next += 1;
            }
            seen.push(rule.id.clone());
        }
        self.rules.retain(|rule| !rule.id.is_empty());

        self.keep_hourly = self.keep_hourly.clamp(1, 24 * 30);
        self.keep_daily = self.keep_daily.min(3650);
        self.keep_weekly = self.keep_weekly.min(520);
        self.lock_timeout_minutes = self.lock_timeout_minutes.min(7 * 24 * 60);
        if self.password_mode != PASSWORD_MODE_COMMON {
            self.password_mode = PASSWORD_MODE_PER_RULE.to_string();
        }
        self.common_key_id = self
            .common_key_id
            .chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .collect();
        // In common mode every rule must point at the common key; otherwise
        // the lock panel says "no password yet" while Settings says the
        // common password is set. If only a rule key survives, adopt it as
        // the common key.
        if self.common_mode() {
            if self.common_key_id.is_empty() {
                if let Some(key) = self
                    .rules
                    .iter()
                    .find(|rule| !rule.key_id.is_empty())
                    .map(|rule| rule.key_id.clone())
                {
                    self.common_key_id = key;
                }
            }
            if !self.common_key_id.is_empty() {
                let common = self.common_key_id.clone();
                for rule in &mut self.rules {
                    rule.key_id = common.clone();
                }
            }
        }
    }

    pub fn rule(&self, id: &str) -> Option<&BackupRule> {
        self.rules.iter().find(|rule| rule.id == id)
    }

    pub fn rule_mut(&mut self, id: &str) -> Option<&mut BackupRule> {
        self.rules.iter_mut().find(|rule| rule.id == id)
    }

    fn next_rule_id(&self) -> String {
        let mut number = self.rules.len() + 1;
        loop {
            let candidate = format!("r{number}");
            if !self.rules.iter().any(|rule| rule.id == candidate) {
                return candidate;
            }
            number += 1;
        }
    }

    pub fn add_rule(&mut self, name: String) -> String {
        let id = self.next_rule_id();
        let mut rule = BackupRule::new(id.clone(), name);
        rule.schedule.enabled = false;
        // In common-password mode a new rule joins the existing common key
        // immediately, so opening it asks for the same password.
        if self.common_mode() {
            if let Some(common) = self.common_key_id() {
                rule.key_id = common.to_string();
            }
        }
        self.rules.push(rule);
        let _ = self.save();
        id
    }

    pub fn common_mode(&self) -> bool {
        self.password_mode == PASSWORD_MODE_COMMON
    }

    pub fn common_key_id(&self) -> Option<&str> {
        if self.common_key_id.is_empty() {
            None
        } else {
            Some(&self.common_key_id)
        }
    }

    pub fn set_password_mode(&mut self, mode: &str) {
        self.password_mode = if mode == PASSWORD_MODE_COMMON {
            PASSWORD_MODE_COMMON.to_string()
        } else {
            PASSWORD_MODE_PER_RULE.to_string()
        };
        let _ = self.save();
    }

    /// Makes every rule use the same rustic key as its viewing password.
    pub fn bind_common_key(&mut self, key_id: String) {
        let key_id: String = key_id
            .chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .collect();
        self.common_key_id = key_id.clone();
        for rule in &mut self.rules {
            rule.key_id = key_id.clone();
        }
        let _ = self.save();
    }

    pub fn remove_rule(&mut self, id: &str) {
        self.rules.retain(|rule| rule.id != id);
        let _ = self.save();
    }

    pub fn set_rule_name(&mut self, id: &str, name: String) {
        if let Some(rule) = self.rule_mut(id) {
            rule.name = name.trim().to_string();
            let _ = self.save();
        }
    }

    pub fn set_rule_one_file_system(&mut self, id: &str, enabled: bool) {
        if let Some(rule) = self.rule_mut(id) {
            rule.one_file_system = enabled;
            let _ = self.save();
        }
    }

    pub fn set_rule_exclude_larger(&mut self, id: &str, size: String) {
        if let Some(rule) = self.rule_mut(id) {
            rule.exclude_larger = size.trim().to_string();
            let _ = self.save();
        }
    }

    pub fn set_rule_enabled(&mut self, id: &str, enabled: bool) {
        if let Some(rule) = self.rule_mut(id) {
            rule.enabled = enabled;
            let _ = self.save();
        }
    }

    /// Binds a rustic key to a rule after its password was created or changed.
    pub fn set_rule_key_id(&mut self, id: &str, key_id: String) {
        if let Some(rule) = self.rule_mut(id) {
            rule.key_id = key_id
                .chars()
                .filter(|c| c.is_ascii_alphanumeric())
                .collect();
            let _ = self.save();
        }
    }

    /// Changes the repository folder. Keys belong to one repository, so every
    /// rule loses its key binding and the caller must remove the credentials.
    pub fn set_repo(&mut self, repo: String) {
        let repo = repo.trim_end_matches('/').to_string();
        if repo == self.repo {
            return;
        }
        self.repo_fingerprint.clear();
        self.repo = repo;
        self.common_key_id.clear();
        for rule in &mut self.rules {
            rule.key_id.clear();
        }
        let _ = self.save();
    }

    pub fn set_lock_timeout_minutes(&mut self, minutes: u32) {
        self.lock_timeout_minutes = minutes.min(7 * 24 * 60);
        let _ = self.save();
    }

    pub fn add_rule_source(&mut self, id: &str, path: String) {
        let path = path.trim().to_string();
        if path.is_empty() {
            return;
        }
        if let Some(rule) = self.rule_mut(id) {
            if !rule.sources.contains(&path) {
                rule.sources.push(path);
                let _ = self.save();
            }
        }
    }

    pub fn remove_rule_source(&mut self, id: &str, index: usize) {
        if let Some(rule) = self.rule_mut(id) {
            if index < rule.sources.len() {
                rule.sources.remove(index);
                let _ = self.save();
            }
        }
    }

    /// Adds an excluded folder or a glob pattern such as `*.log`,
    /// `node_modules` or `~/.cache/**`.
    pub fn add_rule_exclude(&mut self, id: &str, pattern: String) {
        let pattern = pattern.trim().to_string();
        if pattern.is_empty() {
            return;
        }
        if let Some(rule) = self.rule_mut(id) {
            if !rule.excludes.contains(&pattern) {
                rule.excludes.push(pattern);
                let _ = self.save();
            }
        }
    }

    /// Adds the cache/index paths that are never useful in a backup. Returns
    /// how many new exclusions were actually added.
    pub fn add_recommended_excludes(&mut self, id: &str) -> i32 {
        let recommended = recommended_excludes();
        let Some(rule) = self.rule_mut(id) else {
            return 0;
        };
        let mut added = 0;
        for path in recommended {
            if !rule.excludes.contains(&path) {
                rule.excludes.push(path);
                added += 1;
            }
        }
        if added > 0 {
            let _ = self.save();
        }
        added
    }

    pub fn remove_rule_exclude(&mut self, id: &str, index: usize) {
        if let Some(rule) = self.rule_mut(id) {
            if index < rule.excludes.len() {
                rule.excludes.remove(index);
                let _ = self.save();
            }
        }
    }

    pub fn set_rule_schedule_enabled(&mut self, id: &str, enabled: bool) {
        if let Some(rule) = self.rule_mut(id) {
            rule.schedule.enabled = enabled;
            let _ = self.save();
        }
    }

    pub fn set_rule_schedule_mode(&mut self, id: &str, mode: String) {
        if let Some(rule) = self.rule_mut(id) {
            if matches!(
                mode.as_str(),
                SCHEDULE_INTERVAL | SCHEDULE_DAILY | SCHEDULE_WEEKLY
            ) {
                rule.schedule.mode = mode;
                let _ = self.save();
            }
        }
    }

    pub fn set_rule_schedule_minutes(&mut self, id: &str, minutes: i32) {
        if let Some(rule) = self.rule_mut(id) {
            rule.schedule.minutes = (minutes.max(5) as u32).min(7 * 24 * 60);
            let _ = self.save();
        }
    }

    pub fn set_rule_schedule_time(&mut self, id: &str, time: String) {
        if let Some(rule) = self.rule_mut(id) {
            rule.schedule.time = normalize_time(&time);
            let _ = self.save();
        }
    }

    pub fn set_rule_weekday(&mut self, id: &str, day: i32, enabled: bool) {
        if !(1..=7).contains(&day) {
            return;
        }
        if let Some(rule) = self.rule_mut(id) {
            let day = day as u32;
            rule.schedule.weekdays.retain(|known| *known != day);
            if enabled {
                rule.schedule.weekdays.push(day);
            }
            // A weekly rule needs at least one day; re-check the last one
            // instead of writing an empty OnCalendar= line.
            if rule.schedule.weekdays.is_empty() {
                rule.schedule.weekdays.push(day);
            }
            rule.schedule.weekdays.sort_unstable();
            let _ = self.save();
        }
    }

    /// Sources that still exist, used by every backup so that a missing folder
    /// (an unmounted disk, a removed directory) is skipped with a warning
    /// instead of failing the run.
    pub fn existing_sources(rule: &BackupRule) -> (Vec<String>, Vec<String>) {
        let mut existing = Vec::new();
        let mut missing = Vec::new();
        for source in &rule.sources {
            if std::path::Path::new(source).exists() {
                existing.push(source.clone());
            } else {
                missing.push(source.clone());
            }
        }
        (existing, missing)
    }

    /// Exclusion patterns as `rustic --glob` arguments. `~/…` is expanded to
    /// the home directory, a leading `!` is removed (the caller adds the
    /// negation itself) and everything else is passed through as a glob.
    pub fn expanded_excludes(rule: &BackupRule) -> Vec<String> {
        let home = paths::home();
        rule.excludes
            .iter()
            .filter_map(|pattern| {
                let pattern = pattern.trim();
                let pattern = pattern.strip_prefix('!').unwrap_or(pattern);
                if pattern.is_empty() {
                    return None;
                }
                if pattern == "~" {
                    return Some(home.to_string_lossy().into_owned());
                }
                if let Some(rest) = pattern.strip_prefix("~/") {
                    return Some(home.join(rest).to_string_lossy().into_owned());
                }
                Some(pattern.to_string())
            })
            .collect()
    }
}

/// Generates a random repository password. It is used when a backup has to
/// initialize a repository without asking the user: the generated key is saved
/// as a systemd credential and can later be replaced by a per-rule viewing
/// password.
pub fn generate_password() -> String {
    let mut bytes = [0u8; 32];
    use std::io::Read;
    let filled = fs::File::open("/dev/urandom")
        .and_then(|mut file| file.read_exact(&mut bytes))
        .is_ok();
    if !filled {
        // /dev/urandom is always available on Linux; this is only a last
        // resort so a backup is not blocked by an unusual sandbox.
        let seed = format!(
            "{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|elapsed| elapsed.as_nanos())
                .unwrap_or(0)
        );
        for (index, byte) in bytes.iter_mut().enumerate() {
            *byte = seed.as_bytes()[index % seed.len()];
        }
    }
    let alphabet = b"abcdefghijkmnopqrstuvwxyzABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    bytes
        .iter()
        .map(|byte| alphabet[(*byte as usize) % alphabet.len()] as char)
        .collect()
}

/// "auto" follows the system locale; only languages with a compiled
/// translation are allowed, everything else falls back to English.
pub fn resolve_language(requested: &str) -> String {
    if requested == "en" || requested == "ru" {
        return requested.to_string();
    }
    let locale = std::env::var("LC_ALL")
        .or_else(|_| std::env::var("LC_MESSAGES"))
        .or_else(|_| std::env::var("LANG"))
        .unwrap_or_default();
    let code = locale.get(..2).unwrap_or("").to_lowercase();
    if code == "ru" {
        "ru".to_string()
    } else {
        "en".to_string()
    }
}

fn backup_path(path: &Path) -> PathBuf {
    path.with_extension("json.bak")
}

fn read_backup(path: &Path) -> Option<Config> {
    let text = fs::read_to_string(backup_path(path)).ok()?;
    let mut config = serde_json::from_str::<Config>(&text).ok()?;
    config.normalize(false);
    Some(config)
}

/// Returns the source that contains the repository, if any. Backing up the
/// repository into itself grows without bound, so the caller must exclude it.
pub fn repository_inside_source(rule: &BackupRule, repo: &str) -> Option<String> {
    let repo = fs::canonicalize(repo).unwrap_or_else(|_| PathBuf::from(repo));
    for source in &rule.sources {
        let source_path = fs::canonicalize(source).unwrap_or_else(|_| PathBuf::from(source));
        if repo == source_path || repo.starts_with(&source_path) {
            return Some(source.clone());
        }
    }
    None
}

/// Exclusion patterns for one run, with the repository folder added when the
/// repository lives inside one of the rule's sources. The second value is a
/// warning for the job result.
pub fn effective_excludes(rule: &BackupRule, repo: &str) -> (Vec<String>, Option<String>) {
    let mut patterns = Config::expanded_excludes(rule);
    let warning = repository_inside_source(rule, repo).map(|source| {
        if !patterns.iter().any(|pattern| pattern == repo) {
            patterns.push(repo.to_string());
        }
        format!("the repository is inside the source {source}; it was excluded")
    });
    (patterns, warning)
}

/// A stable fingerprint of the repository's encrypted `config` file. The file
/// itself cannot be parsed without the key, but its bytes identify the
/// repository well enough to notice a missing or swapped drive.
pub fn repository_fingerprint(path: &Path) -> Option<String> {
    let bytes = fs::read(path.join("config")).ok()?;
    // FNV-1a; this is an identity check, not a security boundary.
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for byte in bytes {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100_0000_01b3);
    }
    Some(format!("{hash:016x}"))
}

/// Remembers a freshly initialized repository so a later missing path is
/// treated as an error instead of a new repository.
pub fn remember_repository_fingerprint(repo_path: &str) {
    let Some(fingerprint) = repository_fingerprint(Path::new(repo_path)) else {
        return;
    };
    let mut config = Config::load();
    if config.repo != repo_path || config.repo_fingerprint == fingerprint {
        return;
    }
    config.repo_fingerprint = fingerprint;
    if let Err(error) = config.save() {
        crate::logln(&format!(
            "config: cannot remember the repository fingerprint: {error}"
        ));
    }
}

/// Stores the result of a repository integrity check without clobbering
/// settings changed by the GUI while the check was running: the current file
/// is re-read, only the check fields are replaced, and it is saved atomically.
pub fn record_repository_check(ok: bool, detail: &str) {
    let mut config = Config::load();
    config.last_check = chrono::Local::now().to_rfc3339();
    config.last_check_ok = ok;
    config.last_check_detail = detail.chars().take(500).collect();
    if let Err(error) = config.save() {
        crate::logln(&format!("config: cannot save the repository check result: {error}"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(text: &str) -> Config {
        let mut config: Config = serde_json::from_str(text).expect("valid config");
        let has_rules = serde_json::from_str::<serde_json::Value>(text)
            .map(|value| value.get("rules").is_some())
            .unwrap_or(false);
        config.normalize(!has_rules);
        config
    }

    fn repo_scratch(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "kustos-repo-{}-{name}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&path);
        path
    }

    #[test]
    fn backup_options_round_trip() {
        let mut rule = BackupRule::new("r1".into(), "R".into());
        rule.one_file_system = true;
        rule.exclude_larger = " 2G ".into();
        rule.normalize();
        assert!(rule.one_file_system);
        assert_eq!(rule.exclude_larger, "2G");

        let text = serde_json::to_string(&rule).unwrap();
        let loaded: BackupRule = serde_json::from_str(&text).unwrap();
        assert!(loaded.one_file_system);
        assert_eq!(loaded.exclude_larger, "2G");
    }

    #[test]
    fn recommended_excludes_are_added_once() {
        let mut config = Config::default();
        assert!(config.rules[0]
            .excludes
            .iter()
            .any(|path| path.ends_with("baloo")));
        config.rules[0].excludes.clear();
        let id = config.rules[0].id.clone();
        let added = config.add_recommended_excludes(&id);
        assert_eq!(added, recommended_excludes().len() as i32);
        assert_eq!(config.add_recommended_excludes(&id), 0);
        for path in recommended_excludes() {
            assert!(config.rules[0].excludes.contains(&path));
        }
    }

    #[test]
    fn a_repository_inside_a_source_is_detected() {
        let root = repo_scratch("inside");
        let source = root.join("source");
        let repo = source.join("repo");
        std::fs::create_dir_all(&repo).unwrap();
        let mut rule = BackupRule::new("r".into(), "R".into());
        rule.sources = vec![source.to_string_lossy().into_owned()];

        assert_eq!(
            repository_inside_source(&rule, &repo.to_string_lossy()),
            Some(source.to_string_lossy().into_owned())
        );

        let outside = root.join("other");
        std::fs::create_dir_all(&outside).unwrap();
        assert!(repository_inside_source(&rule, &outside.to_string_lossy()).is_none());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn a_vanished_repository_is_not_reinitialized() {
        let path = repo_scratch("guard");
        let mut config = Config::default();
        config.repo = path.to_string_lossy().into_owned();
        config.repo_fingerprint = "abc123".into();

        // The drive is gone: do not treat the missing path as a new repo.
        assert!(config.verify_repository().is_err());
        std::fs::create_dir_all(&path).unwrap();
        std::fs::write(path.join("config"), r#"{"id":"abc123"}"#).unwrap();
        config.repo_fingerprint = repository_fingerprint(&path).unwrap();
        assert!(config.verify_repository().is_ok());

        // A different repository at the same path is refused too.
        std::fs::write(path.join("config"), r#"{"id":"other"}"#).unwrap();
        assert!(config.verify_repository().is_err());

        // Without a remembered fingerprint, a non-empty folder is not
        // initialized either.
        config.repo_fingerprint.clear();
        std::fs::remove_file(path.join("config")).unwrap();
        std::fs::write(path.join("stray"), b"x").unwrap();
        assert!(config.verify_repository().is_err());

        let _ = std::fs::remove_dir_all(path);
    }

    #[test]
    fn changing_the_repository_forgets_the_remembered_fingerprint() {
        let mut config = Config::default();
        config.repo_fingerprint = "abc123".into();
        let same = config.repo.clone();
        config.set_repo(same);
        assert_eq!(config.repo_fingerprint, "abc123");

        config.set_repo("/tmp/kustos-other-repo".into());
        assert!(config.repo_fingerprint.is_empty());
    }

    #[test]
    fn legacy_scope_becomes_one_rule() {
        let config = parse(
            r#"{
                "repo": "/tmp/repo",
                "sources": ["/home/u/a", "/home/u/b"],
                "excludes": ["/home/u/a/cache"],
                "language": "en",
                "notifications": true,
                "schedule_enabled": true,
                "schedule_minutes": 90,
                "keep_hourly": 24,
                "keep_daily": 30,
                "keep_weekly": 12
            }"#,
        );
        assert_eq!(config.rules.len(), 1);
        let rule = &config.rules[0];
        assert_eq!(rule.id, "default");
        assert_eq!(rule.sources, ["/home/u/a", "/home/u/b"]);
        assert_eq!(rule.excludes, ["/home/u/a/cache"]);
        assert!(rule.schedule.enabled);
        assert_eq!(rule.schedule.minutes, 90);
    }

    #[test]
    fn rules_round_trip_without_legacy_fields() {
        let mut config = Config::default();
        let mut rule = BackupRule::new("work".to_string(), "Work".to_string());
        rule.sources = vec!["/home/u/work".to_string()];
        rule.excludes = vec!["*.tmp".to_string()];
        rule.schedule.mode = SCHEDULE_WEEKLY.to_string();
        rule.schedule.time = "7:5".to_string();
        rule.schedule.weekdays = vec![7, 1, 1];
        config.rules.push(rule);
        config.normalize(false);

        let text = serde_json::to_string(&config).unwrap();
        let value: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert!(value.get("sources").is_none());
        assert!(value.get("schedule_enabled").is_none());
        assert!(value.get("rules").is_some());
        assert!(value["rules"][0].get("sources").is_some());
        let loaded = parse(&text);
        let rule = loaded.rules.last().unwrap();
        assert_eq!(rule.name, "Work");
        assert_eq!(rule.sources, ["/home/u/work"]);
        assert_eq!(rule.schedule.mode, SCHEDULE_WEEKLY);
        assert_eq!(rule.schedule.time, "07:05");
        assert_eq!(rule.schedule.weekdays, [1, 7]);
    }

    #[test]
    fn explicit_empty_rule_list_is_not_migrated() {
        let config = parse(
            r#"{
                "repo": "/tmp/repo",
                "rules": [],
                "language": "en",
                "notifications": true,
                "keep_hourly": 24,
                "keep_daily": 30,
                "keep_weekly": 12
            }"#,
        );
        assert!(config.rules.is_empty());
    }

    #[test]
    fn rule_ids_stay_unique() {
        let mut config = Config::default();
        config.rules.clear();
        config.rules.push(BackupRule::new("r2".to_string(), "Two".to_string()));
        config.rules.push(BackupRule::new(String::new(), "No id".to_string()));
        config.rules.push(BackupRule::new("r2".to_string(), "Duplicate".to_string()));
        config.normalize(false);
        let ids: Vec<&str> = config.rules.iter().map(|rule| rule.id.as_str()).collect();
        let unique: std::collections::HashSet<&&str> = ids.iter().collect();
        assert_eq!(ids.len(), unique.len());
        assert!(ids.iter().all(|id| !id.is_empty()));
        assert_eq!(ids[0], "r2");
        assert_eq!(config.next_rule_id(), "r4");
    }

    #[test]
    fn rule_key_ids_round_trip() {
        let mut config = Config::default();
        config.rules[0].key_id = "a1b2c3d4".to_string();
        let text = serde_json::to_string(&config).unwrap();
        let loaded = parse(&text);
        assert_eq!(loaded.rules[0].key_id, "a1b2c3d4");
        assert!(loaded.rules[0].has_password());
        assert!(!config.rules[0].key_id.is_empty());
    }

    #[test]
    fn changing_the_repository_forgets_key_bindings() {
        let mut config = Config::default();
        config.rules[0].key_id = "deadbeef".to_string();
        config.set_repo("/tmp/other-repo/".to_string());
        assert_eq!(config.repo, "/tmp/other-repo");
        assert!(config.rules[0].key_id.is_empty());
        assert!(!config.rules[0].has_password());
    }

    #[test]
    fn lock_timeout_defaults_and_is_capped() {
        let mut config = Config::default();
        assert_eq!(config.lock_timeout_minutes, 15);
        config.lock_timeout_minutes = u32::MAX;
        config.normalize(false);
        assert_eq!(config.lock_timeout_minutes, 7 * 24 * 60);
        config.set_lock_timeout_minutes(0);
        assert_eq!(config.lock_timeout_minutes, 0);
    }

    #[test]
    fn old_configs_get_a_lock_timeout() {
        let config = parse(
            r#"{
                "repo": "/tmp/repo",
                "rules": [],
                "language": "en",
                "notifications": true,
                "keep_hourly": 24,
                "keep_daily": 30,
                "keep_weekly": 12
            }"#,
        );
        assert_eq!(config.lock_timeout_minutes, 15);
    }

    #[test]
    fn stale_config_does_not_lose_key_bindings() {
        let mut disk = Config::default();
        disk.repo = "/data/Backup/kustos".to_string();
        disk.common_key_id = "abc123".to_string();
        disk.rules[0].key_id = "abc123".to_string();

        let mut stale = Config::default();
        stale.repo = disk.repo.clone();
        stale.common_key_id.clear();
        stale.rules[0].key_id.clear();
        stale.merge_keys_from(&disk);

        assert_eq!(stale.common_key_id, "abc123");
        assert_eq!(stale.rules[0].key_id, "abc123");
    }

    #[test]
    fn common_mode_propagates_the_common_key_to_every_rule() {
        let mut config = Config::default();
        config.password_mode = PASSWORD_MODE_COMMON.to_string();
        config.common_key_id = "abc123".to_string();
        config.rules[0].key_id.clear();
        config.normalize(false);
        assert_eq!(config.rules[0].key_id, "abc123");
    }

    #[test]
    fn password_mode_and_common_key_round_trip() {
        let mut config = Config::default();
        assert_eq!(config.password_mode, PASSWORD_MODE_PER_RULE);
        assert!(!config.common_mode());
        assert!(config.common_key_id().is_none());

        config.rules.push(BackupRule::new("r2".into(), "Second".into()));
        config.set_password_mode(PASSWORD_MODE_COMMON);
        config.bind_common_key("a1b2c3d4".into());
        assert!(config.common_mode());
        assert_eq!(config.common_key_id(), Some("a1b2c3d4"));
        assert!(config.rules.iter().all(|rule| rule.key_id == "a1b2c3d4"));

        let added = config.add_rule("Third".into());
        let rule = config.rule(&added).unwrap();
        assert_eq!(rule.key_id, "a1b2c3d4");

        let text = serde_json::to_string(&config).unwrap();
        let loaded = parse(&text);
        assert_eq!(loaded.password_mode, PASSWORD_MODE_COMMON);
        assert_eq!(loaded.common_key_id, "a1b2c3d4");

        config.set_password_mode("something-else");
        assert_eq!(config.password_mode, PASSWORD_MODE_PER_RULE);
    }

    #[test]
    fn repository_check_fields_round_trip() {
        let config = Config {
            last_check: "2026-11-22T11:46:00+07:00".into(),
            last_check_ok: true,
            last_check_detail: "ok".into(),
            ..Config::default()
        };
        let loaded = parse(&serde_json::to_string(&config).unwrap());
        assert_eq!(loaded.last_check, config.last_check);
        assert!(loaded.last_check_ok);
        assert_eq!(loaded.last_check_detail, "ok");
    }

    #[test]
    fn changing_the_repository_clears_the_common_key() {
        let mut config = Config::default();
        config.set_password_mode(PASSWORD_MODE_COMMON);
        config.bind_common_key("deadbeef".into());
        config.set_repo("/tmp/other-repo".into());
        assert!(config.common_key_id().is_none());
        assert!(config.rules.iter().all(|rule| rule.key_id.is_empty()));
    }

    #[test]
    fn excludes_expand_home_and_strip_bang() {
        let mut rule = BackupRule::new("r".into(), "R".into());
        rule.excludes = vec![
            "~/cache".into(),
            "!*.log".into(),
            "node_modules".into(),
            "  ".into(),
        ];
        let patterns = Config::expanded_excludes(&rule);
        let home = paths::home();
        assert_eq!(patterns[0], home.join("cache").to_string_lossy());
        assert_eq!(patterns[1], "*.log");
        assert_eq!(patterns[2], "node_modules");
        assert_eq!(patterns.len(), 3);
    }
}
