# REPORT: m1-a8-finite-cuda-qualification, antigravity-56

Lab: antigravity. Item: [m1-a8-finite-cuda-qualification](../../../../roadmap/items/m1-a8-finite-cuda-qualification.md). Closes #2830.

## What did not work

- **Readiness gate ratio `R = 78.41% > 15%` (`STOP_OVER_15_PERCENT_GATE_COST`, classed `instrument`, `mechanism_negative: false`):** While every finite qualification and restoration control passed on CUDA (`synthetic_control` PASS, `host_metadata_bookkeeping` PASS, and all `5/5` stage checks `canonical_initlock_c2`, `required_negatives`, `g_occ`, `sensitivity`, and `audited_restore_epoch_cursor` PASS on both `relation` and `frozen-zero` arms), the recorded timing projection in `timing-protocol.json` (`d97bb13a8f30b24c2c4ad99c63308adbd4f537e31d776f697eeb5036aa651f86`) measured `projected_added_gate_ns = 4,947,754,777,700 ns` (`4,947.75 s`) against `projected_ungated_two_arm_ns = 6,309,716,689,996 ns` (`6,309.72 s`), or `R = 78.41%` (`7,842 bps` ceiling vs the `1,500 bps` / `15%` readiness bar; `38.06%` / `3,807 bps` even at zero restores). Because `rows_v1` accelerated 64-step training intervals to `17.66 s` (`relation`) and `6.88 s` (`frozen-zero`), running a `4.444 s` 61-batch (`3,847`-row) TRAIN-anchor evaluation plus synchronous JSON checkpoint seal/fsync/readback every `64` batches (`516` audits across both arms = `2,292.89 s`, `36.34%`) plus charging worst-case full restores across all `514` intervals (`2,546.26 s`, `40.35%`) exceeds the `15%` gate budget and triggers the registered `STOP_OVER_15_PERCENT_GATE_COST` diagnostic stop (`failure.json` `79be9c6cc128e28dd9b437cd3240cc4a7bbc047237bfc5a7f483fe4fd82d7feb`, `dose_qualification: "WITHHELD"`, `mechanism_negative: false`), requiring an audit-cadence / restore-bound replan in `m1-a8-full-dose-rerun` (`#2831`) before full-dose training.
- **Historical `300 s` `resource_admission` expiry in the 24 Oct Stage 1 tar (`results/claude/a8-stage1-pod1-20261024-r2/a8s1-pod1.tar`):** The Stage 1 pod archive from 24 Oct (`claude-87`) contained the sheet-bound `discrete-stack` binary (`executable_sha256 = fb4493253d3e26dc7cc5fbcbcf7fa4dc882d93a5c7b02d340b25918a2d25b00c`) and all `16` zero-update inherited observation JSONs, but its `resource_admission` timestamp had expired under the binary's `300 s` freshness check before `attention-audit-qualify` could run. We built the Rust staging/verification tool `a8-qualify-stage` in `crates/uor-r4-discrete/src/bin/a8-qualify-stage.rs` to stage a fresh `0600`-under-`0700` `resource_admission` observation, `plan.json` (`fe09466e1dac9b2ed46036a09c80d25df2ae043601680ef8e294ea1afeb842d0`), and `admission.json` (`b93e7d95620c762c96d95d21d2bab23d54ffa6ccfee0354689db5e98e5203cde`) under our D67 pre-run review (`#2830` comment `6393777094`) and run `attention-audit-qualify` immediately within the `300 s` window.
- **`uor-pod down` built-in scp archive target directory error:** After we had already downloaded the sealed qualification archive (`a8-finite-cuda-qual-20261025.tar.gz`, `a9c1228b…`), verified its `SHA256SUMS` locally, and uploaded it to `caseyallard/uor-r4-store:results/antigravity/a8-finite-cuda-qual-2026-10-25/`, `uor-pod down vast-57614891` hit the known `uor-r4/pods: Is a directory` scp target bug during its automatic archive copy; running `uor-pod down vast-57614891 --lab antigravity --session m1-a8-finite-cuda-qualification --confirm-archived` cleanly terminated the pod and released the lease.

## What changed

- Built `crates/uor-r4-discrete/src/bin/a8-qualify-stage.rs` (`stage` and `verify` subcommands plus `3` unit tests) to stage the D67 finite qualification plan/admission artifacts on the pod and to verify all `792` raw artifacts, `142` bookkeeping artifacts, `synthetic-control.json`, `host-metadata-bookkeeping/measurement.json`, `relation/qualification.json`, `frozen-zero/qualification.json`, `production-proof.json`, `timing-protocol.json`, and `failure.json` on the host.
- Executed the finite CUDA qualification and restoration run on `vast-57614891` (`1 x RTX 5090`, `$0.908/h`, ~`16.2 min`, `~$0.25`) under the sheet-bound `fd409fcdf3bc43d607e768bae6f8cf403aa75901` binary (`fb4493253d3e26dc7cc5fbcbcf7fa4dc882d93a5c7b02d340b25918a2d25b00c`), archived the complete output to `caseyallard/uor-r4-store:results/antigravity/a8-finite-cuda-qual-2026-10-25/`, and committed the verified summary [`docs/labs/m1-a8-finite-cuda-qualification/qualification-summary.json`](../../../labs/m1-a8-finite-cuda-qualification/qualification-summary.json) and lab report [`docs/labs/m1-a8-finite-cuda-qualification/README.md`](../../../labs/m1-a8-finite-cuda-qualification/README.md).
- Added the `2026-10-25` result row to [`docs/research/notebook/m1-discrete-attention.md`](../../../research/notebook/m1-discrete-attention.md) and marked [`docs/roadmap/items/m1-a8-finite-cuda-qualification.md`](../../../roadmap/items/m1-a8-finite-cuda-qualification.md) `status: done`.
- Diff size: `8` files changed (`+1,950 / -3` lines: `tests 0 file(s) +0/-0, code 2 file(s) +1352/-0, docs 5 file(s) +220/-3, generated 0 file(s) +0/-0, other 1 file(s) +378/-0` across `crates/uor-r4-discrete/Cargo.toml`, `crates/uor-r4-discrete/src/bin/a8-qualify-stage.rs`, `docs/labs/m1-a8-finite-cuda-qualification/{README.md,qualification-summary.json}`, `docs/research/notebook/m1-discrete-attention.md`, `docs/roadmap/items/m1-a8-finite-cuda-qualification.md`, and `ORDER.md` + `REPORT.md`).

## Measurements

- **Pod & Environment:** `vast-57614891` (`1 x NVIDIA GeForce RTX 5090`, driver `580.95.05`, CUDA `12.8`, `$0.908/h`, leased `2026-10-25T05:13:02Z`–`2026-10-25T05:29:15Z`, cost proxy `~$0.25`).
- **Command run on pod:**
  ```sh
  /workspace/uor-r4/target/release/discrete-stack attention-audit-qualify \
    /workspace/uor-r4/docs/labs/a8-training-configs-2026-10-19/evidence/a8-relation-seed1.json \
    /workspace/a8s1/qual-private/admission.json \
    /workspace/a8s1/qual-out
  ```
- **Finite Qualification & Restoration Controls (`PASS`):**
  - `synthetic-control.json` (`69eb5866e3e66d658602a463ff236a2f59647347764a97e569ada86b17f762ea`): `status = "PASS"`, `accepted_updates = 1`, `stale_apply_refusals = 1`, `stale_carry_refusals = 1`, `observed_restored_epoch = 0`, `fresh_binding = true`.
  - `host-metadata-bookkeeping/measurement.json` (`95c64e51c67504cb1c2d1382b698fd5df5d249c07439cb20068754c311b8d301`): `maximum_step_shape_bytes = 2,132 <= 16,384`, `maximum_boundary_shape_bytes = 37,381 <= 65,536`, `maximum_progress_shape_bytes = 1,202,016` (`16,922` entries), `max_step_sample_ns = 2,909,154 ns`, `pass_boundary_ns = 25,491,179 ns`, `breach_restore_ack_ns = 27,105,652 ns`.
  - `relation/qualification.json` (`73876b488bdb2123457d46fd28b0e4b1296cc398a930db5542b67f72cc9fc2d4`): `5/5` stages `PASS` (`canonical_initlock_c2` `9/9`, `required_negatives` `[2, 2]`, `g_occ` `b3_psucc = 1,351/3,847`, `b3_full = 4,244/14,287`, `top1 = 133`, `occupied_cells = 256`, `distinct_vectors = 4,096`, `tie_mass = 0`, `sensitivity` `[1, 1]`, `audited_restore_epoch_cursor` `384` steps, `58` accepted updates before restore, restored to `epoch = 0` with cursor at `384`, `target_admitted_rows = 1,211/3,847` and `objective_loss_numerator_sum = 3,656,320` identical before and after restore).
  - `frozen-zero/qualification.json` (`312b5370522afddbf46675220ba5e05a93e1eab5e0cf75a2f448aeb59032badc`): `5/5` stages `PASS` (`canonical_initlock_c2` `9/9`, `required_negatives` `[2, 2]`, `g_occ` `b3_psucc = 1,351/3,847`, `b3_full = 4,244/14,287`, `top1 = 133`, `occupied_cells = 256`, `distinct_vectors = 4,096`, `tie_mass = 0`, `sensitivity` `[1, 1]`, `audited_restore_epoch_cursor` `384` steps, `54` accepted updates before restore with `0` relation gate acceptances, restored to `epoch = 0` with cursor at `384`, `target_admitted_rows = 1,211/3,847` and `objective_loss_numerator_sum = 3,656,320` identical before and after restore).
- **Recorded CUDA Timing (`timing-protocol.json`, `d97bb13a8f30b24c2c4ad99c63308adbd4f537e31d776f697eeb5036aa651f86`):**
  - `gate_samples_ns = [4,443,590,804]` (`4.444 s`), `restore_samples_ns = [4,926,717,208, 3,984,920,202]` (`4.927 s` `relation`, `3.985 s` `frozen-zero`).
  - `arm_family_elapsed_ns`: `relation` `[91,860,297,786, 3,244,655,721, 10,865,641,990]` ns (`105.97 s` for `384` steps -> `4,545.14 s` full arm); `frozen-zero` `[36,034,291,782, 2,355,685,424, 2,895,767,894]` ns (`41.29 s` for `384` steps -> `1,764.57 s` full arm).
  - `projected_ungated_two_arm_ns = 6,309,716,689,996 ns` (`6,309.72 s`).
  - `projected_added_gate_ns = 4,947,754,777,700 ns` (`4,947.75 s`: `projected_gate_ns = 2,292.89 s` + `projected_restore_ns = 2,546.26 s` + `projected_step_bookkeeping_ns = 95.44 s` + `projected_boundary_bookkeeping_ns = 13.15 s`).
  - Readiness ratio `R = 78.41%` (`7,842 bps` ceiling vs `1,500 bps` bar; `38.06%` at zero restores).

## Checks run at the head

- `~/.cargo/bin/cargo test --locked -p uor-r4-discrete --bin a8-qualify-stage --no-fail-fast`: `3/3` tests PASS.
- `~/.cargo/bin/cargo test --locked -p uor-r4-discrete --lib attention_audit -- --nocapture`: `39/39` tests PASS.
- `~/.cargo/bin/cargo test --locked -p uor-r4-discrete --lib attention_anchor -- --nocapture`: `14/14` tests PASS.
- `~/.cargo/bin/cargo run --locked -p uor-r4-discrete --bin a8-qualify-stage -- verify --out-dir /Users/casey.allard/uor-r4/local/antigravity/a8-finite-cuda-qual-20261025/a8-finite-cuda-qual-20261025/qual-out --summary-out docs/labs/m1-a8-finite-cuda-qualification/qualification-summary.json`: `VERIFIED_FINITE_CUDA_QUALIFICATION` (`142` bookkeeping pins and `792` raw pins verified).
- `~/.cargo/bin/cargo clippy --no-deps --locked -p uor-r4-discrete --bin a8-qualify-stage --tests`: PASS (`0` warnings/errors under `#![deny(warnings, clippy::all)]`).
- `uor-harness loop report --check --order docs/loop/tasks/m1-a8-finite-cuda-qualification/antigravity-56/ORDER.md`: PASS.
- `uor-harness loop pr-check --local`: PASS.
- `uor-harness loop gates --full`: PASS.

## Skills used

- `uor-research-architect`, `scope-guard`, `solution-integrity`, `gates`, `context-guard`.

## Still open

- `m1-a8-finite-cuda-qualification` (`#2830`) is complete (`status: done`).
- In `m1-a8-full-dose-rerun` (`#2831`), replan the audit cadence / restore bound so the projected gate overhead is `<= 15%` of the `rows_v1` two-arm training dose before launching the 16,404-batch audited dose.

## Plain summary

We ran the finite CUDA qualification and restoration test (`#2830`) for the admission-preserving A8 attention controller on an RTX 5090 GPU using the exact reviewed binary from commit `fd409fcdf`. Every finite qualification and restoration check passed on the GPU:
1. **Synthetic restoration control (`PASS`):** The controller applied one model update, restored the model back to epoch `0`, refused stale update and carry handles from the old epoch, and bound a fresh handle.
2. **Bookkeeping envelope (`PASS`):** All `142` step and boundary bookkeeping files stayed well inside their byte limits (`2,132 B <= 16,384 B` per step; `37,381 B <= 65,536 B` per boundary).
3. **Both model arms (`relation` and `frozen-zero`) passed all 5 qualification stages (`PASS`):**
   - **C2 initialization lock (`canonical_initlock_c2`):** `9` of `9` controls passed with `0` updates.
   - **Required negative controls (`required_negatives`):** Both invalid configurations (`K=16` and a non-root codebook table) were refused with exit code `2`.
   - **Starting admission and occupancy (`g_occ`):** Both arms admitted `1,351` of `3,847` copy-successor (`P_succ`) rows and `4,244` of `14,287` full development (`DEV`) rows at `133` top-1, with all `256` cells occupied and `4,096` distinct token vectors.
   - **Probe sensitivity (`sensitivity`):** Both combine-gate and relation-gate probes changed candidate costs (`[1, 1]`).
   - **Production post-update restoration (`audited_restore_epoch_cursor`):** Each arm ran `384` real GPU training steps (`58` accepted updates on `relation`, `54` on `frozen-zero`), then executed a controlled rollback to the initial checkpoint (`epoch = 0`) while advancing the dataset cursor to batch `384`. Both arms restored the exact initial model storage and snapshot hashes, refused stale handles, and scored identically on the `3,847`-row training anchor before and after restoration (`1,211` rows admitted, `43` top-1, `3,656,320` objective loss numerator sum).
4. **Recorded timing and readiness stop (`78.41% > 15%`):** Because the fast row-parallel (`rows_v1`) GPU kernel finishes `64` training steps in `17.66 s` (`relation`) and `6.88 s` (`frozen-zero`), the entire 16,404-step two-arm training dose takes `6,309.72 s` (`1.75 hours`). Scoring the `3,847`-row training anchor and sealing/verifying a full JSON checkpoint takes `4.444 s`, and a full checkpoint restore takes `4.927 s`. Running that `4.444 s` audit every `64` steps (`516` times = `2,292.89 s`, `36.34%`) plus charging worst-case restores at all `514` intervals (`2,546.26 s`, `40.35%`) projects `4,947.75 s` of gate overhead (`78.41%` of training time, or `38.06%` with zero restores). The qualification runner therefore triggered its built-in `> 15%` readiness stop (`STOP_OVER_15_PERCENT_GATE_COST`, `mechanism_negative: false`), unlocking the cadence/restore replan in `#2831` before full-dose training.
