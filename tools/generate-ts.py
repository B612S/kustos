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
    "action_save": ("Save", "Сохранить"),
    "action_delete": ("Delete", "Удалить"),
    "action_settings": ("Settings", "Настройки"),
    "action_about": ("About Kustos", "О программе"),
    "action_quit": ("Quit", "Выход"),
    "action_open_backup_folder": ("Open Backup Folder", "Открыть папку копий"),
    "action_backup_rules": ("Edit Rule…", "Настроить правило…"),
    "action_save_copies": ("Save Copies…", "Сохранить копии…"),
    "action_stop_backup": ("Stop Backup", "Остановить копирование"),
    "action_set_password": ("Set Password…", "Задать пароль…"),
    "action_change_password": ("Change Password…", "Изменить пароль…"),
    "action_lock_backups": ("Lock Backups", "Заблокировать копии"),
    "action_lock_now": ("Lock Now", "Заблокировать сейчас"),
    "status_repo_missing": (
        "Repository not found at %1 — is the drive mounted?",
        "Репозиторий не найден: %1 — диск смонтирован?",
    ),
    "status_locked": (
        "Snapshots are locked · open a backup to enter its password",
        "Снимки заблокированы · откройте копию, чтобы ввести пароль",
    ),
    "toast_locked": ("Snapshots locked", "Снимки заблокированы"),
    "toast_credential_failed": (
        "The password was saved, but the systemd credential could not be created: %1",
        "Пароль сохранён, но systemd-credential создать не удалось: %1",
    ),
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
    "status_copying_rule": ("Copying %1… %2, %3 left", "Копирование «%1»… %2, осталось %3"),
    "status_preparing": ("Preparing the file list…", "Подготовка списка файлов…"),
    "status_restoring": ("Restoring %1", "Восстановление %1"),
    "status_saving": ("Saving %1", "Сохранение %1"),
    "status_opening": ("Opening %1…", "Открытие %1…"),
    "status_summary": ("%1 snapshot(s) · %2 · %3 free", "Снимков: %1 · %2 · свободно %3"),
    "status_working": ("Working…", "Выполняется…"),
    "toast_backup_done": ("Backup finished", "Резервное копирование завершено"),
    "toast_backup_done_locked": (
        "Backup finished — unlock the backup to see the new snapshot",
        "Копия создана — разблокируйте её, чтобы увидеть новый снимок",
    ),
    "toast_backup_failed": ("Backup failed: %1", "Ошибка резервного копирования: %1"),
    "toast_backup_running": (
        "Another backup or restore is already running",
        "Уже выполняется другое копирование или восстановление",
    ),
    "toast_restore_done": ("Restored %1 item(s)", "Восстановлено элементов: %1"),
    "toast_restore_failed": ("Restore failed: %1", "Ошибка восстановления: %1"),
    "toast_export_done": ("Saved %1 item(s) to the chosen folder", "Сохранено элементов: %1"),
    "toast_export_failed": ("Could not save the copies: %1", "Не удалось сохранить копии: %1"),
    "toast_open_failed": ("Could not open the file: %1", "Не удалось открыть файл: %1"),
    "toast_backup_warnings": ("Backup finished with warnings: %1", "Копия создана с замечаниями: %1"),
    "toast_load_failed": (
        "Could not load the backup list: %1",
        "Не удалось загрузить список копий: %1",
    ),

    # ---- timeline (TimelinePanel.qml) ----
    "timeline_title": ("Snapshots", "Снимки"),
    "timeline_rule_title": ("Snapshots: %1", "Снимки: %1"),
    "timeline_empty_rule": (
        "No snapshots for this rule yet.",
        "У этого правила пока нет снимков.",
    ),
    "timeline_loading": ("Loading backups…", "Загрузка копий…"),
    "timeline_hint": (
        "Newest first. Pick a snapshot to browse its files.",
        "Сверху свежие. Выберите снимок, чтобы открыть файлы.",
    ),
    "timeline_count": ("%1 snapshot(s)", "Снимков: %1"),
    "timeline_empty": (
        "The timeline appears here after the first backup.",
        "Таймлайн появится после первой резервной копии.",
    ),
    "day_today": ("Today", "Сегодня"),
    "day_yesterday": ("Yesterday", "Вчера"),
    "snapshot_on_disk": ("%1 on disk", "%1 на диске"),
    "snapshot_changes_tooltip": (
        "Files: %1 · %2 new, %3 changed · %4",
        "Файлов: %1 · %2 новых, %3 изменённых · %4",
    ),
    "snapshot_added_tooltip": (
        "New data before compression: %1. Written to the repository: %2.",
        "Новых данных до сжатия: %1. Записано в репозиторий: %2.",
    ),
    "snapshot_latest": ("Latest", "Последняя"),
    "snapshot_status_failed": ("Failed", "Ошибка"),
    "menu_copy_id": ("Copy Snapshot ID", "Скопировать ID снимка"),
    "menu_pin_snapshot": ("Pin Snapshot", "Закрепить снимок"),
    "menu_unpin_snapshot": ("Unpin Snapshot", "Открепить снимок"),
    "menu_delete_snapshot": ("Delete Snapshot…", "Удалить снимок…"),

    # ---- browser (SnapshotBrowser.qml) ----
    "browser_home": ("Home", "Домой"),
    "browser_loading": ("Loading snapshot…", "Загрузка снимка…"),
    "browser_load_failed": (
        "Could not read the snapshot: %1",
        "Не удалось прочитать снимок: %1",
    ),
    "browser_snapshot": ("Snapshot %1, %2", "Снимок %1, %2"),
    "browser_name": ("Name", "Имя"),
    "browser_modified": ("Modified", "Изменён"),
    "browser_size": ("Size", "Размер"),
    "browser_up": ("Up", "Вверх"),
    "browser_search": ("Search (Ctrl+F)", "Поиск (Ctrl+F)"),
    "browser_search_placeholder": ("Search this snapshot…", "Поиск в снимке…"),
    "browser_search_hint": ("Type at least 2 characters", "Введите хотя бы 2 символа"),
    "browser_search_count": ("%1 match(es)", "Найдено: %1"),
    "browser_search_none": ("Nothing found", "Ничего не найдено"),
    "browser_search_clear": ("Close search", "Закрыть поиск"),
    "browser_selected": ("Selected: %1", "Выбрано: %1"),
    "browser_select_all": ("Select All", "Выбрать всё"),
    "browser_clear_selection": ("Clear Selection", "Снять выделение"),
    "browser_empty": ("This folder is empty", "Папка пуста"),
    "browser_no_snapshot": ("No snapshot selected", "Снимок не выбран"),
    "browser_no_snapshot_hint": (
        "Pick a snapshot in the middle column to browse its files.",
        "Выберите снимок в средней колонке, чтобы просмотреть его файлы.",
    ),
    "browser_empty_hint": (
        "Pick another snapshot or go up a level.",
        "Выберите другой снимок или поднимитесь выше.",
    ),
    "browser_choose_export_folder": ("Save Copies to Folder", "Куда сохранить копии"),
    "browser_save_hint": (
        "Copy the selected items directly into a folder of your choice",
        "Скопировать выбранное прямо в выбранную папку",
    ),
    "menu_save_item": ("Save This Item…", "Сохранить этот элемент…"),
    "menu_open_item": ("Open", "Открыть"),
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

    # ---- per-backup passwords (UnlockDialog.qml, TimelinePanel.qml) ----
    "password_create_title": ("Viewing Password", "Пароль для просмотра"),
    "password_common_create_title": ("Common Backup Password", "Общий пароль для копий"),
    "password_common_change_title": ("Change Common Password", "Изменить общий пароль"),
    "password_common_unlock_title": ("Unlock All Backups", "Разблокировать все копии"),
    "password_common_create_hint": (
        "One password for all backups. It is asked when any backup is opened; "
        "backups themselves use the saved credential and never ask. There is "
        "no recovery if you forget it.",
        "Один пароль для всех копий. Он запрашивается при открытии любой копии; "
        "сами копии используют сохранённый credential и пароль не спрашивают. "
        "Восстановить забытый пароль невозможно.",
    ),
    "password_common_change_hint": (
        "Choose a new common password for all backups. A new key is created and "
        "every rule switches to it.",
        "Задайте новый общий пароль для всех копий. Создаётся новый ключ, и все "
        "правила переключаются на него.",
    ),
    "password_common_unlock_hint": (
        "Enter the common password. It unlocks every backup for viewing.",
        "Введите общий пароль. Он открывает все копии для просмотра.",
    ),
    "password_for_all_backups": ("All backups", "Все резервные копии"),
    "password_change_title": ("Change Viewing Password", "Изменить пароль для просмотра"),
    "password_unlock_title": ("Unlock Backup", "Разблокировать резервную копию"),
    "password_for_rule": ("Backup “%1”", "Резервная копия «%1»"),
    "password_for_backup": ("This backup", "Эта резервная копия"),
    "password_create_hint": (
        "Give this backup its own viewing password. It is asked when the backup "
        "is opened; backups themselves use the saved credential and never ask "
        "for it. There is no recovery if you forget it.",
        "Задайте пароль для просмотра этой копии. Он запрашивается при её "
        "открытии; сами копии используют сохранённый credential и пароль не "
        "спрашивают. Восстановить забытый пароль невозможно.",
    ),
    "password_create_auth_hint": (
        "Enter the password of an existing backup, then choose a new password for this one.",
        "Введите пароль существующей копии, затем задайте новый пароль для этой.",
    ),
    "password_change_hint": (
        "Choose a new password for this backup.",
        "Задайте новый пароль для этой резервной копии.",
    ),
    "password_unlock_hint": (
        "Enter the password of this backup. It is kept in memory only and is "
        "forgotten after the inactivity timeout.",
        "Введите пароль этой резервной копии. Он хранится только в памяти и "
        "забывается после периода неактивности.",
    ),
    "password_current_placeholder": ("Current password", "Текущий пароль"),
    "password_existing_placeholder": ("Existing backup password", "Пароль существующей копии"),
    "password_new_placeholder": ("New password", "Новый пароль"),
    "password_placeholder": ("Password", "Пароль"),
    "password_confirm_placeholder": ("Repeat the password", "Повторите пароль"),
    "password_show": ("Show password", "Показать пароль"),
    "password_mismatch": ("The passwords do not match.", "Пароли не совпадают."),
    "password_wrong": ("Wrong password", "Неверный пароль"),
    "rule_password_title": ("Password", "Пароль"),
    "rule_password_set": ("A viewing password is set", "Пароль для просмотра задан"),
    "rule_password_common_set": ("Uses the common password", "Используется общий пароль"),
    "rule_password_common_missing": (
        "Common password not set (Settings → Security)",
        "Общий пароль не задан (Настройки → Безопасность)",
    ),
    "rule_password_common_hint": (
        "All rules use one common password. It is managed in Settings → Security.",
        "Все правила используют один общий пароль. Он настраивается в «Настройки → Безопасность».",
    ),
    "rule_password_missing": (
        "No viewing password — opening this backup will ask you to create one. "
        "Backups themselves do not ask.",
        "Пароль для просмотра не задан — при открытии копии его нужно будет "
        "создать. Для самих копий пароль не нужен.",
    ),
    "rule_password_hint": (
        "The viewing password is asked when this backup is opened and is kept "
        "in memory only. Backups use the saved systemd credential, which is "
        "created after the first backup or when you set this password.",
        "Пароль для просмотра запрашивается при открытии этой копии и хранится "
        "только в памяти. Сами копии используют сохранённый systemd-credential; "
        "он создаётся после первой копии или при задании этого пароля.",
    ),
    "rule_password_unattended_ready": (
        "Automatic backups can run without the window (systemd credential ready).",
        "Автоматические копии могут запускаться без окна (systemd-credential готов).",
    ),
    "rule_password_unattended_missing": (
        "No saved credential yet — run a backup once (or set a viewing password) to create it.",
        "Сохранённого credential ещё нет — выполните одну копию (или задайте пароль для просмотра), чтобы создать его.",
    ),
    "rule_schedule_needs_password": (
        "Run a backup once first: the systemd service needs a saved credential.",
        "Сначала выполните одну копию: службе systemd нужен сохранённый credential.",
    ),
    "rule_locked": ("Locked", "Заблокировано"),
    "rule_no_password": ("No viewing password", "Пароль для просмотра не задан"),
    "menu_unlock_rule": ("Unlock Backup…", "Разблокировать копию…"),
    "timeline_locked_title": ("This backup is locked", "Копия заблокирована"),
    "timeline_locked_hint": (
        "Unlock it and enter this backup's password to see its snapshots.",
        "Нажмите «Разблокировать» и введите пароль этой копии, чтобы увидеть снимки.",
    ),
    "timeline_no_password_hint": (
        "This backup has no password yet. Set one to open it.",
        "У этой копии ещё нет пароля. Задайте его, чтобы открыть копию.",
    ),

    # ---- settings (SettingsDialog.qml) ----
    "settings_title": ("Settings", "Настройки"),
    "backup_title": ("Backup Rule", "Правило копирования"),
    "rules_title": ("What and when to back up", "Что и когда копировать"),
    "rules_hint": (
        "This rule defines what is copied, what is excluded and when it runs. "
        "Changes are saved immediately.",
        "Это правило определяет, что копируется, что исключается и когда запускаться. "
        "Изменения сохраняются сразу.",
    ),
    "rules_list_title": ("Rules", "Правила"),
    "rules_panel_title": ("Rules", "Правила"),
    "rules_panel_hint": (
        "Pick a rule to see its snapshots. Right-click for actions.",
        "Выберите правило, чтобы увидеть его снимки. Правый клик — действия.",
    ),
    "rules_count": ("Rules: %1", "Правил: %1"),
    "rules_other": ("Other Backups", "Другие копии"),
    "rules_other_hint": (
        "Snapshots without a matching rule",
        "Снимки без подходящего правила",
    ),
    "rules_empty_panel_hint": (
        "No rules yet. Add one to choose what is copied and when.",
        "Правил пока нет. Добавьте правило: что и когда копировать.",
    ),
    "menu_stop_rule": ("Stop Backup", "Остановить копирование"),
    "menu_edit_rule": ("Edit Rule…", "Настроить правило…"),
    "menu_pause_rule": ("Pause Rule", "Приостановить правило"),
    "menu_enable_rule": ("Enable Rule", "Включить правило"),
    "menu_delete_rule": ("Delete Rule…", "Удалить правило…"),
    "delete_rule_title": ("Delete Rule", "Удалить правило"),
    "delete_rule_message": (
        "Delete the rule “%1”? Its snapshots stay in the repository and appear under Other Backups.",
        "Удалить правило «%1»? Его снимки останутся в репозитории и появятся в «Других копиях».",
    ),
    "rules_scheduled": ("Scheduled: %1 rule(s)", "По расписанию: %1"),
    "rules_empty": ("No backup rules", "Нет правил копирования"),
    "rules_empty_hint": (
        "Add a rule to choose what is copied, what is skipped and when it runs.",
        "Добавьте правило: что копировать, что пропускать и когда запускать.",
    ),
    "rule_add": ("New Rule", "Новое правило"),
    "rule_new_name": ("New rule", "Новое правило"),
    "rule_untitled": ("Untitled rule", "Без названия"),
    "rule_paused": ("paused", "на паузе"),
    "rule_last_backup": ("last %1", "последний %1"),
    "rule_remove": ("Remove Rule", "Удалить правило"),
    "rule_remove_source": ("Remove folder", "Убрать папку"),
    "rule_remove_exclude": ("Remove exclusion", "Убрать исключение"),
    "rule_name_placeholder": ("Rule name", "Название правила"),
    "rule_enabled": ("Enabled", "Включено"),
    "rule_automatic": ("Automatic", "Автоматически"),
    "rule_schedule_title": ("Schedule", "Расписание"),
    "rule_schedule_hint": (
        "Installed as a systemd user service and timer. The saved credential — "
        "the viewing password or the generated backup key — is passed to the "
        "service, so the window does not have to be open.",
        "Устанавливается как служба и таймер systemd. Службе передаётся "
        "сохранённый credential — пароль для просмотра или сгенерированный ключ "
        "копии, — поэтому окно приложения не нужно.",
    ),
    "rule_schedule_off": ("No schedule", "Без расписания"),
    "rule_mode_interval": ("Every N minutes", "Каждые N минут"),
    "rule_mode_daily": ("Daily", "Ежедневно"),
    "rule_mode_weekly": ("Certain days", "По дням недели"),
    "rule_minutes_value": ("%1 min", "%1 мин"),
    "rule_daily_at": ("Daily at %1", "Ежедневно в %1"),
    "rule_weekly_at": ("Weekly on %1 at %2", "%1 в %2"),
    "rule_every_day": ("every day", "каждый день"),
    "weekday_short_mon": ("Mon", "Пн"),
    "weekday_short_tue": ("Tue", "Вт"),
    "weekday_short_wed": ("Wed", "Ср"),
    "weekday_short_thu": ("Thu", "Чт"),
    "weekday_short_fri": ("Fri", "Пт"),
    "weekday_short_sat": ("Sat", "Сб"),
    "weekday_short_sun": ("Sun", "Вс"),
    "rule_time_placeholder": ("HH:MM", "ЧЧ:ММ"),
    "rule_time_hint": ("24-hour, HH:MM", "24 часа, ЧЧ:ММ"),
    "rule_advanced": ("Advanced", "Дополнительно"),
    "rule_one_filesystem": (
        "Stay on one filesystem",
        "Не переходить на другие файловые системы",
    ),
    "rule_one_filesystem_hint": (
        "Skip mounted disks inside a source instead of following them.",
        "Пропускать смонтированные диски внутри источника.",
    ),
    "rule_exclude_larger": ("Skip files larger than", "Пропускать файлы больше"),
    "rule_exclude_larger_placeholder": ("e.g. 2G", "например 2G"),
    "rule_exclude_larger_hint": (
        "Rustic size suffix: K, M, G, T. Empty means no limit.",
        "Суффиксы rustic: K, M, G, T. Пусто — без ограничения.",
    ),
    "rule_no_excludes": ("Nothing is excluded", "Исключений нет"),
    "backup_include_title": ("What to copy", "Что копировать"),
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
    "backup_add_recommended": ("Recommended", "Рекомендуемые"),
    "backup_recommended_hint": (
        "Adds caches, the KDE file index (baloo) and other rebuildable data",
        "Добавляет кэши, индекс KDE (baloo) и другие пересоздаваемые данные",
    ),
    "backup_add_exclude": ("Exclude Folder…", "Исключить папку…"),
    "backup_add_pattern": ("Add Pattern", "Добавить шаблон"),
    "backup_pattern_placeholder": (
        "Pattern, e.g. *.log, node_modules or ~/.cache/**",
        "Шаблон, например *.log, node_modules или ~/.cache/**",
    ),
    "backup_no_sources": (
        "Add at least one folder or file to back up.",
        "Добавьте хотя бы одну папку или файл для копирования.",
    ),
    "settings_scope": ("Backup Scope…", "Что копировать…"),
    "settings_rules": ("Edit Rule…", "Настроить правило…"),
    "settings_general": ("General", "Общие"),
    "settings_security": ("Security", "Безопасность"),
    "settings_password_mode": ("Backup passwords", "Пароли резервных копий"),
    "password_mode_per_rule": ("Each rule has its own password", "У каждого правила свой пароль"),
    "password_mode_common": ("One common password", "Один общий пароль"),
    "settings_common_password": ("Common password", "Общий пароль"),
    "settings_recovery_title": ("Recovery Key", "Ключ восстановления"),
    "settings_recovery_hint": (
        "A saved key lets backups run without asking for a password. Keep a copy "
        "somewhere safe: without a key or a viewing password the repository "
        "cannot be opened.",
        "Сохранённый ключ позволяет копиям работать без запроса пароля. Сохраните "
        "его в надёжном месте: без ключа или пароля для просмотра репозиторий "
        "не открыть.",
    ),
    "settings_recovery_show": ("Show Key", "Показать ключ"),
    "settings_recovery_hide": ("Hide Key", "Скрыть ключ"),
    "settings_recovery_warning": (
        "Anyone who sees this key can read every backup.",
        "Тот, кто увидит этот ключ, сможет прочитать все копии.",
    ),
    "settings_recovery_missing": (
        "No saved key was found. If you set a viewing password, use it instead.",
        "Сохранённый ключ не найден. Если вы задавали пароль для просмотра, используйте его.",
    ),
    "settings_common_password_set": ("Set", "Задан"),
    "settings_common_password_missing": ("Not set", "Не задан"),
    "settings_password_mode_hint": (
        "Choose how viewing passwords work. Backups themselves always use the "
        "saved credential and never ask.",
        "Выберите, как устроены пароли для просмотра. Сами копии всегда "
        "используют сохранённый credential и пароль не спрашивают.",
    ),
    "settings_common_password_hint": (
        "One password opens every backup. It is kept in memory only and "
        "forgotten after the inactivity timeout. Without it, opening a backup "
        "asks you to set it.",
        "Один пароль открывает все копии. Он хранится только в памяти и "
        "забывается после периода неактивности. Пока он не задан, при открытии "
        "копии будет предложено его создать.",
    ),
    "settings_sources": ("Sources", "Источники"),
    "settings_schedule": ("Schedule", "Расписание"),
    "settings_retention": ("Retention", "Хранение"),
    "settings_maintenance": ("Maintenance", "Обслуживание"),
    "settings_check_title": ("Repository Check", "Проверка репозитория"),
    "settings_check_hint": (
        "Verifies the repository structure and index. The data check also reads "
        "a random 10% of the pack files and can take several minutes.",
        "Проверяет структуру и индекс репозитория. Проверка с данными "
        "дополнительно читает случайные 10% pack-файлов и может занять несколько минут.",
    ),
    "settings_check_now": ("Check Now", "Проверить сейчас"),
    "settings_check_data": ("Check with Data (10%)", "Проверить с данными (10%)"),
    "settings_check_running": ("Checking the repository…", "Проверка репозитория…"),
    "settings_check_no_repo": ("The repository does not exist yet.", "Репозитория ещё нет."),
    "settings_check_never": (
        "The repository has not been checked yet.",
        "Репозиторий ещё не проверялся.",
    ),
    "settings_check_stale": (
        "Last check: %1 · more than 30 days ago",
        "Последняя проверка: %1 · больше 30 дней назад",
    ),
    "settings_check_ok": (
        "Last check: %1 · passed",
        "Последняя проверка: %1 · успешно",
    ),
    "settings_check_failed": (
        "Last check: %1 · errors found",
        "Последняя проверка: %1 · найдены ошибки",
    ),
    "settings_repo": ("Repository folder", "Папка репозитория"),
    "settings_choose_folder": ("Choose Folder", "Выбрать папку"),
    "settings_language": ("Language", "Язык"),
    "language_auto": ("System", "Системный"),
    "settings_notifications": ("Notify when a backup finishes", "Уведомлять о завершении"),
    "settings_lock_timeout": ("Lock after inactivity", "Блокировать после неактивности"),
    "settings_lock_hint": (
        "Passwords are kept in memory only. After this time without activity in "
        "the window they are forgotten again; “Never” keeps them until you quit "
        "or lock them yourself.",
        "Пароли хранятся только в памяти. Через указанное время без действий в "
        "окне они забываются; «Никогда» оставляет их до выхода или ручной блокировки.",
    ),
    "lock_never": ("Never", "Никогда"),
    "lock_minutes": ("%1 min", "%1 мин"),
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
    "settings_prune_title": ("Delete %1 snapshot(s)?", "Удалить снимков: %1?"),
    "settings_prune_message": (
        "These snapshots are outside the retention policy and will be deleted. "
        "Pinned snapshots are kept.",
        "Эти снимки выходят за пределы политики хранения и будут удалены. "
        "Закреплённые снимки сохраняются.",
    ),
    "settings_prune_confirm": ("Delete", "Удалить"),
    "settings_prune_failed": ("Could not prune the repository (see the log)", "Не удалось проредить репозиторий (см. журнал)"),
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
    "about_key": ("Backup passwords", "Пароли резервных копий"),
    "action_unlock": ("Unlock", "Разблокировать"),
    "action_continue": ("Continue", "Продолжить"),
    "action_copy": ("Copy", "Копировать"),
    "about_key_hint": (
        "Every backup has its own password — a real repository key. Passwords are "
        "not written to the configuration: they are asked for when a backup is "
        "opened and forgotten after the inactivity timeout. Automatic backups use "
        "per-rule encrypted systemd credentials.",
        "У каждой копии свой пароль — настоящий ключ репозитория. Пароли не "
        "записываются в настройки: они запрашиваются при открытии копии и "
        "забываются после периода неактивности. Автоматические копии используют "
        "отдельные зашифрованные systemd-credentials.",
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
