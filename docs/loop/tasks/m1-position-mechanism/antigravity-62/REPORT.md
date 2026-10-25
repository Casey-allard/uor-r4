# REPORT: m1-position-mechanism, antigravity-62

Lab: antigravity. Item: [m1-position-mechanism](../../../../roadmap/items/m1-position-mechanism.md) (#2843). Order: [ORDER.md](./ORDER.md).
References #2843

## What did not work

- **Attempt 1 of `position-qualify` on `vast-57638243` (`06b8f5843`):** The initial `position-qualify.rs` binary duplicated a 16-entry `const ROOTS: [[i8; 8]; 16]` array whose root ordering differed from the build-generated `geometry::ROOTS` table in `build.rs`, causing `root_index_of` to fail at startup with `invalid discrete model: table block is not one of the 16 orbit roots`. Classified as an **instrument** defect (`D31`), not a mechanism negative; repaired in commit `77d7cb568` by removing the hardcoded array and resolving root coordinates directly through `apply_frame_to_root(candidate_to_frame(0), r)`.
- **Pre-`#3111` (`5844d911c`) `build.rs` source-anchor check in `job-3205.sh`:** Substituting `discrete-5844d911c.cu` alone into the post-`#3111` crate tree failed in `decomposition_source::generate` because `decomposition_source.rs` after `#3111` expects the 14-argument `read_vector(..., position_steps, ...)` signature. Resolved on `vast-57638243` by compiling `discrete-5844d911c.cu` with the exact `build.rs` `nvcc -ptx -std=c++17 -arch=compute_120` and `nvcc -cubin --ptxas-options=-v -arch=compute_120 --gpu-code=sm_120` commands (`5844d911c-pre3111/ordinary-resources.txt`, SHA-256 `6484c287e150cbc36b1c3ebd5221f478e5ccb4bc3f26070416764f3677005aec`).

## What changed

- **Size:** 11 files changed (`+1,852 / -8` lines total across code, tests, JSON receipts, and docs; `m1-position-mechanism.md` declares no numeric line estimate, ratio N/A, well within the loop's uncapped research bookkeeping rule `D48`):
  - **Code (`+628 / -8` lines):**
    - `crates/uor-r4-discrete/src/discrete.cu` (`+8 / -7`): Compacted per-thread position arrays `prefix[MAX_POSITIONS]` (`243` entries), `transport_root[MAX_NODES]` (`364` entries), and `transport[MAX_NODES]` (`364` entries) in `flock_vector` from 4-byte `U` (`3,884` B raw) to 1-byte `unsigned char` (`971` B raw), updating `position_action` to accept `const unsigned char* prefix`.
    - `crates/uor-r4-discrete/src/device.rs` (`+20 / -1` in `Device::from_snapshot`): Checks `position` shape and `ReadSpec::Route` refusal on the host, constructs the base `Device` with `None`, and uploads `snapshot.position`'s step words to `model.position_steps` before `model.kernels.validate` (`discrete_validate`) runs on the GPU, so a deserialized `Snapshot` with a malformed step candidate (`>= 16`) reaches `discrete_validate` and is refused on CUDA with `status 10`.
    - `crates/uor-r4-discrete/src/bin/position-qualify.rs` (`+596 / -0`) and `crates/uor-r4-discrete/Cargo.toml` (`+4 / -0`): Standalone CUDA qualification and stack/throughput profiler binary `position-qualify` (keeps `DRIVER_SOURCE_SHA256` unchanged at `31789abea8b955cb0e137257fc9e8b3e5eb42865fec7728396fa377c9d4228ea`).
  - **Tests (`+500 / -0` lines):**
    - `crates/uor-r4-discrete/src/device.rs` (`+359 / -0`, `mod position_device_tests`): 3 GPU tests (`position_modes_match_host_holonomy_and_carried_roots_in_flock_and_relation`, `prefix_holonomy_detects_interior_swap_on_device_while_endpoint_control_is_invariant`, `from_snapshot_roundtrips_position_modes_and_cuda_validate_refuses_step_16`).
    - `crates/uor-r4-discrete/tests/position_device_controls.rs` (`+141 / -0`): 3 host/source controls verifying byte-packed `unsigned char` arrays in `discrete.cu`, `from_snapshot` delegation of step-candidate bounds to `discrete_validate`, and non-trivial interior-swap separation between `PrefixHolonomyV1` and `EndpointControlV1`.
  - **Docs & Receipts (`+724 / -0` lines):**
    - `docs/labs/m1-position-mechanism/README.md` (`+51 / -0`), `position-qualify-build1.json` (`+185`), `position-qualify-build2.json` (`+185`), `position-qualify-dc5f409b0.json` (`+185`).
    - `docs/research/notebook/m1-discrete-attention.md` (`+1 / -0`): Added 2026-10-25 row.
    - `docs/loop/tasks/m1-position-mechanism/antigravity-62/{ORDER,REPORT}.md`.

## Measurements

All GPU measurements ran on `vast-57638243` (`1 x NVIDIA GeForce RTX 5090`, driver `580.178.04`, `nvcc` 12.8 V12.8.93, `UOR_NVCC_ARCH=compute_120`, commit `77d7cb5681fe8c0f94d370316dc009505f5f1460`, D67 pre-run review logged on `#2843`), archived to `caseyallard/uor-r4-store:results/antigravity/m1-position-mechanism-2026-10-25` (`SHA256SUMS`: `dc61c5edf29444996cd6a659ce476e00459d834bcea87ae8bb1f6ab37d8876c5`):

1. **RTX 5090 Throughput (`tokens/s`, lead metric per CHARTER §1) at full M1 shape (`V=4096, B=4, C=192, K=8`, 64 rows $\times$ 10 iterations = 122,880 tokens/arm):**
   - `Flock (K=8)`:
     - `legacy_bfs (s[14]=0)`: **327,988 / 327,775 tok/s** forward (B1/B2, vs `325,322` on `dc5f409b0`), **286,998 / 287,023 tok/s** evaluate (vs `285,756`).
     - `PrefixHolonomyV1 (s[14]=1)`: **305,795 / 305,829 tok/s** forward (vs `302,954` on `dc5f409b0`, `+0.94%`), **269,974 / 269,999 tok/s** evaluate (vs `268,234`, `+0.65%`).
     - `EndpointControlV1 (s[14]=2)`: **312,159 / 312,080 tok/s** forward (vs `309,736`), **274,899 / 274,925 tok/s** evaluate (vs `273,494`).
     - `IdentityControlV1 (s[14]=3)`: **315,313 / 315,203 tok/s** forward (vs `312,850`), **277,328 / 277,317 tok/s** evaluate (vs `275,868`).
   - `Relation (K=8, S=1)`:
     - `legacy_bfs (s[14]=0)`: **321,588 / 321,438 tok/s** forward (vs `318,331`), **282,417 / 282,411 tok/s** evaluate (vs `281,054`).
     - `PrefixHolonomyV1 (s[14]=1)`: **295,120 / 295,079 tok/s** forward (vs `291,705`, `+1.17%`), **261,672 / 261,683 tok/s** evaluate (vs `259,538`, `+0.82%`).
     - `EndpointControlV1 (s[14]=2)`: **301,073 / 300,930 tok/s** forward (vs `297,933`), **266,329 / 266,321 tok/s** evaluate (vs `264,455`).
     - `IdentityControlV1 (s[14]=3)`: **304,054 / 303,947 tok/s** forward (vs `300,908`), **268,637 / 268,641 tok/s** evaluate (vs `266,800`).
   - **Determinism:** Build 1, Build 2, and `dc5f409b0` produce byte-identical `vectors_sha256` across all 8 arms (`9dd7cdfe4aa180a41e9358479a0b019df134eff1aeee0087545bcda4da3ea27f` on `legacy_bfs`, `endpoint_control_v1`, and `identity_control_v1`; `a267ccf13fc55e95f8ba9ffd7bf59575bbcd81a55af32710e6411ca26267ebfc` on `prefix_holonomy_v1`), and Build 1 and Build 2 emit identical `discrete.ptx` (`be2b90a8ec7704be94db115c4627fef71467659c9623ff00503ea4fa56637699`) and `ordinary.cubin` (`a150695dffeb4b5602fec112af539d9c17f60d6b09c2a6af6a5f7725c8bc8512`).
2. **Per-thread `ptxas` local-stack bytes (`sm_120`, `0` spill stores / `0` spill loads across all builds):**
   - `discrete_forward`: `21,056` B (`5844d911c` pre-`#3111`) $\to$ `24,976` B (`dc5f409b0` `#3111` 32-bit `U`) $\to$ **`22,080` B** (`77d7cb568` byte-packed `unsigned char`, **`-2,896` B / `-11.60%`** vs `#3111`).
   - `discrete_counterfactual`: `21,536` B (`5844d911c`) $\to$ `25,456` B (`dc5f409b0`) $\to$ **`22,560` B** (`77d7cb568`, **`-2,896` B / `-11.38%`** vs `#3111`).
   - `discrete_evaluate`: `21,184` B (`5844d911c`) $\to$ `25,104` B (`dc5f409b0`) $\to$ **`22,208` B** (`77d7cb568`, **`-2,896` B / `-11.54%`** vs `#3111`).
3. **Finite GPU controls (`position-qualify` & `position_device_tests`):**
   - `checked_leaf_traces = 72 / 72` (exact match between GPU traced actions/carried roots and host `relative_holonomy` + `apply_frame_to_root` across `Flock` and `Relation` for modes `1..=3`).
   - `identity_control_matches_identity_table = true`.
   - `endpoint_control_interior_swap_invariant = true` and `prefix_holonomy_interior_swap_sensitive = true`.
   - `snapshot_roundtrip_modes_verified = 3`, `route_position_refused = true`, and `step_16_cuda_refusal_message = "discrete CUDA snapshot validation refused with status 10"`.

## Checks run at the head

- On RTX 5090 pod `vast-57638243` (`UOR_NVCC_ARCH=compute_120`):
  - `cargo test --release --locked -p uor-r4-discrete --features cuda --lib position_device_tests -- --include-ignored`: `3 passed; 0 failed` (`0.33s`)
  - `cargo test --release --locked -p uor-r4-discrete --features cuda --lib unit_snap_device_tests -- --include-ignored`: `3 passed; 0 failed` (`0.22s`)
  - `cargo test --release --locked -p uor-r4-discrete --features cuda --lib position_mechanism::tests`: `10 passed; 0 failed`
  - `cargo test --release --locked -p uor-r4-discrete --features cuda --test position_device_controls`: `3 passed; 0 failed`
- On laptop host:
  - `~/.cargo/bin/cargo test --no-fail-fast -p uor-r4-discrete --lib`: `300 passed; 0 failed; 1 ignored`
  - `~/.cargo/bin/cargo test --no-fail-fast -p uor-r4-discrete --test position_device_controls`: `3 passed; 0 failed`
  - `NVCC=local/antigravity/fake-nvcc/nvcc UOR_NVCC_ARCH=compute_120 ~/.cargo/bin/cargo check --offline -p uor-r4-discrete --features cuda --all-targets`: exit `0`
  - `uor-harness ratchet d23`, `d25`, `d32`: `0` regressions (`4451`, `93`, `31`)
  - `uor-harness loop gates --full`, `uor-harness loop report --check`, `uor-harness loop pr-check --local`: green

## Skills used

- `uor-research-architect`: Enforced integer-only (`D32`), squashing-free (`D23`), matmul-free (`D25`) CUDA kernel stack compaction, two-build determinism, and `DRIVER_SOURCE_SHA256` stability.
- `solution-integrity`: Verified that `Device::from_snapshot` actually reaches `discrete_validate` (`status 10`) on the GPU for out-of-range `position_steps` entries instead of short-circuiting on the host.
- `scope-guard`: Kept all changes strictly within `m1-position-mechanism`'s declared scope.

## Still open

- The registered two-seed GPU order test (`H_pos` vs `C_match` and `C_shuf` on the frozen 1,620-row order-sensitive DEV slice, `#2843`) remains open for a subsequent unit (`m1-position-mechanism` stays `status: ready`; `References #2843`).

## Plain summary

This unit completed the two post-merge GPU verification and optimization steps for the discrete position mechanism (`#2843` comment `6391161531`). First, we added GPU tests (`position_device_tests` in `device.rs` and the `position-qualify` binary) confirming on an RTX 5090 that all three position transport modes (`PrefixHolonomyV1`, `EndpointControlV1`, and `IdentityControlV1`, which rotate token representations using exact 64-element `Cl(0,6)` signed-permutation lookup tables) match the host reference bit-for-bit across all 72 leaf traces in both `Flock` (nearest-neighbor tree read) and `Relation` (relational sweep read), that swapping interior tokens changes the GPU output under `PrefixHolonomyV1` while leaving `EndpointControlV1` invariant, and that loading a snapshot with an out-of-range step entry (`16`) is rejected by the GPU validation kernel (`discrete_validate`) with status code `10`. Second, we compacted the per-thread CUDA stack arrays (`prefix[243]`, `transport_root[364]`, and `transport[364]`) in `discrete.cu` from 32-bit unsigned integers (`3,884` bytes) to 8-bit unsigned bytes (`971` bytes), cutting the compiled `ptxas` per-thread stack frame by `2,896` bytes (`-11.6%`, down to `22,080` bytes in `discrete_forward` with zero register spills) and running at `305,795` tokens/s (`Flock`) and `295,120` tokens/s (`Relation`) for `PrefixHolonomyV1` at full M1 model shape (`V=4096, B=4, C=192, K=8`).
