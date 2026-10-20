# Bottleneck continuation retains a small complete-reply gain

**KEEP: 175/512 → 177/512 complete replies**, with nine gains, seven losses and
168 retained successes on the unchanged frozen panel. The net gain is two;
entry correctness, teacher-prefix token correctness, complete source-swap pairs
and reassert completions regress. The registered developmental bar is met, but
335 complete replies still fail, length8 reaches only 11/128, and M2's 256/512
threshold and subsequent fresh-draw 40% qualification remain unmet. Fresh
qualification is **NOT_RUN**.

**Line: cross-state continuation · count 0/3 · headline 175/512 → 177/512.**
No preparation PR was used. The stopped phase-balanced dose/rate/seed recipe,
ordinary 24/96 recipe and protected constructor/attribution/solver line remain
closed. M3 is unchanged.

## Registered question and unchanged learning method

The [pre-registration](https://github.com/UOR-Foundation/uor-r4/issues/2030#issuecomment-6097748714)
([retained text](preregister.md)) asks whether four additional passes of the
successful [episode-bottleneck objective](../m2-bottleneck-2026-10-10/README.md)
from saved175 improve complete replies. The parent objective intervention had
gained 34 and lost four; this cycle tests parameter continuation with fresh Adam
moments, preserving the objective and native scorer.

For unweighted native-forward alias token CE `ell[e,t]`, including EOS:

```
J[e] = log((1 / T[e]) * sum_t exp(ell[e,t]))
J[batch] = (1 / B) * sum_e J[e]
dJ[batch] = (1 / B) * sum_e sum_t softmax_t(ell[e,*]) * d ell[e,t]
```

Temperature remains 1, with no phase weighting or extra 1/T gradient factor.
Stable detached streaming composes this gradient with the existing raw-identity
alias and Q4 straight-through adjoints, using one live token graph. Native hard
probability supplies the scalar forward loss. This is the declared surrogate,
not an exact derivative of integer rounding or greedy generation. There is no
objective, gradient, scorer, evaluator or schedule change in this continuation.

Only the existing 115,200 cross-state Q4 coefficients learn. Directed signed H4
relatives of the factual post-bridge state and independently replayed
query/prior-prefix state determine a shared token energy, applied to Generate
and every physical Copy alias before the existing common clip and pool.
Source48/Generate64, Context, Potential, Generate prototypes/coefficients,
bridge, Cue, Prefix, tokenizer and exponent table remain frozen. The native
kernel, shift22 policy and 65,536-byte padded payload are unchanged; existing
dense vocabulary access remains.

The [configuration](model-config.json) fixes 256 updates, batch 8, seed 1001,
learning rate 0.03, AdamW betas 0.9/0.999, epsilon 1e-8, weight decay 0, global
norm clip 1 and quarter-master clamp [-1.75,1.75]. Four repetitions of the same
shuffled 512-row order give 2,048 episode draws and 26,656 answer/EOS draws over
6,664 cached positions. Targets are at most 32 including EOS; teacher-prefix
context is at most 128. Local 0→256 corresponds to lineage **576→832**.
Checkpoints every 32 updates serve recovery; 256 is the sole selected endpoint,
with no adaptive selection or rate, dose, seed or temperature sweep.

## Resume integrity and saved evaluation

The source extension admits only the exact saved175 report/manifest/field
triple and fractional master hash, with `cross_state_bottleneck=true`.
It authenticates saved175's local checkpoint256/lineage576 and saved145 ancestry,
including the prior local256/lineage320 provenance, objective and initial/final
receipt policies. Mixed profiles and phase-balanced saved145/saved175 requests
reject; existing fresh64, saved22 and bottleneck saved145 behavior is retained.
Exact fractional F32 masters restore and re-export to the pinned native field;
Adam moments explicitly reset. A separate zero-field cache ensures learned
energy is added once, and output-overlap guards protect the sealed parent.

Before any update, full native replay reproduced all 512 saved175 row IDs,
generated sequences and completion flags, including exactly 175 complete
replies. The endpoint was independently saved/reloaded. Its early175 check
retained 168, and full 512 scoring followed despite seven losses. Labels,
membership grading and own-prefix generation remain unchanged. The
[independent saved reader](reviewer-resume175.json) verifies exact baseline,
frozen cache provenance, schedule, native payloads and saved result comparisons.

## Measured result

| Measure | Saved175 parent | Saved832-lineage endpoint |
| --- | ---: | ---: |
| Complete replies | 175/512 | **177/512** |
| Correct entry token | 340/512 | **298/512, worse** |
| EOS-ending outputs | 182/512 | 183/512 |
| Complete source-swap pairs | 35 | **32, worse** |
| Teacher-prefix all-token correctness | 175/512 | 177/512 |
| Teacher-prefix correct tokens | 3,523/6,664 | **3,304/6,664, worse** |
| Equal-episode teacher-prefix CE | 3.48473505826869 | 3.2068708694739447 |

| Memory stratum | Cases | Before | After |
| --- | ---: | ---: | ---: |
| length2 | 128 | 55 | 56 |
| length4 | 128 | 57 | 57 |
| length8 | 128 | 8 | 11 |
| reassert | 64 | 34 | **32, worse** |
| update | 64 | 21 | 21 |

The reader retains exact gained, lost and retained IDs. Seven previously
complete replies are lost:

- `development-diverse-length2-04-swap0-q1-forward-job`
- `development-diverse-length2-04-swap0-q4-forward-home`
- `development-diverse-length4-05-swap1-q4-reverse-job`
- `development-diverse-length4-06-swap0-q2-reverse-home`
- `development-diverse-reassert-00-swap0-q1-forward-home`
- `development-diverse-reassert-02-swap0-q3-forward-job`
- `development-diverse-reassert-03-swap1-q4-forward-home`

There are 344 changed outputs versus saved175. Against accepted145, 140 successes
remain, 37 are gained and five are lost; against saved22, 20 remain and two are
lost. Six of the original eight remain. Against rejected139, 135 remain, 42 are
gained and four are lost. Exact comparisons also cover #2141, #2143, #2148,
#2149 and old U64; #2148 loses one of its four successes. These compare pinned
saved reports, not fresh reruns or unions of outputs. Native inspection finds
18,419 changed legal coefficients of 115,200, 112,755 nonzero final codes,
range [-7,7] and valid padding.

The net gain coexists with 42 fewer correct entry tokens, 219 fewer correct
teacher-prefix tokens, three fewer complete source-swap pairs and two fewer
reassert completions. Lower evaluation CE does not erase those regressions.
The evaluator reports ordinary equal-episode mean token CE, distinct from both
the former phase-balanced training loss and the bottleneck J. All recorded J
values are finite; first/last batch values 1.0792547287188712 and
6.995252101188744 belong to different scheduled batches and are not a
like-for-like learning curve. The independent reader verifies metadata and
finite scalars without recomputing training J or gradients.

## Decision and limits

Retain saved177 under the prospective **complete>175** bar, preserving parents
and exact losses. The small two-reply net gain and regressions warrant reviewing
a distinct model-changing intervention rather than automatically repeating the
dose. That is a proposal for the next decision after delivery and cleanup;
no particular successor intervention is selected, implemented or registered here.

**Before opening the PR: if this came out the other way, would my next step
differ? Yes.** At 175 or below, retain saved175, advance to count 1/3 and stop
unchanged bottleneck dose/rate/seed continuation. The observed gain retains
saved177 and count 0/3; the successor still needs an evidence-supported decision.

All 512 rows are exposed development data used in both training and evaluation.
Teacher-prefix training and own-prefix endpoint generation are distinct. This
single run establishes neither held-out transfer nor general conversation,
geometric advantage, unique factual-carrier necessity or full-path energy
improvement. M2 still requires at least 256/512 followed by at least 40% on a
fresh draw with criteria fixed first. Fresh qualification is NOT_RUN; 335
complete replies fail and length8 remains weak. The standing goal continues.

## Verification, identities and cost

Producer source is `25b36a074c682935d7319f04049ff3831afd8d26`, based on
`4f12574a4f442445909af5ec885aba84fb4dc1ae`. The optimized executable SHA-256 is
`864164a113a3957bffc9e5e17d5eaa96611799f6f44b62c3dbc1eb92db460683`;
configuration SHA-256 is
`522e96a4d76820d5f8c79cd77a43e9177bc7838acc9019dc11bebd6c58718f12`.
The [producer receipt](producer.json) records optimized build, formatting/diff,
**19 focused tests PASS** and independent exact-head source review PASS.
The two added tests cover saved175 ancestry/master/policy mutation rejection
and baseline175/net-bar/lineage832; the 17 prior objective, restoration,
admission, schedule and integrity tests remain. Stock bootstrap took 196 s,
with 37 parity tests passed and three ignored.

| Identity | SHA-256 |
| --- | --- |
| Parent report | `9632d32651d0bcce82b2cda2ba70dee7d7a7878ff99577f88002442e23acb572` |
| Parent manifest | `1a6216a307a739a804f81bf94e93fd89638a896fda1904986dd3aee0a280fccc` |
| Parent field | `c82c376df10f14d5c1c9af970fd3b1fe239fb3560e2ccd0020dd2805d8fb773e` |
| Restored fractional master | `c59f6eb8898a7b7f3ebef0a877a7a26fef3ab3eb681a55e8c6e02cf701d11f10` |
| Endpoint report | `13fa577beacbbe3fd0544095c8f4a6e8555458721325f000b8a419d0a90a8e48` |
| Endpoint manifest | `10db47cb5e29d562df485e8364fe60bf503b5b4bd40f7b4797f40a4c65e30156` |
| Endpoint field | `dfb945d567ce25f5c200967ce1f9e5e31740f9cfaa8f6a1a6d6dd11d156a2e13` |
| Independent reader | `680f131e3c6074e81b3f755d7d1140a79d18a6625463c0286d31de73e9368480` |
| Independent review JSON | `90baf49f2f193dcb30d60b255d5463ce2c2a0de537941695a1cee784958c8ec2` |

[Artifact receipts](artifact-receipts.json) retain complete upstream, input and
master identities. The producer seals and verifies the report set. Independent
review checks saved hashes/IDs, actual-prefix chains, EOS, frozen membership and
Q4 payloads; it does not rerun the model, backward pass, tokenizer decoding or
native scores. Full BLAKE3 seal verification is producer evidence; the reader
checks inventory and sizes. Those are the limits of the saved-evidence review.

The model process began 2026-10-10T13:12:07.769577Z and exited 0 after
769.1431097490713 s, with peak child RSS 792,956 KiB and sampled GPU memory peak
824 MiB over 78 samples. Producer elapsed time was 765.236100299 s:
48.751815221 s cache preparation, 216.856460077 s fit excluding checkpoints,
3.930260363 s checkpoints and 489.971639466 s evaluation. The
[measurement summary](measurement-summary.json) retains exact values. These
are process measurements, not complete-cycle cost or serving efficiency.

The registered full-cycle projection is 90 min/about $1.785 for one 5090,
including preparation/build/checks/model/review/preservation/delivery/cleanup;
resource projection is at most 6 GiB RAM/GPU, 16 GiB remote temporary storage
and 512 MiB compressed retained storage with a 128 MiB margin. The local source
projection is at most 2 GiB with 41 GiB free; no local model, build or archive
was planned. These are admission projections, not final charges. Complete-cycle
cost, final exact-head delivery checks, protected merge/tree verification, pod
release/deletion and branch/worktree cleanup remain **pending**, with final receipts
on [#2030](https://github.com/UOR-Foundation/uor-r4/issues/2030).

The full preservation package is 135,495,680 bytes, MD5
`efb0195a8143d92bd4826ecefe748564`. iCloud full-stream readback matches, and the
standard six-column index readback matches. No local archive was staged.
The [package descriptor](PACKAGE.json), [preservation receipt](preservation.json)
and [independent review](independent-review.md) retain exact scope and evidence.
Restore with `cloud-store fetch codex-m2-bottleneck-resume-20261010 <absolute-owned-destination>`
and follow the descriptor's separate unpack and identity checks. It includes the
saved175 and earlier evidence, source bundle/patch, model/runtime and run receipts.
This model KEEP and preservation decision does not imply protected delivery.
