# A8 address-cells learned-attention dose: staged runbook for owner release (2026-10-19)

- **Lab:** Claude (#2029 intent `6259261908`).
- **Sheet:** [`m1-attention-a8-address-cells-learning-2026-10-19.md`](../../harness/prep/m1-attention-a8-address-cells-learning-2026-10-19.md), **TEST FITNESS: FIT**, SHA-256 `3e622bebd7c468c6cd3d15a38486768c7902807998f71cae039a4341ab55da2b`. It is registered in `attention_learning.rs` (`A8_SHEET_SHA256`, #2755 `50ed2977`), and `registered_sheet` binds it only for the A8 table file `6600bdb1…`.
- **Operator split** is inherited from the injective runbook (#2029 `6257545219`):
  - Codex owns the executor packet and brings Casey his exact command;
  - Claude supplies the source, sheet and this runbook;
  - Kimi dispatches.
- **Status: STAGED. Nothing has run.** Training is HELD. Running needs:
  - Kimi's PM dispatch with an exact costed plan;
  - then Casey's release for each arm. Under D40, Casey types the phrase himself in a terminal with `UOR_LAB` unset; no lab types it.

## What the test is

The test is one variable against the PrimeZeta injective runbook (`docs/labs/injective-attention-runbook-2026-10-19/`): the pinned table becomes **`address_structured_v1`** (A8). It has the same 4,096 distinct vectors and 256 cells, but each token's cell is placed near its TRAIN predecessors' cells.

At checkpoint 0 (#2751), A8 admits 1,351 of 3,847 P_succ targets, against PrimeZeta's 127.

Two arms run in parallel, one GPU each:
- **Treatment:** `Relation{8,1}` LearnV1.
- **Control:** the matched FrozenZeroV1.

Both use seed 1, K = 8, objective A, `active_leaf_v1`, `interleaved_read_every8_v1`, `rows_v1` and the Dose 1 pins. The contrast is only Relation vs FrozenZero on the same A8 initialization.

## Pinned inputs

| File | What | SHA-256 |
|---|---|---|
| `../a8-training-configs-2026-10-19/evidence/a8-relation-seed1.json` | Treatment config. It is byte-identical to the pre-flight's `a8.json`. | `469a2ff0fe17a6a80f5875a077b3e6a9436553efdf1c6e7884541220553e9d5e` |
| `../a8-training-configs-2026-10-19/evidence/a8-frozen-zero-seed1.json` | Control config | `cf7ebcc85d2e463ae8f6691b63d90a0e005701cb38b8249a8d0d5460d8508479` |
| `../injective-attention-runbook-2026-10-19/evidence/parent-rows-relation-seed1.json` | The **unpinned** (seeded) initializer for the treatment config | `52b32dc8747148e0cd385d309dbfb1c3cc750569d11ff3932745313e134cc670` |
| `../injective-attention-runbook-2026-10-19/evidence/parent-rows-frozen-zero-seed1.json` | The unpinned initializer for the control config | `6a035717130a5e7106e19ca585b4de4383506b425ce403d8a13d7ab44622e1cf` |
| `../injective-attention-runbook-2026-10-19/evidence/prime-zeta-v4096-b4-c192-k8.json` | The PrimeZeta table. The A8 rebuild reads its config's spec (step 3). | `83c1380a8f507e51d0342faa1b089dea6a20561a03552208c093fa640afc124b` |
| `evidence/neg-b-a8-nonroot-table.json` | The G-Init negative (b) input, built from A8 (see below) | `e35bb48d800cd41e5e6b0b48d4b41b8b61fc147f4fde98c54e8dff919d15f90e` |

Both A8 configs differ from the injective runbook configs, and from the parent seeded `rows_v1` configs, **only** in `codebook_initialization`. This was checked by `jq` diff. The arms differ only in `relation_learning_policy`.

**Negative (b) input:**
- It is the A8 table (`6600bdb1…`, fetched from the HF archive `vast-archive/vast-56660582/`) with token 0's block 0 set to `[0,2,2,0,0,0,0,0]`.
- That word is a ±2 pair, but not among A8's 16 root blocks (checked: 0 matches).
- It is re-hashed (word digest `f9e1ea54345d6e8528801484062a84c4db4747888529f05a278f0776c2cd2db5`) and keeps 4,096 distinct vectors (awk count), so it passes every host check.
- The digest method was first checked against A8's declared `table_sha256` `4122cbab…`, and reproduced it exactly.

**Scopes**, from `attention_learning::scope`, which uses the table file SHA prefix `6600bdb1306d`:
- treatment: `m1-attention-learning-injective-6600bdb1306d-relation-k8-seed1-batches0..16404`
- control: `m1-attention-learning-injective-6600bdb1306d-relation-frozen-zero-k8-seed1-batches0..16404`

## Runbook

**Lab side, after PM dispatch.** Use one Vast 2 × RTX 5090 pod via `uor-pod up --ref <merged main SHA>`, and record the head. `discrete-stack` is the `uor-r4-discrete` CLI.

1. **Build.** Run `cargo build --release --locked -p uor-r4-discrete --features cuda`, and record `driver_source_sha256`. Every plan binds it, so build nothing else in between.
2. **Data.** Extract the HF data tars `ft-dev`, `ft-balp` and `tokenizer.json` into `/workspace/uor-r4/local/driver/data/`. Copy the PrimeZeta table to `…/data/injective/`. Then `sha256sum -c` the six pins of `a8-relation-seed1.json`:
   - TRAIN `3dee858f…` / `b5e7352d…`;
   - DEV `0919e90f…` / `a838bb1a…`;
   - tokenizer `d36d3e87…`;
   - and, once step 3 has placed it, the A8 file `6600bdb1…`.
   
   Any mismatch is an instrument blocker; stop.
3. **Table.** Run `address-cells-codebook docs/labs/address-cells-preflight-configs-2026-10-19/evidence/p8.json OUT structured`. That is the invocation #2751 ran.
   - Required output: `file_sha256 = 6600bdb1…`, `table_sha256 = 4122cbab…`, `counts_sha256 = 755a5a81…`.
   - Copy OUT to `/workspace/uor-r4/local/driver/data/address-cells/address-structured-v1.json`.
   - Any other value is an instrument blocker; stop.
4. **Configs.** Run `discrete-stack attention-build-identity CONFIG` on both A8 configs. Each must succeed: it validates the scope, injectivity and the K = 8 pin.
5. **Pre-run gates** (sheet §3; GPU, zero updates; both arms). Run `init-corpus`, `eval-corpus`, `attention-dev-b3`, `attention-panel`, `attention-export-token-cells`, then `coverage-census --token-cells … --bar-rows 500`, as in `docs/labs/address-cells-preflight-2026-10-19/evidence/asc-battery.sh`. Then:
   - **G-Init, table:** the checkpoint-0 `.model.table` equals the pinned file's `.table` word for word, and the digest is `4122cbab…`.
   - **G-Init, unpinned comparison (NOT_RUN in #2751; required here):** run `init-corpus` on each parent `rows_v1` config. Compare its checkpoint-0 `.model.routes`, `.model.gates`, `.model.seed` and `.model.epoch` with the matching A8 arm's. All four must be equal.
   - **G-Init, negative (a):** an edited A8 file with stale hashes is refused on the host.
   - **G-Init, negative (b):** `evidence/neg-b-a8-nonroot-table.json`, re-pinned by file SHA in a copy of the treatment config, passes the host checks and is refused on the GPU with status 6.
   - **G-Occ:** occupied cells > 16 (256 expected), tie mass 0, no split alias group.
   - **Reproduction gate (D31), both arms:** B3_init on P_succ is 1,351 / 3,847, on the full panel 4,244 / 14,287, and top-1 on the full panel is 133. These are byte-identical inputs, so any difference is an instrument fault.
   - **Bar (i):** `top1_control(P_succ, cp0) + 24`, from the FrozenZero arm's own evaluation. It is graded on the host from stored outputs: the count of P_succ rows (joined by `(target_position, target_token)` to the panel) where `top8[0] == target_token`.
   - **The gate:** net G-Cap > max(500, Bar (i)).
   - **Inherited contract:** the exact commands are in the next subsection.

   **Inherited controls: exact commands, pass predicates and prior receipt** (grandparent `m1-learned-attention-link-2026-10-17.md` §7.8 item 3 and §7.12.1 items 2–3; #2029 `6259335138`). Codex's Battery3 ran this same set on the seeded lineage and passed it (`docs/labs/attention-preflight-2026-10-17/`, report `80cae1fc…`, verdict `6243075217`). Only the snapshot changes here.

   | Control | Exact command at the build head | Pass predicate | Battery3 receipt (seeded) |
   |---|---|---|---|
   | Bind the config | `discrete-stack attention-build-identity CONFIG` (step 4) | exit 0 | build identity PASS |
   | Gate22 sensitivity (level-0 `e=2` relation trit) | `discrete-stack propose SNAPSHOT.json REQUEST.json gate 22` | **Bound predicate:** Kimi PM's binding sensitivity interpretation (#2029 `6241890920` §3, used by dispatch `6242232012`). The probe is sensitive when `any_candidate_cost_differs_from_identity == true`, with `epoch == 0`, zero applies, and `candidate_losses` retained. Codex's packet calls this `both_probes_any_active_candidate_cost_differs_v1`. **The receipt retains `before`, `after`, `candidate` and `candidate_losses` verbatim**, so the grandparent §7.12.1 reading (inert iff `before == after` and `candidate == 0`) is also reported. That reading is not substituted for the bound one. | "Fixed Gate22 and Gate0 probes … predicates true; epoch0 / applied false" |
   | Gate0 sensitivity (block-0 leaf combine) | `discrete-stack propose SNAPSHOT.json REQUEST.json gate 0` | the same bound predicate and retained fields as Gate22 | the same row |
   | C2 ACTIVE / INERT, and the relation suite | `discrete-stack qualify-relation NEW_DIR` | `NEW_DIR/relation-controls.json` has `status: "PASS"`, `registered: 9`, `passed: 9`, `not_run: 0`, `applied_updates: 0` | "9 registered / 9 attempted / 9 PASS … including the registered C2 / INIT LOCK requirements" |
   | **INIT LOCK, the registered record** | the same `qualify-relation NEW_DIR` run | In `relation-controls.json`, the control `active_leaf_initialization_and_matched_off_path` passes, and each of its `init_locks[]` records (part `INIT LOCK`) has `treatment_control_snapshot_bit_identical: true`, `reload_bit_identical: true`, `plus_one_combine_indices: [0,5,10,15]` and `applied_updates: 0`. The record's `snapshot_sha256`, `model_storage_sha256` and `model_snapshot_sha256` are retained (`relation_qualification.rs` INIT LOCK block). It covers the generated initializer at the registered V4096 / B4 / C192 / K8 shape. | "9 registered / 9 PASS … including the registered C2 / INIT LOCK requirements" |
   | INIT LOCK, A8 arm supplement (not a substitute for the record) | on each arm's checkpoint 0: `jq -e '.model.gates as $g \| ($g\|length)==45 and ([0,5,10,15]\|map($g[.]))==[1,1,1,1] and ([range(0;20)]-[0,5,10,15]\|all($g[.]==0)) and ($g[20:45]\|all(.==0))' INIT/report.json`. Then, across the two arms: `cmp <(jq -cS .model RELATION/init/report.json) <(jq -cS .model FROZEN/init/report.json)` | both exit 0 | On A8 checkpoint 0 (#2751, HF archive) the gate predicate already holds. The cross-arm `cmp` is the pinned-table analogue of the record's treatment / control bit-identity. |
   | Reference battery (§7.12.1 item 1) | **Executable now:** the `init-corpus` / `eval-corpus` / `attention-dev-b3` / `attention-panel` runs above (B3_init on P_succ, P_comp and the full panel). **DEFERRED_TO_DOSE:** the T1 / T1b / T2 / U_succ reference, `attention-cell-reference-scorer preflight-dev --panel PANEL --capture CAPTURE --eval EVAL --d34 D34_RECORD [--token-cells … --checkpoint … --train-tokens … --train-mask … --dev-tokens … --dev-mask …]`. It needs (a) a `--capture` from `attention-capture-dev`, a separately released zero-apply GPU slot capture (`main.rs:172`), and (b) the sealed D34 successor-table record, which is not in the repository. | B3 rows are as above. The reference row is **NOT_RUN pre-run**, with blockers (a) and (b). Battery3 also deferred the 224-slot capture and full same-cell scorer ("NOT_RUN"). It runs inside the dose, or earlier if a capture release and the D34 record are supplied. | "Initial DEV evaluation … Initial B3 admission" rows; scorer "NOT_RUN" in Battery3 |

   - **INIT LOCK predicate, checked on the host:** it exits 0 on the archived A8 and P8 checkpoint-0 reports, and exits 1 on two mutants: relation trit `gates[22]=1`, and stray combine `gates[3]=1`.
   - **The two `propose` inputs:**
     - `SNAPSHOT.json` is the `.model` field of the arm's `init-corpus` `report.json`. `propose` reads a `Snapshot`, not the `DriverCheckpoint`.
     - `REQUEST.json` is `{"windows": [[Identity…]…], "targets": […]}`: at most 64 real P_succ rows, each with its 192-token causal window and its target (§7.12.1 item 2).
     - Reuse Battery3's request builder from Codex's packet; this record does not invent one.
     - `propose` computes candidate losses only. It never calls `Device::apply`.
   - **Order and stop:** Gate22 and Gate0 run on **both** arms' checkpoint 0. `qualify-relation` is source-level, so it runs once per build head.
     - An inert probe or any non-PASS control stops before training (D31). That is an activation or instrument fault, not a mechanism negative.
     - The §7.8 item 2 control-lock ledger proof (45 coordinates, 1,130 relation refusals) is a Q1 and end-of-dose check, not a pre-run gate.

   Post the gate receipt on #2029. Any FAIL stops the runbook for diagnosis (D31), and no plan is written.
6. **Plans and review.** These are the same as the injective runbook's step 5:
   - one `uor-r4.attention-learning-owner-plan/2` per arm, with a fresh absolute `run_state_directory`;
   - `transfer` set to the fresh V2 source: tokens `0b7e754c…`, mask `a1cda529…`, seal `df5d074a…`;
   - one `APPROVE_ATTENTION_LEARNING_ADMISSION_DIFF` review per arm, with the seat named and its findings posted;
   - `discrete-stack attention-check-plan PLAN` must print `PLAN_METADATA_VALID` and `owner_release: STILL_REQUIRED`.
   
   The plan's `sheet` pin must be the A8 sheet (`3e622beb…`); `registered_sheet` refuses any other.

**Owner step (Casey, personally; about two minutes per arm).** On the pod, with `UOR_LAB` unset:

```
env -u UOR_LAB <ABS>/target/release/discrete-stack attention-owner-release <ABS>/a8-relation-seed1.plan.json <ABS>/release-a8-relation-seed1
```

The tool prints the plan, then `Type exactly:`. For the treatment arm the phrase is:

```
I am Casey; release D4 for m1-attention-learning-injective-6600bdb1306d-relation-k8-seed1-batches0..16404 only through batch 16404 with the pinned DEV transfer and relation-off evaluations
```

Repeat with `a8-frozen-zero-seed1.plan.json` and `release-a8-frozen-zero-seed1`:

```
I am Casey; release D4 for m1-attention-learning-injective-6600bdb1306d-relation-frozen-zero-k8-seed1-batches0..16404 only through batch 16404 with the pinned DEV transfer and relation-off evaluations
```

The phrases come from `attention_learning.rs:408` (`format!("I am Casey; release D4 for {} …", plan.scope)`), and the scopes come from line 186. If the tool prints anything else, type what the tool prints, not this page, and report the difference.

**Lab side, run and transfer:** these are the injective runbook's steps 6–8, unchanged.
- `discrete-stack attention CONFIG <release>/admission.json NEW_REPORT_DIR`, one arm per GPU. Every attempt uses a fresh output directory, and a Q1 stop is not resumable.
- `attention-transfer` runs only after a genuine sealed paired DEV H_geom decision **and** a typed PM transfer authorization.
- Archive to the HF store and iCloud, then delete the pod.

## Cost (sheet §9; assumptions, not a quote)

The sheet inherits the parent's planning estimates: about 6.0 h for the full dose (roughly $11.4 at about $1.89/h for 2 × 5090) and about 1.5 h for a Q1 stop (roughly $2.8). The pre-run gates add under 30 min (#2751's battery took 5 min of GPU time). Kimi's costed plan carries the actual quote. The owner caps (≤ 4 pods, ≤ $8/h) and the 30 GiB disk floor are hard stops.

## Not covered by this runbook

- **Seed 2** is reproducibility only, with its own later release.
- **The plan JSONs, reviews, release directories and run receipts** are produced on the pod in steps 4–6 and the run. They are not committed here, because the plans bind the exact build's `driver_source_sha256`.

## Post-merge correction (2026-10-19)

#2757 merged before three review fixes. This section supersedes the inherited-controls rows above; no bar, arm or run changes.

- **Gate22/Gate0 predicate.** The named prior-accepted predicate is `both_probes_any_active_candidate_cost_differs_v1` (dispatch `6242232012` / residual FIT `6241890920`), not the bare field alone. Record `before`, `after` and `candidate` for each probe; both probes inert (no active candidate-cost difference) is a stop/fault, not a pass.

- **INIT LOCK.** The canonical check is the `qualify-relation` record, not the jq arm-gates predicate alone: `relation-controls.json` control `active_leaf_initialization_and_matched_off_path` must be PASS, and the stdout record `part: INIT LOCK` must carry `plus_one_combine_indices: [0,5,10,15]`, `treatment_control_snapshot_bit_identical: true` and `reload_bit_identical: true`. The jq predicate is supplementary.

- **Reference battery.** Use `attention-cell-reference-scorer preflight-dev --panel PANEL --capture CAPTURE --eval EVAL --d34 D34 --token-cells token_cells.u16 --checkpoint CHECKPOINT --train-tokens ... --train-mask ... --dev-tokens ... --dev-mask ... --out OUT`. The sealed D34 record SHA is `2717d34bcfcdd1486991925ecab066266a6b56ba34b89727c37f78c821fe3bec` and is outside the repo, so the packet must fetch and bind it. If the pre-run window cannot produce the 224-slot `attention-capture-dev` outputs (the prior battery deferred them to DOSE), record the reference component as `NOT_RUN/DEFERRED_TO_DOSE`; do not claim metadata equals the full reference battery.
