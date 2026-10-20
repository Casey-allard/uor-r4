# `check` read its arguments as a rule filter, so `--repo` ran ZERO rules

**Date:** 2026-10-14 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** a defect in the ported `check`, found while verifying the `team roster` unit (#2028)

## The defect, measured on a clean `main`

```
$ uor-harness check --json            | grep '"checked"'   ->  "checked": 52
$ uor-harness --repo $PWD check --json | grep '"checked"'  ->  "checked": 0
$ uor-harness --repo $PWD check
0 rules, 0 failed (0 error, 0 warn)
```

**`--repo` is forwarded into the subcommand's argument list on purpose** — `main.rs` pushes it into
`rest` because `receipt` needs to know which repository it is writing to — **and `cmd_check_rules`
handed that same list to `checkdispatch::run` as the `--only` prefix set.** Every token was therefore
a filter, and no rule id starts with `--repo`, so **a path silently disabled the entire rule set and
the output was indistinguishable from a clean repository.** Controls that show the mechanism:
`--only arena` checks 4 rules, `--only zzz` checks 0, `--repo <path>` checked 0.

**The pre-change binary reproduces it.** This is the third time this port has found the same shape —
a command that reports success having checked nothing — after `cmd_check` running only the format
contract, and three rules that were registered and never dispatched. It is the first time it was
reachable from an invocation a lab would type by habit.

## What changed

* **`checkdispatch::parse_args`** parses `check`'s own arguments: `--only` (`action="append"`,
  `nargs="+"`, flattened across repeats exactly as Python's flattening loop does) and `--staged`.
  `--repo` and `--lab` are **consumed** because the top-level parser forwards them on purpose;
  everything else is a **usage error, exit 2**, which is what argparse does — **an unknown token must
  never be read as a prefix that happens to match nothing.**
* **`Scope` and `run_scoped`** make `--staged` what Python makes it: **a rule SET, not a filter over
  results.** `STAGED_RULES` is empty today because every rule in Python's registry declares the
  default `repo` scope, so `check --staged` legitimately checks nothing on both sides. **The
  emptiness is checked rather than assumed**: an `#[ignore]`d live test compares `STAGED_RULES`
  against `checkrules.rules(scope="staged")`, so a staged rule added in Python fails the test
  instead of being skipped silently.
* **`--staged` was also missing from `SUBCOMMAND_FLAGS`**, so the top-level parser rejected it with
  `unknown option --staged` before `check` ever saw it. Found by running the flag, not by reading
  the list.

## Verification: every invocation compared against Python

| invocation | rule ids | records | exit |
|---|---|---|---|
| `check --json` | **41 rust = 41 python** | 51 / 58 | 0 / 0 |
| `--repo DIR check --json` | **41 = 41** (was **0**) | 51 / 58 | 0 / 0 |
| `check --staged --json` | **0 = 0** | 0 / 0 | 0 / 0 |
| `check --json --only arena` | **4 = 4** | 4 / 4 | 0 / 0 |
| `check --json --only prep budget` | **8 = 8** | 13 / 13 | 0 / 0 |
| `check --json --only prep --only budget` | **8 = 8** | 13 / 13 | 0 / 0 |
| `check --bogus` | refused, names the token | — | **2 / 2** |
| `check nonsense` | refused, names the token | — | **2 / 2** |

**The one place the two do not agree is the record count on the unfiltered run (51 vs 58), and that
predates this unit**: Python's count includes PR-body gates that a `check` with no body cannot answer.
The **rule-id sets** — which rules ran — match on every invocation, and that is the property this
unit is about.

## Found while fixing, NOT fixed here

### 1. `status.no_orphan_branches` reports `ok` where Python warns

```
$ python3 scripts/harness/cli.py check --only status.no_orphan_branches
harness check: 1 rules, 1 failed (0 error, 1 warn)
warn status.no_orphan_branches  local branch bpb-split still exists but PR #2081 merged 2026-10-09
$ uor-harness check --only status.no_orphan_branches
1 rules, 0 failed (0 error, 0 warn)
```

**The ported rule covers the local half only** — branches merged by a proof it can see offline —
while Python's rule reads `branch_pr_check` (a **merged-PR** check: `bpb-split` was squash-merged,
which leaves no ancestry link) and also reports **remote branches with no PR**. **So the rule is
absent where Python's fires, and an absent rule and a passing rule look identical in output** — the
same sentence this port wrote when it fixed `heartbeat_receipts` and `registry_vs_github`. Not fixed
here: it needs the `gh` merged-PR chain and the remote-branch listing, which is its own unit with its
own comparison.

### 2. The Rust form accepts `check --repo DIR`, which Python rejects

Python's argparse answers `unrecognized arguments: --repo <path>` with exit 2; this port **consumes**
the forwarded global flags and runs the full rule set. **A divergence in the port's favour, stated
rather than hidden** — the same one already recorded for `git switch other --repo <path>`.

## Verification

| check | result |
|---|---|
| invocation matrix above | **all match** |
| `cargo test -p uor-r4-harness` | **432 passed, 0 failed, 6 ignored** (was 425/5: 7 new tests, 1 ignored) |
| the `#[ignore]`d staged-set test | **passes** |
| `cargo fmt --check`, clippy `-D warnings` | clean, 0 findings |
| `python3 -m unittest discover -s tests/harness -t .` | exit 0, 0 failure lines |
| `python3 scripts/harness/parity.py` | **exit 0** |
| `python3 scripts/check_no_softmax.py` | OK — 1641, baseline 1641 |
| `delegate_to_python` sites | **5 — unchanged; this unit ports no command** |

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

**Next:** `status.no_orphan_branches` (its own unit: the `gh` merged-PR chain and the remote-branch
list, compared against Python on the same tree), then `team dispatch` — port `receipts.record` and
compare the written receipt and the shard file against Python's — then `mistakes show|capture`,
`multilab sync-live|migrate-ledger`, and `arena`/`budget`/`panel`.
