# Review after PR #1302: test learned older-prefix information explicitly

Audited base: `3095c1d48e213deb234d31d21c94da7adcca7e6e`, September 20, 2026. The [complete next DeepSeek prompt](deepseek-ordered-prefix-step-2026-09-20.md) implements one learned ordered prefix-state/read pilot above the frozen prior. This review inspected source, artifact/data identities and saved statistics, with targeted primary research. No Rust build, model forward or training ran.

## Decision and retained result

**Advance to one bounded learned older-prefix channel, with fixed-action and fitted local-only controls. Do not retrain the existing prior or repeat the full frozen replay.** The new experiment asks whether a small exact group state learns predictive information beyond the existing two-token input. That is a project-directed hypothesis, not a conclusion established by the count-reference gap.

PR [#1302](https://github.com/UOR-Foundation/uor-r4/pull/1302) is merged; reviewed head `6027d0e61b95bcd23688585479e8106f92e9341e` and merge share tree `dff3bc695a346505c6775f7c3ed087bc7c3508df`. The corrected contextual-learning result survives independent saved-data verification:

| Development population | Frozen model CE | Exact unigram CE | Quantized bias CE | Full-panel permutation penalty |
| --- | ---: | ---: | ---: | ---: |
| Legacy, 32 docs / 6,048 targets | 7.142472 | 9.059915 | 9.131630 | +3.057214 |
| First/last windows, 36 docs / 3,734 targets | 7.472841 | 9.243832 | 9.309115 | +2.836726 |

All eight scorers' micro/macro means and all six full-panel paired confidence intervals reproduce from saved document rows. The legacy donor mapping has 6,048 unique occurrence IDs, a valid bijection within document/PAD strata, unchanged targets and the recorded 5,996 moved / 5,992 changed-context counts. Both development gates remain supported. The 36-document panel overlaps 32 legacy documents; this is a positional extension on correlated repository prose, not independent final qualification.

The 14 replay members, three parity-before members and two parity-after members match BLAKE3/size entries with zero unlisted files. All 429 corpus document hashes and sizes match retained inputs. Summing the 4,096 consumed windows gives **257,113 targets**. Replay `result.json` SHA256 is `6db96a1c7f9cccefe13b9c07a610bbb02559ece6e3762a7c36f35ba9dcc6f661`; retained evaluator binary SHA256 is `7137852298b0b3d1ebee758d9bfca520cf6f235b45ae51e61434466fc99860e1`. Original CPL2 SHA256 remains `cd5a3aa1b804ccc3582a4a25090e25c862489e0bc4e5bb86c1d1f363a7338a00`.

## Corrections that do not overturn the loss result

1. **The count gap is not a capacity diagnosis.** The full-fit and consumed-count references score 3.9691 and 5.1965 bits on the same inputs as the 7.1425-bit learned model. This shows unrecovered local predictive performance. It does not separate finite optimization dose, quantization, regularization and representational capacity. The consumed reference does not need later fit windows, but it uses a different fitting procedure and the full-reference smoothing choice. Missing older context cannot explain this same-input comparison. Accordingly, a new prefix channel needs a fitted local-capacity control, not just comparison with the frozen parent.

2. **One-step repetition is not a fixed point.** `prior-frozen-evaluate.rs:985` tests only `best == cur`; the pair map requires `prev == cur && best == cur`. The recorded path `(284,198) -> (198,198) -> (198,504)` exits newline; it is not a self-loop. `(32,32) -> (32,32)` is a genuine fixed point. The actual generated-trace pair detector is correct, so all six observed period-one trajectories remain valid for the old prior. Count-reference generation was not run; agreement at one pair establishes a shared local choice, not the cause or frequency of entering it. Do not conclude the entire collapse is unrelated to learning defects.

3. **Fit-window provenance needs a derived correction.** Index writers at lines 1818/1826 use global window ID times 64 rather than document-relative window index times 64. Consequently 38,152 of 38,987 token offsets are wrong. Document IDs, within-document indices, lengths and byte offsets remain valid, so exposure and measured CE are unaffected. Correct both writers and create a new derived index; preserve the sealed original. This matters before reading older prefixes.

4. **Some reporting labels exceed execution.** The 38 tune documents form a pool; the 64-window tuning loop actually uses 32 documents. The recorded permutation seed string disagrees with source (`0xA5A51234`, effective initial state `0xA5A51235`); the persisted donor map preserves the intervention. Historical "eligible" means changed-context records, not every permutable record. The supposedly nontraining parity test performs 60 small optimizer steps: the real-text artifact was frozen, but not every test was update-free. None of these facts requires repeating the full replay.

5. **The shared resource JSON was stale.** It still contained `152238565 /154400000 ms` despite the merged prose recording another 1,800,000 ms. This review reconciled that existing charge exactly once, giving **154038565 /154400000 ms**, with **361435 ms (6.02 minutes)** remaining. No new review work was charged. Preserve the charge as recorded; the complete approximate build/test duration was not independently reconstructed. The earlier prospective million-millisecond extension was released in the final receipt and is not active headroom.

## The selected mechanism

Freeze the retained local prior, including embeddings, readout, biases, tokenizer and exponent. At prediction of `x[i+1]`, it consumes `x[i-1],x[i]`. The new channel summarizes only the older portion of the same at-most-64-token context, excluding that pair:

`q_i = chronological_product(action(x[j])) for max(0,i-63) <= j <= i-2`.

An empty older prefix contributes zero. Use one exact 120-element binary-icosahedral state, artifact-bound signed canonical roots/table, and an **eight-element fixed action palette** containing identity and noncommuting generators/inverses whose closure is the full group. The learned token-to-palette choice uses three bits (packed in nibbles), keeping learned coefficients/action codes within D0-b. A state index is not another learned scalar weight. Palette restriction is a disclosed capacity restriction.

Read one signed ternary width-16 row `R[q]`, apply a separate ternary `Wg[V,16]`, and add its bounded integer scores to the frozen prior. A separate small reader avoids restricting the new state to the existing underperforming decoder's directions. It adds 65,536 output coefficient visits at V4096, one eighth of the parent's 524,288, plus at most 62 group lookups. These are counts, not a latency or energy result. Action masters/moments at V4096/K8 use approximately 384 KiB before gradients; reader/output masters and working memory are additional. The frozen prior still dominates inference arithmetic.

Train with the **same hard integer forward used at serving**, and an explicitly biased categorical straight-through surrogate offline. For a distributional extension of composition, `c[h] = sum_{s,k: s*gamma[k]=h} p[s] pi[k]`. At hard state `q` and action `k0`, an incoming adjoint `u[h]` gives:

- `d_previous[s] = u[s*gamma[k0]]`;
- `d_action[k] = u[q*gamma[k]]`;
- in code units, reader state credit is `u[s]=sum_j g[j]*Rcode[s,j]`, with `g[j]=256*sum_v dZ[v]*Wcode[v,j]`; do not apply another dyadic scale.

Apply the softmax Jacobian only to the offline action logits. This takes O(120+K) per transition, not a dense 120² forward. It is not an exact derivative of hard argmax or a proof of learnability. Independently test these equations on the continuous multilinear extension, then judge actual exported hard loss and actions. A zero output with nonzero reader initialization permits a short reader warm-up; zeroing both factors would block all useful gradient.

One seed and three fitted variants suffice for this pilot: learned older-prefix actions/reader; the same reader with frozen initial actions; and the same learned palette/reader applied only to the local tail. Match data, update count, parameterization and numerical scales, and report differences in actual computational work. The full-fit/count references remain visible, but are not a justification for claiming the new state has solved local fitting.

Conditional permutation must exchange state among occurrences sharing the **exact unchanged local pair and older-prefix length**. Donors may cross development documents, so this estimand includes topic/document information carried by the prefix. Coarse length buckets would allow a pure position counter `q=g^L` to pass without using older token content. An unconditional prefix shuffle can reward recoding local inputs. Freeze donor identities before fitting; report all permutable records, changed states and support separately. Reversed older-prefix order preserves the local pair/multiset but is an out-of-distribution sensitivity diagnostic, not a standalone usefulness or noncommutativity proof. A consistently relabeled group/table is an isomorphism, not a destructive control.

## Research basis and limits

[PD-SSM, Structured Sparse Transition Matrices](https://arxiv.org/html/2509.22284v2), section 3.1, supplies a relevant precedent for deterministic hardmax forward and softmax-Jacobian backward; its Arithmetic ablation is task-specific. We adopt that learning device as a hypothesis, not its dense generators, complex state or language architecture. The group-convolution adjoints above are derived for this restricted finite palette. They need independent fixtures and actual hard-artifact evaluation.

[Diagonal SSM expressive limits, revised August 2026](https://arxiv.org/abs/2603.01959), and [error-control dynamics, May 2026](https://arxiv.org/abs/2605.07755), distinguish algebraic representation, learnability and state error under their particular assumptions. Exact group lookup removes continuous rounding drift, not wrong learned actions or collisions. [Li et al., ICML 2025](https://proceedings.mlr.press/v267/li25r.html) shows how state-tracking tasks can be solved with different mechanisms/heuristics; a finite arithmetic/group witness does not qualify natural-language state use.

A single 120-state register has at most `log2(120) = 6.91` bits. Pure group actions are bijections and cannot selectively overwrite. The context window supplies explicit bounded forgetting in this pilot; context-dependent actions, resets/erasure and exact memory remain later candidates if the measured failure warrants them. Signed H4 identity and the deterministic golden/Galois companion remain distinct; no independent companion capacity is introduced. Prime identities bind records, not semantic distance. Fixed zeta channels are not added without a specific role. The September RiLM paper remains research-only for the reasons in the preceding review.

## Roadmap and resources

The validated local baseline is now available. The next dependency is **measured older-prefix information and learned action contribution**, followed by context-sensitive update/read design, exact occurrence/version-memory integration, and common-artifact conversation/coding with complete M1 costs. A fixed-action win is learned reading of a fixed carrier; a tail-only win suggests added local capacity; a primary win over both with a conditional intervention effect supports this scoped learned prefix channel. None establishes 2I superiority: a matched C120 or other algebra control is required before that separate claim. A negative is scoped to this one register, palette, surrogate, width, dose and data.

The next prompt proposes **5,400,000 ms (90 minutes)** of complete local work and a corresponding **+5,400,000 ms allowance increment**, which would make the limit 159,800,000 ms. That extension is not applied by this review: refresh and record before execution under standing owner authorization. Use one worker, at most four build jobs, 8 GiB RSS, 512 MiB new data/checkpoints plus 1 GiB incremental reused build output, and the 128 MiB protected margin. No paid compute, deletion or new corpus. Preserve all prior artifacts and the owner's checkout.
