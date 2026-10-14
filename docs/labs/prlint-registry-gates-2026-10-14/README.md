# The registry gates — and one of them cannot catch the mistake it was written for

**Date:** 2026-10-14 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** the registry-dependent `prlint` gates, unblocked by `body_text`

## What landed

**`prlint.count_matches_registry` and `prlint.headline_vs_registry`**, plus the closure they need —
ported whole rather than assumed small, because the previous unit measured it:

`_norm` · `_find_line` · `_registry_headline` · `_same_number` · `_fmt_num` · `_target_matches_registry`
· `Headline.render()` · and **`_find_milestone`**, which **scores candidate milestones rather than
trusting the first issue number** — a line match is worth 3, the milestone id in the body 1, an
agreeing headline 2 — because **a body cites the tracker as well as its own milestone, and the first
number is often the tracker.**

Coverage: **38 of 54** implemented, up from 36.

## The finding: `headline_vs_registry` cannot fire on the mistake it exists for

Its guard is:

```python
if milestone and current is not None and not _target_matches_registry(after, milestone) and not stated_move:
```

**`stated_move` is `before.numeric and not _same_number(before.value, after.value)`.** And
**`after.numeric` requires an arrow, which puts a value on the left.** So:

| before | after | stated_move | fires |
|---|---|---|---|
| 37 | 5 | **True** | no |
| 5 | 5 | **False** | **yes** |
| 37 | 37 | False | no (matches) |

**A lab that moves a headline and forgets the registry has `stated_move` true and is never reported.**
The rule fires only on a body whose headline **does not move** yet disagrees with the registry —
`headline 5/232 -> 5/232`, which is self-contradictory. I confirmed it against the live registry
rather than by reading: that body produces the finding, and every realistic one produces none.

**I did not change it.** Whether the guard should exclude `stated_move` when the after value *disagrees*
with the registry is a policy question about D21, and **the port matches Python either way** — a
deliberate divergence here would make parity lie about which implementation a lab is running.

## Two of my own mistakes, both caught by the fixtures

**1. I wrote the count test the wrong way round.** I asserted that declaring `0` against a registry
`12/3` was *not* this branch. **M2's line sits at 12, so 0 is below it and must be reported.** The
test failed and said so.

**2. Rust's `{:?}` quotes with `"` where Python's `!r` uses `'`.** The messages are compared character
for character, so I added `py_repr`. **This is not cosmetic**: a port emitting `"name"` where Python
emits `'name'` differs from the tool it replaces, **and the difference would show up in every lab's
terminal rather than in a test that only checks the rule id.**

## Verification

| check | result |
|---|---|
| messages match Python character for character | asserted, including the line-name list |
| a count below the registry is reported | asserted, against M2's real `12/3` |
| a count at the registry is silent | asserted |
| the milestone is the one the body is about | asserted for M1 and M4 by line name |
| `_norm`/`_find_line` prefer the exact match | asserted |
| the headline gate's only reachable firing body | asserted, and its unreachability recorded |
| `cargo test -p uor-r4-harness` | **340 passed** (7 new) |
| `cargo fmt --check`, clippy | clean, 0 findings |
| `python3 -m unittest discover -s tests/harness -t .` | 332 passed |
| `python3 scripts/check_no_softmax.py` | OK — 1641, baseline 1641 |
| `parity.py` | exit 0, **38 implemented** |

## Limitations, stated

- **`check_pr` is still not ported.** Nine of its fourteen gates remain, plus the dispatcher and the
  `lint` command. **`lint` still delegates, and none of the six ported gates is reachable from any
  command** — they are verified units, not a working feature.
- **`count_matches_registry`'s first branch is asserted only in the below-registry direction.**
  The `of` value falls back to 3 when a line omits it, and **I did not test a line whose `of` is
  absent**, so that fallback is ported but unverified.
- **`_find_milestone`'s scoring was tested with two bodies.** The three weights and their tie-break
  are ported from the source, but **a body where two milestones score equally is untested**, and
  `is_none_or` keeps the *first* on a tie — which is Python's behaviour only because `score > best[0]`
  is strict. **That equivalence is reasoned, not measured.**
- **`same_number` returns `false` when either side is non-numeric**, where Python's `float()` would
  raise and fall back to `==`. **A string-valued headline number would therefore behave differently**;
  the committed registry has none, so the case does not arise today.
