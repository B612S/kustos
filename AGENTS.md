# AGENTS.md

Aion: Time Machine-style incremental snapshots of a Linux home folder.
C++23 + Qt 6 / QML (Kirigami) GUI over `rsync --link-dest`. CMake is the only
build system. The window, menu, settings-dialog and localization conventions
are deliberately copied from the sibling project **Luma**
(`../luma`): 960x640 window with a 640x480 minimum, `GlobalDrawer` for the
application menu, custom overlay context menus, an 800x600 settings card with a
category sidebar, key-style `.ts` sources and a runtime language switch.

## Build & verify

```bash
cmake -B build -S . -DCMAKE_BUILD_TYPE=Debug
cmake --build build -j$(nproc)
./build/bin/aion
```

- The binary is `build/bin/aion` (`CMAKE_RUNTIME_OUTPUT_DIRECTORY`).
- Tests are opt-in (`-DAION_BUILD_TESTS=ON`), need no display and cover pure
  logic only: retention buckets, configuration resolution, snapshot scanning.
  They redirect `XDG_CONFIG_HOME` to a temporary directory.
- Headless UI check (no display, no real data):

  ```bash
  XDG_CONFIG_HOME=/tmp/xdg XDG_CACHE_HOME=/tmp/cache \
  QT_QPA_PLATFORM=offscreen QT_QUICK_BACKEND=software \
  ./build/bin/aion --smoke-test
  ```

  It loads `Main.qml`, prints `AION_SMOKE_OK` and exits 0. A QML error makes
  `engine.rootObjects()` empty and the process exits 1.
- Development-only screenshots, useful for reviewing the layout without a
  desktop session:

  ```bash
  ./build/bin/aion --capture /tmp/main.png
  ./build/bin/aion --capture /tmp/settings.png --capture-dialog settings
  ./build/bin/aion --capture /tmp/sources.png --capture-dialog settings:sources
  ```

  `--capture-dialog` also accepts `settings:schedule`, `settings:retention`,
  `restore`, `about`. These flags are not part of the user-facing CLI.

## Architecture map

- `main.cpp` is the composition root: it decides between the CLI and the GUI,
  installs the `QTranslator`, builds `ConfigManager`, `SnapshotStore`,
  `SnapshotModel`, `DirectoryModel`, `BackupRunner`, `RestoreRunner`,
  `Autostart`, `AppController` and exposes them to QML as context properties
  (`Config`, `Store`, `Timeline`, `Browser`, `Backup`, `Restore`, `Scheduler`,
  `App`). Single-instance is `QLocalServer` `aion_single_instance_server`.
- `core/app/` — `ConfigManager` (QSettings INI) and `Logger`.
- `core/backup/` — `Snapshot` (meta.json value type), `SnapshotStore` (scan,
  meta, `latest` link, delete, prune), `Retention` (pure policy logic),
  `BackupRunner` (one rsync run), `RestoreRunner` (per-item rsync),
  `SnapshotModel`/`DirectoryModel` (QML models).
- `core/schedule/Autostart` — writes and controls the systemd user timer.
- `core/ui/` — `AppController` (version, paths, clipboard, `openPath`),
  `Notifier` (org.freedesktop.Notifications).
- `core/util/` — `Format` (bytes/duration) and `Loc` (`Loc::t` for strings
  produced outside QML).
- `qml/` — flat directory, no Qt Quick Designer forms. `Main.qml` owns the
  window and the dialogs; `TimelinePanel`, `SnapshotBrowser` are views;
  `SettingsDialog`, `RestoreDialog`, `AboutDialog` are overlay cards;
  `AionMenu`/`AionMenuButton` are the Luma-style context menus.
- Every C++ class exposed to QML must be reachable: properties through
  `Q_PROPERTY`, methods called from QML through `Q_INVOKABLE`. A plain C++
  method silently fails at runtime with "is not a function".

## Localization

- User-visible strings are key-style: `qsTr("settings_language")` in QML,
  `Loc::t("error_no_sources")` in C++ (context `aion`).
- `.ts` files are generated, not hand-edited:

  ```bash
  tools/generate-ts.py   # fails on a key without translation
  ```

  The script scans `qml/*.qml` and the C++ sources; the translation table lives
  in it. Add the German/English pair there, then rebuild.
- `qt_add_translations` compiles `locales/*.ts` into `.qm` and embeds them as
  `:/i18n/<lang>.qm`; `main.cpp` also tries `/usr/share/aion/translations/` and
  the binary directory. The language combo switches at runtime through
  `QTranslator` (QML re-evaluates `qsTr` bindings automatically).
- Keep the TS list in `CMakeLists.txt` in sync when adding a language.

## Snapshot engine rules

- A snapshot directory is created before rsync runs and deleted when rsync
  fails, so a failed run never leaves a half snapshot behind.
- `--link-dest` points at the *latest* snapshot; unchanged files become hard
  links. Never write into an existing snapshot.
- Missing sources are skipped with a warning; an empty source list is an error.
- The destination may not live inside a source (checked with canonical paths).
- A `QLockFile` in the backup folder serializes the GUI, CLI and timer.
- rsync exit codes 23/24 mean "partial": the snapshot is kept and marked
  `warning`, not deleted.

## Conventions

- Code comments and commit messages in English; strings in `qsTr`/`Loc::t`.
- `VERSION` is the single source of truth (`MAJOR.MINOR.PATCH`); CMake reads it
  and passes `AION_VERSION` to `main.cpp`.
- Keep `CHANGELOG.md` current under `## [Unreleased]`.
- No X11-specific code: the app runs on Wayland and X11, but nothing depends on
  either platform.
- Deleting a snapshot is the only destructive operation on user data; it stays
  behind a confirmation dialog.
