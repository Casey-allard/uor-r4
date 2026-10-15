# #2465 — Port arena board; the empty arena answers before --json (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-14T10:09:27Z
- **Updated:** 2026-10-14T10:16:11Z
- **Closed:** 2026-10-14T10:15:47Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2465
- **Merged:** 2026-10-14T10:15:47Z
- **Merge commit:** d4523264dd49fb01b063310e8f36e2965c2d14c7
- **Base / Head:** main / UOR-Foundation:deepseek/arena-board
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21)

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

Decision: **KEEP** (tooling).

## What this delivers

**`uor-harness arena board` is Rust**, text and `--json`, on the reader that landed with `lineage`.
**Unported commands 4 → 3**; live call sites stay 2 (the general fallback now carries only `budget` x2).

## Two things the comparison caught

1. **The empty arena answers BEFORE `--json`.** Python checks `if not arena.rows` first and prints the
   two-line `no rows recorded yet` message **whichever flag was passed**; the first version of this arm
   checked `json` first and answered `{}`.
2. **`scores` keeps the FILE's key order, and the real data needs it.** `addressed-read` writes
   `wrong_value_failures` before `memory`; `serde_json` would sort that away, so `scores` is read through
   `ordered::Ordered`.

## Verification: both implementations, six case groups

| case | bytes (rust/python) | result |
|---|---|---|
| real arena, text | 981 / 981 | MATCH |
| real arena, `--json` | 4612 / 4612 | MATCH |
| no rows, text | 134 / 134 | MATCH |
| no rows, `--json` | 134 / 134 | MATCH |
| synthetic arena (unsorted scores, padded limits) | 457 / 814 | MATCH |

| check | result |
|---|---|
| `cargo test -p uor-r4-harness --offline` | 475 passed, 0 failed, 6 ignored (was 472) |
| fmt, clippy `-D warnings` | clean, 0 findings |
| `python3 -m unittest discover -s tests/harness -t .` | exit 0, 0 failure lines |
| `python3 scripts/harness/parity.py` | exit 0 |
| `python3 scripts/check_no_softmax.py` | OK — 1641, baseline 1641 |

## Checklist

- [x] own worktree, port with tests, record at `docs/labs/arena-board-port-2026-10-14/README.md` plus a newest-first current-state entry, full verification, protected PR
- [x] both implementations exercised and compared byte for byte, not read back
- [x] limitations stated below

### Stated rather than hidden

* **`board --json` uses `ensure_ascii` ON**, unlike `panel show --json`: Python calls plain
  `json.dumps(..., indent=2)` here and passes `ensure_ascii=False` there. Two calls in one module, two
  behaviours, both copied.
* **Rows are sorted by `date` alone and stably** — this is a lineage in measurement order, not a ranking.
* **`compare` still delegates**; `bar_meta` and `_is_better` are not yet ported.

References #2028


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6165956263 — 2026-10-14T10:09:35Z — Casey-allard

## Verification at the exact head (local, this worktree)

```
CARGO_TARGET_DIR=$PWD/local/harness-target cargo fmt -- --check   -> clean
cargo clippy --all-targets -- -D warnings                         -> clean, 0 findings
cargo test -p uor-r4-harness --offline                            -> 475 passed, 0 failed, 6 ignored
python3 -m unittest discover -s tests/harness -t .                -> exit 0, 0 failure lines
python3 scripts/harness/parity.py                                 -> exit 0
python3 scripts/check_no_softmax.py                               -> OK 1641, baseline 1641
```

**Both implementations, six case groups — every one MATCH:**

```
arena board  (real arena, 5 rows)   981 / 981 bytes
arena board --json                 4612 / 4612 bytes
arena board  (no rows)              134 / 134
arena board --json (no rows)        134 / 134
arena board  (synthetic)            457 / 457
arena board --json (synthetic)      814 / 814
```

The synthetic arena carries an **unsorted `scores` map** (`z_metric` before `a_metric`), a
whitespace-padded `limits`, an absent `parent` and an absent `scores` — the normalisation Python does in
`Candidate.__dict__`. That is the case that proves `scores` must be read through `ordered::Ordered`: a
`BTreeMap` prints `a_metric` first.

**And the empty arena is the case that caught a real defect:** Python checks `if not arena.rows` *before*
it looks at `--json`, so a lab with no rows gets the two-line `no rows recorded yet` message whichever
flag it passed. The first version of this arm checked `json` first and answered `{}` there.


---
_Comment/review ids in this file: 6165956263_

