# Geometric attention composition: bank-only answer learning

**Status: source, checks and both fixed 640-update fits completed with exit code 0.** Both independently reloaded native banks agree with their source counterparts on all 512 evaluated answers. They learn substantially from their fixed initialization. The comparison with the retained parents is mixed: seed 1 gains greedy accuracy with some lost rows, seed 2 retains perfect accuracy on these panels, and both have worse mean answer cross-entropy than their parents. Preserve the learned banks; this result does not promote a universal replacement, establish geometric superiority or authorize another bank dose.

This task learns the attention value-to-residual operator introduced by [the composition implementation](geometric-read-composition-2026-10-02.md). It stays inside geometric attention. It changes neither the GroundedSession consumer nor the final vocabulary head. The preceding zero-update construction showed a working answer-gradient path and native operator, but the fixed, untrained bank produced 7/5 correct answers for seed 1 and 0/4 for seed 2 on 32 original/32 stress examples. The retained NoRead parents with their learned `read.out` maps scored 24/22 and 32/32 on those same rows. That is a reason to train the new bank, not a negative result about a trained geometric operator.

The prospective [driver and two-batch check card](https://github.com/UOR-Foundation/uor-r4/issues/1512#issuecomment-5953002174) preceded source execution. The separate [fixed fit card](https://github.com/UOR-Foundation/uor-r4/issues/1512#issuecomment-5953221861) followed the measured costs. The source check card did not itself authorize optimizer updates.

## Mechanism and fixed learning scope

Each of the two heads has four input quaternion lanes. For each of eight output lanes and each input lane, two terms apply `q/4 * left * value * inverse(right)`. The bank consumes the actual two typed H4/radius atoms produced for every occurrence. It preserves signed roots, radius, validity and occurrence identity. Its native path forms 32 coordinates per head, preserves that head's score/age/NoRead-inclusive denominator, and sums the two normalized head outputs before entering the remaining floating model tail. It bypasses the donor `read.out` map.

Only the following three parameter families enter AdamW and its clipping norm:

| Family | Offline shape | Scalars |
| --- | --- | ---: |
| Gain shadows | `[128]` | 128 |
| Left action logits | `[128, 120]` | 15,360 |
| Right action logits | `[128, 120]` | 15,360 |
| Total | Three families | 30,848 |

These are offline learning variables. The deployed bank contains 256 byte-sized categorical root selectors and 64 packed gain bytes: 320 bytes before metadata and shared geometry. Selectors are categorical H4 identities, not four-bit linear coefficients. The dimensionless gain coefficients are signed four-bit values from -7 through 7, scaled by the fixed factor 1/4. Wider native score tables in other components are not thereby qualified as four-bit learned weights.

Forward uses the already admitted hard root choices and integer composition. Backward uses the declared conditional 120-way full-value surrogate for each selector, holding its counterpart at its hard choice. It includes both atoms, radius and the gain. Gain shadows use the existing identity STE. This is a biased training bridge; it is not differentiation of hard argmax or the final quarter rounding. The driver does not alter the operator, codec, surrogate, gain scale, number of terms or causal support.

Every context, event, span, potential, value, age and NoRead producer is frozen. All base-model, residual/trunk and vocabulary-output parameters are also frozen. Autodiff may calculate gradients for some frozen tail variables; those variables are excluded from both the optimizer and the selected clipping norm. Before/after artifact-file and base-variable hashes check that they stayed fixed.

## Data, objective and dose

The source-bound parents contain 1,280 completed context/value updates and 640 separate NoRead updates. The new composition bank starts from the retained answer-independent asymmetric initialization, with no donor projection or answer-based choice of actions.

The driver reads the scalar-final RNG as an exact Rust `u64` and checks its checkpoint and final source bytes. Seed 1 resumes `9347350181718662555`; seed 2 resumes `10697677911788535647`. The unchanged grammar continues at absolute sampler step `1920 + j`. These indices identify the shared data stream; they do not mean that every component was trained for 1,920 updates.

The admitted fit is 640 new composition updates per seed, batch size 8, alternating two and four facts with independent source/query noise draws. Width is 32 with two heads. Maximum model context and causal access are 128 positions. Actual episode lengths and padded batch lengths are recorded separately. Padding follows the answer; exactly one unit loss weight per episode selects its actual final answer marker. Consequently the primary objective is mean answer cross-entropy with denominator 8. There is no source-selection, packet-imitation, event or scalar auxiliary loss.

Optimizer settings are fixed: AdamW learning rate 0.003, beta values 0.9 and 0.95, epsilon `1e-8`, zero weight decay, and global norm clipping at 1 over the three bank families only. After each update, gain shadows are projected to `[-1.75, 1.75]`. Selector changes, gain-code changes, clipping, saturation and family gradients are reported. None is required to cross an arbitrary threshold for a run to count as executed. There is no learning-rate, gain-scale, term-count, selector or alphabet sweep.

## Executed source and actual-parent checks

The release build completed with exit code 0. Both focused driver tests passed:

- `composition_fit_sampler_mask_and_exact_u64_continuation`
- `composition_fit_selected_global_clip_excludes_frozen_variables`

The first checks exact RNG progression and answer masks across the two fact counts. The second supplies a deliberately large gradient to a separate frozen variable, then verifies that clipping and optimizer updates apply only to the bank; it also checks gain projection. Earlier signed-group, wide arithmetic, cancellation, native bypass, causal-prefix and artifact tests are retained from the composition implementation rather than repeated as a new campaign.

Both actual-parent check attempts completed with exit code 0. Each performed two successive B8 forward/backward calculations at absolute steps 1920 and 1921, **zero optimizer updates**, and 32 original plus 32 stress source/native comparisons. The two checks covered 32 diagnostic training episodes in total. Their original bank parameter hashes and returned next-training RNG values were unchanged. The locally advanced diagnostic RNG and both input batches are separately retained.

| Seed | Facts in batch | Padded time | Actual positions | Padded positions | Calculation seconds | Mean answer CE |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 2 | 42 | 274 | 336 | 0.209978 | 2.398585 |
| 1 | 4 | 66 | 472 | 528 | 0.340134 | 2.569288 |
| 2 | 2 | 38 | 256 | 304 | 0.205335 | 4.633334 |
| 2 | 4 | 70 | 476 | 560 | 0.362114 | 4.700758 |

All four diagnostic batches produced finite, nonzero gradients at all 128 gain shadows and all 15,360 entries in each selector family. This establishes the exercised answer-credit path on these batches; positive entries are an observation, not a general learning or quality guarantee.

The check panels reproduce the admitted construction scores: seed 1 has 7/5 correct and seed 2 has 0/4 correct, each out of 32 per panel. Source/native choices agree on all 128 checked evaluation rows across the two seeds. Actual K2 packets, raw NoRead scores, occurrence weights, denominators and score maxima match the frozen parent on every checked prefix. The maximum source/native difference across all actual-position logits is `0.0002684593200683594`. This combines the declared real-softmax/F32 versus integer-LUT/Q16 reduction differences; it is not an isolated transport-bank error bound.

The test command took 2.149 seconds including its incremental compilation, and the example build took 3.247 seconds. Actual check process wall times were 3.678 and 3.160 seconds. Their measured peak RSS values were 217,989,120 and 218,693,632 bytes. These are small construction/check costs, not full-serving energy measurements or guarantees for longer workloads.

## Artifact and comparison contract

The driver verifies the sealed query, NoRead-fit and composition-construction roots. It pins the accepted scalar report per seed, joins its query-parent identity, checks the exact saved RNG and verifies equality between checkpoint and final scalar-source files. Existing loaders bind the tokenizer, geometric algebra, numerical policy and actual producer dependencies. The initial bank must match the fixed constructor's exact parameter bits and the independently loaded construction metadata.

Before fitting, the driver records the full 128 original and 128 stress rows for the initial bank and replays the actual NoRead parent. It compares parent predictions/logits to the retained rows and compares every available construction-prefix row. Historical NoRead records, including their nested initial-zero scalar comparisons, remain retained. These are open development panels; they are not a new unseen language qualification set.

After fitting or a partial stop, the source bank is saved and independently reloaded. The native bank is then compiled, saved and independently reloaded against the actual frozen dependencies. Final source/native evaluation records every prediction and answer-logit vector, all-position numerical differences, and query head traces. It checks all actual packets and score/NoRead normalization inputs against the unchanged parent. Gains and losses are reported separately relative to the initial geometric bank and the retained NoRead parent; aggregate improvement alone is insufficient to describe which rows changed.

Checkpoint directories are claimed exclusively at step zero, every 80 completed updates or 900 seconds at a step boundary, and at the final partial step when needed. They retain source weights and the exact next RNG. **AdamW moments are not serialized.** These are recoverable weights and data-stream state, not a claim of exact optimizer continuation. A failed step retains its input, failure record and preceding valid checkpoint. A partial run keeps its completed rows and receives no quality verdict.

## Completed fixed-dose results

Seed 1 completed all 640 bank updates and 5,120 new training episodes. It processed 245,708 actual token positions and 285,632 padded positions; actual episode lengths were 24–78, with maximum batch time 78 and model context/access still capped at 128. The process exited 0 after 193.721 seconds and measured 209,453,056 bytes peak RSS. Its fitting phase lasted 188.212 seconds, including 186.918 seconds of forward/backward/update calculation. These measured costs belong to this training task, not to an inference-energy claim.

| Seed 1 panel | Initial bank correct | Frozen NoRead parent correct | Fitted source/native correct | Gains / losses versus parent |
| --- | ---: | ---: | ---: | ---: |
| Original, 128 rows | 28 | 98 | 102 / 102 | 7 / 3 |
| Stress, 128 rows | 23 | 104 | 109 / 109 | 9 / 4 |

Relative to the initial bank, the original panel gains 74 rows and loses none; the stress panel gains 88 and loses 2. Source/native predictions agree on all 256 rows. Every checked row retains the actual upstream packets, raw null scores, occurrence weights, denominator and maximum. The largest source/native difference across all actual-position logits is `0.0007262229919433594`, on the stress panel. Source and native numerical policies remain distinct despite agreement in greedy predictions.

Mean answer cross-entropy derived from the retained answer logits is mixed relative to the parent:

| Seed 1 panel | Initial bank CE | Frozen parent CE | Fitted source CE | Fitted native CE |
| --- | ---: | ---: | ---: | ---: |
| Original | 2.731975 | 0.938979 | 1.099175 | 1.099175 |
| Stress | 2.715729 | 0.894337 | 1.091402 | 1.091402 |

Thus the learned bank materially improves over its initialization and has higher greedy accuracy than the retained parent on these panels, while its mean answer CE remains worse than the parent's. This is a useful learned component with a mixed comparison, not a drop-in promotion. The parent did not receive a matched additional 640-update extension in this experiment, so the result does not establish geometric superiority or a controlled advantage per training dose.

The history records 255 cumulative left-choice changes, 346 cumulative right-choice changes and 1,117 cumulative gain-code flips across updates; these are event counts, not counts of distinct final parameters. Selected global clipping applied on 131 updates and no gain required range projection. The exact next RNG is `461503785751815875`. Saved source, compiled bank, final checkpoint, complete rows and execution identity remain under `fit-1-s1/` and `fit-1-s1-execution.json`; the saved-logit derivation is `saved-logit-ce-s1.json`. The second seed is measured independently below.

Seed 2 also completed 640 updates and 5,120 new episodes, with 245,396 actual positions and 286,288 padded positions. Actual episode lengths were 24–80. The process exited 0 after 184.426 seconds and measured 210,092,032 bytes peak RSS. Its fitting phase lasted 178.702 seconds, including 177.424 seconds of forward/backward/update calculation.

| Seed 2 panel | Initial bank correct | Frozen NoRead parent correct | Fitted source/native correct | Gains / losses versus parent |
| --- | ---: | ---: | ---: | ---: |
| Original, 128 rows | 0 | 128 | 128 / 128 | 0 / 0 |
| Stress, 128 rows | 9 | 128 | 128 / 128 | 0 / 0 |

The original panel gains all 128 rows relative to initialization; stress gains 119, with no losses from initialization on either panel. Mean answer CE falls from 3.986997 to 0.470688 on original and from 4.242985 to 0.458134 on stress. The retained parent remains lower at 0.286856 and 0.281316. Perfect greedy accuracy on this finite grammar therefore does not imply retention of the parent's full answer distribution.

Seed 2 records 398 cumulative left-choice changes, 279 cumulative right-choice changes and 1,310 cumulative gain-code flips. Clipping applied on 183 updates and no gain required range projection. The exact next RNG is `1220542080958524937`. Its maximum all-position source/native logit difference is `0.00032579898834228516`. Full records are retained under `fit-1-s2/` and `fit-1-s2-execution.json`.

Across both completed fits, source/native answers match 512/512, with unchanged actual upstream packets, raw NoRead scores and head-specific normalization inputs. Combined fitted greedy accuracy is 467/512, versus 60/512 at initialization and 458/512 for the retained parents. The combined count does not erase seed 1's lost parent rows or the worse mean answer CE on all four panels. Actual fit process wall time totals 378.147 seconds; this excludes separate source/check/delivery work and must not replace the cumulative ledger charge.

## Capacity and interpretation

Two scaled proper four-dimensional rotations do not span arbitrary real 4×4 maps. If `M = aR + bS`, then `MᵀM = (a² + b²)I + ab(U + Uᵀ)` with `U = RᵀS`. Its singular values occur in equal pairs, so even continuous choices cannot reproduce the generic block `diag(1, 2, 3, 4)`. Finite H4 actions and four-bit gains impose additional restrictions.

This capacity limit prevents interpreting an imperfect donor reconstruction as evidence of a broken training gradient. Conversely, a functioning gradient and native compiler do not establish that this restricted bank will learn the current task well. The fixed dose can inform task capability and credit progression without pretending to settle the entire architecture. A mixed or negative result does not automatically retire geometric composition or authorize another identical run.

The broader offline conversion option remains available as a separate design: 16 left/right axis-quaternion actions span real 4×4 maps before coefficient quantization, with additional parameter-access, range and error costs. That more expressive bridge is not silently substituted into this experiment. There is no perfect donor-reconstruction or perfect answer-score admission gate.

## Cost admission and completed disposition

The actual worst diagnostic batch cost, approximately 0.362115 seconds, projects to 232 seconds for 640 updates. The prospective card doubled that estimate for optimizer work, longer draws and variance, then reserved another 60 seconds for preparation, full pre/post panels, compilation and checkpoints: approximately 524 seconds per seed. The two workers are sequential, each using two threads and a 4 GB RAM allowance, within the shared eight-thread/11 GB ceiling.

Each fit receives an explicit internal limit of 840 seconds and an external worker limit of 900 seconds. The source stops admitting new training batches 300 seconds before the internal limit, at elapsed 540 seconds, preserving an internal finalization/evaluation reserve. The additional 60 seconds is the external stop margin. One in-flight bounded operation can cross a cooperative threshold; the external worker remains the final wall-time bound. The format's 7,200-second parser ceiling does not grant an execution allowance.

The complete source/review/delivery allowance is 60 minutes including the two workers, with at most 30 minutes of worker time. Research/temporary retention is bounded at 256 MiB, with a separate 256 MiB archive allowance; the existing cache ceiling is 14 GiB and the physical reserve is 128 MiB. The driver checkpoint charged cumulative work through `978081958 / 1130000000` milliseconds before the fixed-fit phase. Later fit and delivery charges remain root-owned. Preparation time includes prefit evaluation, so those nested timings must not be added twice.

**Disposition: both fixed doses are complete; no further bank dose is adopted.** Preserve both learned sources, native banks, initial banks, prior parents and all changed rows. The selected next attention boundary is the K2 value producer: constrain its token, own-state, neighbor-state, prior-span and validity coefficients to an explicit packed four-bit codec, then regenerate the signed-root factor tables from those coefficients and the pinned geometric basis. Freeze context, events, address potentials, NoRead, the learned composition bank and the remaining tail. Preserve all 120 signed roots, both value atoms, explicit zero/presence and the existing radius alphabet. The first measurement is independently loaded projection through the actual reader, before admitting an answer-learning dose. Exact packet retention is not a quality gate; record changed packets, numerical effects and answer rows. The implementation must distinguish categorical root IDs and wider precision derived from fixed geometry from free wide learned coefficients, preserve ordinary answer credit, and avoid hiding a dense wide projection behind a table. This next boundary needs its own source-backed task and cost admission; the present fit does not authorize a scale/term sweep or a final vocabulary-head detour.

The unfinished residual/trunk nonlinear computation, normalization and vocabulary output remain floating. Free wide learned coefficients in other native producers and address potentials remain an explicit qualification gap. This experiment makes no claim of general prose, coding/reasoning, geometric superiority, complete native attention, whole-model serving or laptop energy savings.

## Retained source and receipts

Retained research root: `~/uor-r4-local/workspace/research/geometric-read-composition-fit-20261002/`. It contains the work cards, independent binding/mathematics reviews, execution receipts, logs, sealed check attempts and both completed fit attempts. The executed example SHA-256 is `153de9d37b123155882b50e40908e79c2eef5a0906fb130e98d0c921b8b0966c`.

| Source | SHA-256 |
| --- | --- |
| [Composition fit driver](../../crates/uor-r4-training/examples/attention-geometric-composition-fit.rs) | `b6c65eaf27a3615e21234c1b1ec9c8356df8e7dbf9464bc659bbc7bcae8f2936` |
| [Private fit helper](../../crates/uor-r4-training/examples/attention-geometric-composition-fit/fit.rs) | `0d350cbe095803640296d230d5e8e69b07be33e02c91bb95130697eb432b7a84` |
| [Retained grammar helper](../../crates/uor-r4-training/examples/attention-geometric-value-learned/data.rs) | `f8e3088878f8d82550e5a00491d77a0f070a67ca129d03fda2c912b2f90d1682` |

`driver-tests-1-execution.json`, `driver-build-1-execution.json`, `check-1-s1-execution.json` and `check-1-s2-execution.json` report exit code 0 and unchanged source identities. Check reports, histories and raw row records are under `check-1-s1/` and `check-1-s2/`. Both `fit-1-s1-execution.json` and `fit-1-s2-execution.json` report exit code 0, the same executed example SHA-256 and unchanged source identities. Report sealing and verification belong to the executed Rust driver; SHA-256 source/executable identities above are distinct from its report-manifest integrity scheme. Independent reviews establish their stated source scope and are not substituted for these executed receipts.
