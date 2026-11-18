# Changelog

All notable changes to Aion are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project follows
[Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added

- Time Machine-style incremental snapshots of the home folder: `rsync`
  hard-link trees under `<backup>/snapshots/<timestamp>/`, with a `latest`
  symlink and a `meta.json` per snapshot.
- Kirigami GUI (960x640, `GlobalDrawer`, overlay menus) with a snapshot
  timeline, a file browser inside any snapshot, and restore of single items or
  whole folders.
- Restore policies: replace, keep the existing file, keep both.
- Retention policy: hourly window, one snapshot per day, one snapshot per week;
  the newest snapshot is never removed.
- Settings dialog with sources, excludes, backup folder, language (English and
  Russian, switched at runtime), notifications, ACL/xattr preservation,
  schedule and retention.
- Automatic backups through a systemd user timer installed from the settings.
- CLI: `--backup`, `--list`, `--prune`, `--restore`, `--print-config`,
  `--help`, `--version`.
- Desktop notifications when a backup or a restore finishes, and a log per
  backup in `~/.cache/aion/logs/`.
- Opt-in tests for retention, configuration and snapshot scanning.
