# #1903 — Correct the export method: tonight's artifact is GPTQ; the untested arm is round-to-nearest (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T12:56:22Z
- **Updated:** 2026-10-09T13:21:46Z
- **Closed:** 2026-10-08T12:58:03Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1903
- **Merged:** 2026-10-08T12:58:03Z
- **Merge commit:** 6ba41f51709c84f614f1b09390b02a299d2b337a
- **Base / Head:** main / UOR-Foundation:deepseek/quant-method-correction
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## A second correction, found by trying to run the "GPTQ arm"

Round 21 proposed exporting the model with GPTQ, on the premise that every artifact tonight used round-to-nearest. Reading `export_mode` to select that arm showed the method is chosen by **whether calibration tokens are supplied** (`geometric-stack.rs:2954-2975`):

```rust
let quantizer = match &calibration_tokens {
    Some(path) => json!({"method": "gptq", "calibration": …, "damp": …}),
    None       => { let method_name = export_quantizer_method(&model); json!({"method": method_name}) }
};
```

Both round-19 arms passed `calibration=`, and the artifact record confirms it: `"quantizer": "gptq"`, `"calibration_positions": 98304`.

**Three published statements were wrong:**

1. **"Exported round-to-nearest"** (round-10 comment, `chat_stack_served_d11_2026-10-08.txt`, `current-state.md`) — the served artifact is **GPTQ**. Corrected chain: fused float **0.87755** → transport-snapped **0.91333** → **GPTQ-served 0.93277**.
2. **Round 21's premise** that the GPTQ quantiser was "unused tonight" — it was in use throughout, and the 0.9379-vs-0.9244 motivation I gave compared *different artifacts on the same method*, not different methods. It does not support the conclusion I drew.
3. **The untested arm is the opposite one** — round-to-nearest, i.e. export *without* calibration.

## What stands unchanged

- **Round 19's a1-vs-a2 result** (4× calibration windows, 10× finer damping → 0.00091 BPB *worse*) is a comparison *within* GPTQ and stands as measured.
- **Round 20's attribution** (97 % of quantisation loss in the weight matrices) is independent of which quantiser produced the artifact.

The record is **amended, not rewritten** — the original round-21 reasoning is kept with the correction attached, alongside the earlier 0.9550 → 0.93277 correction.

## The experiment that is now informative (running)

Same model, exported with **no calibration** — the export record already confirms `"quantizer": "round_to_nearest"` — then `d11-evaluate` on the same held-out stream, against the GPTQ artifact's nll `1.8345359993679817`:

- **RTN worse** → method choice is already buying the recovery; keep calibration in the export path.
- **RTN better or equal** → GPTQ's calibration is *hurting* this model; the export path should default to no calibration, and round 19's result reads as "more calibration of a harmful method is worse still".

Both decisive; one export plus one integer evaluation. No result is claimed until it lands.

References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

