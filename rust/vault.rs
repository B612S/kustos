// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 B612S <sem.alimov@inbox.ru>

//! Unlocked backup passwords live in memory only.
//!
//! Every rule can have its own password, which is a real `rustic` key of the
//! repository. The password is asked for when the user opens that backup, not
//! when the application starts, and it is dropped again after a configurable
//! period without activity. Nothing is written to `kustos.json`; the systemd
//! units get their own encrypted credentials (see `schedule.rs`).

use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

const DEFAULT_TIMEOUT_MINUTES: u32 = 15;

#[derive(Default)]
struct VaultState {
    /// Rule id -> repository password. Every password opens the same
    /// repository (they are different keys of it), so any entry can be used
    /// for repository-wide operations such as listing or pruning.
    passwords: HashMap<String, String>,
    generation: u64,
}

#[derive(Clone)]
pub struct Vault {
    state: Arc<Mutex<VaultState>>,
    timeout_minutes: Arc<AtomicU32>,
}

impl Default for Vault {
    fn default() -> Self {
        Self {
            state: Arc::new(Mutex::new(VaultState::default())),
            timeout_minutes: Arc::new(AtomicU32::new(DEFAULT_TIMEOUT_MINUTES)),
        }
    }
}

impl Vault {
    pub fn password_for(&self, rule_id: &str) -> Option<String> {
        let state = self.state.lock().ok()?;
        state.passwords.get(rule_id).cloned()
    }

    /// Any unlocked password, used for operations that are not tied to a
    /// single rule (listing snapshots, pruning, restoring).
    pub fn any_password(&self) -> Option<String> {
        let state = self.state.lock().ok()?;
        state.passwords.values().next().cloned()
    }

    pub fn set_rule_password(&self, rule_id: &str, password: String) {
        if let Ok(mut state) = self.state.lock() {
            state.passwords.insert(rule_id.to_string(), password);
        }
    }

    pub fn forget_rule(&self, rule_id: &str) {
        if let Ok(mut state) = self.state.lock() {
            state.passwords.remove(rule_id);
        }
    }

    pub fn is_unlocked(&self, rule_id: &str) -> bool {
        self.state
            .lock()
            .map(|state| state.passwords.contains_key(rule_id))
            .unwrap_or(false)
    }

    pub fn unlocked_rules(&self) -> Vec<String> {
        let mut rules: Vec<String> = self
            .state
            .lock()
            .map(|state| state.passwords.keys().cloned().collect())
            .unwrap_or_default();
        rules.sort();
        rules
    }

    pub fn is_empty(&self) -> bool {
        self.state
            .lock()
            .map(|state| state.passwords.is_empty())
            .unwrap_or(true)
    }

    /// Forgets every password. Returns true when something was actually
    /// locked, so callers can bump the UI state exactly once.
    pub fn lock_all(&self) -> bool {
        if let Ok(mut state) = self.state.lock() {
            if state.passwords.is_empty() {
                return false;
            }
            state.passwords.clear();
            state.generation = state.generation.wrapping_add(1);
            return true;
        }
        false
    }

    /// Increases whenever the vault was locked; QML polls it to hide the
    /// snapshots that were visible before.
    pub fn generation(&self) -> u64 {
        self.state
            .lock()
            .map(|state| state.generation)
            .unwrap_or(0)
    }

    pub fn set_timeout_minutes(&self, minutes: u32) {
        self.timeout_minutes.store(minutes, Ordering::Relaxed);
    }

    pub fn timeout_minutes(&self) -> u32 {
        self.timeout_minutes.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passwords_are_kept_per_rule_and_lock_all_clears_them() {
        let vault = Vault::default();
        assert!(vault.is_empty());
        assert!(vault.any_password().is_none());

        vault.set_rule_password("a", "one".into());
        vault.set_rule_password("b", "two".into());
        assert!(vault.is_unlocked("a"));
        assert!(vault.is_unlocked("b"));
        assert_eq!(vault.unlocked_rules(), vec!["a", "b"]);
        assert_eq!(vault.password_for("b").as_deref(), Some("two"));
        assert!(vault.any_password().is_some());

        assert!(vault.lock_all());
        assert!(vault.is_empty());
        assert!(!vault.lock_all());
        assert_eq!(vault.generation(), 1);
    }

    #[test]
    fn forgetting_one_rule_keeps_the_others() {
        let vault = Vault::default();
        vault.set_rule_password("a", "one".into());
        vault.set_rule_password("b", "two".into());
        vault.forget_rule("a");
        assert!(!vault.is_unlocked("a"));
        assert!(vault.is_unlocked("b"));
        assert_eq!(vault.generation(), 0);
    }

    #[test]
    fn timeout_is_shared_between_clones() {
        let vault = Vault::default();
        let clone = vault.clone();
        clone.set_timeout_minutes(0);
        assert_eq!(vault.timeout_minutes(), 0);
    }
}
