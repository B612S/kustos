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
c8af56e5  2026-11-18T19:46:17  74550 files  27.4 GB total  25.0 GB added (24.1 GB on disk)
```

## Features

- **Incremental** — identical chunks are stored once, so every next snapshot
  costs only what changed.
- **Encrypted and compressed** — the repository is unreadable without the key;
  zstd shrinks text and source trees.
- **Three panes** — your backup rules on the left, their snapshots in the
  middle, the files of the selected snapshot on the right. Right-click a rule
  to back it up, stop it, edit it, pause it or delete it.
- **Time Machine-like browsing** — a timeline of snapshots per rule, a file
  browser inside any snapshot, restore of one file, a folder or a whole tree,
  and **Save Copies…** to copy an item straight into a folder picked in the
  file manager.
- **Backup rules** — every rule has its own folders/files, exclusion
  paths/globs and schedule. Several rules can coexist: one for documents, one
  for photos, one for a project tree. All enabled rules run on **Back Up Now**.
- **Schedules with a clock** — a rule can repeat every N minutes, run daily at
  a chosen time, or run on selected weekdays. Each scheduled rule gets its own
  systemd user service and timer, so the window does not have to be open.
- **Optional viewing passwords** — each rule can have its own password (a
  real `rustic` key of the repository), or one common password can protect all
  of them. Choose the mode in **Settings → Security → Backup passwords**. The
  password is asked only when a backup is opened, never when a backup starts:
  the GUI uses the saved credential, just like the systemd service. A new
  repository is initialized with a generated key that is saved as a
  credential.
- **Automatic re-lock** — viewing passwords are kept in memory only. After a
  chosen period without activity (or *Never*) they are forgotten; **Lock Now**
  (`Ctrl+L`) locks immediately. Locked backups hide their snapshots.
- **Unattended systemd backups** — the generated per-rule service receives the
  rule password as an encrypted `systemd-creds` credential and runs
  `kustos --backup --rule ID`, no GUI required.
- **Exclusion patterns** — paths, folder names and globs such as `*.log`,
  `node_modules` or `~/.cache/**` are skipped inside included folders.
- **Advanced rule options** — do not cross filesystem boundaries and skip
  files larger than a chosen size, per backup rule.
- **Pinned snapshots** — pin a snapshot from its context menu; pruning keeps
  it regardless of the retention limits, and **Prune Now** previews exactly
  which snapshots will be deleted.
- **Search inside a snapshot** — **Ctrl+F** filters the whole snapshot by name
  or path and jumps to the containing folder.
- **Guarded repository** — Kustos remembers the repository's identity and
  refuses to initialize a new one when the configured drive is missing; a
  repository inside a backed-up folder is excluded automatically.
- **Recovery key** — **Settings → Security** can reveal the saved key so you
  can store a copy; without it or a viewing password the repository cannot be
  opened.
- **Repository checks** — **Settings → Maintenance** verifies the repository
  structure and index, optionally reading a random 10% data sample, and
  remembers the last result. Also available as `kustos --check [--read-data]`.
- **English and Russian UI**, switched at runtime.

## Requirements

- Rust 1.75+ and Qt 6.5+ development files (Qt Core, Gui, Qml, Quick)
- Kirigami and `qqc2-desktop-style` (runtime QML imports)
- `rustic` 0.12+ in `PATH`
- `systemd` user session for the timers (optional)
- `systemd-creds` (systemd 250+) for encrypted per-rule credentials; a private
  0600 file is used as a fallback

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

After installing from the package, open **Edit Rule…** once and set a password
(or toggle the rule's schedule): the systemd units are rewritten with the new
`ExecStart` (`/usr/bin/kustos`).

Qt is located through `qmake`; this machine also has Qt 5 installed, so
`.cargo/config.toml` pins `QMAKE=/usr/bin/qmake6`. The small C++ bridge
(`src/bridge.cpp`) is compiled by `build.rs` and only installs the
`QTranslator` and uses the Qt clipboard.

## First backup

Defaults:

| Setting | Value |
|---|---|
| Repository | `/data/Backup/kustos` when `/data` is writable, else `~/Backup/kustos` |
| Rules | one rule, *Home*, with the folders below; no schedule |
| Included | `~/.config`, `~/.local`, `~/Music`, `~/Pictures`, `~/Videos`, `~/.bashrc`, `~/.bash_profile` |
| Excluded | `~/.local/share/Steam`, `~/.local/share/twintaillauncher`, `~/.local/share/Trash` |
| Retention | hourly 24 h, daily 30 days, weekly 12 weeks (repository-wide) |
| Passwords | one optional viewing password per rule; backups use a saved credential |
| Lock after | 15 minutes without activity (*Never* is available) |
| Language | system locale, English fallback |

The window has three columns: **rules**, the **snapshots of the selected
rule**, and the **files of the selected snapshot**. Clicking a rule filters the
middle column; clicking a snapshot opens it on the right. The rule context menu
(right-click) starts a backup of that rule, stops the running one, edits the
rule, pauses/resumes it or deletes it.

**Back Up Now** in the window header starts a snapshot of every enabled rule.
**Edit Rule…** opens the editor for the rule selected in the left column:
name, folders/files, exclusions and schedule. There is no second rule list in
the dialog; sources and exclusions sit side by side. Changes are saved
immediately and **Save** closes the editor. A locked rule asks for its viewing
password before the editor opens. The same editor is reachable from
**Settings → Edit Rule…**.

**Starting a backup never asks for a password.** The GUI uses the rule's saved
systemd credential — the same encrypted credential the timer uses. If the
repository does not exist yet, it generates a random key, initializes the
repository and stores that key as a credential. Only an old repository with no
saved credential asks for its key once; that unlock also saves the credential,
so the next backup is passwordless again.

**Opening a backup for viewing asks for its password.** Choose in **Settings →
Security → Backup passwords** between *Each rule has its own password* and *One
common password*:

- per-rule mode gives each rule its own `rustic` key; unlocking one rule leaves
  the others locked;
- common mode binds one key to every rule. Setting the common password creates
  a new key, updates every rule and every saved credential, and unlocks all
  backups at once; changing it revokes the previous common key.

Passwords may repeat. The application checks that a password opens the
repository, not which rule's key it belongs to; because all keys open the same
repository, a valid password from one backup can open another one in the UI.

A rule that was backed up with the generated key has no viewing password yet;
opening it asks you to create one (or the common one), and the saved credential
authorizes it. **Set/Change Password…** in the rule editor manages per-rule
passwords; the common one is managed in Settings. All keys unlock the same
repository data, so this separates backups in the UI rather than
cryptographically isolating them; use separate repositories if you need that.

Viewing passwords live in memory only. After the period chosen in **Settings →
Security → Lock after inactivity** (including *Never*) without activity in the
window they are forgotten; **Lock Now** / `Ctrl+L` locks immediately. Opening a
locked backup asks for its password again, and its snapshots stay hidden until
then. Backups keep working while the snapshots are locked.

The credential is an encrypted `systemd-creds` file
(`$XDG_CONFIG_HOME/kustos/credentials/<rule>.cred`), so the per-rule service can
run `kustos --backup --rule ID` without the window. Where `systemd-creds` is
unavailable, a private 0600 file is used instead. The CLI reads
`KUSTOS_PASSWORD`, `KUSTOS_PASSWORD_FILE` or
`$CREDENTIALS_DIRECTORY/kustos-password` (the name used by the generated
units).

## Command line

```bash
kustos                     # GUI
kustos --backup            # create snapshots for every enabled rule
kustos --backup --rule ID  # only one rule (used by the systemd units)
kustos --backup --quiet    # no progress output
kustos --list              # snapshots in the repository (with the rule name)
kustos --prune             # apply the retention policy
kustos --restore <id> <path>... [--to DIR] [--policy overwrite|skip|keep_both]
kustos --print-config      # effective configuration

# The CLI unlocks the repository like the generated units do:
KUSTOS_PASSWORD=… kustos --list
```

`--print-config` also shows the inactivity timeout, the password mode and
whether each rule has its own viewing password. The GUI initializes a new repository and backs it up
without asking; the CLI has no GUI, so `--backup` for a new repository needs
`KUSTOS_PASSWORD`, `KUSTOS_PASSWORD_FILE` or a credential.

Restore paths are snapshot-relative, for example
`home/soneriald/.config/kdeglobals`. `--to` defaults to `/` (original
location); a folder destination reproduces the tree below it. `--policy`
decides what happens to an existing target: replace it, keep it, or restore
alongside as `name (restored <timestamp>)`.

`--print-config` lists every rule with its sources, exclusions and schedule.
`--list` appends the rule name in brackets when the snapshot has a rule tag.

## How it works

Every backup is a rustic snapshot with the tag `kustos`. Snapshots created by
a rule additionally carry `kustos-rule:<rule id>`, which is how the timeline
knows where a snapshot came from:

- paths are stored relative to the snapshot root (`home/<user>/...`);
- every rule password is a separate key in `<repository>/keys/`; the master key
  is shared, so all rules deduplicate into the same chunk pool;
- unchanged files are deduplicated by content, not by hard links, so the
  replaced-in-place case is handled correctly and old versions survive;
- `rustic forget --keep-hourly … --keep-daily … --keep-weekly … --prune` thins
  the timeline after each successful backup, keeping the newest snapshot;
- the repository can also be opened by `restic` 0.18+ or `rustic` itself, which
  is a useful escape hatch.

The browser reads `rustic ls --json --long <snapshot>` once per selected
snapshot and builds every folder from that listing. Restore decrypts one path
into a staging directory and then copies it into place with `cp -a`, so
permissions and timestamps survive. **Save Copies…** uses the same staging
step but copies the item directly under the chosen folder (`name`, without the
`home/user/...` prefix); an existing name gets a timestamped sibling.
Double-click a file — or choose **Open** in its context menu — to hand it to
the default application, like Dolphin. The file is extracted to a private
temporary folder under `/tmp` first, so a reboot cleans it up. A single click
selects only that row (it never opens the file or toggles a selected row off);
**Ctrl+click** toggles rows for *Save Copies…*, *Restore…* or *Select All*.

Snapshots of a deleted rule, and snapshots from older versions without a rule
tag, are grouped under **Other Backups** instead of disappearing from the
window.

A schedule becomes one systemd user service and timer per rule:
`~/.config/systemd/user/kustos-backup-<rule>.service` and
`kustos-backup-<rule>.timer`, running
`kustos --backup --rule <rule> --quiet`. The service loads the rule
or generated backup key from the encrypted credential in
`~/.config/kustos/credentials/<rule>.cred`; the CLI reads it from
`$CREDENTIALS_DIRECTORY/kustos-password`. Interval rules use
`OnUnitActiveSec`, daily/weekly rules use `OnCalendar`.

## Repository layout and retention

See [docs/BACKUP-FORMAT.md](docs/BACKUP-FORMAT.md).

## Where things live

| Path | Contents |
|---|---|
| `~/.config/kustos/kustos.json` | settings (never passwords) |
| `~/.config/kustos/credentials/<rule>.cred` | encrypted systemd credential with the rule or generated backup key |
| `~/.cache/kustos/kustos.log` | application log |
| `/tmp/kustos-open-<uid>/` | temporary files extracted for *Open*; removed on reboot |
| `~/.config/systemd/user/kustos-backup-<rule>.service` | one service per scheduled rule |
| `~/.config/systemd/user/kustos-backup-<rule>.timer` | one timer per scheduled rule |

## License

GPL-3.0-or-later.
