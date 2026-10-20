# #2452 — Port team roster; two recorded findings corrected by measurement (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-14T08:06:10Z
- **Updated:** 2026-10-14T08:07:44Z
- **Closed:** 2026-10-14T08:07:21Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2452
- **Merged:** 2026-10-14T08:07:21Z
- **Merge commit:** 64237ffe6bd1f0dc787230d2b90bb6bcc4b56abc
- **Base / Head:** main / UOR-Foundation:deepseek/team-roster
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21)

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

This is a tooling unit: it moves no acceptance number, and it claims none.

Decision: **KEEP** (tooling).

## What this delivers

`uor-harness team roster` is Rust. **Its output is byte-identical to Python's** — 1,936 bytes for the
text form and 6,017 for `--json`, `diff` empty on both. The `--json` form matched only after the
non-ASCII escape fix below.

Canonical record: `docs/labs/team-roster-port-2026-10-14/README.md`, with a newest-first entry in
`docs/integration/current-state.md`.

## Measured evidence

| check | result |
|---|---|
| `team roster` vs Python | byte-identical, 1,936 = 1,936 |
| `team roster --json` vs Python | byte-identical, 6,017 = 6,017 |
| live comparison test (`#[ignore]`d) | passes |
| `cargo test -p uor-r4-harness --offline` | 425 passed, 0 failed, 5 ignored (was 411/4) |
| `cargo fmt --check` | clean |
| `cargo clippy --all-targets -- -D warnings` | clean, 0 findings |
| `python3 -m unittest discover -s tests/harness -t .` | exit 0 |
| `python3 scripts/harness/parity.py` | exit 0 |
| `python3 scripts/check_no_softmax.py` | OK — 1641 remaining, baseline 1641 |
| `delegate_to_python` sites in `main.rs` | 5 — unchanged in count; line 1582 is `dispatch` alone |

## Two recorded findings corrected by measurement

**1. The recorded blank line does not exist.** The findings record for the earlier, reverted attempt
says Python's header is followed by a blank line. `python3 scripts/harness/cli.py team roster | head -c 80`
returns `b'9 dispatchable roles (docs/harness/roles.json)\n  architecture-reviewer    planne'` — one
newline. Python's `emit` writes the text as-is when it already ends in `\n`. A port that trusted the
record would have shipped a blank line; only the bytes settle it.

**2. `json.dumps` escapes non-ASCII and `serde_json` does not.** Found by diffing, not by reading: after
the text form matched byte for byte, the JSON differed in exactly 12 bytes, because three role purposes
carry an em dash. Python emits `\u2014`; the port emitted the raw character. The document is now escaped
as Python escapes it, astral characters as a surrogate pair.

The order is the file's order, which is why the first attempt was reverted: `roles.json` is an object
keyed by id and Python prints file order, while `serde_json`'s default `Value` is a `BTreeMap` that sorts
it. The loader collects map entries in document order, and the JSON is serialised from a struct rather
than through `serde_json::json!`, which routes through `to_value` and would sort each role's fields too.

## Found while verifying, NOT fixed here

**`uor-harness --repo DIR check` runs ZERO rules**, on `main` as well as on this branch — the pre-change
binary reproduces it:

```
uor-harness check --json                  -> "checked": 52
uor-harness --repo $PWD check --json      -> "checked": 0
uor-harness --repo $PWD check             -> 0 rules, 0 failed (0 error, 0 warn)
python3 scripts/harness/cli.py --repo $PWD --json check -> checked 59, failed 20
```

`--repo` is forwarded into the subcommand argument list, and `cmd_check_rules` reads that list as its
`--only` filter, so a path is taken as a filter that matches no rule id. Controls: `--only arena` checks
4, `--only zzz` checks 0. It is the next unit rather than this one, because the fix is in argument
handling, not in `roster`, and every later unit verifies through `check`.

## Checklist

- [x] own worktree, port with its tests, record, full verification, protected PR
- [x] canonical dated record in `docs/labs/` and a newest-first `current-state.md` entry
- [x] Python output compared byte for byte, not inferred from reading either implementation
- [x] `--json` and text forms both diffed against Python, not only the human-readable one
- [x] limitations stated below rather than left to be rediscovered
- [x] no capability claim, no headline movement, no compute

### Stated rather than hidden

* The ported `str()` covers scalar JSON types; a list or object in a text field would render as JSON text
  rather than Python's `repr`. A test asserts on the real registry that every text field is a string, so
  the limit cannot be crossed silently — but it is an assertion about a file other people edit.
* A missing `roles.json` reads as an empty roster; a broken one is an error. That is a deliberate
  divergence in the error text only: Python raises a traceback, this prints the path and the parse error,
  and both refuse to print `0 dispatchable roles`.
* **The non-ASCII divergence may not be confined to `roster`.** Any ported `--json` path whose data
  carries non-ASCII has the same divergence unless its own comparison covered such a value. Not measured
  here.
* The Python unittest suite exited 1 once during this unit without a recorded failure name; three
  re-runs were clean with zero failure lines. Consistent with the documented intermittent flake under
  concurrent lab activity, and it did not reproduce.
* `team dispatch` still delegates, deliberately: it also WRITES a receipt, so porting it means porting
  `receipts.record` with its redaction, lab attribution and unattributed warning.

References #2028


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6163835037 — 2026-10-14T08:06:24Z — Casey-allard

## Verification at the exact head `6306a43a4` (local, this worktree)

```
export CARGO_TARGET_DIR=$PWD/local/harness-target
cargo fmt -p uor-r4-harness -- --check                 -> clean
cargo clippy -p uor-r4-harness --offline --all-targets -- -D warnings -> clean, 0 findings
cargo test -p uor-r4-harness --offline                 -> 425 passed, 0 failed, 5 ignored
cargo test -p uor-r4-harness --offline --lib -- --ignored roster_matches_python -> ok
python3 -m unittest discover -s tests/harness -t .     -> exit 0
python3 scripts/check_no_softmax.py                    -> OK 1641, baseline 1641
python3 scripts/harness/parity.py                      -> exit 0
```

**Byte comparison against Python, from this worktree's build:**

```
uor-harness --repo $PWD team roster        > rust-roster.txt   # 1936 bytes
python3 scripts/harness/cli.py team roster > py-roster.txt     # 1936 bytes
diff py-roster.txt rust-roster.txt                             # empty
uor-harness --repo $PWD --json team roster > rust-roster.json  # 6017 bytes
python3 scripts/harness/cli.py --json team roster > py-roster.json # 6017 bytes
diff py-roster.json rust-roster.json                           # empty
```

`uor-harness team roster` prints no delegation notice; `uor-harness team dispatch …` still prints one,
so the line shrank from `roster | dispatch` to `dispatch` alone.

**The `--repo`/`check` finding in the body is reproducible on `main` with the pre-change binary in this
same checkout** (`/Users/casey.allard/uor-r4/local/harness-target/debug/uor-harness`, built 01:40 from
`a83232d4e`): `--repo $PWD check --json` reports `"checked": 0` there too. It is recorded, not fixed here.



---
_Comment/review ids in this file: 6163835037_

