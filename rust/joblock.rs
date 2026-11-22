// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 B612S <sem.alimov@inbox.ru>

//! A process-wide backup lock.
//!
//! Several rules can be scheduled for the same minute, and the GUI can start a
//! manual backup while a timer is running. `rustic` would then fail with a
//! repository lock error, so every backup job takes this advisory `flock`
//! first and waits for the current one to finish.

use std::fs::{File, OpenOptions};
use std::os::unix::io::AsRawFd;
use std::path::PathBuf;

use crate::paths;

pub struct JobLock {
    file: File,
}

impl JobLock {
    /// Blocks until no other Kustos backup job is running. The lock is
    /// released when the guard is dropped (or when the process exits).
    pub fn acquire() -> Result<Self, String> {
        let path = lock_path();
        paths::ensure_parent(&path).map_err(|error| error.to_string())?;
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(&path)
            .map_err(|error| format!("cannot open {}: {error}", path.display()))?;
        let fd = file.as_raw_fd();

        let rc = unsafe { libc::flock(fd, libc::LOCK_EX | libc::LOCK_NB) };
        if rc != 0 {
            let error = std::io::Error::last_os_error();
            if error.raw_os_error() != Some(libc::EWOULDBLOCK) {
                return Err(format!("cannot lock {}: {error}", path.display()));
            }
            crate::logln("backup: another job is running; waiting for it to finish");
            let rc = unsafe { libc::flock(fd, libc::LOCK_EX) };
            if rc != 0 {
                return Err(format!(
                    "cannot lock {}: {}",
                    path.display(),
                    std::io::Error::last_os_error()
                ));
            }
        }
        Ok(Self { file })
    }
}

impl Drop for JobLock {
    fn drop(&mut self) {
        unsafe {
            libc::flock(self.file.as_raw_fd(), libc::LOCK_UN);
        }
    }
}

fn lock_path() -> PathBuf {
    paths::cache_dir().join("backup.lock")
}
