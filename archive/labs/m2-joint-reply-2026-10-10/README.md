# Joint Context and prototype complete-reply qualification

**REJECT.** Joint Context–prototype learning produces **0/512** complete replies
and **0/24** trained replies, down from the accepted parent's 8/512 and 8/24.
All eight accepted successes are lost; none are gained. Accepted M2 remains
**8/512** against target 256.

**Line: ordinary reply-completion gradient learning · count 4/3 (the single
D21 decisive continuation after the 3/3 pivot) · headline 8/512 → 8/512.**
No count reset follows this intervention. This negative closes the ordinary
24-row/96-update recipe; the standing goal continues with a different mechanism.

## Question and fixed intervention

The [pre-registration](https://github.com/UOR-Foundation/uor-r4/issues/2030#issuecomment-6095654613)
and its [exact text](preregister.md) precede compute. The run is the single
decisive continuation named in the [3/3 pivot](https://github.com/UOR-Foundation/uor-r4/issues/2030#issuecomment-6095599251).
It adds all nine Context parameter families to #2148's prototype-enabled fit,
keeping the accepted Source48/Generate64 parent, frozen 512 inputs/labels,
24 exposed rows, seed 1001, batch 8 and 96-update endpoint fixed.
The [config](model-config.json) specifies 32 repetitions, 768 episode draws and
9,280 answer/EOS target positions. These correlated development examples are
not independent held-out trials.

Earlier #1822 trained Context and prototypes from an older checkpoint under
legacy marker credit. Accepted-parent #1862/#1867 used categorical credit but
froze prototypes. #2148 enabled prototypes but froze Context. This run tests
their joint adaptation under categorical credit; it is not the first Context
fit or evidence that Context is a proven bottleneck. Removing U alone supplies
no novel causal explanation.

Context AdamW uses 0.002 with clamp [-1.75, 1.75]; prototypes use 0.01,
Generate coefficients and Potential 0.003. All active gradients enter common
clipping. Other Source, bridge, Cue, Prefix and exponent masters remain frozen
and authenticated. Potential may change Source selection. The existing graph
carries categorical full120 conditional credit, which is not a discrete argmax
derivative or a guarantee of native descent. No new estimator, constructor,
solver, U or supplied serving source is added.

The discarded eight-episode admission requires all nine Context gradients
present and finite with positive aggregate norm, and the same for prototypes.
Checkpoints 24/48/72 are recovery-only; 96 is the sole endpoint. The initial
saved/reloaded baseline reproduces the original eight IDs. Final saved/reloaded
native own-prefix full512 evaluation runs regardless of loss. KEEP requires all
original eight retained plus at least one new complete reply; exposed-fit
qualification separately requires 24/24.

## Measured result

| Measure | Accepted parent | Update 96 |
| --- | ---: | ---: |
| Complete replies, full512 | 8 | 0 |
| Complete replies, trained24 | 8 | 0 |
| Entry correctness, full512 | 8 | 1 |
| Entry correctness, trained24 | 8 | 0 |
| EOS-ending outputs, full512 | 11 | 0 |
| Complete source-swap pairs | 4 | 0 |
| Teacher-prefix all-token correctness, trained24 | 8 | 0 |
| Teacher-prefix correct tokens, trained24 | 96/290 | 34/290 |
| Equal-episode teacher-prefix CE, full512 | 6.1992635176344875 | 5.338729831415192 |

All sixteen added training rows still fail at entry. All 512 outputs change
versus the accepted parent and versus each prior rejected endpoint. Exact-ID
comparisons with #2141, #2143 and #2148 give complete counts 2→0, 2→0 and 4→0:
every prior success lost, no additions. The [saved-row review](saved-row-review.json)
lists identities and comparisons.

The intervention changes all nine Context master families and
**12,307 of 8,963,136 native Context Q4 coefficients**, plus
**25,718 of 32,768 Generate prototype lane codes**, 269 pair, 118 unary and
one bias code. The finite admission norms are 0.23880831091868573 for Context
and 0.01705411995955738 for prototypes. Independent review reconstructs both
native Context and prototype crossing counts directly from saved payloads.
Thus absent native updates do not explain the negative. A changed Source
consumer-tree digest also includes metadata; it is not a categorical-state
crossing count.

## Decision and next

Reject and preserve update 96; retain the accepted 8/512 parent. Close this
ordinary 24/96 joint/frozen-Context recipe without another family-by-family,
dose, rate or seed continuation. The single D21 decisive continuation is
exhausted. A genuinely different mechanism needs a new causal justification,
registration and fixed-panel result. The protected constructor/attribution/
solver line remains closed.

The new joint mode is **not activated on main**: its exact producer diff is
retained as an [indexed source patch](../../history/branch-archive/joint-reply-negative-20261010.patch)
from base 52022ae10c35ed68011124bb3a8994154425041c to producer
1833cb4dd1ee1dad65ca6c850b209ab16628e062. The precompile failure is separately
archived. Existing main modes are unchanged.

**Counterfactual before the PR:** yes. Retained new complete replies would keep
a candidate, and 24/24 fit would support expanded qualification. This regression
instead closes the recipe. Lower CE does not override complete-reply failure.
The result does not establish architectural impossibility, convergence,
unattainability of 512/512, or a uniquely identified cause. M3, general
conversation and serving-energy claims remain unchanged.

## Verification and cost

[Producer receipt](producer.json) binds source
1833cb4dd1ee1dad65ca6c850b209ab16628e062 and executable SHA-256
261920246ba254c4fd6dae035b78fedfdc1750ae2b1886477fed7322be80523a.
The first build at 2b0651a failed with three borrowed-string type errors:
187.354 s, no model execution. The corrected optimized CUDA-feature build
passed in 119.814 s; 10 focused reply tests passed, none ignored, in 22.566 s.
Source review passed separately; it is not build evidence. Stock bootstrap
took 205 s (144 s build, 47 s parity; 37 tests passed, 3 ignored), with its
stock 15 build jobs; changed builds used 4 jobs and model execution 2 threads.

The Rust producer reloads, seals and verifies the complete report file set.
The [saved reader](review-result.py) authenticates all 1,024 endpoint row
hashes/IDs, own-prefix feedback/chosen-token chains, EOS, fixed answer membership,
schedule and all three prior reports. It does not independently regenerate
gradients, native scores or tokenizer decoding. [Independent result review](independent-review.md)
passed, including direct native-payload checks.

Report SHA-256: bed3e8eec305f38cfbff7fcb54a1fc7fbb159b3657e4efd17709ddf74e8c8616.
Manifest SHA-256: f8172b7b06ebc94230bdb2336a31a2b39840cbb180d2ec22fc71c12408b0cfc5.
[Measurements](measurement-summary.json): baseline 211.024 s, fit/checkpointing
1289.211 s, endpoint evaluation 303.483 s, producer 1824.345 s, whole process
1828.860 s. Peak RSS 1,463,804 KiB; sampled GPU peak 1,702 MiB (184 samples).
These are research-run costs, not optimized serving or energy measurements.

The projection is 120 minutes total cycle, one RTX 5090 at $1.19/h for an
estimated two hours (≤$2.38), model/build threads 2/4, peak RAM/GPU 6 GiB,
remote storage ≤16 GiB and retained compressed material ≤512 MiB. Cumulative
model-time stood at 1,475,947,627 ms at 08:21:30 UTC; a recorded standing-authorized
3,600,000 ms extension raises the limit to 1,485,000,028 ms. Whole-cycle
preparation, failed build, model, review, preservation, delivery and cleanup
charges are posted after protected delivery; no reset. Final delivery-head
checks/review are recorded on the result PR, separately from producer tests:
the joint Rust mode is archived there, so its two added tests are not active.

## Durable evidence

The verified package `codex-m2-joint-reply-20261010` in
`icloud:UOR-R4/results/codex/` preserves the sealed run/checkpoints, executable,
exact accepted parent/panel, all three prior reports, source bundles and
execution/build receipts. [Component identities](PACKAGE.json) bind these
materials. Restore with `cloud-store fetch codex-m2-joint-reply-20261010 <destination>`,
extract the component archives and remap config/prior-report paths. The producer
source bundle requires published base 52022ae10c35ed68011124bb3a8994154425041c.

Local disk was below the 30 GiB floor; required hygiene preserved other labs'
active/recent/unarchived material. The owned worktree and durable outputs use
/workspace on pod 8uckvwxtq0oruw; only its per-lab build cache is transient.
No local model/build/archive copy is created. The outer archive is streamed to
the existing iCloud remote, then fully streamed back for MD5 verification before
publishing and rereading the six-column index. [Cloud receipt](cloud-put.txt)
records verification. Final protected delivery and cleanup receipts follow on
the result PR and in a separate closeout archive.
