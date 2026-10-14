# The decision after the reading — `registry_vs_github` is registered

**Date:** 2026-10-14 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** the last piece of the rule I have deferred longest

## What landed

**`registry_vs_github`** — the six-outcome decision after the card reading — **is registered.**

Coverage: **52 of 54.** **ONE of `check`'s own rules remains**: `harness.documented_commands`.

## Verified against Python on LIVE data

**All eight findings on the real repository agree message for message** — severity, `ok`, and text:

| milestone | outcome |
|---|---|
| M1, M4, M5–M8 | `no status card found…; nothing to compare` |
| M2, M3 | `has no machine-readable number; registry says…` |

**The test is `#[ignore]`d and asserts the two implementations AGREE rather than what the findings
say** — the latter would be the *"true of the tree when written"* mistake this port has made three
times.

## Three outcomes are "did not compare", and that is the point

**GitHub unavailable → `warn`, `ok=false`.** No comments cached → info. No card → info. **The port
keeps all three distinct**: an earlier unit fixed this rule reporting a green pass when it had not
checked, and **collapsing "not compared" into "agrees" is exactly the defect that fix closed.**

## The bug the live comparison found: `repr` chooses its quote

My `py_repr` always used `'`. **The live run failed on M2**, whose card text contains an apostrophe —
**Python's `repr` uses `"` when the string contains a `'` and no `"`.**

**A message compared character-for-character is behaviour**, so *"usually the same quote"* is a port
bug that shows up **only in the messages that happen to contain an apostrophe — which is most prose a
lab writes.**

**This is the second time this port has needed `py_repr`**, and the first time it was the same
mistake in its naive form. **The live test caught what the unit tests could not**: the fixture texts
had no apostrophes.

## Verification

| check | result |
|---|---|
| the decision vs Python on **live data** (8 findings, message for message) | **agrees** |
| the six outcomes, including three "did not compare" | asserted by the comparison |
| `cargo test -p uor-r4-harness` | **388 passed, 2 ignored** |
| `cargo fmt --check`, clippy | clean, 0 findings |
| `python3 -m unittest discover -s tests/harness -t .` | 332 passed |
| `python3 scripts/check_no_softmax.py` | OK — 1641, baseline 1641 |
| `parity.py` | exit 0, **52 implemented**, **1 of `check`'s own rules left** |

## Limitations, stated

- **`check` still delegates.** One rule remains — `harness.documented_commands` — **and until it
  lands, `check` keeps delegating, because a `check` that skipped it would be a silent downgrade.**
- **The live test is `#[ignore]`d, so CI does not run it.** **The `repr` bug it caught would not have
  been caught by CI either**, and the reason is structural: **the fixtures were text I wrote, and I
  did not put an apostrophe in any of them.**
- **`github_issues` is now called by the decision with `refresh` passed through**, so a rule run with
  `refresh=false` reads the cache. **That makes the rule's answer depend on cache age** — Python has
  the same shape, and it is why the rule reports *"no comments cached"* rather than refetching on its
  own.
- **Two `["current"], ["of"]` accesses are ported as `None` for a missing key rather than raising**,
  which is Python's `.get()` — **but a card whose number is present and non-numeric is a case neither
  implementation handles by design**, since the reading parses only digits.
