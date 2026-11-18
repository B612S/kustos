// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 B612S <sem.alimov@inbox.ru>

//! Thin wrapper around the `rustic` binary (restic-compatible repositories:
//! content-defined chunking, zstd compression, AES-256 encryption).
//!
//! Every command is a separate process; JSON modes (`--json`,
//! `--json-progress`) are parsed instead of scraping the human output.

use serde::Deserialize;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub const TAG: &str = "kustos";

#[derive(Clone, Debug)]
pub struct Repo {
    pub path: String,
    pub password_file: PathBuf,
}

#[derive(Clone, Debug, Default)]
pub struct Snapshot {
    pub id: String,
    pub time: String,
    pub paths: Vec<String>,
    pub hostname: String,
    pub files: u64,
    pub dirs: u64,
    pub total_bytes: u64,
    pub added_bytes: u64,
}

impl Snapshot {
    pub fn short_id(&self) -> &str {
        self.id.get(..8).unwrap_or(&self.id)
    }
}

#[derive(Clone, Debug)]
pub struct Entry {
    pub path: String,
    pub is_dir: bool,
    pub is_link: bool,
    pub size: u64,
    pub mtime: String,
    pub mode: u32,
}

#[derive(Clone, Debug, Default)]
pub struct BackupSummary {
    pub snapshot_id: String,
    pub files_new: u64,
    pub files_changed: u64,
    pub total_files: u64,
    pub total_bytes: u64,
    pub data_added: u64,
    pub duration: f64,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct JobUpdate {
    pub percent: f64,
    pub bytes_done: u64,
    pub total_bytes: u64,
}

#[derive(Deserialize)]
struct RawGroup {
    #[serde(default)]
    snapshots: Vec<RawSnapshot>,
}

#[derive(Deserialize)]
struct RawSnapshot {
    #[serde(default)]
    id: String,
    #[serde(default)]
    time: String,
    #[serde(default)]
    paths: Vec<String>,
    #[serde(default)]
    hostname: String,
    #[serde(default)]
    summary: Option<RawSummary>,
}

#[derive(Deserialize, Default)]
struct RawSummary {
    #[serde(default)]
    files_new: u64,
    #[serde(default)]
    files_changed: u64,
    #[serde(default)]
    total_files_processed: u64,
    #[serde(default)]
    total_bytes_processed: u64,
    #[serde(default)]
    data_added: u64,
    #[serde(default)]
    total_dirs_processed: u64,
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
    #[serde(default)]
    mode: u32,
}

impl Repo {
    pub fn new(path: &str, password_file: PathBuf) -> Self {
        Self {
            path: path.trim_end_matches('/').to_string(),
            password_file,
        }
    }

    pub fn exists(&self) -> bool {
        Path::new(&self.path).join("config").exists()
    }

    fn base(&self) -> Command {
        let mut command = Command::new("rustic");
        command
            .arg("--repository")
            .arg(&self.path)
            .arg("--password-file")
            .arg(&self.password_file);
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
                    hostname: raw.hostname,
                    files: summary.total_files_processed,
                    dirs: summary.total_dirs_processed,
                    total_bytes: summary.total_bytes_processed,
                    added_bytes: summary.data_added,
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
                mode: entry.mode,
            })
            .collect())
    }

    pub fn backup<F>(&self, sources: &[String], excludes: &[String], cancel: &std::sync::atomic::AtomicBool, mut on_update: F) -> Result<BackupSummary, String>
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
                            total_bytes: value.get("total_bytes").and_then(|v| v.as_u64()).unwrap_or(0),
                        });
                    }
                    Some("summary") => {
                        summary.snapshot_id = value
                            .get("snapshot_id")
                            .and_then(|v| v.as_str())
                            .unwrap_or_default()
                            .to_string();
                        summary.files_new = value.get("files_new").and_then(|v| v.as_u64()).unwrap_or(0);
                        summary.files_changed =
                            value.get("files_changed").and_then(|v| v.as_u64()).unwrap_or(0);
                        summary.total_files = value
                            .get("total_files_processed")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0);
                        summary.total_bytes = value
                            .get("total_bytes_processed")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0);
                        summary.data_added = value.get("data_added").and_then(|v| v.as_u64()).unwrap_or(0);
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
                        total_bytes: value.get("total_bytes").and_then(|v| v.as_u64()).unwrap_or(0),
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

    pub fn forget_policy(&self, hourly: u32, daily: u32, weekly: u32) -> Result<(), String> {
        let hourly = hourly.to_string();
        let daily = daily.to_string();
        let weekly = weekly.to_string();
        self.run(&[
            "forget",
            "--prune",
            "--keep-hourly",
            hourly.as_str(),
            "--keep-daily",
            daily.as_str(),
            "--keep-weekly",
            weekly.as_str(),
        ])?;
        Ok(())
    }

    pub fn forget_snapshot(&self, snapshot_id: &str) -> Result<(), String> {
        self.run(&["forget", snapshot_id])?;
        Ok(())
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

    /// Password of the repository, for the "save this key" dialog.
    pub fn password(&self) -> String {
        std::fs::read_to_string(&self.password_file)
            .map(|text| text.trim().to_string())
            .unwrap_or_default()
    }
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
            mode: 0,
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
}
