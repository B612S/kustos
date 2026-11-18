<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE TS>
<TS version="2.1" language="ru_RU">
<context>
<name>AboutDialog</name>
<message><source>about_config</source><translation>Настройки</translation></message>
<message><source>about_credits</source><translation>Сделано на Rust, Qt 6, Kirigami и rustic (репозитории формата restic). Конвенции окон, меню и локализации — как в Luma.</translation></message>
<message><source>about_description</source><translation>Снимки домашней папки в духе Time Machine. Они лежат в зашифрованном, сжатом репозитории с дедупликацией (формат restic), поэтому неизменённые данные не занимают лишнего места.</translation></message>
<message><source>about_key</source><translation>Ключ репозитория</translation></message>
<message><source>about_key_hint</source><translation>Ключ расшифровывает все снимки. Он хранится в ~/.config/kustos/repo.password и является единственным способом прочитать копии, если этот компьютер потеряется — сохраните его отдельно, например в менеджере паролей.</translation></message>
<message><source>about_license</source><translation>Лицензия: GPL-3.0-or-later</translation></message>
<message><source>about_log</source><translation>Журнал</translation></message>
<message><source>about_repo</source><translation>Репозиторий</translation></message>
<message><source>about_title</source><translation>О программе Kustos</translation></message>
<message><source>about_version</source><translation>Версия %1</translation></message>
<message><source>action_close</source><translation>Закрыть</translation></message>
<message><source>action_copy_key</source><translation>Скопировать ключ</translation></message>
<message><source>action_hide</source><translation>Скрыть</translation></message>
<message><source>action_show</source><translation>Показать</translation></message>
</context>
<context>
<name>BackupDialog</name>
<message><source>action_back_up_now</source><translation>Создать резервную копию</translation></message>
<message><source>action_cancel</source><translation>Отмена</translation></message>
<message><source>backup_add_exclude</source><translation>Исключить папку…</translation></message>
<message><source>backup_add_file</source><translation>Добавить файл…</translation></message>
<message><source>backup_add_folder</source><translation>Добавить папку…</translation></message>
<message><source>backup_exclude_hint</source><translation>Ничего внутри этих путей не копируется, даже если они лежат внутри копируемой папки.</translation></message>
<message><source>backup_exclude_title</source><translation>Исключения</translation></message>
<message><source>backup_include_hint</source><translation>Все перечисленные папки и файлы попадают в снимок.</translation></message>
<message><source>backup_include_title</source><translation>Какие папки копировать</translation></message>
<message><source>backup_no_sources</source><translation>Добавьте хотя бы одну папку или файл для копирования.</translation></message>
<message><source>backup_title</source><translation>Что копировать</translation></message>
</context>
<context>
<name>Main</name>
<message><source>action_about</source><translation>О программе</translation></message>
<message><source>action_back_up_now</source><translation>Создать резервную копию</translation></message>
<message><source>action_cancel</source><translation>Отмена</translation></message>
<message><source>action_delete</source><translation>Удалить</translation></message>
<message><source>action_open_backup_folder</source><translation>Открыть папку копий</translation></message>
<message><source>action_quit</source><translation>Выход</translation></message>
<message><source>action_restore</source><translation>Восстановить…</translation></message>
<message><source>action_settings</source><translation>Настройки</translation></message>
<message><source>app_title</source><translation>Kustos</translation></message>
<message><source>day_today</source><translation>Сегодня</translation></message>
<message><source>day_yesterday</source><translation>Вчера</translation></message>
<message><source>delete_snapshot_message</source><translation>Удалить снимок %1? Файлы, которые есть только в нём, восстановить не получится.</translation></message>
<message><source>delete_snapshot_title</source><translation>Удалить снимок</translation></message>
<message><source>empty_hint</source><translation>Kustos хранит историю домашней папки, как Time Machine. Каждый снимок занимает место только за изменения с предыдущего.</translation></message>
<message><source>empty_title</source><translation>Резервных копий пока нет</translation></message>
<message><source>status_cancelling</source><translation>Отмена…</translation></message>
<message><source>status_copying</source><translation>Копирование… %1, осталось %2</translation></message>
<message><source>status_preparing</source><translation>Подготовка списка файлов…</translation></message>
<message><source>status_restoring</source><translation>Восстановление %1</translation></message>
<message><source>status_summary</source><translation>Снимков: %1 · %2 · свободно %3</translation></message>
<message><source>status_working</source><translation>Выполняется…</translation></message>
<message><source>toast_backup_done</source><translation>Резервное копирование завершено</translation></message>
<message><source>toast_backup_failed</source><translation>Ошибка резервного копирования: %1</translation></message>
<message><source>toast_restore_done</source><translation>Восстановлено элементов: %1</translation></message>
<message><source>toast_restore_failed</source><translation>Ошибка восстановления: %1</translation></message>
</context>
<context>
<name>RestoreDialog</name>
<message><source>action_cancel</source><translation>Отмена</translation></message>
<message><source>action_restore</source><translation>Восстановить…</translation></message>
<message><source>restore_choose_folder</source><translation>Выбрать папку</translation></message>
<message><source>restore_conflict</source><translation>Если файл уже существует</translation></message>
<message><source>restore_count</source><translation>Элементов к восстановлению: %1</translation></message>
<message><source>restore_destination</source><translation>Куда восстанавливать</translation></message>
<message><source>restore_folder</source><translation>В другую папку…</translation></message>
<message><source>restore_into</source><translation>Папка назначения</translation></message>
<message><source>restore_more</source><translation>и ещё %1</translation></message>
<message><source>restore_original</source><translation>Исходное место</translation></message>
<message><source>restore_policy_keep_both</source><translation>Сохранить оба</translation></message>
<message><source>restore_policy_overwrite</source><translation>Заменить</translation></message>
<message><source>restore_policy_skip</source><translation>Оставить существующий</translation></message>
<message><source>restore_title</source><translation>Восстановление из снимка</translation></message>
</context>
<context>
<name>SettingsDialog</name>
<message><source>action_close</source><translation>Закрыть</translation></message>
<message><source>language_auto</source><translation>Системный</translation></message>
<message><source>schedule_every_hours</source><translation>Каждые %1 ч</translation></message>
<message><source>schedule_every_minutes</source><translation>Каждые %1 мин</translation></message>
<message><source>schedule_state_absent</source><translation>не установлен</translation></message>
<message><source>schedule_state_active</source><translation>работает</translation></message>
<message><source>schedule_state_inactive</source><translation>установлен, не запущен</translation></message>
<message><source>schedule_state_unknown</source><translation>неизвестно</translation></message>
<message><source>settings_add_excluded_folder</source><translation>Исключить папку…</translation></message>
<message><source>settings_add_file</source><translation>Добавить файл…</translation></message>
<message><source>settings_add_folder</source><translation>Добавить папку…</translation></message>
<message><source>settings_choose_folder</source><translation>Выбрать папку</translation></message>
<message><source>settings_general</source><translation>Общие</translation></message>
<message><source>settings_keep_daily</source><translation>Хранить ежедневные (дней)</translation></message>
<message><source>settings_keep_hourly</source><translation>Хранить почасовые (часов)</translation></message>
<message><source>settings_keep_weekly</source><translation>Хранить еженедельные (недель)</translation></message>
<message><source>settings_language</source><translation>Язык</translation></message>
<message><source>settings_nothing_to_prune</source><translation>Нечего удалять</translation></message>
<message><source>settings_notifications</source><translation>Уведомлять о завершении</translation></message>
<message><source>settings_open_config</source><translation>Открыть файл настроек</translation></message>
<message><source>settings_prune_now</source><translation>Проредить сейчас</translation></message>
<message><source>settings_pruned_now</source><translation>Политика хранения применена</translation></message>
<message><source>settings_repo</source><translation>Папка репозитория</translation></message>
<message><source>settings_reset</source><translation>Сбросить настройки</translation></message>
<message><source>settings_retention</source><translation>Хранение</translation></message>
<message><source>settings_retention_hint</source><translation>Старые снимки прореживаются как в Time Machine: всё за последние часы, затем по одному в день, затем по одному в неделю.</translation></message>
<message><source>settings_schedule</source><translation>Расписание</translation></message>
<message><source>settings_schedule_enabled</source><translation>Автоматические копии</translation></message>
<message><source>settings_schedule_interval</source><translation>Интервал</translation></message>
<message><source>settings_schedule_state</source><translation>Таймер: %1</translation></message>
<message><source>settings_scope</source><translation>Что копировать…</translation></message>
<message><source>settings_title</source><translation>Настройки</translation></message>
</context>
<context>
<name>SnapshotBrowser</name>
<message><source>action_restore</source><translation>Восстановить…</translation></message>
<message><source>browser_clear_selection</source><translation>Снять выделение</translation></message>
<message><source>browser_empty</source><translation>Папка пуста</translation></message>
<message><source>browser_empty_hint</source><translation>Выберите другой снимок или поднимитесь выше.</translation></message>
<message><source>browser_home</source><translation>Домой</translation></message>
<message><source>browser_modified</source><translation>Изменён</translation></message>
<message><source>browser_name</source><translation>Имя</translation></message>
<message><source>browser_select_all</source><translation>Выбрать всё</translation></message>
<message><source>browser_selected</source><translation>Выбрано: %1</translation></message>
<message><source>browser_size</source><translation>Размер</translation></message>
<message><source>browser_snapshot</source><translation>Снимок %1, %2</translation></message>
<message><source>browser_up</source><translation>Вверх</translation></message>
<message><source>menu_copy_path</source><translation>Скопировать путь</translation></message>
<message><source>menu_restore_item</source><translation>Восстановить этот элемент…</translation></message>
<message><source>snapshot_latest</source><translation>Последняя</translation></message>
</context>
<context>
<name>TimelinePanel</name>
<message><source>menu_copy_id</source><translation>Скопировать ID снимка</translation></message>
<message><source>menu_delete_snapshot</source><translation>Удалить снимок…</translation></message>
<message><source>snapshot_added</source><translation>добавлено %1</translation></message>
<message><source>snapshot_latest</source><translation>Последняя</translation></message>
<message><source>timeline_count</source><translation>Снимков: %1</translation></message>
<message><source>timeline_empty</source><translation>Таймлайн появится после первой резервной копии.</translation></message>
<message><source>timeline_hint</source><translation>Сверху самые свежие. Выберите снимок, чтобы просмотреть его и восстановить файлы.</translation></message>
<message><source>timeline_title</source><translation>Резервные копии</translation></message>
</context>
</TS>
