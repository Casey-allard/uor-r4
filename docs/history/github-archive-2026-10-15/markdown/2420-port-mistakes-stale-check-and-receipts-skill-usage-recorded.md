# #2420 — Port mistakes.stale_check and receipts.skill_usage_recorded: 5 of check's rules remain (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-14T02:43:23Z
- **Updated:** 2026-10-14T02:46:07Z
- **Closed:** 2026-10-14T02:45:39Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2420
- **Merged:** 2026-10-14T02:45:39Z
- **Merge commit:** 09959d7691204d7dbde73552a90a53aa746a40c6
- **Base / Head:** main / UOR-Foundation:deepseek/check-tail
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

**`mistakes.stale_check` and the rule that cannot fail; 5 of `check`'s rules left.** Tooling only.

Decision: **KEEP** (tooling).

## What landed

| rule | severity | what it does |
|---|---|---|
| `mistakes.stale_check` | info | every lesson's `paths` still match a file in the repo |
| `receipts.skill_usage_recorded` | info | **nothing — both of its branches return empty** |

Coverage: **47 of 54** — **5 of `check`'s own rules remain**, down from 7.

## `stale_check`: two branches with *different* matching rules

**The matcher is not one algorithm with a fast path.** Python splits patterns into **literals**, resolved by exact path, suffix **or substring**, and **globs**, resolved by `fnmatch`. **Porting only the fast path would have made a lesson written as a bare filename stop firing** — the fnmatch-semantics bug this port already hit once, where separator-aware `*` meant `*team.rs*` did not match `crates/x/src/team.rs`.

**`<dir>/**` is rewritten to a directory prefix literal**, not left as a glob, because the two match differently. Asserted.

**The index comes from `git ls-files`, not a walk.** Python's original traversed ~11k files with three `fnmatch` calls per (file, pattern) pair per lesson — **~46 seconds, dominating the whole `check`.** `git ls-files` is the already-built index, excludes ignored build output for free, **and `-z` is used because paths can contain newlines.**

## And one rule is ported as the no-op it is

```python
recs = read(paths, kinds=("skill",))
if recs:
    return []
return []  # informational only: the engine prints "ok" for an empty list
```

**Both branches return empty, so no input produces a finding.** It is declared, registered and reported as passing, **and it asserts nothing.**

**Ported as an explicit empty list rather than a plausible check.** Writing it as *"report when skills were used but no skill receipt exists"* would be **inventing a rule the engine does not have** — **a behaviour change dressed as a port.** **A test asserts it produces nothing for any state**, so a future change is **visible rather than hidden behind a check that passes silently.**

## A mistake the check caught on me

**My first stale-lesson fixture reported 0 findings, and the check was right.** No git repository meant no index — and the port **deliberately reports nothing rather than every lesson as stale**, because **no index means no evidence, and reporting everything would be a finding that is wrong rather than missing.** The fixture now initialises a repo, and **both behaviours are asserted.**

```
cargo test -p uor-r4-harness                    -> 369 passed (7 new)
cargo fmt --check / clippy -D warnings          -> clean / 0 findings
python3 -m unittest discover -s tests/harness   -> 332 passed
python3 scripts/check_no_softmax.py             -> OK — 1641, baseline 1641
parity.py                                       -> exit 0, 47 implemented
```

Record: `docs/labs/stale-check-port-2026-10-14/README.md`

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record + current-state entry carried, presence asserted before commit
- [x] Branch deleted after merge

### Stated rather than hidden

- **`check` still delegates, so the objective is not met. Five of its own rules remain**: `harness.entry_point`, `harness.documented_commands`, `prlint.gates_declared`, `budget.no_new_spending_class`, `status.registry_vs_github`. **Until all five land, `check` keeps delegating — porting a subset would be the silent downgrade this port refuses.**
- **`harness.entry_point` and `harness.documented_commands` describe the file they live in** — the "two-rule/one-file" pattern, so **they should become unnecessary rather than translated**, and two of the five belong with retiring `cli.py`.
- **`stale_check`'s index is `git ls-files`**, so an untracked-and-ignored lesson path reads as stale even though the file exists. Python is the same — **a lesson pointing at build output is reported, correctly, as pointing at something the repo does not track.**
- **The no-op rule's test asserts today's behaviour, not a requirement.** If skill usage *should* be checked, the test fails — **which is the intended way for that decision to become visible.**

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

