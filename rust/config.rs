// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 B612S <sem.alimov@inbox.ru>

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::paths;

/// Persisted settings, a single JSON document in
/// `$XDG_CONFIG_HOME/kustos/kustos.json`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub repo: String,
    pub sources: Vec<String>,
    pub excludes: Vec<String>,
    pub language: String,
    pub notifications: bool,
    pub schedule_enabled: bool,
    pub schedule_minutes: u32,
    pub keep_hourly: u32,
    pub keep_daily: u32,
    pub keep_weekly: u32,
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
        let share = home.join(".local/share");
        Self {
            repo: repo.to_string_lossy().into_owned(),
            sources: default_sources(),
            excludes: vec![
                share.join("Steam").to_string_lossy().into_owned(),
                share.join("twintaillauncher").to_string_lossy().into_owned(),
                share.join("Trash").to_string_lossy().into_owned(),
            ],
            language: "auto".into(),
            notifications: true,
            schedule_enabled: false,
            schedule_minutes: 60,
            keep_hourly: 24,
            keep_daily: 30,
            keep_weekly: 12,
        }
    }
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

impl Config {
    pub fn load() -> Self {
        let path = paths::config_file();
        match fs::read_to_string(&path) {
            Ok(text) => match serde_json::from_str::<Config>(&text) {
                Ok(mut config) => {
                    config.normalize();
                    config
                }
                Err(error) => {
                    crate::logln(&format!("config: cannot parse {}: {error}", path.display()));
                    Config::default()
                }
            },
            Err(_) => {
                let config = Config::default();
                let _ = config.save();
                config
            }
        }
    }

    pub fn save(&self) -> std::io::Result<()> {
        let path = paths::config_file();
        paths::ensure_parent(&path)?;
        let text = serde_json::to_string_pretty(self).unwrap_or_default();
        fs::write(path, text)
    }

    fn normalize(&mut self) {
        self.repo = self.repo.trim_end_matches('/').to_string();
        self.sources.retain(|s| !s.trim().is_empty());
        self.excludes.retain(|s| !s.trim().is_empty());
        if self.schedule_minutes < 5 {
            self.schedule_minutes = 5;
        }
        if self.schedule_minutes > 7 * 24 * 60 {
            self.schedule_minutes = 7 * 24 * 60;
        }
        self.keep_hourly = self.keep_hourly.clamp(1, 24 * 30);
        self.keep_daily = self.keep_daily.min(3650);
        self.keep_weekly = self.keep_weekly.min(520);
    }

    /// Sources that still exist; used by every backup so that a missing folder
    /// (an unmounted disk, a removed directory) is skipped with a warning
    /// instead of failing the run.
    pub fn existing_sources(&self) -> (Vec<String>, Vec<String>) {
        let mut existing = Vec::new();
        let mut missing = Vec::new();
        for source in &self.sources {
            if std::path::Path::new(source).exists() {
                existing.push(source.clone());
            } else {
                missing.push(source.clone());
            }
        }
        (existing, missing)
    }

    pub fn add_source(&mut self, path: String) {
        if !path.is_empty() && !self.sources.contains(&path) {
            self.sources.push(path.clone());
            let _ = self.save();
        }
    }

    pub fn remove_source(&mut self, index: usize) {
        if index < self.sources.len() {
            self.sources.remove(index);
            let _ = self.save();
        }
    }

    pub fn add_exclude(&mut self, path: String) {
        if !path.is_empty() && !self.excludes.contains(&path) {
            self.excludes.push(path.clone());
            let _ = self.save();
        }
    }

    pub fn remove_exclude(&mut self, index: usize) {
        if index < self.excludes.len() {
            self.excludes.remove(index);
            let _ = self.save();
        }
    }
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

/// Generates a fresh repository key. It is not persisted anywhere: the caller
/// shows it once and keeps it in memory for the session.
pub fn generate_password() -> String {
    let mut bytes = [0u8; 24];
    use std::io::Read;
    if let Ok(mut urandom) = fs::File::open("/dev/urandom") {
        let _ = urandom.read_exact(&mut bytes);
    }
    let alphabet = b"abcdefghijkmnopqrstuvwxyzABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    bytes
        .iter()
        .map(|b| alphabet[(*b as usize) % alphabet.len()] as char)
        .collect()
}
