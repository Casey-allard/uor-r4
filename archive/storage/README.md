# iCloud cold archive: index and gaps

From 20 October 2026, the iCloud folder `UOR-R4/results/` is a read-only cold archive. Nothing new is added to it. New results go to the private HF store `caseyallard/uor-r4-store` under `results/<lab>/<topic-date>/`, with a `SHA256SUMS`. See [Workspace, storage and hygiene](../../docs/labs/workspace.md).

## What is in this folder

| File | What it is |
|---|---|
| `icloud-index-2026-10-20.tsv` | The index of the archive: one row per tar, sorted by remote path. Columns: name, bytes, md5, remote path, uploaded UTC, lab. Built on 20 October 2026 from the small per-object index files that `cloud-store` kept under `UOR-R4/index/`. |
| `icloud-unindexed-2026-10-20.tsv` | The gaps. Columns: path, bytes, status, note. Status `no-index-row` is a file under `results/` that has no valid row in the index (31 files). Status `missing-file` is an index row whose tar is not under `results/` (42 rows). |

## What the index holds

The index lists 1,076 objects, 197.8 GB in all. Bytes per lab, computed from `icloud-index-2026-10-20.tsv`:

| Lab | Objects | GB |
|---|---|---|
| workspace-2026-10-01 | 42 | 66.8 |
| deepseek | 375 | 47.4 |
| codex | 509 | 41.8 |
| claude | 101 | 22.9 |
| archive | 25 | 7.1 |
| antigravity | 1 | 6.0 |
| shared | 3 | 3.3 |
| kimi | 11 | 1.4 |
| owner-docs | 3 | 1.0 |
| answer-span, codex-m2-bank-transfer-d22-20261010, memory-v4, span-extract | 6 | under 0.1 |

## About the two gap files

- The 31 `no-index-row` files are the `deepseek/*-report-root.tar` roots. Their old index rows were written in a different format (no lab column, size in KB, absolute path), so they are left out of the main index. The tars themselves are present.
- The 42 `missing-file` rows are the `workspace-2026-10-01` backups. They were indexed with a path outside `results/` (`icloud:X10 Pro backup 2026-10-01/workspace/...`). The tars exist at that path, so they are listed in the index but not under `results/`.
- The old per-object index also held one stray header line inside a checkpoint file. It is dropped, so the index has 1,076 rows, not 1,077.

## Fetch one object

```sh
~/.local/share/uor-r4/bin/cloud-store fetch <name> <dest>
```

This downloads `<name>.tar`, checks its md5 against the index, and extracts it into `<dest>`. Use `cloud-store list [pattern]` to find a name. Do not use `cloud-store put`: the archive is read-only.
