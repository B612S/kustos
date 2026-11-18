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
- `src/bridge.cpp` is compiled by `build.rs` with `pkg-config` (Qt6Core, Gui,
  Qml, Quick). It exists only for the `QTranslator` and the clipboard.
- Headless UI check (offscreen, no display):

  ```bash
  XDG_CONFIG_HOME=/tmp/xdg XDG_CACHE_HOME=/tmp/cache \
  QT_QPA_PLATFORM=offscreen KUSTOS_CAPTURE=/tmp/shot.png \
  ./target/debug/kustos
  ```

  Renders the window to the file and exits; `KUSTOS_CAPTURE_DIALOG=backup`,
  `settings`, `about` or `restore` opens that dialog first. Development-only.
- QML warnings are not reliably printed by the Qt build in use. When a binding
  silently fails, add `App.trace("…")` calls (they append to
  `~/.cache/kustos/kustos.log`) or check `rustic`/repository errors in the log.

## Architecture map

- `src/main.rs` — CLI (`--backup`, `--list`, `--prune`, `--restore`,
  `--print-config`, `--version`, `--help`), resource registration (`qrc!`),
  GUI bootstrap: `QmlEngine`, `QTranslator`, context properties, `exec()`.
- `src/objects.rs` — the QObjects exposed to QML: `Config`, `Store`, `Backup`,
  `Restore`, `Browser`, `Scheduler`, `App`. Long jobs run on `std::thread` and
  publish `Arc<Mutex<JobStatus>>`; QML polls with a `Timer` and a JS property.
- `src/rustic.rs` — process wrapper and JSON parsing (`backup --json-progress`,
  `snapshots --json`, `ls --json --long`, `restore --json-progress`, `forget`).
- `src/config.rs` — JSON settings (`~/.config/kustos/kustos.json`), defaults,
  password generation; `src/paths.rs` — XDG paths.
- `src/schedule.rs` — systemd user timer; `src/notify.rs` — `notify-send`;
  `src/i18n.rs` — translator installation through the bridge.
- `qml/` — flat directory. `Main.qml` owns the window, the polling timers and
  the dialogs; `TimelinePanel`, `SnapshotBrowser` are views exchanging plain JS
  arrays with Rust (`Store.snapshots()`, `Browser.entries()`); `BackupDialog`
  edits the scope; `SettingsDialog`, `RestoreDialog`, `AboutDialog` are cards;
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
- Data reaches QML as `QVariantList` of `QVariantMap`; QML treats it as an
  array of objects (`modelData.name`). Models are not used, so every list is
  pulled by an invokable and cached in a JS property.
- Never call `engine.load_file()` before `QmlEngine::new()`: the engine creates
  the `QGuiApplication`, and a `QTranslator` installed earlier is dropped.

## Snapshot engine rules

- Tag every backup `kustos`; `snapshots` is filtered by that tag.
- Exclusions are passed as `--glob '!<absolute path>'`; without `!` rustic
  treats a pattern as an *include* and backs up almost nothing.
- Snapshot paths are relative to the snapshot root (`home/<user>/...`).
- Restore: `rustic restore <id>:<path> <staging>` then `cp -a` into the target;
  policies are overwrite / skip / keep_both.
- After a successful backup, `forget --keep-hourly/daily/weekly --prune` is
  applied. Never remove the only snapshot.
- The repository key is `~/.config/kustos/repo.password` (0600); it is the only
  way to read the backups and is shown in About.

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
  behind a confirmation dialog.
