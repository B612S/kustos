<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE TS>
<TS version="2.1" language="en_US">
<context>
<name>AboutDialog</name>
<message><source>about_config</source><translation>Configuration</translation></message>
<message><source>about_credits</source><translation>Built with Rust, Qt 6, Kirigami and rustic (restic-compatible repositories). Window, menu and localization conventions follow Luma.</translation></message>
<message><source>about_description</source><translation>Time Machine-like snapshots of your home folder. Snapshots live in an encrypted, compressed, deduplicated repository (restic format), so unchanged data never costs extra space.</translation></message>
<message><source>about_key</source><translation>Repository key</translation></message>
<message><source>about_key_hint</source><translation>The key decrypts every snapshot. It is stored in ~/.config/kustos/repo.password and is the only way to read the backups if this machine is lost — save it somewhere else, for example in a password manager.</translation></message>
<message><source>about_license</source><translation>License: GPL-3.0-or-later</translation></message>
<message><source>about_log</source><translation>Log</translation></message>
<message><source>about_repo</source><translation>Repository</translation></message>
<message><source>about_title</source><translation>About Kustos</translation></message>
<message><source>about_version</source><translation>Version %1</translation></message>
<message><source>action_close</source><translation>Close</translation></message>
<message><source>action_copy_key</source><translation>Copy Key</translation></message>
<message><source>action_hide</source><translation>Hide</translation></message>
<message><source>action_show</source><translation>Show</translation></message>
</context>
<context>
<name>BackupDialog</name>
<message><source>action_back_up_now</source><translation>Back Up Now</translation></message>
<message><source>action_cancel</source><translation>Cancel</translation></message>
<message><source>backup_add_exclude</source><translation>Exclude Folder…</translation></message>
<message><source>backup_add_file</source><translation>Add File…</translation></message>
<message><source>backup_add_folder</source><translation>Add Folder…</translation></message>
<message><source>backup_exclude_hint</source><translation>Nothing below these paths is copied, even if it is inside a backed-up folder.</translation></message>
<message><source>backup_exclude_title</source><translation>Excluded paths</translation></message>
<message><source>backup_include_hint</source><translation>Every listed folder and file is copied into the snapshot.</translation></message>
<message><source>backup_include_title</source><translation>Folders to back up</translation></message>
<message><source>backup_no_sources</source><translation>Add at least one folder or file to back up.</translation></message>
<message><source>backup_title</source><translation>What to Back Up</translation></message>
</context>
<context>
<name>Main</name>
<message><source>action_about</source><translation>About Kustos</translation></message>
<message><source>action_back_up_now</source><translation>Back Up Now</translation></message>
<message><source>action_cancel</source><translation>Cancel</translation></message>
<message><source>action_delete</source><translation>Delete</translation></message>
<message><source>action_open_backup_folder</source><translation>Open Backup Folder</translation></message>
<message><source>action_quit</source><translation>Quit</translation></message>
<message><source>action_restore</source><translation>Restore…</translation></message>
<message><source>action_settings</source><translation>Settings</translation></message>
<message><source>app_title</source><translation>Kustos</translation></message>
<message><source>day_today</source><translation>Today</translation></message>
<message><source>day_yesterday</source><translation>Yesterday</translation></message>
<message><source>delete_snapshot_message</source><translation>Delete snapshot %1? Files that exist only in this snapshot cannot be recovered.</translation></message>
<message><source>delete_snapshot_title</source><translation>Delete Snapshot</translation></message>
<message><source>empty_hint</source><translation>Kustos keeps a history of your home folder, like Time Machine. Every snapshot stores only what changed since the previous one.</translation></message>
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
<message><source>settings_general</source><translation>General</translation></message>
<message><source>settings_keep_daily</source><translation>Keep daily snapshots for (days)</translation></message>
<message><source>settings_keep_hourly</source><translation>Keep hourly snapshots for (hours)</translation></message>
<message><source>settings_keep_weekly</source><translation>Keep weekly snapshots for (weeks)</translation></message>
<message><source>settings_language</source><translation>Language</translation></message>
<message><source>settings_nothing_to_prune</source><translation>Nothing to prune</translation></message>
<message><source>settings_notifications</source><translation>Notify when a backup finishes</translation></message>
<message><source>settings_open_config</source><translation>Open Config File</translation></message>
<message><source>settings_prune_now</source><translation>Prune Now</translation></message>
<message><source>settings_pruned_now</source><translation>Retention policy applied</translation></message>
<message><source>settings_repo</source><translation>Repository folder</translation></message>
<message><source>settings_reset</source><translation>Reset to Defaults</translation></message>
<message><source>settings_retention</source><translation>Retention</translation></message>
<message><source>settings_retention_hint</source><translation>Old snapshots are thinned out like Time Machine: everything from the last hours, then one per day, then one per week.</translation></message>
<message><source>settings_schedule</source><translation>Schedule</translation></message>
<message><source>settings_schedule_enabled</source><translation>Automatic backups</translation></message>
<message><source>settings_schedule_interval</source><translation>Interval</translation></message>
<message><source>settings_schedule_state</source><translation>Timer: %1</translation></message>
<message><source>settings_scope</source><translation>Backup Scope…</translation></message>
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
</context>
<context>
<name>TimelinePanel</name>
<message><source>menu_copy_id</source><translation>Copy Snapshot ID</translation></message>
<message><source>menu_delete_snapshot</source><translation>Delete Snapshot…</translation></message>
<message><source>snapshot_added</source><translation>%1 added</translation></message>
<message><source>snapshot_latest</source><translation>Latest</translation></message>
<message><source>timeline_count</source><translation>%1 snapshot(s)</translation></message>
<message><source>timeline_empty</source><translation>The timeline appears here after the first backup.</translation></message>
<message><source>timeline_title</source><translation>Backups</translation></message>
</context>
</TS>
