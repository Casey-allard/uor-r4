# #1308 — Fixed-feature ternary head projection: QG beats Q0 but not E (References #973)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-20T09:08:05Z
- **Updated:** 2026-10-09T13:17:36Z
- **Closed:** 2026-09-20T09:10:26Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1308
- **Merged:** 2026-09-20T09:10:26Z
- **Merge commit:** d92bd072b53891e83bc5ccf09c07a7ba3e5eba2b
- **Base / Head:** main / UOR-Foundation:codex/head-projection
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Executes the complete second document of the #1307 pair. The parent, E, S and F are never retrained; no sealed root is modified.

## Frozen boundary and evidence corrections

Parent `e_old`/`e_new`, their row shifts, bounded ReLU/normalization, bias codes/scale, `elements`, vocabulary, `F=10` and the argmax tie rule stay byte-identical; only the output head changes. No new Adam fit, wider layer, new precision, new corpus or prefix refit.

- The retained E/S `CPL2` files carried the placeholder tokenizer digest `[7;32]`. They are re-exported with the real derived digest `a7ac75b6...` as the **sole** change (packed codes, shifts, bias, elements and embeddings verified identical).
- The raw float file (exactly 4096×128 finite LE f32) is bound in the new manifest to parent, tokenizer, dimensions, `F`, bias, the actual teacher λ=(0.8,0.7) and source identity; offline-only, not a standalone served model, evaluated unrounded.
- Reproduction within the declared 1e-8-bit tolerance: E 4.4e-10, S 2.5e-10, F 2.6e-10.
- New `learner/head_projection.rs` factors and tests the screen helper (practical gain against the declared baseline, paired S-vs-E attribution, floating gate on S's own teacher KL), with a fixture where both heads beat the parent but smoothed does not beat empirical. The prescribed λ=(0.7,0.5) belongs to the 4,096 *reference* family and was not used by S/F; that deviation is disclosed and old artifacts are preserved.

## Calibration

Uncentered Gram `G = (1/N) Σ_c n_c h(c)h(c)^T` over the recovered 4,096 windows / 257,113 targets / 62,973 distinct contexts, accumulated in checked `i128`, never inverted, jittered, centered or rank-truncated (singular is allowed). **Q0** = `TernaryLinear::quantize(W_F, 4096, 128)` applied exactly once. **QG** = one activation-aware projection: candidate shifts `{s0−1, s0, s0+1, sE}` filtered to 0..=15, nearest-ternary and (where distinct) empirical-row seeds, exactly two coordinate sweeps per seed with strictly-negative acceptance of `dJ = 2δg_j + δ²G_jj` and ascending tie-break, independent recomputation of `e^T G e`, Q0 kept as an explicit candidate. Probe 64 rows in 0.005 s; full 4,096 rows in 0.30 s; seeds 2,098 nearest / 1,998 empirical.

## Measured outcome (36 docs / 288 windows / 17,342 targets)

| Head | dev micro bits/target | fit J vs F | zero frac. | 64-token latency |
|---|---:|---:|---:|---:|
| frozen parent | 7.558429 | — | — | — |
| **E empirical (incumbent)** | **7.170816** | 7.9336e9 | — | 0.0862 s |
| S smoothed | 7.188585 | — | — | — |
| F floating (offline) | 6.874144 | 0 | — | — |
| Q0 existing quantizer | 7.416178 | 3.9915e9 | 0.505 | 0.0834 s |
| QG activation-aware | 7.190872 | **6.2484e8** | 0.399 | 0.1038 s |

**Decisions.** Practical screen (QG vs E ≥ 0.10 with paired lower bound > 0): **FAILED** (CE_E − CE_QG = −0.020057 [−0.039538, −0.000183]). Q0 vs E: **FAILED** (−0.245363). Method attribution (Q0 − QG): **PASSED** (+0.225306 [0.200621, 0.250299]). Smoothing attribution (E − S): FALSE (−0.017769). Floating gate: RUN (S smoothed-teacher KL 3.186145).

**Interpretation.** Activation-aware calibration is decisively better than the existing quantizer, and QG reconstructs F's scores **12.7×** better than E does — yet E predicts best. **Lower squared-score reconstruction error does not imply lower cross-entropy.** Both bounded conversions are rejected, so **E remains the stronger qualified numerical baseline**; this does not establish ternary infeasibility, feature-rank saturation or a need to widen, and the earlier capacity-diagnosis reading is withdrawn.

## Common-objective matrix, controls, generation, cost

On the same 257,113 fit occurrences (bits/occurrence, teacher mass error 3.0e-14): true-label CE parent 7.281159, E 6.802581, S 6.839079, F 6.505433, Q0 7.063655, QG 6.852872; smoothed-target KL S 3.186145 and F 2.866958 — reproducing the recorded and review-derived values. E's true-label fit CE equals its empirical-target CE as the objective identity requires; heads are compared only under the same teacher.

Pair-association permutation (per `(document, PAD-status)` Fisher–Yates over occurrence indices, recipient target fixed, donor context substituted — the `prior-frozen-evaluate` semantics, **not** the older-prefix conditional permutation): 72 strata, 17,342 eligible, 17,248 changed contexts; penalties E +4.2775, Q0 +3.7635, QG +3.7451, all intervals well above zero. Frozen-bias/context-disabled logits are invariant under it for every candidate.

Generation (64 greedy tokens, unchanged decoding, six retained prompts): parent and E period-1 pair cycles on all six; Q0 period-1 and the most degenerate; **QG period-3 on all six**, i.e. it does not enter the single-token fixed point the others fall into — still degenerate, and diversity is not language. Raw IDs/bytes saved for every prompt and model.

Cost: E 0.0862 s, Q0 0.0834 s, QG 0.1038 s per 64 tokens; all three artifacts 454,788 bytes (identical format and parameter count). QG's slightly higher latency is consistent with fewer zero weights. Calibration excluded from inference (0.30 s search + 0.84 s Gram). Peak RSS ≈1.3 GB. Physical energy UNAVAILABLE.

## Validation

`cargo fmt --check`; offline check and release builds of the changed binaries; 7 new `head_projection` fixtures (Gram vs direct repeated-occurrence sum; coordinate `dJ` vs direct objective change; zero/collinear features; deterministic two-sweep order and non-increasing fit vs Q0; ternary tie semantics; invalid-shift exclusion; the screen-helper fixture), plus the existing 5 `prefix_artifact`, 3 `realtext_support`, 12 `prefix_state` and named `prior_learning` tests; `python3 scripts/check_claim_wording.py`. Three complete runs were needed because runs 1 and 2 were found incomplete against the prompt's own requirements before any result was reported (run 1 used full-fit conditional counts for the smoothed teacher; run 2 omitted F from the matrix); both are preserved and charged once.

## One next architectural recommendation

Return to a **query-conditioned geometric state read/update** on the strengthened local baseline, with a separable-prefix residual and a matched local-only control at the same dose and panel. The falsifiable hypothesis: conditioning the read on both the local query and the stored state changes the pairwise-logit effect in a way `Z(local,q) = Z_parent(local) + r(q)` cannot, and it should beat that residual and the local-only control. That fit is **not** launched here.

References #973; #820/#963/#964 remain open. Ledger/JSON now **170,238,565 / 170,900,000 ms** after the revision recorded before the final run.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

