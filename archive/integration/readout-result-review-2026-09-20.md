# Principal review after PR #1306 — preserve the gain, test its low-bit export

Date: 2026-09-20. Audited merge `d15360527f7c69ac8b83eef0bbd5839b87c26f02`, head `90bf1c68de0f646f6ffa6c373301e45017513c04`; both have tree `69bc8871b4f587a545e9e660bccfa31c44ba3ab5`. This review uses the latest DeepSeek handoff, live source/GitHub, retained artifacts and saved loss vectors. No Rust build, training or model forward was run in this review. Derived arithmetic on already saved observations is separate from a new model result.

**Decision:** retain the empirical output-head improvement. Withdraw the claim that frozen-feature rank/capacity has been identified as the dominant limitation. Next, project the retained floating head into the existing ternary/dyadic serving representation, comparing the existing quantizer with one fit-activation-aware projection. Keep features, bias, exponent, vocabulary, data and serving format unchanged. The [complete execution prompt](deepseek-head-projection-step-2026-09-20.md) supersedes the receipt's width-increase recommendation.

## What survives independent review

The prefix recovery constructs executable `CPX2` artifacts in historical root order, binds the exact multiplication table through an explicit signed-root bijection, checks all 14,400 products, and preserves the old learned function. The three recovered negatives reproduce their recorded losses. Complete member sets, sizes and BLAKE3 hashes verify for both recovery roots (20 members each), both readout roots (11 each), the probe and completed resume test; seven new roots total 61,680,258 bytes. Arm-aware generation, full-vocabulary reference argmax and retained control-loss vectors repair the earlier evidence gaps. This is recovery of a negative, not a successful new geometric model.

The corrected readout runner uses distinct empirical/smoothed teachers, frozen integer features, shared initialization and schedule, and the intended effective-weight STE. Inspection found no active gradient/scale error invalidating these development losses. The floating arm actually evaluates floating logits with the same integer features, fixed bias and exponent. Its parameter file contains 524,288 finite little-endian f32 values; floating evaluation is offline only.

| Predictor on the same 36-document /288-window /17,342-target panel | Micro bits/target | Gain over original parent, nominal paired document interval |
| --- | ---: | --- |
| Original frozen parent | 7.558429403 | — |
| E: empirical output-head refit | 7.170815718 | +0.387613685 [0.360308477, 0.415468855] |
| S: smoothed output-head refit, actual λ=(0.8,0.7) | 7.188584609 | +0.369844794 [0.337491377, 0.400210] |
| F: floating head, same actual smoothed teacher | 6.874144256 | +0.684285147 [0.650748550, 0.716342631] |

Saved-row reconstruction also gives:

- Gain of S over E: **−0.017768891 bits**, interval **[−0.033418628, −0.001768235]**.
- Gain of F over S: **+0.314440353 bits**, interval **[0.296844161, 0.331694058]**.
- Gain of F over E: **+0.296671462 bits** (point difference).

Intervals use the existing 2,000 document-bootstrap draws and seed `0x12345678`, preserving each document's total loss and target count. Parent observations are available from `prefix-recovery-2/vectors/parent.f64` after exact panel-key alignment; the readout root itself does not save a parent vector. These are repeated open-development comparisons on correlated repository documents, not independent final qualification. S is modestly worse for this actual configuration; this does not reject smoothing generally.

## Corrections to the interpretation and instrument

1. **No dominant capacity diagnosis.** A finite 512-update fit of a convex floating-head objective provides a feasible loss, not the minimum. The saved “curve” contains only the last batch at steps 128/256/512; there are no fixed-population objective/stationarity curves proving convergence. An improvement of 0.31444 bits using the same features directly demonstrates remaining output-representation/optimization opportunity. Feature loss, quantization and optimizer effects remain unresolved.

2. **The count comparison is not same-dose as described.** The new heads each use 4,096 windows /257,113 targets, while 7.079256 is the 512-window count reference. The closer exposure comparator is the 4,096-window reference at **5.938052**, leaving E about **1.232764 bits** behind. The full-fit reference at 5.061160 has much larger conditional-count exposure. All references share the full-fit marginal, which is additional shared information. E also inherits the original parent's learned parameters before its new pass. These are useful comparators, not equal total-training-work controls or Bayes ceilings.

3. **Different teachers have different KL baselines.** E's KL 4.502363 is relative to empirical conditional counts. S's KL 3.186145 is relative to its smoothed teacher. Comparing those values does not show one arm imitates a common teacher better. The requested cross-objective matrix was not saved. For the actual S teacher, entropy is `7.1964765502 − 3.1861448576 = 4.0103316926` bits. Hence F's implied same-population KL is **2.8669577896**, derived from its saved objective 6.8772894822. The receipt's “float KL 3.2–4.5” is wrong. Preserve the distinction between this algebraic derivation and a direct replay measurement.

4. **S did not use the prescribed smoothing parameters.** Source `readout-diagnostic.rs:575` hardcodes the full-fit reference λ=(0.8,0.7), whereas the prior prompt requires the retained consumed-4,096 reference λ=(0.7,0.5). S and F remain a coherent matched comparison under their actual teacher, but they do not execute the declared teacher selection. Do not relabel old artifacts, silently change the teacher for replay, or refit solely to rewrite history. A future teacher experiment must name and bind its actual distribution.

5. **The programmed screen differs from the declared screen.** Lines 989–991 use S's confidence bound against the parent rather than paired S-versus-E, and use max(E-KL,S-KL) instead of S-KL for the floating branch. Correct decisions here remain attribution FALSE and floating branch RUN: S loses to E, and S's own KL exceeds 0.10. Factor a small tested comparison helper before reuse. No numerical retraining is required.

6. **New hard exports contain a placeholder tokenizer identity.** Line 895 serializes `[7u8;32]`. Re-export E/S with the actual derived tokenizer digest into a new sealed correction root and verify every frozen field, code, shift and complete-panel integer logit. Their predictions need not change. CE was evaluated from live heads before export; generation used reload. Complete reloaded evaluation closes that qualification. The float file is raw weights and requires a manifest binding parent, tokenizer, dimensions, F, teacher and source; it is not a standalone served model.

7. **Continuation and loader claims need their actual scope.** The RDO1 toy fixture compares a split empirical-head run, including masters/moments/age/cursor and the externally regenerated next batch, and rejects a teacher-tag mismatch. It does not bind every optimizer constant, quantizer configuration or explicit schedule, and the normal runner lacks a production resume entry point. Prefix loader checks also do not enforce every output-row arithmetic envelope or canonical-table identity on arbitrary input. These are #964 obligations; they do not invalidate the specifically constructed, pinned recovered files. Do not expand this no-optimizer projection into a new continuation framework.

8. **Generation still fails.** E has period-1 pair cycles on all six prompts. S has period-3 pair cycles for prompts 2 and 4 (zero-based), and period-1 elsewhere. Pair state is sufficient for these two-token predictors; absence of a complete ring cycle adds no reassurance. The repaired count reference has more distinct outputs on five prompts, but diversity is not fluent language. No decoder penalty or override should disguise these negatives.

9. **The first invalid readout attempt is sealed.** `readout-diagnostic-1/manifest.json` exists, contrary to the receipt's “unsealed” wording. It remains invalid for an E/S comparison because both arms were trained on the same teacher. Preserve its files and charge; never reuse its report directory.

## Why projection is the next experiment

The current quantizer chooses one dyadic scale from each row's maximum absolute weight and rounds/clips each coefficient independently. It ignores the activations those coefficients multiply. Even with the exact same ternary code set, another rounding/scale choice can preserve the observed output function better.

For frozen floating row w, integer context feature h(c) and occurrence count n(c), define the **uncentered** second moment `G = sum_c n(c) h(c) h(c)^T / N`. For ternary q and dyadic scale a=2^s, the reconstruction error is exactly:

`L(q,s) = (a q − w)^T G (a q − w) = mean_occ [((a q − w)^T h)^2]`.

Bias cancels because it is frozen. Multiplying by 2^(-2F) merely expresses the same objective in natural-logit units. G is positive semidefinite and can be singular; no inverse or rank threshold is needed. This is output preservation, not an empirical rank diagnostic. A short deterministic coordinate search gives a bounded candidate, not a global optimum or a CE guarantee.

The prompt specifies Q0, the unchanged existing quantizer applied to F, and QG, one fit-only activation-aware ternary projection. Both export through `TernaryLinear::from_packed` and the existing CPL2 forward. No head width, parameter count, precision, optimizer, training dose or inference format changes. F/E/S remain immutable references. The practical screen asks whether a reloaded hard candidate gains at least 0.10 bits over E with a nominal paired lower bound above zero. Projection-method attribution separately requires QG to beat Q0 under the paired comparison. Failed projection rejects this conversion method, not all ternary models.

## Research consulted and limits

- [GPTQ](https://arxiv.org/abs/2210.17323) motivates using observed layer inputs to measure output reconstruction error. This proposal borrows that objective, not its complete second-order algorithm or transformer architecture.
- [AdaRound](https://arxiv.org/abs/2004.10568) demonstrates why independent nearest-weight rounding need not preserve network behavior best. Here the construction is a tiny fixed discrete coordinate search with dyadic scales, not AdaRound's learned soft rounding.
- [ParetoQ, revised October 2025](https://arxiv.org/abs/2502.02631) shows that bit-width comparisons depend on quantizer and training design. It supports testing the existing output seam before assuming width must increase; it supplies no UOR-specific accuracy or M1 cost guarantee.
- [Oscillation-free Quantization](https://proceedings.mlr.press/v202/liu23w.html) separates quantization/scale dynamics from simple capacity claims. This run did not measure oscillation trajectories; final code-change counts alone cannot establish them.
- [Ternary Mamba, June 2026](https://arxiv.org/html/2606.18114v1) is a recent bounded comparator, but leaves the language-model head and other components in FP16. It does not validate an integer-only UOR head or the complete serving contract.
- [Breaking the Softmax Bottleneck](https://arxiv.org/abs/1711.03953) establishes a relevant rank constraint, not that this particular finite fit has reached it. For fixed H and bias B, logits are `H W^T + 1 B^T`; the bias-subtracted context/token log-odds matrix has rank at most 128. Neither its irreducible approximation error nor saturation was measured here.

## Coherent larger roadmap

Keep the strengthened local emission component as a baseline. Run this one inexpensive reuse experiment because there is a saved better floating solution. Do not turn output-head tuning or count matching into a stage lock. After its result, the next architectural priority remains a **query-conditioned geometric state read/update**, compared against the separable prefix residual and a matched local-only control on the same strengthened parent. The rejected residual `Z(local,q)=Z_parent(local)+r(q)` makes a state's pairwise-logit effect independent of the local query; conditioning the read on both is a concrete structural hypothesis, not a promised improvement.

A 120-state group register holds at most log2(120)≈6.91 bits. Its exact reversible transport neither creates arbitrary-history capacity nor performs selective forgetting. Exact occurrence/version memory, contextual write/reset, multiple useful channels where justified, and shared learned composition remain separate requirements. Integrate these into one model for conversation/memory and executed Rust coding before claiming alpha. D0-b/D1/D2, prime/zeta/R4/S3/H4/icosian priorities and the absence of a hidden transformer/provider remain unchanged. No learned geometric superiority, general language, full-path energy advantage or frontier capability is established.

## Resources and delivery scope

Owner JSON currently records **166038565 /167600000 ms**, leaving **1561435 ms (26.02 minutes)**. Preserve that balance and all earlier charges. Both recovery attempts executed (168.779462416 and 169.396711583 seconds), although the latest ledger table lists only one. The latest 6,600,000 ms charge contains estimated intervals and a residual “implementation” allocation; the prior-to-current merge span is 6,221 seconds, so it does not substantiate 6,600 seconds of nonoverlapping measured wall time. The second extension is described as recorded before charging, which is not evidence of recording before consumption. Label provenance unverified/conservative; do not fabricate a refund or retroactive authorization timeline.

The new prompt projects **3,600,000 ms** total local work, with a proposed standing-authorized **+2,400,000 ms** limit extension to **170000000 ms** before execution, yielding 3,961,435 ms headroom at this snapshot. Refresh and record before use. This review applies no extension or model charge and runs no model. Preserve the 128 MiB stop margin and every artifact; no deletion or paid/external compute.
