# #2467 — Port budget show; only budget scan still delegates (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-14T10:49:10Z
- **Updated:** 2026-10-14T10:50:50Z
- **Closed:** 2026-10-14T10:50:35Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2467
- **Merged:** 2026-10-14T10:50:35Z
- **Merge commit:** 268d646aa5f59c52f53b4d87124db9382a36582a
- **Base / Head:** main / UOR-Foundation:deepseek/budget
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21)

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

Decision: **KEEP** (tooling).

## What this delivers

**`uor-harness budget show` is Rust**, text and `--json`, over a faithful port of `collect()` and its eight
helpers. **Unported commands 2 → 1: only `budget scan` still delegates.**

## The comparison found three defects, and the third was arithmetic

1. **The owner's band is a tuple of integers**: Rust printed `owner band 30.0-70.0 GB` and
   `"band_gb": [30.0, 70.0]` where Python writes `30-70` and `[30, 70]`.
2. **An empty cost ledger sums to the integer `0`** — Python's `round(sum([]), 4)` is an int, so the JSON
   prints `0`, not `0.0`.
3. **Python's `round()` is not Rust's `round()`, and on this ledger that is worth 0.0011.** The pod total
   was `371.9637` against Python's `371.9626`: 211 down-events are each rounded to four decimals *as they
   accumulate*, and Rust rounds a tie away from zero while Python rounds to even. `round_ties_even` reached
   `371.9625`; **rounding the decimal** — `format!("{:.4}", x).parse()` — lands on `371.9626`.

A fourth `DIFF` in the first run was not a defect: free disk space moves while a build writes artifacts.

## Verification: both implementations, three case groups

| case | bytes (rust/python) | result |
|---|---|---|
| `budget show` | 1417 / 1417 | MATCH |
| `budget show --json` | 3164 / 3164 | MATCH |
| `budget` (no action, which is `show`) | 1417 / 1417 | MATCH |

Against the real repo: 352 ledger records, 12 resource ledgers, a 2396-record pod ledger, the real disk
figure.

| check | result |
|---|---|
| `cargo test -p uor-r4-harness --offline` | 491 passed, 0 failed, 6 ignored (was 481) |
| fmt, clippy `-D warnings` | clean, 0 findings |
| `python3 -m unittest discover -s tests/harness -t .` | exit 0, 0 failure lines |
| `python3 scripts/harness/parity.py` | exit 0 |
| `python3 scripts/check_no_softmax.py` | OK — 1641, baseline 1641 |

## Checklist

- [x] own worktree, port with tests, record at `docs/labs/budget-show-port-2026-10-14/README.md` plus a newest-first current-state entry, full verification, protected PR
- [x] both implementations exercised and compared byte for byte, not read back
- [x] limitations stated below

### Stated rather than hidden

* **`disk_free` uses `df -k`** — Rust's std has no `disk_usage` and this crate depends only on
  `regex`/`serde`/`serde_json`; `df` gives the same numbers without a new dependency or an `unsafe` block.
  The band comparisons use the **unrounded** figure, since rounding first would flip the floor case.
* **`parse_ts` assumes UTC for a naive timestamp**, as Python does; the pod durations were computed with
  that assumption, so changing it would shift every closed-pod cost.
* **A rate is never summed as spend**, and a down event with no rate and no remembered rate is skipped
  rather than charged at zero.
* **`budget scan` still delegates**; it shells out to `scripts/pod/uor-pod` and needs its own unit.

References #2028


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6166650085 — 2026-10-14T10:49:19Z — Casey-allard

## Verification at the exact head (local, this worktree)

```
CARGO_TARGET_DIR=$PWD/local/harness-target cargo fmt -- --check   -> clean
cargo clippy --all-targets -- -D warnings                         -> clean, 0 findings
cargo test -p uor-r4-harness --offline                            -> 491 passed, 0 failed, 6 ignored
python3 -m unittest discover -s tests/harness -t .                -> exit 0, 0 failure lines
python3 scripts/harness/parity.py                                 -> exit 0
python3 scripts/check_no_softmax.py                               -> OK 1641, baseline 1641
```

**Both implementations, three case groups, against the real repo:**

```
budget show               1417 / 1417 bytes  MATCH
budget show --json        3164 / 3164 bytes  MATCH
budget (no action)        1417 / 1417 bytes  MATCH
```

That tree carries 352 ledger records, 12 `resource-ledger.json` files, a 2396-record pod ledger and the
live disk figure.

**The three defects the comparison caught** — the first two are output shape, the third is arithmetic:

1. `owner band 30.0-70.0 GB` / `"band_gb": [30.0, 70.0]` where Python writes `30-70` / `[30, 70]`, because
   its band is a tuple of **ints**.
2. An empty cost ledger's `usd_total` printed `0.0`; Python's `round(sum([]), 4)` is the **int** `0`.
3. **The pod total: `371.9637` against Python's `371.9626`.** 211 down-events are each rounded to four
   decimals *as they accumulate*, and Rust rounds a tie away from zero where Python rounds to even.
   `round_ties_even` reached `371.9625` — still one tie off, because multiplying by 10⁴ introduces binary
   error before the tie is seen. **Rounding the decimal** (`format!("{:.4}", x).parse()`) is what matches.

**One `DIFF` in the first bar run was not a defect**: free disk space moves while a build writes artifacts,
so a live `df` figure can only be compared on a quiet tree. Re-run there, the comparison is clean — and the
record says so rather than hiding the first result.


---
_Comment/review ids in this file: 6166650085_

