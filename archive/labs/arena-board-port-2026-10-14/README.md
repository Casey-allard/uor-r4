# `arena board` is Rust — and the empty arena answers before `--json` does

**Date:** 2026-10-14 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** the last delegation site (`arena`/`budget`), second of `arena`'s three subcommands

## What landed

**`uor-harness arena board` runs in Rust**, text and `--json`, on the reader that landed with `lineage`.
**Unported commands 4 → 3** (`arena compare`, `budget show`, `budget scan`); **live call sites stay 2**
(the general fallback and the one `arena compare` line) — the fallback now carries only `budget` x2.

## The two things the comparison caught

**1. The empty arena answers BEFORE `--json`.** Python checks `if not arena.rows` first and returns the
two-line `no rows recorded yet` message **whichever flag was passed**; the first version of this arm
checked `json` first and answered `{ ... }` where Python answers with the sentence. **A lab with no
rows is the most likely reader of this command**, and it was the one case the shape got wrong.

**2. `scores` keeps the FILE's key order, and the real data needs it.** `addressed-read` writes
`wrong_value_failures` before `memory`, and `starling-tree` writes `hit_rate, recall,
candidates_per_query`. `serde_json`'s object is a `BTreeMap` and would sort both away, so `scores` is
read through `ordered::Ordered` — the same type `panel show --json` needed for its raw bars.

## Verified: both implementations, six case groups

| case | bytes (rust/python) | result |
|---|---|---|
| `arena board` (real arena, 5 rows) | 981 / 981 | MATCH |
| `arena board --json` | **4612 / 4612** | MATCH |
| `arena board` (no rows) | 134 / 134 | MATCH |
| `arena board --json` (no rows) | 134 / 134 | MATCH |
| a synthetic arena: unsorted `scores`, whitespace-heavy `limits`, missing fields | 457 / 814 | MATCH |

The synthetic case exercises the normalisation Python does in `Candidate.__dict__`: an absent `parent`
renders `null` (and `-` in the text form), an absent `scores` renders `{}`, and `limits` is
whitespace-collapsed and cut to 300 code points on an ellipsis.

| check | result |
|---|---|
| `cargo test -p uor-r4-harness` | **475 passed, 0 failed, 6 ignored** (was 472: 3 new tests) |
| fmt, clippy `-D warnings` | clean, 0 findings |
| `python3 -m unittest discover -s tests/harness -t .` | exit 0, 0 failure lines |
| `python3 scripts/harness/parity.py` | **exit 0** |
| `python3 scripts/check_no_softmax.py` | OK — 1641, baseline 1641 |

## Stated rather than hidden

* **`board --json` uses `ensure_ascii` ON**, unlike `panel show --json`: Python calls plain
  `json.dumps(..., indent=2)` here, whose default escapes non-ASCII, while `show_panel` passes
  `ensure_ascii=False` explicitly. Two calls in one module, two behaviours, both copied.
* **The rows are sorted by `date` alone and stably.** That is the point of the table — a lineage in the
  order things were measured, not a ranking; ties keep file order.
* **`compare` still delegates**, and the two commands it will need (`bar_meta`, `_is_better`) are not
  yet ported.

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

**Next:** `arena compare` (91 lines), then `budget show` (which sits behind `collect()`, ~400 lines
including the resource-ledger pointer-chain walk), then `budget scan`. Three subcommands left before
the general fallback can be deleted.
