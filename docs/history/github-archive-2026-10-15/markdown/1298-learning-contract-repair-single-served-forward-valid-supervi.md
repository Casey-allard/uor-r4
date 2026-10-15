# #1298 — Learning-contract repair: single served forward, valid supervision, honest Adam — corrected gate passes (References #973)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-20T02:43:47Z
- **Updated:** 2026-10-09T13:17:39Z
- **Closed:** 2026-09-20T02:44:48Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1298
- **Merged:** 2026-09-20T02:44:48Z
- **Merge commit:** a3c5403587bd27b32399622ec0bebc010447b9f2
- **Base / Head:** main / UOR-Foundation:codex/learning-contract
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Repairs the three decisive defects from the learning-contract review, adds an authored capacity witness, and **the corrected small learning gate passes on the first run**. The conditional floating comparator was correctly not needed.

## Defect 1 — a parallel training forward with different units

The trainer kept a duplicated floating forward `z_train = R + b·2^(s−F)` against the served `z = (R + b·2^s)·2^-F` — 1,024× on the contextual term at F = 10 — while the backward applied `2^-F` anyway. Training now obtains its score from **`PriorCore::trace`**, the same integer computation greedy serving uses, with `2^-F/ln 2` applied once at credit to the integer score.

Evidence is non-vacuous by construction: the parity test forces nonzero contextual output (asserting the output codes left zero), nonflat bias, nonunit shifts and active bounds, requires trainer bits = served bits to `1e-12` on all 16 sequences, **and** asserts the old unscaled-context variant gives a *different* loss, so a regression cannot pass silently.

## Defect 2 — supervision that trained contradictory laws

Scoring every position of `[x,y,(x+y)%4]` trains `(x,y) → y` (four equally likely targets for a fixed x) and, over the repeated triple, labels encoding `b−a` and `a−b`. Supervision is now an explicit **per-position mask shared by training, loss and accuracy**, scoring only the final target; real-text scoring stays all `n−1`. Tested: one target per context, all four targets equally frequent, and either single input leaves the target uniform.

## Defect 3 — Adam dropped small gradients

`lowbit_core::adam_update` gated on `v_hat > 1e-12`, silently discarding legitimate updates (`g = 1e-7, lr = 0.05` gave 0 instead of ≈ `−0.04545`). The update is now unconditional, with an independent scalar reference over small/zero gradients and carried moments. The helper is shared with `lowbit_attention`, `geometric_attention` and `cold_prior`; **their recorded results were obtained under the earlier optimizer and are not claimed unchanged.**

## A real forward bug the old gate could not isolate

`trace` recorded the bounded-ReLU STE mask but **never applied the clamp**, leaving negative activations in the hidden vector. The authored capacity witness caught it immediately (all four scores equal at −20480).

## Also repaired

Real seeded **Fisher–Yates** schedule, exact once-per-pass coverage, a short final batch never wrapped to fill · **complete checkpoint identity** (masters, both moments, step, lr/betas/weight_decay/grad_clip, seed, data identity, numerical config, frozen bias) with stage-then-commit loading so a rejected load leaves the trainer byte-identical · **artifact bounds** with a declared i32 envelope (`bias_scale_bits > 27` rejected; `7 << 30` cannot represent the intended bias), checked sizes before allocation, element/code/shift checks, trailing-byte rejection, replacing an always-true `is_err() || true` test with real serialized field-offset mutations · corrected stale bias and initialization comments.

## Authored capacity witness (separate from every learned arm)

16/16 correct, mean CE **0.0772 bits** against the analytic `log2(1 + 3e^-4)`, export/reload identical. Finite representability only.

## The corrected gate

Frozen before results: accuracy ≥ 0.9, CE ≥ 0.10 bit below 2.0, positive permutation penalty. Run under a report root claimed exclusively before any model work, sealed and verified.

```
step    0: acc 0.2500  CE 2.0000   context-disabled 0.2500/2.0000   knockouts 0.2500 / 0.2500
step  128: acc 0.9375  CE 1.3546
step  512: acc 1.0000  CE 0.2107
step 2000: acc 1.0000  CE 0.0497
permuted context->target: acc 0.4375, CE 4.7836, penalty +4.7338 bits, association changed
GATE: accuracy PASS | CE margin PASS | permutation penalty PASS
```

Step 0 is **exactly** the constant control — the zero-exported-residual initialization working as designed. The context-disabled control and **both** position knockouts sit at chance throughout, so the prediction depends on both inputs rather than a marginal correction. The earlier 0.375 was an artefact of these defects, **not** a quantization limit. This is an all-pairs fitting/instrument gate on a 16-context authored task, not a generalization or language result.

## The conditional real-text stage was NOT executed

Prerequisites A–C now pass (single parity-tested forward; optimizer fixed against an independent reference; real permutation schedule; complete, tested checkpoint identity). The recorded reason is **remaining session budget**, not a failing gate and not a new approval stop, and the balance was not silently increased to cover it. No real-text manifest, artifact, checkpoint or generation was produced.

## Validation

`cargo fmt --all --check` clean · `python3 scripts/check_claim_wording.py` passed · `cargo test --release -p uor-r4-core --lib -- prior_learning adam_reference` → **12 passed, 0 failed** (plus the gate run).

Retained under `.uor-models/prior-learning-gate-2026-09-20/attempt-1`: `result.json`, `curve.json`, `predictions.json` (all 16 integer-score predictions per checkpoint), a 264-byte `prior_only.cpl2` and a 5,196-byte `prior_only.ckpt`. Ledger: 1,200,000 ms measured → `151,038,565 / 154,400,000` (~56.0 min remaining). No chat/coding/reasoning/energy claim; energy UNAVAILABLE.

References #973, #820


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

