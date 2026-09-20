# Review after PR #1304 — preserve the negative, isolate the next question

Audited base: `88e7122122717333aed7ee79049854337cf60ead`, September 20, 2026. [Next complete DeepSeek prompt](deepseek-readout-diagnostic-step-2026-09-20.md). This review inspected source, retained files, saved statistics and targeted primary research; no Rust build, model forward or training ran.

## Decision

**Recover the rejected prefix candidates as replayable hard artifacts without retraining, correct their generation/evidence reports, then execute one bounded fixed-feature readout diagnostic.** Compare an empirical conditional-target objective against the smoothed count objective with identical output-head training. Use a floating output-only relaxation only under the specified unresolved-result branch. Do not require matching the count reference before returning to learned geometric state research.

The negative prefix result remains a useful hard-forward development observation. The count gap is unrecovered local predictive performance, not an isolated diagnosis of the output head, and not evidence that older context is unneeded. This review corrects the stronger conclusions and incomplete delivery claims in the original receipt; that raw receipt is preserved.

## What survives

PR [#1304](https://github.com/UOR-Foundation/uor-r4/pull/1304) is merged. Head `d43ac8d5e06655b17ce8901804c0ab028cab2826` and merge share tree `c53dbc370b4dcb5be1c635b908b91ff4b7f52ad3`. The source implements the intended causal older-prefix slice, code-unit gradient scales, once-only averaging, categorical surrogate, 64 reader warm-up updates and 192 action updates. The three arms each received 256 batch-eight updates over four passes of 512 windows / 32,046 distinct scored occurrences. Changed final hard maps establish actual discrete action learning, not beneficial history use.

| Predictor, same 36-doc / 288-window / 17,342-target development panel | CE bits/target |
| --- | ---: |
| Frozen parent | 7.558429 |
| Learned older-prefix, step 256 | 7.575604 |
| Fixed-action older-prefix | 7.723496 |
| Learned local-tail | 7.576439 |
| Count reference: 512-window conditional counts | 7.079256 |
| Count reference: 4,096-window conditional counts | 5.938052 |
| Count reference: full-fit conditional counts | 5.061160 |

The learned prefix loses about 0.01717 bits to the parent, beats the fixed carrier by 0.14789 and has no selected advantage over the local-tail control (+0.000835, interval includes zero). Reported conditional-permutation penalty is −0.001532 bits on 1,177 eligible targets, interval [−0.015194,+0.012677]. Support passes the predeclared minimum but covers only about 7% of history-bearing records. This is no detected benefit under this intervention, not demonstrated equivalence or absence of useful older information.

All sealed member sets/hashes for probe, both pilots, derived corrections and original prior/replay/parity roots verify. Parent CPL2 and CPCK remain unchanged. The second pilot's three final CPXS checkpoints are byte-identical to the first pilot's: changing generation labels caused a needless repeat of the fits. Future report-only repairs must recover saved parameters.

**Independent verification has a boundary.** Three arm and five reference per-document rows are saved; arm-versus-arm means/intervals can be reconstructed. The parent, permuted and reversed per-document/per-occurrence loss vectors are not saved. Their aggregate reported effects and CIs are source-consistent but cannot be independently regenerated from saved arithmetic alone. Recovery replay must persist those sufficient statistics. Do not claim every headline interval was independently reproduced in this review.

## Corrections required, with their scope

1. **A hash descriptor is not a serving artifact.** `prefix-state-pilot.rs:1104–1120` writes floating CPXS checkpoints and `.cpl2.json` files containing action IDs and reader/output hashes, without the packed reader/output bytes or executable group table. Evaluation uses live `hard_core()` values. Numerical hard forward was measured; independently reloaded hard-prefix inference was not.
2. **Group identity was not bound as requested.** `prefix_state.rs:364/380` calls `group_table()`, which lazily constructs products by floating nearest-root classification. The intended exact artifact-bound table and root-order mapping were not serialized. A correct recovery must establish all 14,400 products against the exact signed canonical construction and preserve the learned function through an explicit index mapping. This is a served-initialization qualification gap, not evidence that measured integer products were numerically wrong.
3. **Two generation comparators are invalid.** Both `PrefixCore::generate` and runner `generate_full_state` always use older-prefix state, including for the local-tail arm. Its teacher-forced CE uses the right state; its named generation does not. `generate_reference` scans only `{unigram mode, prev, cur}`, not all V4096 candidates. Its reported collapse is not count-model greedy behavior. Repair both and record fresh generation from the same parameters/prompts.
4. **Complete mid-pass continuation is not established.** CPXS saves masters/moments/update ages/pass, but no runner schedule/cursor or bound optimizer/group contract. `resume_from` adopts the stored data identity instead of rejecting mismatches; the test manually repeats supplied batches. Its `parent_sha256` is SHA256 of the hexadecimal parent-hash string, not the parent-file digest. Preserve that legacy convention when recovering; bind the actual parent digest explicitly in new artifacts. Do not spend this task building continuation for a rejected prefix branch; the new diagnostic must have complete continuation identity.
5. **Some measurements are mislabeled.** The probe performs two batch updates over sixteen windows, not sixteen updates. The quoted action-gradient norm is the pre-softmax-Jacobian palette adjoint norm, not the Adam action-logit gradient norm. The reported 9,891-entry /162 MB cache is the initial panel cache; saved final cache is 21,251 entries /348,176,384 bytes. Reference `gain_vs_parent` fields have the opposite sign convention to arm gains. These do not overturn the CE values or changed hard action IDs.
6. **The cumulative charge is recorded, not verified wall time.** JSON and prose agree at `159438565 /159800000 ms`. The 5,400,000 ms charge exactly equals the projection, whereas PR #1303 to #1304 merge timestamps span only 33m48s. Listed model/probe timings do not support a measured 90-minute wall charge. Preserve the existing conservative charge, mark its basis unverified, and do not invent a refund. Subsequent work must charge measured nonoverlapping elapsed intervals, not the full reservation.

The corrected 38,152 document-relative offsets and pair fixed-point rule are supported; original sealed reports remain intact. Report-directory validity is preserved even when a scientific label is corrected.

## Mathematical interpretation

The local prior already has interaction through `h(a,b)=normalize(ReLU(E_old[a]+E_new[b]))`; it is not simply additive token logits. Its output still has `Z(c)=H(c)W^T+B`. After subtracting fixed bias, the context-by-token log-odds matrix has rank at most 128, with additional ternary/dyadic restrictions. An arbitrary count table need not be representable. This is a structural limit, not a measurement attributing the present gap to rank.

The rejected prefix mechanism is `Z(c,q)=Z_parent(c)+r(q)`. For two prefix states q/q' and output tokens u/v, their change in log-odds is

`2^-F * [r_u(q)-r_v(q)-r_u(q')+r_v(q')]`,

independent of the local context c. A query cannot change how that stored state affects the u/v preference. One register's capacity, learning surrogate and this separable read all remain possible limitations. A future read such as `R[q * a(cur)]`, implemented by an exact group lookup, would introduce a context-dependent permutation; it is a hypothesis for a later measured mechanism, not an adopted result or a claim of sufficient capacity. Exact occurrence/version memory and selective overwrite remain separate needs.

## Why the next diagnostic is more informative than unpaired distillation

Freeze both parent embedding tables, normalization, bias and F. Only the existing V4096×128 output head changes. Train two heads from the same verified parent output masters with fresh moments, identical occurrence-weighted contexts and dose:

- empirical conditional histogram `p_emp(v|c)=C(c,v)/N_c`;
- the existing normalized interpolated count distribution `q(v|c)`.

The empirical objective satisfies `sum_c N_c H(p_emp,p_theta) = -sum_(c,v) C(c,v) log2 p_theta(v|c)`: it is the original token-CE population objective. Using complete conditional histograms for BOTH arms avoids adding an unmatched target-sampling variance change. Never weight unique contexts uniformly.

For the smoothed arm, `H(q,p)=H(q)+KL(q||p)`, and score credit is `(p-q)*2^-F/ln2`. This can test whether the target estimator helps this frozen representation; it does not grant a larger student function class. The full-fit unigram used in backoff and the parent's bias remain shared prior information, so “4,096-window conditional counts” is more accurate than “all information came from 4,096 windows.”

A quantizer-specific initialization hazard matters: constructing masters exactly as `code*2^shift` puts every nonzero row maximum on a dyadic scale boundary. For a positive row shift, a tiny decrease can halve its effective scale; shift 0 is clamped. Prefer the preserved CPCK output masters after independently verifying that they quantize to the exact CPL2 head, then reset moments and ages. This is a new output-only fit, not resuming the legacy run. Record shift changes separately from code changes.

If the soft hard-head result remains unresolved, one bounded floating head on the SAME frozen integer features relaxes only the output constraint. Its objective is convex, but a finite optimizer run is not a certified optimum. A float win combines continuous expressivity and easier optimization; it does not isolate quantized representability from STE difficulty. The floating head never becomes a serving path.

## Research used and transfer limits

- [Yang et al., softmax bottleneck](https://arxiv.org/abs/1711.03953): the output-factorization restriction motivates measuring approximation separately from optimization; it does not prove that rank causes this particular gap or justify silently adopting a mixture-of-softmax architecture.
- [Hinton et al., distillation](https://arxiv.org/abs/1503.02531), and [Malagutti et al., NAACL 2024, n-gram smoothing](https://aclanthology.org/2024.naacl-long.382/): distributional supervision and count-derived regularization justify the bounded target-objective comparison. The teacher here is an explicit local statistical estimator, not a runtime provider.
- [BitDistiller](https://arxiv.org/abs/2402.10631) and [ParetoQ, revised October 2025](https://arxiv.org/abs/2502.02631): low-bit training depends on objective/quantizer/training choices; their pretrained-model results are not UOR-R4 evidence.
- [Ternary Mamba, June 2026](https://arxiv.org/abs/2606.18114): a recent QAT/distillation example also reports scale-related failure modes. Its pretrained large SSM, FP16 activations, GPU training and inference design do not meet or validate this project's complete D0-b target. No architecture or code from it is adopted.

## Roadmap and resources

The sequence is: preserve/recover the scoped prefix negative; run one output-objective diagnostic; choose between training/quantizer work, a conditional geometric read, and a representation change using its observed failure or gain; then connect selective state/read and exact memory toward the same conversation/coding artifact. There is no “perfect local count match” stage lock. Geometry remains the architectural priority; the local head is an explicitly identified component, not geometric advantage.

Current recorded balance: **159,438,565 /159,800,000 ms**, **361,435 ms** remaining. The next prompt proposes a **5,400,000 ms** complete tranche and corresponding local limit extension, making the proposed limit **165,200,000 ms**. This review does not apply it or run a model. Record it before work under standing authorization, then charge actual elapsed intervals. Keep one worker, at most four Cargo jobs, 8 GiB RSS, 512 MiB new retained/temporary data plus 1 GiB incremental reused build output, and the 128 MiB storage margin. Preserve original checkouts and all negative artifacts; no paid compute or deletion.
