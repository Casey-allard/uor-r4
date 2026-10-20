# `panel show` is Rust — the last delegation site has one call site again

**Date:** 2026-10-14 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** the last delegation site (`arena`/`budget`/`panel`), second of its subcommands

## What landed

**`uor-harness panel show` runs in Rust, and the `delegate_to_python` call-site count went 2 → 1**: the
general fallback is the only one left, carrying `arena` x3 and `budget` x2. **Unported commands 6 → 5.**

## Why `show` needed a new type, and `score` did not

`panel show --json` does not print a shape this crate invented. It prints, for each panel, a dict whose
**`bars` value is the panel file's own list of bar objects** — keys in the order the file wrote them
(`id, label, current, target, of, unit, direction, note`), **`note` included.**

Three things made that impossible with `serde_json::Value`:

1. **`Value`'s object is a `BTreeMap`**, so parsing sorts the keys away.
2. **`registries::Bar` has no `note` field at all**, so re-serialising the struct would drop it.
3. Re-serialising would also reorder the keys it does have. **The output would look right and diff
   wrong** — which is the failure mode this port has been avoiding all along.

So `ordered.rs` adds an `Ordered` JSON value that keeps every object's key order at every depth, with
Python's two renderings (compact `", "`/`": "`, and `indent=2`). **The scope is deliberate**: turning on
`serde_json`'s `preserve_order` feature would change *every* `Value` in the crate, including the paths
that rely on the sorting to match Python's `sort_keys=True` writers. This is a separate representation
used only where document order is part of the output. `panels_json` renders the panel dict from the
struct and takes **`bars` verbatim from the file**.

## Verified: both implementations, nine case groups

| case | bytes (rust/python) | result |
|---|---|---|
| `panel show` (all 8 panels) | 6096 / 6096 | MATCH |
| `panel show --json` | **11943 / 11943** | MATCH |
| `--milestone m1` text + json | 1352, 4106 | MATCH |
| `--milestone m2` text + json | 1128, 2014 | MATCH |
| `--milestone m3` text + json | 1119, 1700 | MATCH |
| unknown milestone | exit 2/2, stderr | MATCH |

**The `--json` case is the one that matters**: 11,943 bytes of nested document with the bars' `note`
keys and file order intact.

| check | result |
|---|---|
| `cargo test -p uor-r4-harness` | **468 passed, 0 failed, 6 ignored** (was 461: 5 `ordered` + 2 `show` tests) |
| fmt, clippy `-D warnings` | clean, 0 findings |
| `python3 -m unittest discover -s tests/harness -t .` | exit 0, 0 failure lines |
| `python3 scripts/harness/parity.py` | **exit 0** |
| `python3 scripts/check_no_softmax.py` | OK — 1641, baseline 1641 |
| `delegate_to_python` call sites | **1 — the general fallback only** |

## Stated rather than hidden

* **`no panels found ... in docs/harness/panels/` hard-codes the relative path** that Python computes
  with `_rel(panels_dir, repo)`. They agree for the standard layout; a relocated panels directory would
  print a different string.
* **`no --milestone` selects every panel in file order**, as Python does; the milestone filter is
  `p.milestone.lower() == m.key or p.id == m.panel`, copied rather than tightened.
* **A bar whose `of` is a non-string, non-null value** prints through Python's `str()` — an int prints
  as `40`, not `40.0`, and this port keeps that.

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

**Next:** `arena` (`board`, `compare` 91 lines, `lineage`) then `budget` (`show`, `scan`) — five
subcommands, and `ordered.rs` is now available for both if their `--json` prints a raw document.
