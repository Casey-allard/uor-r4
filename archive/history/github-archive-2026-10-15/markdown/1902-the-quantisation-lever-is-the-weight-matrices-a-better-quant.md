# #1902 — The quantisation lever is the weight matrices; a better quantiser is already implemented (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T12:50:43Z
- **Updated:** 2026-10-09T13:21:44Z
- **Closed:** 2026-10-08T12:52:22Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1902
- **Merged:** 2026-10-08T12:52:22Z
- **Merge commit:** d7453603e3537d375fcd60a7120228260660794f
- **Base / Head:** main / UOR-Foundation:deepseek/quant-lever
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What the attribution groups actually are

Round 20 put **97 %** of the quantisation loss in the `mixer_maps` group. Reading the source (`geometric-stack.rs:3374`) shows that group is exactly the recurrence and read path **weight matrices**:

```
mixer_maps     rec_norm/rec.in/rec.gate/rec.out.weight,
               read_norm/read.query/read.key/read.value/read.null/read.out.weight
mixer_scalars  rec.conv.weight/bias, rec.gate.bias, rec.decay,
               read.null.bias, read.age, read.log_beta, read.offset
mlp            mlp_norm/mlp.gate/mlp.up/mlp.down.weight
```

They're ordinary dense learned matrices, **not algebraic tables**. So the finding is *not* "geometric structure is being rounded away" — it's that **the loss sits where the parameters sit**, in the linear maps of the recurrence and read path. The scalars (dozens per layer) cost 2.7 %; the maps cost 97 %.

## Why round 19's failure was expected

Calibration and damping choose the **grid**; no grid choice changes how much of a matrix one cell can carry. A method with **error feedback** can — because it accounts for what each rounded weight does to the layer's output instead of rounding each weight independently.

## The lever is already in the tree

`export_mode` carries a second quantiser — `"method": "gptq"` at `geometric-stack.rs:2956` — alongside the round-to-nearest path every artifact tonight used. The project ran it once before, on the `d11-l2` artifact, recording **`top1_agreement` 0.9379** against its float reference. Tonight's round-to-nearest artifact reaches **0.9244** on the comparable comparison. **The alternative quantiser measured better where both exist.**

## The experiment, with its prediction

Export the same 19.9 M chat stack with GPTQ, evaluate with `d11-evaluate` on the same held-out stream, compare against `nll 1.8345359993679817`:

- **Below it** → the quantisation third is recoverable by method choice, ceiling = the 0.01886-nat maps term.
- **At or above it** → the quantiser is not the lever on this model, and the remaining served gap is the transport snap alone (0.03578 BPB measured).

Both outcomes decisive, **no training needed**, one export + one integer evaluation per arm. A second lever on the same target: reallocate bits from scalars (0.00046 nats) to maps (0.01685) — the attribution already quantifies the trade.

**Scope**: the 0.9379 and 0.9244 figures come from different artifacts, corpora and tools, cited as motivation, **not** as a controlled comparison. The controlled comparison is the experiment above, which has **not been run** — no result is claimed for it. Evidence: `docs/evidence/quantisation_lever_2026-10-08.txt`.

References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

