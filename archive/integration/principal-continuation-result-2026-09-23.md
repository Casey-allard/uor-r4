# Principal continuation: integer readout, recovered learning, and Hamilton-vector scope

September 23, 2026. This study implements the five continuation workstreams and retains their negative results. It preserves the owner's terminal objective: a fully transformerless geometric language model in which geometry replaces floating-point matrix multiplication. A table-native output layer alone is not that endpoint.

## Decision

Retain a stronger **ordinary-lexical research candidate**, not a general-language or product release. An exact served-weight warm start with a fresh optimizer, served-window supervision and grounded rehearsal reaches **6.27370696 bits/target** from its grounded parent's **7.03267881**, while retaining the declared 32/32 temporal, 36/36 training, 4/4 class and 3/3 held-out grounded panels. A second shuffled-data seed reaches 6.26984301 with the same grounding counts. Free generation remains repetitive, and native post-copy decisions still track the selected-source fingerprint rather than the isolated emitted-token intervention.

A separately selected marginal-relative correction has now been implemented as an integer readout and a loadable composite artifact. It improves the matched local prediction and the four external prose sources, but does not demonstrate H4-specific advantage. A retuned reversed-history control ties the full-history correction. Preserve this distinction before prescribing more geometric state.

[Machine-readable evidence](../evidence/principal-continuation-summary-2026-09-23.json) and [all recorded generation examples](../evidence/principal-generated-examples-2026-09-23.json) accompany this report. The native verifier checked 19 sealed experiment roots and the retained-candidate root. These are executed research records, not CI compatibility acknowledgements.

## 1. Native readout and matched full-Tune adjudication

All primary coefficient comparisons use the same 2,048 stride-selected Tune windows and **114,364 served-conditioning targets** as the preceding main-branch diagnostic. Development remains 24 documents, 96 windows and **5,376 targets**, including 4,954 frequent-token and 422 rare-token targets. An additional 100-pair interpolation search on exactly that Tune population independently retains the existing count weights **(0.7, 0.4)**.

`learner/lexical_residual.rs` compiles Fit-only sparse counts and Q12 log-add tables. Its scoring path uses integer lookups, additions, subtraction and shifts. The `IPL1` prior binds the native model hash; the `PRC2` container holds the native model, prior, independent dyadic coefficients and correction-frame flag. The enclosing sealed report binds tokenizer, corpus and source identities. `PRC1` remains readable. Bounded parsing rejects foreign native bindings, malformed dimensions, invalid indices, truncation and trailing bytes.

The composite preserves the native Generate-group log mass exactly in the implemented Q12 reduction and leaves Copy/Stop scores unchanged. This does not mathematically guarantee unchanged greedy Copy/Stop choices: individual Generate scores can move. The actual authored grounded rollouts are therefore evaluated separately.

| Readout on the same development population | Bits/target | Important qualification |
| --- | ---: | --- |
| Frozen prose artifact | 6.684104957 | Original native scorer |
| Count reference with matched Stop mass | 5.121723622 | Same interpolation retained by full-Tune search |
| Independently weighted raw artifact/count product | 5.069201610 | Rare-token loss worsens by 0.6574 bits; not promoted |
| Marginal-relative correction, floating reference | 5.038016933 | Both head and tail point estimates improve |
| Marginal-relative correction, integer implementation | **5.038010616** | Maximum per-target rounding discrepancy 0.000417797 bits |

The original raw product's lower aggregate loss did not justify concealing its rare-tail regression. The follow-up tests `C^alpha (A/U)^gamma`, where U is the Fit-only unigram. This avoids automatically applying A's marginal preference on top of a prior that already contains it. It makes no conditional-independence claim. Tune-only selection chooses alpha=28/32 and gamma=8/32 from the same declared grid; both lie at grid boundaries, so this is not an unrestricted optimum.

The marginal-relative arm's development difference from counts is **-0.083706689 bits/target**, paired-document interval **[-0.116375, -0.051045]**. Its rare-tail difference is **-0.032126 bits/target as a point estimate**; no separate tail confidence claim is made.

## 2. Matched explanatory controls and transported depth

Every explanatory input receives the same alpha/gamma grid and full-Tune selection opportunity. The marginal-relative full-state model scores 5.03802; its last-two-only counterpart scores 5.06194, a **+0.023924 [0.017963, 0.030968]** disadvantage. However, reversing the older prefix while preserving the final two tokens, and independently retuning its coefficients, scores **5.03750**: **-0.000516 [-0.010976, 0.010406]** versus full history. Thus this experiment does not establish a unique chronological-order or geometric advantage. The current-token-only ratio control scores 5.09956, and U/U reduces exactly to the count-temperature baseline.

The training-free depth observer operates on 4,992 fixed-Generate-event positions with no true token identities supplied to the decoder. At lags one through four, it recovers **4,884 / 3,737 / 933 / 30** identities. Fixed rotated-label controls recover **56 / 46 / 13 / 9**. These are about 97.84%, 74.86%, 18.69% and 0.60% for the actual observer. A low lag-four recovery is a limitation of this approximate greedy observer, not proof that every possible decoder lacks the information. The old blanket previous-token absence claim remains unsupported.

## 3. The warm-start experiment recovers a stronger native learner

`TlTrainer::from_model_fresh_optimizer` validates an existing artifact, reconstructs its masters, explicitly resets optimizer history and requires re-export to reproduce every original model byte. It rejects a source that cannot be reconstructed exactly. This is a **fresh optimization experiment from served weights**, not a complete training-checkpoint resume.

Both schedules start from `olx-form-4` and run 128 updates, with four prose windows and two grounded examples per update, grounded weight 4.0. One schedule keeps learning rate 0.01; the other decays linearly from 0.02 to 0.002. The second shuffle seed changes only the fit-window order. The original two arms are also repeated in a fresh process, reproducing both artifact hashes and losses exactly.

| Seed / schedule | Development bits/target | Train / class / held-out grounding |
| --- | ---: | --- |
| Parent `olx-form-4` | 7.032678813 | 36/36, 4/4, 3/3 |
| 20260923 / constant | 6.300379318 | 36/36, 4/4, **2/3** |
| 20260923 / decay | **6.273706960** | 36/36, 4/4, **3/3** |
| 20260924 / constant | 6.261867705 | 36/36, 4/4, **2/3** |
| 20260924 / decay | 6.269843013 | 36/36, 4/4, **3/3** |

Retain the first seed's decay candidate rather than selecting the best source-evaluation outcome. Its independent 50,000-draw paired-document interval versus its grounded parent is **-0.758972 [-0.822381, -0.694998]**, with all 24 documents improving. Against the earlier best prose-only artifact it is **-0.410398 [-0.488340, -0.330223]**, with 23/24 documents improving. Head loss improves 6.55369 to 5.74819, and tail loss improves 12.65566 to 12.44293.

The warm recipe changes several things together relative to the original run: optimizer moments are reset, latent masters are re-anchored to served weights, the original per-position-trained parent receives served-window supervision, and extra mixed training exposure is supplied. The result is not a single-cause proof about learning-rate reset or quantization. It does contradict treating the earlier doubled, restarted prose-only recipe as exhaustion of every practical optimization route.

The native crossed post-copy intervention still reports fingerprint dependence rather than isolated emitted-token dependence. Passing the authored 32/4/3 panels does not settle that causal obligation or the complete scoped-memory lifecycle.

## 4. Source-family evaluation and same-artifact grounding

The selected coefficients and the first-seed warm candidate were frozen before downloading four Project Gutenberg sources: *Alice's Adventures in Wonderland*, *Pride and Prejudice*, *The Adventures of Sherlock Holmes*, and *Frankenstein*. Each supplies sixteen evenly spaced 64-token windows with eight observed prefix tokens: 896 scored targets per book, **3,584 total**. Source URLs, unmodified downloaded headers/licenses and SHA-256 identities are retained locally. No source text was fitted. This is a small classical-prose domain shift, not a general-language benchmark or a claim about unseen tokenizer pretraining.

| Frozen predictor | External-source bits/target |
| --- | ---: |
| Original grounded native parent | 12.98036 |
| Warm decaying native model | **12.41532** |
| Count-only reference with matched Stop mass | 11.54306 |
| Frozen prose-model integer ratio | **11.42440** |

The native warm model improves on its own parent in all four books; the frozen integer ratio improves on counts in all four. With only four source clusters, no narrow broad-generalization interval is asserted. Absolute losses are still poor and the recorded continuations loop or drift into repository terminology. The original six-prompt generator covers two of the four books; the loss population covers all four. No four-book generation-coverage claim is made.

Transferring the already frozen ratio coefficients to the warm model produces one loaded composite that scores **5.01773800** on development while retaining **32/32 temporal, 36/36 training, 4/4 class and 3/3 held-out authored grounding**. The same transfer to the original grounded parent also retains those panels. Copy/Stop preservation was measured through actual action rollouts, not inferred solely from unchanged row scores. The full prior scoped-session regression suite and a crossed-feedback intervention on the composite were not run.

A secondary warm-composite evaluation scores **11.39628** on the same four books. This is explicitly a follow-up on the now-exposed source set with unchanged coefficients, not a second blind draw. All previous outcomes, including rare-tail harm and lost-grounding constant schedules, remain available.

## 5. Cost, arithmetic and the Hamiltonian lead

The sparse prior serializes to **2,269,044 bytes**; the `PRC2` bundle is **2,735,770 bytes**. A nine-round interleaved M1 microbenchmark over 64 stored contexts measures median **610.46 microseconds** for native readout plus update and **662.23 microseconds** for the integer ratio plus update, about **8.48% additional time**. It includes per-step allocations but excludes prompt ingestion and setup. It is not whole-system, energy, zero-allocation or quality-matched incumbent evidence.

The new `hamilton_transport` component implements exact signed Q8 actions on four-coordinate vectors, with inverse, order, norm, overflow and relative-frame checks. For a pure-axis skew generator K, `K^2=-I`; `H=iK` is Hermitian and `exp(-i*pi*H/2)=K`. The implemented finite action executes sign changes and coordinate selection, not a complex exponential. Applying the same orthogonal action to every query and key preserves every dot product and cannot itself improve attention selection.

Hamilton's quaternion algebra, a Coxeter H4 root system, and an energy-based Hamiltonian dynamics model are different objects. H4 roots have a quaternion realization via the binary icosahedral structure; H4 is not simply another name for the quaternion algebra. The hint therefore motivates learned relative/content-conditioned transport, not an unsupported claim that fixed rotation solves attention. See [the mechanism interpretation and primary references](hamilton-vector-interpretation-2026-09-23.md).

Neither the warm-start nor the ratio language result uses Q8 as its causal prediction mechanism. Their gains are not attributed to Hamiltonian, H4, prime or zeta geometry. The terminal geometric objective is preserved; this local prior and improved native learner are controlled components and comparators toward it, not a replacement project goal.

## Verification, storage and continuation

Focused executed coverage includes seven integer-prior/ratio tests, six Hamilton-vector tests, thirteen transferable-lexical tests including the new exact warm-start test, and twenty-three runner tests. The legacy default serving path is unchanged. Source, corpus, candidate serialization, integer/float parity, fresh-process warm replay and transferred-bundle replay were checked. This is not a blanket repository-suite, complete binary instruction, full scoped-session or energy qualification.

The owner checkout's pre-existing work is preserved. Debug intermediates and audited redundant checkouts were reclaimed only after unique uncommitted files were archived and hash-verified; commits were retained under archive refs. Physical free space rose to **55.10 GB immediately after cleanup**. Detailed recovery metadata remains in the investigation's `worktree-archives` directory. The initial 2 GiB compilation-memory projection was exceeded and disclosed; later compilation used a 4 GiB allowance while model experiments retained the 2 GiB allowance. Resource receipts distinguish observations, projections and charges.

Retained candidates are under `.uor-models/principal-continuation-2026-09-23/`, in a newly claimed, sealed and native-verified root:

- `warm-decay-candidate.tlx`: SHA-256 `69e8b88b41bb09d9149be1ac7e83e5e50db2974f470dce05405a55cc1f0bfb7e`.
- `prose-ratio-candidate.prc`: integer ratio on the original prose artifact.
- `grounded-warm-ratio-candidate.prc`: unchanged frozen correction transferred to the warm candidate.
- `selection.json` and `registry.json`: frozen coefficient choice and exact source-report links.

These are research candidates; no global serving default, Studio/API model or full capability bundle is silently replaced. The complete training optimizer state is not available in a `.tlx` file, so future continued training must distinguish another fresh warm start from exact optimizer continuation.

Full immutable reports, experimental inputs, executables and original-source refs remain in `/Users/casey.allard/uor-r4-investigations/principal-20260923T214312`. The local archived pre-delivery source branch preserves the exact experiment commits. The subsequent rebase onto owner-direction PR #1370 was checked to leave the complete Rust code diff byte-identical. The five-day review covered 85 main-branch messages at start and the additional owner-direction commit; critical source was traced rather than claiming every historical patch line was reviewed.

Read [the principal handoff](principal-handoff-2026-09-23.md) before repeating work. New final-source evaluations require new sources: these four books are now exposed regression data. Do not claim the successful warm recipe, the old raw blend, and the marginal-relative family are the same experiment or the same causal mechanism.
