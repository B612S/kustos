# AGENTS.md

Kustos: Time Machine-style backups of a Linux home folder. **Rust** application
with a QML/Kirigami GUI (Qt 6 via `qmetaobject`), backed by `rustic`
(restic-compatible encrypted, compressed, deduplicated repository). Window,
menu, settings-dialog and localization conventions follow the sibling project
**Luma** (`../luma`): 960x640 window with a 640x480 minimum, `GlobalDrawer` for
the application menu, custom overlay context menus, an 800x600 settings card
with a category sidebar, key-style `.ts` sources and a runtime language switch.

## Build & verify

```bash
cargo build                 # debug build in target/debug/kustos
cargo build --release
cargo test                  # unit tests, no display needed
```

- Qt is found through `qmake`; `.cargo/config.toml` pins `QMAKE=/usr/bin/qmake6`
  because the machine also has Qt 5. Never remove that line.
- `rust/bridge.cpp` is compiled by `build.rs` with `pkg-config` (Qt6Core, Gui,
  Qml, Quick). It installs the `QTranslator`, uses the Qt clipboard and keeps
  the timestamp of the last user event for the inactivity lock.
- Headless UI check (offscreen, no display):

  ```bash
  XDG_CONFIG_HOME=/tmp/xdg XDG_CACHE_HOME=/tmp/cache \
  QT_QPA_PLATFORM=offscreen KUSTOS_CAPTURE=/tmp/shot.png \
  ./target/debug/kustos
  ```

  Renders the window to the file and exits; `KUSTOS_CAPTURE_DIALOG=rules` (or
  the legacy `backup`), `settings`, `about`, `password` (legacy `key`) or
  `restore` opens that dialog first. Set `KUSTOS_PASSWORD=…` together with
  `KUSTOS_CAPTURE` to capture a real, unlocked repository (development-only).
  `KUSTOS_CAPTURE_ACTION=backup` starts a backup through the normal GUI path
  (credential resolution included) before the screenshot; this is how the
  passwordless backup flow is tested headlessly.
- QML warnings are not reliably printed by the Qt build in use. When a binding
  silently fails, add `App.trace("…")` calls (they append to
  `~/.cache/kustos/kustos.log`) or check `rustic`/repository errors in the log.
- Tests must never touch the real `~/.config/kustos/kustos.json`: under
  `cfg(test)`, `paths::config_dir()`/`cache_dir()` redirect XDG to a
  process-specific temporary tree. `cargo test` runs inside `makepkg -si`;
  writing the real config from a test wipes the user's password key bindings.

## Architecture map

- `rust/main.rs` — CLI (`--backup`, `--list`, `--prune`, `--restore`,
  `--print-config`, `--version`, `--help`), resource registration (`qrc!`),
  GUI bootstrap: `QmlEngine`, `QTranslator`, context properties, `exec()`.
- `rust/objects.rs` — the QObjects exposed to QML: `Config`, `Store`, `Backup`,
  `Restore`, `Browser`, `Scheduler`, `App`. Long jobs run on `std::thread` and
  publish `Arc<Mutex<JobStatus>>`; QML polls with a `Timer` and a JS property.
- `rust/rustic.rs` — process wrapper and JSON parsing (`backup --json-progress`,
  `snapshots --json`, `ls --json --long`, `restore --json-progress`, `forget`).
- `rust/config.rs` — JSON settings (`~/.config/kustos/kustos.json`), the
  `BackupRule` list (sources, excludes, per-rule schedule, the rustic key ID),
  the inactivity-lock timeout, the password mode (`per_rule` / `common`) and
  the common key ID, legacy single-scope migration and defaults;
  `rust/paths.rs` — XDG paths.
- `rust/vault.rs` — in-memory per-rule **viewing** passwords (each one is a
  rustic key), the lock generation QML polls, and the inactivity timeout.
- `rust/joblock.rs` — advisory `flock` that serializes backup jobs across
  rules, GUI and CLI.
- `rust/schedule.rs` — one systemd user **service and timer** per scheduled
  rule (`kustos-backup-<id>.service` + `.timer`) and the per-rule credential
  (`systemd-creds` or a 0600 fallback) that lets the service — and the GUI's
  backup job — unlock the repository without a prompt; `rust/notify.rs` — `notify-send`;
  `rust/i18n.rs` — translator installation through the bridge.
- `qml/` — flat directory. `Main.qml` owns the window, the polling timers and
  the dialogs; `TimelinePanel`, `SnapshotBrowser` are views exchanging plain JS
  arrays with Rust (`Store.snapshots()`, `Browser.entries()`). The main window
  is three columns: `RulesPanel` (rules + context menu), `TimelinePanel`
  (snapshots of the selected rule) and `SnapshotBrowser` (files). `Main.qml`
  owns the rule filtering, selection and the synthetic "Other Backups" group
  for untagged snapshots. `BackupDialog` is the detailed rule editor;
  `UnlockDialog` is the password dialog (per-rule unlock/create/change and the
  common-password variants); `SettingsDialog` has the General, Security and
  Retention pages; `RestoreDialog`, `AboutDialog` are cards;
  `KustosMenu`/`KustosMenuButton` are the Luma-style context menus.

## QML rules (learned the hard way)

- A QML `id` shadows a property with the same name inside JS functions:
  `Row { id: crumbs }` plus `property var crumbs` makes `crumbs = …` fail with
  *"left-hand side of assignment operator is not an lvalue"*. Keep ids and
  property names distinct.
- Bindings that call a C++ invokable (`Browser.path()`) do not re-evaluate when
  the object changes; recompute such values in a function and store them in a
  property (`crumbs`, `entries`) instead of binding to the call.

## qmetaobject rules (learned the hard way)

- A method is exposed to QML by the **field name** of `qt_method!`, and the Rust
  function must have exactly the same identifier. Use camelCase
  (`setSnapshot`, `freeBytes`); a snake_case method is simply missing in QML and
  the call fails with a hidden TypeError.
- Properties use the field name as-is (`repo`, `scheduleEnabled`); notify
  signals are `<field>_changed` and are emitted explicitly after changing the
  field.
- A QML `Connections` handler for a qmetaobject signal keeps the underscore:
  `rules_changed` is handled by `function onRules_changed()`, **not**
  `onRulesChanged`; the camelCase spelling silently never fires.
- Data reaches QML as `QVariantList` of `QVariantMap`; QML treats it as an
  array of objects (`modelData.name`). Models are not used, so every list is
  pulled by an invokable and cached in a JS property.
- Never call `engine.load_file()` before `QmlEngine::new()`: the engine creates
  the `QGuiApplication`, and a `QTranslator` installed earlier is dropped.

## Snapshot engine rules

- Tag every backup `kustos`; `snapshots` is filtered by that tag. A backup made
  by a rule also gets `kustos-rule:<id>` so the UI can resolve the rule name.
- A backup runs once per enabled rule (or once for an explicit
  `--backup --rule <id>`), so one run can create several snapshots.
- Every backup job takes an advisory `flock` on
  `~/.cache/kustos/backup.lock` (`rust/joblock.rs`) before touching `rustic`,
  so two timers firing at once wait instead of failing on the repository lock.
- Exclusions are passed as `--glob '!<pattern>'`; without `!` rustic treats a
  pattern as an *include* and backs up almost nothing. `~/` is expanded and a
  user-supplied leading `!` is stripped before the caller adds it.
- Snapshot paths are relative to the snapshot root (`home/<user>/...`).
- Restore: `rustic restore <id>:<path> <staging>` then `cp -a` into the target;
  policies are overwrite / skip / keep_both.
- "Save Copies…" reuses the restore job with `mode=extract`: the staged item is
  copied flat into the user-chosen folder (`<folder>/<name>`) and an existing
  target is never overwritten — it gets a `(restored <timestamp>)` sibling.
- After a successful backup, `forget --keep-hourly/daily/weekly --prune` is
  applied. Never remove the only snapshot.
- Passwords are per rule and live in memory only (`rust/vault.rs`). Each rule
  password is a separate `rustic key` of the same repository; `rule.key_id`
  records which key belongs to which rule. There is no startup prompt: a rule
  asks for its viewing password when it is opened. A backup can initialize a
  new repository with a generated key instead, saving it as a credential. All
  keys unlock the same master key, so this is per-rule access control in the
  UI, not per-rule encryption; do not claim otherwise.
  Passwords may repeat. `rustic` cannot tell which key a duplicate password
  opened, so the application verifies only that the password opens the
  repository; it does not enforce a password-to-rule mapping. `rule.key_id` is
  for management (binding, removal), not authentication. Any valid repository
  password unlocks a rule, and all keys open the same data.
- **Common password mode.** `Config.password_mode` (`per_rule` / `common`) and
  `Config.common_key_id` are the switch. In common mode `Store.setCommonPassword`
  adds one key, stores it in every rule's credential and caches it for every
  rule; `Config.bindCommonKey` points all rules at it and stores the canonical
  ID. A common unlock caches the password for every rule and returns
  `bind_all`; QML then calls `Config.bindCommonKey`. Changing the common
  password adds a new key and best-effort removes the old common key. In
  per-rule mode none of this applies; each unlock affects one rule.
- **Starting a backup never needs the vault.** `BackupObject` resolves the
  key from the vault, a saved credential (`schedule::load_credential`) or a
  generated key for a repository that does not exist yet (it initializes the
  repository and saves the key as a credential). Do not put that key into the
  vault: a backup must not unlock the timeline. When a backup finishes while
  its rule is locked, QML must not select the new snapshot (the browser would
  try to read the repository without a password); show the *unlock to view*
  toast instead. Viewing still needs the per-rule password.
- Inactivity re-lock is driven by the C++ event filter in `rust/bridge.cpp`
  (`kustos_last_activity_ms`) and the guard thread in `rust/activity.rs`; QML
  polls `Store.lockGeneration()` and hides the snapshots when it changes.
- Backups without a prompt need a credential on disk. `rust/schedule.rs`
  encrypts the rule or generated backup key with `systemd-creds --user` and the
  unit loads it with `LoadCredentialEncrypted=`; without `systemd-creds` it
  falls back to a 0600 file. The GUI also decrypts this credential to start a
  backup while the viewing passwords are locked. This is the explicit
  exception to "never write the key to disk" — it exists to make passwordless
  GUI and systemd backups possible. The CLI reads `KUSTOS_PASSWORD`,
  `KUSTOS_PASSWORD_FILE` or `$CREDENTIALS_DIRECTORY/kustos-password`.
- `rustic key add` can leave the newly created key ambiguous when passwords
  collide; `rust/rustic.rs` identifies it by the difference of the key lists
  before and after. Duplicate passwords are allowed on purpose.

## Localization

- User-visible strings are key-style: `qsTr("settings_language")` in QML only.
  Rust keeps messages untranslated on purpose (logs, CLI), so the TS files have
  QML contexts only.
- `.ts` files are generated, never hand-edited: `tools/generate-ts.py` fails on
  a key without a translation. Compile with
  `/usr/lib/qt6/bin/lrelease locales/<lang>.ts -qm locales/<lang>.qm`; the `.qm`
  files are committed and embedded through `qrc!`.
- `qt_add_translations` is gone; the resource path is `:/i18n/<lang>.qm`.

## Conventions

- Comments and commit messages in English.
- `Cargo.toml`, `VERSION` and `PKGBUILD` carry the same version (0.2.0).
- `PKGBUILD` builds from the working tree (`$startdir`); `makepkg -si` is the
  Arch install path. It runs `cargo build --release --locked` and `cargo test`.
- Keep `CHANGELOG.md` current under `## [Unreleased]`.
- Deleting a snapshot is the only destructive user-data operation; it stays
  behind a confirmation dialog. Deleting a rule removes its configuration,
  credential and (best effort, when another key can keep the repository
  accessible) its rustic key; its snapshots stay in the repository and move to
  the synthetic *Other Backups* group.
