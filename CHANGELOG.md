# Changelog

All notable changes to Kustos are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project follows
[Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added

- **Per-backup viewing passwords.** Every rule can have its own password — a
  real `rustic` key of the repository. The application no longer asks for a
  repository key at startup. A backup uses the saved credential and never asks
  for a password; the per-rule password is asked only when the backup is
  opened for viewing. The password dialog can create the first viewing
  password, unlock a backup, or change its password; the editor shows whether a
  rule is protected.
- **Password mode in Settings → Security.** Choose *each rule has its own
  password* or *one common password for all rules*. Common mode binds a single
  `rustic` key to every rule, updates every saved credential and unlocks all
  backups at once; changing it revokes the previous common key. A new rule
  added in common mode joins that key automatically.
- **Missing-repository guard.** Kustos remembers a fingerprint of the
  repository's encrypted `config` file. If the configured path disappears (an
  unmounted external drive), a backup now fails with a clear "repository not
  found" error instead of silently initializing a new repository on the system
  disk. A different repository at the same path is refused too.
- **Repository inside a source.** A rule that includes the repository folder
  now gets that folder added to its exclusions automatically, with a warning in
  the backup result, so a backup cannot recursively swallow its own repository.
- **Recovery key.** **Settings → Security → Recovery Key** reveals the saved
  key (after an explicit click) and copies it, with a warning that whoever sees
  it can read every backup. The generated key was previously only stored in the
  systemd credential; a lost credential made the repository unrecoverable.
- **Advanced per-rule backup options.** A rule can stay on one filesystem
  (`--one-file-system`) and skip files larger than a given size
  (`--exclude-larger-than`, e.g. `2G`). Both are passed by the GUI and CLI
  backups; an empty size means no limit.
- **Pinned snapshots.** The snapshot context menu can pin or unpin a
  snapshot. Pins are stored as the `kustos-pin` tag; the retention policy passes
  `--keep-tags kustos-pin`, so pinned snapshots survive pruning. The timeline
  marks them with a star, and `kustos --list` prints `[pinned]`.
- **Per-rule retention and prune preview.** `forget` now groups snapshots by
  their tags as well as host/label/paths, so rules with identical sources no
  longer share a retention history. **Prune Now** first runs a dry run and shows
  the exact snapshots it would delete; the same preview is available as
  `kustos --prune --dry-run`.
- **Retry for scheduled backups.** Generated user services use `Type=exec` and
  restart a failed backup after 15 minutes, so a backup that ran while an
  external drive was offline is retried.
- **Log rotation.** `kustos.log` is rotated to `kustos.log.1` once it exceeds
  1 MiB instead of growing forever.
- **Snapshot search.** **Ctrl+F** in the file browser searches the whole
  snapshot by name or path, shows the matches with their full paths, and opens
  the containing folder (or the folder itself) on click.
- **Change details in the timeline tooltip.** Hovering a snapshot now shows the
  total file count, new and changed files, and the backup duration next to the
  raw/packed byte figures.
- **Last backup in the rule list.** Each rule shows when its newest snapshot was
  made and turns the line red when the schedule is overdue (twice the interval,
  36 hours for daily and 8 days for weekly rules).
- **Recommended exclusions.** The rule editor can add caches, the KDE file index
  (`~/.local/share/baloo`), Akonadi and `.var/app/*/cache` in one click; new
  rules get these exclusions by default.
- **Repository integrity checks.** **Settings → Maintenance** can verify the
  repository structure and index in the background, or additionally read a
  random 10% of the pack data. The result and its date are saved, and a check
  older than 30 days is highlighted. The same check is available headlessly as
  `kustos --check [--read-data]`.
 Viewing passwords are kept in memory only and can be forgotten
  after a configurable period without activity (**Settings → Security →
  Lock after inactivity**, including *Never*). **Lock Now** / `Ctrl+L` locks immediately.
  Locked rules hide their snapshots and show an unlock panel instead.
- **Per-rule systemd user units.** A scheduled rule gets its own
  `kustos-backup-<id>.service` and `.timer`; the service runs
  `kustos --backup --rule <id> --quiet`, so the window does not have to be
  open. The rule (or generated backup) key is supplied as an encrypted
  `systemd-creds` credential (`LoadCredentialEncrypted=`); on systems without
  `systemd-creds` a private 0600 credential file is used as a fallback. The GUI
  uses the same credential to start backups without asking.
- Backup-rule editor: any number of rules, each with its own folders/files,
  excluded paths and glob patterns (`*.log`, `node_modules`, `~/.cache/**`),
  and a schedule (interval, daily time or selected weekdays). Old single-scope
  settings migrate into a default *Home* rule.
- A `kustos --backup --rule ID` entry point and rule names in the snapshot
  timeline and `--list` output. Overlapping jobs wait on a lock file instead of
  failing on the repository lock.
- *Save Copies…* / *Save This Item…*: copy files or folders from a snapshot
  directly into a folder chosen in the file manager (flat, with a timestamped
  sibling instead of overwriting).
- **Open**: double-click a file in a snapshot (or use *Open* in the toolbar
  and context menu) to hand it to the default application, like Dolphin. The
  file is extracted into a private folder under `/tmp` first, so a reboot
  cleans it up. A single click only selects that row; *Ctrl+click* toggles
  items for bulk save/restore.
- The main window is split into rules / snapshots of the selected rule / files.
  A rule's context menu starts or stops its backup, edits it, pauses or resumes
  it and deletes it; snapshots without a matching rule are grouped under
  *Other Backups*.
- Encrypted, compressed and deduplicated snapshots: foreign formats (restic v2
  repository written by `rustic`), repository location and password policy in
  About.
- File browser inside a snapshot, per-item restore with overwrite / skip /
  keep-both policies, and a destination folder.
- Automatic backups, runtime English/Russian switch, `notify-send`
  notifications.

### Fixed

- Duplicate viewing passwords are accepted. The *"this password is already used
  by another backup"* error is gone; a password is verified only by opening the
  repository. A valid repository password may therefore open any rule in the
  UI, which matches the shared master key.
- The window no longer freezes while a backup password is verified, the
  snapshot list is read, or a snapshot is opened. These repository calls now run
  in the background and show a busy indicator.
- Timeline and browser panels use `backgroundColor`/`textColor` from one color
  set, so labels stay readable in dark Plasma themes.
- The breadcrumb was never filled (a QML `id` shadowed the property of the same
  name); the browser now shows where you are and the snapshot date/time.
- The rule editor puts *What to copy* and *Excluded paths* side by side and
  uses **Save** instead of starting a backup from the editor. The whole page
  scrolls as one — the bottom margin and long source/exclusion lists are no
  longer cut off below the fold, and hovering a list no longer steals the
  wheel from the page. Card dialogs are opaque, context menus cannot cover an
  open password prompt, lists reserve room for their scrollbar, and no
  snapshot or file is selected automatically.
- The timeline reports the space a snapshot actually added to the repository
  after compression and deduplication, not the pre-compression volume of new
  data. The latter is still available in the tooltip, together with an
  explanation of both numbers.
- The snapshot file list reserves a gutter for its scrollbar and a fixed
  checkmark column, so a selected row never runs under the scrollbar and the
  Name/Modified/Size columns do not jump. As in Dolphin, a single click selects
  only that row (it no longer toggles a selected row off), *Ctrl+click* toggles
  selection, and double-click opens a file.
- The backup status can no longer stay stuck on "running": a divide-by-zero in
  the speed formatter (an elapsed time between 0.5 and 1 second was truncated
  to zero) is fixed, and backup/restore worker panics now reset the job to
  failed instead of blocking every later run. The `Kustos` logo was removed
  from the application drawer header.
- **Passwords no longer disappear during a rebuild or reinstall.** `cargo test`
  (and therefore the `check()` step of `makepkg`) used to write the real
  `~/.config/kustos/kustos.json`; tests now get a process-specific temporary
  XDG tree. Config writes are atomic, keep `kustos.json.bak`, fall back to it
  when the main file is unreadable, and preserve key IDs that are still on
  disk. Common mode also propagates `common_key_id` to every rule, so the lock
  panel cannot say "no password yet" while Settings says the common password
  is set.
- Right-clicking a locked rule only selects it and opens its context menu; it
  no longer asks for the viewing password just because the menu was opened.
  The password is requested by the explicit *Unlock Backup…* and *Edit Rule…*
  actions (and by left-clicking a locked rule to view its snapshots); after a
  successful unlock in the *Edit Rule…* case the editor opens automatically.

### Security

- Rule passwords are never written to `kustos.json`: they are asked for when a
  backup is opened for viewing and dropped again after the inactivity timeout.
  Starting a backup decrypts the saved credential instead; a local user who
  can run the GUI can therefore always start a backup, while the viewing
  password gates the snapshot browser. Each rule's
  password is a separate key in the repository and unlocks that rule's
  snapshots in the UI. The repository master key is shared, so someone who
  knows one key can still read all snapshots with `rustic` or `kustos --list`;
  this is per-rule access control in the application, not a cryptographic
  per-rule boundary. Passwords may repeat: `rustic` cannot tell which key a
  duplicate password opened, so the application verifies only that the
  password opens the repository and does not enforce a password-to-rule
  mapping. In common mode every rule uses the same viewing key; per-rule
  passwords are separate keys.
- Passwordless backups (GUI and systemd) need a credential on disk. Kustos
  encrypts the key with `systemd-creds --user` and passes it to the service as
  `LoadCredentialEncrypted=`; the plaintext file exists only briefly while the
  credential is created. Where `systemd-creds` is unavailable, a 0600 file is
  used instead. This is the explicit trade-off for backups without a prompt.
- The CLI reads `KUSTOS_PASSWORD`, `KUSTOS_PASSWORD_FILE` or the
  `$CREDENTIALS_DIRECTORY/kustos-password` file created by the generated
  units.

### Changed

- New Papirus-style application icon: three stacked snapshot layers on a teal
  tile. The window, About dialog, desktop entry and package use it; 32/48/64/
  128/256 px exports are in `assets/icon-concepts/png/`.
- **Backups no longer ask for a password.** The GUI resolves the key from an
  already unlocked backup, a saved systemd credential, or — for a repository
  that does not exist yet — a generated key that it initializes the repository
  with and saves as a credential. Only an old repository with no saved
  credential asks for its key once; that unlock saves the credential too.
- The startup *Unlock the Repository* dialog and the one-time generated key
  dialog are gone. A new repository is initialized with the first viewing
  password or with a generated backup key.
- A backup that finishes while its snapshots are locked no longer selects the
  new snapshot (that would try to read the repository without a password); it
  shows *unlock the backup to see the new snapshot* instead.
- Existing installations keep their repository key: the first backup or the
  first open without a saved credential asks for the existing key, then saves
  it as a credential. Rules without a credential no longer get a timer until a
  backup creates one or a viewing password is set.
- The header *Back Up Now* now starts backups immediately instead of opening
  the scope dialog; the selected rule is edited from its own *Edit Rule…* action.
- The backup scope is no longer global: each rule carries its own sources and
  exclusions, while retention stays repository-wide.
- Rewritten in Rust (`qmetaobject` + QML/Kirigami) instead of C++/CMake; the
  rsync hard-link engine is replaced by a restic-compatible repository.
- Renamed from Aion to Kustos (the old name collides with a game).
- Arch packaging: `PKGBUILD` builds from the working tree (`makepkg -si`).

## [0.1.0]

- First GUI prototype: rsync hard-link snapshots, timeline, restore, settings.
