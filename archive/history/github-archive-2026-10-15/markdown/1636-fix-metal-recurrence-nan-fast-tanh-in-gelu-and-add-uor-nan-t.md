# #1636 — Fix Metal recurrence NaN (fast tanh in GELU) and add UOR_NAN_TRACE diagnostics

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-02T20:49:41Z
- **Updated:** 2026-10-09T13:08:45Z
- **Closed:** 2026-10-02T21:03:41Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1636
- **Merged:** 2026-10-02T21:03:41Z
- **Merge commit:** c21f1814ab6ee809664e0fcd7a5bc12d7ac6920c
- **Base / Head:** main / UOR-Foundation:codex/metal-lorentz-nan
- **Reviews / inline review comments:** 1 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #820

**Defect.** A Metal training run of the 8M ladder rung (seed 2) stopped deterministically with "nonfinite training loss at step 995". The same run on the CPU trains past step 1,000 (gradient norm 0.500).

**Diagnosis**, with a new `UOR_NAN_TRACE=1` switch:
1. **Before each update** it checks every gradient and parameter. The trace showed that all gradients and parameters were finite through step 994.
2. **On a nonfinite loss** it reruns the failing batch through `hidden_with_capture` and names the first forward site whose input is nonfinite. From the step-900 checkpoint, the result was:
   `first nonfinite site of 25: RecurrenceOut(4): 3/1769472 nonfinite, largest finite 8.08e1`.

**Cause.** The Metal recurrence forward's `gelu_fwd` used the fast `tanh`, which returns NaN for large arguments (a ratio of overflowing exponentials). A large GELU gate value reached it. The CPU `tanh` saturates to ±1 instead.

**Fix.** `precise::tanh(clamp(v, -20, 20))` in both GELU helpers. Clamping does not change the f32 value, since tanh is exactly ±1 beyond about 9.

**Checks:**
- New `test_recurrence_core_large_gate_parity`: gate values up to ±2000. Metal output is finite and within 1e-3 relative of the CPU.
- `metal_stack_ops_parity`: 17 passed, 1 ignored, on the M1.
- **End-to-end replay** from the step-900 checkpoint with the fixed binary: it passes step 995 and reaches step 1,000 with gradient norm 0.491 and development NLL 2.468.
- GitHub evidence: `codex/ci/v3/metal-nan-fix` (fmt, check of all targets, lib, and `geometric-stack` example tests).

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### review 5396819551 — 2026-10-02T21:02:50Z — Casey-allard — COMMENTED

Exact-head review at 0497832eef7d359797cd461f0a852c1e99f6f68c (Claude lab; self-review per owner rule on #820). The fix is limited to the two GELU helpers (precise tanh of a clamped argument) plus a new parity test, plus the UOR_NAN_TRACE diagnostics, which are off by default. Checks:
- metal_stack_ops_parity: 17 passed on the M1, including the new large-gate test.
- End-to-end replay from the step-900 checkpoint now passes the failing step 995.
- GitHub evidence https://github.com/UOR-Foundation/uor-r4/actions/runs/37063052033 (parent = this head): fmt, check of all targets, lib 586 passed / 0 failed, geometric-stack example tests passed.

### comment 5961449768 — 2026-10-02T21:04:10Z — Casey-allard

Merged as c21f1814; all three changed files on main equal the reviewed head 0497832e (blob comparison).


---
_Comment/review ids in this file: 5961449768, 5396819551_

