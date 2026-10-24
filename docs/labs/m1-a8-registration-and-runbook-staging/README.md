# Admission-Preserving A8 Finite Qualification & Audited-Dose Staged Runbook (`#2832`)

- **Lab:** `antigravity` ([`#2832`](https://github.com/UOR-Foundation/uor-r4/issues/2832#issuecomment-6343653097))
- **Milestone:** M1 (`#2029`) — `m1-discrete-attention`
- **Status:** **STAGED (host-only, `$0.00` spend, training HELD).** Nothing has executed on GPU.
- **Immutable Admission-Preserving Dose Sheet (unchanged):** [`../../harness/prep/m1-attention-admission-preserving-DRAFT-2026-10-19.md`](../../harness/prep/m1-attention-admission-preserving-DRAFT-2026-10-19.md) — **TEST FITNESS: FIT**, SHA-256 `fcb770e66d38f607d25e16dade14ac7a6223dd8af0ccc9ec22c80db71f2c3450` (`17,745` B, restored byte-identical in `#2785` `c6eb0526566b976ca8cc281c867a1d0c4c3f0b40`, pinned in [`../../../crates/uor-r4-discrete/src/attention_anchor_device.rs`](../../../crates/uor-r4-discrete/src/attention_anchor_device.rs) as `FIT_SHEET_SHA256`).
- **Registered Finite-Qualification Sheet:** [`../../harness/prep/m1-a8-registration-and-runbook-staging-2026-10-22.md`](../../harness/prep/m1-a8-registration-and-runbook-staging-2026-10-22.md) — **TEST FITNESS: FIT**, binding [`../admission-preserving-qualification-2026-10-19/README.md`](../admission-preserving-qualification-2026-10-19/README.md) (`train61-synthetic1-interval64-two-arm-throughput-v1`, scope `m1-attention-active-audit-finite-source-qualification-v1`).
- **Parent A8 Learning Sheet:** [`../../harness/prep/m1-attention-a8-address-cells-learning-2026-10-19.md`](../../harness/prep/m1-attention-a8-address-cells-learning-2026-10-19.md) — **TEST FITNESS: FIT**, SHA-256 `3e622bebd7c468c6cd3d15a38486768c7902807998f71cae039a4341ab55da2b` (`12,697` B, `A8_SHEET_SHA256`).
- **Bound Source Commit:** `fd409fcdf3bc43d607e768bae6f8cf403aa75901` (`#2782`, [`../admission-anchor-controller-2026-10-19/README.md`](../admission-anchor-controller-2026-10-19/README.md)), consumer-verified on host by [`../address-cells-consumer-verifier-2026-10-19/verify_a8_address_cells_dose.py`](../address-cells-consumer-verifier-2026-10-19/verify_a8_address_cells_dose.py) (`82f29cdcbd387ba19597b9dea79711eb54dcbd1df49ec8c0b74e623d080b8223`, `#2783`, `64/64` self-tests PASS).
- **Earlier runbook (Codex, 19 Oct):** kept byte-identical as [`codex-audited-dose-runbook-2026-10-19.md`](codex-audited-dose-runbook-2026-10-19.md). Its sheet pin and outside hash table are superseded; the steps below win. What it adds is in §6.

---

## 1. Pinned Inputs & Identity Table

| Artifact / Field | Path / Identifier | SHA-256 / Value |
| :--- | :--- | :--- |
| **Admission-Preserving Dose Sheet** | [`../../harness/prep/m1-attention-admission-preserving-DRAFT-2026-10-19.md`](../../harness/prep/m1-attention-admission-preserving-DRAFT-2026-10-19.md) | `fcb770e66d38f607d25e16dade14ac7a6223dd8af0ccc9ec22c80db71f2c3450` (`17,745` B, `FIT_SHEET_SHA256`) |
| **Parent A8 Learning Sheet** | [`../../harness/prep/m1-attention-a8-address-cells-learning-2026-10-19.md`](../../harness/prep/m1-attention-a8-address-cells-learning-2026-10-19.md) | `3e622bebd7c468c6cd3d15a38486768c7902807998f71cae039a4341ab55da2b` (`12,697` B, `A8_SHEET_SHA256`) |
| **Bound Source Commit (`#2782`)** | `fd409fcdf3bc43d607e768bae6f8cf403aa75901` | `driver_source_sha256 = 0f8ce4061996acdcce3ad4ae01d9e267f280703a77536eb6ecc24bfdc6e6df1d`, `compiled_source_sha256 = a5d59806d7aa12ce7bed05bacc8cffa700da717a172efbe787075cce78c60831`, `geometry_sha256 = 46ddbb76d4f7f509fa9d8e5ee326b1cc8e882a20d532ce514a6a7cc80f37eb5a` |
| **Treatment Config (`Relation`, `LearnV1`)** | [`../a8-training-configs-2026-10-19/evidence/a8-relation-seed1.json`](../a8-training-configs-2026-10-19/evidence/a8-relation-seed1.json) | File `469a2ff0fe17a6a80f5875a077b3e6a9436553efdf1c6e7884541220553e9d5e` (`1,924` B); `config_sha256 = f223d78f45fe26dee4080ae7ca9a5b9ff12d9ef0df847375671d52db733fbecd`; `lineage_sha256 = f22ebd7129f194c7b3089a3b647b3d174d8c0c95fb18395611bba77036fa3cf3` |
| **Control Config (`FrozenZero`, `FrozenZeroV1`)** | [`../a8-training-configs-2026-10-19/evidence/a8-frozen-zero-seed1.json`](../a8-training-configs-2026-10-19/evidence/a8-frozen-zero-seed1.json) | File `cf7ebcc85d2e463ae8f6691b63d90a0e005701cb38b8249a8d0d5460d8508479` (`1,930` B); `config_sha256 = c3fbffe08d13008b8b7ab400a4ef946f2c49632906d88e431d5e80314b9f5536`; `lineage_sha256 = e4c3efc99f51bc3eb4c13844cc3e7dc12e9e42d32a2cde39603e6052d649e79a` |
| **Unpinned Treatment Initializer** | [`../injective-attention-runbook-2026-10-19/evidence/parent-rows-relation-seed1.json`](../injective-attention-runbook-2026-10-19/evidence/parent-rows-relation-seed1.json) | `52b32dc8747148e0cd385d309dbfb1c3cc750569d11ff3932745313e134cc670` |
| **Unpinned Control Initializer** | [`../injective-attention-runbook-2026-10-19/evidence/parent-rows-frozen-zero-seed1.json`](../injective-attention-runbook-2026-10-19/evidence/parent-rows-frozen-zero-seed1.json) | `6a035717130a5e7106e19ca585b4de4383506b425ce403d8a13d7ab44622e1cf` |
| **`PrimeZeta` Table (`P8`)** | [`../injective-attention-runbook-2026-10-19/evidence/prime-zeta-v4096-b4-c192-k8.json`](../injective-attention-runbook-2026-10-19/evidence/prime-zeta-v4096-b4-c192-k8.json) | `83c1380a8f507e51d0342faa1b089dea6a20561a03552208c093fa640afc124b` (`table_sha256 = e9e03bcd…`) |
| **Pinned `A8` Table (`address_structured_v1`)** | Rebuilt on pod via `address-cells-codebook` | File `6600bdb1306dbaea4d77f5bf021e717a6bef791e44c1ec7d7c9bb9f0e8cd3cd1`; `table_sha256 = 4122cbab44bd88658743984fc1ab79059bf032c3e2fe526ffcf4ea5f95d9cd74`; `counts_sha256 = 755a5a81ddbf0f9d81b4d0bd88e54435b96f5485f87ad92a22d235651fe6ca44` |
| **`G-Init` Negative (b) Non-Root Table** | [`../a8-dose-runbook-2026-10-19/evidence/neg-b-a8-nonroot-table.json`](../a8-dose-runbook-2026-10-19/evidence/neg-b-a8-nonroot-table.json) | `e35bb48d800cd41e5e6b0b48d4b41b8b61fc147f4fde98c54e8dff919d15f90e` (`table_sha256 = f9e1ea54…`) |
| **Fixed TRAIN Audit Anchor (`3,847` rows)** | `uor-r4.train-copy-successor-anchor/v1` (`start_eligible_index = 1,049,856`, `count = 3,847`, `context = 192`) | `REGISTERED_CSV_SHA256 = 8c1b80e9b15e4e2bcc765037afa7800b785418be3e20b8cc80ce13ee2e32b3c1` (`61` device batches: `60 x 64` + `1 x 7` tail) |
| **Host Consumer Verifier (`64/64` PASS)** | [`../address-cells-consumer-verifier-2026-10-19/verify_a8_address_cells_dose.py`](../address-cells-consumer-verifier-2026-10-19/verify_a8_address_cells_dose.py) | `82f29cdcbd387ba19597b9dea79711eb54dcbd1df49ec8c0b74e623d080b8223` (`417,600` B) |

---

## 2. Stage 1 Runbook & Executor — Finite Source Qualification & Calibration (`m1-attention-active-audit-finite-source-qualification-v1`)

Finite qualification runs **before** any full-dose release. It requires its own FIT sheet ([`../../harness/prep/m1-a8-registration-and-runbook-staging-2026-10-22.md`](../../harness/prep/m1-a8-registration-and-runbook-staging-2026-10-22.md)), owner cost/scope approval (`APPROVE_EXACT_FINITE_QUALIFICATION_SCOPE_AND_COST`), PM dispatch (`DISPATCH_FINITE_SOURCE_QUALIFICATION_ONLY`), independent plan review (`APPROVE_EXACT_QUALIFICATION_PLAN`), and Casey's personal qualification release (`RELEASE_FINITE_AUDITED_SOURCE_QUALIFICATION_ONLY`).

### Step 1.1 — Pod Checkout, CUDA Build & Identity Verification (Lab Side, After PM Dispatch)

On a leased `2 x RTX 5090` pod (`uor-pod up --ref fd409fcdf3bc43d607e768bae6f8cf403aa75901`, `<= 4` pods, `<= $8.00/h`, `>= 30 GiB` free disk plus reserve):

```sh
git checkout --detach fd409fcdf3bc43d607e768bae6f8cf403aa75901
test "$(git rev-parse HEAD)" = "fd409fcdf3bc43d607e768bae6f8cf403aa75901"

cargo build --release --locked -p uor-r4-discrete --features cuda
BIN="$(pwd)/target/release/discrete-stack"
CODEBOOK_BIN="$(pwd)/target/release/address-cells-codebook"

# Verify data pins (mw-balp TRAIN, DEV, tokenizer, PrimeZeta P8, and rebuilt A8)
"$CODEBOOK_BIN" docs/labs/address-cells-preflight-configs-2026-10-19/evidence/p8.json \
  /workspace/uor-r4/local/driver/data/address-cells/address-structured-v1.json structured

"$BIN" attention-audit-build-identity docs/labs/a8-training-configs-2026-10-19/evidence/a8-relation-seed1.json \
  > "$STAGE1_DIR/build-identity-relation.json"
"$BIN" attention-audit-build-identity docs/labs/a8-training-configs-2026-10-19/evidence/a8-frozen-zero-seed1.json \
  > "$STAGE1_DIR/build-identity-frozen-zero.json"
```

Both `attention-audit-build-identity` outputs must report:
- `driver_source_sha256 == "0f8ce4061996acdcce3ad4ae01d9e267f280703a77536eb6ecc24bfdc6e6df1d"`
- `compiled.geometry_sha256 == "46ddbb76b7fbe6ac6222ea24779fc10055b1d6cea6a8da786895676fd29fe327"`
- `compiled.kernel_sha256 == "5cd0e73241fd895f2dd0004c83af3d1eb9f89e00ec2e933c1acc44fc7127bfd6"` and `compiled.decomposition_kernel_sha256 == "29615646becefc333c697f685157a4c124847b5022e070ab00d7b9c749bc8506"`
- `compiled.cuda_images.ordinary_sha256 == "19306ffdc323eb29b0e8ce971f2724f0ee27a5e133b8fc7552c58b8886437ee6"`, `compiled.cuda_images.decomposition_sha256 == "e03d01314f0d1eddf2b5cb02cae25d9d6231316dc14ba7af4f4db0997ea2fb8b"`, `compiled.baseline_reuse_image.ptx_sha256 == "2ec171013d10f14ed4552436c0fdc33ccd8005eeba80f146f8c4c8016f71c3b0"` and `compiled.baseline_reuse_image.cubin_sha256 == "6749bfe3c8dcab20c1628ce342fb654589461d442ecdc60edfb39bace401aabe"`
- The run binary: the saved `discrete-stack` with `executable_sha256 == "fb4493253d3e26dc7cc5fbcbcf7fa4dc882d93a5c7b02d340b25918a2d25b00c"` and `compiled.source_sha256 == "81651081145fd4712046a8158a26c3bd6940ff5a60e5cdddf666057cb9fc0084"`. It is restored from the HF store (`results/claude/a8-stage1-pod1-20261024-r2/a8s1-pod1.tar`, sha256 `01ac2067506abd9a79c9645775075e07137a36c93ea09ee763dedde37fcf0360`), never rebuilt.

**Erratum 2 (claude-70, 2026-10-24).** The lines above replace the claude-64 pin `compiled_identity.source_sha256 == 177f1b3d…`, which no rebuild can reproduce.
- **Why.** `compiled.source_sha256` hashes the three ptxas resource reports in `OUT_DIR` (`ordinary-`, `decomposition-` and `baseline-reuse-resources.txt`). Those reports carry wall-clock lines (`ptxas info : Compile time = 10.726 ms`), so the hash changes on every build.
- **Measured.** A8 pod 1 rebuilt `fd409fcdf` on 2026-10-24 with the same nvcc 12.8.93 and `sm_120`. It reported `source_sha256 81651081…`, against `177f1b3d…` on 23 Oct, while every PTX, cubin, kernel, geometry and driver digest above was byte-identical to 23 Oct. Only the three `*_report_sha256` values differed (#2842 6379702498 and 6379753507).
- **So.** Step 1.1 pins the deterministic digests plus the one saved binary that every Stage 1 and Stage 2 step must use. The JSON keys are `compiled.*` and `audited_scope`, as `attention-audit-build-identity` prints them.

**Erratum (claude-64, 2026-10-23).** The two lines above replace the earlier pins `a5d59806…` and `46ddbb76d4f7…`, which this commit's binary does not report.
- **How they were measured.** `fd409fcdf` was built on an RTX 5090 pod with `--features cuda` (nvcc 12.8.93, `sm_120`), and `discrete-stack build-identity` was run, with no model run (Casey's approval: #2842, 6372197194). The results are in the HF store at `results/claude/craw-a8id-20261023/` (`a8-build-identity.json`, `SHA256SUMS`).
- **The geometry digest** matches 134 other records on main, including `verify_a8_address_cells_dose.py:453`.
- **`a5d59806…` is a different quantity.** It is the dose verifier's 3-file `compiled_source_sha256` (`verify_a8_address_cells_dose.py:1417`, and row "Bound Source Commit" above), not `compiled_identity.source_sha256`.
- **The source digest depends on the compiler.** It covers the nvcc PTX, so a different nvcc version can change it. Build with the bootstrap's nvcc 12.8.
- `cuda_context_created == false`, `training == "HELD"`
- `scope == "m1-attention-active-whole-interval-rejection-v1-relation-seed1-attempts0..16404"` (treatment) and `"m1-attention-active-whole-interval-rejection-v1-frozen-zero-seed1-attempts0..16404"` (control).

### Step 1.2 — Zero-Update Inherited Observations (`16` Records: `8` per Arm)

Produce the `8` `uor-r4.attention-audit-inherited-command/1` records per arm (`16` total) pinned by `QualificationPlan.inherited_observations`:
1. `canonical_initlock_c2`: `"$BIN" qualify-relation "$STAGE1_DIR/relation-qual"` (`9/9` PASS, `applied_updates == 0`, `training == "HELD"`).
2. `negative_k16`: `"$BIN" attention-build-identity docs/labs/address-cells-preflight-configs-2026-10-19/evidence/a16.json` (`native_exit == 2`, stderr `"invalid discrete model: attention requires K8, nearest-address objective, active-leaf initialization and interleaved read-every-8 schedule"`).
3. `negative_nonroot`: `"$BIN" init-corpus "$STAGE1_DIR/neg-b-config.json" "$STAGE1_DIR/neg-b-out"` (`native_exit == 2`, stderr `"invalid discrete model: discrete CUDA snapshot validation refused with status 6"`, report `status == "FAILED"`).
4. `initial_checkpoint`: `"$BIN" init-corpus CONFIG "$STAGE1_DIR/$ARM/init"` (`completed_batches == 0`, `accepted_updates == 0`, `epoch == 0`).
5. `unpinned_checkpoint`: `"$BIN" init-corpus UNPINNED_CONFIG "$STAGE1_DIR/$ARM/unpinned-init"` (`routes`, `gates`, `seed`, `epoch` equal to `initial_checkpoint`).
6. `initial_dev_evaluation`: `"$BIN" eval-corpus CONFIG "$STAGE1_DIR/$ARM/init/report.json" "$STAGE1_DIR/$ARM/eval"` (`b3_psucc == 1,351`, `b3_full == 4,244`, `top1 == 133`, `occupied_cells == 256`, `distinct_vectors == 4,096`, `tie_mass == 0`, `1,351 - psucc_reference_top1 > 500`).
7. `combine_probe`: `"$BIN" propose "$STAGE1_DIR/$ARM/snapshot.json" "$STAGE1_DIR/$ARM/probe-request.json" gate 0` (`any_candidate_cost_differs_from_identity == true`, `epoch == 0`, `applied == false`).
8. `relation_probe`: `"$BIN" propose "$STAGE1_DIR/$ARM/snapshot.json" "$STAGE1_DIR/$ARM/probe-request.json" gate 22` (`any_candidate_cost_differs_from_identity == true`, `epoch == 0`, `applied == false`).

### Step 1.3 — Qualification Plan Validation (`attention-audit-qualification-check-plan`)

Write `qualification.plan.json` (`schema = "uor-r4.attention-audit-source-qualification-plan/1"`, `scope = "m1-attention-active-audit-finite-source-qualification-v1"`, `status = "HELD_FINITE_QUALIFICATION_OWNER_ACTION_REQUIRED"`, `dose_sheet.sha256 = "fcb770e66d38f607d25e16dade14ac7a6223dd8af0ccc9ec22c80db71f2c3450"`, `anchor_csv_sha256 = "8c1b80e9b15e4e2bcc765037afa7800b785418be3e20b8cc80ce13ee2e32b3c1"`, `limits.protocol = "train61-synthetic1-interval64-two-arm-throughput-v1"`) with its 4 `uor-r4.attention-audit-qualification-authorization/1` pins (`qualification_fit`, `owner_scope_cost`, `pm_dispatch`, `independent_review`) and fresh `0600`-under-`0700` `resource_admission` (`uor-r4.attention-audit-resource-observation/1`), then run:

```sh
"$BIN" attention-audit-qualification-check-plan "<ABS>/qualification.plan.json"
```

Required output: `status: "QUALIFICATION_PLAN_METADATA_VALID"`, `cuda_context_created: false`, `dose_release: false`, `qualification_executed: false`.

**Template approval (owner ruling, Casey, 2026-10-24, #2842 6379923206: "Template approval (Recommended)").**

`validate_plan` refuses a `resource_admission` observation older than 300 s, at `check-plan`, at the owner release and at `attention-audit-qualify` (`attention_audit_qualification.rs`, `validate_resource(…, fresh = true)`). The plan payload covers that observation, so signing the exact final payload ahead of time is impossible. For this Stage 1 run only:

1. **Template.** Claude commits the plan template (`evidence/qualification.plan.template.json`). It is every plan field, with the live resource fields (`resource_admission`, `rate_usd_micros_per_hour`, `gpu_count`, `host_available_bytes`, `staging_archive_reserve_bytes`) set to the placeholder documented beside the file, together with its template hash. The template hash is the payload hash computed exactly as `payload_sha256` does, with those five fields also removed.
2. **Template sign-offs.** Each signer approves the template hash, with these bounds: `gpu_count == 2`, `rate_usd_micros_per_hour <= 2_500_000`, `host_available_bytes >= staging_archive_reserve_bytes + the binary's reserve rule`. The signers are a non-author reviewer (`TEST FITNESS: FIT` and `APPROVE_EXACT_QUALIFICATION_PLAN`), Kimi (`DISPATCH_FINITE_SOURCE_QUALIFICATION_ONLY`) and Casey (`APPROVE_EXACT_FINITE_QUALIFICATION_SCOPE_AND_COST`). Each approval is a GitHub comment that quotes the template hash.
3. **Binding.** On pod 2, the operator (Claude) restores the pod-1 tar and takes the fresh `uor-r4.attention-audit-resource-observation/1` (`0600` under `0700`). It checks the observation against the bounds, fills the five fields, computes the final payload hash, and writes the four `…-authorization/1` records with that hash. Each record's `receipt_url` is the signer's template-approval comment. The operator posts the final hash and the observation on #2842.
4. **Release and run.** `check-plan` runs, Casey types the release phrase personally (Step 1.4, `UOR_LAB` unset, over SSH), and Step 1.5 starts. All of this happens within 300 s of the observation. If the window lapses, the observation is retaken and step 3 is repeated; the template approvals still hold while the bounds hold.

### Step 1.4 — Staged Finite-Qualification Owner Release Command (Casey Personally, `UOR_LAB` Unset)

> **Owner boundary (CHARTER §5.1, D40):** No lab or subagent may type this command or phrase. Casey runs it personally in an interactive terminal with `UOR_LAB` unset after owner cost/scope approval and PM dispatch.

```sh
env -u UOR_LAB <ABS>/target/release/discrete-stack attention-audit-qualification-owner-release \
  <ABS>/qualification.plan.json \
  <ABS>/release-qualification
```

The binary prints `Finite source qualification; dose training remains held.` and prompts `Type exactly:`. The exact phrase from `crates/uor-r4-discrete/src/attention_audit_qualification.rs:1993-1996` is:

```text
I am Casey; release finite audited source qualification only for plan <PLAN_FILE_SHA256>
```

where `<PLAN_FILE_SHA256>` is `shasum -a 256 <ABS>/qualification.plan.json` (printed by `attention-audit-qualification-check-plan` as `plan_file_sha256` and echoed by `attention-audit-qualification-owner-release` after `Type exactly:`). This writes `<ABS>/release-qualification/owner.json` (`uor-r4.attention-audit-source-qualification-owner-decision/1`) and `<ABS>/release-qualification/admission.json` (`uor-r4.attention-audit-source-qualification-admission/1`).

### Step 1.5 — Execute Finite Qualification & Consumer-Verify Bundle (Lab Side)

```sh
"$BIN" attention-audit-qualify \
  docs/labs/a8-training-configs-2026-10-19/evidence/a8-relation-seed1.json \
  "<ABS>/release-qualification/admission.json" \
  "<ABS>/qualification-report"

python3 docs/labs/address-cells-consumer-verifier-2026-10-19/verify_a8_address_cells_dose.py \
  --verify-qualification-bundle "<ABS>/qualification-bundle.json"
```

If any qualification stage fails, if `baseline_admitted < 500`, or if `R = projected_added_gate_ns / ungated_two_arm_dose_ns > 0.15`, **stop before training (`INSTRUMENT_UNREACHABLE`, D31)** and replan.

---

## 3. Stage 2 Runbook & Executor — Audited Whole-Interval Rejection Dose (`m1-attention-active-whole-interval-rejection-v1`)

Stage 2 starts **only after** Stage 1 passes (`qualification.json` `5/5` stages PASS and `calibration.json` `R <= 0.15`), a `CostedDoseDecision` (`uor-r4.attention-audit-costed-dose-decision/1`) is bound with Casey's `APPROVE_EXACT_COSTED_DOSE_PLAN` and Kimi's `DISPATCH_EXACT_COSTED_DOSE_PLAN` attestations (`uor-r4.attention-audit-dose-cost-attestation/1`), and each arm's `AuditedPlan` (`uor-r4.attention-audit-owner-plan/1`) has an independent exact-plan review (`APPROVE_ACTIVE_WHOLE_INTERVAL_ADMISSION_PLAN`).

### Step 2.1 — Audited Dose Plans & Metadata Check (Lab Side)

Write one `uor-r4.attention-audit-owner-plan/1` per arm (`a8-audited-relation-seed1.plan.json` and `a8-audited-frozen-zero-seed1.plan.json`) binding:
- `sheet`: `docs/harness/prep/m1-attention-admission-preserving-DRAFT-2026-10-19.md` (`fcb770e66d38f607d25e16dade14ac7a6223dd8af0ccc9ec22c80db71f2c3450`)
- `parent_sheet`: `docs/harness/prep/m1-attention-a8-address-cells-learning-2026-10-19.md` (`3e622bebd7c468c6cd3d15a38486768c7902807998f71cae039a4341ab55da2b`)
- `anchor_csv_sha256`: `8c1b80e9b15e4e2bcc765037afa7800b785418be3e20b8cc80ce13ee2e32b3c1` (`AnchorPolicy { start_eligible_index: 1_049_856, count: 3_847, context: 192 }`)
- `policy`: `AuditPolicy { cadence: 64, max_batches: 16_404, final_tail_batches: 20, margin: 24, minimum_baseline: 500, reject_whole_interval: true, consecutive_full_rejections: 3, audits_per_arm: 258 }`
- `qualification`, `calibration`, `resource_admission`, `costed_dose_decision`, `independent_review`
- `transfer`: sealed fresh V2 (`tokens.u16` `0b7e754cfd242bf53145fe9cbca563be8503e6cd75df5449abb325b5c51ee1b8`, `response_mask.u8` `a1cda5295547bb641b1ea71ba4ebabb58d58c40e6c048cda91f35a878fecbf5e`, seal `df5d074a…`).

Validate both plans before presenting them to Casey:

```sh
"$BIN" attention-audit-check-plan "<ABS>/a8-audited-relation-seed1.plan.json"
"$BIN" attention-audit-check-plan "<ABS>/a8-audited-frozen-zero-seed1.plan.json"
```

Both must print `status: "PLAN_METADATA_VALID"`, `cuda_context_created: false`, `fresh_content_read: false`, `owner_release: "STILL_REQUIRED"`, `training: "HELD"`, `readiness_is_pretraining_only: true`.

### Step 2.2 — Staged Audited-Dose Owner Release Commands (Casey Personally, `UOR_LAB` Unset)

> **Owner boundary (CHARTER §5.1, D40):** Casey types these personally in an interactive terminal on the pod with `UOR_LAB` unset. No lab or subagent types them.

1. **Treatment arm (`Relation`, `LearnV1`, seed `1`):**

```sh
env -u UOR_LAB <ABS>/target/release/discrete-stack attention-audit-owner-release \
  <ABS>/a8-audited-relation-seed1.plan.json \
  <ABS>/release-a8-audited-relation-seed1
```

Exact confirmation phrase (`crates/uor-r4-discrete/src/attention_audited.rs:577`):

```text
I am Casey; release D40 for m1-attention-active-whole-interval-rejection-v1-relation-seed1-attempts0..16404 only with the pinned active whole-interval audit through attempted batch 16404
```

2. **Control arm (`FrozenZero`, `FrozenZeroV1`, seed `1`):**

```sh
env -u UOR_LAB <ABS>/target/release/discrete-stack attention-audit-owner-release \
  <ABS>/a8-audited-frozen-zero-seed1.plan.json \
  <ABS>/release-a8-audited-frozen-zero-seed1
```

Exact confirmation phrase (`crates/uor-r4-discrete/src/attention_audited.rs:577`):

```text
I am Casey; release D40 for m1-attention-active-whole-interval-rejection-v1-frozen-zero-seed1-attempts0..16404 only with the pinned active whole-interval audit through attempted batch 16404
```

### Step 2.3 — Execute Two-Arm Audited Dose & Consumer-Verify Bundle (Lab Side)

Run one arm per GPU (`CUDA_VISIBLE_DEVICES=0` and `CUDA_VISIBLE_DEVICES=1`):

```sh
CUDA_VISIBLE_DEVICES=0 "$BIN" attention-audit \
  docs/labs/a8-training-configs-2026-10-19/evidence/a8-relation-seed1.json \
  "<ABS>/release-a8-audited-relation-seed1/admission.json" \
  "<ABS>/run-relation-seed1"

CUDA_VISIBLE_DEVICES=1 "$BIN" attention-audit \
  docs/labs/a8-training-configs-2026-10-19/evidence/a8-frozen-zero-seed1.json \
  "<ABS>/release-a8-audited-frozen-zero-seed1/admission.json" \
  "<ABS>/run-frozen-zero-seed1"

python3 docs/labs/address-cells-consumer-verifier-2026-10-19/verify_a8_address_cells_dose.py \
  --verify-audited-dose-bundle "<ABS>/audited-dose-bundle.json"
```

---

## 4. Host Verification Receipts (`antigravity-19`)

1. **Immutable admission-preserving sheet SHA-256 (`shasum -a 256 docs/harness/prep/m1-attention-admission-preserving-DRAFT-2026-10-19.md`):**
   - Before & after: `fcb770e66d38f607d25e16dade14ac7a6223dd8af0ccc9ec22c80db71f2c3450` (`17,745` B, unchanged).
2. **`#2785` merge verification (`gh pr view 2785 --json state,mergedAt,mergeCommit`):**
   - `state = "MERGED"`, `mergedAt = "2026-10-20T16:48:41Z"`, `mergeCommit = "c6eb0526566b976ca8cc281c867a1d0c4c3f0b40"`.
3. **Sheet-bound source check (`verify_a8_address_cells_dose.py`):**
   - Running `python3 docs/labs/address-cells-consumer-verifier-2026-10-19/verify_a8_address_cells_dose.py --self-test` against later `main` (`767f653b2`) refuses `driver_source_sha256 = b13d4920d37a49dd0b1c2305ceaf406e51d5baadd058572cbf481f534bdf9b24 != 0f8ce4061996acdcce3ad4ae01d9e267f280703a77536eb6ecc24bfdc6e6df1d`, confirming the source-pinning guard (`NOW.md` lines 32–34) and why Step 1.1 checks out `fd409fcdf3bc43d607e768bae6f8cf403aa75901` before building or running `verify_a8_address_cells_dose.py`.
4. **Harness check (`uor-harness check`):**
   - `0 error` across all rules; `docs/harness/prep/m1-a8-registration-and-runbook-staging-2026-10-22.md` passes `check_sheet` with all `8/8` `REQUIRED_BLOCKS` present and `TEST FITNESS: FIT`.

---

## 5. Research Notebook Row (`m1-discrete-attention`)

| Date | Commit | Change | Metric / Result | Classification | Verdict |
| :--- | :--- | :--- | :--- | :--- | :--- |
| 2026-10-22 | `fd409fcdf` (`#2832`) | Registered prospective finite source-qualification sheet (`m1-a8-registration-and-runbook-staging-2026-10-22.md`, protocol `train61-synthetic1-interval64-two-arm-throughput-v1`, scope `m1-attention-active-audit-finite-source-qualification-v1`) with `TEST FITNESS: FIT` while preserving `m1-attention-admission-preserving-DRAFT-2026-10-19.md` byte-exact (`fcb770e6…`, `#2785`), and staged the two-stage qualification + audited-dose runbook, executor sequence, and exact D40 owner-release commands for Casey. | `fcb770e6…` SHA-256 unchanged (`17,745` B); `8/8` prep blocks `FIT`; `64/64` consumer verifier self-tests `PASS`; `$0.00` GPU spend | instrument / registration staging (not a mechanism negative) | **STAGED (`#2832` closed; GPU execution HELD pending owner cost approval, PM dispatch, and Casey's personal D40 release)** |

---

## 6. From Codex's 19 Oct audited-dose runbook

Codex wrote an audited-dose runbook on 19 October 2026 that never reached main. It is kept byte-identical as [`codex-audited-dose-runbook-2026-10-19.md`](codex-audited-dose-runbook-2026-10-19.md) (SHA-256 `59953debf2f855a10a9d18c4c42b68104a054384e1fecab49ebc8ed8de9630d3`), credit Codex. Most of it is covered by §1–§3 or superseded:

- **Superseded.** Its warning that `#2784` changed the dose sheet to `059037ea…` is resolved: the sheet on main hashes `fcb770e6…` again (restored in `#2785`, re-checked at `204cb0c9f`). Its hash table points at an outside folder (`uor-r4-local/a8-audited-dose-executor-packet-20261019`) that no longer exists. Its rate quote ($1.775/h) and 120-minute, $3.55 planning envelope are a dated 19 Oct observation; the Step 1.3 template bounds replace them. Its 300 s freshness handling is covered by the Step 1.3 template-approval procedure, and the 300 s rule still binds there.

The points below are not in §1–§3. They are Codex's wording, shortened. Nothing here has run.

1. **No resume and no reuse.** The consumed, unguarded A8 attempt stays closed. Do not resume or replay it, and do not reuse its owner release. There is no audited resume command; the old `attention` and `attention-resume` routes do not stand in for `attention-audit`. A qualification phrase cannot release either dose arm.
2. **The anchor manifest is path-sensitive.** The `TrainAnchorManifest` hash covers its serialized FilePin paths. Codex saw `9398efac…` with relative paths and `f85466ff339d8fb848fb1151a7d672fba5687296b81b1ad6fb39c36ddaca27de` with absolute `/workspace/uor-r4/local/driver/data/mw-balp/train` paths. Build the manifest on the pod with the existing host constructor and bind the one it writes; use neither example. The CSV pin (`8c1b80e9…`, §1) does not change. The second, `f85466ff…`, matches the pin in main's verifier `docs/labs/address-cells-consumer-verifier-2026-10-19/verify_a8_address_cells_dose.py` (#2783). The first, `9398efac…`, is not on main and is not re-checked.
3. **Leads for the missing plan inputs.** claude-70 found `accepted_initlock_disposition` and `residual_fit` nowhere. Codex names them as `#2029` comments 6242232012 (disposition) and 6241890920 (residual FIT), with the probe predicate `both_probes_any_active_candidate_cost_differs_v1`. It also says full same-cell reference capture is NOT_RUN before the run and deferred to the dose under the accepted scope. Both comment ids and the predicate are already on main, in `docs/labs/a8-dose-runbook-2026-10-19/README.md` and in `docs/integration/current-state.md`. There, 6241890920 is Kimi's binding sensitivity interpretation, and 6242232012 is the dispatch that used it, not a residual FIT and a disposition as Codex labels them. Whether they supply claude-70's two missing inputs is not settled here.
4. **Probe inputs.** The Step 1.2 probe request is the `.probe_request` field of `attention-panel` output (first 64 P_succ rows). The snapshot is the `.model` field of each arm's initial DriverCheckpoint, not the whole checkpoint.
5. **Same hardware for the dose.** The dose must run on the hardware class and GPU count that qualification timed (`2 x RTX 5090`). A different class or count invalidates the timing basis and needs requalification. A later pod of the same class still needs its own fresh resource observation.
6. **Dose disk is a separate bound.** The finite stage's reserve does not cover the dose. The dose writes 514 boundary checkpoints (parent checkpoints were about 6.97 MB each), restored continuations, progress records and audit outputs. Inventory that bound before dose admission.
7. **Custody and close.** If a review, cost decision or Casey's terminal cannot be done promptly, seal custody off the pod and close it rather than pay for a wait. At the end, seal all raw outputs, failed and setup attempts included, and verify SHA and readback before a normal `down`. Report startup and down times, the selected rate, uptime × rate as a cost proxy (not an invoice), and the outcome of each arm.
