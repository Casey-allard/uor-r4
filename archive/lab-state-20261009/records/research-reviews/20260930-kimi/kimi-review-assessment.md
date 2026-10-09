# Assessment of Kimi's external UOR-R4 review

Date: 2026-09-30. Source baseline: main `61bbd85a0a49e1bb72c71b4e02752d575e3bee4b`. This is a research assessment, not an adopted policy change, new experiment authorization, or model qualification. The owner requested independent expert comparison of the supplied external synthesis.

## Verdict

Agree with the separation of numerical execution, learned capability, and geometric contribution. Agree that external memory, training with the deployed discretization, and capacity/data planning deserve substantial attention. Disagree with declaring the serving problem finished, describing the active model as the old 1.68M single-head architecture, treating recipe-specific failures as impossibility results, and adopting a 1–3B/100B-token programme from these comparisons.

The review offers useful hypotheses but is not reliable enough as written to replace the current research plan. The project itself also has an evidence-navigation problem: an older numerically accepted artifact, the active learned stack, comparators, and integration candidates are easy to conflate. A current model identity must name source, bundle, configuration, evaluator, serving mode and qualification scope together.

## Review method and limits

Three independent specialist passes covered live repository source/evidence; primary literature and hardware/numerical claims; and adversarial mathematics, learning and resource implications. The principal compared their findings against current decisions, the integrated plan and the retained integer-serving record. Wolfram checked arithmetic ratios and the arbitrary-frequency relative-rotation identity. Detailed supporting reports accompany this assessment.

This review did not train a model, execute Cargo tests, repeat sealed evaluations, inspect every external kernel, or independently reproduce the literature's measurements. Existing UOR results below are source-linked retained evidence, not newly executed results. Different experts were separate review roles, not claimed provider diversity.

## Corrections that change the recommendation

| Assertion | Assessment |
|---|---|
| The full256 kernel is done, with 4,096 hashes exactly matching F32 | The 4,096 exact hashes concern preservation across integer extraction/optimization. F32-to-integer fidelity is a separate comparison. The September 25 report explicitly excludes whole-process multiplier freedom and retains dense parameter access/allocation. It is a useful reusable numerical implementation, not final D11/D5 qualification for the current stack. |
| Current model is 1.68M, single-head, with no content-conditioned query | That describes parts of an older model, and even that learner computes a token/history-conditioned query. Main records a roughly 7.15M stack with multi-head Lorentz reads and quaternion transport. Neither description establishes useful chat. |
| No H4 machinery is on the gradient path | Too broad for current main: trained-in 2I transport uses canonical H4 roots and a straight-through backward path. The stronger defensible concern is that the complete prime/zeta/exact-memory architecture is not yet demonstrated as one qualified learned-and-served model. |
| Quaternion transport loses and should be retired | The old 2.110/2.085 comparison is scope-specific. Later transport attribution reports improvements, but its identity arm has 332,928 fewer parameters, so it does not isolate geometry from capacity. S4's approximately +0.0106-nat cost is retention evidence, not superiority. Preserve both results and use matched controls. |
| External work solves chat under our serving restrictions | Useful low-bit chat and efficient kernels exist. The cited examples do not jointly demonstrate transformer-free architecture, no floating-point arithmetic, no multiply/divide numerical instructions, selected weight access, and useful chat on this laptop. |
| QK cannot be ternarized | A failure of a particular quantized-Q/K training recipe is not a general impossibility theorem. Quantizing Q/K, quantizing their similarity scores and normalizing those scores are distinct interventions. |
| Float autodiff plus integer export is exactly QAT | Only if the relevant deployed quantization is present during learning's forward path, with a documented surrogate/backward treatment. Export alone does not establish this. Training from scratch is also not the only route; QAT adaptation exists. |
| Memory literature validates exact prime semantics | It supports useful external/sparse memory. Learned nearest-neighbor search, product keys and colliding normalized n-gram hashes are different mechanisms. They do not establish collision-free occurrence identity, prime semantic distance or inexpensive whole-history retrieval. |
| Exactly three legitimate geometric niches; no quality gains | An unsupported universal restriction. Quaternion ASR results include modest quality gains at fewer parameters, with task-specific limits. Relative rotations, structured sharing, group tracking, composition and geometric codebooks warrant scoped tests, not guaranteed general benefits. |
| 1–3B and 100B tokens are the required next step | The literature supplies examples, not a necessary or sufficient threshold. BitNet's cited chat result uses 4T tokens plus post-training. A massive campaign would combine architecture, optimization, data and resource risks before identifying our present failure. |

## External precedents, accurately scoped

BitNet's official card reports MT-bench 5.85 and explicitly identifies a transformer. Pinned official TL1/TL2 code includes floating-point scaling multiplies. T-MAC's reported 30 tokens/s single-core result is on M2 Ultra and retains a mixed-precision host model. Neither is a complete D11 instruction audit. [BitNet card](https://huggingface.co/microsoft/bitnet-b1.58-2B-4T), [TL1 source](https://github.com/microsoft/BitNet/blob/0b341e582afbf9e1011f24744b554c96a3477eb5/preset_kernels/bitnet_b1_58-3B/bitnet-lut-kernels-tl1.h#L120), [T-MAC](https://arxiv.org/html/2407.00088v2).

MatMul-free LM is a particularly relevant recurrent comparator: its 2.7B experiment reports zero-shot results close to Transformer++, not chat qualification. The 59.4-token/s Loihi figure concerns 370M, and complete-model latency is derived from single-block hardware measurements with inter-chip assumptions. MatMul-free does not imply no elementwise multiplication. [Paper v7](https://arxiv.org/html/2406.02528v7).

I-BERT already supplies integer-only nonlinear inference including softmax approximations. Finite integer normalization can be exact relative to its specified table/rounding/mass contract without equaling real exponential softmax or all F32 outputs. It cannot restore information already discarded from Q/K. [I-BERT](https://proceedings.mlr.press/v139/kim21d.html).

The cited community Llama3 ternary model used a 100B-token adaptation campaign; it is not training-free PTQ. ParetoQ demonstrates full-precision pretraining followed by QAT. These support inspecting our actual discretized forward graph, not rejecting every conversion route. [Community model card](https://huggingface.co/HF1BitLLM/Llama3-8B-1.58-100B-tokens), [ParetoQ](https://arxiv.org/html/2502.02631v2).

Engram is a real 2026 precedent for predictable sparse n-gram lookup and contextual integration, with MMLU improvement in its comparison. It augments a transformer, permits hash collisions, normalizes tokens and retains floating-point operations. Its indexing advantage does not prove prime semantic geometry or exact conversation-version semantics. [Engram](https://arxiv.org/html/2601.07372v1).

Kimi's GHRR language-model reference is supported by the May 2026 v2, which includes language experiments absent from older impressions of the paper. It replaces attention inside a transformer using complex matrix binding and softmax; it motivates a composition mechanism rather than proving our serving contract. [GHRR v2](https://arxiv.org/html/2405.09689v2).

## First-principles implications

Representation, selection, composition and emission can fail independently. More state width does not teach the query which record matters. More heads do not guarantee independent useful reads. Exact storage guarantees retained information only under its write/version/eviction contract; it does not guarantee selection or use. A copying channel preserves unfamiliar values only if the model learns when and where to copy. A parameter-compressed quaternion map still has a mathematical linear action, with execution and bandwidth costs that need measurement.

For a fixed frequency w, ordinary rotations satisfy R(mw)^T R(nw) = R((n-m)w). Wolfram verified this identity. It explains relative-position structure for arbitrary fixed schedules. It does not privilege zeta zeros. A zeta schedule should therefore be compared with ordinary, random-fixed and learned schedules at matched feature count, precision, context lengths and cost. Similarly, a common orthogonal transform of both query and key preserves dot products; useful changes must come from relative/content-dependent transforms, asymmetric operations, transported values or a changed readout.

Slots are a reasonable hypothesis for independent entity/role/version retention. They require an explicit write/allocation/eviction rule, addressing mechanism and read budget. A K-by-256 state without these is a larger array, not a memory solution. With shared operators, adding slots need not imply billions of weights: 256 slots of 256 FP32 coordinates take 256 KiB per layer/session. Similarly, a 256-to-4096 tied readout restricts the family of logit distributions, but the dimension inequality alone does not prove inability to identify any of the 4,096 tokens. Multiple heads and hops should be added only after identifying whether the information exists but is not selected, or is selected but cannot be composed.

Capacity is plausible but not proven as the dominant present bottleneck. The old 1.68M-to-2–3B parameter ratio is about 1,190–1,786; the 30M-to-100B token-visit ratio is about 3,333. Neither axis is five orders of magnitude. Repeated visits are not unique training data. Raw 4-bit weights for 1–3B parameters occupy roughly 0.47–1.40 GiB, but four FP32 arrays for weights, gradients and two optimizer moments already occupy 14.9–44.7 GiB before activations. Alternative optimizers/offloading change that budget; they do not remove its cost. At an illustrative 1,000 tokens/s, 100B visits take 1,157 days; at 10,000/s they take 116 days. These are arithmetic scenarios, not measured M1 rates or a universal lower bound.

## Recommended next decisions

Keep the retrieval-first integrated programme and the transformer-free D17 boundary. Refine its work cards rather than replacing it with Kimi's scaling prescription.

1. **Pin the model under discussion.** Identify the current research stack, retained numerical reference and candidate served bundle separately. Resolve saved-mode/reload defects before using cross-mode evidence as model quality. Reuse valid numerical kernels, but do not freeze an incompatible old session API merely because its arithmetic is correct.
2. **Run the smallest discriminating retrieval experiment already authorized by the current work card once admissions permit.** Use unseen values, paraphrases, distractors, role reversals and relation updates. Compare learned read/copy with NoRead, source edits, an ordinary matched selector and an oracle-selected record. Keep data, capacity and tuning budgets comparable.
3. **Use distinct outcome decisions.** Oracle access succeeds while learned access fails: repair addressing/read training. Both fail despite correct retained records: investigate composition/emission or task learning. Float succeeds and deployed discrete mode fails: fix QAT/export/scale fidelity. Wrong/missing records before querying: fix writes/versioning/eviction. Increasing size/exposure consistently improves held-out behavior after these defects are controlled: justify the next bounded scale step.
4. **Require one independently loaded session artifact.** Exercise copy/selection, transport, exact persisted memory, tokenizer and numerical mode together. Record reload behavior, output changes, parameter bytes touched, latency and memory. Capability, numerical fidelity, architectural compliance and energy remain separate gates.
5. **Use modest learning curves to choose scale.** Compare a small set of predeclared sizes and exposures using actual throughput and complete resource projections. Include teaching/distillation as an offline option. Do not assume 20 tokens/parameter, 100B tokens or one billion parameters is a universal law.
6. **Evaluate geometric mechanisms by causal role.** Use parameter-sharing real controls for quaternion structure, alternative frequency schedules for zeta phases, exact identity beside approximate retrieval, and trained-in versus post-hoc discretization controls for finite groups/codebooks. D17 retention is not an advantage claim. D12 preserves failed mechanisms and D9 prevents unchanged retries.

The frontier-equivalence objective remains an open research objective. None of this evidence establishes that the proposed geometry can reach it, or that a locally affordable training route exists. It also does not establish that geometry cannot materially improve the quality/cost tradeoff. The responsible direction is a sequence of integrated, discriminating results that reduce that uncertainty while preserving the project's intended architecture.

## Evidence navigation

- [Current plan at inspected main](https://github.com/UOR-Foundation/uor-r4/blob/61bbd85a0a49e1bb72c71b4e02752d575e3bee4b/docs/plans/2026-09-29-path-to-chat.md)
- [Current-state retained stack evidence](https://github.com/UOR-Foundation/uor-r4/blob/61bbd85a0a49e1bb72c71b4e02752d575e3bee4b/docs/integration/current-state.md#L666)
- [Standalone integer-serving scope](https://github.com/UOR-Foundation/uor-r4/blob/61bbd85a0a49e1bb72c71b4e02752d575e3bee4b/docs/integration/integer-serving-result-2026-09-25.md)
- [D11, D12 and D17](https://github.com/UOR-Foundation/uor-r4/blob/61bbd85a0a49e1bb72c71b4e02752d575e3bee4b/docs/integration/DECISIONS.md)
- Supporting records: `kimi-review-source-audit.md`, `kimi-review-literature-audit.md`, `kimi-review-first-principles.md`, `kimi-review-wolfram-checks.json`.
