# A check that cannot run no longer reports as a pass

**Date:** 2026-10-13 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** the defect named in the `harness.state_formats` port

## The defect

`harness.state_formats` returns **`severity="info"`, `ok=True`** when the Rust binary is not built:

```python
if status == "skipped":
    return [RuleResult(..., severity="info", ok=True, message=detail)]
```

**So a machine without the binary got a green `check` on the one rule whose entire job is to notice
that persisted state drifted from its frozen contract.** The rule reports that it did *not* check, at
a severity that **does not fail and does not warn**.

**The Rust port cannot enter that state** — the rule *is* the binary, so an unloadable contract is an
error there. **This fixes the Python half**, which is the half labs actually run today.

## The fix, and why `warn` rather than `error`

**`warn` with `ok=False`**, and the message now opens with **`the frozen contract was NOT checked`**.

**`error` would be wrong.** A lab that has not built the binary is not violating anything, and
**a permanently-red check gets ignored** — which is the argument that produced `info` in the first
place. **What changes is that it is VISIBLE**: a warning the reader can act on, rather than a green
line asserting a verification that did not happen.

**Both halves of the argument are honoured**: no permanent red, and no false green.

## Verified in both states

```
binary absent   -> 1 rules, 1 failed (0 error, 1 warn)
                   warn ...: the frozen contract was NOT checked: the Rust harness is not built here
binary present  -> 1 rules, all pass
```

**The first line is the fix working**: before it, that state reported `all pass`.

Two tests, both function-level rather than environment-dependent — one asserts `check_contract`
returns `skipped` for a directory with no binary, the other asserts the rule turns that into
`ok=False`, `severity="warn"`, and a message containing `NOT checked`.

## Verification

| check | result |
|---|---|
| binary absent | **warn, not a pass** |
| binary present | all pass |
| `python3 -m unittest discover -s tests/harness -t .` | **324 passed** (2 new) |
| `cargo test -p uor-r4-harness` | 321 passed |
| `python3 scripts/check_no_softmax.py` | OK, no softmax added |
| `parity.py` | exit 0 |

## Limitations, stated

- **This fixes one rule's cannot-run path.** `status.registry_vs_github` has the same shape —
  `info`/`ok=True` when GitHub is unavailable — and **is not fixed here**. Two rules, one defect, and
  the second is still live.
- **The severity change alters `check`'s output on any machine without the binary**: a run that
  previously reported `0 warn` there now reports one. **That is the intent**, and it is a visible
  change to a number other labs read.
- **`ok=False` with `severity="warn"` is an unusual pair.** Elsewhere in the engine `ok=False` means a
  finding and warnings are findings, so this is consistent — **but a reader who assumes `ok=True`
  means "no finding" would misread the old code, which is exactly how the defect survived.**
- **The suite is flaky under concurrent lab activity.** One run during this unit reported
  **`324 FAILED (failures=2)`** and two subsequent runs reported **`OK`**, while the registered rule
  count moved between 54, 55 and 56 across runs. **The failure was not mine** — I re-ran before
  concluding — and **a single red run in this worktree is not evidence of a regression.**
