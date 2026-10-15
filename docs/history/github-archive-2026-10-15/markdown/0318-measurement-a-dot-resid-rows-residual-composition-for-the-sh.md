# #318 — Measurement: A-dot-resid rows — residual composition for the shift-add dot path

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-08-01T00:45:33Z
- **Updated:** 2026-08-01T21:38:18Z
- **Closed:** 2026-08-01T21:38:18Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/318
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Follow-up to #243 / #310. The #310 run's matrix has one missing composition cell: the A-f32 ceiling (32.1/36.4) is (normalized work) × (norm-aware assignment) × (residual updates), but the buildable po2-dot rows (A-dot-po2 30.6, A-dot-po2x2 30.7) measured quantization **without** residual subtraction, and the round-2 scale analysis only covers raw-scale subtraction (recorded as negligible there). At **unit** scale — after L2-normalizing the centered bundle — subtraction is meaningful, and no row has measured dot assignment with residual updates at that scale.

New certifier-side rows (same harness discipline as #243/#310):

| Row | Isolates |
|---|---|
| A-dot-resid | f32 dot assignment on normalized work + per-stage centroid subtraction — the residual effect under the dot metric |
| A-dot-po2-resid | same, 1-term po2 tables (assignment AND subtraction from the same quantized values — the kernel-buildable form) |
| A-dot-po2x2-resid | same, 2-term po2 tables |
| A-dot-po2x3 | completes the no-residual terms sweep (1,2 → 3) |

DoD: rows run on the rows-only harness; if A-dot-po2-resid recovers a visible fraction of the 30.7→32.1 gap, Phase B (integer residual wiring for the dot path: per-stage integer centroid copies, add/sub only — contract-clean) gets a design note; otherwise recorded negative result like #310.


## Comments (4)

### comment 5148735062 — 2026-08-01T01:00:38Z — auser

## Measurement: dot-residual composition — POSITIVE result

Fresh-era rows-only run (`R4_CERTIFY_ROWS_ONLY=1`, 150k/754 corpus; equality witness 512/512; baselines reproduce the #310 run exactly — A-f32 32.1/36.4, A-dot-po2x2 30.7/34.3).

| Row | top-1 | agreement | WB bits | keys |
|---|---|---|---|---|
| A-f32 (ceiling) | 32.1 | 36.4 | 9.70 | 92,903 |
| **A-dot-po2x2-resid (buildable)** | **31.2** | **35.3** | 10.13 | 96,080 |
| **A-dot-po2-resid (buildable)** | **31.1** | **35.2** | 10.18 | 89,704 |
| A-dot-resid (f32 values) | 31.0 | 35.0 | 10.12 | 96,805 |
| A-dot-po2x3 (sweep extension) | 30.4 | 34.2 | 9.65 | 47,638 |
| A-dot-po2x2 (shipped shape) | 30.7 | 34.3 | 9.64 | 47,418 |

Findings:

1. **Residual composition recovers ~36–48% of the remaining gap** to the ceiling (+0.5 top-1 / +1.0 agreement over the shipped shape) — meets this issue's DoD; Phase B is justified.
2. **The po2 value-set costs nothing** once residuals carry the refinement (po2-resid 31.1 ≥ f32 dot-resid 31.0) — best possible outcome for kernel realizability.
3. **Terms sweep saturates at 2** (x3 regresses to 30.4).
4. **Trade-off on record**: WB worsens ~0.5 bits/token and keys roughly double — residual codes disperse. Phase B's DoD includes a store-shape decision on the #244 harness with a bounded WB regression, and a 3× bit-for-bit reproduction requirement before kernel code.

Branch `issue-318-dot-resid`: `b65ec30` (the rows) + `eb090d9` (`docs/dot_residual_phase_b_design.md` — po2 norm fold via L1 bit-length as the division-free normalization candidate, integer centroid copies per the token stage-book precedent, WB/key-dispersion criterion, phasing). Local gates: clippy/fmt clean on the touched crate; full workspace gates run before merge per batch discipline.


### comment 5149642898 — 2026-08-01T03:45:13Z — auser

## Phase A.5 verdict + reproduction certificate + data-scaling result

### Reproduction (design-note gate): PASSED

Three sequential rows-only runs (`R4_CERTIFY_ROWS_ONLY=1`): identical artifact κ (`blake3:cf64fe7493d0e366e322154853955246d14bd1c9de59595a3e780e28ad4ca20d`) and bit-for-bit identical measurement content (only progress timestamps differ). The reproduction requirement for Phase B is met.

### Phase A.5 rows (150k era, 3× reproduced)

| Row | top-1 | agreement | WB |
|---|---|---|---|
| A-dot-po2x2-resid (Phase A leader) | 31.2 | 35.3 | 10.13 |
| A-dot-po2-resid-cpy8 | 31.1 | 35.1 | 10.17 |
| A-dot-po2-resid-cpy16 | 31.1 | 35.1 | 10.16 |
| A-dot-po2-nf-resid (po2 norm fold, CONST 4) | 30.7 | 34.7 | 10.19 |

- **Copy width**: i8 ≡ i16 at 150k → i8 (295 KB) settled. Integer-copy subtraction costs nothing vs the po2-table reference.
- **Norm fold as drafted fails its ≤0.1pp gate** (−0.4/−0.5 vs true norm) — but still beats the shipped row (30.5/34.0). See the 300k data below, which reframes this.

### Data scaling (300k corpus, 2× — measured 2026-07-31; pinned 150k corpus restored intact after the run)

| Row | 150k | 300k | Δ |
|---|---|---|---|
| A (shipped) | 30.5 / 34.0 / 9.01 | **32.3 / 35.8 / 7.90** | **+1.8 / +1.8 / −1.1 bits** |
| A-f32 (ceiling) | 32.1 / 36.4 | **34.6 / 38.9** | +2.5 / +2.5 |
| A-dot-po2x2-resid | 31.2 / 35.3 | **33.8 / 38.2** | +2.6 / +2.9 |
| A-dot-po2-nf-resid | 30.7 / 34.7 | **33.5 / 37.7** | +2.8 / +3.0 |
| cpy8 / cpy16 | 31.1 / 31.1 | 33.7 / 33.9 | cpy16 +0.2 |

Findings:

1. **Data scaling is the largest measured lever** (+1.8pp top-1/agreement, −1.1 bits WB for 2× data; the ceiling gained +2.5pp — the curve is not flat). Gen cost for the extra 150k tokens: 8 minutes at 307 tok/s (idle machine, post-#316 top-k).
2. **Phase B's value grows at scale**: the residual margin over the shipped shape widened from +0.5/+1.0 (150k) to **+1.4/+2.1pp** (300k). The levers compound.
3. **The crude norm fold's penalty shrinks to −0.1/−0.1 at 300k** — within the design-note gate. If the corpus era moves to 300k, no mantissa refinement is needed; the simplest fold ships. (cpy16 edges cpy8 by +0.2pp at 300k — width choice reopens slightly at the new era.)

Eval-set caveat: the 300k held-out covers different stories than the 150k held-out; the +1.8pp delta is far above plausible eval-set noise.

**Consequence**: corpus-scaling adoption (new fixture era + κ re-pin) is filed as a separate maintainer-decision issue; Phase B should target the 300k era if adopted.


### comment 5152872544 — 2026-08-01T18:46:46Z — auser

## Phase B quality confirmation: TLA7 kernel residual path — PASSED (500k corpus)

Quality run on the Phase B implementation (PR #329, commits `43b35e3` + `6b583f2`), rows-only, 500k corpus:

- **Equality witness: 512/512** kernel == plain on real corpus data (the first run caught a routing gap — plain entry not routing to the residual path; fixed, with a non-vacuous fixture-corpus witness added).
- **A (shipped, TLA7 residual): 34.7 / 39.0 / WB 8.02** vs TLA6 shipped 33.4 / 37.3 / 7.15 — **+1.3 / +1.7pp**, inside the A.5-validated band (cpy8 35.3/39.6 true-norm, nf-resid 34.6/38.8 crude-fold) — the kernel path lands between them as designed (crude fold + i8 copies).
- A-f32 unchanged (35.7/40.3) — ablation path doesn't touch resid data, sanity holds.
- Artifact: TLA7, 1,346,836 bytes (+288 KB residual sections), norm-fold CONST 24.

Remaining Phase C items per the design note: WB/key-dispersion store-shape decision (TLA7 WB 8.02 vs TLA6 7.15 — the residual trade-off is real and needs the #244-harness decision), κ re-pin (tracks #327; recommended sequence: merge #329, then re-pin 500k + TLA7 in one pin), mantissa-bit fold refinement row, BASELINE.md.


### comment 5153562123 — 2026-08-01T21:38:17Z — auser

Closing on DoD: rows ran on the rows-only harness (PR #323, merge 0e49a5e — positive: A-dot-po2x2-resid 31.2/35.3 recovering ~36–48% of the 30.7→32.1 gap), Phase B design note merged in the same PR (docs/dot_residual_phase_b_design.md), and Phase B shipped as era TLA7 (PR #329, merge cff684f; quality confirmation in the 2026-08-01 comment above: 512/512 equality witness, 34.7/39.0 vs TLA6 33.4/37.3). The Phase C adoption items (WB store-shape decision, 500k+TLA7 κ re-pin, mantissa fold row, BASELINE.md/witness rows) were found to be untracked — #327 closed having delivered only the refresh tooling — so they now live in #335.


---
_Comment/review ids in this file: 5148735062, 5149642898, 5152872544, 5153562123_

