// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 B612S <sem.alimov@inbox.ru>

//! Thin wrapper around the `rustic` binary (restic-compatible repositories:
//! content-defined chunking, zstd compression, AES-256 encryption).
//!
//! Every command is a separate process; JSON modes (`--json`,
//! `--json-progress`) are parsed instead of scraping the human output.

use serde::Deserialize;
use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use std::process::{Command, Stdio};

use crate::config::RULE_TAG_PREFIX;
use crate::paths;

pub const TAG: &str = "kustos";
/// Pinned snapshots survive the retention policy.
pub const PIN_TAG: &str = "kustos-pin";

#[derive(Clone, Debug)]
pub struct Repo {
    pub path: String,
    pub password: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct Snapshot {
    pub id: String,
    pub time: String,
    pub paths: Vec<String>,
    pub tags: Vec<String>,
    pub files: u64,
    pub total_bytes: u64,
    /// New data before compression (rustic's `data_added`).
    pub added_bytes: u64,
    /// New data actually written to the repository (`data_added_packed`).
    pub added_packed_bytes: u64,
    pub files_new: u64,
    pub files_changed: u64,
    pub duration: f64,
}

impl Snapshot {
    pub fn short_id(&self) -> &str {
        self.id.get(..8).unwrap_or(&self.id)
    }

    /// The rule that produced this snapshot, when the snapshot was created by
    /// a version that tags rules.
    pub fn rule_id(&self) -> Option<&str> {
        self.tags
            .iter()
            .find_map(|tag| tag.strip_prefix(RULE_TAG_PREFIX))
    }

    pub fn pinned(&self) -> bool {
        self.tags.iter().any(|tag| tag == PIN_TAG)
    }
}

#[derive(Clone, Debug)]
pub struct Entry {
    pub path: String,
    pub is_dir: bool,
    pub is_link: bool,
    pub size: u64,
    pub mtime: String,
}

#[derive(Clone, Debug, Default)]
pub struct BackupSummary {
    pub snapshot_id: String,
    pub total_files: u64,
    /// New data before compression (rustic's `data_added`).
    pub data_added: u64,
    /// New data actually written to the repository (`data_added_packed`).
    pub data_added_packed: u64,
    pub duration: f64,
}

/// One password-protected key of the repository. `active` marks the key that
/// opened the repository for the current command.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyInfo {
    pub id: String,
    pub active: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct JobUpdate {
    pub percent: f64,
    pub bytes_done: u64,
}

#[derive(Deserialize)]
struct RawGroup {
    #[serde(default)]
    snapshots: Vec<RawSnapshot>,
}

#[derive(Deserialize)]
struct RawForgetGroup {
    #[serde(default)]
    items: Vec<RawForgetItem>,
}

#[derive(Deserialize)]
struct RawForgetItem {
    #[serde(default)]
    snapshot: RawSnapshot,
    #[serde(default)]
    keep: bool,
}

/// Turns `forget --dry-run --json` output into the snapshots that would be
/// deleted. Kept outside the method so it can be tested without a repository.
fn parse_forget_preview(json: &str) -> Result<Vec<Snapshot>, String> {
    let groups: Vec<RawForgetGroup> = serde_json::from_str(json)
        .map_err(|error| format!("cannot parse the prune preview: {error}"))?;
    let mut snapshots = Vec::new();
    for group in groups {
        for item in group.items {
            if item.keep {
                continue;
            }
            snapshots.push(Snapshot {
                id: item.snapshot.id,
                time: item.snapshot.time,
                tags: item.snapshot.tags,
                ..Default::default()
            });
        }
    }
    snapshots.sort_by(|a, b| b.time.cmp(&a.time));
    Ok(snapshots)
}

#[derive(Deserialize, Default)]
struct RawSnapshot {
    #[serde(default)]
    id: String,
    #[serde(default)]
    time: String,
    #[serde(default)]
    paths: Vec<String>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    summary: Option<RawSummary>,
}

#[derive(Deserialize, Default)]
struct RawSummary {
    #[serde(default)]
    total_files_processed: u64,
    #[serde(default)]
    total_bytes_processed: u64,
    #[serde(default)]
    data_added: u64,
    #[serde(default)]
    data_added_packed: u64,
    #[serde(default)]
    files_new: u64,
    #[serde(default)]
    files_changed: u64,
    #[serde(default)]
    total_duration: f64,
}

#[derive(Deserialize)]
struct RawEntry {
    #[serde(default)]
    path: String,
    #[serde(rename = "type", default)]
    kind: String,
    #[serde(default)]
    size: u64,
    #[serde(default)]
    mtime: Option<String>,
}

/// Extra `backup` flags for the per-rule advanced options. Kept separate so
/// the argument construction can be tested without a repository.
fn backup_option_arguments(one_file_system: bool, exclude_larger: &str) -> Vec<String> {
    let mut arguments = Vec::new();
    if one_file_system {
        arguments.push("--one-file-system".to_string());
    }
    let size = exclude_larger.trim();
    if !size.is_empty() {
        arguments.push("--exclude-larger-than".to_string());
        arguments.push(size.to_string());
    }
    arguments
}

impl Repo {
    pub fn new(path: &str, password: Option<String>) -> Self {
        Self {
            path: path.trim_end_matches('/').to_string(),
            password,
        }
    }

    pub fn exists(&self) -> bool {
        Path::new(&self.path).join("config").exists()
    }

    fn base(&self) -> Command {
        let mut command = Command::new("rustic");
        command.arg("--repository").arg(&self.path);
        // The key is never written to disk: the child process reads it from
        // the environment, which is not visible in the process list.
        if let Some(password) = &self.password {
            command.env("RUSTIC_PASSWORD", password);
            command.env("RESTIC_PASSWORD", password);
        }
        command
    }

    fn run(&self, arguments: &[&str]) -> Result<String, String> {
        let output = self
            .base()
            .args(arguments)
            .output()
            .map_err(|error| format!("cannot run rustic: {error}"))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let detail = stderr
                .lines()
                .filter(|line| !line.trim().is_empty())
                .last()
                .unwrap_or("rustic failed")
                .to_string();
            return Err(detail);
        }
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }

    pub fn init(&self) -> Result<(), String> {
        std::fs::create_dir_all(&self.path).map_err(|error| error.to_string())?;
        self.run(&["init"])?;
        Ok(())
    }

    pub fn ensure_repo(&self) -> Result<(), String> {
        if self.exists() {
            Ok(())
        } else {
            self.init()
        }
    }

    /// Lists the repository keys. This also validates the password. With
    /// duplicate passwords the active marker is only one of the matching
    /// keys, so callers must not rely on it to identify a key.
    pub fn key_list(&self) -> Result<Vec<KeyInfo>, String> {
        let output = self.run(&["key", "list"])?;
        let keys = parse_key_list(&output);
        if keys.is_empty() {
            return Err("rustic did not report any repository keys".to_string());
        }
        Ok(keys)
    }

    /// Returns the key rustic used to open the repository. With duplicate
    /// passwords this is one of the matching keys, not necessarily the one a
    /// caller meant.
    pub fn active_key(&self) -> Result<KeyInfo, String> {
        self.key_list()?
            .into_iter()
            .find(|key| key.active)
            .ok_or_else(|| "rustic did not report the active key".to_string())
    }

    /// Adds a key protected by `new_password` and returns it. The repository
    /// must be opened with an existing password (`self.password`).
    ///
    /// The new key is identified by the difference between the key list before
    /// and after the command: two keys may have the same password, so the
    /// active one is not necessarily the new one.
    pub fn add_key(&self, new_password: &str) -> Result<KeyInfo, String> {
        let before = self.key_list()?;
        with_password_file(new_password, |file| {
            self.run(&["key", "add", "--new-password-file", file])
        })?;
        self.find_new_key(&before, new_password)
    }

    fn find_new_key(&self, before: &[KeyInfo], password: &str) -> Result<KeyInfo, String> {
        let after = Repo::new(&self.path, Some(password.to_string())).key_list()?;
        after
            .into_iter()
            .find(|key| !before.iter().any(|old| old.id == key.id))
            .ok_or_else(|| "rustic did not report the new key".to_string())
    }

    /// Removes one key by ID. The caller must know another password; otherwise
    /// the repository would become inaccessible.
    pub fn remove_key(&self, key_id: &str) -> Result<(), String> {
        self.run(&["key", "remove", key_id])?;
        Ok(())
    }

    pub fn snapshots(&self) -> Result<Vec<Snapshot>, String> {
        if !self.exists() {
            return Ok(Vec::new());
        }
        let json = self.run(&["snapshots", "--json", "--filter-tags", TAG])?;
        let groups: Vec<RawGroup> = serde_json::from_str(&json)
            .map_err(|error| format!("cannot parse snapshot list: {error}"))?;
        let mut snapshots: Vec<Snapshot> = groups
            .into_iter()
            .flat_map(|group| group.snapshots)
            .map(|raw| {
                let summary = raw.summary.unwrap_or_default();
                Snapshot {
                    id: raw.id,
                    time: raw.time,
                    paths: raw.paths,
                    tags: raw.tags,
                    files: summary.total_files_processed,
                    total_bytes: summary.total_bytes_processed,
                    added_bytes: summary.data_added,
                    added_packed_bytes: summary.data_added_packed,
                    files_new: summary.files_new,
                    files_changed: summary.files_changed,
                    duration: summary.total_duration,
                }
            })
            .collect();
        snapshots.sort_by(|a, b| b.time.cmp(&a.time));
        Ok(snapshots)
    }

    /// Full listing of one snapshot (all paths, with metadata). The browser
    /// builds every folder from this single tree.
    pub fn ls(&self, snapshot_id: &str) -> Result<Vec<Entry>, String> {
        let json = self.run(&["ls", "--json", "--long", snapshot_id])?;
        let raw: Vec<RawEntry> = serde_json::from_str(&json)
            .map_err(|error| format!("cannot parse snapshot listing: {error}"))?;
        Ok(raw
            .into_iter()
            .map(|entry| Entry {
                path: entry.path.trim_end_matches('/').to_string(),
                is_dir: entry.kind == "dir",
                is_link: entry.kind == "symlink",
                size: entry.size,
                mtime: entry.mtime.unwrap_or_default(),
            })
            .collect())
    }

    pub fn backup<F>(
        &self,
        sources: &[String],
        excludes: &[String],
        rule_id: &str,
        one_file_system: bool,
        exclude_larger: &str,
        cancel: &std::sync::atomic::AtomicBool,
        mut on_update: F,
    ) -> Result<BackupSummary, String>
    where
        F: FnMut(JobUpdate),
    {
        self.ensure_repo()?;

        let mut command = self.base();
        command
            .arg("backup")
            .arg("--json-progress")
            .arg("--tag")
            .arg(TAG);
        if !rule_id.is_empty() {
            command.arg("--tag").arg(format!("{RULE_TAG_PREFIX}{rule_id}"));
        }
        for argument in backup_option_arguments(one_file_system, exclude_larger) {
            command.arg(argument);
        }
        for exclude in excludes {
            command.arg("--glob").arg(format!("!{exclude}"));
        }
        for source in sources {
            command.arg(source);
        }

        let mut child = command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| format!("cannot run rustic: {error}"))?;

        let stderr = child.stderr.take();
        let stderr_thread = std::thread::spawn(move || {
            let mut text = String::new();
            if let Some(mut stream) = stderr {
                let _ = stream.read_to_string(&mut text);
            }
            text
        });

        let mut summary = BackupSummary::default();
        if let Some(stdout) = child.stdout.take() {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                if cancel.load(std::sync::atomic::Ordering::Relaxed) {
                    let _ = child.kill();
                    break;
                }
                let Ok(value) = serde_json::from_str::<serde_json::Value>(&line) else {
                    continue;
                };
                match value.get("message_type").and_then(|m| m.as_str()) {
                    Some("status") => {
                        on_update(JobUpdate {
                            percent: value
                                .get("percent_done")
                                .and_then(|v| v.as_f64())
                                .unwrap_or(0.0),
                            bytes_done: value.get("bytes_done").and_then(|v| v.as_u64()).unwrap_or(0),
                        });
                    }
                    Some("summary") => {
                        summary.snapshot_id = value
                            .get("snapshot_id")
                            .and_then(|v| v.as_str())
                            .unwrap_or_default()
                            .to_string();
                        summary.total_files = value
                            .get("total_files_processed")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0);
                        summary.data_added = value.get("data_added").and_then(|v| v.as_u64()).unwrap_or(0);
                        summary.data_added_packed = value
                            .get("data_added_packed")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0);
                        summary.duration = value
                            .get("total_duration")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(0.0);
                    }
                    _ => {}
                }
            }
        }

        let status = child.wait().map_err(|error| error.to_string())?;
        let stderr_text = stderr_thread.join().unwrap_or_default();
        if cancel.load(std::sync::atomic::Ordering::Relaxed) {
            return Err("cancelled".to_string());
        }
        if !status.success() {
            let detail = stderr_text
                .lines()
                .filter(|line| !line.trim().is_empty() && !line.contains("[INFO]"))
                .last()
                .unwrap_or("backup failed")
                .to_string();
            return Err(detail);
        }
        Ok(summary)
    }

    /// Restores one path into `staging`; the content of a directory lands
    /// directly in `staging`, a single file as `staging/<name>`.
    pub fn restore<F>(&self, snapshot_id: &str, path: &str, staging: &Path, cancel: &std::sync::atomic::AtomicBool, mut on_update: F) -> Result<(), String>
    where
        F: FnMut(JobUpdate),
    {
        std::fs::create_dir_all(staging).map_err(|error| error.to_string())?;
        let mut command = self.base();
        command
            .arg("restore")
            .arg("--json-progress")
            .arg(format!("{snapshot_id}:{path}"))
            .arg(staging);

        let mut child = command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| format!("cannot run rustic: {error}"))?;

        let stderr = child.stderr.take();
        let stderr_thread = std::thread::spawn(move || {
            let mut text = String::new();
            if let Some(mut stream) = stderr {
                let _ = stream.read_to_string(&mut text);
            }
            text
        });

        if let Some(stdout) = child.stdout.take() {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                if cancel.load(std::sync::atomic::Ordering::Relaxed) {
                    let _ = child.kill();
                    break;
                }
                let Ok(value) = serde_json::from_str::<serde_json::Value>(&line) else {
                    continue;
                };
                if value.get("message_type").and_then(|m| m.as_str()) == Some("status") {
                    on_update(JobUpdate {
                        percent: value
                            .get("percent_done")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(0.0),
                        bytes_done: value.get("bytes_done").and_then(|v| v.as_u64()).unwrap_or(0),
                    });
                }
            }
        }

        let status = child.wait().map_err(|error| error.to_string())?;
        let stderr_text = stderr_thread.join().unwrap_or_default();
        if cancel.load(std::sync::atomic::Ordering::Relaxed) {
            return Err("cancelled".to_string());
        }
        if !status.success() {
            let detail = stderr_text
                .lines()
                .filter(|line| !line.trim().is_empty() && !line.contains("[INFO]"))
                .last()
                .unwrap_or("restore failed")
                .to_string();
            return Err(detail);
        }
        Ok(())
    }

    /// Arguments shared by the real prune and its dry-run preview.
    fn forget_arguments(&self, hourly: u32, daily: u32, weekly: u32) -> Vec<String> {
        vec![
            "forget".to_string(),
            // Rules are separated by their tags, so one rule's history cannot
            // thin out another rule with the same source paths.
            "--group-by".to_string(),
            "host,label,paths,tags".to_string(),
            // Pinned snapshots are always kept, even when the schedule policy
            // would drop them.
            "--keep-tags".to_string(),
            PIN_TAG.to_string(),
            "--keep-hourly".to_string(),
            hourly.to_string(),
            "--keep-daily".to_string(),
            daily.to_string(),
            "--keep-weekly".to_string(),
            weekly.to_string(),
        ]
    }

    pub fn forget_policy(&self, hourly: u32, daily: u32, weekly: u32) -> Result<(), String> {
        let mut arguments = self.forget_arguments(hourly, daily, weekly);
        arguments.push("--prune".to_string());
        let refs: Vec<&str> = arguments.iter().map(String::as_str).collect();
        self.run(&refs)?;
        Ok(())
    }

    /// Snapshots the retention policy would delete. Uses `--dry-run`, so the
    /// repository is unchanged.
    pub fn forget_preview(
        &self,
        hourly: u32,
        daily: u32,
        weekly: u32,
    ) -> Result<Vec<Snapshot>, String> {
        let mut arguments = self.forget_arguments(hourly, daily, weekly);
        arguments.push("--dry-run".to_string());
        arguments.push("--json".to_string());
        let refs: Vec<&str> = arguments.iter().map(String::as_str).collect();
        let json = self.run(&refs)?;
        parse_forget_preview(&json)
    }

    /// Adds or removes the pin tag. Pinned snapshots are kept by
    /// `forget_policy` regardless of the retention limits.
    pub fn set_pinned(&self, snapshot_id: &str, pinned: bool) -> Result<(), String> {
        let action = if pinned { "--add" } else { "--remove" };
        self.run(&["tag", action, PIN_TAG, snapshot_id])?;
        Ok(())
    }

    pub fn forget_snapshot(&self, snapshot_id: &str) -> Result<(), String> {
        self.run(&["forget", snapshot_id])?;
        Ok(())
    }

    /// Runs `rustic check`. With `read_data`, verifies a random 10% sample of
    /// the pack data instead of only the repository structure and index.
    /// Returns a short detail line for the UI; rustic prints its progress to
    /// stderr, so the last non-INFO lines are used on failure.
    pub fn check(&self, read_data: bool) -> Result<String, String> {
        let mut command = self.base();
        command.arg("check");
        if read_data {
            command.args(["--read-data", "--read-data-subset", "10%"]);
        }
        let output = command
            .output()
            .map_err(|error| format!("cannot run rustic: {error}"))?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let detail = |text: &str| -> String {
            let lines: Vec<&str> = text
                .lines()
                .map(|line| line.trim())
                .filter(|line| !line.is_empty() && !line.contains("[INFO]"))
                .collect();
            lines.join("\n")
        };
        if !output.status.success() {
            let text = detail(&stderr);
            let text = if text.is_empty() { detail(&stdout) } else { text };
            return Err(if text.is_empty() {
                "repository check failed".to_string()
            } else {
                text
            });
        }
        let text = detail(&stdout);
        Ok(if text.is_empty() { "ok".to_string() } else { text })
    }

    pub fn free_bytes(&self) -> u64 {
        let target = if Path::new(&self.path).exists() {
            self.path.clone()
        } else {
            Path::new(&self.path)
                .parent()
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_else(|| "/".to_string())
        };
        let output = Command::new("df")
            .args(["-B1", "--output=avail", &target])
            .output();
        match output {
            Ok(output) => String::from_utf8_lossy(&output.stdout)
                .lines()
                .nth(1)
                .and_then(|line| line.trim().parse::<u64>().ok())
                .unwrap_or(0),
            Err(_) => 0,
        }
    }

}

/// Parses the plain-text table printed by `rustic key list`:
///
/// ```text
/// | ID        | User | Host | Created |
/// |-----------|------|------|---------|
/// | *349b4174 |      |      |         |
/// | 875deb58  |      |      |         |
/// ```
///
/// The `*` marks the key that opened the repository.
fn parse_key_list(output: &str) -> Vec<KeyInfo> {
    let mut keys = Vec::new();
    for line in output.lines() {
        let line = line.trim();
        if !line.starts_with('|') {
            continue;
        }
        let Some(cell) = line.split('|').nth(1) else {
            continue;
        };
        let cell = cell.trim();
        if cell.is_empty() || cell == "ID" || cell.chars().all(|c| c == '-') {
            continue;
        }
        let (active, id) = match cell.strip_prefix('*') {
            Some(id) => (true, id.trim()),
            None => (false, cell),
        };
        if id.is_empty() || !id.chars().all(|c| c.is_ascii_alphanumeric()) {
            continue;
        }
        keys.push(KeyInfo {
            id: id.to_string(),
            active,
        });
    }
    keys
}

/// Rustic takes the new password from a file, which keeps it out of the
/// process list. The file is private and removed immediately.
fn with_password_file<T>(
    password: &str,
    run: impl FnOnce(&str) -> Result<T, String>,
) -> Result<T, String> {
    let directory = paths::cache_dir().join("keys");
    std::fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_nanos())
        .unwrap_or(0);
    let path = directory.join(format!("password-{}-{stamp}", std::process::id()));
    let mut file = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(0o600)
        .open(&path)
        .map_err(|error| error.to_string())?;
    file.write_all(password.as_bytes())
        .map_err(|error| error.to_string())?;
    file.flush().map_err(|error| error.to_string())?;
    drop(file);

    let result = run(&path.to_string_lossy());
    let _ = std::fs::remove_file(&path);
    result
}

/// Groups the full snapshot listing into direct children of `directory`
/// ("" is the snapshot root). Used by the browser; the listing itself is
/// cached by the caller.
pub fn children_of<'a>(entries: &'a [Entry], directory: &str) -> Vec<&'a Entry> {
    let prefix = if directory.is_empty() {
        String::new()
    } else {
        format!("{}/", directory.trim_end_matches('/'))
    };
    let mut children: Vec<&Entry> = entries
        .iter()
        .filter(|entry| {
            if entry.path == directory {
                return false;
            }
            let Some(rest) = entry.path.strip_prefix(&prefix) else {
                return false;
            };
            !rest.is_empty() && !rest.contains('/')
        })
        .collect();
    children.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.path.to_lowercase().cmp(&b.path.to_lowercase()),
    });
    children
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(path: &str, is_dir: bool) -> Entry {
        Entry {
            path: path.to_string(),
            is_dir,
            is_link: false,
            size: 0,
            mtime: String::new(),
        }
    }

    #[test]
    fn children_are_direct_and_sorted_dirs_first() {
        let entries = vec![
            entry("home", true),
            entry("home/user", true),
            entry("home/user/a.txt", false),
            entry("home/user/b", true),
            entry("home/user/b/c.txt", false),
        ];
        let children = children_of(&entries, "home/user");
        let names: Vec<&str> = children.iter().map(|e| e.path.as_str()).collect();
        assert_eq!(names, vec!["home/user/b", "home/user/a.txt"]);
    }

    #[test]
    fn root_children_have_no_leading_slash() {
        let entries = vec![entry("home", true), entry("tmp", true)];
        let children = children_of(&entries, "");
        assert_eq!(children.len(), 2);
    }

    #[test]
    fn key_list_marks_the_active_key() {
        let output = "\
| ID        | User | Host | Created |
|-----------|------|------|---------|
| *349b4174 |      |      |         |
| 875deb58  |      |      |         |
";
        let keys = parse_key_list(output);
        assert_eq!(keys.len(), 2);
        assert_eq!(keys[0].id, "349b4174");
        assert!(keys[0].active);
        assert_eq!(keys[1].id, "875deb58");
        assert!(!keys[1].active);
    }

    #[test]
    fn key_list_ignores_headings_and_bad_rows() {
        let output = "\
Some [INFO] line
| ID | User |
|----|------|
| not a key! | |
| 1a2b3c4d | |
";
        let keys = parse_key_list(output);
        assert_eq!(
            keys,
            vec![KeyInfo {
                id: "1a2b3c4d".into(),
                active: false
            }]
        );
    }

    #[test]
    fn backup_options_become_rustic_flags() {
        let args = backup_option_arguments(true, " 2G ");
        assert_eq!(args, vec!["--one-file-system", "--exclude-larger-than", "2G"]);
        assert!(backup_option_arguments(false, " ").is_empty());
    }

    #[test]
    fn pinned_snapshots_are_recognized_and_kept_by_forget() {
        let pinned = Snapshot {
            tags: vec![TAG.to_string(), PIN_TAG.to_string()],
            ..Default::default()
        };
        assert!(pinned.pinned());
        let plain = Snapshot {
            tags: vec![TAG.to_string()],
            ..Default::default()
        };
        assert!(!plain.pinned());

        let args = Repo::new("/tmp/repo", None).forget_arguments(24, 30, 12);
        assert!(args.iter().any(|arg| arg == PIN_TAG));
        assert!(args
            .windows(2)
            .any(|pair| pair[0] == "--group-by" && pair[1].contains("tags")));
    }

    #[test]
    fn forget_preview_keeps_only_unpinned_snapshots() {
        let json = r#"[
          {"items": [
            {"snapshot": {"id": "aaa", "time": "2026-01-01T00:00:00Z",
                          "tags": ["kustos"]}, "keep": false},
            {"snapshot": {"id": "bbb", "time": "2026-01-02T00:00:00Z",
                          "tags": ["kustos", "kustos-pin"]}, "keep": true}
          ]}
        ]"#;
        let plan = parse_forget_preview(json).unwrap();
        assert_eq!(plan.len(), 1);
        assert_eq!(plan[0].id, "aaa");
    }

    #[test]
    fn snapshot_summary_separates_raw_and_packed_added_data() {
        let summary: RawSummary = serde_json::from_str(
            r#"{"total_files_processed": 3, "total_bytes_processed": 4096,
                "data_added": 1048576, "data_added_packed": 327680,
                "files_new": 2, "files_changed": 1, "total_duration": 4.5}"#,
        )
        .unwrap();
        assert_eq!(summary.data_added, 1_048_576);
        assert_eq!(summary.data_added_packed, 327_680);
        assert_eq!(summary.files_new, 2);
        assert_eq!(summary.files_changed, 1);
        assert_eq!(summary.total_duration, 4.5);
    }

    #[test]
    fn snapshot_rule_id_comes_from_the_tag() {
        let snapshot = Snapshot {
            tags: vec![TAG.to_string(), format!("{RULE_TAG_PREFIX}work")],
            ..Default::default()
        };
        assert_eq!(snapshot.rule_id(), Some("work"));
        let legacy = Snapshot {
            tags: vec![TAG.to_string()],
            ..Default::default()
        };
        assert_eq!(legacy.rule_id(), None);
    }
}
