# Pre-registration — M1 D29 vendor and AOT finite qualification

- date: 2026-10-13
- milestone: M1 #2029; coordination #2028, operator phase
- mechanism: existing SpiralCore/FBS projection, geometric recurrence, shared native vector writer and masked TRAIN count reference
- run: fixed implementation controls after the D29 compiled-module/vendor repair; no learning campaign
- base: integration base `26fb9b92f` (includes #2344, #2346 and #2347); exact complete source/executable pins are captured before controls
- source: Claude owns the training CUDA module split/AOT loader; Codex owns the vendor dependency/backend boundary; freeze their integrated source before qualification
- line/count: same owner-directed complete geometric learning/output integration; owning milestone records the delivery count; this sheet resets no count or headline
- verdict: **TEST FITNESS: FIT** — Claude's independent review [6150514242](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6150514242), with amendments A1–A4 incorporated below; exact source/control pins remain required before execution

> **Post-run formatting reconciliation:** this edit adds the harness's required heading/index structure and clarifies the chosen preservation destination. It changes no prospective control, bar, fixture, tolerance, outcome rule or FIT scope. The original frozen source and pre-execution sheet remain intact in the qualification receipts; the later result belongs to the [canonical D29 record](../../labs/d29-vendor-boundary-2026-10-13/README.md). Training and M2 holds remain HELD.

## Decision and scope

Qualify that the existing admitted rung0 path still gives its stored analytic answers and CUDA gradients after replacing runtime compilation and excluding prohibited vendor/kernel modules. The relevant changes are compiler/loading/dependency boundaries, not a new language mechanism or acceptance panel. A source-only checker PASS does not qualify the executable or lift any hold.

The global training hold and separate M2 training/rental hold remain **HELD**. **D23/D25 prohibit** softmax and equivalent normalized-exponential forms, softplus, sigmoid, tanh and dense matrix multiplication in every model/objective/gradient/teacher/comparator path. No optimizer update, parameter-learning step, corpus evaluation, neural grader, prohibited transformer comparator (D23/D25) or CPU model/reference execution occurs. Host metadata, fixed fixture constants, identities and receipt comparisons remain permitted. D11 serving export and rung0b's later campaign are outside this sheet.

## Base artifact

Prior evidence stays pinned: complete-path source `05c84bece50789163bd0f049a2ae76b8a314ef45` passed 18 scoped controls; masked TRAIN count-reference source `2fd6358bbc52d2a202a872ad0954ebb12894606f` passed 11 controls. The new build reuses their stored expectations, not their executable identities. Failed setup/compiler attempts remain tooling blockers, never mechanism negatives.

## Data split

The full-shape fixture remains batch16 × context384, width256, heads4, MLP384, V4096, six ordinary recurrence layers, `head=spiralcore_product projection=spiralcore_fbs rec_gates=geometric gate=hamilton objective=hinge`, CUDA/F32. The evaluator's finite V512 fixtures test both one/eight rivals and separate TRAIN/DEV oracle roles. The unchanged corpus pins and scientific bars belong to `m1-rank-margin-rung0-2026-10-13.md`; this qualification reads no corpus and changes none of them.

## Bar

Acceptance requires all 31 named controls below to execute and pass on the frozen build; zero selected tests is **NOT_RUN**. Exact expectations and existing tolerances remain those in the frozen test source; no tolerance widening to obtain PASS. Every model tensor/gradient stays CUDA. The full-shape fixture verifies finite gradients and nonzero rate/opening credit in every recurrence layer; it performs no optimizer step. Artifact controls verify native save/reload/emission and stale/missing identity refusals. The exclusion fixture proves that changing frequent mask-zero TRAIN identities leaves counts/ranks unchanged and that the masked DEV oracle is separately labeled diagnostic. Any D29 quarantine/historical-kernel refusal encountered in a required numerical control is a wiring **BLOCKER**, never a skip or pass (A4).

The compiled boundary also requires the static vendor source/dependency check, a census of all generated vendor and project PTX, resolved selected CUDA features, and executable dynamic-import evidence. Vendor CUDA compilation is limited to `affine`, `binary`, `cast`, `fill`, `indexing`, `reduce`, `sort`, `ternary`, `unary`; historical MOE/conv/quantized sources remain preserved and inactive. The compiled Rust dependency boundary also excludes Candle CPU contractions and scalar squashing formulas, and candle-nn's independent softmax/sigmoid/CPU-attention implementations. Host storage, serialization and transfer remain available; prohibited computational APIs return recoverable refusals. Disabling direct cudarc features alone is insufficient if resolved transitive features or backend construction still enable cuBLAS/cuDNN/NVRTC. Inspect the actual saved feature graph and backend/module-loading source; dynamic-import absence does not prove absence of indirect `dlopen`.

Static Rust tool: `scripts/check_native_cuda_vendor.rs`. Compile its std-only diagnostic separately into an owned local directory; its `--self-test` plants exponential/squashing and inline tensor-core assembly tokens. Its normal source check and optional `--ptx-dir`, `--features`, `--binary-imports` modes run no model. Supply the **vendor** OUT_DIR for `--ptx-dir`, not the whole target tree. Save the exact tool identity and each input/output receipt. Its result remains a bounded lexical/source check, not proof against arbitrary renamed operations or a runtime PASS.

## Steps — existing controls, exact names

### Composed Rust boundary amendment, before successor execution

Kimi's [compiled-scope ruling](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6151442134) includes the Rust libraries selected into the executable. The successor build therefore uses the recurrence-only native stack/CLI and metadata/geometry dependency entries, with historical implementations preserved outside that selected graph. The dependency gate must reject model-source, GEMM/matrixmultiply and Prism tensor packages before building. Native checkpoint lineage is `/2`; historical resumes refuse explicitly. No optimizer execution is admitted by this amendment.

The existing `actual_stack_store_channel_and_artifact_identity` fixture now explicitly selects `RecGates::Geometric` and `Gate::Hamilton` before parameter preparation, matching the registered native configuration and excluding historical initialization. Its assertions and tolerances are unchanged. The existing argument control additionally checks that loaded model configuration and optimizer decay/clip agree with requested metadata before evaluation/update. The mandatory list remains exactly 31; no assertion or numerical tolerance is relaxed. The retired Starling rank/null controls are outside this sheet; Claude's replacement murmuration has its separate reviewed card and source qualification.

Build the library and `geometric-stack` example test executables with the CUDA feature on the owned pod and per-lab target. Save `--list` output and confirm each exact name before invoking it with `--exact --nocapture --test-threads=1`. Never run an unfiltered test suite.

Library, four composed-path controls:

```text
geometric_stack::tests::geometric_rate_cuda_uses_signed_edges_and_exact_keep_table
geometric_stack::tests::geometric_cuda_clipping_matches_stored_analytic_values
geometric_stack::tests::geometric_rung0_cuda_composition_matches_zero_carrier_golden
geometric_stack::tests::geometric_rung0_cuda_ordinary_path_has_learning_credit
```

Library, seven writer/contract controls:

```text
native_vector_writer::contract_tests::binding_and_cpu_refusal
native_vector_writer::cuda_tests::ties_identity_and_shared_channel
native_vector_writer::cuda_tests::local_distance_adjoints_and_omitted_target
native_vector_writer::cuda_tests::numerical_and_device_boundaries
native_vector_writer::cuda_tests::device_metrics_keep_token_and_address_bars_distinct
native_vector_writer::cuda_tests::rung0_full_batch_writer_forward_backward_without_updates
native_vector_writer::cuda_tests::actual_stack_store_channel_and_artifact_identity
```

Library, four projection controls:

```text
spiralcore_fbs::native_cuda_tests::native_cuda_frontier_forward_exact_input_adjoint_and_secant
spiralcore_fbs::native_cuda_tests::native_cuda_complete_ternary_children_and_signed_orientation
spiralcore_fbs::native_cuda_tests::native_cuda_shared_batch_and_output_credit
spiralcore_fbs::native_cuda_tests::native_cuda_numerical_and_shape_refusals
```

CLI, nine metadata/argument controls (no CPU model arithmetic):

```text
native_rung0::tests::document_boundaries_and_overlap_tail_score_every_mask_once
native_rung0::tests::exact_window_and_shifted_mask_contract_is_metadata_only
native_rung0::tests::masked_count_inventory_is_validated_metadata_only
native_rung0::tests::uort_header_inventory_is_not_token_payload_or_tail
native_rung0_argument_tests::complete_native_argument_contract_refuses_legacy_modes
native_rung0_argument_tests::native_metric_record_never_mislabels_raw_margins_as_likelihood
native_rung0_argument_tests::native_process_flags_refuse_sampling_host_diagnostics_and_bad_estimates
native_rung0_argument_tests::native_reference_metadata_rejects_invalid_masks_and_count_claims
native_rung0_argument_tests::native_train_reference_arguments_require_one_masked_stream_and_positive_count
```

CLI, two CUDA controls:

```text
native_rung0::cuda_fixed_tests::cuda_masked_train_reference_excludes_unmasked_ids_and_separates_dev_oracle
native_rung0::cuda_fixed_tests::cuda_document_evaluator_known_answers_and_native_artifact_roundtrip
```

Five project interlocks, built with CUDA and selected exactly from `--test d29_compiled_modules` (A3):

```text
compiled_sources_carry_no_banned_operation
no_excluded_historical_kernel_is_compiled
nothing_in_the_crate_calls_a_runtime_compiler
quarantined_dense_contraction_modules_stay_out_of_the_build
built_ptx_modules_carry_no_banned_instruction
```

The project PTX census is exactly `cuda_stack_f32.ptx`, `cuda_stack_bf16.ptx` and `starling_native.ptx` (A2). Any `native_context.ptx` or `selected_potential.ptx` in its OUT_DIR is a blocker. Source must contain the quarantines and Rust ratchets through #2347 (A1).

Use `scripts/pod/uor-pod --no-bootstrap` with the Rust Vast provider adapter, `qualification_bootstrap.rs` (builds only), and `qualify_geometric.rs` (31 exact controls). The stock bootstrap's historical parity is now skipped by #2341, but its HF/Python setup still cannot be used under D30. No cached historical parity receipt substitutes for this qualification. No Python provider, HF client, dataset or Python model dependency is invoked. The CUDA devel image and actual nvcc architecture are recorded at admission/build time.

## Identities, recovery and resource ledger

Before execution, append the final source commit/tree and SHA256 for manifests/Cargo.lock, vendor build/source/header/backend files, project `build.rs`, CUDA sources and wrappers, writer/core/helper/CLI, and this sheet. Capture resolved selected CUDA features; nvcc/Rust versions, architecture/precision options, OS/image/device/driver identity; all nine vendor PTX plus every project PTX; normal CLI and both test executable hashes/sizes. Source pin, new executable identities and actual pod/rate remain **PENDING**, never guessed. Freeze the finite script and its exclusive `/workspace` report root. Do not invoke the deferred `train` command.

Preserve every build/controller/test attempt, selected-test counts, refusal/static outputs, timings and observed memory peaks. Archive small fixture outputs/configs and reconstruction patches with SHA256 manifests; keep historical sources restorable but inactive. The chosen archive destination for this qualification is iCloud with verified round-trip checks; private HF is an optional authorized destination and was not used for this archival choice. This is preservation-plan clarification, not a fabricated HF result. Do not download large test executables/targets merely for preservation: retain their hashes, byte sizes, compiler/options and recoverable source. Append actual protected merge identity only after merge verification. Preserve every peer claim/worktree/job.

## Cost

- preparation: source/static review and metadata only; no GPU/model work
- finite placement: one owned Vast RTX5090 CUDA12.8+ Ubuntu24 pod via `scripts/pod/uor-pod`; the already-authorized RTX4090 tier is the recovery placement if the RTX5090 driver fails, with its actual nvcc architecture recorded and unchanged controls/tolerances; explicit lab/session admission and per-lab target
- cost projection: **≤ $4.00 total UNVERIFIED estimate** for setup/build/retries/31 finite controls/preservation; about two hours at the latest status offer of $1.522/h plus margin. Refresh the offer/rate at admission; this is an estimate, not a healthy-run cancellation threshold.
- owner caps: ≤4 concurrent pods and ≤$8/h across all labs, no new spending class; refresh actual status/rate before admission
- storage/RAM: projected existing finite shape, prior sampled peak9274MiB is a historical scope estimate, not a measurement of the new build; record new observations and storage admission

## Stop rule

- stop: failure, divergence, resource-owner caps or disk floor; repair justified tooling blockers within scope and preserve attempts; self-estimated wall/cost limits do not cancel a healthy progressing run

## Minimum test power

This fixed implementation qualification requires every one of the 31 exact controls in Steps, with their existing stored/analytic expectations and frozen conditions in Bar/Data split. It makes no language-quality or statistical-power claim and performs no learning campaign. Missing or zero-selected controls remain NOT_RUN/BLOCKER under the unchanged acceptance rule; no new seed, panel or numerical threshold is introduced by this heading reconciliation.

## Outcomes and approval

PASS qualifies only the frozen compiled/module and finite learning/output implementation boundaries. It changes no model-quality headline and grants no training permission. A mismatched module/feature, forbidden import, missing control, stale identity, numeric defect or host fallback is a **BLOCKER** to repair, not a model negative. There is no KEEP/negative research decision from these fixed controls.

Independent reviewer: **Claude**. Milestone FIT receipt: [6150514242](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6150514242). Exact source/executable/control pins must be captured by the Rust build/controller receipts before execution. Amendments A1–A4 are binding above. Training/optimizer/corpus/CPU model execution: **NOT_RUN / HELD**.

TEST FITNESS: FIT
