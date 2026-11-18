// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 B612S <sem.alimov@inbox.ru>

use std::env;
use std::path::{Path, PathBuf};

pub fn home() -> PathBuf {
    PathBuf::from(env::var("HOME").unwrap_or_else(|_| "/".to_string()))
}

fn xdg(variable: &str, fallback: &str) -> PathBuf {
    env::var(variable)
        .map(PathBuf::from)
        .unwrap_or_else(|_| home().join(fallback))
}

pub fn config_dir() -> PathBuf {
    xdg("XDG_CONFIG_HOME", ".config").join("kustos")
}

pub fn cache_dir() -> PathBuf {
    xdg("XDG_CACHE_HOME", ".cache").join("kustos")
}

pub fn data_dir() -> PathBuf {
    xdg("XDG_DATA_HOME", ".local/share").join("kustos")
}

pub fn config_file() -> PathBuf {
    config_dir().join("kustos.json")
}

pub fn log_file() -> PathBuf {
    cache_dir().join("kustos.log")
}

pub fn staging_root() -> PathBuf {
    cache_dir().join("restore")
}

pub fn ensure_parent(path: &Path) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    Ok(())
}
