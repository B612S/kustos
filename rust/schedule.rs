// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 B612S <sem.alimov@inbox.ru>

//! One systemd user service and timer per scheduled rule.
//!
//! The service runs `kustos --backup --rule <id> --quiet`, so the application
//! does the backup in its command-line mode even when the window is closed.
//! The rule's password is handed to the service as a systemd credential
//! (`LoadCredentialEncrypted=`), not as a plain environment variable.

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::config::{BackupRule, SCHEDULE_DAILY, SCHEDULE_WEEKLY};
use crate::paths;

const UNIT_PREFIX: &str = "kustos-backup-";
const LEGACY_TEMPLATE: &str = "kustos-backup@.service";
const LEGACY_TIMER: &str = "kustos-backup.timer";
const LEGACY_SERVICE: &str = "kustos-backup.service";
const CREDENTIAL_NAME: &str = "kustos-password";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CredentialKind {
    /// Encrypted with the user's systemd credential key (`systemd-creds`).
    Encrypted,
    /// Fallback for systems without `systemd-creds`: a 0600 file.
    Plain,
}

fn units_dir() -> PathBuf {
    let base = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| paths::home().join(".config"));
    base.join("systemd/user")
}

fn credential_dir() -> PathBuf {
    paths::config_dir().join("credentials")
}

pub fn credential_path(rule_id: &str) -> PathBuf {
    credential_dir().join(format!("{rule_id}.cred"))
}

fn plain_credential_path(rule_id: &str) -> PathBuf {
    credential_dir().join(format!("{rule_id}.pass"))
}

/// Which credential file exists for a rule, if any.
pub fn credential_kind(rule_id: &str) -> Option<CredentialKind> {
    if credential_path(rule_id).is_file() {
        Some(CredentialKind::Encrypted)
    } else if plain_credential_path(rule_id).is_file() {
        Some(CredentialKind::Plain)
    } else {
        None
    }
}

pub fn remove_credential(rule_id: &str) {
    let _ = std::fs::remove_file(credential_path(rule_id));
    let _ = std::fs::remove_file(plain_credential_path(rule_id));
}

/// Reads back a rule password without asking the user. The GUI uses this to
/// start a backup while the backup itself is still locked for viewing, and the
/// generated systemd units use systemd's `$CREDENTIALS_DIRECTORY` instead.
pub fn load_credential(rule_id: &str) -> Result<String, String> {
    let encrypted = credential_path(rule_id);
    if encrypted.is_file() {
        let output = Command::new("systemd-creds")
            .args([
                "decrypt",
                "--user",
                "--no-ask-password",
                "--name",
                CREDENTIAL_NAME,
            ])
            .arg(&encrypted)
            .arg("-")
            .output();
        match output {
            Ok(result) if result.status.success() => {
                let password = String::from_utf8_lossy(&result.stdout).trim().to_string();
                if password.is_empty() {
                    return Err("the saved password is empty".to_string());
                }
                return Ok(password);
            }
            Ok(result) => {
                let detail = String::from_utf8_lossy(&result.stderr).trim().to_string();
                crate::logln(&format!(
                    "schedule: cannot decrypt the credential for {rule_id}: {detail}"
                ));
                // A plain file may still be present from a fallback; try it
                // below instead of failing immediately.
            }
            Err(error) => {
                crate::logln(&format!("schedule: cannot run systemd-creds: {error}"));
            }
        }
    }

    let plain = plain_credential_path(rule_id);
    if plain.is_file() {
        let password = std::fs::read_to_string(&plain)
            .map_err(|error| error.to_string())?
            .trim()
            .to_string();
        if password.is_empty() {
            return Err("the saved password is empty".to_string());
        }
        return Ok(password);
    }

    Err("no saved password".to_string())
}

fn write_private(path: &Path, contents: &[u8]) -> Result<(), String> {
    std::fs::write(path, contents).map_err(|error| error.to_string())?;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
        .map_err(|error| error.to_string())
}

/// Saves a rule password as a systemd credential. Prefers the encrypted form
/// (bound to the user and machine); falls back to a private file when
/// `systemd-creds` is unavailable, so automatic backups still work.
pub fn store_credential(rule_id: &str, password: &str) -> Result<CredentialKind, String> {
    let directory = credential_dir();
    std::fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    let _ = std::fs::set_permissions(&directory, std::fs::Permissions::from_mode(0o700));

    let encrypted = credential_path(rule_id);
    let plain = plain_credential_path(rule_id);
    let temporary = directory.join(format!(".{rule_id}-{}.tmp", std::process::id()));
    write_private(&temporary, password.as_bytes())?;

    let output = Command::new("systemd-creds")
        .args([
            "encrypt",
            "--user",
            "--no-ask-password",
            "--name",
            CREDENTIAL_NAME,
            "--with-key=auto",
        ])
        .arg(&temporary)
        .arg(&encrypted)
        .output();

    let encrypted_ok = matches!(output, Ok(ref result) if result.status.success());
    let _ = std::fs::remove_file(&temporary);

    if encrypted_ok {
        let _ = std::fs::set_permissions(&encrypted, std::fs::Permissions::from_mode(0o600));
        let _ = std::fs::remove_file(&plain);
        return Ok(CredentialKind::Encrypted);
    }

    if let Ok(result) = output {
        let detail = String::from_utf8_lossy(&result.stderr).trim().to_string();
        crate::logln(&format!(
            "schedule: systemd-creds failed for {rule_id}: {detail}"
        ));
    }
    write_private(&plain, password.as_bytes())?;
    let _ = std::fs::remove_file(&encrypted);
    Ok(CredentialKind::Plain)
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

fn service_name(rule_id: &str) -> String {
    format!("{UNIT_PREFIX}{rule_id}.service")
}

fn timer_name(rule_id: &str) -> String {
    format!("{UNIT_PREFIX}{rule_id}.timer")
}

fn unit_description(rule: &BackupRule) -> String {
    let name: String = rule
        .name
        .chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect();
    // systemd expands % specifiers even in Description.
    let name = name.replace('%', "%%");
    if name.trim().is_empty() {
        format!("Kustos backup rule {}", rule.id)
    } else {
        format!("Kustos backup: {} ({})", name.trim(), rule.id)
    }
}

/// Makes a path safe for a systemd unit file. `%` introduces specifiers, so it
/// must be doubled.
fn unit_path(path: &Path) -> String {
    path.to_string_lossy().replace('%', "%%")
}

fn service_body(rule: &BackupRule, kind: CredentialKind) -> String {
    let credential = credential_path(&rule.id);
    let load = match kind {
        CredentialKind::Encrypted => format!(
            "LoadCredentialEncrypted={CREDENTIAL_NAME}:{}",
            unit_path(&credential)
        ),
        CredentialKind::Plain => format!(
            "LoadCredential={CREDENTIAL_NAME}:{}",
            unit_path(&plain_credential_path(&rule.id))
        ),
    };
    format!(
        "[Unit]\n\
         Description={}\n\
         \n\
         [Service]\n\
         Type=exec\n\
         ExecStart=\"{}\" --backup --rule {} --quiet\n\
         {}\n\
         Nice=10\n\
         IOSchedulingClass=idle\n\
         Restart=on-failure\n\
         RestartSec=15min\n",
        unit_description(rule),
        executable(),
        rule.id,
        load
    )
}

fn weekday_names(days: &[u32]) -> String {
    const NAMES: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
    let mut names: Vec<&str> = Vec::new();
    for day in 1..=7u32 {
        if days.contains(&day) {
            names.push(NAMES[(day - 1) as usize]);
        }
    }
    if names.len() == 7 {
        "*-*-*".to_string()
    } else {
        names.join(",")
    }
}

/// The `[Timer]` section. Kept separate so it can be unit tested without
/// systemd present.
fn timer_section(rule: &BackupRule) -> String {
    match rule.schedule.mode.as_str() {
        SCHEDULE_DAILY => format!(
            "OnCalendar=*-*-* {}:00\nPersistent=true\nAccuracySec=1min\n",
            rule.schedule.time
        ),
        SCHEDULE_WEEKLY => format!(
            "OnCalendar={} {}:00\nPersistent=true\nAccuracySec=1min\n",
            weekday_names(&rule.schedule.weekdays),
            rule.schedule.time
        ),
        _ => format!(
            "OnActiveSec=5min\nOnUnitActiveSec={}min\nAccuracySec=1min\nPersistent=true\n",
            rule.schedule.minutes
        ),
    }
}

fn timer_body(rule: &BackupRule) -> String {
    format!(
        "[Unit]\n\
         Description={}\n\
         \n\
         [Timer]\n\
         Unit={}\n\
         {}\n\
         [Install]\n\
         WantedBy=timers.target\n",
        unit_description(rule),
        service_name(&rule.id),
        timer_section(rule)
    )
}

fn scheduled_rules(rules: &[BackupRule]) -> Vec<&BackupRule> {
    rules
        .iter()
        .filter(|rule| rule.enabled && rule.schedule.enabled && !rule.id.is_empty())
        .collect()
}

fn remove_unit_file(directory: &Path, name: &str) {
    let path = directory.join(name);
    if path.exists() {
        let _ = systemctl(&["disable", "--now", name]);
        let _ = std::fs::remove_file(path);
    }
}

/// Unit files left behind by this or an older version.
fn our_unit_files(directory: &Path) -> Vec<String> {
    let mut files = Vec::new();
    let Ok(entries) = std::fs::read_dir(directory) else {
        return files;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let ours = (name.starts_with(UNIT_PREFIX)
            || name.starts_with("kustos-backup@")
            || name == LEGACY_TIMER
            || name == LEGACY_SERVICE)
            && (name.ends_with(".timer") || name.ends_with(".service"));
        if ours {
            files.push(name);
        }
    }
    files
}

/// Rewrites the user units so that exactly the enabled rules with a saved
/// password have a service and a timer. Returns the first error or warning;
/// the unit files are still written, because a failed `systemctl` (no user
/// session) should not lose the configuration.
pub fn sync(rules: &[BackupRule]) -> Result<(), String> {
    let directory = units_dir();
    std::fs::create_dir_all(&directory).map_err(|error| error.to_string())?;

    let mut messages: Vec<String> = Vec::new();
    let mut installed: Vec<(&BackupRule, CredentialKind)> = Vec::new();
    for rule in scheduled_rules(rules) {
        match credential_kind(&rule.id) {
            Some(kind) => installed.push((rule, kind)),
            None => messages.push(format!(
                "rule '{}': set a backup password to enable automatic backups",
                if rule.name.trim().is_empty() {
                    rule.id.clone()
                } else {
                    rule.name.clone()
                }
            )),
        }
    }

    let mut desired: Vec<String> = Vec::new();
    for (rule, _) in &installed {
        desired.push(service_name(&rule.id));
        desired.push(timer_name(&rule.id));
    }
    for name in our_unit_files(&directory) {
        if !desired.contains(&name) || name == LEGACY_TEMPLATE {
            remove_unit_file(&directory, &name);
        }
    }

    for (rule, kind) in &installed {
        std::fs::write(directory.join(service_name(&rule.id)), service_body(rule, *kind))
            .map_err(|error| error.to_string())?;
        std::fs::write(directory.join(timer_name(&rule.id)), timer_body(rule))
            .map_err(|error| error.to_string())?;
    }

    if let Err(error) = systemctl(&["daemon-reload"]) {
        messages.push(error);
    }
    for (rule, _) in &installed {
        let timer = timer_name(&rule.id);
        if let Err(error) = systemctl(&["enable", "--now", &timer]) {
            messages.push(format!("{timer}: {error}"));
        }
    }
    if messages.is_empty() {
        Ok(())
    } else {
        Err(messages.join("; "))
    }
}

/// Number of rule timers currently installed on disk.
pub fn scheduled_count() -> usize {
    let directory = units_dir();
    let Ok(entries) = std::fs::read_dir(&directory) else {
        return 0;
    };
    entries
        .flatten()
        .filter(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            name.starts_with(UNIT_PREFIX) && name.ends_with(".timer")
        })
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RuleSchedule;

    fn rule(mode: &str) -> BackupRule {
        BackupRule {
            id: "r1".into(),
            name: "Work".into(),
            enabled: true,
            schedule: RuleSchedule {
                enabled: true,
                mode: mode.to_string(),
                minutes: 30,
                time: "07:05".into(),
                weekdays: vec![3, 1, 5],
            },
            key_id: "a1b2c3d4".into(),
            ..Default::default()
        }
    }

    #[test]
    fn interval_timer_uses_minutes() {
        let section = timer_section(&rule("interval"));
        assert!(section.contains("OnUnitActiveSec=30min"));
        assert!(!section.contains("OnCalendar"));
    }

    #[test]
    fn daily_timer_uses_the_clock_time() {
        let section = timer_section(&rule(SCHEDULE_DAILY));
        assert!(section.contains("OnCalendar=*-*-* 07:05:00"));
    }

    #[test]
    fn weekly_timer_lists_selected_weekdays() {
        let section = timer_section(&rule(SCHEDULE_WEEKLY));
        assert!(section.contains("OnCalendar=Mon,Wed,Fri 07:05:00"));
    }

    #[test]
    fn weekly_timer_with_all_days_is_daily() {
        let mut rule = rule(SCHEDULE_WEEKLY);
        rule.schedule.weekdays = vec![1, 2, 3, 4, 5, 6, 7];
        assert!(timer_section(&rule).contains("OnCalendar=*-*-* 07:05:00"));
    }

    #[test]
    fn disabled_rules_have_no_timer() {
        let mut rule = rule(SCHEDULE_DAILY);
        rule.schedule.enabled = false;
        assert!(scheduled_rules(&[rule]).is_empty());
    }

    #[test]
    fn each_rule_gets_its_own_service_and_timer() {
        let rule = rule(SCHEDULE_DAILY);
        assert_eq!(service_name("r1"), "kustos-backup-r1.service");
        assert_eq!(timer_name("r1"), "kustos-backup-r1.timer");
        let timer = timer_body(&rule);
        assert!(timer.contains("Unit=kustos-backup-r1.service"));
        assert!(timer.contains("WantedBy=timers.target"));
    }

    #[test]
    fn the_encrypted_service_uses_a_systemd_credential() {
        let rule = rule(SCHEDULE_DAILY);
        let service = service_body(&rule, CredentialKind::Encrypted);
        assert!(service.contains("--backup --rule r1 --quiet"));
        assert!(service.contains("LoadCredentialEncrypted=kustos-password:"));
        assert!(service.contains("Restart=on-failure"));
        assert!(!service.contains("KUSTOS_PASSWORD="));

        let plain = service_body(&rule, CredentialKind::Plain);
        assert!(plain.contains("LoadCredential=kustos-password:"));
    }
}
