# Snapshot format

Aion owns exactly two things on the backup volume: the `snapshots/` directory
and a lock file next to it. Everything below is plain files, readable without
Aion.

```
<backup-folder>/
  .aion.lock                        # QLockFile, held while a backup runs
  snapshots/
    latest -> 2026-11-18_19-00-00   # relative symlink to the newest snapshot
    2026-11-18_19-00-00/
      meta.json
      home/soneriald/...            # mirrored absolute paths of the sources
    2026-11-17_19-00-00/
      ...
```

## Why the paths look like that

The sources are passed to rsync with `--relative`, so an absolute source
`/home/soneriald/.config` becomes `<snapshot>/home/soneriald/.config`. This has
two consequences:

- a snapshot is a faithful image of the absolute paths, which makes
  `--link-dest` and restore path arithmetic trivial and unambiguous;
- restoring into `/` puts every file back exactly where it was.

## Hard links

`rsync -aHAX --link-dest=<previous snapshot>` compares the new tree with the
previous snapshot and, for every unchanged file, creates a hard link instead of
a copy. The bytes exist once on disk and are referenced from every snapshot.
Deleting a snapshot only drops its links; the data survives in the snapshots
that still reference it. Files that are modified in place by an application
are copied on the next run (the old inode stays untouched in the old
snapshots).

`--delete` makes the new snapshot reflect the current sources: files removed
from the home folder disappear from the *new* snapshot only.

## meta.json

```json
{
  "id": "2026-11-18_19-00-00",
  "started_at": "2026-11-18T19:00:01",
  "finished_at": "2026-11-18T19:03:42",
  "status": "ok",
  "total_bytes": 34567890123,
  "total_files": 412345,
  "transferred_bytes": 12345678,
  "transferred_files": 42,
  "duration_seconds": 221,
  "rsync_exit": 0,
  "error": "",
  "warnings": [],
  "hostname": "host",
  "app_version": "0.1.0",
  "sources": ["/home/soneriald/.config"],
  "excludes": ["/home/soneriald/.local/share/Steam"]
}
```

`status` is one of `ok`, `warning` (rsync exit 23/24: some files or attributes
were not transferred), `failed` (normally removed) or `running` (a backup is
in progress). The file is written with `QSaveFile` and only after a successful
run.

## Retention

Snapshots are thinned with a three-bucket policy, evaluated against the
snapshot start time:

1. everything younger than `hourlyHours` is kept;
2. up to `dailyDays` days: the newest snapshot of each calendar day is kept;
3. up to `weeklyWeeks` weeks after that: the newest snapshot of each ISO week
   is kept;
4. anything older is deleted.

The newest snapshot is always kept, even if it is the only one and it is older
than every window. A snapshot is never thinned while it is the only one.

## Restore semantics

Every item is restored by a single `rsync -aHAX` run, which creates missing
parent directories and preserves symlinks, permissions, hard links, ACLs and
xattrs:

- `--to /` (default, "original location") writes `/<relative path>`;
- `--to DIR` writes `DIR/<relative path>`, producing a safe parallel copy;
- `--policy skip` leaves an existing target untouched;
- `--policy keep_both` writes next to it as `name (restored <timestamp>)`;
- anything else replaces the target.

Paths are validated (no absolute paths, no `..`) before anything is copied.
