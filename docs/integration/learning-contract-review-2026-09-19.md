# Review after PR #1296: repair the learning contract before diagnosing quantization

September 19, 2026 local date. Source audited: `f4fc1c6089f4b047ec8feb92ba184c2f81bb077d`. This is a source/mathematical/literature review; no Rust build or model was executed. The [next executable prompt](deepseek-learning-contract-step-2026-09-19.md) carries the implementation. The [previous review](prior-learning-review-2026-09-19.md) remains historical context.

## Decision and verified state

Keep the current low-bit prior architecture. First repair its forward/gradient contract, optimizer behavior and synthetic supervision; then run the corrected small learning gate. A floating comparator is conditional on failure of that corrected experiment. Continue into the already selected bounded real-text curve if its prerequisites pass and measured resources permit. Do not build another competing prior module or replace the mechanism on the strength of the old 0.375 score.

PR [#1296](https://github.com/UOR-Foundation/uor-r4/pull/1296) is merged. Reviewed head `e5a29cf22506cb51bd7f2c44fe2a0045b82000ba` and merge `f4fc1c60` have identical tree `0010fd23b6813ffdc88f0dedeb666831d446444e`. It adds `prior_learning.rs`, module registration and a receipt. Real-text fitting and the floating comparator were not run. No new persisted trained artifact/checkpoint or manifest was delivered. The previous joint artifact remains 614,542 bytes, SHA256 `fb780ff6ff19eec58f861135004250ae8874107099e2a071f7bfd1b4318b5a39`, independently reverified at the owner's retained path.

Useful additions include an uncapped log-sum-exp loss routine, an n−1 target iterator, frozen low-bit bias, checkpoint serialization machinery and reported tests. **The claim that Stage 1 is complete or training/serving parity is established is incorrect.** Preserve the [raw receipt](../evidence/native_geometric_prior_recovery_2026-09-19.txt) with this correction.

## Three decisive defects

### 1. Contextual logits have different units in training and export

Let R be the integer contextual readout, b the bias code, s the bias shift and F the common logit exponent. The serving evaluation computes

`z_served = (R + b * 2^s) * 2^-F`.

At `prior_learning.rs:570-576`, the duplicated floating training forward instead computes

`z_train = R + b * 2^(s-F)`.

At F=10 the contextual term is 1,024 times the served value. Backpropagation at line 593 nevertheless multiplies the softmax error by `2^-F`, so it is not even the declared surrogate of that training forward. Zero output codes hide the defect at initialization. With flat bias the argmax may still agree despite radically different probabilities; an accuracy-only test will not expose it.

Use one actual integer forward/trace for both training and serving. Check nonzero contextual scores, nonflat bias, nonunit shifts, nonzero F and active normalization, then compare probability logits and loss. Independently verify the stated effective-weight STE derivatives with quantizer/shift choices frozen. Nonzero gradients are insufficient.

### 2. The small fixture trains incompatible laws

The test at lines 1067–1127 trains every next-token target in `[x,y,s,x,y,s]`, where `s=(x+y) mod 4`, over all 16 pairs. It evaluates only the second prediction. For each non-prefix context (a,b), training labels are:

`a+b` twice, `b-a` once, `a-b` once, all modulo four.

Thus rotating the triple introduces subtraction laws that conflict with the intended addition mapping. The absent-prefix context also has four equally likely targets.

| Context class | Number of contexts | Targets per context | Maximum empirical conditional accuracy | Conditional entropy |
| --- | ---: | ---: | ---: | ---: |
| PAD,current | 4 | 4 | 1/4 | 2 bits |
| Both tokens even | 4 | 4 | 1 | 0 |
| Mixed parity | 8 | 4 | 3/4 | H₂(1/4) ≈ 0.811278 bits |
| Both tokens odd | 4 | 4 | 1/2 | 1 bit |

The unrestricted optimum on these **80 authored training targets** has accuracy 52/80 and CE `0.4 + 0.4*H₂(1/4) + 0.2 = 0.9245112498` bits. At the exact empirical conditional distribution, the current lowest-ID tie rule yields 14/16 = 0.875 on the selected addition targets, below the 0.9 gate. This is not a hard ceiling on that selected-position accuracy: a near-optimal predictor can break ties differently. It shows that the training objective does not uniquely reward the gate's answers.

The measured 0.375 remains a six-of-sixteen score on an inconsistent authored fitting task. It does not isolate quantization, ReLU, optimizer convergence or capacity. Use all 16 triples `[x,y,(x+y)%4]` with **only target index 2 scored**, through an explicit fixture mask shared by training/evaluation. Do not change real-text scoring, which remains all n−1 predictions. Test conditional purity and uniform target marginals before fitting. This addition task requires both inputs but is commutative, so it is not an order-reversal qualification.

### 3. Adam silently drops small legitimate gradients

`lowbit_core.rs::adam_update` updates a parameter only when `v_hat > 1e-12`. This is an extra threshold, separate from the epsilon in its denominator. On a first step with g=1e−7, default betas, lr=0.05 and zero weight decay, the current update is zero; ordinary epsilon-regularized Adam gives approximately −0.04545. Fixed-point and per-target scaling can push valid gradients into this range. The fraction of updates lost in the reported fit is unmeasured.

Repair the threshold behavior with an independently computed scalar reference, including zero gradients, small gradients and nonzero carried moments. Account for the shared helper's affected callers with focused checks. This is not permission to silently reinterpret old results obtained under the earlier optimizer. [Adam's original algorithm](https://arxiv.org/abs/1412.6980) supplies the reference, not a convergence guarantee for this model.

## Capacity can be checked constructively

The existing ternary architecture can express the intended 16-pair task. Index 16 hidden coordinates by (a,b), with embedding shifts zero:

- `E_old(x)[a,b] = +1` if x=a, otherwise −1.
- `E_new(y)[a,b] = 0` if y=b, otherwise −1.

The sum is 1 only at the matching pair, otherwise 0, −1 or −2. ReLU gives a one-hot pair feature; normalization at six bits does nothing. Set output code `W[r,a,b]=1` when `r=(a+b)%4`, otherwise zero. Output shift 12 and F=10 give a correct-class margin of four. With flat bias, `p_correct=1/(1+3e^-4)≈0.9479`, and CE is approximately 0.0772 bits. Set the PAD row to −1 and unused coordinates to zero to use dv32.

This is an exact mathematical construction for the declared operations, **not an executed fixture or a learned result** in this review. Export/reload it in the next run and verify all 16 predictions and probabilities. Keep these authored weights entirely separate from the random-initialized learning arm. They prove representability, not generalization or geometric advantage.

## Other completion claims that need correction

| Source / claim | Actual status and necessary correction |
| --- | --- |
| `next_batch` / `perm_index`, lines 690–717 | Independent hash-modulo indices are not Fisher–Yates and do not guarantee no replacement. The final batch wraps into the next pass. Use an actual seeded permutation and a partial final batch; assert exact once-per-pass coverage. |
| Checkpoint lines 720–831 | Saves masters/moments, but omits beta1/beta2/weight_decay and discards data identity at load; source/tokenizer identity is not verified. Test equality of complete training state and subsequent batch/updates, not only quantized output codes. Reject mismatch without partially changing the live trainer. |
| Artifact lines 295–379 | Discards seed/tokenizer digest, allocates from unchecked large counts, leaves element/packed-code validation incomplete and does not prove arithmetic overflow bounds. A shift ≤30 alone does not make sums safe: even `7 << 30` cannot represent the intended positive bias in i32. Bound configurations or widen checked accumulators, then reject unsupported envelopes before execution. |
| Malformed-shift test line 1017 | `is_err() || true` always succeeds, and byte 60 is inside the tokenizer digest, not a table shift. Replace with an actual field-offset mutation and a real rejection assertion. Other rejection cases do not establish this one. |
| Learning test lines 1097–1125 | Does not assert the stated learned > constant or ≥0.9 condition. Separate a persisted research gate result from correctness tests; passing unit tests are not a learning pass. |
| “Fit-unigram gap 0.10 bits” | Measured on a short authored evaluation sequence under a highly skewed count distribution, not a general bound or a real-corpus calibration result. Source comments still describe an obsolete factor-of-two bias convention. |
| New iterator shared by all instruments | Used within the new module; old real-text runner/references are untouched. No corrected real-text harness is connected yet. |
| Report preservation conditional on real text | Executed small fits need unique claimed evidence too. No new framework is needed; use the existing report API for the actual experiment. |
| CPR1 replay unavailable because training bias differed | Training mismatch prevents reconstructing the missing training state, not evaluating the retained served artifact. Exact old-population replay depends on recovering data; a labeled new-population diagnostic remains possible. |

These repairs are bounded to the active learner and its exercised interfaces. They do not require a blanket security campaign, old model replay or architectural rewrite. The floating comparator was already authorized conditionally in the previous prompt; its omission is recorded, not a new approval requirement. Remaining budget is not evidence that execution was forced to stop.

## Roadmap adjustment and mathematical rationale

The [canonical plan](project-track.md) now makes the dependency order explicit:

1. **Numerical and experimental correctness:** one served forward, valid supervised targets, verified surrogate/optimizer and reproducible evidence. Current blocker.
2. **Local-context learning baseline:** a corrected exported prior learns beyond marginal frequency on representative text with context interventions. NOT_RUN. This is a component/control, not the final architecture.
3. **Predictive state and geometric context:** learn an older-prefix-dependent state/read contribution above that baseline, including paired histories with the same local tail but different relevant earlier evidence. Compare directed geometric transport/learned packaging against capacity/cost-matched alternatives; keep exact occurrence/version memory distinct from modulo value buckets.
4. **Integrated capability and complete cost:** conversation/memory and executed coding/reasoning on one artifact, causal retention controls, bounded scoring/parameter traffic, full-path M1 latency/RAM/storage and physically measured energy; then API/WASM/Studio integration and alpha qualification.

For a predictor restricted to the last two tokens, identical local tails force identical distributions. On a fixed population its optimal loss is H(Y|tail); the optimal gap to conditioning also on older history is I(Y;older history|tail). This identity states the architectural limitation; it does **not** turn finite fitted count scores into Bayes ceilings. The local prior must therefore lead into state/memory, not become an endless trigram-scaling programme. In the new module, the modulo-120 element table is stored but does not participate in prediction; its presence is not an implemented geometric contribution. The two-table/ReLU/ternary readout is presently a local learning baseline. A single 120-state geometric root is also finite and cannot losslessly retain arbitrary history. Fixed invertible re-labeling does not add information. Geometry must contribute a tested transition, useful sharing or cheaper selected access; exact identity and orientation remain preserved.

## Targeted external and internal research

The existing [mathematics](architecture-2026-09/mathematics.md) and [import audits](architecture-2026-09/imports.md) supply exact H4/2I composition, signed actions, typed icosian and UOR identity roles. Their old blanket serving exclusions are superseded by D0-b. No new dependency is adopted.

- [Yin et al., STE analysis](https://arxiv.org/abs/1903.05662): coarse-gradient behavior depends on the surrogate and assumptions. Its binarized-activation/Gaussian analysis does not certify this dynamic ternary-weight learner; it supports checking the actual backward rule.
- [Learned Step Size Quantization](https://arxiv.org/abs/1902.08153): quantizer scale and gradient scaling are optimization variables, not harmless notation. It motivates a scale diagnostic if a corrected matched test still exposes a gap; no unrestricted floating serving scale is adopted.
- [He et al., February 2026 modular-addition dynamics](https://arxiv.org/abs/2602.16849): Fourier features, phase alignment and diversity explain specified two-layer experiments. Its analysis uses particular activations/initialization and does not predict convergence of this 4-token ternary test. Fitting all 16 pairs is not grokking.
- [Moisescu-Pareja et al., December 2025 representation geometry](https://arxiv.org/abs/2512.25060): studies collective geometry of modular-addition circuits. A geometric-looking representation alone does not establish a new algorithm, language advantage or efficient serving.
- The previous [low-bit research refresh](prior-learning-review-2026-09-19.md#research-refresh) retains BitNet, continual QAT, Sparse-BitNet and CAT-Q. A bounded precision curriculum is conditional on a valid measured learning gap; pretrained-model quantization does not remove pretraining cost.

This is a decision-focused refresh, not a claim to have exhaustively reviewed every available paper or repository. No new arithmetic activation, answer-coded group law, MoE or transformer is justified by the flawed gate.

## Resources

Live recorded JSON: `149838565 / 154400000 ms`, leaving `4561435 ms` (76.02 minutes). This review changed no model charge/limit and ran no Rust build/model. The new prompt proposes a complete 3,600,000 ms tranche, including repairs, tiny experiments, conditional real-text fitting and reserve. Refresh and record before execution; necessary local extensions remain owner-authorized when documented before use. Original checkout, negative artifacts and sealed material are preserved; no paid compute or deletion. #973 and #820 remain open.
