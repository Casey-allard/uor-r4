# Cold-context architecture review after PR #1292

Date: September 19, 2026. Source audited: `5b5bc8f5aa842f73f635cdc7f3b4f5531d7c3622`. This is a source/mathematical/literature review, not a new model run. The [next execution prompt](deepseek-cold-context-step-2026-09-19.md) turns the findings into one bounded implementation and pilot. The [previous takeover review](takeover-review-2026-09-19.md) remains the broader history and mechanism map.

## Decision

Keep DeepSeek's decision not to spend the large training block on the unchanged memory-only configuration. Implement an **always-available learned local-context prior plus the existing causal memory residual**, with one shared low-bit decoder. Use exact token identities for the prior. Keep the current modulo-pair memory addressing unchanged in this experiment so its contribution remains identifiable.

This is a small learned substrate inside the geometric research programme. It is not yet a demonstration that H4/2I/zeta geometry improves language. It establishes the missing separation between corpus knowledge stored in parameters and evidence written during the current prefix. The architecture should eventually combine this with learned geometric state/transport and exact provenance memory; no integrated artifact currently does so.

## What the last run established

PR [#1292](https://github.com/UOR-Foundation/uor-r4/pull/1292) is merged. Reviewed head `80e076bbc8d6589040e5d0d7bb84d93340df964e` and merge `5b5bc8f5` have the same Git tree, `2e4168255fd10e71c23369af4794ed7c2148e57e`. The [qualification receipt](../evidence/native_geometric_causal_qualification_2026-09-19.txt) is preserved.

- Checked short-prefix padding repairs the unsigned underflow. Removing the second sequence-length factor from value/kernel gradients is consistent with mean-loss credit already normalized at the output.
- Token-only coverage replays the causal write-before-read sequence correctly. At V=4096 and 64-token resets, recorded empty-query fractions are 0.9381 for docs and 0.8549 for crates. The memory-only, zero-bias path necessarily assigns uniform logits at these positions. Therefore its loss on the measured population is at least 11.2575 and 10.2586 bits/target respectively. This bound does not require fitted value weights.
- Batch-8, length-64, dv-128 complete trainer calls were timed at a warm mean 0.5649 s over five samples. This supplies a useful starting estimate for a new projection. It does not time a new prior, batch 32, checkpoint serialization, or a complete training run.
- The six-step timing probe performed optimizer updates. No convergence pilot or retained real-text model artifact was delivered. Retuned filter fixtures are open development experiments; they do not preserve the original learning-rate results.

The missing path is real. However, a low empty-route fraction is not itself a requirement for a useful hybrid: memory can improve a small valuable subset while a learned base predicts everywhere. Its conditional and aggregate contribution must be measured.

## Corrections still needed

These are source-derived findings, not newly executed tests. Repair them with the next implementation, rather than spending another run only auditing.

| Finding at audited source | Consequence and repair |
| --- | --- |
| `geometric-realtext-ceiling.rs::blended_argmax` initializes `best_p` with `unigram_p`, then compares it with interpolated `family_p` values | Top-1 remains wrong for some nonzero interpolation weights. Initialize the incumbent with `family_p` too. Test exhaustive vocabulary argmax against the sparse search at nonzero and mixed lambdas, ties and unseen contexts. This defect does not change target-probability CE. |
| `geometric_attention.rs::reference_order2` multiplies the value-gradient suffix sum by `sv[t]`; runtime accumulation does not | For the declared effective-weight STE, `q(w)=s*Q(w/s)` with frozen scale gives derivative one, so the extra scale is wrong in the reference. Existing fixtures use values ±1 and miss it. Correct the reference and exercise nonunit scales, active normalization and nonzero gradients before any optimizer/clipping. Do not change a correct runtime derivative to match the defective reference. |
| Coverage aggregates all collected documents and scores window targets beginning at offset one; count scores use held documents and targets beginning at offset two | The cold-route bound is valid for its own population, but the quoted numerical comparison with the held-document unigram is not a matched-population proof. Make one manifest and target mask for the next comparison. The structural reason for adding a prior remains. |
| Count BPB divides losses for targets after token two by the decoded bytes of whole held documents | Label the old quantity as partial-target loss per whole-document byte. Prefer bits/scored target for the pilot; compute exact scored-byte denominators before new BPB claims. |
| Coverage successor totals are divided by queried-address counts, not consistently by written-address entries or live-query events | The reported near-one statistic does not establish that useful live reads are unique or uninformative. Report separately defined query-weighted and written-bucket statistics if needed. Do not use this statistic to retire memory. |
| Probe reports 512 tokens/step, but predicts 8×63=504 targets; evaluation is after six updates; `to_core()` is only conversion | Report both processed input tokens and scored targets: about 906.4 and 892.2 per second at that measured mean. Call the loss post-probe, not untrained. Measure an actual save/load separately. Scaling batch-8 timing to batch 32 is a projection. |
| Receipt has a truncated crates digest, no complete durable split manifest, and inconsistent 40/44 test totals across accounts | Recover raw output if available; otherwise retain these as unavailable provenance details. List actual test names/counts for the next run. Do not fabricate a missing digest from a later corpus. `norm_bits=0` disables the shift; it does not normalize to zero bits as one receipt sentence states. |

For the argmax error, a simple distribution example is sufficient: unigram `(0.6,0.3,0.1)`, conditional `(0.1,0.5,0.4)`, lambda 0.5 gives `(0.35,0.40,0.25)`. A search initialized at the *unmixed* threshold 0.6 incorrectly keeps the first class.

The optional trainer `readout_bias` is currently a floating diagnostic, and `to_core()` drops its bias. Turning that flag on is not a served-prior implementation. A constant prior also cannot respond to token identity or order. Include an exported quantized constant/bias control, but do not mistake it for the main experiment.

## Selected mechanism

At prediction of token `x[t+1]`, let `a[t]` be the current ordered residue-pair address and `S[t]` contain only already observed successor writes. Use:

```text
p[t] = bounded_relu(E_old[x[t-1]] + E_new[x[t]])
m[t] = bounded_signed_read(S[t][a[t]])
h[t] = p[t] + m[t]
z[t] = shared_lowbit_readout(h[t]) + quantized_bias
```

`E_old` and `E_new` are position-specific learned low-bit row tables addressed by the full 4,096-token ID, with an explicit missing-prefix marker outside the real vocabulary. The prior is present on both cold and warm routes. The memory read is zero when unwritten. The two streams have separately declared fixed-point scale/range bounds before addition; normalizing only their sum can erase one stream. A ReLU/clamp gives a small nonlinear context interaction. Without it, a linear readout of the summed rows is only an additive log-bilinear predictor. No learned expert gate, dense hidden transition, second vocabulary projection or floating sampler is required.

This fusion is a hidden-state residual. Under a shared linear decoder it adds logits and is product-like after softmax; it is **not** a probability mixture. A fixed common logit scale, integer bias representation, saturation and surrogate derivatives must be defined and exported. Float softmax/log loss belongs to offline training/evaluation; start served generation with integer greedy argmax.

At V=4096, dv=128, two new row tables contain 1,048,576 low-bit coefficients. Reusing the existing `lowbit::TernaryLinear` representation gives 256 KiB of packed two-bit weights plus 32 KiB of u32 row shifts, before absent-token rows and metadata. Temporary offline quantization codes use i8 and are a separate allocation. Four f32 arrays for masters, gradients and Adam add about 16 MiB. Two selected row reads and O(dv) arithmetic feed the existing single 524,288-coefficient vocabulary readout. Report actual bytes touched: adding many sparsely accessed parameters can lower the accessed fraction without reducing absolute work. Full-vocabulary decoding remains the dominant cost, and candidate routing is deferred until predictive learning is demonstrated.

Exact-ID rows remove a forced alias in the prior; they do not guarantee learned embeddings will be distinct or useful. The fast memory still has modulo collisions and must not be described as exact occurrence/version memory. That older subsystem is separate.

## Alternatives and why they are not first

| Alternative | Assessment |
| --- | --- |
| Learned constant bias | Necessary baseline and useful calibration, but only a corpus marginal. Does not establish context learning. |
| Learned initial vector for each of 14,400 residue-pair buckets | A plausible later control; it preserves the forced token aliases and entangles persistent knowledge with adaptive writes. Two exact-ID tables are smaller in coefficient count and allow sharing across unseen pairs. Neither is guaranteed to win. |
| Persist state across documents or greatly widen windows | Changes privacy/reset and information boundaries, still leaves cold starts, and cannot be silently substituted for the stated task. Evaluate within-document persistence later if coverage is a demonstrated limit. |
| Exact n-gram/cache probabilities | Strong, inexpensive controls with explicit fit/causal regimes. Do not insert their answers into the candidate and call the result learned geometric prediction. |
| Product/compositional codes or learned root assignments immediately | Promising compression/structured routing mechanisms. Full-token rows are cheap at this vocabulary, so adding a code learner now would confound the missing-prior diagnosis. |
| Delta-rule writes, learned gates or multi-scale geometric recurrence immediately | Candidate future ways to improve adaptive memory and long context. Current evidence first demands a cold prediction path and an actual two-channel contribution. |

## Research refresh and geometric direction

This was a targeted primary-source refresh through September 19, not a claim to exhaust all available mathematics.

- [Bengio et al., A Neural Probabilistic Language Model](https://www.jmlr.org/papers/v3/bengio03a.html) supplies the basic case for sharing statistical strength through learned token representations in a finite-context predictor. The proposed bounded integer implementation is a project-specific hypothesis, not that paper's tested architecture.
- [Grave, Joulin and Usunier, Continuous Cache](https://arxiv.org/abs/1612.04426) supports separating a learned base from adaptation to recent history. Their similarity scoring and probability interpolation differ from this addressed residual; do not import their results or formula unchanged.
- [Engram, July 2026 revision](https://arxiv.org/abs/2601.07372v2) is a current example of static conditional lookup complementing dynamic computation. It augments a transformer/MoE backbone and does not establish a standalone multiplier-free model. The official [demo at source pin `fb7f84a`](https://github.com/deepseek-ai/Engram/blob/fb7f84a21f91223715394a33a1dc24bbfb7f788e/engram_demo_v1.py) was inspected: it uses linear projections, RMSNorm, variable products, square roots and sigmoid gates. Adopt the decomposition lesson, not this implementation or its large-model benchmark claims.
- [Lngram, May 2026](https://arxiv.org/abs/2605.24869) learns discrete latent symbols for conditional memory; it is a relevant future learned-packaging direction. Its reported backbone-assisted results do not establish our geometry or integer serving path.
- [Memory Grafting, May 2026](https://arxiv.org/abs/2605.20948) constructs conditional memory from offline model hidden states. This provides a possible declared-teacher research path, but adds donor dependence and a new experiment; it is not needed to diagnose the current missing prior.
- [Compositional code learning](https://arxiv.org/abs/1711.01068) supplies a later representation-compression option. Code capacity and exact token identity remain separate requirements.

Keep the project's prime/zeta/R4/S3/H4/2I/icosian programme as the intended structure. The next geometric comparison should assign a concrete role to a directed relative action or learned multi-component packaging, then compare with matched identity/randomized structure at equal data and cost. A fixed invertible permutation in front of an unconstrained learned table can be absorbed into that table; its inclusion alone does not test geometry. Do not describe modulo-120 pair indexing as group transport. Preserve orientation and exact lexical identity separately from any learned geometric code. Only select that next operator after the prior/memory result identifies where it is needed.

## Resources and delivery state

At this review, the operational JSON is `147638565 / 154400000 ms`, leaving `6761435 ms` (112.69 minutes). It is the recorded balance, not a newly certified historical sum. No model/build charge or limit change was made in this documentation review. Reused an existing clean isolated full worktree on a new `codex/cold-context-handoff` branch; original checkout and all research/artifacts were preserved. Host free space was about 44 GiB; that is not a full storage-accounting certificate.

The next prompt proposes a complete 3,600,000 ms tranche, including preparation, focused repairs/builds, two matched small fits, controls, export/reload, generation and reserve. Record the actual projection before execution and recompute from new-path timing. Necessary local extensions remain owner-authorized when recorded before use. No paid compute or destructive cleanup is authorized. #973 stays active/open; #820 stays the programme tracker. Both have no linked GitHub project items at inspection, so there is no board status to invent.
