# Ordinary complete-reply gradient learning from the accepted native model

## Question and fixed experiment

Does ordinary complete-answer training improve the frozen 512-reply headline
when the accepted Source48/Generate64 learner's Potential and Generate coefficients
learn together while its Context, prototypes, bridge, Cue and Prefix stay fixed?
The [pre-registration](https://github.com/UOR-Foundation/uor-r4/issues/2030#issuecomment-6093395330)
fixes one endpoint and all outcome decisions. This follows the owner's D21 pivot;
no protected constructor, discrete proposal, attribution or solver step runs.

The accepted field learner previously trained eight cases. A historical full-panel
joint Context/Potential/Generate/U run regressed from 8 to 0 complete replies.
This intervention broadens the successful field learner's supervision to all512
complete answers, without that moving-Context/U intervention. It is an exposed
development construction experiment; training and evaluation use the same frozen
panel. There is no held-out or general-conversation claim.

## Artifact, data and mechanism

Base: `native-prediction-control/recomposition-503d64b39-attempt1/checkpoint-0000`.
Report SHA256 `a1277d2be4ff962100f857d0da553257ad5596ac8bc72d9257299e74d2f2e278`;
manifest `b37e2ca588bf1cc4dc5971fa5da97b9e83c90a94f4ea0bfdb0655b8f92bf94ca`.
Input SHA256 `b9661606b280884217a64e0a5b643f8324a90390e47ade7241da0889a5f7c86a`;
labels `84991e0657b5697c0e061eaa3fe86e4a0ec7ce6bc2be8371b62698c6b8126155`.
The actual runtime verifies the complete seals and restores fractional masters,
then verifies their native exports against the accepted model.

The new `reply_completion` mode in `geometric-frozen-map-fit` uses the existing
complete-answer teacher-prefix objective, including EOS, native Generate/physical
Copy pool and equal-nonempty-phase weighting. It fixes seed1001,64 AdamW updates,
batch8, one pass over512 episodes, raw-identity score adjoint and categorical
pullback. Generate coefficients and Potential each use learning rate0.003;
betas0.9/0.999, epsilon1e-8, weight decay0. Active gradients are filtered before
shared norm clipping. No continuation U is introduced. Potential can change
physical Source selection; frozen Context parameters do not imply frozen donor
or post-read state. The shared H4 geometry and integer serving path are unchanged.

All non-Potential Source masters, Generate prototypes and bridge masters are
fingerprinted after every update. Final saved Cue, joint-Cue, Prefix and exponential
lookup payloads must equal the parent. Checkpoint32 is retained for recovery,
not selection; checkpoint64 is the single endpoint. The initial native evaluation
must reproduce the same eight successful row indices, not merely an aggregate8.
Both endpoints use the existing own-prefix generation limit32, total admission128,
and accepted-answer membership plus EOS. State width is eight lanes; vocabulary4096.

## Decision and validation

Local KEEP requires every original complete reply retained and at least one new
complete reply. The milestone remains256/512 followed by a fresh>=40% draw under
unchanged acceptance. The full final512 evaluation runs regardless of training CE.
No loss-dependent constructor gate can suppress endpoint scoring.

Source-only independent review found no blocking defect. Focused tests cover mode
admission, excluding frozen gradients from clipping, and rejecting a candidate
whose gains accompany losses. The zero-update admission also computes and discards
one eight-episode gradient batch; this preparation cost is additional to the64
optimizer batches. The completed result and exact validation receipts follow.

## Resources and preservation

The complete projection is120 minutes, one leased RTX5090 at$1.19/hour through
`uor-pod`, model RAM<=6GiB/GPU<=4GiB, local transient<=6GiB, retained reports<=2GiB,
and task pod transient<=12GiB. Task builds use four threads; the stock bootstrap
used its15-thread default. Local floor30GiB plus128MiB is retained. Standing
same-class allowance was extended120min from limit1467000028ms to1474200028ms;
prior cumulative1464005243ms is preserved, with all preparation/run/delivery charged.

The first remote build wrapper failed before Cargo because `/usr/bin/time` is absent.
Its source/log are retained; a subprocess/resource accounting wrapper repairs only
that execution boundary. The first focused compile also caught two endpoint scorer
calls receiving an artifact binding instead of the required source-action binding;
repairing those call arguments yields three passing CPU tests (zero ignored). The first transfer also reported ownership metadata errors;
repeating with no source ownership restoration and no macOS extended attributes
repaired transport. Neither setup fault is model-quality evidence.

Closed-line source remains on main or in the branch archive. The indexed absolute
legal representation, initial observer labels and unpromoted residual-backend patch
remain preserved; D21 closes their next-step recommendations without deleting evidence.

## Result: REJECT the candidate; accepted headline unchanged

**Line: ordinary full-panel reply-completion gradient learning · count 1/3 ·
M2 headline 8/512 → 8/512 (candidate 8/512 → 2/512).** Zero preparation PRs;
this integrated source/result is the session deliverable.

| Frozen open-development measurement | Accepted parent | Saved update64 |
| --- | ---: | ---: |
| Complete exact accepted reply with EOS | 8/512 | 2/512 |
| Complete source-swap pairs | 4/256 | 0/256 |
| Correct entry token | 8/512 | 7/512 |
| Any EOS emitted | 11/512 | 3/512 |
| Equal-episode native teacher-prefix CE | 6.1992635176344875 | 5.4545225648632965 |

Two original successes survive, six are lost and zero are gained. All remaining
complete replies are `length2-00-swap1`, forward-home queries q0 and q3.
510/512 generated token sequences change. The [machine summary](result-summary.json)
contains every retained/lost ID and additional conditional metrics; they do not
replace the complete-reply headline. The saved-row reader authenticates all1,024
row hashes/IDs, own-prefix feedback, chosen tokens, EOS and exact frozen answer
membership. It does not independently rerun the tokenizer, native scores or backwards.
The producer independently reloads checkpoint64 and seals/verifies the whole report.

The run completes64 updates,512 episode draws and6,664 target positions including
EOS. Baseline evaluation takes206.302s; fit including checkpoints1,139.810s;
final evaluation301.867s. The full process takes1,680.353s (28.01min), peak RSS
1,467,840KiB and sampled GPU maximum2,015MiB at10-second sampling. These are
training/evaluation costs, not served performance or energy measurements.

Producer commit `9a1394f4efb480fbabe95c2c8c80b3c4ad24dce0`; executable SHA256
`dbcf785773136115ececb066a828b6e3102edf5bb1b12d19accd1194ea253b8e`.
Report SHA256 `678f4ff4d8b97559260076607f6168b7d5f4317cc1472bd9fe5e253ca76bdbb7`;
manifest SHA256 `3e5407ada60dc04f3188ab946136ad2c68021c1c0500bbcdb16c287ffa2adda1`.
CPU and CUDA-feature focused tests each pass3/3 with zero ignored; the initial
CUDA bootstrap parity suite separately passed37 tests with3 ignored. The corrected
producer CUDA build takes114.731s and its focused-test command25.075s; failed
compile211.189s is charged separately. Source review includes the main integration
at `afebac942d0553ed73c6ea729634c0fbb5c0bb3e`.

**Counterfactual PR question:** if the result came out the other way, would the
next step differ? Yes: a retained complete-reply gain would justify wider
qualification; this regression rejects the candidate and disallows an unchanged
run or dose/rate/seed sweep. Lower CE does not justify promotion. This one fixed
intervention does not establish an architectural capacity limit.

**Next:** retain the accepted8/512 parent; stop this fixed Potential/Generate
full-panel recipe. Any successor must pre-register a distinct model-changing
complete-reply intervention and score the same512 panel; no constructor,
attribution or solver work is reopened. No successor compute is started here.
Held-out, general conversation, durable memory and complete-path energy remain
unqualified by this result. M3 was not owner-blocked work and is unchanged.

## Durable evidence

The full sealed baseline/candidate reports, checkpoints0/32/64, exact executable,
setup/compile failures, telemetry and actual parent/panel are preserved in
`icloud:UOR-R4/results/codex/codex-m2-reply-gradient-20261010.tar`.
`cloud-store put` verifies the uploaded tar by MD5 download round-trip before
publishing its index. [Package hashes](PACKAGE.json) bind the compressed components;
[preservation receipt](cloud-put.txt) records the verified package. Source history
is included as a Git bundle with published main
prerequisites, so precompile failed source is recoverable after branch deletion.
Restore using `cloud-store fetch codex-m2-reply-gradient-20261010 <destination>`;
extract the two compressed input/run archives and remap the config's local paths.
Do not reuse a sealed report root for a rerun.

The final PR check, exact-head review, merge verification, pod shutdown and complete
session-cost/ledger closeout are posted on the result PR and #2030. They remain
separate from this measured producer identity and do not rerun training.
