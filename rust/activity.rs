// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 B612S <sem.alimov@inbox.ru>

//! Idle detection for the automatic re-lock.
//!
//! The Qt side installs an application-wide event filter and stores the
//! timestamp of the last mouse, keyboard, wheel or focus event. A small
//! background thread compares it with the configured timeout and forgets the
//! unlocked passwords, so the lock works even while QML is busy.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::vault::Vault;

extern "C" {
    fn kustos_install_activity_filter();
    fn kustos_last_activity_ms() -> i64;
}

/// Installs the Qt event filter. Must be called after `QmlEngine::new()`
/// created the `QGuiApplication`.
pub fn install() {
    unsafe { kustos_install_activity_filter() };
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_millis() as i64)
        .unwrap_or(0)
}

pub fn start_idle_guard(vault: Vault) {
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_secs(1));
        let timeout = vault.timeout_minutes();
        if timeout == 0 || vault.is_empty() {
            continue;
        }
        let last = unsafe { kustos_last_activity_ms() };
        if last <= 0 {
            continue;
        }
        let idle = now_ms().saturating_sub(last);
        if idle >= i64::from(timeout) * 60_000 && vault.lock_all() {
            crate::logln(&format!(
                "vault: locked after {timeout} min without activity"
            ));
        }
    });
}
