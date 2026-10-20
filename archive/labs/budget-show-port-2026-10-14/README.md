# `budget show` is Rust — and the last delegation site is down to `budget scan`

**Date:** 2026-10-14 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** the last delegation site; `budget show` was the large half of it

## What landed

**`uor-harness budget show` runs in Rust**, text and `--json`, over a faithful port of `collect()` and its
eight helpers (`_num`, `_read_jsonl_tolerant`, `_sum_by`, `_fmt_map`, `_chain`, `_latest_cumulative`,
`_receipt_costs`, `disk_free`, `pod_ledger_spend`, `resource_ledger`, `model_time_ledger`). **Unported
commands 2 → 1: only `budget scan` still delegates.**

## The comparison found three defects, and the third was arithmetic

**1. The owner's band is a tuple of INTEGERS.** Rust printed `owner band 30.0-70.0 GB` and
`"band_gb": [30.0, 70.0]`; Python writes `30-70` and `[30, 70]`. `DISK_BAND_GB` is now `(i64, i64)`, and
the reason is in the constant.

**2. An empty cost ledger sums to the INTEGER `0`.** Python's `round(sum([]), 4)` is `0`, an int, so the
JSON prints `0` and not `0.0`. The type depends on whether any cost record exists, and the JSON shows it.

**3. Python's `round()` is not Rust's `round()`, and on this ledger that is worth 0.0011.** The pod total
came out `371.9637` against Python's `371.9626`: 211 down-events are each rounded to four decimals *as they
accumulate*, and Rust's round-half-away-from-zero disagrees with Python's tie handling. `round_ties_even`
moved it to `371.9625` — still one tie off, because multiplying by 10^4 first introduces binary error
before the tie is seen. What matches is **rounding the decimal**: `format!("{:.4}", x).parse()`, correctly
rounded on the decimal value, ties to even. That lands on `371.9626`.

**The first run of the bar showed a fourth `DIFF` that was not a defect**: free disk space moves while a
build writes artifacts, so comparing a live `df` figure between two invocations is only valid on a quiet
tree. On a quiet tree the comparison is clean.

## Verified: both implementations, three case groups

| case | bytes (rust/python) | result |
|---|---|---|
| `budget show` | 1417 / 1417 | MATCH |
| `budget show --json` | 3164 / 3164 | MATCH |
| `budget` (no action, which is `show`) | 1417 / 1417 | MATCH |

Run against the real repo: 352 ledger records, 12 resource ledgers, a 2396-record pod ledger and the real
disk figure. `budget scan` stays delegated and still announces itself.

| check | result |
|---|---|
| `cargo test -p uor-r4-harness` | **491 passed, 0 failed, 6 ignored** (was 481: 10 new tests) |
| fmt, clippy `-D warnings` | clean, 0 findings |
| `python3 -m unittest discover -s tests/harness -t .` | exit 0, 0 failure lines |
| `python3 scripts/harness/parity.py` | **exit 0** |
| `python3 scripts/check_no_softmax.py` | OK — 1641, baseline 1641 |

## Stated rather than hidden

* **`disk_free` uses `df -k`.** Rust's std has no `disk_usage` and this crate depends only on
  `regex`/`serde`/`serde_json`; `df` gives the same numbers (`Avail` is `shutil`'s `free`) without a new
  dependency or an `unsafe` block. The band comparisons use the **unrounded** figure, because rounding
  first would flip a boundary case exactly at the floor.
* **`parse_ts` assumes UTC for a naive timestamp**, as Python's `_parse_ts` does — the pod durations were
  computed with that assumption, so "fixing" it would shift every closed-pod cost.
* **A rate is never summed as spend**: `usd_per_h` is reported separately, and a KILL with no rate and no
  remembered rate is skipped rather than charged at zero.
* **`budget scan` still delegates**, and it shells out to `scripts/pod/uor-pod`, so it needs its own unit.

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

**Next:** `budget scan` — **the last live command** — and then the general fallback can be deleted and D30
closed.
