# #1844 — Expose native prediction and source-swap learning in the joint fitter

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-07T15:05:57Z
- **Updated:** 2026-10-09T13:17:41Z
- **Closed:** 2026-10-07T17:03:51Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1844
- **Merged:** 2026-10-07T17:03:51Z
- **Merge commit:** 84459eaf62361944c8450bd1edac4e1eaefa53d9
- **Base / Head:** main / UOR-Foundation:codex/native-prediction-control-20261007
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The joint geometric fits change conditional source choices but still fail at the first token of every complete native reply. Add a construction control to the existing Rust learner that admits eight fixed source-swap cases, trains Context/Potential/Generate/prototype parameters jointly, and exports, reloads and evaluates actual native own-prefix replies every eight updates.

The control retains all 4,096 Generate tokens and every physical Copy occurrence. Entry diagnostics compare the authentic mixed pool with a diagnostic Generate-only reduction, report native code-cell movement and bind the source/state artifacts. The success flag requires all eight complete accepted replies with normal EOS and all four source-swap pairs. These are exposed construction cases; this does not qualify unseen generalization or chat.

Fixed fit/admission configurations cannot accept control settings and retain their previous behavior. No forced opening, supplied selected record, reply template or runtime teacher is added.

Validation at 3fc63bec66e03a3148c83c48c795667582d258c2: independent source review READY; touched-file formatting and diff check; exact-head CUDA package/example check passed on Linux x86_64, GPU masked. Focused example tests and CUDA construction execution are in progress and will be posted before any queue request.

References #820.


## Timeline (7 entries; chronological: conversation comments, reviews, review comments)

### comment 6041162521 — 2026-10-07T15:31:49Z — Casey-allard

Exact-head validation at3fc63bec: touched-file rustfmt and diffcheck PASS; cargo check --release --offline -p uor-r4-training --features cuda --lib --example geometric-frozen-map-fit PASS; all9 focused example tests PASS, zero ignored; optimized executable SHAe96291a0. Linux x86_64, CUDA12.8/sm120,2 buildthreads/GPUmasked. Initial0-update actual-artifact admission and independent native bank/Generate/bridge/pool parity passed on84 positions, fullRust report seal verified; generated entry/complete0/8 and Generate-only entry0/8, not a capability positive.32-update joint learning nowrunning with actual native checks every8. No queue/merge request until result review.

### comment 6041513938 — 2026-10-07T15:50:28Z — Casey-allard

Continuation at commit051e44a639e806abd98cf74c7a83dd054029f105: authenticate the completed32-update native control and resume its exact native model with fixed Context/Potential/prototypes, training Generate unary/pair/bias only through the full84-position complete alias objective. Checkpoint0000 byte identity and frozen master identities are asserted. This changes offline learning; it adds no serving semantic gate or canned reply. Independent source-only review found no blocker; touched rustfmt and diff check passed. Exact-head package compilation, nine focused example tests and CUDA model execution are pending.

Prior3fc63bec control32 completed348.948seconds, passed independent full-file seal/row identity review, and remains entry/complete/EOS0/8. All84canonical states are distinct; this is not a capacity or transfer claim. The field block resumes rather than repeats those updates. Work card: https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-6041480316


### comment 6041632369 — 2026-10-07T15:56:42Z — Casey-allard

Exact head051e44a639e806abd98cf74c7a83dd054029f105 — focused verification and source review

Touched-file rustfmt --check and git diff --check: PASS. On Runpod Linux x86_64, clean git archive of this committed/pushed head, CUDA12.8 SM120, GPU masked for compilation and two build threads:
- cargo check --release --offline -p uor-r4-training --features cuda --lib --example geometric-frozen-map-fit: exit0,34.036s.
- cargo test --release --offline -p uor-r4-training --features cuda --example geometric-frozen-map-fit -- --test-threads=1: exit0,9passed0failed0ignored,191.696s including compilation. These are focused CPU tests of this CUDA-enabled source, not execution of a CUDA parity suite.
- cargo build --release --offline -p uor-r4-training --features cuda --example geometric-frozen-map-fit: exit0,5.392s.

Release binary SHA256dc4f535fbc47b0587f2446a2f58fdf580bdf0524ee348017d9ff23a9d0e6f24a (36,078,424bytes). Independent source-only review found no blocker in authenticated resume, exact initial component reproduction or coefficient-only update/frozen-master guards. Complete84-position fullalias learning and unmasked native actual-feedback are retained. New actual CUDA learning/exact native resume execution is running; no prediction success asserted and no merge queue requested yet.


### comment 6041886360 — 2026-10-07T16:10:33Z — Casey-allard

Exact-head verification b451571aa638d1f93e3ccf25f718f27ed2ea731f

Source replay matches the committed Git tree:17,090Git blobs/LFS objects verified, including1,824expanded LFS assets by declaredSHA256+size. Reused exact051earchive with only the newly committed fitter substituted, preserving cache paths; no source-code changes outside this head. FitterSHA256cf0648deebc3468d2034ade92c02ba732ec5fd92c5e6d5915895fd687e5f4dc4. Independent source review READY for provenance repair and Potential+Generate-only optimizer ownership.

On Runpod Linuxx86_640j24j90eakf14v, CUDA12.8SM120, GPUmasked2threads:
- cargo check --release --offline -p uor-r4-training --features cuda --lib --example geometric-frozen-map-fit: exit0,9.343s.
- cargo test --release --offline -p uor-r4-training --features cuda --example geometric-frozen-map-fit -- --test-threads=1: exit0,9passed0failed0ignored,60.657s including compilation.
- cargo build --release --offline -p uor-r4-training --features cuda --example geometric-frozen-map-fit: exit0,5.387s.
Touched-file rustfmt and diffcheckPASS. These tests executed the focused CPU instrument contracts in a CUDA-enabled build; CUDA model execution is separately underway.

BinarySHA2560509b23b46a3a6f5024e631d8a8edd20f692e5e1819429041041e1113e362567. ConfigSHA2560a8ac0472ee713042c60411d588d45eb71222c94c2ac091dc89327b08529b5a6. Actual64-update Potential+Generate CUDA learning is launched from exactsaved3fc32; no success verdict or merge queue request yet. The priorfield-onlyconfiguration is withdrawn as a perfect-replytest after the11position native alias-mass exclusion certificate. Its051eattempt naturallyfailedbeforetraining dueprovenanceguard, notmodelquality. Workcard https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-6041787346.


### comment 6042520105 — 2026-10-07T16:45:29Z — Casey-allard

Exact-head self-review at503d64b39aaaa769b60e3c2e1ce30dc5ec2b5696 (fitterSHAd5665cc643b03ad8c8f392d8600341634360dba488f891b54821879c2805e6da): source review READY by nonauthor critic; no blocking issue, execution distinct. Touched rustfmt--check and gitdiff--check PASS. On Runpod Linuxx86_64 RTX5090 pod0j24j90eakf14v, GPUmasked for compilation: releaseoffline trainingpackage/lib/fitter check exit0(9.727s), focusedfittertests9PASS0FAIL0ignored exit0(65.294s inclcompile), releaseCUDAfeaturefitterbuild exit0(5.318s). All17,090sourceGitblobs/expandedLFS contents verifiedagainstexacthead; only committedfitter changedfrompriorfullyverifiedarchive. Sharedowncache,2CPUthreads. BinarySHA74764ca62f96f4e44b9a73260b0a467af69033be069a445a3ea5447f905a036a; durable/workspace/bin/503d64b39aaaa769b60e3c2e1ce30dc5ec2b5696-sm120-native-prediction-control/. CUDA execution for the zero-update recomposition is next; CPUfocusedtestsarenotCUDAparity. Source48+Generate64 recomposition authenticates bothcomponents, frozenContext/prototypes/sidecars, save/reload; exposeddevelopmentselection only. PR remainsdraft until actualboundarycheck.


### comment 6042584164 — 2026-10-07T16:48:32Z — Casey-allard

Concrete native prediction construction win at PR #1844 head503d64b39aaaa769b60e3c2e1ce30dc5ec2b5696: authenticated zero-update Source/Potential48+Generate64 recomposition completed exit0 in52.2935s. Independently saved/reloaded coherent native artifact produced8/8 complete accepted own-prefix replies withEOS, all8 openings,44/44 canonical Copy positions and4/4 matched source-swap divergence pairs. Full4096Generate vocabulary and11physicalSource candidates remain; no answer mask, forced opening/period/Stop, suppliedselectedrecord or additional training. Both facts are available in each prompt, and swapping their assigned roles changes the actual emitted answer. Exact reportSHAa1277d2be4ff962100f857d0da553257ad5596ac8bc72d9257299e74d2f2e278, manifestSHAb37e2ca588bf1cc4dc5971fa5da97b9e83c90a94f4ea0bfdb0655b8f92bf94ca; binarySHA74764ca62f96f4e44b9a73260b0a467af69033be069a445a3ea5447f905a036a.

Scope: eight exposed development constructions from one learned lineage, component choice informed by development labels. This is a working learned native prediction loop for this panel, not heldout/generalchat/reasoning or a newgeometryadvantage claim. Independent whole-seal/row/output audit and canonical preservation in progress. The Source48/Gen64 intervention supports an emission-energy calibration obstruction: priorSource48 full4/8 and jointterminal64 full1/8, despite identical selected Source/transported state at all84 canonical positions. Prediction should qualify fresh source combinations/paraphrases next before broader higher-dimensional recursive-read work. No new fit is started here.


### comment 6042649238 — 2026-10-07T16:51:56Z — Casey-allard

Exact-head self-review complete at 503d64b39aaaa769b60e3c2e1ce30dc5ec2b5696. Touched formatting/diff check, release CUDA-feature touched-package/library/example check, focused 9/9 tests and build all PASS with actual commands/exits in comment6042520105. The actual leased RTX5090 zero-update CUDA/native84-position recomposition and native8case own-prefix generation completed exit0 in52.2935s:8/8 complete+EOS,4/4matched source-swap pairs,84/84canonical predictions. Independent reviewer verified whole Rust report seal, all exact component bytes against Source48 and Generate64, every empty-prefix prompt and generated-prefix sequence, full emitted outputs/accepted membership and EOS. No source changes after checks. Model artifact scope is explicitly eight exposed development cases, selected component mix, zero new optimization; no generalization or runtime performance qualification. ReportSHAa1277d2be4ff962100f857d0da553257ad5596ac8bc72d9257299e74d2f2e278; manifestSHAb37e2ca588bf1cc4dc5971fa5da97b9e83c90a94f4ea0bfdb0655b8f92bf94ca. Canonical copy exists with matching report/manifest hashes; full archival closeout in progress. No recoverable error guard was bypassed. Source admits only authenticated intended producers/component steps; original fitter mode unaffected.



---
_Comment/review ids in this file: 6041162521, 6041513938, 6041632369, 6041886360, 6042520105, 6042584164, 6042649238_

