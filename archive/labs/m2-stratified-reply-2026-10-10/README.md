# Stratified complete-reply fitting qualification

The owner requested a direct learnability check after the frozen512 headline
remained 8/512. The preceding [full-panel one-pass result](../m2-reply-gradient-2026-10-10/README.md)
regressed 8→2 despite lower teacher-prefix CE. The subsequent source/data assessment
found that all canonical answers fit vocabulary/context/output limits, but no
learned artifact demonstrates full-panel realizability. The old 512 answerability
reference is explicitly not a native prediction result.

## Question and pre-registration

Can ordinary Potential/Generate training fit a prospectively fixed 24-row
complete-answer curriculum while retaining its original 8 successes? The causal
intervention combines a narrower training distribution with repeated exposure;
it does not isolate extra steps or prove architectural capacity. This is the
same ordinary-learning line, entering at count 1/3; the closed protected/discrete
constructor, attribution and solver line stays closed.

[Pre-registration](https://github.com/UOR-Foundation/uor-r4/issues/2030#issuecomment-6093892615)
and [exact text](preregister.md) precede compute. The [fixed row list](qualification-rows.json)
contains 8 original controls and 16 initially unsuccessful rows: length 4 reverse,
length 8 forward, update reverse and reassert forward. Both job/home and both
source-swap arms are present, with selected q0/q2/q3/q4 wordings. All selected
banks are 00: correlated exposed examples, not 24 independent tasks or transfer.

## Fixed mechanism and endpoint

Accepted Source48/Generate64 initializes fresh ordinary AdamW at 0.003 for
Potential and Generate coefficients. Context, Generate prototypes, bridge,
Cue, Prefix and exponential payloads remain fixed; no U or target-supplied source
selection. Source selection can still change through learned Potential.

Mode `reply_qualification` fixes seed 1001, 96 updates, batch 8 and one deterministic
24-row shuffle repeated 32 times: 768 episode draws,9,280 canonical answer/EOS
positions. Equal-nonempty-phase loss uses canonical prefixes during training.
One discarded 8-episode gradient admission is charged separately as preparation.
Checkpoints 24/48/72 are recovery only; checkpoint 96 is the sole final endpoint.
There is no intermediate score selection, constructor gate or CE-based cutoff.

The baseline must reproduce exactly the original 8 successful row indices.
Both baseline and final saved/reloaded native models are scored on the unchanged
full 512 panel, using actual emitted prefixes, 32 output tokens, 128 total admission,
eight retained H4 lanes and 4096 legal Generate actions. The subset report reads
those authenticated endpoint rows without another inference pass.

`qualified_fit` requires 24/24 complete replies with EOS, including all original 8.
`model_keep` separately requires all original 8 retained and at least one new
complete reply on 512; partial gain is not full subset qualification. Teacher
all-token correctness, entry/EOS, complete swap pairs and gained/lost IDs are
reported separately. The milestone remains 256/512 followed by fresh >=40%; no
held-out draw is part of this experiment.

## Outcome decision

Before opening the result PR: if the result came out the other way, would the
next step differ? Yes. A qualified fit supports a larger fixed qualification;
partial retained gains support preserving that artifact without claiming 24/24;
a regression/no gain rejects this candidate and stops this recipe without an
unchanged dose/rate/seed sweep. Failure is not a proof of all-family incapacity.
No successor starts before protected delivery and cleanup.

## Resources and validation

Full projection 120 min includes recovery, source, bootstrap/build/tests, model,
controls, full evaluation, retries, review, preservation, merge and hygiene.
A single RTX5090 is leased through `uor-pod` at $1.19/h; model 2 CPU threads and
build 4 threads (stock bootstrap 15 separately accounted). Model RAM <=6 GiB and
GPU <=4 GiB; task pod transient <=16 GiB. Local compilation is disabled to preserve
30 GiB + 128 MiB disk floor; local transient <=768 MiB, retained compressed <=512 MiB.
Standing same-class 60 min ledger extension changes limit 1474200028→1477800028ms,
retaining prior cumulative 1467760243 ms. Full elapsed session starts 04:40 UTC.

Exact source review at 30c7d018825dcd4b3f2d2e7791af66ab4c192635 passed with no
blocking finding. It covers strict schedule/ID admission, legacy 64 preservation,
active-only clipping/frozen families, saved native reload and full 512 scoring
before the subset decision. Optimized CUDA build passed in 232.170 s (peak 3,576,160 KiB); all 6 focused tests
passed with 0 ignored in 25.234 s command time. Bootstrap separately took 200 s
(build 132 s, parity 52 s: 37 pass / 3 ignored). Producer executable SHA256
`71fe01d312bd37549c68a03d9d54d01bf063817fc5c8b67aebe4b06f88a27713`.
The measured endpoint follows below; checks alone are not a result.

A concurrent local storage decrease crossed the floor before model launch. Hash-verified
redundant restored cloud components were removed, and Git sparse checkout omits only
the tracked research/ tree from this owned worktree; full native source/current docs
remain present and omitted tracked material remains on main. This routine reversible
workspace adjustment restores the floor without touching another lab's material.
The admission hygiene pass completed and preserved other labs' active folders.


## Result: REJECT; the exposed subset did not fit

**Line: ordinary reply-completion gradient learning · count 2/3 · M2 headline
8/512 → 8/512 (candidate 8/512 → 2/512).** This inherits the first negative in
#2141. Zero preparation PRs; one integrated model-result delivery.

| Measurement | Accepted parent | Saved update 96 |
| --- | ---: | ---: |
| Complete correct replies on unchanged panel | 8/512 | 2/512 |
| Complete correct replies on trained subset | 8/24 | 2/24 |
| Trained replies with every teacher-prefix token correct | 8/24 | 2/24 |
| Correct opening token on trained subset | 8/24 | 8/24 |
| Complete source-swap pairs on full panel | 4/256 | 0/256 |
| Any EOS emitted on full panel | 11/512 | 3/512 |
| Equal-episode native teacher-prefix CE on full panel | 6.1992635176344875 | 5.591268740946919 |

The candidate retains two original successes, loses six and gains none. The two
survivors are the same forward-home, swap1, q0/q3 rows as the rejected #2141
candidate. Against either prior artifact, 510/512 generated sequences change;
against #2141 the complete-correct row set is identical. All 16 newly supervised
subset rows still miss the opening token. Length4, length8, update and reassert
strata each remain 0/4 complete, including under all-token teacher-prefix
correctness. The [saved-row summary](result-summary.json) retains every selected
row, gained/lost ID, and stratum count.

**Interpretation:** this fixed Potential/Generate-only recipe did not establish
24-example learnability after 32 exposures each. The failure is already present
with canonical prefixes and at the entry of every new selected example, so an
explanation consisting only of accumulated free-generation errors is insufficient.
Lower CE does not qualify the model. This does not prove that all 512 answers are
unrealizable by the native family, isolate a specific representation defect, or
exclude a different training intervention. The selected examples are exposed,
correlated bank00 rows, not independent held-out trials.

**Decision and next:** reject and preserve update96, retain the accepted8/512
parent, and stop this concentrated Potential/Generate-only recipe. Do not expand
this curriculum or repeat an unchanged dose/rate/seed sweep: its fit prerequisite
failed. Any next M2 run needs a distinct model-changing intervention and a new
pre-registration; a larger panel alone is not justified. No successor compute
starts in this delivery, and no closed constructor/attribution/solver line reopens.
M3, held-out performance, useful conversation and energy claims are unchanged.

**Counterfactual before opening the PR:** yes, the next step would differ if the
result came out the other way. A retained 24/24 fit would support a larger fixed
qualification; a retained partial full-panel gain would preserve that candidate.
This measured regression instead stops the recipe and rules out that expansion.

## Executed checks and cost

The saved-row reader authenticates all 1,024 row hashes and exact IDs, verifies
own-prefix feedback/chosen-token chains, EOS and frozen answer membership, checks
the fixed 24 ×32 schedule and 9,280 target positions, and reproduces both decisions.
The native producer independently reloads its saved endpoint and seals/verifies
the complete report. Reader verification does not independently rerun native
scores, backward gradients or tokenizer decoding.

The model process exits zero after 1,986.435 s (33.11 min), with peak RSS
1,437,044 KiB. Baseline scoring takes 190.889 s; fitting including checkpoints
1,544.861 s; final scoring 224.130 s. The 10-second GPU samples reach 4,118 MiB,
22 MiB above the 4 GiB projection; this small observed estimate overrun is disclosed,
not a claim of compliance with that estimate. It stays within the leased device
and introduces no new spending class. [Measurement receipt](measurement-summary.json)
separates process time from model phase timings; these are not serving benchmarks.

Report SHA256 `16d03a31fc1eb27ee270cec96a7d388c108522c79b2a87856a0ca94b232aab68`;
manifest SHA256 `987bc137e5876821aef3b80777986c40bf3808f90203a46060fd135bb1e53720`.
The exact producer identity and six executed focused CUDA-feature tests are above.
Final-head review/checks and whole-session elapsed cost are recorded on the result
PR and #2030 after delivery; producer and delivery identities remain distinct.


## Durable evidence

The full sealed endpoints, all recovery/final checkpoints, executable, telemetry,
exact parent/panel, source bundle and preparation receipts are packaged as
`codex-m2-stratified-reply-20261010` in `icloud:UOR-R4/results/codex/`.
[Package hashes](PACKAGE.json) bind each compressed component. Restore through
`cloud-store fetch codex-m2-stratified-reply-20261010 <destination>`, then extract
the run/input archives and remap the config paths. The source bundle requires
published main commits `0d1314053` and `f01e91230`. Never reuse a sealed report root.

Local free space fell below the 30 GiB floor during the remote run; the required
hygiene pass preserved other labs' active work and freed no material space. To
avoid local archive staging, this package is streamed from the pod through
`uor-pod ssh` into the existing iCloud remote, then downloaded through a pipe for
full MD5 verification before publishing the standard six-column storage index.
This is a transport fallback equivalent to the `cloud-store put` integrity/index
contract; no result tar is staged locally and `cloud-store fetch` stays compatible.
The [preservation receipt](cloud-put.txt) records bytes, digest and verification.
No other lab's work was removed. Final cleanup hygiene and remaining free space
are reported honestly on #2030, separate from the measured result.
