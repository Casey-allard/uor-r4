# #2455 — check parsed its arguments as a rule filter, so --repo ran zero rules (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-14T08:22:44Z
- **Updated:** 2026-10-14T08:25:17Z
- **Closed:** 2026-10-14T08:24:53Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2455
- **Merged:** 2026-10-14T08:24:53Z
- **Merge commit:** f56db3b32adb802fbfdd1d6cc408c79ac79263af
- **Base / Head:** main / UOR-Foundation:deepseek/check-flags
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21)

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

Decision: **KEEP** (tooling).

## The defect

**`uor-harness --repo DIR check` checked NOTHING and printed `0 rules, 0 failed (0 error, 0 warn)`.**

```
uor-harness check --json               -> "checked": 52
uor-harness --repo $PWD check --json   -> "checked": 0
uor-harness --repo $PWD check          -> 0 rules, 0 failed (0 error, 0 warn)
controls: --only arena -> 4 rules;  --only zzz -> 0 rules
```

`--repo` is forwarded into the subcommand argument list on purpose (`receipt` needs to know where it
writes), and `cmd_check_rules` handed that same list to `checkdispatch::run` as the `--only` prefix
set. Every token became a filter, no rule id starts with `--repo`, and **the output was
indistinguishable from a clean repository.** The pre-change binary reproduces it, so it was live on
`main`. This is the third time this port has found the same shape — after `cmd_check` running only the
format contract, and three rules registered and never dispatched — and the first time the trigger was
an invocation a lab would type by habit.

## What changed

* `checkdispatch::parse_args` parses `check`'s own flags: `--only` with Python's
  `action="append"`/`nargs="+"` flattening, and `--staged`. The forwarded `--repo`/`--lab` are
  consumed; anything else is a usage error at exit 2, because an unknown token must never be a prefix
  that happens to match nothing.
* `Scope`/`run_scoped` make `--staged` what Python makes it: a rule SET, not a filter over results.
  `STAGED_RULES` is empty today because every rule declares the default `repo` scope, and an
  `#[ignore]`d live test compares it against `checkrules.rules(scope="staged")` so a staged rule added
  in Python fails the test instead of being skipped silently.
* `--staged` was missing from `SUBCOMMAND_FLAGS` entirely, so the top-level parser rejected it before
  `check` saw it. Found by running the flag, not by reading the list.

## Verification: both implementations, seven invocations

| invocation | rule ids | records | exit |
|---|---|---|---|
| `check --json` | **41 = 41** | 51 / 58 | 0 / 0 |
| `--repo DIR check --json` | **41 = 41** (was **0**) | 51 / 58 | 0 / 0 |
| `check --staged --json` | **0 = 0** | 0 / 0 | 0 / 0 |
| `--only arena` | **4 = 4** | 4 / 4 | 0 / 0 |
| `--only prep budget` | **8 = 8** | 13 / 13 | 0 / 0 |
| `--only prep --only budget` | **8 = 8** | 13 / 13 | 0 / 0 |
| `check --bogus`, `check nonsense` | refused, token named | — | **2 / 2** |

The only disagreement is the number of result records when nothing is filtered (**51 here, 58 in
Python**), which **predates this unit**: Python counts PR-body gates that a `check` with no body
cannot answer.

| check | result |
|---|---|
| `cargo test -p uor-r4-harness --offline` | 432 passed, 0 failed, 6 ignored (was 425/5) |
| the `#[ignore]`d staged-set test | passes |
| `cargo fmt --check`, clippy `-D warnings` | clean, 0 findings |
| `python3 -m unittest discover -s tests/harness -t .` | exit 0, 0 failure lines |
| `python3 scripts/harness/parity.py` | exit 0 |
| `python3 scripts/check_no_softmax.py` | OK — 1641, baseline 1641 |

## Found while fixing, NOT fixed here

**`status.no_orphan_branches` fires in Python and reports `ok` in Rust:** Python warns `local branch
bpb-split still exists but PR #2081 merged 2026-10-09`; the ported rule covers the local half only,
while Python reads `branch_pr_check` — a merged-PR lookup, and `bpb-split` was squash-merged, leaving
no ancestry link — and remote branches with no PR. **An absent rule and a passing rule look identical
in output**, which is the sentence this port wrote when it fixed `heartbeat_receipts` and
`registry_vs_github`. It is its own unit (the `gh` chain plus the remote-branch list), not this one.

## Checklist

- [x] own worktree, fix with its tests, record in `docs/labs/check-flags-2026-10-14/README.md`, full verification, protected PR
- [x] both implementations run and compared on every invocation the fix touches
- [x] the regression is a named test (`a_forwarded_repo_flag_narrows_nothing`), not a comment
- [x] limitations stated below
- [x] no capability claim, no headline movement, no compute

### Stated rather than hidden

* **The Rust form accepts `check --repo DIR`, which Python's argparse rejects** with `unrecognized
  arguments` and exit 2. Consuming the forwarded global flags is deliberate; the exit-2 refusal is
  kept for every token that is not one of `check`'s flags or a forwarded global. A divergence in the
  port's favour, stated rather than hidden.
* `STAGED_RULES` being empty is a fact about Python's registry today, checked by a live test — not a
  hardcoded "staged runs nothing".
* `check`'s JSON container shape still differs from Python's (`{contract, formats, rules}` here versus
  `{summary, results}` there). **This unit compares rule-id sets, which is the property it changes**;
  the container difference is pre-existing and untouched.
* The record-count difference (51 vs 58) is pre-existing and not closed here.

References #2028


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6164108219 — 2026-10-14T08:22:52Z — Casey-allard

## Verification at the exact head `f5f4c2f29` (local, this worktree)

```
export CARGO_TARGET_DIR=$PWD/local/harness-target
cargo fmt -p uor-r4-harness -- --check                 -> clean
cargo clippy -p uor-r4-harness --offline --all-targets -- -D warnings -> clean, 0 findings
cargo test -p uor-r4-harness --offline                 -> 432 passed, 0 failed, 6 ignored
cargo test -p uor-r4-harness --offline --lib -- --ignored the_staged_rule_set -> ok
python3 -m unittest discover -s tests/harness -t .     -> exit 0, 0 failure lines
python3 scripts/check_no_softmax.py                    -> OK 1641, baseline 1641
python3 scripts/harness/parity.py                      -> exit 0
```

**Both implementations, seven invocations, rule-id sets compared:**

```
plain          ids MATCH  (41 rust / 41 py)  records 51 / 58  exit 0/0
repo-global    ids MATCH  (41 rust / 41 py)  records 51 / 58  exit 0/0   <- was 0 rust
staged         ids MATCH  (0 rust / 0 py)    records 0 / 0    exit 0/0
only-arena     ids MATCH  (4 rust / 4 py)    records 4 / 4    exit 0/0
only-two       ids MATCH  (8 rust / 8 py)    records 13 / 13  exit 0/0
only-repeat    ids MATCH  (8 rust / 8 py)    records 13 / 13  exit 0/0
bogus-flag     exit 2/2  names-token True/True
bare-token     exit 2/2  names-token True/True
```

The defect before the fix, on the pre-change binary built from `64237ffe6`:

```
uor-harness --repo $PWD check          -> 0 rules, 0 failed (0 error, 0 warn)
uor-harness --repo $PWD check --json   -> "checked": 0
uor-harness check --json               -> "checked": 52
```

The second finding in the body is reproduced the same way:

```
python3 scripts/harness/cli.py check --only status.no_orphan_branches
  -> warn  local branch bpb-split still exists but PR #2081 merged 2026-10-09
uor-harness check --only status.no_orphan_branches
  -> 1 rules, 0 failed (0 error, 0 warn)
```



---
_Comment/review ids in this file: 6164108219_

