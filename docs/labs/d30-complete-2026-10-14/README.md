# `budget scan` is Rust, and the fallback is GONE — D30 is complete

**Date:** 2026-10-14 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** the last two delegation lines in `crates/uor-r4-harness/src/main.rs`

## What landed

**`uor-harness budget scan` runs in Rust**, and **`delegate_to_python` NO LONGER EXISTS in the crate:** the
general fallback arm was replaced with a plain usage error and the function was deleted. `grep -c
delegate_to_python crates/uor-r4-harness/src/main.rs` is **0**.

**D30's condition is met: `uor-harness` runs with no `delegate_to_python` fallback for live commands**, because
there is no `delegate_to_python` at all. Every one of the 14 top-level commands runs Rust end to end.

## The last unit

`budget scan` needed `resolve_pod_script` (repo -> main checkout -> `$UOR_POD`, reporting **what was checked**
as well as what was found), `parse_pod_status` and `parse_gpu_costs`, and `_emit_scan`'s four payload shapes.
Three details carry the meaning:

* **The four shapes are genuinely different documents.** The not-found case carries the LIST of places checked
  and no `script_source`; the not-executable case carries `script_source` and no attempts; the both-failed
  case adds `status_tail`; only the available case has `caps`, `over_cap` and the cost table. One struct
  serialised one way would invent keys Python never writes.
* **Nothing is fabricated and no pod state is touched.** Only `status` and `gpus` are ever run, both of which
  print; a missing tool, a non-executable one, and a failing pair each produce `UNAVAILABLE` with the attempts
  that led there, rather than a zero.
* **`run_capture` implements Python's subprocess timeout by hand** - Rust's std has none, so it polls
  `try_wait` against a deadline and kills the child. Without that, a hung pod tool would hang the harness,
  which is exactly what `timeout=POD_TIMEOUT_S` exists to prevent.

## Verified: both implementations, two case groups

| case | bytes (rust/python) | result |
|---|---|---|
| `budget scan` | 588 / 588 | MATCH |
| `budget scan --json` | 935 / 935 | MATCH |

Run against the live `scripts/pod/uor-pod`. **The first comparison showed a `bytes` field 13 apart, and that
was NOT a defect**: `uor-pod status` prints live pod state, so the raw output length changes between two
invocations. On a stable tree it matches - the same lesson as the disk figure in the `budget show` unit.

| check | result |
|---|---|
| `cargo test -p uor-r4-harness` | **491 passed, 0 failed, 6 ignored** |
| fmt, clippy `-D warnings` | clean, 0 findings |
| `python3 -m unittest discover -s tests/harness -t .` | exit 0, 0 failure lines |
| `python3 scripts/harness/parity.py` | **exit 0** (it does not depend on the delegation) |
| `python3 scripts/check_no_softmax.py` | OK — 1641, baseline 1641 |
| `delegate_to_python` references in the crate | **0** |

## What D30 was, end to end

At the start of this work the harness had **5 `delegate_to_python` sites covering 6 items**: the general
fallback (`arena`/`budget`/`panel`), `mistakes show|capture`, `multilab sync-live|migrate-ledger`, and `team
roster|dispatch`. Every one of them is now Rust, delivered as its own protected PR with its own record: the
team roster, `check`'s argument parsing, team dispatch, `mistakes show|capture`, the heartbeat interface
items, `multilab sync-live|migrate-ledger`, `panel score`, `panel show`, `arena lineage`, `arena board`,
`arena compare`, `budget show`, and now `budget scan` and the fallback's removal.

**The method was the same every time and it is the reason this record can say "matches":** port the module,
run BOTH implementations on the same inputs, and compare bytes. It found a defect in almost every unit - a
missing D22 sentence, a quoted id, an ordering bug, raw bar objects, a sorted `scores` map, an int-vs-float
band, and Python's decimal rounding against Rust's. None of those were visible by reading the code.

## Stated rather than hidden

* **An unrecognised command is now a usage error**, not a delegation: `uor-harness budgett` prints this
  binary's usage and exits 2, where it used to forward to `cli.py` and print argparse's message. The exit
  code is the same; the text differs, and that is the point - there is no Python to forward to.
* **`scripts/harness/cli.py` still exists** and still runs standalone. It is no longer reachable from
  `uor-harness`, and the five `python3` calls left in the crate are all inside `#[cfg(test)]`: they are the
  live-parity tests that compare the port against the Python it came from.

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

**D30 is complete.** The next unit on this line is the PM queue's items 5-7 (cargo pinning, `claims_file()`,
and the stale PATH install).
