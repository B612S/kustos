# Kustos

Time Machine-style backups of a Linux home folder: encrypted, compressed and
deduplicated snapshots in a [restic](https://restic.net)-compatible repository,
with a Kirigami GUI to browse the timeline and restore single items.

The application is written in **Rust**; the GUI is QML/Kirigami on top of Qt 6
through [`qmetaobject`](https://github.com/woboq/qmetaobject-rs). Snapshots are
created by [`rustic`](https://rustic.cli.rs), a Rust implementation of the
restic repository format: content-defined chunking (deduplication), zstd
compression and AES-256 encryption.

```
$ kustos --list
c8af56e5  2026-11-18T19:46:17  74550 files  27.4 GB total  25.0 GB added
```

## Features

- **Incremental** — identical chunks are stored once, so every next snapshot
  costs only what changed.
- **Encrypted and compressed** — the repository is unreadable without the key;
  zstd shrinks text and source trees.
- **Time Machine-like UI** — a timeline of snapshots, a file browser inside any
  snapshot, and restore of one file, a folder or a whole tree.
- **Choose what to back up** — the *What to Back Up* dialog edits the list of
  included folders/files and the list of excluded paths; exclusions may live
  inside included folders.
- **Automatic backups** — a systemd user timer runs `kustos --backup` every N
  minutes with the window closed.
- **English and Russian UI**, switched at runtime.

## Requirements

- Rust 1.75+ and Qt 6.5+ development files (Qt Core, Gui, Qml, Quick)
- Kirigami and `qqc2-desktop-style` (runtime QML imports)
- `rustic` 0.12+ in `PATH`
- `systemd` user session for the timer (optional)

Arch Linux:

```bash
sudo pacman -S qt6-base qt6-declarative kirigami qqc2-desktop-style rust
# rustic: pacman -S rustic, or a release binary from GitHub into ~/.local/bin
```

## Build and install

Arch Linux — a PKGBUILD is included; it builds from this working tree:

```bash
cd ~/Projects/kustos
makepkg -si          # installs /usr/bin/kustos, desktop file, icons, license
```

Manual build:

```bash
cargo build --release
install -m755 target/release/kustos ~/.local/bin/
install -m644 kustos.desktop ~/.local/share/applications/
install -m644 assets/kustos.svg ~/.local/share/icons/hicolor/scalable/apps/
install -m644 assets/kustos.png ~/.local/share/icons/hicolor/256x256/apps/
```

After installing from the package, open **Settings → Schedule** once: the
systemd unit is rewritten with the new `ExecStart` (`/usr/bin/kustos`).

Qt is located through `qmake`; this machine also has Qt 5 installed, so
`.cargo/config.toml` pins `QMAKE=/usr/bin/qmake6`. The small C++ bridge
(`src/bridge.cpp`) is compiled by `build.rs` and only installs the
`QTranslator` and uses the Qt clipboard.

## First backup

Defaults:

| Setting | Value |
|---|---|
| Repository | `/data/Backup/kustos` when `/data` is writable, else `~/Backup/kustos` |
| Included | `~/.config`, `~/.local`, `~/Music`, `~/Pictures`, `~/Videos`, `~/.bashrc`, `~/.bash_profile` |
| Excluded | `~/.local/share/Steam`, `~/.local/share/twintaillauncher`, `~/.local/share/Trash` |
| Retention | hourly 24 h, daily 30 days, weekly 12 weeks |
| Language | system locale, English fallback |

**Back Up Now** opens *What to Back Up*, where the included and excluded paths
are edited; **Back Up Now** in that dialog starts the snapshot. The same lists
are reachable from **Settings → Backup Scope…**.

The repository key is generated on the first run and stored in
`~/.config/kustos/repo.password` (mode 0600). It is the only way to read the
backups: copy it to a password manager or print it, otherwise a lost machine
means lost backups. **About → Repository key** shows it.

## Command line

```bash
kustos                     # GUI
kustos --backup            # create a snapshot (used by the timer)
kustos --backup --quiet    # no progress output
kustos --list              # snapshots in the repository
kustos --prune             # apply the retention policy
kustos --restore <id> <path>... [--to DIR] [--policy overwrite|skip|keep_both]
kustos --print-config      # effective configuration
```

Restore paths are snapshot-relative, for example
`home/soneriald/.config/kdeglobals`. `--to` defaults to `/` (original
location); a folder destination reproduces the tree below it. `--policy`
decides what happens to an existing target: replace it, keep it, or restore
alongside as `name (restored <timestamp>)`.

## How it works

Every backup is a rustic snapshot with the tag `kustos`:

- paths are stored relative to the snapshot root (`home/<user>/...`);
- unchanged files are deduplicated by content, not by hard links, so the
  replaced-in-place case is handled correctly and old versions survive;
- `rustic forget --keep-hourly … --keep-daily … --keep-weekly … --prune` thins
  the timeline after each successful backup, keeping the newest snapshot;
- the repository can also be opened by `restic` 0.18+ or `rustic` itself, which
  is a useful escape hatch.

The browser reads `rustic ls --json --long <snapshot>` once per selected
snapshot and builds every folder from that listing. Restore decrypts one path
into a staging directory and then copies it into place with `cp -a`, so
permissions and timestamps survive.

## Repository layout and retention

See [docs/BACKUP-FORMAT.md](docs/BACKUP-FORMAT.md).

## Where things live

| Path | Contents |
|---|---|
| `~/.config/kustos/kustos.json` | settings |
| `~/.config/kustos/repo.password` | repository key (0600) |
| `~/.cache/kustos/kustos.log` | application log |
| `~/.config/systemd/user/kustos-backup.*` | timer and service |

## License

GPL-3.0-or-later.
