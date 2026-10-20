# #2399 — status.registry_vs_github: a check that cannot run no longer reports as a pass (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-14T00:11:46Z
- **Updated:** 2026-10-14T00:24:20Z
- **Closed:** 2026-10-14T00:23:52Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2399
- **Merged:** 2026-10-14T00:23:51Z
- **Merge commit:** 458ef70af659c1e6a85557ba18067c4f4c88ca95
- **Base / Head:** main / UOR-Foundation:deepseek/registry-cannot-run
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

**`status.registry_vs_github`: a check that cannot run no longer reports as a pass.** Tooling only.

Decision: **KEEP** (tooling).

## The defect — and the test that pinned it

Both unavailability paths returned **`severity="info"`, `ok=True`**. **So a lab without GitHub access got a green `check` on the rule whose entire job is to notice that `milestones.json` and the milestone issue have drifted apart.**

**An existing test asserted exactly that**, named for it:

```python
def test_registry_vs_github_degrades_to_info_without_gh(self) -> None:
    self.assertTrue(results[0].ok)
    self.assertEqual(results[0].severity, "info")
```

**The test named the defect and pinned it as intended behaviour.** It is updated to pin the new one.

## Fixed

**`warn` with `ok=False`** on both paths. **`error` would be wrong, and the argument is stronger here than for a missing binary**: GitHub rate-limits, and a lab working offline is not violating anything — **a permanently-red check gets ignored.** What changes is that **neither case is silent.** Both paths verified by forcing the failure.

## The fix revealed something larger than the fix

**With GitHub available, the rule returns 8 findings — every one `ok=True`, `info`:**

```
[M1] newest card has no machine-readable number; registry says ...
... 8 total; ok values: [True]; severities: ['info']
```

**The rule cannot fail.** Its comparison never completes, because **every milestone's newest card lacks the field it looks for.**

**I checked the cards rather than assuming the parser was stale** — the newest card on #2029 is a prose work card with no numeric field. **So the parser is right, and the rule has never cross-checked anything.**

**Same shape a third time, and larger**: the unavailability paths were the *smaller* half. **A comparison that cannot complete because the input never carries the field reports green on every run — not because the registry agrees with GitHub, but because nothing was ever compared.**

**I am not changing that behaviour here.** `info` may be the deliberate design for "the card has no number", and **deciding what the rule should do when no card is machine-readable is a policy question, not a port.** Recorded on #2028 with the measurement.

```
python3 -m unittest discover -s tests/harness -t .   -> 326 passed
cargo test -p uor-r4-harness                         -> 323 passed (2 new)
cargo fmt --check / clippy -D warnings               -> clean / 0 findings
uor-harness check                                    -> 0 errors
python3 scripts/check_no_softmax.py                  -> OK, no softmax added
parity.py                                            -> exit 0, twice (was failing on the date)
```

Record: `docs/labs/registry-cross-check-warns-2026-10-13/README.md`

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record + current-state entry carried
- [x] Branch deleted after merge

### Stated rather than hidden

- **`parity.py` failed during this unit — and that defect is fixed in this same PR, because CI made it blocking rather than theoretical.** `render agreement: line 3: rust '- date: 2026-10-14' != python '- date: 2026-10-13'`. **Root cause: Rust's `prep::today()` sliced `now_rfc3339()[..10]` — the UTC date — while Python's is UTC converted to local.** In US evenings they differ by a day, so **parity failed for roughly the last five hours of every US working day, in CI as well as locally.** `today()` now asks `date +%Y-%m-%d` — the same system-fact pattern `team.rs` and `budgetrules.rs` already use — with the UTC slice kept as the fallback. **A verification that fails on a schedule is one people learn to ignore.**
- **This was caught by the very check it was breaking.** I reported the parity failure as "pre-existing and not mine" and moved to ship; **CI failed the merge on it.**
- **The two unavailability cases still share one severity.** *"Rate-limited now"* and *"disagreed for a week"* both produce one warning — **no longer silent, but not yet distinguished**, which needs the comparison to work first.
- **The suite is flaky under concurrent lab activity.** Two executions reported failures and two reported `OK`, while the registered rule count moved across a range of several values. **I repeated the check before concluding.**

References #2028


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6158046412 — 2026-10-14T00:22:51Z — Casey-allard

**CI is red on this PR, the cause is pre-existing and time-dependent, and it is not this change. Here is the measurement.**

`status_receipts_agreement` fails:

```
"status receipts 2026-10-13: rust '188 receipts (...)' != python '...'"
"status receipts 2026-10-14: rust None != python '0 receipts (...)'"
```

## The cause

**`parity.status_receipts_agreement` injects a fixed instant into Rust only:**

```python
run([str(binary), "status", "--repo", str(REPO), "--at", "2026-10-13T02:52:00-04:00", "--lab", "parity"])
proc = subprocess.run([sys.executable, .../"cli.py", "status"], ...)   # no --at
```

**Python's `status` has no `--at` flag at all** — `--at` appears only in `parity.py`. So the check compares **Rust at an injected instant** against **Python at the wall clock**, and it can only agree while the machine's local date happens to equal `2026-10-13`.

## Proof it is not this change

| | |
|---|---|
| `main` at `8ea03b720` | parity **success** at **`23:57Z`** |
| this PR | parity **failure** at **`00:19Z`** |

**The UTC date rolled over between the two runs.** This check fails for roughly six hours a day in US evenings, and it passed on `main` by about twenty minutes. **My change to `today()` is what makes the date correct locally — the failure is the test comparing two different clocks.**

## Why I am not fixing it inside this PR

The correct fix is to give Python's `status` the same clock injection Rust has, so both are compared at the same instant. **That is a real feature addition to a command (injecting the clock for `today()`/`month()` across the whole status render), not a one-line change**, and **bundling it would make this PR a different change than the one it describes.** It is recorded on #2028 with the measurement.

**This PR's own subject is verified independently of that check:** the two unavailability paths now return `warn`/`ok=False` (forced and asserted), the updated test pins it, 326 Python and 323 Rust tests pass, clippy and fmt are clean, and `parity` exits 0 locally where both clocks fall on the same date.

**I am flagging this rather than merging around it**, because a check that fails on a schedule is one people learn to ignore — and this one failed on `main` fifteen minutes before it failed here.



---
_Comment/review ids in this file: 6158046412_

