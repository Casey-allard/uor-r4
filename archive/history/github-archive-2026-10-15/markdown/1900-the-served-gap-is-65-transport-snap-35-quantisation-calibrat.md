# #1900 — The served gap is 65% transport snap, 35% quantisation; calibration does not help (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T12:40:56Z
- **Updated:** 2026-10-09T13:20:01Z
- **Closed:** 2026-10-08T12:42:36Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1900
- **Merged:** 2026-10-08T12:42:36Z
- **Merge commit:** 401e338723eab329d3da3a4cc0d1e072d7ccce9f
- **Base / Head:** main / UOR-Foundation:deepseek/fidelity-decomp
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## The question

The end-to-end chain serves **0.93277** bits/byte where its own fused float model scores **0.87755** on the same held-out stream — a **0.05522 BPB** gap. Is it a knob or a property of the served representation?

## The decomposition

All on `chat-v0-p2/heldout` (sha `e5f400b0…`), 512-window float reference, 24,576-target integer evaluations, byte basis 2.837427:

| stage | nll | bits/byte | cost |
|---|---:|---:|---:|
| fused float reference | 1.7259248122572899 | 0.87755 | — |
| transport-snapped reference | 1.7963021881878376 | 0.91333 | **+0.03578** |
| multiplier-free D11 (export a1, 64 calib windows, damp 0.01) | 1.8345359993679817 | **0.93277** | **+0.01944** |
| **total served-to-fused gap** | | | **+0.05522** |
| share from the transport snap | | | **64.8 %** |
| share from export quantisation | | | **35.2 %** |

## Two findings

1. **Export tuning does not buy fidelity here.** A second arm with **4× the calibration windows** (256) and **10× finer damping** (0.001) scores **0.93368 — 0.00091 BPB *worse***. The evaluation is deterministic (same positions, integer logits, top-1 agreement 1.0 between engines), so this is a property of the artifacts, not sampling noise. **The "calibration is the knob" hypothesis is retired.**
2. **The gap is two thirds transport snap, one third quantisation.** The snap is not an export setting — it's the representation choice mapping continuous transport onto the 120 unit icosians, costing 0.03578 BPB by itself. A finer grid can only address the smaller term.

**Therefore the lever is the representation, not the exporter.** If the served gap is to close, it closes by making the snapped representation richer or by reducing what snapping discards.

## Also recorded

- **Reproduction, measured**: an independent retrain on a different GPU (5090 → 4090), binary commit and pod reproduces the served nll to **2.1e-6** with top-1 agreement 1.0 — the execution policy's *"do not claim bitwise cross-backend reproducibility without measuring it"* case, now measured.
- **Job contention**: identical recipe at **218.6 k tok/s idle** vs **126.9 k** with a CPU-bound evaluation alongside (42 % slowdown). "One job per GPU" doesn't cover a competitor that isn't on a GPU.

Scope: one model, one corpus, one stream, one basis, single training seed, two export configurations differing only in `calibration_windows` and `damp`. Evidence: `docs/evidence/served_fidelity_decomposition_2026-10-08.txt`.

References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

