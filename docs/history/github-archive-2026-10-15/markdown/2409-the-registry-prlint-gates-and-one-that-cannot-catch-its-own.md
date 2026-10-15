# #2409 — The registry prlint gates, and one that cannot catch its own mistake (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-14T01:31:37Z
- **Updated:** 2026-10-14T01:35:05Z
- **Closed:** 2026-10-14T01:33:40Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2409
- **Merged:** 2026-10-14T01:33:40Z
- **Merge commit:** 175fc754c259c8138db8f0af55a5afe18ba8dd44
- **Base / Head:** main / UOR-Foundation:deepseek/registry-gates
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

**The registry gates — and one that cannot catch the mistake it was written for.** Tooling only.

Decision: **KEEP** (tooling).

## What landed

**`prlint.count_matches_registry` and `prlint.headline_vs_registry`**, plus the closure they need, **ported whole rather than assumed small**: `_norm`, `_find_line`, `_registry_headline`, `_same_number`, `_fmt_num`, `_target_matches_registry`, `Headline.render()`, and **`_find_milestone`** — which **scores candidate milestones rather than trusting the first issue number** (line match 3, milestone id in the body 1, agreeing headline 2), because **a body cites the tracker as well as its own milestone.**

Coverage: **38 of 54**, up from 36.

## The finding: `headline_vs_registry` cannot fire on its own subject

```python
if milestone and current is not None and not _target_matches_registry(after, milestone) and not stated_move:
```

**`stated_move` is `before.numeric and not _same_number(before.value, after.value)`** — and **`after.numeric` requires an arrow, which puts a value on the left.** Enumerated against the live registry:

| before | after | stated_move | fires |
|---|---|---|---|
| 37 | 5 | **True** | no |
| 5 | 5 | **False** | **yes** |
| 37 | 37 | False | no (matches) |

**A lab that moves a headline and forgets the registry has `stated_move` true and is never reported.** The rule fires only on a body whose headline **does not move** yet disagrees — `headline 5/232 -> 5/232`, self-contradictory.

**I confirmed that body produces the finding and every realistic one produces none**, against the live registry rather than by reading.

**I did not change it.** Whether the guard should exclude `stated_move` when the after value *disagrees* with the registry **is a D21 policy question**, and **the port matches Python either way** — a deliberate divergence would make parity lie about which implementation a lab is running.

## Two of my own mistakes, both caught by the fixtures

**1. I wrote the count test the wrong way round** — asserting declaring `0` against a registry `12/3` was *not* this branch. **M2's line sits at 12, so 0 is below and must be reported.** The test failed and said so.

**2. Rust's `{:?}` quotes with `"` where Python's `!r` uses `'`.** Added `py_repr`. **Not cosmetic: the messages are compared character for character, so a port emitting `"name"` where Python emits `'name'` differs from the tool it replaces — and that difference would surface in every lab's terminal, not in a test that only checks the rule id.**

```
cargo test -p uor-r4-harness                    -> 340 passed (7 new)
cargo fmt --check / clippy -D warnings          -> clean / 0 findings
python3 -m unittest discover -s tests/harness   -> 332 passed
python3 scripts/check_no_softmax.py             -> OK — 1641, baseline 1641
parity.py                                       -> exit 0, 38 implemented
```

Record: `docs/labs/prlint-registry-gates-2026-10-14/README.md`

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record + current-state entry carried, presence asserted before commit
- [x] Branch deleted after merge

### Stated rather than hidden

- **`check_pr` is still not ported. Nine of fourteen gates remain**, plus the dispatcher and the `lint` command. **`lint` still delegates, and none of the six ported gates is reachable from any command** — verified units, not a working feature.
- **`count_matches_registry`'s first branch is asserted only below the registry**, and the `of`-absent fallback to 3 is **ported but unverified**.
- **`_find_milestone`'s scoring was tested with two bodies.** A body where two milestones score equally is untested, and **the `is_none_or` tie-break keeping the first is reasoned equivalent to Python's strict `score > best[0]`, not measured.**
- **`same_number` returns false when either side is non-numeric** where Python's `float()` raises and falls back to `==`. **A string-valued headline number would behave differently**; the committed registry has none.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

