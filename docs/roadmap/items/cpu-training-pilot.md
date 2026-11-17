---
id: cpu-training-pilot
title: Run and grade the bounded CPU training pilot
milestone: research
area: training
status: ready
open_to: lab
needs_gpu: false
depends_on: [cpu-discrete-training-path, cpu-shared-vector-train, cpu-quantization-exact-memory, cpu-whole-path-exact-updates]
notebook: docs/research/notebook/cpu-training.md
done_class: host
scope: [crates/uor-r4-cpu-train/**, crates/uor-r4-integer/Cargo.toml, crates/uor-r4-integer/README.md, crates/uor-r4-integer/src/bin/uor-chat.rs, crates/uor-r4-integer/src/openai.rs, crates/uor-r4-integer/src/session_codec.rs, crates/uor-r4-integer/src/stack/chat.rs, crates/uor-r4-integer/src/stack/content_readout.rs, crates/uor-r4-integer/src/stack/mod.rs, crates/uor-r4-integer/src/stack/session.rs, crates/uor-r4-integer/tests/openai_loopback.rs, crates/uor-r4-integer/tests/session_codec_interop_fixture.rs, crates/uor-r4-core/Cargo.toml, crates/uor-r4-core/src/native_support.rs, crates/uor-r4-discrete/Cargo.toml, Cargo.toml, Cargo.lock, docs/harness/claims.json, docs/harness/prep/cpu-training-pilot*, docs/integration/current-state.md, docs/labs/cpu-training-pilot-*/resource-ledger.json, docs/labs/cpu-training-pilot-content-serving-2026-11-04/**, docs/labs/cpu-training-pilot-trajectory-density-2026-11-04/**, docs/labs/cpu-training-pilot-copy-key-depth-2026-11-05/**, docs/labs/cpu-training-pilot-copy-key-reply-2026-11-05/**, docs/labs/cpu-training-pilot-copy-termination-oracle-2026-11-05/**, docs/labs/cpu-training-pilot-copy-generated-stop-2026-11-05/**, docs/loop/tasks/cpu-training-pilot/**, docs/research/cpu-training-program.md, docs/research/notebook/cpu-training.md, docs/roadmap/items/cpu-training-pilot.md]
---

## Why it matters

Component speed does not show whether CPU training can improve a model at useful scale. The workstation must be measured under a bounded, pre-registered pilot with the same evaluator as the GPU path.

## Readiness as of 3 November 2026

The bounded public internal-development pilot has executed and failed its registered ranking bar. The full item remains open because genuinely fresh complete-reply inputs and a configuration that clears a registered bar are still unavailable. Public 64→256 interval restart is now deterministic on both seeds.

| Prerequisite | Current state |
|---|---|
| Shared-vector forward and exact serving arithmetic | Supported for the recurrence-only/no-pointer FBS mode, including snapped and raw gate/transition combinations; synthetic and public trained-artifact CPU/serving parity controls pass. The CPU Hamming port now also executes copy key 2 with the integer session's ordered two-token query/key and reports the key in the audit manifest. See [deepseek-87](../../loop/tasks/cpu-training-pilot/deepseek-87/REPORT.md). |
| Exact served-forward contract and workload audit | Delivered for the same supported FBS mode. `cpu-served-forward-audit` binds the artifact, reports active mechanisms, compares every Q12 query/token/kept-cell decision, separates modelled work from measured CPU timing, and inventories float-only gradient treatments. Both public raw seeds pass a 64-position complete-decision control. A follow-up corrected the contract's Hamming-range narrative and added independent integer/CPU/float boundary tests; the kernels were already causal and correct. A composed control exercises Hamming stream/readout reads, opening and reply memory and point-key fallback with exact parity. [Deepseek-91](../../loop/tasks/cpu-training-pilot/deepseek-91/REPORT.md) now trains the writer, selects a Hamming margin by exact served score, derives opening/reply tables and a point row from examples, writes a 536,516-byte full-path artifact, reads it back and passes 704 Q12 coordinates plus 11 complete decisions with zero mismatches. [Deepseek-92](../../loop/tasks/cpu-training-pilot/deepseek-92/REPORT.md) adds full, memory-off, point-off, Hamming-off and stripped arms; every arm preserves exact 704-coordinate/11-decision/88-cell parity while removing its named records. [Deepseek-93](../../loop/tasks/cpu-training-pilot/deepseek-93/REPORT.md) adds a Q24 coordinate surrogate that trains opening and reply movement weights by reparsing the exported artifact and scoring complete served queries. Both finite arms lowered target distance with exact parity, but top-1 stayed unchanged. The initial trunk and training sample are finite; corpus-scale held-out comparison remains unavailable. |
| Generated-content serving override | `IntegerStackModel::serve_step` can consult an optional artifact-bound `uor-r4.stack-content-readout/1` sidecar after the native writer or point-key fallback. Two public seeds matched all **3,296** generated decisions with zero token, fallback, signature, distance, firing or decision mismatches; the deciding attempt-5 run exercised prompt-ID, one-shot, REPL and OpenAI-loopback paths and reports the exact sidecar path/hash in JSON. The selector is allocation-free after session construction and retains **94.44%** of no-readout single-thread generation throughput. Raw `step` and `step_native`, the `UORLUT01` format and the positive content quality are unchanged. See [deepseek-140](../../loop/tasks/cpu-training-pilot/deepseek-140/REPORT.md). |
| Candidate artifact gate | `scripts/audit_served_forward_gate.py` binds a `UORT` token slice and optional expected artifact hash, validates the complete deterministic audit, and atomically promotes only exact-passing artifacts. The exact CPU corpus driver now runs the same audit internally before final promotion, validates all workload and mismatch fields, persists `final.audit.json`, and reloads both outputs. Public seed 1 passed a zero-step 64-position smoke with zero mismatches over 16,384 Q12 coordinates. Studio trained-model staging now requires a locally validated receipt and publishes it beside the model; remote cards pin the artifact and receipt; the release-bundle packager requires adjacent receipts for every UORLUT. A separate read-only gate now checks that every token's two 8-coordinate writer ports survive Q12 export/read-back at its cell roots; a synthetic positive passes and both public unanchored artifacts are rejected as expected. The synthetic demo and arbitrary public-URL slot remain outside the trained-release gate, and automatic workflow publication/receipt signing are still open. See [deepseek-86](../../loop/tasks/cpu-training-pilot/deepseek-86/REPORT.md), [deepseek-94](../../loop/tasks/cpu-training-pilot/deepseek-94/REPORT.md) and [deepseek-181](../../loop/tasks/cpu-training-pilot/deepseek-181/REPORT.md). |
| Tied-table and lawful parameter-group trainer | Tied-token and BVG-K normalization/bias/decay/route groups delivered on finite fixtures. The exact CPU corpus driver binds rank-margin, coordinate-step, parameter-limit and coordinate-count settings into checkpoint identity. The two-seed 1/16/64-Q12 step comparison remained ranking-negative. The FIT-reviewed same-amplitude one-versus-four-coordinate pivot exercised 437/238 accepted four-coordinate blocks, reduced the margin objective on both seeds, and remained ranking-negative; seed 1 also lost admission and top-8. See [deepseek-9](../../loop/tasks/cpu-training-pilot/deepseek-9/REPORT.md) and [deepseek-82](../../loop/tasks/cpu-training-pilot/deepseek-82/REPORT.md). The repeated-key full-path memory comparison then trained eight coordinates with 16-thread candidate evaluation. Both seeds improved held-out target distance and top-1 over frozen and random controls with exact parity; the top-1 gains missed their three-sigma bars. See [deepseek-95](../../loop/tasks/cpu-training-pilot/deepseek-95/REPORT.md). A cached teacher-forced fast path then reproduced both runs exactly while parallelizing independent trace, evaluation and parity samples; walls fell to 8.774/8.397 s with zero artifact or reading changes. See [deepseek-96](../../loop/tasks/cpu-training-pilot/deepseek-96/REPORT.md). A final full-width gain trainer then accepted all eight updates on each seed and reduced held-out distance by 16,799,488/16,154,486 through the complete served query, but top-1 remained 291/243 and the registered ranking bar failed. See [deepseek-97](../../loop/tasks/cpu-training-pilot/deepseek-97/REPORT.md). |
| Output-row decoupling | Captured exact Q12 writer queries on 5,114 TRAIN targets and refit only target rows with at least eight observations on both read-free public FBS artifacts. The matched panel had 5,072 targets. Coordinate-wise median top-1/top-8 was 1,355/1,678 versus original 1,539/2,037 on seed 1 and 1,278/1,455 versus 1,239/1,472 on seed 2. The median beat the shuffled control on both seeds but lost top-8 on both, so the two-artifact bar rejected. The shared-table diagnostic changed 4,695/4,569 decisions and fell to 148/233 top-1, showing that retained output fitting without an input/output split is destructive. See [deepseek-237](../../loop/tasks/cpu-training-pilot/deepseek-237/REPORT.md). |
| Accepted b64L CPU export and local serving | Both Claude-301 b64L width-256 checkpoints now export without CUDA. The CPU converter reads the saved F32 tensors, reconstructs and checks all five SpiralCore/FBS projection plans, validates model/run/Hamming/tensor inventories, carries the retention-rate semantics, and writes the same snapshot and `UORLUT01` formats. Both seeds reproduced byte-for-byte. Each artifact passed a fresh 128-position `cpu-served-forward-audit` with zero mismatches over 32,768 Q12 coordinates, 128 tokens and 1,024 kept cells, then loaded through `fbs-toks` and `uor-chat`. Seed 1's `--say` repeat was exact. This closes the CPU export bottleneck and local integer-serving boundary, not full train-to-serve equivalence: 32-token float and integer replies still differed on 9/16 seed-1 and 4/16 seed-2 prompts, and the replies were repetitive rather than useful chat. See [deepseek-239](../../loop/tasks/cpu-training-pilot/deepseek-239/REPORT.md). |
| Ordered parallel evaluation | Panel rows and counterfactual candidate lanes now use bounded Rayon workers, with source-ordered integer reductions and serial coordinate/joint selection. Tests reproduce panel readings, accepted steps, artifacts and checkpoints exactly across 1, 2, 3, 5, 8, 16 and 32 workers. At 16 workers, the synthetic panel reached 90,010 rows/s (9.13x) and counterfactual groups reached 4,686-5,192 candidate evaluations/s (7.33-8.80x). Thirty-two workers were workload-dependent rather than uniformly faster. See [deepseek-98](../../loop/tasks/cpu-training-pilot/deepseek-98/REPORT.md). On the real public corpus, two reversed-order runs measured 62.15 s at one worker, 8.48 s at eight, 4.63 s at 16 and 3.53 s at 32, an average 17.62x gain with byte-identical artifacts, checkpoints, panels and audits. The update chain and recurrent audit are dependency-ordered; the gain is concentrated in exact panel evaluation. See [deepseek-100](../../loop/tasks/cpu-training-pilot/deepseek-100/REPORT.md) and its corrected [deepseek-101](../../loop/tasks/cpu-training-pilot/deepseek-101/REPORT.md) accounting. |
| CPU build graph | `uor-r4-discrete`'s build script now uses `uor-r4-core` with its default `integer-runtime` feature disabled. The build path therefore executes only the exact SpiralCore/prime-route metadata modules and does not compile a second feature set of `uor-r4-integer`. Ordinary `uor-r4-cpu-train` checks run in a shared target without the host/target `cdylib` collision or a fresh target directory. Normal core callers retain the full default API. See [deepseek-107](../../loop/tasks/cpu-training-pilot/deepseek-107/REPORT.md). |
| Counterfactual corpus comparison | A one-arm runner now applies the existing exact BVG-K counterfactual trainer to the public masked corpus and compares normalization, bias, decay and route groups with zero-update controls. On both seeds the fixed 128-step, four-coordinate, delta-one configurations had exact export but no replicated ranking gain: normalization lost admission on seed 1, decay accepted no update, bias stayed rank-neutral, and route sharply degraded the held-out panel. See [deepseek-11](../../loop/tasks/cpu-training-pilot/deepseek-11/REPORT.md). |
| Matched admission and exact checkpoint export control | Panel evaluator and gate delivered on finite fixtures. |
| Trained artifact plus internal query panel | Bound: public dual-c131 seeds 1 and 2 have exact CPU/serving parity; the strict corpus driver creates document-disjoint public development samples. The public 19-table M1 artifact now has a read-only CPU path through its trained Relation read and all 15 joint-pointer trunk tables: all 14,287 DEV rows pass with zero query, admitted-cell, top-8 or token mismatch and reproduce 9,012 admitted / 7,002 top-1 / 7,485 top-8. See [deepseek-90](../../loop/tasks/cpu-training-pilot/deepseek-90/REPORT.md). |
| Tested public 64→256 interval restart | Passed. A FIT-reviewed post-FIT control resumed hash-bound step-64 checkpoints in fresh processes to step 256 and produced byte-identical final artifact/checkpoint state on both public seeds. Other steps and crash consistency remain untested. |
| Sealed complete-reply and fresh-phrasing panels | Unavailable without credentials; `m2-fresh256` returns HTTP 401. A public 160-prompt panel supports only qualitative generation output, not correctness scoring. |
| Discrete reads/proposals | #3501 is done with finite parity and restart controls. |
| Discrete corpus driver and artifact writeback | A K=1 CPU driver now runs the public four-table artifacts over a strict masked corpus, preserves nonzero source epochs, emits reloadable artifacts and resumes byte-identically. A separate read-only CPU reader now covers the public 19-table M1 trunk with exact full-DEV parity, but training/writeback of its 15 extra tables remains unsupported. See [deepseek-4](../../loop/tasks/cpu-training-pilot/deepseek-4/REPORT.md) and [deepseek-90](../../loop/tasks/cpu-training-pilot/deepseek-90/REPORT.md). |
| Quantization-aware updates and trained float shadow | Finite integer/float shadow controls are delivered; a genuine trained FBS float snapshot is not published, so the trained comparison remains `NOT_RUN`. |
| FIT pre-registration and pre-run review | Complete. The public pilot sheet was FIT before compute and received a fresh non-author GO review. |

## Public pilot result

The pre-registered two-seed zero-versus-256-step run completed on the Ryzen 9 9950X. Exact CPU/serving export and target admission were preserved, but top-1 and top-8 did not move on either seed:

| Seed | Top-1 baseline → 256 steps | Top-8 baseline → 256 steps | Admitted baseline → 256 steps | Verdict |
|---|---:|---:|---:|---|
| 1 | 278 → 278 | 384 → 384 | 396 → 396 | fail registered gain bar |
| 2 | 224 → 224 | 285 → 285 | 292 → 292 | fail registered gain bar |

This is a negative for the tested public 256-step configuration, not a rejection of rank margin, FBS or CPU training generally.

The subsequent two-seed step-size comparison tested 1,024 attempts at 1, 16
and 64 Q12 units per accepted coordinate update. Unit-step seed 1 lost one
admitted target; step-16 lost one top-8; step-64 gained only 3 top-1 rows
against a 43-row bar. Seed 2 was unchanged through step-16, while step-64 lost
3 top-1 rows for a 1-row top-8 gain. Exact CPU/serving export stayed perfect
throughout. This is another bounded configuration negative and does not reject
all rank-margin or CPU credit assignments.

The same-amplitude BVG-K pivot then fixed every coordinate move at one Q12 unit
and compared one coordinate with four. K4 executed 437 four-coordinate blocks
on seed 1 and 238 on seed 2 with exact serving export. It reduced the held-out
margin-loss sum on both seeds, but top-1 did not move. Seed 1 lost one top-8 row
and one admitted target; seed 2 remained 224/285/292 top-1/top-8/admitted.
K4 therefore failed admission and ranking on seed 1 and did not exceed K1 on
either seed. This closes the tested coordinate-breadth configuration, not the
BVG-K mechanism generally.

The next materially different CPU comparison used the exact discrete
proposal/apply learner rather than rank margin. Baseline, K1 and K4 ran for 256
blocks on both public four-table seeds. Baseline read 38/108/361
top-1/top-8/admitted; K1 read 19/82/248 and K4 read 13/63/256. K4 committed 71
moves across 46 blocks, including fourteen blocks that committed more than one
coordinate, so the joint mechanism was exercised. Both trained settings lost
admission and ranking, and K4 trailed K1. This is a preserved negative for the
tested discrete K1/K4 budget, not a rejection of every proposal or credit
assignment. See [deepseek-88](../../loop/tasks/cpu-training-pilot/deepseek-88/REPORT.md).

The subsequent final served-gain comparison trained the full-width final FBS
normalization gain through the exact writer, memory and point path. Both seeds
accepted all eight updates and beat frozen and random distance with exact
five-arm parity, but top-1 stayed 291 and 243. This preserves a positive
bounded distance mechanism and a negative ranking/capability result; it closes
the tested one-code gain configuration rather than the geometric path.

## What is missing

**November 17, 2026.** D84 authorizes this CPU development pilot. Its grading, carry and checkpoint results are recorded below and in the linked experiment history. The grouped carry-hold audit merged in #3889. Its eight-checkpoint CPU comparison preserves 96 diagnostic values. The final disposition of the historical training families remains open. Milestone qualification still requires the selected GPU-scored model. [PR #3889](https://github.com/UOR-Foundation/uor-r4/pull/3889)

The bounded public pilot supplies cost, exact-export, admission and ranking
evidence, but it is not a fresh capability test. The full item still needs
credentialed sealed complete-reply and fresh-phrasing grading, a configuration
that clears a registered ranking or capability bar, and a corpus-trained
full-path model rather than the finite synthetic-initial-trunk control.

The detailed results are preserved in the [pilot experiment history](../records/cpu-training-pilot.md). The public two-seed checkpoint control graded 4,900 development rows and generated 32 bounded replies, all reaching the token cap with zero exact references. [Checkpoint grading report](../../loop/tasks/cpu-training-pilot/deepseek-240/REPORT.md)

## Done when

- The pre-registration exists before compute and names pass, fail and stop outcomes.
- The pilot runs on the named Ryzen 9 9950X and records its source, binary, hardware, OS and checksums.
- Training loss is never the only outcome; admission, top-1/top-8, exact export, complete replies and fresh phrasing are reported.
- Every historical training family has a final supported, superseded or rejected disposition linked to evidence.
- The REPORT states explicitly whether CPU training is useful for development, ready for a named qualification path, or rejected for the tested configurations.

## Cancel if

Any dependency still fails parity or deterministic resume. A fast but unverified trainer cannot enter the pilot.

## How to start

First collect the child REPORTs and rerun their parity/resume checks at the pilot head. Write the pre-registration with explicit pass, fail and stop outcomes and the machine identity. Import only the selected bounded corpus slice after the sheet is FIT.

## Rules that apply

- Pre-register before CPU research compute.
- Name the Ryzen 9 9950X and exact source/binary identity.
- Report capability and cost together; no loss-only success.

## Sources

- `docs/research/cpu-training-program.md`
- The six preceding CPU component items and their REPORTs
- The current evaluator and panel selected by the pre-registration
