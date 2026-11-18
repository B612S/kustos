// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 B612S <sem.alimov@inbox.ru>

//! The repository key lives in memory only.
//!
//! It is generated once, when the repository is created, shown to the user a
//! single time and never written to disk. Later sessions ask for it again, so
//! a stolen backup disk is useless without the key kept elsewhere.

use std::sync::{Arc, Mutex};

#[derive(Clone, Default)]
pub struct Vault {
    password: Arc<Mutex<Option<String>>>,
    generated: Arc<Mutex<Option<String>>>,
}

impl Vault {
    pub fn password(&self) -> Option<String> {
        self.password.lock().ok().and_then(|guard| guard.clone())
    }

    pub fn set_password(&self, password: String) {
        if let Ok(mut guard) = self.password.lock() {
            *guard = Some(password);
        }
    }

    pub fn is_locked(&self) -> bool {
        self.password.lock().map(|guard| guard.is_none()).unwrap_or(true)
    }

    pub fn set_generated(&self, password: String) {
        if let Ok(mut guard) = self.generated.lock() {
            *guard = Some(password);
        }
    }

    pub fn take_generated(&self) -> Option<String> {
        self.generated.lock().ok().and_then(|mut guard| guard.take())
    }
}
