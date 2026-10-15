# #242 — HF observation tokenizer is not the model's BPE — compiled artifacts inherit the distortion

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-07-28T03:50:40Z
- **Updated:** 2026-07-28T15:52:46Z
- **Closed:** 2026-07-28T05:03:24Z
- **Labels:** bug
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/242
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

### Summary
The HF observation path tokenizes with a bespoke tokenizer that is **not** the teacher model's BPE: `crates/uor-r4-core/src/transformerless/scenarios.rs:150-198` — iterative greedy pair merging preferring the *lowest merged token id*, `vocab.json` ingested as raw bytes, **no merges table at all**. The SmolLM2 teacher is teacher-forced onto segmentations its training distribution essentially never contains.

### Evidence
- Teacher floor 14.84 bits/token vs uniform ≈ log2(49152) ≈ 15.59 on its own held-out set (#231's second anomaly) — the metric is faithfully reporting garbage-in, not a numerics bug (HF eval path is f64, max-stabilized, clamped: `uor-r4-graph-cli/src/lib.rs:1434-1444`).
- Same root-cause class as the Jul 26 teacher-oracle fixes (ChatML `<|im_start|>` BPE fragmentation, worked around by plain-text prompts) — the tokenizer mismatch was patched around, not fixed.
- Contamination is real on this path: observation records contexts, `next` targets, and teacher top-8 distributions all conditioned on the wrong segmentation (`observation_text.rs:662,680`), so **every HF-path compiled artifact inherits the distortion**. A plausible root cause of incoherent standalone generation.

### Fix
Load the model's real tokenizer (merges + special tokens; vendor or parse `tokenizer.json`) for HF-path observation, evaluation, and serving. Legacy llama2.c path unaffected.

### DoD
- Teacher argmax-vs-next accuracy on held-out ≫ chance (instrument per #231's suggested counter).
- Artifact recompiled with correct segmentation; before/after Gate C / M.V.G. numbers recorded.
- Blocks meaningful measurement of the geometric merge work (graded-signature assignment, session-signature lane) — prioritize accordingly.

*From the #230/#231 verification pass; see also #237 (measurement-side context defect).*


## Comments (3)

### comment 5099825062 — 2026-07-28T04:08:06Z — Casey-allard

Taking this (assigned). Plan, branch `issue-242-hf-bpe-tokenizer`:

1. **Replace the bespoke merge rule with true BPE** for the HF path: parse the model's `tokenizer.json` (vocab + ordered merges + special tokens + byte-level pre-tokenization scheme) from the pinned snapshot in `.uor-models/sources/<model>/`, and implement merge-rank-ordered BPE (lowest merge *rank* wins, not lowest merged token id). Special tokens matched atomically before pre-tokenization.
2. **One tokenizer per path**: HF observation (`observation_text.rs`), evaluation (`graph-cli` evaluate-report), and serving prompt encoding all resolve the same tokenizer instance for the selected model; the legacy llama2.c path keeps its existing tokenizer (its κ-pinned baselines depend on it — untouched).
3. **Provenance**: compute a real `tokenizer_address()` (blake3 of canonical tokenizer bytes) replacing the placeholder string, so artifacts record which tokenizer produced them — mismatch at load becomes detectable rather than silent.
4. **Validation before recompile**: round-trip encode/decode tests against reference encodings; the decisive metric is teacher argmax-vs-next on held-out ≫ chance (vs today's ≈uniform floor).
5. **Recompile + re-measure**: fresh HF artifact, before/after evaluate-report numbers posted here.

Scope note: compiler/observation-side code — floats and allocation permitted; the deployed runtime kernel is untouched (P-4 unaffected). Rollout risk is contained to the HF path.


### comment 5100175948 — 2026-07-28T05:04:10Z — Casey-allard

Merged in #253 (`c97e82c`). Remaining follow-through:

1. **Serving-side tokenizer** — filed as a separate issue (see reference above this comment).
2. **Measurement run (the DoD's decisive metric)**: recompile the HF artifact with the correct tokenizer and post before/after `evaluate-report` numbers here. Success = teacher argmax-vs-next on held-out ≫ chance, teacher floor well below the ≈uniform 14.84 bits that motivated this issue. This is a long-running local job (observation + compile + eval on the pinned Mac) — scheduled next.


### comment 5106447103 — 2026-07-28T15:52:46Z — Casey-allard

## Measurement follow-through (post-#253): three-way comparison on the D3 held-out stream

| Configuration | top1 vs next | argmax agreement | artifact WB bits/token | teacher floor | artifact overhead |
|---|---|---|---|---|---|
| **Before** (bespoke tokenizer, old artifact — the run that motivated this issue) | — | — | 37.15 | 14.84 (≈ uniform 15.59) | **+22.3** |
| **Interim** (old-tokenizer records, fresh artifact) | 5.0% | 2.6% | 15.12 | 13.12 | +2.0 |
| **Final** (correct byte-level BPE end-to-end: fresh observation → recompile → eval) | 2.9% | **23.1%** | 16.49 | 13.45 | **+3.0** |

**What the fix demonstrably did:**
- **Teacher-argmax agreement 2.6% → 23.1%** (9×). This is the metric this issue predicted: artifacts compiled from wrong-segmentation observations could not mimic the teacher; with consistent tokenization end-to-end the compiled artifact now matches the teacher's argmax on nearly a quarter of held-out positions.
- **Artifact overhead vs its own teacher: +22.3 bits → +3.0 bits.** The compiled store is no longer fighting its observation corpus.
- top1-vs-sampled-next *dropped* (5.0 → 2.9%) while agreement rose — consistent with #234's memorization finding: the degenerate old segmentation was easier to memorize; the honest corpus is harder, and the artifact is now a teacher-mimic rather than a segmentation-artifact-memorizer.

**Residual anomaly, flagged honestly:** the teacher floor itself remains ~13.4 bits (uniform ≈ 15.6) — better than 14.84 but far above the ~4–7 bits a healthy 135M model should score on Simple-Wikipedia text. The tokenizer was *a* confounder, not the only one. Candidates for the remainder: 128-token sequence truncation effects at observation (490 articles truncated), context handling in the floor computation, or teacher-side prompt framing. Needs its own investigation before the floor is quoted as a meaningful bound — filing as follow-up.

**Provenance caveats:** run used a refetched D3 corpus (pinned corpus was not re-materializable on this machine; refetch drifted ~0.5% in text bytes vs the manifest — reproducibility issue filed separately with the fetch script). Fresh-corpus merged κ: `blake3:4010215c…`. Eval report: `/tmp/i242-eval-final.json` (machine-local).



---
_Comment/review ids in this file: 5099825062, 5100175948, 5106447103_

