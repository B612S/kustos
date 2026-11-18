# Changelog

All notable changes to Kustos are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project follows
[Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added

- *What to Back Up* dialog: edit the included folders/files and the excluded
  paths right before a backup; exclusions may be nested inside included
  folders.
- Encrypted, compressed and deduplicated snapshots: foreign formats (restic v2
  repository written by `rustic`), repository key shown in About.
- File browser inside a snapshot, per-item restore with overwrite / skip /
  keep-both policies, and a destination folder.
- Automatic backups through a systemd user timer, runtime English/Russian
  switch, `notify-send` notifications.

### Changed

- Rewritten in Rust (`qmetaobject` + QML/Kirigami) instead of C++/CMake; the
  rsync hard-link engine is replaced by a restic-compatible repository.
- Renamed from Aion to Kustos (the old name collides with a game).

## [0.1.0]

- First GUI prototype: rsync hard-link snapshots, timeline, restore, settings.
