# Aion

Time Machine-style snapshots for a Linux home folder, with a Kirigami GUI.

Aion copies the configured sources into timestamped snapshots on a backup
volume. Every snapshot is an `rsync` hard-link tree, so a new snapshot occupies
only the bytes that changed since the previous one. The window shows the
snapshots as a timeline, lets you browse any of them like a file manager and
restore a single file, a folder or a whole tree.

```
/data/Backup/snapshots/2026-11-18_19-00-00/home/soneriald/.config/...
/data/Backup/snapshots/2026-11-18_19-00-00/meta.json
/data/Backup/snapshots/latest -> 2026-11-18_19-00-00
```

## Requirements

- Qt 6.5+ with Qt Quick / Quick Controls 2
- Kirigami 6 and `qqc2-desktop-style` (runtime QML imports)
- `rsync` 3.2+ (3.5 recommended)
- `systemd` user session for automatic backups (optional)

Arch Linux:

```bash
sudo pacman -S qt6-base qt6-declarative kirigami qqc2-desktop-style rsync
```

## Build and run

```bash
cmake -B build -S . -DCMAKE_BUILD_TYPE=Release
cmake --build build -j$(nproc)
./build/bin/aion
```

Optional tests (Qt Test, no display needed):

```bash
cmake -B build -S . -DAION_BUILD_TESTS=ON
ctest --test-dir build --output-on-failure
```

Install into the user prefix (no root required):

```bash
cmake --install build --prefix "$HOME/.local"
```

## First backup

The built-in defaults already match a typical setup:

| Setting | Default |
|---|---|
| Backup folder | `/data/Backup` when `/data` is writable, else `~/Backup` |
| Sources | `~/.config`, `~/.local`, `~/Music`, `~/Pictures`, `~/Videos`, `~/.bashrc`, `~/.bash_profile` |
| Excludes | `~/.local/share/Steam`, `~/.local/share/twintaillauncher`, `~/.local/share/Trash` |
| Retention | hourly for 24 h, daily for 30 days, weekly for 12 weeks |
| Language | system locale, falling back to English |

Press **Back Up Now**. The first snapshot is a full copy; later runs are
incremental. Everything is configurable in **Settings**: sources, excludes,
language, backup folder, interval, retention.

## Command line

```bash
aion                     # GUI
aion --backup            # create a snapshot (used by the timer)
aion --backup --quiet    # same, without progress output
aion --list              # snapshots on the backup volume
aion --prune             # apply the retention policy
aion --restore <id> <path>... [--to DIR] [--policy overwrite|skip|keep_both]
aion --print-config      # show the effective configuration
```

Restore paths are relative to the snapshot root, for example
`home/soneriald/.config/kdeglobals`. `--to` defaults to `/`, which restores
files to their original locations; a folder destination reproduces the tree
below that folder instead. `--policy` decides what happens when the target
already exists: replace it, keep it, or restore alongside with a
`(restored <timestamp>)` suffix.

Exit codes: `0` success, `1` failure, `2` usage error.

## Automatic backups

**Settings → Schedule** installs a systemd user timer
(`~/.config/systemd/user/aion-backup.{service,timer}`) that runs
`aion --backup --quiet` every N minutes, starting five minutes after it is
enabled. The timer goes on silently with the GUI closed. `aion --backup`
takes a lock on the backup folder, so a manual run and the timer never overlap.

## Retention

After a successful backup (and on `aion --prune`) old snapshots are thinned out
like Time Machine: everything from the last `hourly` hours is kept, then the
newest snapshot of each day for `daily` days, then the newest snapshot of each
week for `weekly` weeks. The newest snapshot is never deleted.

## Where things live

| Path | Contents |
|---|---|
| `~/.config/aion/aion.conf` | settings (INI) |
| `~/.cache/aion/aion.log` | application log |
| `~/.cache/aion/logs/backup-<id>.log` | full rsync output of one backup |
| `~/.config/systemd/user/aion-backup.*` | automatic backup units |

Snapshot layout and the `meta.json` fields are documented in
[`docs/BACKUP-FORMAT.md`](docs/BACKUP-FORMAT.md).

## Notes

- Aion never modifies a finished snapshot; deleting a snapshot is the only
  destructive operation on the backup volume, and it asks first.
- Files that disappear from a source disappear from the next snapshot
  (`rsync --delete`); older snapshots keep them.
- Missing sources are skipped with a warning instead of failing the whole run.
- If the backup folder is inside a backed-up source, the run is refused.

## License

GPL-3.0-or-later.
