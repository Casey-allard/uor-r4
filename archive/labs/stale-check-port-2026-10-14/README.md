# `mistakes.stale_check` and the rule that cannot fail — 5 of `check`'s rules left

**Date:** 2026-10-14 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** the `check` port, which is what the objective's own measure waits on

## What landed

| rule | severity | what it does |
|---|---|---|
| `mistakes.stale_check` | info | every lesson's `paths` still match a file in the repo |
| `receipts.skill_usage_recorded` | info | **nothing — both of its branches return empty** |

Coverage: **47 of 54** — **5 of `check`'s own rules remain**, down from 7.

## `stale_check`: two branches with *different* matching rules

**The matcher is not one algorithm with a fast path.** Python splits patterns into **literals**,
resolved by exact path, suffix **or substring**, and **globs**, resolved by `fnmatch`. **Porting only
the fast path would have made a lesson written as a bare filename stop firing** — and that is the
fnmatch-semantics bug this port already hit once, where separator-aware `*` meant `*team.rs*` did not
match `crates/x/src/team.rs`.

**`<dir>/**` is rewritten to a directory prefix literal**, not left as a glob, because the two match
differently. Asserted.

**The index comes from `git ls-files`, not a filesystem walk.** Python's original walked ~11k files
and ran three `fnmatch` calls per (file, pattern) pair per lesson — **~46 seconds, dominating the
whole `check`.** `git ls-files` is the already-built index and excludes ignored build output for free.
**`-z` because paths can contain newlines.**

## And one rule is ported as the no-op it is

`receipts.skill_usage_recorded` is:

```python
recs = read(paths, kinds=("skill",))
if recs:
    return []
return []  # informational only: the engine prints "ok" for an empty list
```

**Both branches return empty, so no input produces a finding.** It is declared, registered and
reported as passing, and **it asserts nothing.**

**Ported as an explicit empty list rather than as a plausible check.** Writing it as *"report when
skills were used but no skill receipt exists"* would be **inventing a rule the engine does not have**,
and it would fire on every state where the two disagree — **a behaviour change dressed as a port.**

**A test asserts it produces nothing for any state**, so if a future change makes it produce a
finding, that change is visible **rather than hidden behind a check that passes silently.**

## A mistake the check caught on me

**My first stale-lesson fixture reported 0 findings, and the check was right.** The fixture had no
git repository, so `git ls-files` returned nothing, so **there was no index** — and the port
deliberately reports nothing rather than reporting every lesson as stale, because **no index means no
evidence, and reporting everything would be a finding that is WRONG rather than missing.** The
fixture now initialises a git repo, and **both behaviours are asserted**: a stale lesson with an index
is reported; without one, nothing is.

## Verification

| check | result |
|---|---|
| literals match by substring, globs by fnmatch | asserted |
| `<dir>/**` becomes a prefix literal | asserted |
| an empty pattern list is not stale | asserted |
| the real repo has no stale lessons (Python: 0) | asserted |
| a stale lesson is reported, with Python's text | asserted |
| no index reports nothing rather than everything | asserted |
| the no-op rule produces nothing for any state | asserted |
| `cargo test -p uor-r4-harness` | **369 passed** (7 new) |
| `cargo fmt --check`, clippy | clean, 0 findings |
| `python3 -m unittest discover -s tests/harness -t .` | 332 passed |
| `python3 scripts/check_no_softmax.py` | OK — 1641, baseline 1641 |
| `parity.py` | exit 0, **47 implemented** |

## Limitations, stated

- **`check` still delegates, so the objective is not met.** **Five of its own rules remain**:
  `harness.entry_point`, `harness.documented_commands`, `prlint.gates_declared`,
  `budget.no_new_spending_class`, and `status.registry_vs_github`. **Until all five land, `check`
  keeps delegating — and porting a subset would be the silent downgrade this port refuses.**
- **`harness.entry_point` and `harness.documented_commands` describe the file they live in**, which is
  the "two-rule/one-file" pattern: **they should become unnecessary rather than translated**, and two
  of the five belong with retiring `cli.py`.
- **`stale_check`'s index is `git ls-files`, so an untracked-and-ignored lesson path reads as stale**
  even though the file exists on disk. Python has the same behaviour — **and it means a lesson
  pointing at build output is reported, correctly, as pointing at something the repo does not track.**
- **The no-op rule's test asserts today's behaviour, not a requirement.** If the owner decides skill
  usage *should* be checked, the test fails — **which is the intended way for that decision to become
  visible.**
