# #1883 — LUT-4: learnable connectivity + a guard that refuses to report a degenerate term (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T08:13:38Z
- **Updated:** 2026-10-09T13:20:55Z
- **Closed:** 2026-10-08T08:15:18Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1883
- **Merged:** 2026-10-08T08:15:18Z
- **Merge commit:** 9d7415523411d641dccc542a884147b2dc702e63
- **Base / Head:** main / UOR-Foundation:deepseek/lut4-connectivity
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What this fixes

The retraction (#1881) identified the real Stage 4 defect: connectivity never moved, so training collapsed to a constant gate and the "improvement" was a uniform shortlist penalty. This makes selection learnable and makes the harness incapable of reporting that failure as a result.

**Two selection heads per output.** `connectivity` is now `[width_out][2][width_in]` — each slot has its own distribution, so the two inputs a gate reads are two independent decisions, both receiving gradients. The soft forward reads each distribution's *expectation*; the hard export takes the argmaxes, with the runner-up resolving a duplicate so a gate always has two distinct inputs (a single-input gate cannot express XOR — that was the earlier collapse).

**Exact gradients for selection.** `backward` returns the softmax gradient of each head's expectation, `upstream · ∂value · p_j · (x_j − a)`, replacing the utility nudge, and `fit_gates` applies real bounded descent.

**A sharpening constant was required.** With a near-uniform selection softmax the soft forward reads the *mean* of every input, which is a constant, and a constant layer cannot learn. `SELECTION_TEMPERATURE = 0.25` sharpens at initialisation. Same lesson as the gate mixture: a symmetric start is a fixed point, not a neutral one.

## New test obligations (8 passed / 0 failed)

- `connectivity_learns_which_inputs_matter_and_the_result_is_not_degenerate` — sixteen features, only `x[5] XOR x[11]` carries the label. Training reduces the loss (0.2500 → 0.2434 in 4,000 steps, ~75 % accuracy), the first layer wires at least one informative input, and the trained network reports **more than one** distinct hard **and** soft output. That is the assertion the retracted result lacked.
- `a_constant_gate_network_is_reported_as_degenerate` — a gate pinned FALSE reports exactly one distinct output. The guard is tested against the exact failure that produced the retraction.
- The original six still pass, including XOR with an exact hard export.

## The harness now says so by itself

Re-run on the real task (chat-v0-p2 train → held-out, 64 positions, top-64):

```
WARNING: the learned term is DEGENERATE (1 distinct artifact outputs); the reported delta is a
constant shift over the shortlist, not a learned re-ranking, and must not be reported as improvement.
chosen_gates [12 ×16, 0]; distinct_hard_outputs 1; hard_bit_tally [4096, 0]
reported hard delta -0.043631   (NOT a result: the constant shortlist penalty)
```

**Stage 4 remains NOT PASSED** — and for the first time the tool refuses to present the constant shift as improvement instead of my catching it afterwards.

## Why the real task still collapses, and the next lever

Only 12.5–20 % of positions have the target inside the shortlist, so for most training rows every candidate is negative and the least-squares optimum *is* the constant 0. The next lever is to **train only on rows where the target is in the shortlist** — the only rows where re-ranking can help — which balances the labels and matches the decision the term is meant to make. That's the next experiment, not another architectural change.

Scope: one artifact, one corpus pair, 64–256 positions, debug build, no serving default changed; the served form remains a truth-table read. Evidence: `docs/evidence/native_geometric_lut4_connectivity_2026-10-08.txt`.

References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

