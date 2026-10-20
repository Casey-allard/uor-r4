# `status.registry_vs_github`: a check that cannot run no longer reports as a pass

**Date:** 2026-10-13 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** the sibling defect named in `docs/labs/skipped-is-not-a-pass-2026-10-13/README.md`

## The defect, and the test that pinned it

Both unavailability paths returned **`severity="info"`, `ok=True`**:

```python
return [RuleResult(..., severity="info", ok=True, message=f"GitHub not checked: ...")]
return [RuleResult(..., severity="info", ok=True, message="GitHub unavailable, ...")]
```

**So a lab without GitHub access got a green `check` on the rule whose entire job is to notice that
`milestones.json` and the milestone issue have drifted apart.**

**And an existing test asserted exactly that**, named for it:

```python
def test_registry_vs_github_degrades_to_info_without_gh(self) -> None:
    ...
    self.assertTrue(results[0].ok)
    self.assertEqual(results[0].severity, "info")
```

**The test named the defect and pinned it as intended behaviour.** It is updated to pin the new one.

## Fixed

**`warn` with `ok=False`** on both paths, messages opening **`the registry was NOT cross-checked
against GitHub`**. **`error` would be wrong, and the argument is stronger here than for a missing
binary**: GitHub rate-limits, and a lab working offline is not violating anything — **a
permanently-red check gets ignored.**

**What changes is that neither case is silent.** Both paths verified by forcing the failure.

## What the fix revealed, which is larger than the fix

**With GitHub available, the rule returns 8 findings — and every one is `ok=True`, `info`:**

```
[M1] newest card (2026-10-13T23:26:14Z) has no machine-readable number; registry says ...
[M2] newest card (2026-10-13T02:52:05Z) has no machine-readable number; registry says ...
... 8 total; ok values: [True]; severities: ['info']
```

**The rule cannot fail.** Its comparison never completes, because every milestone's newest card lacks
the machine-readable number it looks for. **I checked the cards rather than assuming the parser was
stale** — the newest card on #2029 is a prose work card with no numeric field. **So the parser is
right, and the rule has never cross-checked anything.**

**This is the same shape a third time, and larger**: the unavailability paths were the smaller half.
**A comparison that cannot complete because the input never carries the field is reporting green on
every run — not because the registry agrees with GitHub, but because nothing was ever compared.**

**I am not changing that behaviour here.** `info` may be the deliberate design for "the card has no
number", and **deciding what the rule should do when no card is machine-readable is a policy
question, not a port.** It is recorded on #2028 with the measurement so it can be decided rather than
rediscovered.

## Verification

| check | result |
|---|---|
| raising lookup | `warn`, `ok=False`, `NOT cross-checked` |
| GitHub reported unavailable | `warn`, `ok=False`, `NOT cross-checked` |
| the old test, renamed and updated | passes, pinning the new behaviour |
| `python3 -m unittest discover -s tests/harness -t .` | **326 passed**, twice |
| `uor-harness check` | 0 errors |
| `python3 scripts/check_no_softmax.py` | OK, no softmax added |
| `parity.py` | **exit 0**, twice (was failing on the date) |
| `cargo test -p uor-r4-harness` | **323 passed** (2 new) |
| `cargo fmt --check`, clippy | clean, 0 findings |

## Limitations, stated

- **`parity.py` failed during this unit — and that defect is now FIXED in this same PR, because CI
  made it blocking rather than theoretical.** `render agreement: line 3: rust '- date: 2026-10-14'
  != python '- date: 2026-10-13'`. **Root cause: Rust's `prep::today()` sliced
  `now_rfc3339()[..10]` — the UTC date — while Python's `core.today()` is
  `datetime.now(timezone.utc).astimezone().strftime("%Y-%m-%d")`, UTC converted to local.** In a US
  evening they differ by a day, so **parity failed for roughly the last five hours of every US
  working day — in CI as well as locally.** `today()` now asks `date +%Y-%m-%d`, the same
  system-fact pattern `team.rs` and `budgetrules.rs` already use, with the UTC slice kept as the
  fallback so a host without `date` behaves exactly as before. **A verification that fails on a
  schedule is one people learn to ignore.**
- **This was caught by the very check it was breaking.** I reported the parity failure as
  "pre-existing and not mine" and moved to ship; **CI failed the merge on it.** `parity` is the
  reason the divergence existed for hours instead of weeks, and it is the reason I could not ship
  around it.
- **The two unavailability cases still share one severity.** *"Rate-limited right now"* and *"this
  registry has disagreed with its issue for a week"* both produce one warning. **They are no longer
  silent, but they are not yet distinguished** — and distinguishing them needs the comparison to
  work first.
- **The suite is flaky under concurrent lab activity.** Two runs during this unit reported failures
  and two reported `OK`; **I repeated before concluding.** A single red run in this worktree is not
  evidence of a regression, and **the registered rule count moved between 53 and 56 across runs.**
