# M1 Part B′ — keyed variable-lag retrieval (KVAR): a gated overwrite addressed memory extends the horizon

Owner-adopted milestone executing the amended M1 Part B from the K3 architecture gate
([`direction-decision-2026-09-24.md`](direction-decision-2026-09-24.md)) and the frozen pre-registration
[`kvar-plan-2026-09-24.md`](kvar-plan-2026-09-24.md). It tests whether a **gated, overwriting, addressed memory**
extends the model's horizon beyond the measured ~2–3 tokens — the binding lever identified by
[`frozen-state-readout-adjudication-result-2026-09-24.md`](frozen-state-readout-adjudication-result-2026-09-24.md).
Standalone synthetic harness; **no change** to `learner/transferable_lexical.rs` or `tl_execution.rs`.

Base `f0a7fc4a`. Worktree `…/uor-r4-worktrees/kvar-20260924`. New file `crates/uor-r4-core/src/bin/kvar-recall.rs`.
Evidence root `/Users/casey.allard/uor-r4-investigations/kvar-20260924` (`run4` authoritative; `run1..3` and the
unsealed `run4-crashed-attempt` preserved).

## Panel (frozen)

V = 64 content tokens + explicit `BIND`/`QUERY` markers. `K ∈ {4,8}` keys; every key **rebound ≥2×** with distinct
values; the query answer is the key's **most recent** value; `lag ∈ {4,16,64}` = tokens strictly between the final
binding's value and `QUERY`. 6 cells; **train 600** episodes (seed group 1), **held-out 204** (seed group 2).

## Controls

- **Order-2 count** `C`: held-out accuracy **0.0000** (0/204) against chance `1/64 = 0.015625` → **at chance** (panel
  not solvable by a two-token count prior).
- **(g) hand-coded overwrite table** (key → last value): accuracy **1.0000** (204/204) → panel is solvable by a known
  mechanism at this scale.

## Result (`run4`, primary = plan-compliant **hard-select** readout, §5.1)

`crates/uor-r4-core/src/bin/kvar-recall.rs`, `serve_scores`: `scores = wo@h`, plus `1 << 8` at `argmax(mem)` when the
store read gate fired; no softmax/float in the served path. Held-out (204), 2000-resample episode-cluster bootstrap.

| arm | served acc | acc CI95 | bits/query | bits vs `C` (CI95) | float diag | additive diag |
|---|---|---|---|---|---|---|
| (a) current recurrence, no gate | 0.0098 | [0.0000, 0.0245] | 6.1130 | −0.1130 [−0.178, −0.045] | 0.02 | — |
| (a) seed 2 | 0.0098 | [0.0000, 0.0245] | 6.0113 | −0.0113 [−0.088, +0.064] | 0.01 | — |
| (b) hard gate only | 0.0196 | [0.0049, 0.0392] | 6.0970 | −0.0970 [−0.164, −0.036] | 0.02 | — |
| (b) seed 2 | 0.0196 | [0.0049, 0.0392] | 6.1515 | −0.1515 [−0.222, −0.083] | 0.03 | — |
| **(c) gate + token-addressed overwrite store** | **0.8235** | **[0.7696, 0.8725]** | **2.2510** | **+3.7490 [3.137, 4.311]** | 0.54 | 0.01 |
| **(c) seed 2** | **0.7157** | **[0.6520, 0.7745]** | **2.7627** | **+3.2373 [2.580, 3.924]** | 0.45 | 0.02 |

Per-cell select-served accuracy for (c): seed1 **0.94 / 0.82 / 0.71** (lag 4/16/64, K=4) and **0.94 / 0.88 / 0.65**
(K=8); seed2 **0.94 / 0.76 / 0.59** and **0.65 / 0.76 / 0.59**. Bits/query rise with lag (≈0.83 → 4.33 for seed 1),
i.e. performance degrades with distance but stays far above chance through lag 64.

D5 per token (trained, quantized): (c) **8771** slots inspected / **8672** nonzero selects (emb row 64, `wh` 4096,
`wf` 192, `bh` 64, `wo` 4096, plus gate/store rows).

## Decision (pre-registered)

`beats_chance_and_count_2bits = true`; `g_solves_panel = true`; `panel_valid = true`;
`store_write_lever_c_beats_b = true`; **`branch = ACCEPT_MEMORY_MECHANISM`** (see `run4/receipt.json#decision`).

**The gated token-addressed overwrite store extends the horizon where the plain recurrence and the gate-only
recurrence are at chance.** This is the memory mechanism the mathematics and CS reviews and the K3 gate predicted
would be the lever; it is **not** a geometric result.

## What changed during execution (recorded honestly)

The first pass (`run3`) evaluated an **additive** readout, `scores = wo@h + (mem << beta_shift)`, and rejected. That
additive form was a **deviation** from the pre-registration §5.1 ("reads are argmax/compare-select"; no soft read).
Promoting the plan-compliant **hard select** to the primary served read (`run4`) is the correct evaluation, and it
accepts. The additive form is retained only as a labelled diagnostic (0.01/0.02) and the float training model as a
diagnostic (0.45–0.54). The store's read/write gates are trained with a soft (sigmoid) objective and served with hard
thresholds — a declared train/serve realization; the accept is at that declared scope.

## Limits

- Synthetic authored KVAR panel only. **No** language, capability, reasoning, coding, energy or
  geometric-superiority claim.
- Training used the soft additive readout; the primary served read is the hard select. Re-training end-to-end with the
  hard select (or quantization-aware training) is untested.
- **`(f)` the ordinary equal-cost matched control is `NOT_RUN`**, so "the memory beats an ordinary equal-cost gated
  memory" is **not yet established**; `(b)` (gate-only, model-family control) is the current comparator.
- **`(h)` the geometry gate is `NOT_RUN`** — the owner's "vectors + Hamiltonians" hypothesis is therefore **not
  tested here**; this result is about memory, not geometry. `(d)`, `(e)` and the equal-work stateful n-gram are also
  `NOT_RUN`.
- Two seeds; 204 held-out queries; the lag-64 cells rest on 34 episodes each.

## One recommended next milestone

Re-train arm (c) end-to-end with the **hard-select readout as the training objective** (straight-through), then run
the deferred **`(f)` equal-cost ordinary gated memory** and **`(h)` geometric parameterisation** on the same frozen
panel. That converts this milestone into a served result under its own training objective and finally tests the
geometry gate (`(h)` vs `(f)`), where a tie is the expected outcome and would retire the *geometry claim*, not the
memory.
