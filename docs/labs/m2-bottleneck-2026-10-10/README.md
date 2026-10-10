# Episode-bottleneck learning improves complete replies

**KEEP: 145/512 → 175/512 complete replies**, with 34 gains, four losses and
141 retained successes on the unchanged frozen panel. Entry correctness,
teacher-prefix token correctness and evaluation CE regress; the registered bar
is complete own-prefix replies. The saved endpoint meets that developmental
bar, but M2 remains below 256/512 and its fresh-draw 40% qualification is NOT_RUN.
There are 337 failed complete replies, and length8 reaches only 8/128.

**Line: cross-state continuation · count 0/3 · headline 145/512 → 175/512.**
The gain resets the inherited 1/3 no-headline count. No preparation PR was used.
The stopped phase-balanced dose/rate/seed recipe, ordinary 24/96 recipe and
protected constructor/attribution/solver line remain closed. M3 is unchanged.

## Question, objective and fixed model

The [pre-registration](https://github.com/UOR-Foundation/uor-r4/issues/2030#issuecomment-6097346580)
([retained text](preregister.md); [pre-run metric-label correction](https://github.com/UOR-Foundation/uor-r4/issues/2030#issuecomment-6097395590)) replaces equal weighting of nonempty
entry/Copy/other phases with a smooth episode-bottleneck objective. Starting
from the accepted145 model, rather than the rejected139 endpoint, it tests
whether emphasizing a reply's difficult target positions improves whole replies.

For unweighted native-forward token CE `ell[e,t]`, including EOS, the objective is

```
J[e] = log((1 / T[e]) * sum_t exp(ell[e,t]))
J[batch] = (1 / B) * sum_e J[e]
dJ[batch] = (1 / B) * sum_e sum_t softmax_t(ell[e,*]) * d ell[e,t]
```

Temperature is 1. There is no additional phase factor or 1/T gradient factor.
The scalar token CE uses the actual native pooled token probability; its
backward pass retains the existing raw-identity alias and Q4 coefficient
straight-through adjoints. This is the stated surrogate composition, not an
exact derivative of integer rounding or greedy generation.

Stable streaming keeps a maximum `m`, shifted sum `Z` and detached gradient
numerator `G`. For each token, `m'=max(m,ell)`, `alpha=exp(m-m')`,
`beta=exp(ell-m')`, `Z'=alpha*Z+beta` and `G'=alpha*G+beta*dell`.
The episode contributes `(m+log(Z)-log(T))/B` and `G/(Z*B)` at its end.
There is one forward/backward per token and one live token graph; no retained
whole-episode graph or differentiated weighted-mean loss is substituted.
Batch clipping and AdamW follow the episode-gradient sum.

Only the 115,200 existing cross-state Q4 coefficients learn. They read directed
signed H4 relatives of the factual post-bridge and independently replayed
query/prior-prefix states; the same token energy reaches Generate and every
physical Copy alias before the existing common clip and pool. Source48/Generate64,
Context, Potential, Generate prototypes/coefficients, bridge, Cue, Prefix,
tokenizer and exponent table remain frozen. The scorer, native kernel, decoder,
shift22 policy, 65,536-byte padded payload and evaluation are unchanged.
Existing dense vocabulary access remains; there is no serving-cost claim.

The [configuration](model-config.json) fixes 256 updates, batch 8, seed 1001,
learning rate 0.03, AdamW betas 0.9/0.999, epsilon 1e-8, weight decay 0, global norm
clip 1 and quarter-master clamp [-1.75,1.75]. Four repetitions of the same shuffled
512-row order give 2,048 episode draws and 26,656 answer/EOS draws over 6,664 cached
positions. Target length is at most 32 including EOS; complete teacher-prefix
context is at most 128. Local 0→256 corresponds to lineage 320→576. Checkpoints
every 32 are recovery-only; local 256 is the sole selected endpoint. There is
no rate, dose, seed, temperature or checkpoint sweep.

## Parent integrity and saved evaluation

The strict new objective is admitted only with the complete pinned saved145
report/manifest/field triple. Saved145 under the stopped phase-balanced
objective is rejected; fresh64 and saved22 retain their existing behavior.
The loader authenticates the parent seal, local checkpoint256/cumulative320,
upstream identities, fractional master inventory and saved22 ancestry.
Fractional F32 bits restore and must re-export to the exact parent native field.
Adam moments reset explicitly. A separate zero field supplies the immutable
cache so the learned energy is added once; output-overlap guards protect the
sealed parent before report creation.

Before any update, actual full 512 native replay reproduced every saved row ID,
generated sequence and completion flag, with exactly 145 complete replies.
The saved/reloaded endpoint's early145 check retains 141 successes; full 512
scoring follows regardless of loss. Frozen labels and membership grading are
unchanged. The [independent saved-evidence review](reviewer-bottleneck.json)
verifies 1,024 baseline/endpoint rows, schedule, bindings and native payloads.

## Measured result

| Measure | Saved145 parent | Saved bottleneck endpoint |
| --- | ---: | ---: |
| Complete replies | 145/512 | **175/512** |
| Correct entry token | 454/512 | **340/512, worse** |
| EOS-ending outputs | 148/512 | 182/512 |
| Complete source-swap pairs | 25 | 35 |
| Teacher-prefix all-token correctness | 145/512 | 175/512 |
| Teacher-prefix correct tokens | 4,110/6,664 | **3,523/6,664, worse** |
| Equal-episode teacher-prefix CE | 3.394481661707875 | **3.48473505826869, worse** |

| Memory stratum | Cases | Before | After |
| --- | ---: | ---: | ---: |
| length2 | 128 | 53 | 55 |
| length4 | 128 | 45 | 57 |
| length8 | 128 | 2 | 8 |
| reassert | 64 | 28 | 34 |
| update | 64 | 17 | 21 |

The 34 gained and 141 retained IDs are listed in the independent review. Four
previously complete replies are lost:

- `development-diverse-length2-03-swap1-q3-forward-job`
- `development-diverse-length2-04-swap0-q4-reverse-home`
- `development-diverse-length2-05-swap1-q5-forward-home`
- `development-diverse-reassert-03-swap1-q1-forward-home`

There are 371 changed outputs versus the parent. Six of the original eight
successes remain; against saved22, 20 remain and two are lost. Against rejected139,
134 remain, 41 are gained and five are lost. The review also preserves exact
comparisons to #2141, #2143, #2148, #2149 and old U64; #2148 loses one of its four
successes. These are comparisons of pinned saved reports, not fresh reruns or
unions of outputs. Native inspection finds 29,694 changed legal coefficients out
of 115,200, 112,501 nonzero final codes, range [-7,7] and valid padding.

The whole-reply gain does not imply a general improvement in token prediction:
114 fewer rows have a correct entry token, 587 fewer teacher-prefix tokens are
correct, and evaluation CE rises. The evaluator's CE is the equal-episode mean
of token CE, distinct from both the former phase-balanced training loss and the
new J. New update receipts report J explicitly and set `phase_balanced_loss` to
null. All saved J values are finite; first/last batch values 1.81987490391629 and
7.113865808012021 describe different scheduled batches and are not a like-for-like
loss trajectory. The independent reader checks these values and policy metadata
without recomputing training gradients.

## Decision and limits

Retain the saved175 candidate under the prospective complete>145 bar; preserve
the accepted parents and exact losses. After protected delivery and cleanup,
pre-register parameter continuation from saved175 under this successful new
objective. That is a continuation of the bottleneck objective, not a reopening
of the stopped phase-balanced recipe. No successor has been run in this cycle.

**Before opening the PR: if this came out the other way, would my next step
differ? Yes.** A non-gain would retain 145, advance the count to 2/3 and stop
unchanged bottleneck dose/rate/seed continuation. The observed gain retains 175,
resets the count to 0/3 and supports the next registered learning decision.

All 512 examples are exposed development data used for training and evaluation.
The objective uses teacher prefixes, not the model's own generated trajectories;
the own-prefix endpoint is a separate measurement. This single fixed run does
not establish held-out transfer, general conversation, geometric advantage,
unique factual-carrier necessity or full-path energy improvement. M2 still
requires 256/512 and then at least 40% on a fresh draw with criteria frozen first.
Length8 remains weak and 337 complete replies fail. The standing goal continues.

## Verification, identities and cost

Producer source is `351e93dcc7763551281c22c3b78f64e1c60ab479`, based on
`e74298125b89953957e102ca5a6bc9676f4f3fd2`. The optimized executable SHA-256 is
`302e36db6826a411acb021e0dc862af564309b8921629d18db7461cadb2573d1`;
configuration SHA-256 is
`24f7e26e20819bf1f8358448ea2645e9ebcfb013402884b947d4bc6a1d074dcc`.
The [producer receipt](producer.json) records optimized build, formatting/diff,
17 focused tests and independent exact-head source review PASS. Tests include
analytic and directly differentiated anchored-STE agreement, unequal episode
lengths/EOS, max-shift stability, invalid/finite-gradient cases, strict admission
and inherited restoration/lineage/baseline/schedule/output-root checks.
Stock bootstrap took 181 s with 37 parity tests passed and 3 ignored.

| Identity | SHA-256 |
| --- | --- |
| Parent report | `e3a7e3ebd93c38253afb67f77ad3bb6b64a5e0ac50561f0204e1b83d22b8f4a5` |
| Parent manifest | `172a2aacdce188bd5a37b131a294b877b2af60c87fc5520a9ea371c4b79460dd` |
| Parent field | `03781884edd6c2524c66e724c1fc65d9b9c62968d33b264d976282b49ce8094a` |
| Restored fractional master | `0dd554268ec0b9883d8dd3c80329d3ae8500c817e59524dfe9718acb348fa5fb` |
| Endpoint report | `9632d32651d0bcce82b2cda2ba70dee7d7a7878ff99577f88002442e23acb572` |
| Endpoint manifest | `1a6216a307a739a804f81bf94e93fd89638a896fda1904986dd3aee0a280fccc` |
| Endpoint field | `c82c376df10f14d5c1c9af970fd3b1fe239fb3560e2ccd0020dd2805d8fb773e` |
| Independent review JSON | `1e0477dc4498635a559fcbfc908a6bd7b46791a84a267019142c091870f5c975` |

[Artifact receipts](artifact-receipts.json) retain full input and parent/master
identities. The producer seals and verifies the report set. Independent review
checks saved hashes/IDs, actual-prefix chains, EOS, frozen membership and Q4
payloads; it does not rerun the model, backward pass, tokenizer decoding or native
scores. Full BLAKE3 seal verification is producer evidence; the reader checks
inventory and sizes. Focused checks and saved-evidence review have that scope.

The run began 2026-10-10T12:22:49Z and exited 0 after 705.9431589390151 s, with peak
child RSS 802,204 KiB and sampled GPU memory peak 837 MiB over 72 samples. Producer
elapsed time was 702.13943619 s: 46.834000317 s cache preparation, 206.754529985 s fit
excluding checkpoints, 3.917428561 s checkpoints and 439.67831104699997 s evaluation.
The [measurement summary](measurement-summary.json) records exact values.
These are process measurements, not complete-cycle cost or serving efficiency.
The registered full-cycle estimate is 90 min/about $1.785 for one 5090, including
preparation/build/checks/review/preservation/delivery/cleanup; final charges and
pod release/deletion remain pending and will be recorded on
[#2030](https://github.com/UOR-Foundation/uor-r4/issues/2030).

The preservation package `codex-m2-bottleneck-20261010` is 114,309,120 bytes with
archive MD5 `8b10bf24ca7d5f042a7395eeb4164fb5`. Full-stream iCloud readback
matches that MD5, and the standard six-column index/readback checks pass; the
[preservation receipt](preservation.json) records verification. No local archive
was staged. [PACKAGE.json](PACKAGE.json) describes the exact source, run/runtime
and retained-parent components, and the [independent review](independent-review.md)
records its result assessment. Final exact-head delivery checks, protected
merge/tree verification and branch/worktree cleanup remain pending; they are not
implied by this model KEEP decision. Restore with
`cloud-store fetch codex-m2-bottleneck-20261010 <absolute-owned-destination>`,
following the descriptor's separate unpack and identity checks.
