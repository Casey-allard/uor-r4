# A8 Admission-Anchor Controller Finite CUDA Qualification and Restoration (`#2830`)

- **Lab:** `antigravity` ([`#2830`](https://github.com/UOR-Foundation/uor-r4/issues/2830), PR [`#3180`](https://github.com/UOR-Foundation/uor-r4/pull/3180))
- **Milestone:** M1 (`#2029`) — `m1-discrete-attention`
- **Status:** **FINITE QUALIFICATION CONTROLS PASS (`5/5` stages on both `relation` and `frozen-zero` arms + synthetic restoration + bookkeeping envelope); READINESS GATE `REPLAN_BEFORE_TRAINING` (`R = 78.41% > 15%`, `dose_qualification: WITHHELD`, `mechanism_negative: false`).**
- **Registered Finite-Qualification Sheet:** [`../../harness/prep/m1-a8-registration-and-runbook-staging-2026-10-22.md`](../../harness/prep/m1-a8-registration-and-runbook-staging-2026-10-22.md) — **TEST FITNESS: FIT** ([`#2832`](https://github.com/UOR-Foundation/uor-r4/issues/2832#issuecomment-6343653097))
- **Pre-run Review (D67):** **PRE-RUN REVIEW: PASS** ([`#2830` comment `6393777094`](https://github.com/UOR-Foundation/uor-r4/issues/2830#issuecomment-6393777094))
- **Immutable Admission-Preserving Dose Sheet:** [`../../harness/prep/m1-attention-admission-preserving-DRAFT-2026-10-19.md`](../../harness/prep/m1-attention-admission-preserving-DRAFT-2026-10-19.md) — SHA-256 `fcb770e66d38f607d25e16dade14ac7a6223dd8af0ccc9ec22c80db71f2c3450` (`FIT_SHEET_SHA256`)
- **Sheet-Bound Source & Binary:** `fd409fcdf3bc43d607e768bae6f8cf403aa75901` (`#2782`), `discrete-stack` `executable_sha256 = fb4493253d3e26dc7cc5fbcbcf7fa4dc882d93a5c7b02d340b25918a2d25b00c`, `source_sha256 = 81651081145fd4712046a8158a26c3bd6940ff5a60e5cdddf666057cb9fc0084`, `driver_source_sha256 = 0f8ce4061996acdcce3ad4ae01d9e267f280703a77536eb6ecc24bfdc6e6df1d`, `kernel_sha256 = 5cd0e73241fd895f2dd0004c83af3d1eb9f89e00ec2e933c1acc44fc7127bfd6`, `geometry_sha256 = 46ddbb76b7fbe6ac6222ea24779fc10055b1d6cea6a8da786895676fd29fe327`
- **Pod & Spend:** `vast-57614891` (`1 x NVIDIA GeForce RTX 5090`, driver `580.95.05`, CUDA `12.8`, `$0.908/h`, leased `2026-10-25T05:13:02Z` to `2026-10-25T05:29:15Z`, ~`16.2 min`, cost proxy `~$0.25`, `0` pods left running)
- **HF Store Custody:** `caseyallard/uor-r4-store:results/antigravity/a8-finite-cuda-qual-2026-10-25/`
  - `a8-finite-cuda-qual-20261025.tar.gz`: SHA-256 `a9c1228bc508eb12f61a82b35dd2f18722edee64bb1da4270a1fe37a01a2b2b6` (`5,043,721` B)
  - `SHA256SUMS`: SHA-256 `8669fc6d4b47f522bdd0a7cc0fd6525321944ce3c8ccbf6a7a87a5d51042e4d6`
- **Committed Verification Summary:** [`qualification-summary.json`](qualification-summary.json) (generated and verified by `a8-qualify-stage verify`)

---

## 1. Finite CUDA Qualification Controls (`PASS`)

All finite qualification and restoration controls defined in [`m1-a8-registration-and-runbook-staging-2026-10-22.md`](../../harness/prep/m1-a8-registration-and-runbook-staging-2026-10-22.md) executed on CUDA under the sheet-bound `fd409fcdf3bc43d607e768bae6f8cf403aa75901` binary (`fb449325…`) and **PASSED**:

### 1.1 Synthetic Restoration Control (`synthetic-control.json`)
- **SHA-256:** `69eb5866e3e66d658602a463ff236a2f59647347764a97e569ada86b17f762ea`
- **Schema:** `uor-r4.attention-audit-synthetic-restore/1`
- **Result:** `status = "PASS"`, `apply_calls = 1`, `accepted_updates = 1`, `stale_apply_refusals = 1`, `stale_carry_refusals = 1`, `observed_restored_epoch = 0`, `fresh_binding = true`.

### 1.2 Host Metadata & Bookkeeping Envelope (`host-metadata-bookkeeping/measurement.json`)
- **SHA-256:** `95c64e51c67504cb1c2d1382b698fd5df5d249c07439cb20068754c311b8d301` (`142` pinned step/boundary artifacts verified)
- **Envelope checks:**
  - `maximum_step_shape_bytes = 2,132 <= step_envelope_bytes (16,384)`
  - `maximum_boundary_shape_bytes = 37,381 <= boundary_envelope_bytes (65,536)`
  - `progress_history_entries = 16,922`, `maximum_progress_shape_bytes = 1,202,016`
- **Recorded timing samples:**
  - `128` step write/fsync/readback samples (`max_step_sample_ns = 2,909,154 ns` = `2.909 ms`)
  - `pass_boundary_ns = 25,491,179 ns` (`25.491 ms`)
  - `breach_restore_ack_ns = 27,105,652 ns` (`27.106 ms`)

### 1.3 Five-Stage Arm Qualification (`relation` and `frozen-zero`)

| Stage / Metric | Treatment Arm (`relation`, `LearnV1`) | Control Arm (`frozen-zero`, `FrozenZeroV1`) | Bar / Requirement | Status |
| :--- | :--- | :--- | :--- | :--- |
| **`qualification.json` SHA-256** | `73876b488bdb2123457d46fd28b0e4b1296cc398a930db5542b67f72cc9fc2d4` | `312b5370522afddbf46675220ba5e05a93e1eab5e0cf75a2f448aeb59032badc` | Schema `uor-r4.attention-audit-qualification/1` | **PASS** |
| **1. `canonical_initlock_c2`** | `e16b10b4dfed08ad51979925a810e82dded5089d148075af3828e957a0644f90` (`9/9` controls, `0` updates) | `226cf4e8546bb226fdbd47b3621f906ff7583330c564a9c5d048de4d144e356c` (`9/9` controls, `0` updates) | `9/9` C2 controls PASS, `0` updates | **PASS** |
| **2. `required_negatives`** | `72b3da82860e4f4c65c5a39d0c1f60072451aaee18b4a34d3eec6576aaff124a` (`native_exits: [2, 2]`) | `8aff96274fab2fdfd3567eb7efeb118b155c06c6f9e27e24b897fb4a256ec604` (`native_exits: [2, 2]`) | Both negative controls (`K16` and non-root `A8`) exit `2` with required reasons | **PASS** |
| **3. `g_occ`** | `a6ec05082dcf4e8d69d02c71a65156d983fde74750dad12829c55dbe1324e3df` (`b3_psucc = 1,351/3,847`, `b3_full = 4,244/14,287`, `top1 = 133`, `bar_i = 86`, `psucc_reference_top1 = 194`, `occupied_cells = 256`, `distinct_vectors = 4,096`, `tie_mass = 0`) | `34ab202abe363030357d3a3f0f015c4d1136042c61d31d2acaadeb385beae778` (`b3_psucc = 1,351/3,847`, `b3_full = 4,244/14,287`, `top1 = 133`, `bar_i = 86`, `psucc_reference_top1 = 194`, `occupied_cells = 256`, `distinct_vectors = 4,096`, `tie_mass = 0`) | `b3_psucc = 1,351 >= 500`, `1,351 - 194 = 1,157 > 500`, `256` cells, `4,096` vectors, `tie_mass = 0` | **PASS** |
| **4. `sensitivity`** | `b214528313a1c9c6f4365163791683b145e88169de0dda2a7352b27e08ac193c` (`combine_sensitive = 1`, `relation_sensitive = 1`) | `c88565e6a71cfa05e7fc80bae251ba6db8fdd4120f6cb807c00b4a5a9fba3452` (`combine_sensitive = 1`, `relation_sensitive = 1`) | `both_probes_any_active_candidate_cost_differs_v1` (`[1, 1]`) | **PASS** |
| **5. `audited_restore_epoch_cursor`** | `e0b3fdbf5c7e00490cdda4e05397f85d36490e79038bdc9d430f4aa5f07f6c95` (`384` attempts, `58` accepted updates before restore, restored to `epoch = 0`, cursor advanced to `completed_batches = 384`, `next_target = 24,576`) | `7f76f040ce8523cec54b9ac56db8f2461a0d4869a7093559ee670595ddace23b` (`384` attempts, `54` accepted updates before restore, `0` relation gate acceptances, restored to `epoch = 0`, cursor advanced to `completed_batches = 384`, `next_target = 24,576`) | Exact storage & snapshot restore (`3a06d4ba…` / `a66a2016…`), stale handles refused, continuation distinct, TRAIN anchor identical before/after restore | **PASS** |
| **`production-proof.json` SHA-256** | `85fd0e9ea9f69ffaefca5f9aa9c6fe63e6519a5f528aab24184b4c75de4bd780` | `bfbe30f8d109763aa8e8212188433bc7e6aa22edcc2467edc31fec453fc7d775` | All 8 checkpoint/anchor/intent/observation pins verified | **PASS** |
| **Fixed TRAIN Anchor (`3,847` rows) Before & After Restore** | `scored_rows = 3,847`, `target_admitted_rows = 1,211`, `admitted_strict_win_rows = 43`, `top1_correct = 43`, `top8_correct = 147`, `objective_loss_numerator_sum = 3,656,320` (identical before vs after restore) | `scored_rows = 3,847`, `target_admitted_rows = 1,211`, `admitted_strict_win_rows = 43`, `top1_correct = 43`, `top8_correct = 147`, `objective_loss_numerator_sum = 3,656,320` (identical before vs after restore) | `target_admitted_rows >= 500` and exact aggregate + snapshot equality before vs after restore | **PASS** |

---

## 2. Recorded CUDA Timing & 15% Readiness Gate (`timing-protocol.json`)

- **Timing Protocol Artifact (`timing-protocol.json`):** SHA-256 `d97bb13a8f30b24c2c4ad99c63308adbd4f537e31d776f697eeb5036aa651f86` (`792` raw step/checkpoint/anchor pins verified)
- **Protocol:** `train61-synthetic1-interval64-two-arm-throughput-v1` (`timer_scope = "synchronized-device-snapshot-seal-fsync-readback-lifecycle-v1"`, `cache_policy = "fresh-device-per-arm-no-unreported-warmup-v1"`)

### 2.1 Measured 384-Step Throughput by Coordinate Family (`rows_v1`)

| Arm | Route Steps (`336` -> `14,354`) | Non-Relation Gate Steps (`23` -> `920`) | Relation Gate Steps (`25` -> `1,130`) | 384-Step Total | Extrapolated 16,404-Step Arm Time |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **`relation` (`LearnV1`)** | `91,860,297,786 ns` (`273.39 ms/step`) | `3,244,655,721 ns` (`141.07 ms/step`) | `10,865,641,990 ns` (`434.63 ms/step`) | `105.97 s` (`17.66 s / 64 steps`) | `4,545,144,300,326 ns` (`4,545.14 s`) |
| **`frozen-zero` (`FrozenZeroV1`)** | `36,034,291,782 ns` (`107.25 ms/step`) | `2,355,685,424 ns` (`102.42 ms/step`) | `2,895,767,894 ns` (`115.83 ms/step`) | `41.29 s` (`6.88 s / 64 steps`) | `1,764,572,389,670 ns` (`1,764.57 s`) |
| **Combined Two-Arm Ungated Dose (`projected_ungated_two_arm_ns`)** | — | — | — | `147.26 s` | **`6,309,716,689,996 ns` (`6,309.72 s` = `1.753 h`)** |

### 2.2 Measured Audit Gate & Restoration Cost Decomposition

| Component | Single-Sample Measurement | Full-Dose Multiplier | Projected Two-Arm Cost (`ns` / `s`) | Share of Ungated Dose (`6,309.72 s`) |
| :--- | :--- | :--- | :--- | :--- |
| **61-Batch (`3,847`-Row) TRAIN Anchor Eval + Snapshot Seal/Readback (`projected_gate_ns`)** | `4,443,590,804 ns` (`4.444 s`) | `516` audits (`258 / arm`) | `2,292,892,854,864 ns` (`2,292.89 s`) | `36.34%` (`3,634 bps`) |
| **Worst-Case Full Restoration + Breach Ack (`projected_restore_ns`)** | `4,926,717,208 ns` (`relation`, vs `3,984,920,202 ns` `frozen-zero`) + `27,105,652 ns` ack | `514` intervals (`257 / arm`) | `2,546,264,950,040 ns` (`2,546.26 s`) | `40.35%` (`4,036 bps`) |
| **Step Bookkeeping Envelope (`projected_step_bookkeeping_ns`)** | `2,909,154 ns` (`max` of `128` samples) | `32,808` steps (`16,404 / arm`) | `95,443,524,432 ns` (`95.44 s`) | `1.51%` (`151 bps`) |
| **Boundary Bookkeeping Envelope (`projected_boundary_bookkeeping_ns`)** | `25,491,179 ns` | `516` boundaries (`258 / arm`) | `13,153,448,364 ns` (`13.15 s`) | `0.21%` (`21 bps`) |
| **Total Added Gate Overhead (`projected_added_gate_ns`)** | — | — | **`4,947,754,777,700 ns` (`4,947.75 s`)** | **`78.41%` (`7,842 bps` ceil vs `1,500 bps` bar)** |
| **Zero-Restore Gate Overhead (`projected_added_gate_ns - projected_restore_ns`)** | — | — | `2,401,489,827,660 ns` (`2,401.49 s`) | `38.06%` (`3,807 bps` ceil) |
| **Shared Setup (`shared_setup_ns`, excluded from ratio)** | `926,807,762 ns` (`0.927 s`) | `1` | `926,807,762 ns` (`0.927 s`) | excluded |

### 2.3 Diagnostic Readiness Stop (`failure.json`)
- **SHA-256:** `79be9c6cc128e28dd9b437cd3240cc4a7bbc047237bfc5a7f483fe4fd82d7feb`
- **Payload:**
  ```json
  {
    "automatic_retry": false,
    "dose_qualification": "WITHHELD",
    "error": "invalid discrete model: gate readiness exceeds 15 percent; replan BEFORE training",
    "mechanism_negative": false,
    "remaining_stages": "NOT_RUN",
    "status": "FAILED_OR_NOT_RUN"
  }
  ```
- **Why `R > 15%` occurred (instrument / cadence diagnosis for `#2831`):**
  1. The `rows_v1` counterfactual kernel sped up 64-step training intervals to `17.66 s` (`relation`) and `6.88 s` (`frozen-zero`), so the entire two-arm 16,404-batch ungated dose takes `6,309.72 s` (`1.753 h`).
  2. Each 61-batch (`3,847`-row) TRAIN-anchor audit plus synchronous JSON checkpoint seal/fsync/readback takes `4.444 s`. Running that audit every `64` steps (`258` times per arm, `516` total) costs `2,292.89 s` (`36.34%` of the ungated dose), already above `15%` even with `0` restores (`38.06%` including bookkeeping).
  3. In addition, the conservative readiness formula in `attention_audit_qualification.rs` multiplies the worst-case restoration sample (`4.954 s`) by all `514` intervals (`2,546.26 s`, `40.35%`), even though the protocol stops after `consecutive_full_rejections = 3` consecutive rejections.
  4. Replanning the audit cadence (for example, auditing every `256` steps = `66` audits per arm instead of `258`, or separating full JSON checkpoint sealing from device-resident anchor scoring) and bounding the projected restore count in `m1-a8-full-dose-rerun` (`#2831`) reduces the gate overhead below `15%`.

---

## 3. Host Reproduction & Verification Commands

```sh
# Run unit tests (including committed qualification-summary.json verification)
~/.cargo/bin/cargo test --locked -p uor-r4-discrete --bin a8-qualify-stage --no-fail-fast

# Re-verify the extracted HF store bundle (caseyallard/uor-r4-store:results/antigravity/a8-finite-cuda-qual-2026-10-25/a8-finite-cuda-qual-20261025.tar.gz)
~/.cargo/bin/cargo run --locked -p uor-r4-discrete --bin a8-qualify-stage -- verify \
  --out-dir <EXTRACTED_QUAL_OUT_DIR> \
  --summary-out docs/labs/m1-a8-finite-cuda-qualification/qualification-summary.json
```
