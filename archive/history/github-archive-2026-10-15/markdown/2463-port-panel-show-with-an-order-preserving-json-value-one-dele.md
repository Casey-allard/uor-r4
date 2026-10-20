# #2463 — Port panel show with an order-preserving JSON value; one delegation site left (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-14T09:35:14Z
- **Updated:** 2026-10-14T09:37:34Z
- **Closed:** 2026-10-14T09:37:24Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2463
- **Merged:** 2026-10-14T09:37:24Z
- **Merge commit:** f88fca97c8863c04ef0e038ca8bb832c2bb0dde6
- **Base / Head:** main / UOR-Foundation:deepseek/panel-show
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21)

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

Decision: **KEEP** (tooling).

## What this delivers

**`uor-harness panel show` is Rust, and the `delegate_to_python` call-site count went 2 → 1.** The
general fallback is the only one left, carrying `arena` x3 and `budget` x2; unported commands 6 → 5.

## Why `show` needed a new type

`panel show --json` prints, for each panel, a dict whose **`bars` value is the panel file's own list of
bar objects** — keys in the file's order (`id, label, current, target, of, unit, direction, note`),
**`note` included**. `serde_json::Value` is a `BTreeMap` (parsing sorts the keys away), `registries::Bar`
has no `note` field at all, and re-serialising the struct would reorder the rest: **the output would look
right and diff wrong.**

`ordered.rs` adds an `Ordered` JSON value keeping every object's key order at every depth, with Python's
two renderings. **The scope is deliberate**: `serde_json`'s `preserve_order` feature would change every
`Value` in the crate, including the paths that rely on sorting to match Python's `sort_keys=True`
writers. `panels_json` renders the panel dict from the struct and takes `bars` verbatim from the file.

## Verification: both implementations, nine case groups

| case | bytes (rust/python) | result |
|---|---|---|
| `panel show` (all 8 panels) | 6096 / 6096 | MATCH |
| `panel show --json` | 11943 / 11943 | MATCH |
| `--milestone m1/m2/m3`, text and json | 1352/4106, 1128/2014, 1119/1700 | MATCH |
| unknown milestone | exit 2/2 + stderr | MATCH |

| check | result |
|---|---|
| `cargo test -p uor-r4-harness --offline` | 468 passed, 0 failed, 6 ignored (was 461) |
| fmt, clippy `-D warnings` | clean, 0 findings |
| `python3 -m unittest discover -s tests/harness -t .` | exit 0, 0 failure lines |
| `python3 scripts/harness/parity.py` | exit 0 |
| `python3 scripts/check_no_softmax.py` | OK — 1641, baseline 1641 |
| `delegate_to_python` call sites | **1 — the general fallback only** |

## Checklist

- [x] own worktree, port with tests, record at `docs/labs/panel-show-port-2026-10-14/README.md` plus a newest-first current-state entry, full verification, protected PR
- [x] both implementations exercised and compared byte for byte, not read back
- [x] limitations stated below

### Stated rather than hidden

* **`no panels found ... in docs/harness/panels/` hard-codes the relative path** Python computes with
  `_rel(panels_dir, repo)`; they agree for the standard layout.
* **`no --milestone` selects every panel in file order**, as Python does; the filter is
  `p.milestone.lower() == m.key or p.id == m.panel`, copied rather than tightened.
* **A bar whose `of` is a number** prints through Python's `str()`.

References #2028


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6165351962 — 2026-10-14T09:35:21Z — Casey-allard

## Verification at the exact head (local, this worktree)

```
CARGO_TARGET_DIR=$PWD/local/harness-target cargo fmt -- --check   -> clean
cargo clippy --all-targets -- -D warnings                         -> clean, 0 findings
cargo test -p uor-r4-harness --offline                            -> 468 passed, 0 failed, 6 ignored
python3 -m unittest discover -s tests/harness -t .                -> exit 0, 0 failure lines
python3 scripts/harness/parity.py                                 -> exit 0
python3 scripts/check_no_softmax.py                               -> OK 1641, baseline 1641
```

**Both implementations, nine case groups — every one MATCH (stdout, stderr and exit code):**

```
panel show (8 panels)          6096 / 6096 bytes
panel show --json             11943 / 11943 bytes
panel show --milestone m1      1352 / 1352   json 4106 / 4106
panel show --milestone m2      1128 / 1128   json 2014 / 2014
panel show --milestone m3      1119 / 1119   json 1700 / 1700
panel show --milestone nope    exit 2/2, same stderr
```

**The `--json` case is the point of this PR.** It is an 11,943-byte nested document whose `bars` values
are the panel file's own objects, keys in file order and `note` included. `serde_json::Value` sorts them
away, and `registries::Bar` has no `note` field, so this needed `ordered.rs` — an order-preserving JSON
value scoped to the outputs where document order is data. **`serde_json`'s `preserve_order` feature was
deliberately not used**: it would change every `Value` in the crate, including the paths that rely on
sorting to match Python's `sort_keys=True` writers.

**Delegation: call sites 2 → 1**, unported commands 6 → 5. The general fallback now carries `arena` x3
and `budget` x2, and that is the whole of what is left.


---
_Comment/review ids in this file: 6165351962_

