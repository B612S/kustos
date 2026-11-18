# Repository format

Kustos does not invent a storage format: the repository is a
[restic](https://restic.net) v2 repository written by
[`rustic`](https://rustic.cli.rs). Everything below is the standard layout, so
the data can also be read by `restic` 0.18+ and by `rustic` itself.

```
<repository>/
  config            repository metadata (chunker, version, chunk size)
  keys/             encrypted master keys, one per password
  data/             content-addressed, encrypted, compressed chunks
  index/            chunk index
  snapshots/        one file per snapshot (tree, time, paths, tags)
```

## Encryption

The repository key is derived from the password with `scrypt`; the master key
is stored in `keys/` encrypted with it. Data and metadata are encrypted with
AES-256 (CTR mode with a Poly1305 authenticator). Without the password file the
repository is unreadable — there is no recovery path.

Kustos generates the password on the first run, shows it once and keeps it in
memory only. It is never written to disk: every start asks for it. Without the
key there is no recovery path.

## Compression and deduplication

Files are split into content-defined chunks (Rabin, ~1 MiB average). Each chunk
is hashed, compressed with zstd and stored only once. Identical chunks — the
same file in two snapshots, two copies of a photo, repeated source code — are
stored once. This is why a snapshot of an almost unchanged home directory costs
a few megabytes instead of tens of gigabytes.

## Snapshots

A snapshot is a tree of paths plus metadata:

- paths are relative to the snapshot root: `/home/user/.config` is stored as
  `home/user/.config`;
- Kustos tags every snapshot `kustos` and lists only tagged snapshots;
- the snapshot id is a 64-character hex string; the UI shows the first 8
  characters.

Deleting a snapshot (`rustic forget <id>`) only drops its tree references;
chunks that are still referenced by other snapshots stay.

## Retention

After each successful backup Kustos runs

```
rustic forget --keep-hourly H --keep-daily D --keep-weekly W --prune
```

which keeps the newest snapshot of every hour for `H` hours, then the newest of
every day for `D` days, then the newest of every week for `W` weeks, and prunes
unreferenced chunks. The newest snapshot is always kept.

## Restore semantics

Rustic restores one path at a time. Kustos restores it into a staging
directory (`~/.cache/kustos/restore/…`), then copies it into place with
`cp -a`, preserving mode, owner and timestamps:

- destination `/` (default) — the original location;
- destination folder — the same tree below that folder;
- `--policy skip` keeps an existing target untouched;
- `--policy keep_both` writes alongside as `name (restored <timestamp>)`;
- otherwise the target is replaced.

Hard links inside one snapshot are not recreated on restore; the content is
still stored once in the repository.
