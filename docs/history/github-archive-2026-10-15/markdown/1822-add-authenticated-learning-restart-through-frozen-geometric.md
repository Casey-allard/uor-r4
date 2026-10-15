# #1822 — Add authenticated learning restart through frozen geometric transport

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-07T02:32:09Z
- **Updated:** 2026-10-09T13:17:15Z
- **Closed:** 2026-10-07T05:22:31Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1822
- **Merged:** 2026-10-07T05:22:31Z
- **Merge commit:** 068d6f2c9903a65380c0ec80a1fa3f6b0b395fd8
- **Base / Head:** main / UOR-Foundation:codex/frozen-map-restart-20261007
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The original quarter bridge loses selected-source influence, while the retained categorical export preserves it without yet producing correct predictions. This adds an authenticated Rust/CUDA restart fitter to test whether learning context, potential and Generate through that frozen source-sensitive map improves native own-prefix predictions.

Lab: Codex. References #820, following the measured transport diagnosis in #1819. Based directly on main; one new example. Both credit arms restore checkpoint-0128, use fresh Adam moments and identical per-seed schedules, and differ only in Clipped versus RawIdentity vocabulary score adjoint. Original bridge masters and categorical marker weights stay frozen and outside optimization/clipping. Three seeds vary data order, not pretrained initialization. Training and development are the same declared 512-row construction panel; no held-out or general-chat claim.

Admission verifies original master/export reproduction, categorical reconstruction, independent native state/action/token-mass parity and identical initial forward losses. Checkpoints independently reload saved native/source/Generate artifacts. Evaluation retains actual own-prefix outputs and separates teacher-prefix, source-swap, correct-prefix reach and later Copy measurements. CUDA performs autodiff learning; the integer native evaluator runs on the pod CPU.

Optional initial-baseline reuse verifies the complete sealed compatible producer, all initialized source/native/exp/cue/prefix/decoder/master/input identities, all 512 row hashes and reproduced metrics. Producer and consumer identities stay explicit. Each fit still runs its own seed-specific graph/native admission and independent final native evaluation.

Validation: independent exact-source review cleared current head 7fbfc940ce4abfb056e0a094bf82d1df5d9bc9ae. Formatting, release CUDA-feature package check, all seven focused/support tests and the release build passed with GPU masked on shared pod CPU. Earlier f63 head passed five tests and actual zero-update CUDA admission (119 positions), plus full 512-row integer native evaluation in 242 seconds. Initial entry and complete answers both 0/512; no efficacy conclusion. Current-head seed1001/clipped passed its own 119-position zero-update native/graph checks and authenticated baseline reuse, and has completed two CUDA updates with finite gradients. All six matched fits are running sequentially; final fitted evaluations are pending. Command/exit/artifact receipts are linked in PR comments on canonical /workspace.


## Timeline (5 entries; chronological: conversation comments, reviews, review comments)

### comment 6029708431 — 2026-10-07T02:36:07Z — Casey-allard

Exact-head checks at f63ed81fd4f74d2501ba3d9888420662492416df completed on shared Linux x86_64 pod ianty3o30vhgew, GPUs masked, Rust1.97.1/CUDA12.8, four CPU build threads, clean committed tree. Touched rustfmt PASS; release CUDA-feature touched-package check PASS55.36s; five focused/support tests PASS5/0 (phase weighting, seeded complete schedule, tiny-gradient clipping, output ancestry/resolution); release CUDA example build PASS5.31s after test compilation2m52s. Existing imported support dead-code warning is not a test failure.

Executable /workspace/bin/f63ed81fd4f74d2501ba3d9888420662492416df-sm120/geometric-frozen-map-fit SHA25603efbdd897f49676094a1d0372edcd4a52deda99819781d4cb48b54f8f4bb013. Source bundle SHA25660cdc331e1bbd6defaddc56285b9f70acb7e7a69283fd2bdba6e1108585ad8d1. Command/log/exit receipts on canonical /workspace/uor-r4/codex/frozen-map-restart-f63ed81fd4f74d2501ba3d9888420662492416df/. CPU marker removed; /root/codex3.2GiB. No laptop compilation or large download.

Independent adversarial source review cleared unchanged source. Actual CUDA zero-update admission remains NOT_RUN; allocating one standard5090 now, then inspect the retained report before fitting. This is compile/component evidence, not language qualification.


### comment 6029785019 — 2026-10-07T02:42:54Z — Casey-allard

ACTUAL ZERO-UPDATE ADMISSION COMPLETED at f63ed81fd4f74d2501ba3d9888420662492416df: pod0ackgvv9i5htnb RTX5090/CUDA12.8, job02:37:20–02:41:25Z exit0; driver elapsed242.257269093s. Report /workspace/uor-r4/codex/frozen-map-restart-plan/admission-attempt1/report.json SHA25601c19eb1226293353eefa95fbd23ec5c89874f2bc10536ed3b10335759ac9aed. Driver sealed and verified full report. Native/graph parity and identical clipped/raw initial forward loss PASS across119positions in8seed1001 rows. Original masters, original quarter export, categorical frozen map and independently reloaded native artifacts reproduced. No updates.

Complete initial integer own-prefix development evaluation:512rows/256swap pairs; first-token correct0/512; first-divergent teacher-prefix correct40/512; both teacher-prefix arms correct1/256; correct own-prefix reach0/512; four positional token matches do NOT count as successful continuation. Later Copy-covered teacher-prefix correct351/4128. Training=development authored construction panel, no held-out/general-chat/attention efficacy. CUDA executes graph admission; integer serving evaluation executes CPU on the pod. Independent actual-evidence review underway.

Released02:42:01Z/deleted02:42:26Z once terminal; canonical volume retains all evidence. Creation-to-deletion388s at$0.99/h ≈$0.1067 estimate, not invoice. No current Codex GPU lease. Narrow authenticated initial-baseline reuse will save six duplicate ~4min native evaluations; same seed-specific graph checks and all final native evaluations retained. Proceed to three matched order-seed learning pairs after revised code checks. No unchanged quarter repeat or source-carrier rewrite.


### comment 6029829760 — 2026-10-07T02:46:43Z — Casey-allard

Exact-head source review at7fbfc940ce4abfb056e0a094bf82d1df5d9bc9ae (fileSHAc4ffff33b3f95a67ac603f1491098e84f1c4523659382f36ee4a9204841dd709): optional completed-baseline reuse independently reviewed and cleared. Both root and critic identified the missing cue/prefix tree equality; fixed before commit. Seal verification, explicit compatible producerf63, identical initialized source/native/exp/cue/prefix/Generate/bridge/master/data identities, complete512ordered rows/hashes/labels and reproduced metrics required. Copied evidence is self-contained and producer/consumer identities recorded separately. Native step/evaluation semantics unchanged; each fit still executes its own seed-specific native/graph/scalar admission, 128updates and fresh final evaluation.

Independent actual baseline reduction checked all554manifest file paths/sizes and512row SHA references, unique IDs, canonical targets, own-prefix step feedback,256pair booleans and independently decoded all512 own outputs. Reproduced entry0, teacher source40, positionalsource4 but qualified own-source0, Copy351/4128, complete0. Full BLAKE3 verifier is executed by Rust producer and reuse; reviewer standalone BLAKE3 unavailable, not claimed. Current head release checks/build running CPU-only, GPU masked; no active Codex GPU. Baseline inference is reused evidence, not a newly executed result per seed. No altered fit data/rates/order/dose or source-key supervision.


### comment 6029868798 — 2026-10-07T02:49:40Z — Casey-allard

Exact-head scoped checks and source self-review at7fbfc940ce4abfb056e0a094bf82d1df5d9bc9ae: touched rustfmt PASS; CUDA-feature touched-package release check PASS20.30s; all7focused/support tests PASS7/0; release CUDA example build PASS5.57s after1m11s test compilation. Shared Linux x86_64 CPU host, GPU masked, own target, threads4; clean source tree. Independent repaired-source review PASS fileSHAc4ffff33…dd709. Reviewed actual forward native/graph admission at compatible producerf63; native_step/evaluate and model/learning semantics unchanged in optionalreuse patch. Current-head reuse execution/fits still NOT_RUN, new5090allocation starting only after executable ready.

Binary SHA25652d4d5d037a5a7f392d992cd1b6ab630f427e8a5a4e7104580462523f4bde2bd; sourcebundle SHA256767d5aca8e0867195a72316267e506cae5ba2e649bfb0e76e46e9168bf71b5a2. Canonical command/environment/exit/log receipts /workspace/uor-r4/codex/frozen-map-restart-7fbfc940ce4abfb056e0a094bf82d1df5d9bc9ae/. /root/codex3.2GiB; CPU marker removed. No laptop build/large artifact download. Fit completion—not these checks—will supply efficacy evidence.


### comment 6031531565 — 2026-10-07T05:21:38Z — Casey-allard

Exact-head delivery review at `7fbfc940ce4abfb056e0a094bf82d1df5d9bc9ae`: this direct-main PR adds one authenticated frozen-map restart example. Independent source review, touched formatting, release CUDA-feature package check, seven focused/support tests and release build passed at this exact head (receipts in the preceding review comment). The current executable SHA256 is `52d4d5d037a5a7f392d992cd1b6ab630f427e8a5a4e7104580462523f4bde2bd`.

The exact current-head executable has now admitted and completed seed1001/clipped and seed1001/raw_identity: all 128 CUDA updates per arm completed with finite gradients, sealed reports and independent native evaluation; baseline-reuse and 119-position native/graph admission executed successfully. Both final artifacts remain 0/512 own-prefix entry and 0/512 complete replies; the instrument is delivered without a capability claim. Seed1002/clipped is progressing on the same frozen executable, with all six arms preserved. Merging the instrument does not modify that running executable or the declared experiment.

Main has only added an unrelated bf16 report document since this branch's base. No source overlap. Focused source review covers frozen bridge/marker exclusion from optimization and clipping, initialized-artifact identity, sealed baseline reuse, seed-specific admission and independent final export/reload. Ready for the protected merge queue; no admin bypass or fabricated CI result.



---
_Comment/review ids in this file: 6029708431, 6029785019, 6029829760, 6029868798, 6031531565_

