// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 B612S <sem.alimov@inbox.ru>

//! systemd user timer for automatic backups: `kustos --backup --quiet` every N
//! minutes, starting five minutes after the timer is enabled.

use std::path::PathBuf;
use std::process::Command;

use crate::paths;

fn units_dir() -> PathBuf {
    let base = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| paths::home().join(".config"));
    base.join("systemd/user")
}

fn executable() -> String {
    std::env::current_exe()
        .map(|path| path.to_string_lossy().into_owned())
        .unwrap_or_else(|_| "kustos".to_string())
}

fn systemctl(arguments: &[&str]) -> Result<String, String> {
    let output = Command::new("systemctl")
        .arg("--user")
        .args(arguments)
        .output()
        .map_err(|error| format!("cannot run systemctl: {error}"))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

pub fn install(minutes: u32) -> Result<(), String> {
    let directory = units_dir();
    std::fs::create_dir_all(&directory).map_err(|error| error.to_string())?;

    let service = format!(
        "[Unit]\n\
         Description=Kustos automatic backup\n\
         \n\
         [Service]\n\
         Type=oneshot\n\
         ExecStart=\"{}\" --backup --quiet\n\
         Nice=10\n\
         IOSchedulingClass=idle\n",
        executable()
    );
    std::fs::write(directory.join("kustos-backup.service"), service)
        .map_err(|error| error.to_string())?;

    let timer = format!(
        "[Unit]\n\
         Description=Kustos backup every {minutes} minutes\n\
         \n\
         [Timer]\n\
         OnActiveSec=5min\n\
         OnUnitActiveSec={minutes}min\n\
         AccuracySec=1min\n\
         Persistent=true\n\
         \n\
         [Install]\n\
         WantedBy=timers.target\n"
    );
    std::fs::write(directory.join("kustos-backup.timer"), timer).map_err(|error| error.to_string())?;

    systemctl(&["daemon-reload"])?;
    systemctl(&["enable", "--now", "kustos-backup.timer"])?;
    Ok(())
}

pub fn uninstall() -> Result<(), String> {
    let directory = units_dir();
    let _ = systemctl(&["disable", "--now", "kustos-backup.timer"]);
    let _ = std::fs::remove_file(directory.join("kustos-backup.service"));
    let _ = std::fs::remove_file(directory.join("kustos-backup.timer"));
    systemctl(&["daemon-reload"])?;
    Ok(())
}

pub fn state() -> String {
    let directory = units_dir();
    if !directory.join("kustos-backup.timer").exists() {
        return "absent".to_string();
    }
    match systemctl(&["is-active", "kustos-backup.timer"]) {
        Ok(output) if output == "active" => "active".to_string(),
        _ => "inactive".to_string(),
    }
}
