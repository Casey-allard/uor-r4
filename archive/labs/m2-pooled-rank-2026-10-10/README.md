# Native pooled-token ranking clears the development threshold

**KEEP: 177/512 → 437/512 complete replies**, with 260 gains, zero losses and all 177
parent successes retained on the unchanged frozen development panel. This clears
M2's 256/512 development threshold. The subsequent fresh-draw ≥40% acceptance
remains **NOT_RUN**; M2 is not complete. All 512 development rows were exposed
during training. 75 complete replies still fail.

**Line: cross-state continuation · count 0/3 · headline 177/512 → 437/512.**
No preparation PR was used. The protected constructor/attribution/solver line,
ordinary 24/96 recipe and rejected phase-balanced continuation remain closed.

## Registered intervention and unchanged model

The [pre-registration](https://github.com/UOR-Foundation/uor-r4/issues/2030#issuecomment-6098141610)
tests strongest-wrong native pooled-token ranking from saved177, replacing
absolute target CE while preserving equal-episode bottleneck aggregation.
For canonical target `y` at each teacher-prefix position, build the whole
label-free native action pool first, then define:

```
r = argmax_{v != y} M[v]        (smallest token ID breaks equal-mass ties)
d = log(M[y] / M[r])
ell = softplus(-d) = log(1 + M[r] / M[y])
J[e] = log((1 / T[e]) * sum_t exp(ell[e,t]))
J[batch] = (1 / B) * sum_e J[e]
```

Margin is fixed at **0**, temperature at 1. Rival selection uses exact native
u64 token masses after the ordinary common clip, including every Generate and
physical Copy alias. It refreshes at every current forward and is held fixed
for that backward pass. It does not select the largest physical atom, use raw
unclipped diagnostic masses or introduce a serving gate. Missing targets or
competitors, invalid pools and nonpositive masses reject without a probability
floor.

The existing `vocabulary_log_mass_margin_with_credit` API anchors the forward
log ratio to integer pooled masses and supplies the retained RawIdentity/Q4
straight-through adjoints. Stable softplus computes
`max(z,0)+log1p(exp(-abs(z)))`, with `z=-d`, and composes its first derivative
through a detached stable sigmoid. The existing streaming episode accumulator
then weights token gradients by `softmax(ell)` and averages episodes, including
EOS, with no phase weights or extra 1/T gradient factor. One token graph remains
live. This is the declared first-order surrogate, not the derivative of integer
rounding or the discrete rival selection; native-table and relaxed exponential
probabilities differ, so unrelated-action gradient terms need not cancel.

Only the existing 115,200 cross-state Q4 coefficients learn. Source48/Generate64,
Context, Potential, prototypes, bridge, Cue, Prefix, tokenizer/exponent table,
native scorer, clipping, shift22, decoder and grader remain fixed. No serving or
library scorer source changed. Reusing the existing margin primitive does not
activate historical protected construction, constraint or solver runners.

The fixed schedule remains 256 updates, batch 8, seed 1001, learning rate .03,
AdamW betas .9/.999, epsilon 1e-8, weight decay 0, global clip 1 and quarter-master
clamp [-1.75,1.75]. Four repetitions of the same full512 order give 2,048 episode
and 26,656 answer/EOS draws over 6,664 distinct cached positions. Targets are at
most 32 including EOS; teacher-prefix context is at most 128. Recovery
checkpoints occur every 32 updates; local 256 is the sole selected endpoint.
There is no rate, seed, dose, margin or checkpoint sweep. Training and evaluation
use the same exposed development panel; fresh qualification is not this run.

## Strict saved177 restart and evidence labels

The new opt-in `cross_state_pooled_rank` flag requires `cross_state_bottleneck`
and the exact saved177 profile; unchanged CE continuation from saved177 rejects.
Admission pins report
`13fa577beacbbe3fd0544095c8f4a6e8555458721325f000b8a419d0a90a8e48`,
manifest `10db47cb5e29d562df485e8364fe60bf503b5b4bd40f7b4797f40a4c65e30156`,
field `dfb945d567ce25f5c200967ce1f9e5e31740f9cfaa8f6a1a6d6dd11d156a2e13`,
and fractional F32 master
`0c7dec5f627b8c9c44557922a4d17d30846796381d26a055edc84551f8149acc`.
The master has shape [8,14400] and occupies 460,800 bytes. Parent local 256 and
lineage832, saved175 ancestry, objective and checkpoint policies must authenticate.
Exact fractional bits must restore and re-export to the native field; Adam
moments reset explicitly. This run is parameter continuation from lineage
**832 to 1088**, not optimizer-state continuation.

The existing separate zero-field cache, parent/output non-overlap checks and
independent native reload remain. Before any update, all 512 parent row IDs,
generated sequences and completion flags must reproduce exactly, including177
complete replies. The endpoint must pass saved/reloaded early177 evaluation and
then full512 regardless of losses. The executed baseline reproduced all 512 outputs exactly; early177 retained177,
and the full512 endpoint completed437. The independent reader agrees.

Reports/checkpoints identify the ranking flag and policy. Updates report
`objective_loss` and `pooled_rank_logmeanexp_loss` as J, with
`pooled_rank_target_positions` counting all scored targets including EOS.
`phase_balanced_loss` and the former CE `bottleneck_logmeanexp_loss` are null.
Evaluation retains ordinary equal-episode mean token CE and the unchanged
complete own-prefix grader; neither evaluator CE nor rank J alone is the KEEP
criterion. The prospective bar is full512 complete >177 with integrity passing,
reporting exact gains/losses and all declared regressions.

Rank loss staging accounts for `16N+4G+16` explicit CUDA upload bytes per
position: Generate indices, native raw and hard score vectors, two alias masks,
two probability anchors, the native margin and softplus scalar. N is the physical
action count and G the admitted legal token count. Scalar downloads and kernel
argument immediates are excluded and named separately. The old CE counter
retains its historical narrower scope.

## Source validation and preserved check failures

Production source is `5db7023bb` (record full source and executable identities
from final producer receipts). **All 25 focused tests passed on check attempt3**:
the 19 retained tests plus six added tests covering:

- Actual pooled rival versus strongest physical atom, smallest-ID ties,
  Generate-only competition, EOS and refreshed rival selection.
- Missing targets/competitors, invalid pooled mass and corrupted alias subtotals,
  with no floor substitution.
- Stable softplus at large positive/negative margins, exact native forward
  anchoring and non-grid/duplicate-alias RawIdentity gradients against a CE
  Jacobian reference.
- Unchanged streaming composition against a directly differentiated reference
  with unequal episode lengths and EOS.
- Strict saved177 flags/profile, exact baseline, net-gain bar and lineage 1088.
- Actual published saved175 ancestry and saved177 fractional-master identity,
  rejecting altered policies, provenance and masters.

Two failed check attempts are preserved separately from model evidence. Attempt1
failed compilation with three E0308 string-borrow mismatches in the new test
fixture's error adapters; the repair only borrows the temporary strings.
Attempt2 compiled and passed24/25 tests; the artificial d=1000 softplus case
expected a gradient entry even though exp(-1000) and its affine multiplier were
exactly zero. Candle0.9.2 explicitly prunes zero-multiplier affine dependencies.
The fixture now accepts absent or exactly-zero credit only for that exact-zero
case, asserts zero loss, and retains strict missing-gradient rejection for every
nonzero case and native pooled-gradient reference. Production arithmetic was
unchanged by both repairs. No model execution occurred in either failed check
attempt; they are setup/validation failures, not model negatives.

## Saved endpoint and decision

The producer exported/reloaded the sole local 256 checkpoint at lineage 1088,
scored under its own generated prefix and the unchanged label-membership grader.
The independent saved reader returns **PASS_SAVED_EVIDENCE**. It verifies the
exact 512 baseline, pinned fractional masters, frozen cache provenance, fixed
schedule, all 26,656 rank target positions and 11 prior saved comparisons.

| Measure | Saved177 | Ranked endpoint |
| --- | ---: | ---: |
| Complete own-prefix replies | 177 | 437 |
| Correct entry token | 298 | 507 |
| EOS-ending outputs | 183 | 437 |
| Complete source-swap pairs | 32 | 186 |
| Teacher-prefix all-token correctness | 177 | 437 |
| Teacher-prefix correct tokens /6,664 | 3304 | 6491 |
| Equal-episode teacher-prefix CE | 3.2068708694739447 | 2.182831670795063 |

| Memory stratum | Cases | Before | After |
| --- | ---: | ---: | ---: |
| length2 | 128 | 56 | 116 |
| length4 | 128 | 57 | 114 |
| length8 | 128 | 11 | 90 |
| update | 64 | 21 | 57 |
| reassert | 64 | 32 | 60 |

All 335 previously failing outputs change; 260 become complete and 75 still fail.
The native field changes 34,773 of 115,200 legal coefficients, with 112,020
nonzero codes, range [-7,7] and valid padding. Every success from each prior
comparison remains successful, including all original 8:

| Pinned prior record | Prior complete | Gains | Losses | Retained |
| --- | ---: | ---: | ---: | ---: |
| 2141 | 2 | 435 | 0 | 2 |
| 2143 | 2 | 435 | 0 | 2 |
| 2148 | 4 | 433 | 0 | 4 |
| 2149 | 0 | 437 | 0 | 0 |
| rejected139 | 139 | 298 | 0 | 139 |
| accepted145 | 145 | 292 | 0 | 145 |
| accepted175 | 175 | 262 | 0 | 175 |
| accepted177 | 177 | 260 | 0 | 177 |
| accepted22 | 22 | 415 | 0 | 22 |
| original 8 | 8 | 429 | 0 | 8 |
| old_u64 | 1 | 436 | 0 | 1 |

These are exact saved-row comparisons, not fresh reruns or unions of outputs.
The [reader JSON](reviewer-rank177.json) retains all IDs. Entry correctness rises
298→507, complete source-swap pairs 32→186, and length8 completions 11→90/128.
The headline improves on this exposed panel; generalization remains unmeasured.

The registered complete >177 bar therefore gives **KEEP 437**, count 0/3. The next
cycle after protected delivery and cleanup is to freeze the existing fresh-draw
criteria, generate the fresh panel and score this saved model without additional
training. The development threshold has been passed; another training cycle is
not the next acceptance dependency.

**Before opening the PR: if this came out the other way, would my next step
differ? Yes.** At ≤177 the model would be rejected, saved177 retained, count 1/3,
and this unchanged rank dose/rate/seed recipe stopped. The observed 437 instead
retains this artifact and advances to fresh held-out qualification.

This result does not establish that all 512 cases are attainable, general useful
conversation, a predictive geometric advantage, unique factual-carrier necessity
or full-path energy savings. Fresh criteria and data have not yet been generated
or scored in this cycle. M3 is unchanged; the standing M2 goal continues.

## Verification, identities and cost

Producer source is `5db7023bbc069ab947ae90fdcbec1d10dc3faefb`, based on
`0d4ade129571767a7348808916041f08662469ff`. Only the fitter example and its
cross-state completion helper change; serving/kernel/evaluator source is
unchanged. Producer checks are 25 focused tests, optimized build, formatting/diff
and independent exact-head source review PASS. Stock bootstrap took 190 s,
37 tests passed and three were ignored. The two failed fixture-only check
attempts above are retained with their source bundles and logs.

| Identity | SHA-256 |
| --- | --- |
| Producer executable | `7097976b8c1c16779ca8870ae2d104fec977e375087f987cc35b0c82e1d7b799` |
| Configuration | `84b60e273b548cfa45fd0a7f8748b61a2aaad1128683943cf6689403d190eb3f` |
| Endpoint report | `787f1a694dd894f8dcf636e158b2470fd88a824f3dd73535749bc4f36f05a165` |
| Endpoint manifest | `2f630ad56a6abd09271681fcb514d22caefac8af918321224bc173446c20a84c` |
| Endpoint native field | `de4a3234d6e92f4b12a657cc195425687bc67c243d9eded9a1df0797b9942a61` |
| Reader | `372cb12eb3268b5aea8b802065d05d02b5c36b6ee91b3ee10178751dd4270cd3` |
| Reader output | `5048673d2f67e4e6b1dc05901f0746fa546877fc116c4dc13864c24c1d81b83c` |
| Restorable producer patch | `d4e76a8e4ad0336f57df869f11d45085d0b12a45b6dedc3b2be98442d1f746f7` |

The [artifact receipts](artifact-receipts.json) bind parent/master, data,
upstream model and native policies. Full BLAKE3 seal verification is producer
evidence. The independent reader checks inventory/sizes, saved hashes,
IDs/actual-prefix chains/EOS/frozen membership and Q4 payloads; it does not
regenerate model scores, tokenizer decoding, gradients or training rival IDs.
Current-rival selection is source/test evidence. Its rank staging check accounts
for 2,191,728,640 explicit host-upload bytes over all positions; scalar downloads
and kernel immediates are excluded. The prior CE counter had narrower scope,
so these counters are not a matched throughput comparison.

All saved objective scalars are finite. First/last batch J values
0.0709000916451139 and 0.5878516024565181 belong to different scheduled batches,
not a matched learning curve. Evaluation CE is the unchanged mean token CE,
distinct from rank J. Neither scalar substitutes for complete replies.

The model process started 2026-10-10T14:07:15.247743Z and exited 0 after
757.6145658418536s, with peak child RSS 790,476 KiB
and sampled GPU peak 838 MiB over 77 samples. Producer elapsed time was
752.458554855s: cache49.793682885s,
fit excluding checkpoints233.43904865599998s,
checkpoints4.780493611s and
evaluation458.02048901300003s. These are process measurements,
not full-cycle cost or serving efficiency.

The pre-registered projection covers 90 min/full cycle, one 5090 at $1.19/h
(about $1.785 for 1.5 h), at most 6 GiB RAM/GPU, 16 GiB remote temporary storage,
512 MiB compressed retained storage plus 128 MiB margin, and 2 GiB local source.
No local model/build/archive was staged. The cumulative ledger extension was
recorded before compute, from limit 1,499,400,028 to 1,503,000,028ms; cumulative
before this cycle was 1,495,271,627ms. Complete-cycle charge, final exact-head
checks, merge/tree verification, branch/worktree cleanup, hygiene and poddown
remain pending and will be closed on [#2030](https://github.com/UOR-Foundation/uor-r4/issues/2030).

The preservation package is 156,825,600 bytes, MD5
`49985949980d98d05f9c0274e5529f63`. Full iCloud stream and standard six-column index readbacks passed;
see [preservation.json](preservation.json). The package
contains this run/runtime, source bundle/patch, both failed check attempts,
saved177 and nested prior accepted/negative models and frozen panel evidence.
Restore with `cloud-store fetch codex-m2-pooled-rank-20261010 <absolute-owned-destination>`
and follow [PACKAGE.json](PACKAGE.json). Model KEEP, artifact preservation and
protected delivery are separate states.
