# #2397 — A check that cannot run no longer reports as a pass (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T23:34:04Z
- **Updated:** 2026-10-13T23:58:15Z
- **Closed:** 2026-10-13T23:57:49Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2397
- **Merged:** 2026-10-13T23:57:49Z
- **Merge commit:** 8ea03b720a6b5e5c6dec12cc71b8bf411d927123
- **Base / Head:** main / UOR-Foundation:deepseek/cannot-run-signal
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

**A check that cannot run no longer reports as a pass.** Tooling only.

Decision: **KEEP** (tooling).

## The defect

`harness.state_formats` returned **`severity="info"`, `ok=True`** when the Rust binary was not built.

**So a machine without the binary got a green `check` on the one rule whose entire job is to notice that persisted state drifted from its frozen contract.** The rule reported that it did *not* check, at a severity that neither fails nor warns.

**The Rust port cannot enter that state** — the rule *is* the binary, so an unloadable contract is an error there. **This fixes the Python half, which is the half labs actually run today.**

## The fix, and why `warn` rather than `error`

**`warn` with `ok=False`**, and the message now opens with **`the frozen contract was NOT checked`**.

**`error` would be wrong.** A lab that has not built the binary is not violating anything, and **a permanently-red check gets ignored** — which is the argument that produced `info` in the first place. **What changes is that it is VISIBLE.**

**Both halves of the argument are honoured: no permanent red, and no false green.**

## Verified in both states

```
binary absent   -> 1 rules, 1 failed (0 error, 1 warn)
                   "the frozen contract was NOT checked"
binary present  -> 1 rules, all pass
```

**Before the fix, the first state reported `all pass`.** Two tests, both function-level rather than environment-dependent.

```
python3 -m unittest discover -s tests/harness -t .   -> 324 passed (2 new)
cargo test -p uor-r4-harness                         -> 321 passed
python3 scripts/check_no_softmax.py                  -> OK, no softmax added
parity.py                                            -> exit 0
```

Record: `docs/labs/skipped-is-not-a-pass-2026-10-13/README.md`

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record + current-state entry carried
- [x] Branch deleted after merge

### Stated rather than hidden

- **This fixes one rule's cannot-run path.** `status.registry_vs_github` **has the same shape** — `info`/`ok=True` when GitHub is unavailable — **and is not fixed here. Two rules, one defect, and the second is still live.**
- **The severity change alters `check`'s output on any machine without the binary**: a machine that previously showed no warning there now shows one. **That is the intent**, and it is a visible change to a count other labs read.
- **`ok=False` with `severity="warn"` is an unusual pair.** Elsewhere `ok=False` means a finding and warnings are findings, so this is consistent — **but a reader who assumes `ok=True` means "no finding" would misread the old code, which is exactly how the defect survived.**
- **The suite is flaky under concurrent lab activity.** One execution during this unit reported **two failures**; two subsequent executions reported **OK**, while the registered rule count moved between **54, 55 and 56** across them. **The failure was not mine — I repeated the check before concluding**, and a single red result in this worktree is not evidence of a regression.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

