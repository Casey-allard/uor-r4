# #244 — Multi-membership beam materializes ~4 store keys per position — decide store-time vs query-time expansion (#230 follow-up: size)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-07-28T03:51:35Z
- **Updated:** 2026-07-29T14:55:29Z
- **Closed:** 2026-07-29T14:55:29Z
- **Labels:** enhancement, graph-compiler
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/244
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

### Summary
Follow-up to #230, store-size half. The 3.71× key inflation (344,604 vs 92,903 f32 control / 89,200 certified) is **not** the assignment math — it is store construction: `assign_memberships_plain` (`runtime.rs:370-432`, `TOP_M_MEMBERSHIPS = 3` beam + forced nearest-prefix fallback appended with `u32::MAX`) feeding `add_evidence_multi` (`runtime.rs:721-747`), which inserts evidence under **every** beam key at every depth — up to ~4 keys/position vs exactly 1 in the f32 ablation (`certify.rs:397-420`). 344,604 / 92,903 ≈ the beam width. PROOF.md P2 recorded near-parity (89,200 vs 86,574), so this is post-certification behavior, as #230 inferred.

### Decision to make
Is store-time beam materialization worth 3.71× artifact size? Alternatives:
1. Single-key store + **query-time** beam expansion (reads are cheap; writes are what multiplied).
2. Cap materialization by evidence weight (only materialize alternate keys above a threshold).
3. Keep, but measure: quantify what the beam smoothing actually buys in top-1/agreement vs single-assignment.

### DoD
Store keys within ~1.2× of single-assignment at equal-or-better top-1/agreement, measured by `certify` (Maura's harness, built-in control). Whichever option wins, record the tradeoff measurement in BASELINE.md.

*Mechanism verified in the #230 triage. The accuracy half of #230 is tracked separately (graded-signature assignment).* 


## Comments (4)

### comment 5104717759 — 2026-07-28T13:24:23Z — Casey-allard

Taking this (assigned). Plan, branch `issue-244-store-decision` — **measurement first, decision second**:

`certify` gains two rows completing the decision matrix this issue needs (machinery already exists; ~110 additive lines):

- **A-single**: the shipped mul-free codes, single-key store (`build_store_generic` with the nearest-class code — no `add_evidence_multi` beam materialization). Directly answers "what do the 3.71× extra keys buy?"
- **A-single + query-beam**: same single-key store, with the identical multi-membership beam applied at **read time** — candidate keys per depth from `assign_memberships_plain`, distributions merged (`merged_beam_distribution`, unit-tested) before argmax/backoff. Tests whether the smoothing survives moving from write-side to read-side.

Decision rule per the issue DoD, after one legacy `certify` run: if A-single (or +query-beam) holds accuracy within noise of A-binary at ~1/3.7 the keys, a follow-up PR makes single-key the shipped store construction (with beam reads if they carried the win); if the materialized beam genuinely buys accuracy, we keep it and record the measured price in BASELINE.md.

The measurement run itself is deferred until the in-flight D3 observation finishes (same machine; keeping the timing clean) — numbers will be posted here before any construction change ships.


### comment 5113522202 — 2026-07-29T05:13:50Z — Casey-allard

## Four-way store matrix — measured (2026-07-29)

Machine: Casey's Mac, same corpus/checkpoint as the validation harness (source κ `blake3:0ae733…`, 60,816,028 bytes; corpus 150,000 tokens / 754 stories / 30,036 held-out). Trust chain for this run: teacher floor **1.4737 bits/token** (finite — #231 clamp live, **16/30,036** zero-quantized-weight collisions counted), teacher ceiling 69.5%, equality witness kernel==plain on 512/512 sampled positions, op census multiply **0**. The A-binary/A-f32/B rows below reproduced **bit-for-bit across two independent runs on this machine** and match Maura's matrix exactly.

| Store variant | top1 | agreement | WB bits/token | keys |
|---|---|---|---|---|
| A-binary (mul-free runtime, write-time membership fan-out) | 28.2% | 31.9% | 9.7868 | 344,604 |
| A-f32 (ablation, multiplies at assignment) | 32.1% | 36.4% | 9.6982 | 92,903 |
| **A-single (single-key store, no fan-out)** | 27.0% | 30.5% | 10.2304 | **134,733** |
| **A-single + query-beam (read-time expansion)** | **28.2%** | **31.9%** | 9.8742 | **134,733** |
| B bit-prefix (no codebook classes) | 22.1% | 25.0% | 11.8762 | 405,548 |
| C R4G1 (graph path) | *pending* | *pending* | n/a | — |

## Decision recommendation: ship the single-key store + read-time query-beam

Pre-registered rule: single-key ships if it holds within ~0.3pp of A-binary at a fraction of the keys.

- **A-single alone fails** the rule: −1.2pp top1 / −1.4pp agreement. The membership fan-out is doing real work.
- **A-single + query-beam passes at 0.0pp** (to printed precision): top1 and agreement match A-binary exactly, at **134,733 vs 344,604 keys — a 2.56× smaller store**. Moving the fan-out from write time to read time loses nothing measurable in argmax metrics.
- The measured price: **+0.087 WB bits/token** (9.8742 vs 9.7868) and up to TOP_M lookups+merge per level per query instead of one (still mul-free: compares + table reads). Storage↓61% for a small read-cost increase and a sub-0.1-bit WB delta.
- Note on the anticipated \"1/3.7 keys\": that denominator came from A-f32's 92,903 single-key count. The binary assignment path's single-key store lands at 134,733 keys (noisier codes → more distinct prefixes), so the realized ratio is 1/2.56. The rule's accuracy clause is what binds, and query-beam meets it.

Follow-up PR proposal: `build_store` writes primary-code keys only; `predict` merges the TOP_M membership prefixes at read time (the certifier's `eval_query_beam` is the reference semantics). Store format unchanged (TLS1), so κ-pinning discipline carries over.

## C-row status + cost finding

The C R4G1 evaluation (first honest post-#232/#237 graph measurement — readiness probe classified **Ready**, i.e. scored emissions exist) is still running: ~6h in the eval phase at ~0.5–0.7s/position, profile 100% in `R4G1Runtime::predict_distribution`'s linear scans. Row will be appended here when it lands. The cost itself is a finding: the graph path is ~5–6 orders of magnitude slower per query than the store path — filed as #277 (VP-tree exact-NN index) with this run as the measured motivation.

Gap-to-teacher note: A-f32 remains the accuracy ceiling of the family (32.1/36.4) — the binarization price (#243's Design R/G target) is unchanged by tonight's rows; the store-architecture and addressing questions remain orthogonal, as the design doc assumes.

### comment 5119379239 — 2026-07-29T14:48:16Z — Casey-allard

## C-row amendment: measured, and the answer is zero

**Full-eval infeasibility, recorded:** the first C-row attempt ran the complete 30,036-position held-out eval and was still unfinished after **~13.5h in-phase** (>1.5s/position, profile 100% in `predict_distribution` linear scans) when it was terminated by an external SIGTERM. That duration is now the quantified motivation on #277.

**Bounded amendment run** (this morning): artifact reuse κ-verified (`blake3:81db44…` — byte-identical to both prior fresh compiles), equality witness 512/512, A-binary sanity row reproduced bit-for-bit a **third** time (28.2/31.9/344,604). C evaluated on a deterministic 1,002-position stride subsample:

> **C R4G1 (graph path, deterministic subsample n=1002): top1 0.0% | agreement 0.0% | probe: 64/64 scored**

This is the first honest graph-path measurement (post-#232 readiness guard, post-#237 coherent contexts) — and it says the graph path, as converted, predicts **nothing**. 0/1002 on both metrics is not weak performance; even a most-frequent-token constant would score several percent. The graph emits varied, scored predictions (probe passed: scored on 64/64, non-constant) that are systematically outside the plausible next-token set — while the **same artifacts and store** predict 28.2% through the store path. The evidence is intact; the graph readout of it is broken. Diagnosis issue filed separately with hypotheses.

**Compression certificate** (completed for the first time across these runs): container save→load→save byte-identical (462,092 bytes, κ stable); token-representation rate–distortion mean cosine 0.9565→0.9676 over depths 1–4; end-to-end artifact 11,791,381 bytes vs 60,816,028 source checkpoint = **5.2×** at the certified residual.

**The #244 matrix is now complete.** Final state: ship recommendation stands (single-key store + read-time query-beam: matches A-binary at 2.56× fewer keys, +0.087 WB bits); C row measured at 0.0% (subsample, labeled); full-eval cost recorded on #277. Unblocked next: #243 Phase A (PR #270 review pending Casey/Ari/Alex), the single-key+query-beam runtime PR, and the graph-readout diagnosis.


### comment 5119498240 — 2026-07-29T14:55:29Z — Casey-allard

Closing: the four-way matrix is complete (see the decision comment and C-row amendment above) — decision recorded as single-key + read-time query-beam, implementation tracked in the new issue just filed; C-row perf and readout defects tracked in #278/#280.


---
_Comment/review ids in this file: 5104717759, 5113522202, 5119379239, 5119498240_

