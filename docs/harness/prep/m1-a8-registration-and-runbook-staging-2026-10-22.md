# Pre-registration — M1 Admission-Preserving A8 Finite Source Qualification & Calibration (`train61-synthetic1-interval64-two-arm-throughput-v1`)

- date: 2026-10-22
- milestone: M1 #2029 — Language base: geometric stack LM good enough to carry context
- issue: #2832 (`m1-a8-registration-and-runbook-staging`)
- mechanism: discrete-attention
- mechanism brief: [../../mechanisms/discrete-attention.md](../../mechanisms/discrete-attention.md)
- run: Finite source qualification and same-hardware gate-cost calibration (`m1-attention-active-audit-finite-source-qualification-v1`) ahead of the admission-preserving A8 dose
- line: m1-discrete-attention — count 2/3 (verification/qualification stage; does not increment the D21 mechanism negative count)
- dose sheet (immutable, unchanged): [`m1-attention-admission-preserving-DRAFT-2026-10-19.md`](m1-attention-admission-preserving-DRAFT-2026-10-19.md) (SHA-256 `fcb770e66d38f607d25e16dade14ac7a6223dd8af0ccc9ec22c80db71f2c3450`, `17,745` B, `TEST FITNESS: FIT`, restored byte-exact in `#2785` `c6eb0526566b976ca8cc281c867a1d0c4c3f0b40`)
- parent A8 sheet: [`m1-attention-a8-address-cells-learning-2026-10-19.md`](m1-attention-a8-address-cells-learning-2026-10-19.md) (SHA-256 `3e622bebd7c468c6cd3d15a38486768c7902807998f71cae039a4341ab55da2b`, `12,697` B, `TEST FITNESS: FIT`)
- binding record: [`../../labs/admission-preserving-qualification-2026-10-19/README.md`](../../labs/admission-preserving-qualification-2026-10-19/README.md)
- staged runbook: [`../../labs/m1-a8-registration-and-runbook-staging/README.md`](../../labs/m1-a8-registration-and-runbook-staging/README.md)

## Bar

The finite-qualification and calibration bar is taken directly from the FIT-approved admission-preserving dose sheet [`m1-attention-admission-preserving-DRAFT-2026-10-19.md`](m1-attention-admission-preserving-DRAFT-2026-10-19.md) (§3–§4) and the qualification binding record [`../../labs/admission-preserving-qualification-2026-10-19/README.md`](../../labs/admission-preserving-qualification-2026-10-19/README.md), as enforced in [`../../../crates/uor-r4-discrete/src/attention_audited.rs`](../../../crates/uor-r4-discrete/src/attention_audited.rs) (`validate_qualification` and `validate_calibration`) and [`../../../crates/uor-r4-discrete/src/attention_audit_qualification.rs`](../../../crates/uor-r4-discrete/src/attention_audit_qualification.rs):

1. **Five mandatory qualification stages (`uor-r4.attention-audit-qualification/1`, `status == "PASS"` on both `Relation` and `FrozenZero` arms):**
   - `canonical_initlock_c2`: `qualify-relation` passes all `9/9` controls (`applied_updates == 0`, `training == "HELD"`), including `relation_copy_successor_fixture` (`C2`) and `active_leaf_initialization_and_matched_off_path` (`INIT LOCK` on seeds `1` and `2` with `plus_one_combine_indices == [0, 5, 10, 15]`, `treatment_control_snapshot_bit_identical == true`, and `reload_bit_identical == true`).
   - `required_negatives`: `negative_k16` (`attention-build-identity` refuses `K=16` with exit `2`) and `negative_nonroot` (`init-corpus` on `neg-b-a8-nonroot-table.json` `e35bb48d…` refuses non-root table with exit `2` and GPU status `6`).
   - `g_occ`: checkpoint-0 reproduction on both arms (`b3_psucc == 1,351` on `3,847` `P_succ` rows, `b3_full == 4,244` on `14,287` full-panel rows, `top1 == 133`, `bar_i == 86`, `occupied_cells == 256`, `distinct_vectors == 4,096`, `tie_mass == 0`, `1,351 - psucc_reference_top1 > max(500, bar_i)`, `epoch == 0`, `updates_applied == 0`, and pinned `A8` vs unpinned `SeededDeviceV1` checkpoint-0 `routes`, `gates`, `seed`, `epoch` equality).
   - `sensitivity`: `propose` on `gate 0` (`combine_probe`) and `gate 22` (`relation_probe`) against each arm's authenticated checkpoint-0 model satisfies `both_probes_any_active_candidate_cost_differs_v1` (`combine_sensitive == 1`, `relation_sensitive == 1`, `epoch == 0`, `applied == false`).
   - `audited_restore_epoch_cursor`: `uor-r4.attention-audit-production-restore-proof/1` (`scope == "actual-a8-post-update-restore-and-train-anchor-v1"`) executes `384` finite attempts per arm (`accepted_updates_before_restore > 0`), records durable `CONTROLLED_PRODUCTION_RESTORE_EXERCISE_INTENT` (`natural_audit_breach_claimed == false`), restores the checkpoint-0 model storage and snapshot into a fresh runtime (`continuation_checkpoint.cursor.completed_batches == 384`, `accepted_updates == 0`, `observed_epoch == 0`, `continuation.model_sha256 == initial.model_sha256`), verifies `initial_anchor` and `restored_anchor` evaluations agree across all `3,847` TRAIN anchor rows (`aggregate` and `batches`), and verifies `old_handles_refused == true`.
2. **Readiness ratio bar (`uor-r4.attention-audit-calibration/1`, from `fcb770e6…` §4):**
   - `R = projected_added_gate_ns / ungated_two_arm_dose_ns <= 0.15` (checked in exact integer arithmetic via `u128::from(projected_added_gate_ns) * 100 <= u128::from(ungated_two_arm_dose_ns) * 15`), where:
     $$\text{projected\_added\_gate\_ns} = 516 \cdot \max(\text{gate\_samples\_ns}) + 32{,}808 \cdot \max(\text{step\_samples\_ns}) + 516 \cdot \text{pass\_boundary\_ns} + 514 \cdot (\max(\text{restore\_samples\_ns}) + \text{breach\_restore\_ack\_ns})$$
     and `ungated_two_arm_dose_ns` is extrapolated on the same hardware from the `384`-attempt per-arm family timings (`[336, 22, 26]` attempts scaled to full-dose `[14,354, 920, 1,130]` attempts per arm).
   - Anchor baseline admitted count on the `3,847`-row TRAIN anchor (`8c1b80e9…`) must satisfy `baseline_admitted >= 500`.

## Base artifact

- **Bound source commit:** `fd409fcdf3bc43d607e768bae6f8cf403aa75901` (`#2782`, [`../../labs/admission-anchor-controller-2026-10-19/README.md`](../../labs/admission-anchor-controller-2026-10-19/README.md)):
  - `driver_source_sha256`: `0f8ce4061996acdcce3ad4ae01d9e267f280703a77536eb6ecc24bfdc6e6df1d` (19 integrated source files in `crates/uor-r4-discrete/src/`)
  - `compiled_source_sha256`: `a5d59806d7aa12ce7bed05bacc8cffa700da717a172efbe787075cce78c60831` (the dose verifier's 3-file hash, `verify_a8_address_cells_dose.py:1417`; not the binary's `compiled_identity.source_sha256`)
  - **Superseded by erratum 2 (claude-70, 2026-10-24):** this value cannot be reproduced, because `compiled.source_sha256` hashes ptxas reports that contain `Compile time = N ms`. Step 1.1 now pins the deterministic PTX, cubin, kernel and geometry digests plus the saved run binary (`executable_sha256 fb4493253d3e…`, `source_sha256 81651081145f…`); see the runbook. The claude-64 note follows unchanged. `compiled_identity.source_sha256` (measured, erratum claude-64, 2026-10-23): `177f1b3da11eddbe2cf90d8172c48d33fafca95b533ef8898f3e8d2c245edcfc`. `fd409fcdf` was built with `--features cuda` (nvcc 12.8.93, `sm_120`) and `discrete-stack build-identity` was run, with no model run (#2842, 6372197194; HF `results/claude/craw-a8id-20261023/`).
  - `geometry_sha256`: `46ddbb76b7fbe6ac6222ea24779fc10055b1d6cea6a8da786895676fd29fe327`. This is the erratum claude-64 measured value at `fd409fcdf`, and it matches `verify_a8_address_cells_dose.py:453`. It replaces `46ddbb76d4f7…`, which no build reports.
- **Pinned `A8` codebook (`address_structured_v1`):** file SHA-256 `6600bdb1306dbaea4d77f5bf021e717a6bef791e44c1ec7d7c9bb9f0e8cd3cd1` (`278,778` B), `table_sha256` `4122cbab44bd88658743984fc1ab79059bf032c3e2fe526ffcf4ea5f95d9cd74` (`131,072` `i32` LE words), `counts_sha256` `755a5a81ddbf0f9d81b4d0bd88e54435b96f5485f87ad92a22d235651fe6ca44`.
- **Pinned two-arm configs (`V4096 / B4 / C192 / K8`, seed `1`):**
  - Treatment (`Relation` `LearnV1`): [`../../labs/a8-training-configs-2026-10-19/evidence/a8-relation-seed1.json`](../../labs/a8-training-configs-2026-10-19/evidence/a8-relation-seed1.json) (`469a2ff0fe17a6a80f5875a077b3e6a9436553efdf1c6e7884541220553e9d5e`, `config_sha256 = f223d78f45fe26dee4080ae7ca9a5b9ff12d9ef0df847375671d52db733fbecd`, `lineage_sha256 = f22ebd7129f194c7b3089a3b647b3d174d8c0c95fb18395611bba77036fa3cf3`).
  - Control (`FrozenZero` `FrozenZeroV1`): [`../../labs/a8-training-configs-2026-10-19/evidence/a8-frozen-zero-seed1.json`](../../labs/a8-training-configs-2026-10-19/evidence/a8-frozen-zero-seed1.json) (`cf7ebcc85d2e463ae8f6691b63d90a0e005701cb38b8249a8d0d5460d8508479`, `config_sha256 = c3fbffe08d13008b8b7ab400a4ef946f2c49632906d88e431d5e80314b9f5536`, `lineage_sha256 = e4c3efc99f51bc3eb4c13844cc3e7dc12e9e42d32a2cde39603e6052d649e79a`).

## Data split

- **TRAIN split (`mw-balp`):** `tokens.u16` `3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8`, `response_mask.u8` `b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3` (`7,512,797` tokens, `2,425,925` eligible targets).
- **Fixed TRAIN audit anchor (`uor-r4.train-copy-successor-anchor/v1`):** `3,847` genuine TRAIN `P_succ` rows (`is_copy_successor` in `C=192` in-document window) from the unvisited TRAIN tail starting at `start_eligible_index = 1,049,856` (actual eligible indices `1,049,857..=1,064,112`), canonical CSV SHA-256 `8c1b80e9b15e4e2bcc765037afa7800b785418be3e20b8cc80ce13ee2e32b3c1` (`61` batches: `60 x 64` + `1 x 7` tail). Disjoint from the `16,404 x 64 = 1,049,856` training prefix and disjoint from DEV and fresh V2.
- **DEV split (checkpoint-0 baseline reproduction only during qualification):** `tokens.u16` `0919e90f813bf02f6f0528ebcf1f34c97987113b679619b0b44e065fa90b2546`, `response_mask.u8` `a838bb1ae20c05a1b68265a7aa11c3dd146200b7d1ff6ef185ff999e442f5327` (`14,287` rows, `3,847` `P_succ`). Fresh V2 (`0b7e754c…` / `a1cda529…`) is not read during finite qualification.

## Steps

1. Build `discrete-stack` (`cargo build --release --locked -p uor-r4-discrete --features cuda`) at bound source `fd409fcdf3bc43d607e768bae6f8cf403aa75901` and run `attention-audit-build-identity` on both A8 configs.
2. Execute the 8 per-arm (`16` total) zero-update inherited command observations (`qualify-relation`, `negative_k16`, `negative_nonroot`, `init-corpus` on pinned `A8` and unpinned `SeededDeviceV1`, `eval-corpus` at checkpoint 0, `propose` on `gate 0` and `gate 22`) and record their `uor-r4.attention-audit-inherited-command/1` manifests.
3. Assemble `QualificationPlan` (`uor-r4.attention-audit-source-qualification-plan/1`, scope `m1-attention-active-audit-finite-source-qualification-v1`, protocol `train61-synthetic1-interval64-two-arm-throughput-v1`), bind the 4 `Authorization` records (`qualification_fit`, `owner_cost_scope`, `pm_dispatch`, `source_plan_review`), and validate with `attention-audit-qualification-check-plan` (`QUALIFICATION_PLAN_METADATA_VALID`, `dose_release: false`).
4. Casey personally runs `attention-audit-qualification-owner-release` in an interactive terminal with `UOR_LAB` unset to produce `QualificationAdmission` (`uor-r4.attention-audit-source-qualification-admission/1`).
5. Run `attention-audit-qualify CONFIG QUALIFICATION_ADMISSION NEW_REPORT_DIR` to execute the synthetic apply/refusal fixture, the `384`-attempt per-arm production restore proof (`uor-r4.attention-audit-production-restore-proof/1`), the `61`-batch TRAIN anchor timing samples, and the host bookkeeping envelope measurement, emitting `qualification.json` (`uor-r4.attention-audit-qualification/1`) and `calibration.json` (`uor-r4.attention-audit-calibration/1`).
6. Verify the emitted qualification bundle on host with `python3 docs/labs/address-cells-consumer-verifier-2026-10-19/verify_a8_address_cells_dose.py --verify-qualification-bundle <BUNDLE_JSON>`. Full-dose release (`attention-audit-owner-release`) remains separate and gated on `R <= 0.15`, a `CostedDoseDecision` (`uor-r4.attention-audit-costed-dose-decision/1`), and Casey's separate D40 dose release.

## Minimum test power

- **Inherited DEV reproduction panel:** `14,287` rows (`3,847` `P_succ`), seed `1`.
- **TRAIN audit anchor:** `3,847` genuine unvisited-tail TRAIN `P_succ` rows (`8c1b80e9…`, `>= 200` rows per D22 §3), `baseline_admitted >= 500`, scored across `61` device batches (`1..=3` gate timing samples per arm + pre/post-restore invariance check).
- **Throughput & restoration sample:** `384` actual attempts per arm (`336` vector, `22` combine, `26` relation attempts per arm; `768` total timed transition receipts across the two arms) plus `2` controlled production restores and `128` host step-bookkeeping samples.

## Cost

- **Preparation & host staging:** `$0.00` (host-only registration and runbook staging).
- **Finite qualification & calibration pod window (upon owner cost approval, PM dispatch, and Casey's qualification release):** `<= 0.5 h` on `1 x` Vast.ai `2 x RTX 5090` pod (`~$0.95` at `~$1.89/h`, well within `<= 4` pods and `<= $8.00/h` owner caps; `>= 30 GiB` free disk floor plus declared `staging_archive_reserve_bytes`).
- **Controls & evaluation:** included inside the single `attention-audit-qualify` run (`384` attempts on `Relation` + `384` attempts on `FrozenZero`, `1` synthetic fixture, `1..=3` anchor samples per arm, `2` controlled restores).
- **Placement (D24):** CUDA qualification and timing run on GPU only via `scripts/pod/uor-pod`; host consumer verification runs on CPU with zero model compute.

## Stop

- **Pre-training readiness stop:** if any of the 5 qualification stages fails, if `baseline_admitted < 500`, or if `R = projected_added_gate_ns / ungated_two_arm_dose_ns > 0.15`, stop before any full-dose training (`INSTRUMENT_UNREACHABLE`, D31) and replan.
- **No dose consumption:** `attention-audit-qualify` is non-resumable, never advances a full-dose training schedule, and never consumes a full-dose D40 release.
- **Hard stops:** owner caps (`<= 4` pods, `<= $8.00/h`), the `30 GiB + reserve` disk floor, or any hash/lineage/cursor mismatch.

## Verdict

TEST FITNESS: FIT
- sheet: `docs/harness/prep/m1-a8-registration-and-runbook-staging-2026-10-22.md`
- dose sheet (unchanged): `docs/harness/prep/m1-attention-admission-preserving-DRAFT-2026-10-19.md` (`fcb770e66d38f607d25e16dade14ac7a6223dd8af0ccc9ec22c80db71f2c3450`)
- reviewer: fresh-context non-author review (antigravity fresh-context reviewer subagent + prior Codex/Claude/AGY `#2782`/`#2783`/`#2785` contract reviews `6266518570`, `6267441702`)
- date: 2026-10-22
- note: Clears the finite source-qualification and calibration protocol (`train61-synthetic1-interval64-two-arm-throughput-v1`, scope `m1-attention-active-audit-finite-source-qualification-v1`) at bound source `fd409fcdf3bc43d607e768bae6f8cf403aa75901` (`#2782`), preserving the admission-preserving dose sheet `fcb770e6…` byte-identical. Owner holds, costed-plan approval, PM dispatch, and Casey's personal D40 releases remain required before any pod or GPU execution.
