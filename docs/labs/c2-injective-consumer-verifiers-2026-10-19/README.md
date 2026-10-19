# W3 BVG-K Stage C2 and Injective-Attention Pre-Flight Consumer Verifiers — 2026-10-19

- **Lab:** Antigravity (`ag1`, tooling/verifier lane)
- **Line:** M1 (`#2029`)
- **Scope:** Host-only deterministic consumer verifiers (`verify_bvgk_c2_qualify.py` and `verify_injective_attention_preflight.py`) for the two jobs dispatched in [`#2029` `6257564516`](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6257564516) and recorded in [`docs/labs/injective-preflight-and-c2-qualify-2026-10-19/`](../injective-preflight-and-c2-qualify-2026-10-19/README.md) (`#2733`).
- **Compute:** None (`$0`, host-only; creates no CUDA context, runs no model, and alters no hold).

## 1. Pinned Verifier Artifacts

| File | SHA-256 | Bytes | Self-Test Battery |
|---|---|---:|---|
| [`verify_bvgk_c2_qualify.py`](verify_bvgk_c2_qualify.py) | `db8e4c87b00a7baa4916c9593ae79b786018b37cd22c1896276754b6cfcce05e` | `33,547` | `19/19` PASS (`--self-test --repo-root .`) |
| [`verify_injective_attention_preflight.py`](verify_injective_attention_preflight.py) | `d30e907d492765461ccce9daf152a0b2eb84aa84da56ddbc4f444f337eb8381c` | `46,154` | `18/18` PASS (`--self-test`) |

## 2. W3 BVG-K Stage C2 Consumer Verifier (`verify_bvgk_c2_qualify.py`)

Verifies `bvgk-qualification.json` produced by `crates/uor-r4-discrete/src/bin/bvgk-qualify.rs` (`crates/uor-r4-discrete/src/bvgk_qualification.rs`) against [`docs/harness/prep/m1-bvgk-stagec2-block-qualification-2026-10-18.md`](../../harness/prep/m1-bvgk-stagec2-block-qualification-2026-10-18.md):

- **Top-level and hold invariants:** `schema == "uor-r4.discrete-spiralcore-fbs/2"`, `suite == "bvgk-block-qualification"`, `status == "PASS"`, `training == "HELD"`, `applied_updates_scope` fixture-only string, and `applied_updates == sum(c.applied_updates for c in controls[1:])`.
- **Host source & static apply-guard derivation (`--repo-root`):**
  - Recomputes `driver_source_sha256` (`f4509049add30b7b756cf5a9ddf97d5633ad4aa790028d8b1b210986945d09e9`) across the 11 driver source files in `crates/uor-r4-discrete/src/training.rs`.
  - Reconstructs `rows.cu` (`48d8e6e0…`), `rowsk.cu` (`0b1373a0…`), and `bvgk.cu` (`e646f016…`) directly from `discrete.cu`, `rows_source.rs`, `rowsk_source.rs`, and `bvgk_source.rs`, verifying prefix extension (`V0`) and exact match against `rows_image_sha256[0]`, `rowsk_image_sha256[0]`, and `bvgk_image_sha256[0]`, plus exact match of the 4-tuples `rows_image_sha256` and `rowsk_image_sha256` against C1 qualification.
  - Verifies statically that `bvgk_qualification.rs` contains zero calls to the guarded single-move `Device::apply` and commits only via `apply_block`.
- **Five required controls in exact order:**
  1. `v0_source_identity`: `status == "PASS"`, `bvgk_extends_rowsk_source == true`, `rowsk_extends_rows_source == true`.
  2. `rc_recarry_parity`: `status == "PASS"`, `cases == 64`, `lanes > 0` (`14,080`), `applied_updates == 0`, `commits == 0`.
  3. `ut_jr_dc_v3_block_sweep`: `status == "PASS"`, all four nested sub-controls (`ut_original_untouched`, `jr_joint_reference`, `dc_decision_coverage_v4_v5`, `v3_block_equals_sequential`) `PASS`, `cases == proposals == 804`, `UNDERPOWERED` blockers enforced (`empty > 0`, `single > 0`, `joint_wins > 0`, `joint_loses_or_ties > 0`, `multi_member_proposals > 0`, `v3_joint_cases > 0`), and exact partition identities:
     - `empty (140) + single (345) + multi_member_proposals (319) + refused_blocks (0) == proposals (804)`
     - `joint_wins (260) + joint_loses_or_ties (59) == multi_member_proposals (319)`
     - `commits (664) == single (345) + joint_wins (260) + joint_loses_or_ties (59) == v3_cases (664)`
     - `v3_joint_cases (260) == joint_wins (260)`
  4. `rf_refusals`: `status == "PASS"`, `cases == refusals == 9`, `applied_updates == 1`, `commits == 0`.
  5. `c1r_v2_scoring_parity`: `status == "PASS"`, `cases == 5,130`, `detail == {v2_blocks: 2028, v2_successes: 5130, v2_racy_refusals: 0}`, `applied_updates == 0`.
- **Diagnostic `throughput` (`TP`):** `qualified == false`, 20 positive raw samples per case, exact median (`sorted[10]`) and nearest-rank p95 (`sorted[18]`), `ns_per_coordinate == block_median_ns // k` for `k in {1, 4, 16}`, and `joint_verify_ran == (s_len >= 2)`.
- **Companion evidence (`--evidence-dir`):** verifies all 23 entries in `docs/labs/injective-preflight-and-c2-qualify-2026-10-19/evidence/SHA256SUMS` (`989a3684…`), no self-entry, `d29.txt` (6 `PASS:` lines + `Static scope PASS.`), `d29-selftest.txt`, and `source-head.txt` (`5c7276cfd8eac2fd8c820ec75395971162d6fe06`).

### Verification of Committed Receipt (`#2733`)

```sh
python3 docs/labs/c2-injective-consumer-verifiers-2026-10-19/verify_bvgk_c2_qualify.py \
  --self-test --repo-root . \
  --receipt docs/labs/injective-preflight-and-c2-qualify-2026-10-19/evidence/bvgk-c2-qualify-20261019/report/bvgk-qualification.json \
  --evidence-dir docs/labs/injective-preflight-and-c2-qualify-2026-10-19/evidence
```

Result: `verdict = "PASS"` on `bvgk-qualification.json` (`2fff942fa2b802e180c9bd87336692730d6beaf22f2c0bac8867afb0dea345f3`, `6,802` B), `23/23` `SHA256SUMS` files verified, `applied_updates = 3,419` (fixture-only), `training = "HELD"`.

## 3. Injective-Attention Runbook and Pre-Flight Consumer Verifier (`verify_injective_attention_preflight.py`)

Verifies both the staged runbook (`docs/labs/injective-attention-runbook-2026-10-19/`) and the GPU0 pre-flight receipt directory (`docs/labs/injective-preflight-and-c2-qualify-2026-10-19/evidence/injective-preflight-20261019/`):

- **Runbook & static contract (`--runbook-dir`):**
  - Verifies `evidence/SHA256SUMS` (`a6da44d0eaacd5f4f6bea2430c8418a9de185c4dcf09395b7e384669c687cab3`, no self-entry) and all 6 pinned files byte-for-byte.
  - Recomputes the little-endian `i32` word digest `table_sha256 == "e9e03bcd95b78632a7f18d1b27c07d37f0ba82c52eee692c37da011eadac8804"` and `distinct_vectors == 4,096` from the `131,072` words in `prime-zeta-v4096-b4-c192-k8.json` (`83c1380a…`).
  - Verifies that `injective-relation-seed1.json` (`7927b1dd…`) and `injective-frozen-zero-seed1.json` (`568adfd0…`) differ from their parent `rows_v1` configs (`52b32dc8…` and `6a035717…`) **only** in `codebook_initialization`.
  - Recomputes both `injective-83c1380a8f50-` scopes and D40 owner-release phrases and verifies all 7 items of Codex's documentary contract (`6257545219` / `6257642392`, closed in `#2730` + `#2731`) plus the registered sheet pin (`e47ce5af…`, `TEST FITNESS: FIT`).
- **Pre-flight receipt verification (`--preflight-dir`):**
  - Verifies `table-sha.txt`, `build-identity.json`, `relation/identity.json` (`config_sha256 = 0c3404d4…`, `lineage_sha256 = de807760…`), and `frozen-zero/identity.json` (`config_sha256 = 368f4285…`, `lineage_sha256 = 9ec2a6ad…`), including that `attention-build-identity` does not emit `scope`, `cuda_context_created == false`, and `training == "HELD"`.
  - Verifies `G-Init` checkpoint-0 summaries (`accepted_updates == 0`, `completed_batches == 0`, shared `model_sha256 = 134fd663…`) and `G-Init` negative (a) host refusal (`neg/a.stdout`).
  - Verifies `G-Occ` and raw `G-Cap` in `relation/census.json`: `occupied_cells = 256 > 16`, `within_cell_tie_mass = {dev_rows_targeting_non_representative: 0, tokens_in_alias_groups: 0}`, `distinct_vectors = 4,096`, `dev_alias_ceiling_rows = 14,287`, `host_cell_reconstruction = false`, `model_execution = false`.
  - Verifies zero-update DEV evaluation aggregates (`scored_rows = 14,287`, `target_admitted_rows = 427`, `top1_correct = 38`, `updates_applied_by_evaluation = 0`, `evaluation_mutates_model = false`) and `b3/summary.json` (`uor-r4.attention-preflight-dev-b3/1`, `VALIDATED_RETAINED_DEV_OBSERVATIONS`, full report hashes `9bc36546…` and `b4d45070…`), including the exact partition identity (`#2029` `6257784262`):
    - `psucc.rows (3,847) + complement.rows (10,440) == full.rows (14,287)`
    - `psucc.target_admitted (127) + complement.target_admitted (300) == full.target_admitted (427) == eval_aggregate.target_admitted_rows (427)`
  - Evaluates `B3_init(P_succ) >= 500` (`127 < 500` → `FAIL`) and `net_g_cap > max(500, top1_control(cp0) + 24)` (`127 <= max(500, 38 + 24) = 500` → `FAIL`), returning `verdict = "TERMINAL_PREFLIGHT_FAIL_INSTRUMENT_UNREACHABLE"`, `classification = "INSTRUMENT_UNREACHABLE"` (`mechanism_negative = false`), `dispatch_decision = "DO_NOT_DISPATCH"`, `owner_plans = "WITHHELD"`, and accepting the moot downstream controls (`g_init_neg_b_gpu_status6` and `inherited_battery`) as `"NOT_RUN_MOOT_AFTER_B3_FAIL"` per sheet §3 and [`#2029` `6257784262`](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6257784262).

### Verification of Committed Runbook & Pre-Flight Receipts (`#2726`/`#2730`/`#2731`/`#2733`)

```sh
python3 docs/labs/c2-injective-consumer-verifiers-2026-10-19/verify_injective_attention_preflight.py \
  --self-test \
  --runbook-dir docs/labs/injective-attention-runbook-2026-10-19 \
  --preflight-dir docs/labs/injective-preflight-and-c2-qualify-2026-10-19/evidence/injective-preflight-20261019 \
  --repo-root .
```
