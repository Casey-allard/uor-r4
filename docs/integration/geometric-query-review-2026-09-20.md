# Principal review after PR #1308 — return to a matched geometric query read

Date: 2026-09-20. Reviewed merge `d92bd072b53891e83bc5ccf09c07a7ba3e5eba2b` and head `71af291e279cb0e9c68eb231a115c6430163e0bf`, both tree `d4172d34a2d4650eb81c2cf6ffb0b26fbf183655`. This review reconciles the DeepSeek handoff, current source, retained results, canonical roadmap and primary research. No Rust build, training or model forward ran in this review; saved-data arithmetic is not a new experiment.

**Decision:** freeze the corrected empirical head E as the numerical incumbent. Stop this round of head calibration and implement one controlled **query-conditioned read of the learned geometric prefix state**, using the existing ordered right-product update. Compare it with an equally parameterized separable history/query read and a local-only read. The [complete DeepSeek prompt](deepseek-geometric-query-step-2026-09-20.md) specifies the experiment, its small constructive/learned checks, gradients, artifacts, controls and resource envelope. A selective write/reset mechanism is not part of this change.

## What #1308 establishes

All four new roots verify their complete sets, sizes and BLAKE3 hashes: the probe has two members and each complete run has 25. Four roots total 37,278,797 bytes. All three complete runs contain identical E/S/Q0/QG artifacts; they repaired reporting rather than constructing different candidates. The corrected E/S files differ from the originals at exactly bytes 36–67, replacing the placeholder tokenizer digest; every other byte is unchanged.

On the retained 36-document /288-window /17,342-target panel, independently reaggregated saved vectors give:

| Predictor | Dev micro bits/target | Gain over E, nominal 95% paired interval |
| --- | ---: | --- |
| Corrected empirical incumbent E | 7.170815718 | — |
| Q0, ordinary ternary projection | 7.416178358 | −0.245362640 [−0.266027590, −0.225330297] |
| QG, executed activation-aware search | 7.190872265 | −0.020056547 [−0.039537544, −0.000182978] |
| Retained floating head F, offline only | 6.874144256 | +0.296671462 (point) |

QG improves over Q0 by **0.225306093 bits**, interval **[0.200620619, 0.250299443]**. Both executed conversions fail the predeclared practical screen against E. The paired bootstrap is the existing 2,000 document draws, seed `0x12345678`, ratio of resampled loss sums/counts. The panel exactly matches #1306; these are nominal repeated open-development comparisons on correlated repository documents, not fresh final qualification.

The entire local-pair permutation reconstructs from the saved seed and occurrences: 72 document/PAD strata, 17,342 eligible observations and 17,248 changed contexts. All three penalty intervals reproduce. QG's 4,096 saved row objectives sum to 624,837,379.4630171; seed counts are 2,098 nearest and 1,998 empirical. QG enters period-3 pair cycles on all six prompts; E, parent and Q0 enter period-1 cycles. Changed degeneration is not useful language.

The common-objective matrix now uses the actual 4,096-window conditional counts, shared full-fit marginal and S/F λ=(0.8,0.7), explicitly distinguishing the prescribed reference λ=(0.7,0.5). It reproduces prior known objective values and the derived float KL 2.8669577896. Those fit/tune values remain scalar execution receipts; underlying fit/tune vectors and Gram were not retained for independent reaggregation. Development means, intervals, donor map and generation have stronger saved-observation support.

## Corrections that remain

**The executed QG search omits a prescribed seed.** `learner/head_projection.rs:192` skips the nearest-code seed when `s == s0`. Keeping untouched Q0 as a candidate does not replace two coordinate sweeps from that seed. The measured result remains valid for the reduced search actually executed. The complete originally specified search is **NOT_RUN**; its failure must not be claimed. Correct this small loop and cover it with a nontrivial correlated-feature fixture before future reuse, preserving old artifact/result bytes. Completing that calibration search is not a prerequisite for this independent geometric hypothesis, and no further projection campaign is selected here.

**Gram singularity was not measured.** `bin/head-projection.rs:1322` writes `singular: true` literally. The algorithm permits singular G and does not invert it; that is not a rank measurement. Use `singularity: NOT_MEASURED` plus `singular_inputs_supported: true` in future reports. Do not use the field to revive the feature-capacity diagnosis.

**Method attribution belongs to the complete executed recipe.** QG differs from Q0 in activation weighting, scale search, coordinate optimization and use of empirical-head seeds. The result supports that recipe, not a separately isolated contribution from each ingredient. Its approximately 12.7-fold smaller fit squared-score error than E does not establish a 12.7-fold improvement in predictive fidelity. Softmax is invariant to a common logit shift, whereas raw squared-score error penalizes it; errors affecting probable competing tokens also matter more to CE than errors on negligible tokens. F was fitted against a different objective from E. These explain why the surrogate can disagree with CE without identifying which effect dominates this run.

**Reload scope is narrower than the prior prompt requested.** Full-panel CE used in-memory heads; generation and latency used reloaded Q0/QG. The source does not explicitly check every numerical field and every panel logit across export/reload. The next experiment must include that parity check and report final metrics from reloaded artifacts. This is an unmeasured verification boundary, not evidence that the saved CE is wrong.

**Cost claims retain their scope.** The reported E/Q0/QG timings are 64-token generation from one prompt, eight repeats, approximately 0.0862/0.0834/0.1038 seconds. Fewer zeros are a possible explanation for QG's timing, not an isolated causal measurement. Whole-run RSS around 1.3 GB is not standalone serving RSS. Energy remains UNAVAILABLE. Both superseded runs are sealed, despite the ledger's “unsealed-to-superseded” wording.

The 4,200,000 ms charge combines measured intervals and estimated allocations. The preceding-to-current merge span is 3,705 seconds, and the approximate listed rows sum to 4,094.1 seconds. Preserve the recorded balance, while labeling the full measured-wall basis and pre-use extension timeline unverified. No refund or inferred extra debit follows. Repeating complete evaluation for a missing scalar or teacher-definition correction is avoidable: future runners must retain final artifacts/checkpoints independently and support derived evaluation without repeating fitting or unaffected analysis.

## The next mechanism and its mathematical control

Let A(x) be a learned eight-way write-action map and B(x) a separate learned eight-way query map, both selecting from the bound exact 2I palette. Let e be identity, q the ordered product of older-token A actions, b=B(current), and q_tail=A(previous)A(current). The older prefix excludes the two local tokens and contains at most 62 tokens inside H=64. Each arm uses the same ternary reader R of shape120×16, output W of shape4096×16, both action maps, parent E, optimizer and data dose.

The three residuals, in integer-score units, are:

| Arm | Residual added to E |
| --- | --- |
| Q: query-conditioned older-state read | `128 * W(R[q*b] + R[e])` |
| S: separable older-state and query read | `128 * W(R[q] + R[b])` |
| L: local-only matched read | `128 * W(R[q_tail*b] + R[e])` |

Here W denotes the existing ternary add/subtract contraction and 128 is a left shift, not a serving multiplier. Every arm makes two reader-row accesses, has the same learned parameter count and residual bound `16*2*128=4096`. The state-fold work differs for older versus local context and must be reported. Set W=0 initially so all arms reproduce E. Use independent A/B roles; tying them would confound querying with another write. At q=e or b=e, Q and S agree. With all B codes initially identity and frozen during warm-up, Q/S R/W updates must agree exactly.

At fixed parameter values the difference is:

`r_Q − r_S = 128 * W(R[q*b] − R[q] − R[b] + R[e])`.

This supplies a genuine interaction between the older state and current query. The separable arm already has an active query channel and identical storage; it is not disadvantaged by unused dummy parameters. L can absorb local-query and identity-row benefits without older content. A win must therefore beat E, S and L and respond to an older-prefix intervention. Once fitted separately, the arms have different learned parameters: the algebra describes the architectural distinction, not a numerical decomposition of their trained losses.

Direct product is selected for this first test. Conjugation `b^-1*q*b` would additionally preserve conjugacy classes and leave central states unchanged for every query. Those constraints may have useful future roles, but adding them now would narrow the hypothesis unnecessarily. For fixed b, right multiplication is a permutation of 120 addresses; it adds no information to the q register. Different (q,b) pairs can still collide. The proposed mechanism tests how that information is used, not an increase in history capacity.

## Reduced-form check before real text

Choose the existing noncommuting palette generators a,b; e,a,b,ab are distinct. On eight reader coordinates set R[e]=R[ab]=+1 and R[a]=R[b]=−1; other coordinates/rows are zero. Set class-1 W coefficients on those eight coordinates to +1, others zero. A V=4 fixed parent with class-1 minus class-0 bias −1024, F=10 and distractor biases−7 produces query-arm score differences:

`D(e,e)=+1024, D(e,b)=−1024, D(a,e)=−1024, D(a,b)=+1024`.

All four checkerboard predictions have strict margins. A separable binary log-odds function satisfies `D00+D11=D01+D10`, making these strict signs impossible. This is a constructive representability argument, not a learned-language claim. The prompt requires an actual hard artifact fixture and a separate tiny learned-query test with changed-source/query and read-disabled controls. It scores only the intended target position to avoid the contradictory incidental-target mistake found in earlier work.

For real text, a nonzero mixed difference or changing action IDs is not enough: random tables can show both. The fixed older-prefix donor map must preserve exact local pair and older-prefix length, the local-only arm must be exactly invariant, and Q must gain over its fitted controls. Report the small eligible-intervention subset explicitly; do not relax the strata to obtain a stronger-looking result.

## Research synthesis and larger direction

- [PaTH, February 2026 revision](https://arxiv.org/abs/2505.16381) motivates combining ordered content-dependent transport with querying. It is a floating attention architecture, not a compliant implementation or accuracy guarantee for this model.
- [PD-SSM, December 2025 revision](https://arxiv.org/abs/2509.22284) provides a relevant hard-forward/softmax-Jacobian-backward precedent. Its column-one-hot transitions include non-bijective cases; a permutation-only group register does not inherit its arbitrary-FSA guarantee. Its complex diagonal and hybrid transformer are not adopted.
- [(How) Do Language Models Track State?](https://proceedings.mlr.press/v267/li25r.html) shows the value of distinguishing learned state-tracking mechanisms from easier heuristics. Here matched local controls and interventions serve that purpose; the paper's transformer results do not qualify UOR.
- [Learning State-Tracking from Code, June 2026 revision](https://arxiv.org/abs/2602.14814) connects next-token code traces with partially observable state. It strengthens the roadmap need for state/read/write semantics before broad coding claims; this repository-prose pilot does not itself qualify code execution.
- [Why Do Accumulated Transformations Extrapolate?, June 2026](https://arxiv.org/abs/2606.24975) studies assumptions such as mixing and query/key alignment. Those assumptions are not established for correlated learned token actions in one finite register. Exact or norm-preserving transport alone is not semantic memory.

The project has not yet integrated its retained exact-memory/shared-operator experiments, TinyStories prose path and current prior/group-state research into one useful language model. The corrective roadmap is: retain a trustworthy local emission baseline; test a useful query-conditioned geometric read; then investigate selective writes/reset and exact occurrence/version access, preserving distinctions under changed sources and queries; integrate shared composition and learn useful conversation plus executed Rust behavior on one artifact; measure complete consumer-machine cost. Prime/zeta/R4/S3/H4/icosian identities remain primary design material, with implemented roles distinguished from untested additions. This experiment uses exact signed H4/2I transport and learned addressed reads; it does not newly use zeta phases, prime semantic metrics or paired-H4 capacity.

There is no count-matching, perfect quantization or broad proof prerequisite before that progression. Conversely, a finite routing success cannot bypass useful behavior, uncertainty/conflict handling, exact memory or full-path energy acceptance. Do not label the next read experiment a learned selective update, a transformer replacement or a frontier result.

## Resource and delivery boundary

Live owner JSON is **170238565 /170900000 ms**, leaving **661435 ms (11.02 minutes)**. This review changes no balance. The next prompt proposes **7200000 ms** complete local work and a standing-authorized **+7200000 ms** limit extension to **178100000 ms**, to refresh and record before use. Preserve cumulative charges, all artifacts and the 128MiB margin. No paid compute or deletion. #973/#820/#963/#964 remain open; current documents and live issue bodies must point to the new read experiment while retaining these exact negatives.
