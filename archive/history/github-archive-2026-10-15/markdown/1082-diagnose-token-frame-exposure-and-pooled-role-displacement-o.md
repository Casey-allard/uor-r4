# #1082 — Diagnose token-frame exposure and pooled-role displacement on the frozen construction views

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-09-03T03:54:47Z
- **Updated:** 2026-09-03T04:25:44Z
- **Closed:** 2026-09-03T04:25:09Z
- **Milestone:** Native Rust geometric intelligence — conversation, memory, reasoning and coding
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/1082
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Outcome and authorization
Execute the construction-only successor selected by #973's completion of #1079 (`LANGUAGE_R4_PRESERVED_CONTROL_WEAK`). Preserve the positive R4 result and the weak token-control result. This issue measures exposure and displacement; it does not claim a causal explanation from correlations.

## Fixed reference
Research base e627252e525201815169ffd8364184953a46018d. Reuse the exact #1079 reader/core, data and two existing construction renderings, native frames and current token-source-frame control. #1079 preparation CID blake3:d9c8ad8448365b2039276fdeda6b70da53ef63fde24e02dd1dd8dea437b546a4; result blake3:dee107190172afcb7637d52469662ecab217847271e4bbdb0721514fcfbdc3a5; replay blake3:eaa17433d5cd150a2a0c52adab6104bda4c4dae26221944fcde112ef841ca597. Exact source/reader/core identity checks precede model access.

## Measurements
For used roles, measure learned attention mass on positions with actually changed matrices, weighted sum of individual value displacement, and norm of the net pooled-role displacement. Define normalization, zero cases, mask semantics and units before execution. Relate them descriptively to the already recorded retained/changed supported answers, with no changed scoring population or decision threshold. Distinguish what is mathematically bounded from what is empirically observed.

## Freeze and scope
Publish one immutable preparation/implementation/input identity contract before reading diagnostic outcomes. No fitting, new parameters, alternative control, geometry expansion, new population, development scoring, generation or native export. Existing construction reference/control replay may only establish agreement with already recorded #1079 evidence. Do not search variants or tune on the revealed result.

## Run contract and cost
Metric: descriptive changed-frame attention exposure and value/pooled displacement for the same construction rows. Reachability: only positions/used roles with changed matrices can contribute; weighted triangle bounds and zero exposure cases must be explicit. Cheap instrument: validate exact artifact/control/frame identities and construction-only selection. Freeze concrete row counts, execution plan, peak-memory/disk budget and estimated runtime after source inspection, before outcome access. Prior complete #1079 run plus replay took about 57 seconds; this narrower diagnostic is budgeted at no more than 10 minutes per declared execution/replay, with a recoverable stop and no automatic retries.

## Decision and DoD
A valid result reports the frozen measurements and their limits; it informs a separately frozen successor contract. An identity/scope/bound violation stops with a typed unavailable/error result and no substantive diagnosis. Low exposure, vector cancellation and nonzero displacement with retained answers are descriptive patterns, not preselected winners. The report, machine-readable evidence and one independent reproducibility/identity check are the deliverable. Activate only the focused checks necessary to verify this diagnostic. #973 remains open and #954 remains blocked until its terminal is met.

## Ownership and next action
Casey-allard is actively executing this issue. Prepare the exact implementation contract in an isolated issue worktree, review it independently, freeze it, then perform the bounded run.

## Comments (2)

### comment 5520228872 — 2026-09-03T04:12:17Z — Casey-allard

## Frozen admission — token-frame exposure diagnostic

Preparation is complete; no retained-model diagnostic outcome has been read. The reviewed instrument is admitted for one execution and one fresh-process replay under the contract below.

- Source revision: `e627252e525201815169ffd8364184953a46018d`.
- Preparation CID: `blake3:c8a4a56de77767cbbe8fd31edc83251b42a6f729e182d065caa47d981f248741`.
- Implementation CID: `blake3:1451084b66531dbe5eb0cb1d3b3e60e5f0aa3f4bd59fffbf582a18d4dfc9380e`, binding 316 files; all historical 307 implementation files unchanged.
- Source #1079 preparation/result/replay CIDs: `blake3:d9c8ad8448365b2039276fdeda6b70da53ef63fde24e02dd1dd8dea437b546a4`, `blake3:dee107190172afcb7637d52469662ecab217847271e4bbdb0721514fcfbdc3a5`, `blake3:eaa17433d5cd150a2a0c52adab6104bda4c4dae26221944fcde112ef841ca597`.

Reuse the exact #1077 reader, #1073 core embedding, original frame bundle and two #1079 construction views. Each view contains 10,240 rows (8,192 supported and 2,048 unknown), 624,640 valid tokens and 143,360 used-role measurements. Preparation reproduced 471,638 and 461,246 actual changed source matrices; every supported row has at least one. No development tensor is decoded or scored.

For each of fourteen used roles, using original nonnegative f32 weights promoted to f64 without renormalizing displacement metrics: `M=sum a_i c_i` for actual matrix changes, `A=sum a_i ||delta_i||`, `D=||sum a_i delta_i||`; additionally record changed mass fraction, actual completed f32 role-vector displacement and coherent role norm. `D/A` is defined only when `A>0`; zero cases remain in the other summaries. All sixteen R4 blocks are included. Check weight sums within `1e-5`, `D<=A+1e-12(1+A)`, and pooled closure within `1e-12(1+max_abs_pool)`. Exact reconstruction must match the unchanged helper and #1079 reader-attention/coherent-role/control-role CIDs.

Summaries use the fixed quantiles 0, .1, .25, .5, .75, .9, 1, by role/view and supported/unknown × all/historically-changed/historically-retained answers. Historical predictions are consulted only after measurement. No downstream head/core projection, new answer prediction, fitting, parameter change, replacement control, generation, geometry expansion or development evaluation is admitted. Scope hooks reject forbidden forwards. Associations are descriptive and cannot establish a causal explanation or revise #1079's weak-control verdict.

Runtime: CPU/Apple Accelerate, four intra-op threads, one inter-op thread, one process, batch256, 80 reader batches/20,480 rows per phase. Each arm has307,200 reconstructed plus307,200 frozen-helper role vectors:1,228,800 pooled-vector evaluations per phase. Each execution and replay has a120-second external process timeout,3GiB cooperative peak-RSS limit and256MiB cooperative new-output limit. No automatic retry/budget renewal follows a stop. Binary metrics have deterministic little-endian f64 shape `[10240,14,7]` per view. Fresh-process replay must reproduce raw metrics and summary evidence; identity, scope or numeric failure produces a typed refusal.

Four focused synthetic checks passed in0.115s, Ruff formatting/lint and claim wording passed. Independent mathematical/source review found no blocker. Preparation took4.731441s. Broad QA remains dormant.

If valid, review distributions and freeze a separate successor contract informed by their limits; if invalid, preserve the refusal and repair the diagnostic boundary before interpretation. #973 remains open and #954 blocked. This comment records the freeze before retained outcomes; full preparation and results will land through the protected PR path.


### comment 5520332781 — 2026-09-03T04:25:44Z — Casey-allard

Delivered by PR #1093, merged as `a7f62b025c707640058e48721ef4971f8be789c5`. The reviewed source and documentation are the protected delivery of #1082; the independent source/result review and exact preparation/result/replay envelopes are in `docs/r4_token_exposure_1082*`.

Terminal: `TOKEN_EXPOSURE_DESCRIPTIVE_COMPLETE`. One run and one fresh-process replay reproduced all eight historical bindings, both raw metric byte streams and complete evidence over 286,720 used-role measurements. Result CID `blake3:e88501f05d4c58249806b2c9c5dabddd84eecd71256de9d4001c378cf8b9be03`; replay CID `blake3:3629ea8327a3e3cfc3a35d17da7d78de81ffe95f65b53266f18fac865a8240cc`.

The control's exposure is role-selective. Averaged over four fact slots and 8,192 supported rows per view, view 0 fact locations receive about 0.0035% changed-frame attention; view 1 fact objects receive about 0.0039%. Strongly exposed roles have D/A close to one, while substantial pooling displacement also occurs in historically retained-answer rows. The downstream cause remains unmeasured; preserve #1079's weak-control terminal.

Run/replay total: 31.159102208 seconds internally, 33.659554750 seconds externally; peak 0.9890594482 GiB; output 26,336,012 bytes. Zero new answer predictions, downstream/head forwards, fitting, development scoring, controls, geometry changes or generation. Four focused synthetic checks plus formatting/lint/claim wording passed. Independent review rehashed 316 source files and immutable/raw evidence and reconciled numerical summaries without rerunning the model; final delivery review resolved a population-label clarification. Required GitHub statuses were transport acknowledgements, with broad QA NOT_RUN.

Next: #1085 specifies removal of externally supplied clause segmentation while preserving the learned reader/core, known lexicon/query form and four-fact context. A later adapter-versus-supplied-segmentation comparison needs a separate frozen contract. No additional model experiment follows from this completion. #973 remains open and #954 blocked.



---
_Comment/review ids in this file: 5520228872, 5520332781_

