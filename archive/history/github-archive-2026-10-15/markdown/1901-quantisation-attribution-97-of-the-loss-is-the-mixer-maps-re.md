# #1901 — Quantisation attribution: 97% of the loss is the mixer maps (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T12:46:46Z
- **Updated:** 2026-10-09T13:21:43Z
- **Closed:** 2026-10-08T12:48:25Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1901
- **Merged:** 2026-10-08T12:48:25Z
- **Merge commit:** 1e81355101c4c1d7f11e584b60c9c169478f1271
- **Base / Head:** main / UOR-Foundation:deepseek/quant-attrib
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## The question

Round 19 decomposed the served-to-fused gap (0.05522 BPB) into **64.8 % transport snap** and **35.2 % export quantisation**, and retired calibration as the knob. This asks: inside the quantisation third, *what* is losing the information?

`geometric-stack rounding-attribution` on the 19.9 M chat stack's export, chat-v0-p2/heldout, 64 windows:

| group | parameters rounded | cost (nats) | share | top-1 vs float |
|---|---|---:|---:|---:|
| `all` | every embedded parameter | 0.0173756 | 100.0 % | 0.9244 |
| `all_maps` | every layer's `mixer_maps` + `mlp` + `embed`/`head` | 0.0168514 | **97.0 %** | 0.9256 |
| `all_scalars` | every layer's `mixer_scalars` | 0.0004643 | **2.7 %** | 0.9908 |
| `l7.mlp` | last layer's MLP alone | 0.0006545 | 3.8 % | 0.9815 |

**The mixer maps carry essentially the entire quantisation loss.** Rounding the scalars alone costs 0.00046 nats and keeps **99.1 %** top-1 agreement; rounding the maps costs 0.01685 and drops agreement to **92.6 %**. Rounding everything costs almost exactly what the maps cost alone (0.01738 vs 0.01685) — so these are not separate additive problems; the maps *are* the problem.

## What it means

1. The quantisation term is **concentrated in one parameter class**, and is not a general grid-resolution question. A finer or differently structured representation for the mixer maps is the whole of the available recovery.
2. **It explains why round 19's export arm failed.** More calibration windows and finer damping change how the *grid is chosen*; they don't change how much of a map's information one cell can carry. The lever is the map's representation, not the fitting procedure.
3. The served gap now reads: **0.03578 BPB transport snap + 0.01944 quantisation**, of which **0.01886 is the mixer maps** and ~0.0005 the scalars. *Two thirds representation, one third one parameter class, almost nothing elsewhere.*

## Scope

One model, one export, one corpus, 64 windows, one run of a deterministic tool. Costs are nats/token relative to this model's own float evaluation *inside the same tool* — comparable to each other and to round 19 in sign and share, but slightly different in absolute value from the 24,576-target `d11-evaluate` figure because the scored position set differs. Evidence: `docs/evidence/quantisation_attribution_2026-10-08.txt`.

References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

