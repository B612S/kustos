<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE TS>
<TS version="2.1" language="en_US">
<context>
<name>AboutDialog</name>
<message><source>about_config</source><translation>Configuration</translation></message>
<message><source>about_credits</source><translation>Built with Rust, Qt 6, Kirigami and rustic (restic-compatible repositories). Window, menu and localization conventions follow Luma.</translation></message>
<message><source>about_description</source><translation>Time Machine-like snapshots of your home folder. Snapshots live in an encrypted, compressed, deduplicated repository (restic format), so unchanged data never costs extra space.</translation></message>
<message><source>about_key</source><translation>Backup passwords</translation></message>
<message><source>about_key_hint</source><translation>Every backup has its own password — a real repository key. Passwords are not written to the configuration: they are asked for when a backup is opened and forgotten after the inactivity timeout. Automatic backups use per-rule encrypted systemd credentials.</translation></message>
<message><source>about_license</source><translation>License: GPL-3.0-or-later</translation></message>
<message><source>about_log</source><translation>Log</translation></message>
<message><source>about_repo</source><translation>Repository</translation></message>
<message><source>about_title</source><translation>About Kustos</translation></message>
<message><source>about_version</source><translation>Version %1</translation></message>
<message><source>action_close</source><translation>Close</translation></message>
</context>
<context>
<name>BackupDialog</name>
<message><source>action_cancel</source><translation>Cancel</translation></message>
<message><source>action_change_password</source><translation>Change Password…</translation></message>
<message><source>action_save</source><translation>Save</translation></message>
<message><source>action_set_password</source><translation>Set Password…</translation></message>
<message><source>backup_add_exclude</source><translation>Exclude Folder…</translation></message>
<message><source>backup_add_file</source><translation>Add File…</translation></message>
<message><source>backup_add_folder</source><translation>Add Folder…</translation></message>
<message><source>backup_add_pattern</source><translation>Add Pattern</translation></message>
<message><source>backup_add_recommended</source><translation>Recommended</translation></message>
<message><source>backup_exclude_hint</source><translation>Nothing below these paths is copied, even if it is inside a backed-up folder.</translation></message>
<message><source>backup_exclude_title</source><translation>Excluded paths</translation></message>
<message><source>backup_include_hint</source><translation>Every listed folder and file is copied into the snapshot.</translation></message>
<message><source>backup_include_title</source><translation>What to copy</translation></message>
<message><source>backup_no_sources</source><translation>Add at least one folder or file to back up.</translation></message>
<message><source>backup_pattern_placeholder</source><translation>Pattern, e.g. *.log, node_modules or ~/.cache/**</translation></message>
<message><source>backup_recommended_hint</source><translation>Adds caches, the KDE file index (baloo) and other rebuildable data</translation></message>
<message><source>backup_title</source><translation>Backup Rule</translation></message>
<message><source>rule_advanced</source><translation>Advanced</translation></message>
<message><source>rule_automatic</source><translation>Automatic</translation></message>
<message><source>rule_enabled</source><translation>Enabled</translation></message>
<message><source>rule_exclude_larger</source><translation>Skip files larger than</translation></message>
<message><source>rule_exclude_larger_hint</source><translation>Rustic size suffix: K, M, G, T. Empty means no limit.</translation></message>
<message><source>rule_exclude_larger_placeholder</source><translation>e.g. 2G</translation></message>
<message><source>rule_minutes_value</source><translation>%1 min</translation></message>
<message><source>rule_mode_daily</source><translation>Daily</translation></message>
<message><source>rule_mode_interval</source><translation>Every N minutes</translation></message>
<message><source>rule_mode_weekly</source><translation>Certain days</translation></message>
<message><source>rule_name_placeholder</source><translation>Rule name</translation></message>
<message><source>rule_new_name</source><translation>New rule</translation></message>
<message><source>rule_no_excludes</source><translation>Nothing is excluded</translation></message>
<message><source>rule_one_filesystem</source><translation>Stay on one filesystem</translation></message>
<message><source>rule_one_filesystem_hint</source><translation>Skip mounted disks inside a source instead of following them.</translation></message>
<message><source>rule_password_common_hint</source><translation>All rules use one common password. It is managed in Settings → Security.</translation></message>
<message><source>rule_password_common_missing</source><translation>Common password not set (Settings → Security)</translation></message>
<message><source>rule_password_common_set</source><translation>Uses the common password</translation></message>
<message><source>rule_password_hint</source><translation>The viewing password is asked when this backup is opened and is kept in memory only. Backups use the saved systemd credential, which is created after the first backup or when you set this password.</translation></message>
<message><source>rule_password_missing</source><translation>No viewing password — opening this backup will ask you to create one. Backups themselves do not ask.</translation></message>
<message><source>rule_password_set</source><translation>A viewing password is set</translation></message>
<message><source>rule_password_title</source><translation>Password</translation></message>
<message><source>rule_password_unattended_missing</source><translation>No saved credential yet — run a backup once (or set a viewing password) to create it.</translation></message>
<message><source>rule_password_unattended_ready</source><translation>Automatic backups can run without the window (systemd credential ready).</translation></message>
<message><source>rule_remove_exclude</source><translation>Remove exclusion</translation></message>
<message><source>rule_remove_source</source><translation>Remove folder</translation></message>
<message><source>rule_schedule_hint</source><translation>Installed as a systemd user service and timer. The saved credential — the viewing password or the generated backup key — is passed to the service, so the window does not have to be open.</translation></message>
<message><source>rule_schedule_needs_password</source><translation>Run a backup once first: the systemd service needs a saved credential.</translation></message>
<message><source>rule_schedule_title</source><translation>Schedule</translation></message>
<message><source>rule_time_hint</source><translation>24-hour, HH:MM</translation></message>
<message><source>rule_time_placeholder</source><translation>HH:MM</translation></message>
<message><source>rules_hint</source><translation>This rule defines what is copied, what is excluded and when it runs. Changes are saved immediately.</translation></message>
<message><source>rules_scheduled</source><translation>Scheduled: %1 rule(s)</translation></message>
<message><source>rules_title</source><translation>What and when to back up</translation></message>
<message><source>weekday_short_fri</source><translation>Fri</translation></message>
<message><source>weekday_short_mon</source><translation>Mon</translation></message>
<message><source>weekday_short_sat</source><translation>Sat</translation></message>
<message><source>weekday_short_sun</source><translation>Sun</translation></message>
<message><source>weekday_short_thu</source><translation>Thu</translation></message>
<message><source>weekday_short_tue</source><translation>Tue</translation></message>
<message><source>weekday_short_wed</source><translation>Wed</translation></message>
</context>
<context>
<name>Main</name>
<message><source>action_about</source><translation>About Kustos</translation></message>
<message><source>action_back_up_now</source><translation>Back Up Now</translation></message>
<message><source>action_backup_rules</source><translation>Edit Rule…</translation></message>
<message><source>action_cancel</source><translation>Cancel</translation></message>
<message><source>action_delete</source><translation>Delete</translation></message>
<message><source>action_lock_backups</source><translation>Lock Backups</translation></message>
<message><source>action_open_backup_folder</source><translation>Open Backup Folder</translation></message>
<message><source>action_quit</source><translation>Quit</translation></message>
<message><source>action_restore</source><translation>Restore…</translation></message>
<message><source>action_save_copies</source><translation>Save Copies…</translation></message>
<message><source>action_settings</source><translation>Settings</translation></message>
<message><source>app_title</source><translation>Kustos</translation></message>
<message><source>browser_load_failed</source><translation>Could not read the snapshot: %1</translation></message>
<message><source>day_today</source><translation>Today</translation></message>
<message><source>day_yesterday</source><translation>Yesterday</translation></message>
<message><source>delete_rule_message</source><translation>Delete the rule “%1”? Its snapshots stay in the repository and appear under Other Backups.</translation></message>
<message><source>delete_rule_title</source><translation>Delete Rule</translation></message>
<message><source>delete_snapshot_message</source><translation>Delete snapshot %1? Files that exist only in this snapshot cannot be recovered.</translation></message>
<message><source>delete_snapshot_title</source><translation>Delete Snapshot</translation></message>
<message><source>rule_new_name</source><translation>New rule</translation></message>
<message><source>rule_untitled</source><translation>Untitled rule</translation></message>
<message><source>rules_empty</source><translation>No backup rules</translation></message>
<message><source>rules_other</source><translation>Other Backups</translation></message>
<message><source>status_cancelling</source><translation>Cancelling…</translation></message>
<message><source>status_copying</source><translation>Copying… %1, %2 left</translation></message>
<message><source>status_copying_rule</source><translation>Copying %1… %2, %3 left</translation></message>
<message><source>status_locked</source><translation>Snapshots are locked · open a backup to enter its password</translation></message>
<message><source>status_opening</source><translation>Opening %1…</translation></message>
<message><source>status_preparing</source><translation>Preparing the file list…</translation></message>
<message><source>status_repo_missing</source><translation>Repository not found at %1 — is the drive mounted?</translation></message>
<message><source>status_restoring</source><translation>Restoring %1</translation></message>
<message><source>status_saving</source><translation>Saving %1</translation></message>
<message><source>status_summary</source><translation>%1 snapshot(s) · %2 · %3 free</translation></message>
<message><source>status_working</source><translation>Working…</translation></message>
<message><source>timeline_empty</source><translation>The timeline appears here after the first backup.</translation></message>
<message><source>timeline_empty_rule</source><translation>No snapshots for this rule yet.</translation></message>
<message><source>toast_backup_done</source><translation>Backup finished</translation></message>
<message><source>toast_backup_done_locked</source><translation>Backup finished — unlock the backup to see the new snapshot</translation></message>
<message><source>toast_backup_failed</source><translation>Backup failed: %1</translation></message>
<message><source>toast_backup_running</source><translation>Another backup or restore is already running</translation></message>
<message><source>toast_backup_warnings</source><translation>Backup finished with warnings: %1</translation></message>
<message><source>toast_credential_failed</source><translation>The password was saved, but the systemd credential could not be created: %1</translation></message>
<message><source>toast_export_done</source><translation>Saved %1 item(s) to the chosen folder</translation></message>
<message><source>toast_export_failed</source><translation>Could not save the copies: %1</translation></message>
<message><source>toast_load_failed</source><translation>Could not load the backup list: %1</translation></message>
<message><source>toast_locked</source><translation>Snapshots locked</translation></message>
<message><source>toast_open_failed</source><translation>Could not open the file: %1</translation></message>
<message><source>toast_restore_done</source><translation>Restored %1 item(s)</translation></message>
<message><source>toast_restore_failed</source><translation>Restore failed: %1</translation></message>
</context>
<context>
<name>RestoreDialog</name>
<message><source>action_cancel</source><translation>Cancel</translation></message>
<message><source>action_restore</source><translation>Restore…</translation></message>
<message><source>restore_choose_folder</source><translation>Choose Folder</translation></message>
<message><source>restore_conflict</source><translation>If a file already exists</translation></message>
<message><source>restore_count</source><translation>Items to restore: %1</translation></message>
<message><source>restore_destination</source><translation>Destination</translation></message>
<message><source>restore_folder</source><translation>Another folder…</translation></message>
<message><source>restore_into</source><translation>Restoring into</translation></message>
<message><source>restore_more</source><translation>and %1 more</translation></message>
<message><source>restore_original</source><translation>Original location</translation></message>
<message><source>restore_policy_keep_both</source><translation>Keep both</translation></message>
<message><source>restore_policy_overwrite</source><translation>Replace it</translation></message>
<message><source>restore_policy_skip</source><translation>Keep the existing file</translation></message>
<message><source>restore_title</source><translation>Restore from a Snapshot</translation></message>
</context>
<context>
<name>RulesPanel</name>
<message><source>action_back_up_now</source><translation>Back Up Now</translation></message>
<message><source>action_stop_backup</source><translation>Stop Backup</translation></message>
<message><source>menu_delete_rule</source><translation>Delete Rule…</translation></message>
<message><source>menu_edit_rule</source><translation>Edit Rule…</translation></message>
<message><source>menu_enable_rule</source><translation>Enable Rule</translation></message>
<message><source>menu_pause_rule</source><translation>Pause Rule</translation></message>
<message><source>menu_unlock_rule</source><translation>Unlock Backup…</translation></message>
<message><source>rule_add</source><translation>New Rule</translation></message>
<message><source>rule_daily_at</source><translation>Daily at %1</translation></message>
<message><source>rule_every_day</source><translation>every day</translation></message>
<message><source>rule_last_backup</source><translation>last %1</translation></message>
<message><source>rule_locked</source><translation>Locked</translation></message>
<message><source>rule_no_password</source><translation>No viewing password</translation></message>
<message><source>rule_paused</source><translation>paused</translation></message>
<message><source>rule_schedule_off</source><translation>No schedule</translation></message>
<message><source>rule_untitled</source><translation>Untitled rule</translation></message>
<message><source>rule_weekly_at</source><translation>Weekly on %1 at %2</translation></message>
<message><source>rules_count</source><translation>Rules: %1</translation></message>
<message><source>rules_empty_panel_hint</source><translation>No rules yet. Add one to choose what is copied and when.</translation></message>
<message><source>rules_other_hint</source><translation>Snapshots without a matching rule</translation></message>
<message><source>rules_panel_hint</source><translation>Pick a rule to see its snapshots. Right-click for actions.</translation></message>
<message><source>rules_panel_title</source><translation>Rules</translation></message>
<message><source>schedule_every_hours</source><translation>Every %1 h</translation></message>
<message><source>schedule_every_minutes</source><translation>Every %1 min</translation></message>
<message><source>timeline_count</source><translation>%1 snapshot(s)</translation></message>
<message><source>weekday_short_fri</source><translation>Fri</translation></message>
<message><source>weekday_short_mon</source><translation>Mon</translation></message>
<message><source>weekday_short_sat</source><translation>Sat</translation></message>
<message><source>weekday_short_sun</source><translation>Sun</translation></message>
<message><source>weekday_short_thu</source><translation>Thu</translation></message>
<message><source>weekday_short_tue</source><translation>Tue</translation></message>
<message><source>weekday_short_wed</source><translation>Wed</translation></message>
</context>
<context>
<name>SettingsDialog</name>
<message><source>action_cancel</source><translation>Cancel</translation></message>
<message><source>action_change_password</source><translation>Change Password…</translation></message>
<message><source>action_close</source><translation>Close</translation></message>
<message><source>action_copy_key</source><translation>Copy Key</translation></message>
<message><source>action_lock_now</source><translation>Lock Now</translation></message>
<message><source>action_set_password</source><translation>Set Password…</translation></message>
<message><source>language_auto</source><translation>System</translation></message>
<message><source>lock_minutes</source><translation>%1 min</translation></message>
<message><source>lock_never</source><translation>Never</translation></message>
<message><source>password_mode_common</source><translation>One common password</translation></message>
<message><source>password_mode_per_rule</source><translation>Each rule has its own password</translation></message>
<message><source>settings_check_data</source><translation>Check with Data (10%)</translation></message>
<message><source>settings_check_failed</source><translation>Last check: %1 · errors found</translation></message>
<message><source>settings_check_hint</source><translation>Verifies the repository structure and index. The data check also reads a random 10% of the pack files and can take several minutes.</translation></message>
<message><source>settings_check_never</source><translation>The repository has not been checked yet.</translation></message>
<message><source>settings_check_no_repo</source><translation>The repository does not exist yet.</translation></message>
<message><source>settings_check_now</source><translation>Check Now</translation></message>
<message><source>settings_check_ok</source><translation>Last check: %1 · passed</translation></message>
<message><source>settings_check_running</source><translation>Checking the repository…</translation></message>
<message><source>settings_check_stale</source><translation>Last check: %1 · more than 30 days ago</translation></message>
<message><source>settings_check_title</source><translation>Repository Check</translation></message>
<message><source>settings_choose_folder</source><translation>Choose Folder</translation></message>
<message><source>settings_common_password</source><translation>Common password</translation></message>
<message><source>settings_common_password_hint</source><translation>One password opens every backup. It is kept in memory only and forgotten after the inactivity timeout. Without it, opening a backup asks you to set it.</translation></message>
<message><source>settings_common_password_missing</source><translation>Not set</translation></message>
<message><source>settings_common_password_set</source><translation>Set</translation></message>
<message><source>settings_general</source><translation>General</translation></message>
<message><source>settings_keep_daily</source><translation>Keep daily snapshots for (days)</translation></message>
<message><source>settings_keep_hourly</source><translation>Keep hourly snapshots for (hours)</translation></message>
<message><source>settings_keep_weekly</source><translation>Keep weekly snapshots for (weeks)</translation></message>
<message><source>settings_language</source><translation>Language</translation></message>
<message><source>settings_lock_hint</source><translation>Passwords are kept in memory only. After this time without activity in the window they are forgotten again; “Never” keeps them until you quit or lock them yourself.</translation></message>
<message><source>settings_lock_timeout</source><translation>Lock after inactivity</translation></message>
<message><source>settings_maintenance</source><translation>Maintenance</translation></message>
<message><source>settings_nothing_to_prune</source><translation>Nothing to prune</translation></message>
<message><source>settings_notifications</source><translation>Notify when a backup finishes</translation></message>
<message><source>settings_open_config</source><translation>Open Config File</translation></message>
<message><source>settings_password_mode</source><translation>Backup passwords</translation></message>
<message><source>settings_password_mode_hint</source><translation>Choose how viewing passwords work. Backups themselves always use the saved credential and never ask.</translation></message>
<message><source>settings_prune_confirm</source><translation>Delete</translation></message>
<message><source>settings_prune_failed</source><translation>Could not prune the repository (see the log)</translation></message>
<message><source>settings_prune_message</source><translation>These snapshots are outside the retention policy and will be deleted. Pinned snapshots are kept.</translation></message>
<message><source>settings_prune_now</source><translation>Prune Now</translation></message>
<message><source>settings_prune_title</source><translation>Delete %1 snapshot(s)?</translation></message>
<message><source>settings_pruned_now</source><translation>Retention policy applied</translation></message>
<message><source>settings_recovery_hide</source><translation>Hide Key</translation></message>
<message><source>settings_recovery_hint</source><translation>A saved key lets backups run without asking for a password. Keep a copy somewhere safe: without a key or a viewing password the repository cannot be opened.</translation></message>
<message><source>settings_recovery_missing</source><translation>No saved key was found. If you set a viewing password, use it instead.</translation></message>
<message><source>settings_recovery_show</source><translation>Show Key</translation></message>
<message><source>settings_recovery_title</source><translation>Recovery Key</translation></message>
<message><source>settings_recovery_warning</source><translation>Anyone who sees this key can read every backup.</translation></message>
<message><source>settings_repo</source><translation>Repository folder</translation></message>
<message><source>settings_reset</source><translation>Reset to Defaults</translation></message>
<message><source>settings_retention</source><translation>Retention</translation></message>
<message><source>settings_retention_hint</source><translation>Old snapshots are thinned out like Time Machine: everything from the last hours, then one per day, then one per week.</translation></message>
<message><source>settings_rules</source><translation>Edit Rule…</translation></message>
<message><source>settings_security</source><translation>Security</translation></message>
<message><source>settings_title</source><translation>Settings</translation></message>
</context>
<context>
<name>SnapshotBrowser</name>
<message><source>action_restore</source><translation>Restore…</translation></message>
<message><source>action_save_copies</source><translation>Save Copies…</translation></message>
<message><source>browser_choose_export_folder</source><translation>Save Copies to Folder</translation></message>
<message><source>browser_clear_selection</source><translation>Clear Selection</translation></message>
<message><source>browser_empty</source><translation>This folder is empty</translation></message>
<message><source>browser_empty_hint</source><translation>Pick another snapshot or go up a level.</translation></message>
<message><source>browser_home</source><translation>Home</translation></message>
<message><source>browser_loading</source><translation>Loading snapshot…</translation></message>
<message><source>browser_modified</source><translation>Modified</translation></message>
<message><source>browser_name</source><translation>Name</translation></message>
<message><source>browser_no_snapshot</source><translation>No snapshot selected</translation></message>
<message><source>browser_no_snapshot_hint</source><translation>Pick a snapshot in the middle column to browse its files.</translation></message>
<message><source>browser_search</source><translation>Search (Ctrl+F)</translation></message>
<message><source>browser_search_clear</source><translation>Close search</translation></message>
<message><source>browser_search_count</source><translation>%1 match(es)</translation></message>
<message><source>browser_search_hint</source><translation>Type at least 2 characters</translation></message>
<message><source>browser_search_none</source><translation>Nothing found</translation></message>
<message><source>browser_search_placeholder</source><translation>Search this snapshot…</translation></message>
<message><source>browser_select_all</source><translation>Select All</translation></message>
<message><source>browser_selected</source><translation>Selected: %1</translation></message>
<message><source>browser_size</source><translation>Size</translation></message>
<message><source>browser_snapshot</source><translation>Snapshot %1, %2</translation></message>
<message><source>browser_up</source><translation>Up</translation></message>
<message><source>menu_copy_path</source><translation>Copy Path</translation></message>
<message><source>menu_open_item</source><translation>Open</translation></message>
<message><source>menu_restore_item</source><translation>Restore This Item…</translation></message>
<message><source>menu_save_item</source><translation>Save This Item…</translation></message>
<message><source>snapshot_latest</source><translation>Latest</translation></message>
</context>
<context>
<name>TimelinePanel</name>
<message><source>action_set_password</source><translation>Set Password…</translation></message>
<message><source>action_unlock</source><translation>Unlock</translation></message>
<message><source>menu_copy_id</source><translation>Copy Snapshot ID</translation></message>
<message><source>menu_delete_snapshot</source><translation>Delete Snapshot…</translation></message>
<message><source>menu_pin_snapshot</source><translation>Pin Snapshot</translation></message>
<message><source>menu_unpin_snapshot</source><translation>Unpin Snapshot</translation></message>
<message><source>snapshot_added_tooltip</source><translation>New data before compression: %1. Written to the repository: %2.</translation></message>
<message><source>snapshot_changes_tooltip</source><translation>Files: %1 · %2 new, %3 changed · %4</translation></message>
<message><source>snapshot_latest</source><translation>Latest</translation></message>
<message><source>snapshot_on_disk</source><translation>%1 on disk</translation></message>
<message><source>timeline_count</source><translation>%1 snapshot(s)</translation></message>
<message><source>timeline_empty</source><translation>The timeline appears here after the first backup.</translation></message>
<message><source>timeline_hint</source><translation>Newest first. Pick a snapshot to browse its files.</translation></message>
<message><source>timeline_loading</source><translation>Loading backups…</translation></message>
<message><source>timeline_locked_hint</source><translation>Unlock it and enter this backup's password to see its snapshots.</translation></message>
<message><source>timeline_locked_title</source><translation>This backup is locked</translation></message>
<message><source>timeline_no_password_hint</source><translation>This backup has no password yet. Set one to open it.</translation></message>
<message><source>timeline_rule_title</source><translation>Snapshots: %1</translation></message>
<message><source>timeline_title</source><translation>Snapshots</translation></message>
</context>
<context>
<name>UnlockDialog</name>
<message><source>action_cancel</source><translation>Cancel</translation></message>
<message><source>action_change_password</source><translation>Change Password…</translation></message>
<message><source>action_set_password</source><translation>Set Password…</translation></message>
<message><source>action_unlock</source><translation>Unlock</translation></message>
<message><source>password_change_hint</source><translation>Choose a new password for this backup.</translation></message>
<message><source>password_change_title</source><translation>Change Viewing Password</translation></message>
<message><source>password_common_change_hint</source><translation>Choose a new common password for all backups. A new key is created and every rule switches to it.</translation></message>
<message><source>password_common_change_title</source><translation>Change Common Password</translation></message>
<message><source>password_common_create_hint</source><translation>One password for all backups. It is asked when any backup is opened; backups themselves use the saved credential and never ask. There is no recovery if you forget it.</translation></message>
<message><source>password_common_create_title</source><translation>Common Backup Password</translation></message>
<message><source>password_common_unlock_hint</source><translation>Enter the common password. It unlocks every backup for viewing.</translation></message>
<message><source>password_common_unlock_title</source><translation>Unlock All Backups</translation></message>
<message><source>password_confirm_placeholder</source><translation>Repeat the password</translation></message>
<message><source>password_create_auth_hint</source><translation>Enter the password of an existing backup, then choose a new password for this one.</translation></message>
<message><source>password_create_hint</source><translation>Give this backup its own viewing password. It is asked when the backup is opened; backups themselves use the saved credential and never ask for it. There is no recovery if you forget it.</translation></message>
<message><source>password_create_title</source><translation>Viewing Password</translation></message>
<message><source>password_current_placeholder</source><translation>Current password</translation></message>
<message><source>password_existing_placeholder</source><translation>Existing backup password</translation></message>
<message><source>password_for_all_backups</source><translation>All backups</translation></message>
<message><source>password_for_backup</source><translation>This backup</translation></message>
<message><source>password_for_rule</source><translation>Backup “%1”</translation></message>
<message><source>password_mismatch</source><translation>The passwords do not match.</translation></message>
<message><source>password_new_placeholder</source><translation>New password</translation></message>
<message><source>password_placeholder</source><translation>Password</translation></message>
<message><source>password_show</source><translation>Show password</translation></message>
<message><source>password_unlock_hint</source><translation>Enter the password of this backup. It is kept in memory only and is forgotten after the inactivity timeout.</translation></message>
<message><source>password_unlock_title</source><translation>Unlock Backup</translation></message>
<message><source>password_wrong</source><translation>Wrong password</translation></message>
</context>
</TS>
