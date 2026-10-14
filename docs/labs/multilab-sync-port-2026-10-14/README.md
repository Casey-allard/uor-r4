# `multilab sync-live|migrate-ledger` are Rust — and this is PM queue item 4

**Date:** 2026-10-14 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** the fourth of the six remaining items on the measured scope (#2028), which is also
item 4 of the PM queue for this lab (claude/review1's note, 6163960613)

## What landed

**`uor-harness multilab sync-live` and `multilab migrate-ledger` run in Rust, and the `multilab`
delegation site is GONE: live `delegate_to_python` call sites 2 → 1.** The only one left is the
general fallback, which is `arena`/`budget`/`panel`.

**`sync-live` is the committed presence tier's only refresh path**, and that is what makes it PM item 4:
the durable presence files are a day stale because **the heartbeat writes the live tier only, on
purpose** (a check-in every few minutes must never dirty a checkout, R10) — so the committed record is
refreshed when a lab chooses to publish it. Until this unit that command was Python.

**`migrate-ledger` moves pre-sharding ledgers into per-lab shards, and it never drops a line.** A row
that cannot be parsed is quarantined in `unreadable-<month>.jsonl`, a row with no lab goes to
`unknown`, and a lab that slugs to nothing keeps Python's own `-<month>.jsonl`. Every migrated row is
re-serialised with Python's `json.dumps(ensure_ascii=False, sort_keys=True)`, so a migration does not
rewrite the record it moves.

## The README was wrong, and is now right

`docs/harness/README.md` said **"Every heartbeat writes both, from one record"** and "the durable write
happens regardless". Neither is true, and the second is the opposite of the design. Both lines now say
what happens: a heartbeat writes the live tier and the live receipt from one record; **the committed
record is refreshed deliberately by `multilab sync-live`**, which copies each live row — **timestamp,
intent and session included, not restamped** — into the committed tier. The section says the old claim
was wrong, so the next reader does not restore it.

## Two formats, because Python has two

| tier | Python | shape |
|---|---|---|
| live (`$UOR_HARNESS_LIVE/presence/`) | `publish_live` → `json.dumps(..., sort_keys=True)` | compact, `{"a": 1}` |
| committed (`docs/harness/presence/`) | `_save` → `json.dumps(..., indent=2, sort_keys=True)` | pretty |

**The committed tier is read in a git diff**, so writing it compact would show every line of every
lab's row as changed. Both are now rendered Python's way.

## The defect my own comparison found

The first version of the legacy-file matcher required **11** bytes; `2026-10.jsonl` is **13**. So
`migrate-ledger` matched no files at all, moved nothing, and **reported zero rows moved with exit 0** —
a command that looks like it worked. The comparison against Python showed it immediately, because
Python's run had five shards and this one had none. **It is recorded in the code as well as here.**

## Verified: both implementations, six case groups

| case | stdout | stderr | exit | files written |
|---|---|---|---|---|
| `sync-live` (two labs live, one broken file) | MATCH | MATCH | 0/0 | MATCH |
| `sync-live --json` | MATCH | MATCH | 0/0 | MATCH |
| `sync-live --json`, nothing live | MATCH | MATCH | 0/0 | MATCH |
| `migrate-ledger` (5 shards: slugged, actor-named, unknown, unreadable, merged) | MATCH | MATCH | 0/0 | MATCH |
| `migrate-ledger --json` | MATCH | MATCH | 0/0 | MATCH |
| `migrate-ledger`, nothing to migrate | MATCH | MATCH | 0/0 | MATCH |

"files written" means **every file in the directory**, compared byte for byte — the committed presence
tier for the syncs, and the whole ledger directory (including that the legacy file is gone) for the
migrations.

| check | result |
|---|---|
| `cargo test -p uor-r4-harness` | **450 passed, 0 failed, 6 ignored** (was 446: 4 new tests) |
| `cargo fmt --check`, clippy `-D warnings` | clean, 0 findings |
| `python3 -m unittest discover -s tests/harness -t .` | exit 0, 0 failure lines |
| `python3 scripts/harness/parity.py` | **exit 0** |
| `python3 scripts/check_no_softmax.py` | OK — 1641, baseline 1641 |
| live `delegate_to_python` call sites | **1 — the general fallback only** |

## Stated rather than hidden

* **Python requires `--json` BEFORE the subcommand** (`multilab sync-live --json` is `unrecognized
  arguments`, exit 2); this binary accepts it in either position. The divergence already recorded for
  `check` and `git switch`, in the port's favour.
* **A live presence file with `schema` other than 1 would be rewritten as 1** by the sync, because the
  record is rebuilt from the row and this crate's writer stamps the current schema. Every live file in
  use says 1.
* **The live tier is written compact without Python's spaces** (`{"a":1}` rather than `{"a": 1}`).
  Since the Python heartbeat is now a transport to this binary, nothing writes that tier but Rust; the
  committed tier, which is read in a diff, does match Python byte for byte.

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

**Next:** `arena`/`budget`/`panel` behind the general fallback — the last delegation site — then the
recorded follow-ups (`check`'s text rendering, `status.no_orphan_branches`, the receipt `ts`), and PM
queue items 5–7.
