# #2461 — Port multilab sync-live and migrate-ledger; the last subcommand site (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-14T09:14:53Z
- **Updated:** 2026-10-14T09:16:06Z
- **Closed:** 2026-10-14T09:15:57Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2461
- **Merged:** 2026-10-14T09:15:56Z
- **Merge commit:** 70510dd8b4a4c91d3e8eb6fdc82ba0fa17c01e57
- **Base / Head:** main / UOR-Foundation:deepseek/multilab-sync
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21)

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

Decision: **KEEP** (tooling).

## What this delivers

**`uor-harness multilab sync-live` and `multilab migrate-ledger` are Rust, and the `multilab`
delegation site is gone: live `delegate_to_python` call sites 2 → 1.** The only one left is the general
fallback, which is `arena`/`budget`/`panel`.

**This is also PM queue item 4.** `sync-live` is the committed presence tier's only refresh path: the
durable files are a day stale because the heartbeat writes the **live** tier only, on purpose (a check-in
every few minutes must never dirty a checkout, R10) — so the committed record is refreshed when a lab
chooses to publish it, and until this unit that command was Python.

**The README was wrong and is now right.** It said *"Every heartbeat writes both, from one record"* and
*"the durable write happens regardless"* — neither is true, and the second is the opposite of the design.
Both lines now say what happens, and the section records that the old claim was wrong so the next reader
does not restore it.

**`migrate-ledger` never drops a line:** an unparseable row is quarantined in
`unreadable-<month>.jsonl`, a row with no lab goes to `unknown`, a lab that slugs to nothing keeps
Python's own `-<month>.jsonl`, and every moved row is re-serialised with Python's
`json.dumps(ensure_ascii=False, sort_keys=True)` — a migration must not rewrite the record it moves.

## The defect my own comparison found

The legacy-file matcher required **11** bytes; `2026-10.jsonl` is **13**. The migration therefore matched
nothing, moved nothing, and **reported zero rows moved with exit 0** — a command that looks like it
worked. Python's run produced five shards and this one produced none, which is the only reason it was
visible.

## Verification: both implementations, six case groups

| case | stdout | stderr | exit | files written |
|---|---|---|---|---|
| `sync-live` (two labs, one broken file) | MATCH | MATCH | 0/0 | MATCH |
| `sync-live --json` | MATCH | MATCH | 0/0 | MATCH |
| `sync-live --json`, nothing live | MATCH | MATCH | 0/0 | MATCH |
| `migrate-ledger` (slugged / actor / unknown / unreadable / merged) | MATCH | MATCH | 0/0 | MATCH |
| `migrate-ledger --json` | MATCH | MATCH | 0/0 | MATCH |
| `migrate-ledger`, nothing to migrate | MATCH | MATCH | 0/0 | MATCH |

"files written" compares **every file in the directory** byte for byte — the committed presence tier
for the syncs, the whole ledger directory (including that the legacy file is gone) for the migrations.

| check | result |
|---|---|
| `cargo test -p uor-r4-harness --offline` | 450 passed, 0 failed, 6 ignored (was 446) |
| `cargo fmt --check`, clippy `-D warnings` | clean, 0 findings |
| `python3 -m unittest discover -s tests/harness -t .` | exit 0, 0 failure lines |
| `python3 scripts/harness/parity.py` | exit 0 |
| `python3 scripts/check_no_softmax.py` | OK — 1641, baseline 1641 |
| live `delegate_to_python` call sites | **1 — the general fallback only** |

## Checklist

- [x] own worktree, port with tests, record in `docs/labs/multilab-sync-port-2026-10-14/README.md`, full verification, protected PR
- [x] both implementations run and the WRITTEN files compared byte for byte, not read back
- [x] the defect is covered by a named test (`only_unsharded_month_files_are_legacy`)
- [x] limitations stated below

### Stated rather than hidden

* **Python requires `--json` BEFORE the subcommand** (`multilab sync-live --json` → `unrecognized
  arguments`, exit 2); this binary accepts either position. The divergence already recorded for `check`
  and `git switch`, in the port's favour.
* **A live presence file with `schema` other than 1 would be rewritten as 1** by the sync, because the
  record is rebuilt from the row and this crate's writer stamps the current schema. Every live file in
  use says 1.
* **The live tier stays compact without Python's spaces** (`{"a":1}` vs `{"a": 1}`). Nothing writes
  that tier but this binary now; the committed tier, which is read in a diff, matches Python byte for
  byte.

References #2028


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6164997076 — 2026-10-14T09:15:02Z — Casey-allard

## Verification at the exact head (local, this worktree)

```
CARGO_TARGET_DIR=$PWD/local/harness-target cargo fmt -- --check      -> clean
cargo clippy --all-targets -- -D warnings                            -> clean, 0 findings
cargo test -p uor-r4-harness --offline                               -> 450 passed, 0 failed, 6 ignored
python3 -m unittest discover -s tests/harness -t .                   -> exit 0, 0 failure lines
python3 scripts/harness/parity.py                                    -> exit 0
python3 scripts/check_no_softmax.py                                  -> OK 1641, baseline 1641
```

**Both implementations, six case groups, comparing stdout, stderr, exit code AND every file written:**

```
sync-live text              exit 0/0  out ok  err ok  files ok
sync-live json              exit 0/0  out ok  err ok  files ok
sync-live json (none live)  exit 0/0  out ok  err ok  files ok
migrate text                exit 0/0  out ok  err ok  files ok
migrate json                exit 0/0  out ok  err ok  files ok
migrate nothing             exit 0/0  out ok  err ok  files ok
ALL MATCH
```

The `migrate-ledger` case is the interesting one: a legacy `2026-10.jsonl` holding a slugged lab
(`Kimi` → `kimi`), an `actor`-named row, a row with no lab at all, **a line that is not JSON**, and a
second row for the first lab, merged into an **existing** shard. Both sides produce the same five
shards, with the same ordering by `ts`, the same quarantined line in `unreadable-2026-10.jsonl`, and
the legacy file removed.

**Before the fix that comparison read:** Rust `moved` empty, legacy file untouched, `migrate-ledger`
reporting `migrated 0 receipt(s)` with exit 0 — the matcher required 11 bytes and `2026-10.jsonl` is 13.

**Committed-file shape, after:** the committed presence tier is written pretty (Python's `_save`), the
live tier compact (Python's `publish_live`), and a synced row keeps its **recorded** timestamp rather
than being restamped.


---
_Comment/review ids in this file: 6164997076_

