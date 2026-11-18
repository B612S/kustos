<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE TS>
<TS version="2.1" language="en_US">
<context>
<name>AboutDialog</name>
<message><source>about_backup_folder</source><translation>Backup folder</translation></message>
<message><source>about_config</source><translation>Configuration</translation></message>
<message><source>about_credits</source><translation>Built with Qt 6, Kirigami and rsync. Window, menu and localization conventions follow Luma.</translation></message>
<message><source>about_description</source><translation>Time Machine-like snapshots of your home folder. Every snapshot is an rsync hard-link tree, so files that did not change take no extra space.</translation></message>
<message><source>about_license</source><translation>License: GPL-3.0-or-later</translation></message>
<message><source>about_log</source><translation>Log</translation></message>
<message><source>about_title</source><translation>About Aion</translation></message>
<message><source>about_version</source><translation>Version %1</translation></message>
<message><source>action_close</source><translation>Close</translation></message>
</context>
<context>
<name>Main</name>
<message><source>action_about</source><translation>About Aion</translation></message>
<message><source>action_back_up_now</source><translation>Back Up Now</translation></message>
<message><source>action_cancel</source><translation>Cancel</translation></message>
<message><source>action_delete</source><translation>Delete</translation></message>
<message><source>action_open_backup_folder</source><translation>Open Backup Folder</translation></message>
<message><source>action_quit</source><translation>Quit</translation></message>
<message><source>action_restore</source><translation>Restore…</translation></message>
<message><source>action_settings</source><translation>Settings</translation></message>
<message><source>app_title</source><translation>Aion</translation></message>
<message><source>delete_snapshot_message</source><translation>Delete snapshot %1? Files that exist only in this snapshot cannot be recovered.</translation></message>
<message><source>delete_snapshot_title</source><translation>Delete Snapshot</translation></message>
<message><source>empty_hint</source><translation>Aion keeps a history of your home folder, like Time Machine. Every snapshot stores only what changed since the previous one.</translation></message>
<message><source>empty_title</source><translation>No backups yet</translation></message>
<message><source>status_cancelling</source><translation>Cancelling…</translation></message>
<message><source>status_copying</source><translation>Copying… %1, %2 left</translation></message>
<message><source>status_preparing</source><translation>Preparing the file list…</translation></message>
<message><source>status_restoring</source><translation>Restoring %1</translation></message>
<message><source>status_summary</source><translation>%1 snapshot(s) · %2 · %3 free</translation></message>
<message><source>status_working</source><translation>Working…</translation></message>
<message><source>toast_backup_done</source><translation>Backup finished</translation></message>
<message><source>toast_backup_failed</source><translation>Backup failed: %1</translation></message>
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
<name>SettingsDialog</name>
<message><source>action_close</source><translation>Close</translation></message>
<message><source>language_auto</source><translation>System</translation></message>
<message><source>schedule_every_hours</source><translation>Every %1 h</translation></message>
<message><source>schedule_every_minutes</source><translation>Every %1 min</translation></message>
<message><source>schedule_state_absent</source><translation>not installed</translation></message>
<message><source>schedule_state_active</source><translation>running</translation></message>
<message><source>schedule_state_inactive</source><translation>installed, not running</translation></message>
<message><source>schedule_state_unknown</source><translation>unknown</translation></message>
<message><source>settings_add_excluded_folder</source><translation>Exclude Folder…</translation></message>
<message><source>settings_add_file</source><translation>Add File…</translation></message>
<message><source>settings_add_folder</source><translation>Add Folder…</translation></message>
<message><source>settings_choose_folder</source><translation>Choose Folder</translation></message>
<message><source>settings_excludes</source><translation>Excluded paths</translation></message>
<message><source>settings_excludes_hint</source><translation>Never backed up: games, caches, the recycle bin. Paths are matched by rsync.</translation></message>
<message><source>settings_general</source><translation>General</translation></message>
<message><source>settings_keep_daily</source><translation>Keep daily snapshots for (days)</translation></message>
<message><source>settings_keep_hourly</source><translation>Keep hourly snapshots for (hours)</translation></message>
<message><source>settings_keep_weekly</source><translation>Keep weekly snapshots for (weeks)</translation></message>
<message><source>settings_language</source><translation>Language</translation></message>
<message><source>settings_nothing_to_prune</source><translation>Nothing to prune</translation></message>
<message><source>settings_notifications</source><translation>Notify when a backup finishes</translation></message>
<message><source>settings_open_config</source><translation>Open Config File</translation></message>
<message><source>settings_preserve_acl</source><translation>Preserve ACLs</translation></message>
<message><source>settings_preserve_xattr</source><translation>Preserve extended attributes</translation></message>
<message><source>settings_prune_now</source><translation>Prune Now</translation></message>
<message><source>settings_pruned</source><translation>Removed %1 snapshot(s)</translation></message>
<message><source>settings_remove</source><translation>Remove</translation></message>
<message><source>settings_reset</source><translation>Reset to Defaults</translation></message>
<message><source>settings_retention</source><translation>Retention</translation></message>
<message><source>settings_retention_hint</source><translation>Old snapshots are thinned out like Time Machine: everything from the last hours, then one per day, then one per week.</translation></message>
<message><source>settings_schedule</source><translation>Schedule</translation></message>
<message><source>settings_schedule_enabled</source><translation>Automatic backups</translation></message>
<message><source>settings_schedule_interval</source><translation>Interval</translation></message>
<message><source>settings_schedule_state</source><translation>Timer: %1</translation></message>
<message><source>settings_single_instance</source><translation>Single window</translation></message>
<message><source>settings_sources</source><translation>Sources</translation></message>
<message><source>settings_sources_hint</source><translation>Folders and files that are backed up. Missing entries are skipped with a warning.</translation></message>
<message><source>settings_target</source><translation>Backup folder</translation></message>
<message><source>settings_title</source><translation>Settings</translation></message>
</context>
<context>
<name>SnapshotBrowser</name>
<message><source>action_restore</source><translation>Restore…</translation></message>
<message><source>browser_clear_selection</source><translation>Clear Selection</translation></message>
<message><source>browser_empty</source><translation>This folder is empty</translation></message>
<message><source>browser_empty_hint</source><translation>Pick another snapshot or go up a level.</translation></message>
<message><source>browser_select_all</source><translation>Select All</translation></message>
<message><source>browser_selected</source><translation>Selected: %1</translation></message>
<message><source>browser_up</source><translation>Up</translation></message>
<message><source>menu_copy_path</source><translation>Copy Path</translation></message>
<message><source>menu_restore_item</source><translation>Restore This Item…</translation></message>
<message><source>menu_reveal_item</source><translation>Show in File Manager</translation></message>
</context>
<context>
<name>TimelinePanel</name>
<message><source>day_today</source><translation>Today</translation></message>
<message><source>day_yesterday</source><translation>Yesterday</translation></message>
<message><source>menu_copy_path</source><translation>Copy Path</translation></message>
<message><source>menu_delete_snapshot</source><translation>Delete Snapshot…</translation></message>
<message><source>menu_reveal_snapshot</source><translation>Show in File Manager</translation></message>
<message><source>snapshot_added</source><translation>%1 added</translation></message>
<message><source>snapshot_latest</source><translation>Latest</translation></message>
<message><source>snapshot_status_failed</source><translation>Failed</translation></message>
<message><source>timeline_count</source><translation>%1 snapshot(s)</translation></message>
<message><source>timeline_empty</source><translation>The timeline appears here after the first backup.</translation></message>
<message><source>timeline_title</source><translation>Backups</translation></message>
</context>
<context>
<name>aion</name>
<message><source>error_backup_running</source><translation>A backup is already running</translation></message>
<message><source>error_invalid_path</source><translation>Invalid path: %1</translation></message>
<message><source>error_no_sources</source><translation>None of the configured sources exist</translation></message>
<message><source>error_no_space</source><translation>Not enough free space: %1</translation></message>
<message><source>error_no_target</source><translation>The backup folder is not configured</translation></message>
<message><source>error_restore_failed</source><translation>Could not restore %1 (%2)</translation></message>
<message><source>error_rsync_missing</source><translation>rsync is not installed or not in PATH</translation></message>
<message><source>error_snapshot_missing</source><translation>Snapshot not found</translation></message>
<message><source>error_target_inside_source</source><translation>The backup folder is inside the source %1</translation></message>
<message><source>error_target_missing</source><translation>Cannot create %1</translation></message>
<message><source>error_target_not_writable</source><translation>No write access to %1</translation></message>
<message><source>notify_backup_cancelled</source><translation>Backup cancelled</translation></message>
<message><source>notify_backup_done</source><translation>Backup finished</translation></message>
<message><source>notify_backup_done_body</source><translation>%1 added in %2</translation></message>
<message><source>notify_backup_failed</source><translation>Backup failed</translation></message>
<message><source>notify_restore_done</source><translation>Restore finished</translation></message>
<message><source>notify_restore_done_body</source><translation>%1 item(s) restored</translation></message>
<message><source>restore_suffix</source><translation>restored</translation></message>
<message><source>warning_partial_transfer</source><translation>Some files could not be copied (see the log)</translation></message>
<message><source>warning_source_missing</source><translation>Skipped missing source: %1</translation></message>
</context>
</TS>
