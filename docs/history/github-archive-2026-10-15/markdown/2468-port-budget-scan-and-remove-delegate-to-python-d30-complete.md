# #2468 — Port budget scan and remove delegate_to_python: D30 complete (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-14T10:56:19Z
- **Updated:** 2026-10-14T10:57:41Z
- **Closed:** 2026-10-14T10:57:20Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2468
- **Merged:** 2026-10-14T10:57:20Z
- **Merge commit:** 3e2534eacc66d8fc5ef7dd34ffca349e67b3ed91
- **Base / Head:** main / UOR-Foundation:deepseek/budget-scan
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21)

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

Decision: **KEEP** (tooling).

## What this delivers

**`uor-harness budget scan` is Rust, and `delegate_to_python` no longer exists in the crate.** The general
fallback arm became a plain usage error and the function was deleted; `grep -c delegate_to_python
crates/uor-r4-harness/src/main.rs` is **0**.

**D30's condition is met: no live command reaches Python, because there is no `delegate_to_python` at all —
every top-level command is Rust end to end.**

## The last unit

`budget scan` needed pod resolution (repo → main checkout → `$UOR_POD`, reporting **what was checked** as well
as what was found), the two status parsers, and `_emit_scan`'s **four genuinely different payload shapes** —
a single struct would invent keys Python never writes. **Nothing is fabricated and no pod state is touched:**
only `status` and `gpus` are run, and every failure prints `UNAVAILABLE` with the attempts that led there
rather than a zero. **`run_capture` implements Python's subprocess timeout by hand** (Rust's std has none): a
hung pod tool must not hang the harness.

## Verification: both implementations

| case | bytes (rust/python) | result |
|---|---|---|
| `budget scan` | 588 / 588 | MATCH |
| `budget scan --json` | 935 / 935 | MATCH |

Against the live `scripts/pod/uor-pod`. **The first comparison showed `bytes` 13 apart, and that was not a
defect** — `uor-pod status` prints live pod state, so the raw output length moves between invocations.

| check | result |
|---|---|
| `cargo test -p uor-r4-harness --offline` | 491 passed, 0 failed, 6 ignored |
| fmt, clippy `-D warnings` | clean, 0 findings |
| `python3 -m unittest discover -s tests/harness -t .` | exit 0, 0 failure lines |
| `python3 scripts/harness/parity.py` | exit 0 — it does not depend on the delegation |
| `python3 scripts/check_no_softmax.py` | OK — 1641, baseline 1641 |
| `delegate_to_python` references | **0** |

## D30, end to end

5 delegation sites covering 6 items at the start; every one ported as its own protected PR with its own
record. **The method was the same every time:** port the module, run BOTH implementations on the same inputs,
and compare bytes — which found a real defect in almost every unit (a missing D22 sentence, a quoted id, an
ordering bug, raw bar objects, a sorted `scores` map, an int-vs-float band, and Python's decimal rounding
against Rust's). None of those were visible by reading the code.

## Checklist

- [x] own worktree, port with tests, record at `docs/labs/d30-complete-2026-10-14/README.md` plus a newest-first current-state entry, full verification, protected PR
- [x] both implementations exercised and compared byte for byte, not read back
- [x] limitations stated below

### Stated rather than hidden

* **An unrecognised command is now a usage error**, not a delegation: `uor-harness budgett` prints this
  binary's usage and exits 2, where it used to forward to `cli.py`. The exit code is unchanged; the text
  differs, and that is the point — there is no Python to forward to.
* **`scripts/harness/cli.py` still exists** and still runs standalone; it is simply no longer reachable from
  `uor-harness`. The five `python3` calls left in the crate are all inside `#[cfg(test)]` — the live-parity
  tests that compare the port against the Python it came from.

References #2028


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6166768084 — 2026-10-14T10:56:27Z — Casey-allard

## Verification at the exact head (local, this worktree)

```
CARGO_TARGET_DIR=$PWD/local/harness-target cargo fmt -- --check   -> clean
cargo clippy --all-targets -- -D warnings                         -> clean, 0 findings
cargo test -p uor-r4-harness --offline                            -> 491 passed, 0 failed, 6 ignored
python3 -m unittest discover -s tests/harness -t .                -> exit 0, 0 failure lines
python3 scripts/harness/parity.py                                 -> exit 0
python3 scripts/check_no_softmax.py                               -> OK 1641, baseline 1641
grep -c delegate_to_python crates/uor-r4-harness/src/main.rs      -> 0
```

**Both implementations, against the live `scripts/pod/uor-pod`:**

```
budget scan         588 / 588 bytes   MATCH
budget scan --json  935 / 935 bytes   MATCH
```

**The first comparison showed `bytes` 13 apart, and that was not a defect**: `uor-pod status` prints live pod
state, so the raw output length moves between two invocations. On a stable tree both cases match — the same
lesson the `budget show` unit learned about the disk figure.

**What this closes:** `delegate_to_python` no longer exists in the crate. The general fallback arm is now a
plain usage error (`uor-harness budgett` exits 2 with this binary's usage instead of forwarding to `cli.py`),
and the function is deleted. Every top-level command runs Rust end to end.

**And the method, for the record:** 5 delegation sites covering 6 items at the start, each ported as its own
protected PR with its own record, each verified by running BOTH implementations on the same inputs and
comparing bytes. That found a real defect in almost every unit — a missing D22 sentence, a quoted id, an
ordering bug, raw bar objects, a sorted `scores` map, an int-vs-float band, and Python's decimal rounding
against Rust's. None of them were visible by reading the code.


---
_Comment/review ids in this file: 6166768084_

