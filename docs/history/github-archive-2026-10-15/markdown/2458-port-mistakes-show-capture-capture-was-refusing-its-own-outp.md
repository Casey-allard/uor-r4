# #2458 — Port mistakes show|capture; capture was refusing its own output (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-14T08:50:19Z
- **Updated:** 2026-10-14T08:52:29Z
- **Closed:** 2026-10-14T08:52:07Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2458
- **Merged:** 2026-10-14T08:52:07Z
- **Merge commit:** efcfa28e8e90b9809db7f2faa41bb6eb220c6b74
- **Base / Head:** main / UOR-Foundation:deepseek/mistakes-write
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21)

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

Decision: **KEEP** (tooling).

## What this delivers

**`uor-harness mistakes show` and `mistakes capture` are Rust, and the `mistakes` delegation site is
gone: live `delegate_to_python` call sites 3 → 2.**

`show` reads one lesson body. **`capture` is the writer:** it validates the draft against the rules
`mistakes.registry_valid` applies, refuses to overwrite without `--force`, writes, reads the file back
and removes it if it somehow fails — so a capture cannot leave behind a lesson that turns the lab's
own `check` red.

## Three defects the comparison found, none visible by reading

1. **`capture` refused every lesson with an auto-generated guard.** Rust's front-matter reader did not
   strip quotes; `build_lesson_text` writes the default guard quoted (it contains a colon); so the
   guard began with a `"` instead of `none`, failed the guard rule, and **the command could not write
   the lesson it had just generated.**
2. **The success message was three lines short.** Python also prints the severity/class/guard summary,
   the `read it back` command and the `check --only mistakes` line. The port printed only the write
   line, because `capture` had been read as far as the write.
3. **Every front-matter value goes through the YAML renderer, not only the awkward ones.** Wrapping
   only the punctuated fields meant a title with a space was written bare where Python quotes it — a
   difference in every captured lesson, found by diffing the written file.

## Two defects in the rule the draft is validated against

`mistakes.registry_valid` was **missing three checks Python has** (`'trigger' needs at least two
phrases`, `'paths' is empty`, `'<key>' must be an inline or block list`) and used Rust's debug
formatting where Python uses `!r` and tuple reprs. **They were invisible because Python's registry
loader refuses a lesson with a bad id or severity before the rule runs** — so the comparison needed a
lesson that loads and is still wrong. With one, **both implementations report the same eight findings,
in the same order, with the same messages.**

## Verification: both implementations on scratch checkouts

Twelve case groups, each comparing stdout, stderr, exit code **and every file in the mistakes
directory**: `show` exact / prefix / unknown / missing id; `capture` minimal, full flags, then-exists,
then-`--force`, single trigger, bad id, missing evidence, missing title.

**All match except one message:** with a required flag absent, Python's argparse prints a usage block
and this port prints `mistakes capture needs --id and --title`. Both refuse with exit 2.

| check | result |
|---|---|
| `cargo test -p uor-r4-harness --offline` | 445 passed, 0 failed, 6 ignored (was 438) |
| `cargo fmt --check`, clippy `-D warnings` | clean, 0 findings |
| `python3 -m unittest discover -s tests/harness -t .` | exit 0, 0 failure lines |
| `python3 scripts/harness/parity.py` | exit 0 |
| `python3 scripts/check_no_softmax.py` | OK — 1641, baseline 1641 |
| `check` rule-id set on the real tree | 41 = 41, no new findings from the added checks |
| live `delegate_to_python` call sites | **2 — the `mistakes` site is gone** |

## Checklist

- [x] own worktree, port with its tests, record in `docs/labs/mistakes-write-port-2026-10-14/README.md`, full verification, protected PR
- [x] both implementations compared on the WRITTEN artifacts (stdout, stderr, exit code and the files)
- [x] the defects are covered by named tests, not comments
- [x] limitations stated below
- [x] no capability claim, no headline movement, no compute

### Stated rather than hidden

* **`check`'s TEXT rendering is not Python's.** Same verdicts, different layout: Python prints
  `harness check: 8 rules, 8 failed (8 error, 0 warn)` then `FAIL <rule> [<path>]: <message>` plus a
  trailing hint line; this port prints a bare count then `  error <rule> [<path>]` with the message on
  its own line. It surfaced only because comparing rule messages required reading both renderings side
  by side. **Not fixed here — it is a `check` unit of its own**, and it is the same class as the
  ledger line: identical verdicts, different bytes, invisible to a JSON comparison.
* **The front-matter parser still does not read block lists.** Nothing in the corpus uses one, and the
  gap is stated in the code rather than hidden; a block-list lesson would read as a missing key where
  Python reads a list.
* **A lesson with a bad id or severity makes Python refuse the whole registry** while this port returns
  the lesson with its errors and lets `check` report them. That is pre-existing and deliberate (an
  unreported broken lesson is worse), and it is why the rule-message comparison needed a lesson that
  loads.
* Missing required flags: the refusal and the exit code are Python's, the message text is not.

References #2028


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6164571163 — 2026-10-14T08:50:27Z — Casey-allard

## Verification at the exact head `d08c9fdd2` (local, this worktree)

```
export CARGO_TARGET_DIR=$PWD/local/harness-target
cargo fmt -p uor-r4-harness -- --check                 -> clean
cargo clippy -p uor-r4-harness --offline --all-targets -- -D warnings -> clean, 0 findings
cargo test -p uor-r4-harness --offline                 -> 445 passed, 0 failed, 6 ignored
python3 -m unittest discover -s tests/harness -t .     -> exit 0, 0 failure lines
python3 scripts/check_no_softmax.py                    -> OK 1641, baseline 1641
python3 scripts/harness/parity.py                      -> exit 0
```

**Both implementations, scratch checkouts, twelve case groups** (stdout, stderr, exit code and every
file in the mistakes directory compared):

```
show exact            MATCH   show prefix         MATCH   show unknown     MATCH   show missing id MATCH
capture writes        MATCH   capture full        MATCH   capture exists   MATCH   capture force   MATCH
capture one trigger   MATCH   capture bad id      MATCH   missing evidence MATCH
capture no title      steps DIFFER (exit 2 both; argparse usage text vs one line)
```

**The rule the draft is validated against, on a lesson that loads and is wrong** (valid id and
severity, everything else wrong) — both implementations, same eight findings, same order, same
messages:

```
front matter is missing 'paths'
class must be one of ('process', 'evidence', 'compute', 'measurement', 'delivery', 'coordination', 'assumption'), got 'nope'
status must be one of ('active', 'resolved', 'superseded'), got 'nope'
date must be YYYY-MM-DD, got '14-10-2026'
guard 'some guard' must be a dotted rule id or 'none: <reason>'
'evidence' must be an inline or block list
'trigger' needs at least two phrases, or the lesson is not retrievable
'paths' is empty: name a glob so the lesson is recalled
```

The `check` rule-id set on the real tree is 41 = 41 with no new findings from the three added checks,
and the only failing-rule difference from Python remains the recorded `status.no_orphan_branches`.



---
_Comment/review ids in this file: 6164571163_

