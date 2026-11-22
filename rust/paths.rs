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
    #[cfg(test)]
    isolate_test_paths();
    xdg("XDG_CONFIG_HOME", ".config").join("kustos")
}

pub fn cache_dir() -> PathBuf {
    #[cfg(test)]
    isolate_test_paths();
    xdg("XDG_CACHE_HOME", ".cache").join("kustos")
}

/// Tests must never write the real user configuration. Unless a test already
/// selected its own temporary tree (the password tests do), redirect the XDG
/// paths to a process-specific temporary directory.
#[cfg(test)]
fn isolate_test_paths() {
    use std::sync::Once;
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        if env::var_os("KUSTOS_TEST_XDG_READY").is_some() {
            return;
        }
        let root = env::temp_dir().join(format!("kustos-tests-{}", std::process::id()));
        env::set_var("XDG_CONFIG_HOME", root.join("config"));
        env::set_var("XDG_CACHE_HOME", root.join("cache"));
        env::set_var("KUSTOS_TEST_XDG_READY", "1");
    });
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

/// Files extracted only to be handed to the default application. They live
/// under the system temporary directory so a reboot (or the regular cleanup
/// below) removes them, while the launched application can still read them
/// after `xdg-open` returns. The UID keeps users on a shared `/tmp` apart.
pub fn open_root() -> PathBuf {
    env::temp_dir().join(format!("kustos-open-{}", unsafe { libc::getuid() }))
}

pub fn ensure_parent(path: &Path) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_files_live_in_a_user_scoped_temp_directory() {
        let root = open_root();
        assert!(root.starts_with(env::temp_dir()));
        assert!(root
            .file_name()
            .map(|name| name.to_string_lossy().starts_with("kustos-open-"))
            .unwrap_or(false));
    }
}
