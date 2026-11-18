#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
# SPDX-FileCopyrightText: 2026 B612S <sem.alimov@inbox.ru>
"""Regenerates locales/en.ts and locales/ru.ts from the translation keys.

Kustos uses key-style sources (qsTr("settings_language"), Loc::t("error_no_sources"))
so a missing translation is a hard error instead of a silently English UI. Run
this script after adding or renaming a key; it fails when a key has no entry.

    tools/generate-ts.py
"""

from __future__ import annotations

import pathlib
import re
import sys
from xml.sax.saxutils import escape

ROOT = pathlib.Path(__file__).resolve().parent.parent

# key: (English, Russian)
TRANSLATIONS: dict[str, tuple[str, str]] = {
    # ---- application shell (Main.qml) ----
    "app_title": ("Kustos", "Kustos"),
    "action_back_up_now": ("Back Up Now", "Создать резервную копию"),
    "action_restore": ("Restore…", "Восстановить…"),
    "action_cancel": ("Cancel", "Отмена"),
    "action_delete": ("Delete", "Удалить"),
    "action_settings": ("Settings", "Настройки"),
    "action_about": ("About Kustos", "О программе"),
    "action_quit": ("Quit", "Выход"),
    "action_open_backup_folder": ("Open Backup Folder", "Открыть папку копий"),
    "delete_snapshot_title": ("Delete Snapshot", "Удалить снимок"),
    "delete_snapshot_message": (
        "Delete snapshot %1? Files that exist only in this snapshot cannot be recovered.",
        "Удалить снимок %1? Файлы, которые есть только в нём, восстановить не получится.",
    ),
    "empty_title": ("No backups yet", "Резервных копий пока нет"),
    "empty_hint": (
        "Kustos keeps a history of your home folder, like Time Machine. "
        "Every snapshot stores only what changed since the previous one.",
        "Kustos хранит историю домашней папки, как Time Machine. "
        "Каждый снимок занимает место только за изменения с предыдущего.",
    ),
    "status_cancelling": ("Cancelling…", "Отмена…"),
    "status_copying": ("Copying… %1, %2 left", "Копирование… %1, осталось %2"),
    "status_preparing": ("Preparing the file list…", "Подготовка списка файлов…"),
    "status_restoring": ("Restoring %1", "Восстановление %1"),
    "status_summary": ("%1 snapshot(s) · %2 · %3 free", "Снимков: %1 · %2 · свободно %3"),
    "status_working": ("Working…", "Выполняется…"),
    "toast_backup_done": ("Backup finished", "Резервное копирование завершено"),
    "toast_backup_failed": ("Backup failed: %1", "Ошибка резервного копирования: %1"),
    "toast_restore_done": ("Restored %1 item(s)", "Восстановлено элементов: %1"),
    "toast_restore_failed": ("Restore failed: %1", "Ошибка восстановления: %1"),

    # ---- timeline (TimelinePanel.qml) ----
    "timeline_title": ("Backups", "Резервные копии"),
    "timeline_hint": (
        "Newest first. Pick a snapshot to browse it and restore files.",
        "Сверху самые свежие. Выберите снимок, чтобы просмотреть его и восстановить файлы.",
    ),
    "timeline_count": ("%1 snapshot(s)", "Снимков: %1"),
    "timeline_empty": (
        "The timeline appears here after the first backup.",
        "Таймлайн появится после первой резервной копии.",
    ),
    "day_today": ("Today", "Сегодня"),
    "day_yesterday": ("Yesterday", "Вчера"),
    "snapshot_added": ("%1 added", "добавлено %1"),
    "snapshot_latest": ("Latest", "Последняя"),
    "snapshot_status_failed": ("Failed", "Ошибка"),
    "menu_copy_id": ("Copy Snapshot ID", "Скопировать ID снимка"),
    "menu_delete_snapshot": ("Delete Snapshot…", "Удалить снимок…"),

    # ---- browser (SnapshotBrowser.qml) ----
    "browser_home": ("Home", "Домой"),
    "browser_snapshot": ("Snapshot %1, %2", "Снимок %1, %2"),
    "browser_name": ("Name", "Имя"),
    "browser_modified": ("Modified", "Изменён"),
    "browser_size": ("Size", "Размер"),
    "browser_up": ("Up", "Вверх"),
    "browser_selected": ("Selected: %1", "Выбрано: %1"),
    "browser_select_all": ("Select All", "Выбрать всё"),
    "browser_clear_selection": ("Clear Selection", "Снять выделение"),
    "browser_empty": ("This folder is empty", "Папка пуста"),
    "browser_empty_hint": (
        "Pick another snapshot or go up a level.",
        "Выберите другой снимок или поднимитесь выше.",
    ),
    "menu_restore_item": ("Restore This Item…", "Восстановить этот элемент…"),
    "menu_reveal_item": ("Show in File Manager", "Показать в файловом менеджере"),
    "menu_copy_path": ("Copy Path", "Скопировать путь"),

    # ---- restore dialog (RestoreDialog.qml) ----
    "restore_title": ("Restore from a Snapshot", "Восстановление из снимка"),
    "restore_count": ("Items to restore: %1", "Элементов к восстановлению: %1"),
    "restore_more": ("and %1 more", "и ещё %1"),
    "restore_destination": ("Destination", "Куда восстанавливать"),
    "restore_original": ("Original location", "Исходное место"),
    "restore_folder": ("Another folder…", "В другую папку…"),
    "restore_choose_folder": ("Choose Folder", "Выбрать папку"),
    "restore_into": ("Restoring into", "Папка назначения"),
    "restore_conflict": ("If a file already exists", "Если файл уже существует"),
    "restore_policy_overwrite": ("Replace it", "Заменить"),
    "restore_policy_skip": ("Keep the existing file", "Оставить существующий"),
    "restore_policy_keep_both": ("Keep both", "Сохранить оба"),

    # ---- settings (SettingsDialog.qml) ----
    "settings_title": ("Settings", "Настройки"),
    "backup_title": ("What to Back Up", "Что копировать"),
    "backup_include_title": ("Folders to back up", "Какие папки копировать"),
    "backup_include_hint": (
        "Every listed folder and file is copied into the snapshot.",
        "Все перечисленные папки и файлы попадают в снимок.",
    ),
    "backup_exclude_title": ("Excluded paths", "Исключения"),
    "backup_exclude_hint": (
        "Nothing below these paths is copied, even if it is inside a backed-up folder.",
        "Ничего внутри этих путей не копируется, даже если они лежат внутри копируемой папки.",
    ),
    "backup_add_folder": ("Add Folder…", "Добавить папку…"),
    "backup_add_file": ("Add File…", "Добавить файл…"),
    "backup_add_exclude": ("Exclude Folder…", "Исключить папку…"),
    "backup_no_sources": (
        "Add at least one folder or file to back up.",
        "Добавьте хотя бы одну папку или файл для копирования.",
    ),
    "settings_scope": ("Backup Scope…", "Что копировать…"),
    "settings_general": ("General", "Общие"),
    "settings_sources": ("Sources", "Источники"),
    "settings_schedule": ("Schedule", "Расписание"),
    "settings_retention": ("Retention", "Хранение"),
    "settings_repo": ("Repository folder", "Папка репозитория"),
    "settings_choose_folder": ("Choose Folder", "Выбрать папку"),
    "settings_language": ("Language", "Язык"),
    "language_auto": ("System", "Системный"),
    "settings_notifications": ("Notify when a backup finishes", "Уведомлять о завершении"),
    "settings_single_instance": ("Single window", "Одно окно"),
    "settings_preserve_acl": ("Preserve ACLs", "Сохранять ACL"),
    "settings_preserve_xattr": ("Preserve extended attributes", "Сохранять расширенные атрибуты"),
    "settings_open_config": ("Open Config File", "Открыть файл настроек"),
    "settings_reset": ("Reset to Defaults", "Сбросить настройки"),
    "settings_sources_hint": (
        "Folders and files that are backed up. Missing entries are skipped with a warning.",
        "Папки и файлы, которые копируются. Отсутствующие пропускаются с предупреждением.",
    ),
    "settings_remove": ("Remove", "Удалить"),
    "settings_add_folder": ("Add Folder…", "Добавить папку…"),
    "settings_add_file": ("Add File…", "Добавить файл…"),
    "settings_excludes": ("Excluded paths", "Исключения"),
    "settings_excludes_hint": (
        "Never backed up: games, caches, the recycle bin. Paths are matched by rsync.",
        "Не копируются: игры, кэши, корзина. Пути проверяются rsync.",
    ),
    "settings_add_excluded_folder": ("Exclude Folder…", "Исключить папку…"),
    "settings_schedule_enabled": ("Automatic backups", "Автоматические копии"),
    "settings_schedule_interval": ("Interval", "Интервал"),
    "schedule_every_minutes": ("Every %1 min", "Каждые %1 мин"),
    "schedule_every_hours": ("Every %1 h", "Каждые %1 ч"),
    "settings_schedule_state": ("Timer: %1", "Таймер: %1"),
    "schedule_state_active": ("running", "работает"),
    "schedule_state_inactive": ("installed, not running", "установлен, не запущен"),
    "schedule_state_absent": ("not installed", "не установлен"),
    "schedule_state_unknown": ("unknown", "неизвестно"),
    "settings_retention_hint": (
        "Old snapshots are thinned out like Time Machine: everything from the last hours, "
        "then one per day, then one per week.",
        "Старые снимки прореживаются как в Time Machine: всё за последние часы, "
        "затем по одному в день, затем по одному в неделю.",
    ),
    "settings_keep_hourly": ("Keep hourly snapshots for (hours)", "Хранить почасовые (часов)"),
    "settings_keep_daily": ("Keep daily snapshots for (days)", "Хранить ежедневные (дней)"),
    "settings_keep_weekly": ("Keep weekly snapshots for (weeks)", "Хранить еженедельные (недель)"),
    "settings_prune_now": ("Prune Now", "Проредить сейчас"),
    "settings_pruned_now": ("Retention policy applied", "Политика хранения применена"),
    "settings_nothing_to_prune": ("Nothing to prune", "Нечего удалять"),

    # ---- about (AboutDialog.qml) ----
    "about_title": ("About Kustos", "О программе Kustos"),
    "about_version": ("Version %1", "Версия %1"),
    "about_description": (
        "Time Machine-like snapshots of your home folder. Snapshots live in an "
        "encrypted, compressed, deduplicated repository (restic format), so unchanged "
        "data never costs extra space.",
        "Снимки домашней папки в духе Time Machine. Они лежат в зашифрованном, сжатом "
        "репозитории с дедупликацией (формат restic), поэтому неизменённые данные не "
        "занимают лишнего места.",
    ),
    "about_repo": ("Repository", "Репозиторий"),
    "about_config": ("Configuration", "Настройки"),
    "about_log": ("Log", "Журнал"),
    "about_credits": (
        "Built with Rust, Qt 6, Kirigami and rustic (restic-compatible repositories). "
        "Window, menu and localization conventions follow Luma.",
        "Сделано на Rust, Qt 6, Kirigami и rustic (репозитории формата restic). "
        "Конвенции окон, меню и локализации — как в Luma.",
    ),
    "about_key": ("Repository key", "Ключ репозитория"),
    "about_key_hint": (
        "The key decrypts every snapshot. It is stored in ~/.config/kustos/repo.password "
        "and is the only way to read the backups if this machine is lost — save it "
        "somewhere else, for example in a password manager.",
        "Ключ расшифровывает все снимки. Он хранится в ~/.config/kustos/repo.password и "
        "является единственным способом прочитать копии, если этот компьютер потеряется — "
        "сохраните его отдельно, например в менеджере паролей.",
    ),
    "action_show": ("Show", "Показать"),
    "action_hide": ("Hide", "Скрыть"),
    "action_copy_key": ("Copy Key", "Скопировать ключ"),
    "about_license": ("License: GPL-3.0-or-later", "Лицензия: GPL-3.0-or-later"),
    "action_close": ("Close", "Закрыть"),

    # ---- C++ messages (context "kustos", Loc::t) ----
    "error_no_target": ("The backup folder is not configured", "Не задана папка резервных копий"),
    "error_target_missing": ("Cannot create %1", "Не удаётся создать %1"),
    "error_target_not_writable": ("No write access to %1", "Нет прав на запись в %1"),
    "error_no_sources": (
        "None of the configured sources exist",
        "Ни один из заданных источников не существует",
    ),
    "error_target_inside_source": (
        "The backup folder is inside the source %1",
        "Папка копий находится внутри источника %1",
    ),
    "error_no_space": ("Not enough free space: %1", "Недостаточно места: %1"),
    "error_backup_running": ("A backup is already running", "Копирование уже выполняется"),
    "error_rsync_missing": ("rsync is not installed or not in PATH", "rsync не найден в PATH"),
    "warning_source_missing": ("Skipped missing source: %1", "Пропущен отсутствующий источник: %1"),
    "warning_partial_transfer": (
        "Some files could not be copied (see the log)",
        "Часть файлов не скопирована (см. журнал)",
    ),
    "notify_backup_done": ("Backup finished", "Резервное копирование завершено"),
    "notify_backup_done_body": ("%1 added in %2", "Добавлено %1 за %2"),
    "notify_backup_failed": ("Backup failed", "Ошибка резервного копирования"),
    "notify_backup_cancelled": ("Backup cancelled", "Копирование отменено"),
    "error_snapshot_missing": ("Snapshot not found", "Снимок не найден"),
    "error_invalid_path": ("Invalid path: %1", "Недопустимый путь: %1"),
    "error_restore_failed": (
        "Could not restore %1 (%2)",
        "Не удалось восстановить %1 (%2)",
    ),
    "restore_suffix": ("restored", "восстановлено"),
    "notify_restore_done": ("Restore finished", "Восстановление завершено"),
    "notify_restore_done_body": ("%1 item(s) restored", "Восстановлено элементов: %1"),
}

QSTR_RE = re.compile(r'qsTr\("([A-Za-z0-9_]+)"\)')
LOC_RE = re.compile(r'Loc::t\("([A-Za-z0-9_]+)"\)')


def qml_contexts() -> dict[str, list[str]]:
    contexts: dict[str, list[str]] = {}
    for path in sorted((ROOT / "qml").glob("*.qml")):
        keys = sorted(set(QSTR_RE.findall(path.read_text(encoding="utf-8"))))
        if keys:
            contexts[path.stem] = keys
    return contexts


def cpp_keys() -> list[str]:
    """Keys used outside QML. The Rust code keeps user-visible strings in QML
    only, so this stays empty; it is kept for future non-QML messages."""
    return []


def render(language: str, index: int) -> str:
    contexts = qml_contexts()
    cpp = cpp_keys()
    if cpp:
        contexts["kustos"] = cpp

    missing = sorted(
        key
        for keys in contexts.values()
        for key in keys
        if key not in TRANSLATIONS
    )
    if missing:
        print("Missing translations for: " + ", ".join(missing), file=sys.stderr)
        raise SystemExit(1)

    lines = ['<?xml version="1.0" encoding="utf-8"?>', "<!DOCTYPE TS>",
             f'<TS version="2.1" language="{language}">']
    for context, keys in contexts.items():
        lines.append("<context>")
        lines.append(f"<name>{escape(context)}</name>")
        for key in keys:
            text = TRANSLATIONS[key][index]
            lines.append(
                f"<message><source>{escape(key)}</source>"
                f"<translation>{escape(text)}</translation></message>"
            )
        lines.append("</context>")
    lines.append("</TS>")
    return "\n".join(lines) + "\n"


def main() -> int:
    (ROOT / "locales").mkdir(exist_ok=True)
    (ROOT / "locales/en.ts").write_text(render("en_US", 0), encoding="utf-8")
    (ROOT / "locales/ru.ts").write_text(render("ru_RU", 1), encoding="utf-8")
    total = sum(len(keys) for keys in qml_contexts().values()) + len(cpp_keys())
    print(f"wrote locales/en.ts and locales/ru.ts ({total} messages)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
