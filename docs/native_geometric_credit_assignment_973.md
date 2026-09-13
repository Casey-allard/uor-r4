# Shared geometric state/read credit assignment

September 12, 2026. References #973 and #964. This follows the [block-calibration result](native_geometric_block_learning_973.md). The retained model remains `15baec48`; experimental parent `ade9a1cf` is preserved and unpromoted. Final serving still uses discrete geometric operations without a transformer or matrix products.

## Question and method

The preceding joint fit accepted no transition changes and had a weak prior-state intervention effect. This suggests testing training feedback across earlier choices; it does not establish that credit assignment alone explains the language failure.

The Rust development prototype enumerates one-use interventions in the actual hard runtime. For each transition/read lane it selects the most frequently accessed parameter on the unchanged parent; ties use the lowest index. It uses the first 32 bytes of existing construction documents 0, 3, 7 and 9: four already-open sequences and 132 byte/EOS targets. No target loss participates in parameter selection.

For each selected parameter, each of its baseline occurrences and each of 120 canonical H4 replacements, it changes the parameter for exactly one `observe`, restores the original parameters immediately afterward, and measures all subsequent target losses through EOS. Changed state, stored keys and subsequent routing remain active. Earlier losses cannot change.

Let `D[r]` be the sum of those individual occurrence loss changes. The categorical surrogate is `S(z) = sum_r softmax(z)[r] D[r]`. At uniform logits its derivative is `(D[r] - mean(D))/120`. The proposed complete replacement minimizes this surrogate, keeping the current root on tolerance ties. Independently, all 120 tied replacements are evaluated across the complete sequences using `SharedCore::evaluate`.

This is an exact derivative of the stated finite surrogate. It is **not an unbiased gradient of the tied hard model**: changing a shared parameter can change its subsequent access pattern and interact with its other uses. Enumeration is an expensive reference method, not a scalable trainer or an implemented straight-through geometric derivative. Its value is to test the aggregation assumption before choosing a faster estimator.

The frozen method gate requires all eight rows, six improving proposals, at least two improving proposals in each family and at least half the sum of available per-row oracle gains. Worsening proposals contribute negatively. No available oracle gain fails instead of passing vacuously. These are development method criteria, not language acceptance.

## What ran

**FAIL_OCCURRENCE_CREDIT_DIRECTION_PROBE.** The complete probe used 6,171 suffix replays and 960 tied-parameter replays. None of the eight proposed replacements improved actual loss. Seven selected parameters had no improving tied replacement and retained their current value. This is limited to the selected hot parameters and the short development panel; it is not a statement about the whole parameter space.

The remaining row, read parameter 1930 (lane 1, nine baseline occurrences), exposes the aggregation failure:

| Choice | Actual full-sequence NLL-sum change |
| --- | ---: |
| Current root 119 | 0 |
| Surrogate-preferred root 26 | +0.959586 (worse) |
| Actual best root 76 | -3.075335 (better) |

The signed oracle-gain fraction is -0.312027, below the required 0.5. No parameter update, model export, joint fit, new holdout or new free generation was performed. The previous actual generations remain unusable; this investigation makes no newer language-quality claim.

## Hamming difference measurements

Hamming distance is used over **aligned categorical or Boolean decisions**, not the binary encodings of arbitrary geometric root IDs or IP labels. For each root alternative, compare whether the surrogate and actual loss classify it as improving using tolerance `1e-9`.

The problematic read row has 69 disagreements out of 120 alternatives. The surrogate predicts 75 improvements, actual tied replay finds 22, and only 14 belong to both sets: 61 false positives and 8 false negatives. The other seven rows have no improving alternatives under either calculation, so their disagreement is zero. Reporting only 69/960 would hide the concentrated failure; Hamming disagreement also discards loss magnitude and must accompany the actual loss deltas.

## Research contribution

Project knowledge search and the code graph were used for discovery, followed by current-source verification. The graph snapshot predates this shared core. Its projection-based trainers and direct geometric-attention gradients differentiate different models and were not imported.

[Categorical reparameterization](https://arxiv.org/html/1611.01144v5#S2.SS2) distinguishes hard forward values from biased backward estimates. [Straight-through estimator analysis](https://arxiv.org/abs/1903.05662) motivates checking an estimator's direction under its actual model assumptions. [Stochastic computation graphs](https://arxiv.org/html/1506.05254v3) provides the expected-cost/score-function viewpoint used for the finite categorical reference. None supplies a UOR capability result.

The owner supplied newer [SpiralCore v68 and FBS revision 3.1 sources](../research/spiralcore-v68/README.md) during this step. Both originals were preserved byte-for-byte with provenance; v63 and its Rust adapter are unchanged. The inspected v68 source implements typed symbolic addresses, Bell frequency encodings, deterministic prompt projection/BFS, operator and orbit traces, and bounded Kuramoto simulation. The inspected paths do not implement learned attention or this missing training rule.

Their useful contribution here is explicit separation of identity, full internal route history, chosen action, context and displayed observable. The FBS reference distinguishes lawful route sets from a selected BFS representative and describes distinct intrinsic histories with colliding displays. It also explicitly leaves its canonical action selector unresolved. Those source claims remain external, not newly reproduced UOR measurements. They motivate the aligned intrinsic-state/source/output trace below rather than adopting an extra serving mechanism.

## Actual intrinsic-state and route trace

The bounded follow-up replays the two replacements above, both as full tied changes and as each of the nine individual interventions. Every resulting total loss reproduces the sealed probe within `1e-9`. All nine baseline accesses belong to development document 0; the other documents are unchanged by these replacements. This is a particularly narrow repeated-use case.

| Replacement | Sum of isolated occurrence NLL changes | Actual tied NLL change | Root-slot Hamming / 528 | Source Hamming / 132 | Predicted-byte Hamming / 132 |
| --- | ---: | ---: | ---: | ---: | ---: |
| 26, surrogate preference | -12.590498 | +0.959586 | 40 | 28 | 16 |
| 76, actual best | +5.596980 | -3.075335 | 45 | 24 | 9 |

Both first change state at document 0, target position 3, after observation position 2. Each changes roots at 30 target positions. The original parameter accesses are observation positions `[2,6,10,16,17,18,20,26,31]`. Replacement 26 changes them to `[2,6,7,18,30]`; replacement 76 changes them to `[2,3,6,9,20,26]`. Thus the shared update changes not just values but the future schedule of its own use. Summing interventions on the original schedule predicts the wrong sign for both choices.

The measured interaction residuals (actual tied change minus summed isolated changes) are +13.550084 and -8.672315. Hamming measures discrete changes, not their desirability: the better replacement changes more root slots. Source identity includes null versus a selected occurrence; EOS is scored and included in the prediction trace. These are teacher-forced traces, not new free generations.

## Validation, preservation and next action

Four focused tests pass: categorical-gradient finite differences and baseline-shift invariance; causal access/suffix parity and nonvacuous single-use comparisons; bounded-input rejection; unchanged trace and public evaluator likelihood parity. The actual saved-artifact probe and follow-up trace execute separately and seal their reports. An initial compilation error from private host-loss visibility was corrected with a local reconstruction that calls the actual branch scorer and is checked against public evaluation; that failed build is charged. Final review moved preservation checks before writing a complete trace and binds the trace's source hash. All production shared-core runtime, training and artifact sources remain byte-identical to the preceding commit; no artifact migration, allocation campaign or repeated language fit was necessary.

**Next:** implement a bounded categorical learner whose shared parameter is drawn once per model/trajectory and used consistently at every subsequent access. Credit must use complete hard-trajectory cost, including newly created or removed accesses. Check a small joint parameter block against an exhaustive tied-cost reference and a matched model-call baseline before a larger learning campaign. This addresses the observed occurrence-aggregation failure; it is not evidence that score-function training will scale. Do not install the descriptive root-76 oracle witness, repeat the failed surrogate, infer that more or less Hamming drift implies quality, or promote a candidate. Representation, curriculum and language transfer remain unresolved.

Exact projection, cumulative charges, source/binary bindings and report hashes are in [the evidence record](evidence/native_geometric_credit_assignment_973.json). Research ingestion preserves source claims at their reviewed scope; neither Bell frequencies nor IP bit distances are introduced as learned relevance or geometry.
