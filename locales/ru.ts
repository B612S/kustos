<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE TS>
<TS version="2.1" language="ru_RU">
<context>
<name>AboutDialog</name>
<message><source>about_config</source><translation>Настройки</translation></message>
<message><source>about_credits</source><translation>Сделано на Rust, Qt 6, Kirigami и rustic (репозитории формата restic). Конвенции окон, меню и локализации — как в Luma.</translation></message>
<message><source>about_description</source><translation>Снимки домашней папки в духе Time Machine. Они лежат в зашифрованном, сжатом репозитории с дедупликацией (формат restic), поэтому неизменённые данные не занимают лишнего места.</translation></message>
<message><source>about_key</source><translation>Пароли резервных копий</translation></message>
<message><source>about_key_hint</source><translation>У каждой копии свой пароль — настоящий ключ репозитория. Пароли не записываются в настройки: они запрашиваются при открытии копии и забываются после периода неактивности. Автоматические копии используют отдельные зашифрованные systemd-credentials.</translation></message>
<message><source>about_license</source><translation>Лицензия: GPL-3.0-or-later</translation></message>
<message><source>about_log</source><translation>Журнал</translation></message>
<message><source>about_repo</source><translation>Репозиторий</translation></message>
<message><source>about_title</source><translation>О программе Kustos</translation></message>
<message><source>about_version</source><translation>Версия %1</translation></message>
<message><source>action_close</source><translation>Закрыть</translation></message>
</context>
<context>
<name>BackupDialog</name>
<message><source>action_cancel</source><translation>Отмена</translation></message>
<message><source>action_change_password</source><translation>Изменить пароль…</translation></message>
<message><source>action_save</source><translation>Сохранить</translation></message>
<message><source>action_set_password</source><translation>Задать пароль…</translation></message>
<message><source>backup_add_exclude</source><translation>Исключить папку…</translation></message>
<message><source>backup_add_file</source><translation>Добавить файл…</translation></message>
<message><source>backup_add_folder</source><translation>Добавить папку…</translation></message>
<message><source>backup_add_pattern</source><translation>Добавить шаблон</translation></message>
<message><source>backup_add_recommended</source><translation>Рекомендуемые</translation></message>
<message><source>backup_exclude_hint</source><translation>Ничего внутри этих путей не копируется, даже если они лежат внутри копируемой папки.</translation></message>
<message><source>backup_exclude_title</source><translation>Исключения</translation></message>
<message><source>backup_include_hint</source><translation>Все перечисленные папки и файлы попадают в снимок.</translation></message>
<message><source>backup_include_title</source><translation>Что копировать</translation></message>
<message><source>backup_no_sources</source><translation>Добавьте хотя бы одну папку или файл для копирования.</translation></message>
<message><source>backup_pattern_placeholder</source><translation>Шаблон, например *.log, node_modules или ~/.cache/**</translation></message>
<message><source>backup_recommended_hint</source><translation>Добавляет кэши, индекс KDE (baloo) и другие пересоздаваемые данные</translation></message>
<message><source>backup_title</source><translation>Правило копирования</translation></message>
<message><source>rule_advanced</source><translation>Дополнительно</translation></message>
<message><source>rule_automatic</source><translation>Автоматически</translation></message>
<message><source>rule_enabled</source><translation>Включено</translation></message>
<message><source>rule_exclude_larger</source><translation>Пропускать файлы больше</translation></message>
<message><source>rule_exclude_larger_hint</source><translation>Суффиксы rustic: K, M, G, T. Пусто — без ограничения.</translation></message>
<message><source>rule_exclude_larger_placeholder</source><translation>например 2G</translation></message>
<message><source>rule_minutes_value</source><translation>%1 мин</translation></message>
<message><source>rule_mode_daily</source><translation>Ежедневно</translation></message>
<message><source>rule_mode_interval</source><translation>Каждые N минут</translation></message>
<message><source>rule_mode_weekly</source><translation>По дням недели</translation></message>
<message><source>rule_name_placeholder</source><translation>Название правила</translation></message>
<message><source>rule_new_name</source><translation>Новое правило</translation></message>
<message><source>rule_no_excludes</source><translation>Исключений нет</translation></message>
<message><source>rule_one_filesystem</source><translation>Не переходить на другие файловые системы</translation></message>
<message><source>rule_one_filesystem_hint</source><translation>Пропускать смонтированные диски внутри источника.</translation></message>
<message><source>rule_password_common_hint</source><translation>Все правила используют один общий пароль. Он настраивается в «Настройки → Безопасность».</translation></message>
<message><source>rule_password_common_missing</source><translation>Общий пароль не задан (Настройки → Безопасность)</translation></message>
<message><source>rule_password_common_set</source><translation>Используется общий пароль</translation></message>
<message><source>rule_password_hint</source><translation>Пароль для просмотра запрашивается при открытии этой копии и хранится только в памяти. Сами копии используют сохранённый systemd-credential; он создаётся после первой копии или при задании этого пароля.</translation></message>
<message><source>rule_password_missing</source><translation>Пароль для просмотра не задан — при открытии копии его нужно будет создать. Для самих копий пароль не нужен.</translation></message>
<message><source>rule_password_set</source><translation>Пароль для просмотра задан</translation></message>
<message><source>rule_password_title</source><translation>Пароль</translation></message>
<message><source>rule_password_unattended_missing</source><translation>Сохранённого credential ещё нет — выполните одну копию (или задайте пароль для просмотра), чтобы создать его.</translation></message>
<message><source>rule_password_unattended_ready</source><translation>Автоматические копии могут запускаться без окна (systemd-credential готов).</translation></message>
<message><source>rule_remove_exclude</source><translation>Убрать исключение</translation></message>
<message><source>rule_remove_source</source><translation>Убрать папку</translation></message>
<message><source>rule_schedule_hint</source><translation>Устанавливается как служба и таймер systemd. Службе передаётся сохранённый credential — пароль для просмотра или сгенерированный ключ копии, — поэтому окно приложения не нужно.</translation></message>
<message><source>rule_schedule_needs_password</source><translation>Сначала выполните одну копию: службе systemd нужен сохранённый credential.</translation></message>
<message><source>rule_schedule_title</source><translation>Расписание</translation></message>
<message><source>rule_time_hint</source><translation>24 часа, ЧЧ:ММ</translation></message>
<message><source>rule_time_placeholder</source><translation>ЧЧ:ММ</translation></message>
<message><source>rules_hint</source><translation>Это правило определяет, что копируется, что исключается и когда запускаться. Изменения сохраняются сразу.</translation></message>
<message><source>rules_scheduled</source><translation>По расписанию: %1</translation></message>
<message><source>rules_title</source><translation>Что и когда копировать</translation></message>
<message><source>weekday_short_fri</source><translation>Пт</translation></message>
<message><source>weekday_short_mon</source><translation>Пн</translation></message>
<message><source>weekday_short_sat</source><translation>Сб</translation></message>
<message><source>weekday_short_sun</source><translation>Вс</translation></message>
<message><source>weekday_short_thu</source><translation>Чт</translation></message>
<message><source>weekday_short_tue</source><translation>Вт</translation></message>
<message><source>weekday_short_wed</source><translation>Ср</translation></message>
</context>
<context>
<name>Main</name>
<message><source>action_about</source><translation>О программе</translation></message>
<message><source>action_back_up_now</source><translation>Создать резервную копию</translation></message>
<message><source>action_backup_rules</source><translation>Настроить правило…</translation></message>
<message><source>action_cancel</source><translation>Отмена</translation></message>
<message><source>action_delete</source><translation>Удалить</translation></message>
<message><source>action_lock_backups</source><translation>Заблокировать копии</translation></message>
<message><source>action_open_backup_folder</source><translation>Открыть папку копий</translation></message>
<message><source>action_quit</source><translation>Выход</translation></message>
<message><source>action_restore</source><translation>Восстановить…</translation></message>
<message><source>action_save_copies</source><translation>Сохранить копии…</translation></message>
<message><source>action_settings</source><translation>Настройки</translation></message>
<message><source>app_title</source><translation>Kustos</translation></message>
<message><source>browser_load_failed</source><translation>Не удалось прочитать снимок: %1</translation></message>
<message><source>day_today</source><translation>Сегодня</translation></message>
<message><source>day_yesterday</source><translation>Вчера</translation></message>
<message><source>delete_rule_message</source><translation>Удалить правило «%1»? Его снимки останутся в репозитории и появятся в «Других копиях».</translation></message>
<message><source>delete_rule_title</source><translation>Удалить правило</translation></message>
<message><source>delete_snapshot_message</source><translation>Удалить снимок %1? Файлы, которые есть только в нём, восстановить не получится.</translation></message>
<message><source>delete_snapshot_title</source><translation>Удалить снимок</translation></message>
<message><source>rule_new_name</source><translation>Новое правило</translation></message>
<message><source>rule_untitled</source><translation>Без названия</translation></message>
<message><source>rules_empty</source><translation>Нет правил копирования</translation></message>
<message><source>rules_other</source><translation>Другие копии</translation></message>
<message><source>status_cancelling</source><translation>Отмена…</translation></message>
<message><source>status_copying</source><translation>Копирование… %1, осталось %2</translation></message>
<message><source>status_copying_rule</source><translation>Копирование «%1»… %2, осталось %3</translation></message>
<message><source>status_locked</source><translation>Снимки заблокированы · откройте копию, чтобы ввести пароль</translation></message>
<message><source>status_opening</source><translation>Открытие %1…</translation></message>
<message><source>status_preparing</source><translation>Подготовка списка файлов…</translation></message>
<message><source>status_repo_missing</source><translation>Репозиторий не найден: %1 — диск смонтирован?</translation></message>
<message><source>status_restoring</source><translation>Восстановление %1</translation></message>
<message><source>status_saving</source><translation>Сохранение %1</translation></message>
<message><source>status_summary</source><translation>Снимков: %1 · %2 · свободно %3</translation></message>
<message><source>status_working</source><translation>Выполняется…</translation></message>
<message><source>timeline_empty</source><translation>Таймлайн появится после первой резервной копии.</translation></message>
<message><source>timeline_empty_rule</source><translation>У этого правила пока нет снимков.</translation></message>
<message><source>toast_backup_done</source><translation>Резервное копирование завершено</translation></message>
<message><source>toast_backup_done_locked</source><translation>Копия создана — разблокируйте её, чтобы увидеть новый снимок</translation></message>
<message><source>toast_backup_failed</source><translation>Ошибка резервного копирования: %1</translation></message>
<message><source>toast_backup_running</source><translation>Уже выполняется другое копирование или восстановление</translation></message>
<message><source>toast_backup_warnings</source><translation>Копия создана с замечаниями: %1</translation></message>
<message><source>toast_credential_failed</source><translation>Пароль сохранён, но systemd-credential создать не удалось: %1</translation></message>
<message><source>toast_export_done</source><translation>Сохранено элементов: %1</translation></message>
<message><source>toast_export_failed</source><translation>Не удалось сохранить копии: %1</translation></message>
<message><source>toast_load_failed</source><translation>Не удалось загрузить список копий: %1</translation></message>
<message><source>toast_locked</source><translation>Снимки заблокированы</translation></message>
<message><source>toast_open_failed</source><translation>Не удалось открыть файл: %1</translation></message>
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
<name>RulesPanel</name>
<message><source>action_back_up_now</source><translation>Создать резервную копию</translation></message>
<message><source>action_stop_backup</source><translation>Остановить копирование</translation></message>
<message><source>menu_delete_rule</source><translation>Удалить правило…</translation></message>
<message><source>menu_edit_rule</source><translation>Настроить правило…</translation></message>
<message><source>menu_enable_rule</source><translation>Включить правило</translation></message>
<message><source>menu_pause_rule</source><translation>Приостановить правило</translation></message>
<message><source>menu_unlock_rule</source><translation>Разблокировать копию…</translation></message>
<message><source>rule_add</source><translation>Новое правило</translation></message>
<message><source>rule_daily_at</source><translation>Ежедневно в %1</translation></message>
<message><source>rule_every_day</source><translation>каждый день</translation></message>
<message><source>rule_last_backup</source><translation>последний %1</translation></message>
<message><source>rule_locked</source><translation>Заблокировано</translation></message>
<message><source>rule_no_password</source><translation>Пароль для просмотра не задан</translation></message>
<message><source>rule_paused</source><translation>на паузе</translation></message>
<message><source>rule_schedule_off</source><translation>Без расписания</translation></message>
<message><source>rule_untitled</source><translation>Без названия</translation></message>
<message><source>rule_weekly_at</source><translation>%1 в %2</translation></message>
<message><source>rules_count</source><translation>Правил: %1</translation></message>
<message><source>rules_empty_panel_hint</source><translation>Правил пока нет. Добавьте правило: что и когда копировать.</translation></message>
<message><source>rules_other_hint</source><translation>Снимки без подходящего правила</translation></message>
<message><source>rules_panel_hint</source><translation>Выберите правило, чтобы увидеть его снимки. Правый клик — действия.</translation></message>
<message><source>rules_panel_title</source><translation>Правила</translation></message>
<message><source>schedule_every_hours</source><translation>Каждые %1 ч</translation></message>
<message><source>schedule_every_minutes</source><translation>Каждые %1 мин</translation></message>
<message><source>timeline_count</source><translation>Снимков: %1</translation></message>
<message><source>weekday_short_fri</source><translation>Пт</translation></message>
<message><source>weekday_short_mon</source><translation>Пн</translation></message>
<message><source>weekday_short_sat</source><translation>Сб</translation></message>
<message><source>weekday_short_sun</source><translation>Вс</translation></message>
<message><source>weekday_short_thu</source><translation>Чт</translation></message>
<message><source>weekday_short_tue</source><translation>Вт</translation></message>
<message><source>weekday_short_wed</source><translation>Ср</translation></message>
</context>
<context>
<name>SettingsDialog</name>
<message><source>action_cancel</source><translation>Отмена</translation></message>
<message><source>action_change_password</source><translation>Изменить пароль…</translation></message>
<message><source>action_close</source><translation>Закрыть</translation></message>
<message><source>action_copy_key</source><translation>Скопировать ключ</translation></message>
<message><source>action_lock_now</source><translation>Заблокировать сейчас</translation></message>
<message><source>action_set_password</source><translation>Задать пароль…</translation></message>
<message><source>language_auto</source><translation>Системный</translation></message>
<message><source>lock_minutes</source><translation>%1 мин</translation></message>
<message><source>lock_never</source><translation>Никогда</translation></message>
<message><source>password_mode_common</source><translation>Один общий пароль</translation></message>
<message><source>password_mode_per_rule</source><translation>У каждого правила свой пароль</translation></message>
<message><source>settings_check_data</source><translation>Проверить с данными (10%)</translation></message>
<message><source>settings_check_failed</source><translation>Последняя проверка: %1 · найдены ошибки</translation></message>
<message><source>settings_check_hint</source><translation>Проверяет структуру и индекс репозитория. Проверка с данными дополнительно читает случайные 10% pack-файлов и может занять несколько минут.</translation></message>
<message><source>settings_check_never</source><translation>Репозиторий ещё не проверялся.</translation></message>
<message><source>settings_check_no_repo</source><translation>Репозитория ещё нет.</translation></message>
<message><source>settings_check_now</source><translation>Проверить сейчас</translation></message>
<message><source>settings_check_ok</source><translation>Последняя проверка: %1 · успешно</translation></message>
<message><source>settings_check_running</source><translation>Проверка репозитория…</translation></message>
<message><source>settings_check_stale</source><translation>Последняя проверка: %1 · больше 30 дней назад</translation></message>
<message><source>settings_check_title</source><translation>Проверка репозитория</translation></message>
<message><source>settings_choose_folder</source><translation>Выбрать папку</translation></message>
<message><source>settings_common_password</source><translation>Общий пароль</translation></message>
<message><source>settings_common_password_hint</source><translation>Один пароль открывает все копии. Он хранится только в памяти и забывается после периода неактивности. Пока он не задан, при открытии копии будет предложено его создать.</translation></message>
<message><source>settings_common_password_missing</source><translation>Не задан</translation></message>
<message><source>settings_common_password_set</source><translation>Задан</translation></message>
<message><source>settings_general</source><translation>Общие</translation></message>
<message><source>settings_keep_daily</source><translation>Хранить ежедневные (дней)</translation></message>
<message><source>settings_keep_hourly</source><translation>Хранить почасовые (часов)</translation></message>
<message><source>settings_keep_weekly</source><translation>Хранить еженедельные (недель)</translation></message>
<message><source>settings_language</source><translation>Язык</translation></message>
<message><source>settings_lock_hint</source><translation>Пароли хранятся только в памяти. Через указанное время без действий в окне они забываются; «Никогда» оставляет их до выхода или ручной блокировки.</translation></message>
<message><source>settings_lock_timeout</source><translation>Блокировать после неактивности</translation></message>
<message><source>settings_maintenance</source><translation>Обслуживание</translation></message>
<message><source>settings_nothing_to_prune</source><translation>Нечего удалять</translation></message>
<message><source>settings_notifications</source><translation>Уведомлять о завершении</translation></message>
<message><source>settings_open_config</source><translation>Открыть файл настроек</translation></message>
<message><source>settings_password_mode</source><translation>Пароли резервных копий</translation></message>
<message><source>settings_password_mode_hint</source><translation>Выберите, как устроены пароли для просмотра. Сами копии всегда используют сохранённый credential и пароль не спрашивают.</translation></message>
<message><source>settings_prune_confirm</source><translation>Удалить</translation></message>
<message><source>settings_prune_failed</source><translation>Не удалось проредить репозиторий (см. журнал)</translation></message>
<message><source>settings_prune_message</source><translation>Эти снимки выходят за пределы политики хранения и будут удалены. Закреплённые снимки сохраняются.</translation></message>
<message><source>settings_prune_now</source><translation>Проредить сейчас</translation></message>
<message><source>settings_prune_title</source><translation>Удалить снимков: %1?</translation></message>
<message><source>settings_pruned_now</source><translation>Политика хранения применена</translation></message>
<message><source>settings_recovery_hide</source><translation>Скрыть ключ</translation></message>
<message><source>settings_recovery_hint</source><translation>Сохранённый ключ позволяет копиям работать без запроса пароля. Сохраните его в надёжном месте: без ключа или пароля для просмотра репозиторий не открыть.</translation></message>
<message><source>settings_recovery_missing</source><translation>Сохранённый ключ не найден. Если вы задавали пароль для просмотра, используйте его.</translation></message>
<message><source>settings_recovery_show</source><translation>Показать ключ</translation></message>
<message><source>settings_recovery_title</source><translation>Ключ восстановления</translation></message>
<message><source>settings_recovery_warning</source><translation>Тот, кто увидит этот ключ, сможет прочитать все копии.</translation></message>
<message><source>settings_repo</source><translation>Папка репозитория</translation></message>
<message><source>settings_reset</source><translation>Сбросить настройки</translation></message>
<message><source>settings_retention</source><translation>Хранение</translation></message>
<message><source>settings_retention_hint</source><translation>Старые снимки прореживаются как в Time Machine: всё за последние часы, затем по одному в день, затем по одному в неделю.</translation></message>
<message><source>settings_rules</source><translation>Настроить правило…</translation></message>
<message><source>settings_security</source><translation>Безопасность</translation></message>
<message><source>settings_title</source><translation>Настройки</translation></message>
</context>
<context>
<name>SnapshotBrowser</name>
<message><source>action_restore</source><translation>Восстановить…</translation></message>
<message><source>action_save_copies</source><translation>Сохранить копии…</translation></message>
<message><source>browser_choose_export_folder</source><translation>Куда сохранить копии</translation></message>
<message><source>browser_clear_selection</source><translation>Снять выделение</translation></message>
<message><source>browser_empty</source><translation>Папка пуста</translation></message>
<message><source>browser_empty_hint</source><translation>Выберите другой снимок или поднимитесь выше.</translation></message>
<message><source>browser_home</source><translation>Домой</translation></message>
<message><source>browser_loading</source><translation>Загрузка снимка…</translation></message>
<message><source>browser_modified</source><translation>Изменён</translation></message>
<message><source>browser_name</source><translation>Имя</translation></message>
<message><source>browser_no_snapshot</source><translation>Снимок не выбран</translation></message>
<message><source>browser_no_snapshot_hint</source><translation>Выберите снимок в средней колонке, чтобы просмотреть его файлы.</translation></message>
<message><source>browser_search</source><translation>Поиск (Ctrl+F)</translation></message>
<message><source>browser_search_clear</source><translation>Закрыть поиск</translation></message>
<message><source>browser_search_count</source><translation>Найдено: %1</translation></message>
<message><source>browser_search_hint</source><translation>Введите хотя бы 2 символа</translation></message>
<message><source>browser_search_none</source><translation>Ничего не найдено</translation></message>
<message><source>browser_search_placeholder</source><translation>Поиск в снимке…</translation></message>
<message><source>browser_select_all</source><translation>Выбрать всё</translation></message>
<message><source>browser_selected</source><translation>Выбрано: %1</translation></message>
<message><source>browser_size</source><translation>Размер</translation></message>
<message><source>browser_snapshot</source><translation>Снимок %1, %2</translation></message>
<message><source>browser_up</source><translation>Вверх</translation></message>
<message><source>menu_copy_path</source><translation>Скопировать путь</translation></message>
<message><source>menu_open_item</source><translation>Открыть</translation></message>
<message><source>menu_restore_item</source><translation>Восстановить этот элемент…</translation></message>
<message><source>menu_save_item</source><translation>Сохранить этот элемент…</translation></message>
<message><source>snapshot_latest</source><translation>Последняя</translation></message>
</context>
<context>
<name>TimelinePanel</name>
<message><source>action_set_password</source><translation>Задать пароль…</translation></message>
<message><source>action_unlock</source><translation>Разблокировать</translation></message>
<message><source>menu_copy_id</source><translation>Скопировать ID снимка</translation></message>
<message><source>menu_delete_snapshot</source><translation>Удалить снимок…</translation></message>
<message><source>menu_pin_snapshot</source><translation>Закрепить снимок</translation></message>
<message><source>menu_unpin_snapshot</source><translation>Открепить снимок</translation></message>
<message><source>snapshot_added_tooltip</source><translation>Новых данных до сжатия: %1. Записано в репозиторий: %2.</translation></message>
<message><source>snapshot_changes_tooltip</source><translation>Файлов: %1 · %2 новых, %3 изменённых · %4</translation></message>
<message><source>snapshot_latest</source><translation>Последняя</translation></message>
<message><source>snapshot_on_disk</source><translation>%1 на диске</translation></message>
<message><source>timeline_count</source><translation>Снимков: %1</translation></message>
<message><source>timeline_empty</source><translation>Таймлайн появится после первой резервной копии.</translation></message>
<message><source>timeline_hint</source><translation>Сверху свежие. Выберите снимок, чтобы открыть файлы.</translation></message>
<message><source>timeline_loading</source><translation>Загрузка копий…</translation></message>
<message><source>timeline_locked_hint</source><translation>Нажмите «Разблокировать» и введите пароль этой копии, чтобы увидеть снимки.</translation></message>
<message><source>timeline_locked_title</source><translation>Копия заблокирована</translation></message>
<message><source>timeline_no_password_hint</source><translation>У этой копии ещё нет пароля. Задайте его, чтобы открыть копию.</translation></message>
<message><source>timeline_rule_title</source><translation>Снимки: %1</translation></message>
<message><source>timeline_title</source><translation>Снимки</translation></message>
</context>
<context>
<name>UnlockDialog</name>
<message><source>action_cancel</source><translation>Отмена</translation></message>
<message><source>action_change_password</source><translation>Изменить пароль…</translation></message>
<message><source>action_set_password</source><translation>Задать пароль…</translation></message>
<message><source>action_unlock</source><translation>Разблокировать</translation></message>
<message><source>password_change_hint</source><translation>Задайте новый пароль для этой резервной копии.</translation></message>
<message><source>password_change_title</source><translation>Изменить пароль для просмотра</translation></message>
<message><source>password_common_change_hint</source><translation>Задайте новый общий пароль для всех копий. Создаётся новый ключ, и все правила переключаются на него.</translation></message>
<message><source>password_common_change_title</source><translation>Изменить общий пароль</translation></message>
<message><source>password_common_create_hint</source><translation>Один пароль для всех копий. Он запрашивается при открытии любой копии; сами копии используют сохранённый credential и пароль не спрашивают. Восстановить забытый пароль невозможно.</translation></message>
<message><source>password_common_create_title</source><translation>Общий пароль для копий</translation></message>
<message><source>password_common_unlock_hint</source><translation>Введите общий пароль. Он открывает все копии для просмотра.</translation></message>
<message><source>password_common_unlock_title</source><translation>Разблокировать все копии</translation></message>
<message><source>password_confirm_placeholder</source><translation>Повторите пароль</translation></message>
<message><source>password_create_auth_hint</source><translation>Введите пароль существующей копии, затем задайте новый пароль для этой.</translation></message>
<message><source>password_create_hint</source><translation>Задайте пароль для просмотра этой копии. Он запрашивается при её открытии; сами копии используют сохранённый credential и пароль не спрашивают. Восстановить забытый пароль невозможно.</translation></message>
<message><source>password_create_title</source><translation>Пароль для просмотра</translation></message>
<message><source>password_current_placeholder</source><translation>Текущий пароль</translation></message>
<message><source>password_existing_placeholder</source><translation>Пароль существующей копии</translation></message>
<message><source>password_for_all_backups</source><translation>Все резервные копии</translation></message>
<message><source>password_for_backup</source><translation>Эта резервная копия</translation></message>
<message><source>password_for_rule</source><translation>Резервная копия «%1»</translation></message>
<message><source>password_mismatch</source><translation>Пароли не совпадают.</translation></message>
<message><source>password_new_placeholder</source><translation>Новый пароль</translation></message>
<message><source>password_placeholder</source><translation>Пароль</translation></message>
<message><source>password_show</source><translation>Показать пароль</translation></message>
<message><source>password_unlock_hint</source><translation>Введите пароль этой резервной копии. Он хранится только в памяти и забывается после периода неактивности.</translation></message>
<message><source>password_unlock_title</source><translation>Разблокировать резервную копию</translation></message>
<message><source>password_wrong</source><translation>Неверный пароль</translation></message>
</context>
</TS>
