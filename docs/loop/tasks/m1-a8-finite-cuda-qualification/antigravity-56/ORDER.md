# ORDER: m1-a8-finite-cuda-qualification, antigravity-56

Lab: antigravity. Item: [m1-a8-finite-cuda-qualification](../../../../roadmap/items/m1-a8-finite-cuda-qualification.md). Base: `9ae329d4e`.

## Goal

The item's done-when reads:

> All finite qualification controls PASS on CUDA with restoration and timing recorded (currently NOT_RUN), under the sheet-bound source fd409fcdf.

## Cancel probe (run first)

- Command:
  - `~/.cargo/bin/cargo test -p uor-r4-discrete --lib attention_audit -- --nocapture`
  - `~/.cargo/bin/cargo test -p uor-r4-discrete --lib attention_anchor -- --nocapture`
  - `shasum -a 256 docs/harness/prep/m1-attention-admission-preserving-DRAFT-2026-10-19.md`
  - `scripts/pod/uor-store ls results/claude/a8-stage1-pod1-20261024-r2`
  - `scripts/pod/uor-pod status`
- Cancel if: the host finite qualification or anchor tests fail, the registered sheet hash `fcb770e66d38f607d25e16dade14ac7a6223dd8af0ccc9ec22c80db71f2c3450` mismatches `FIT_SHEET_SHA256`, the Stage 1 pod archive `results/claude/a8-stage1-pod1-20261024-r2/a8s1-pod1.tar` is missing from the HF store, or the GPU caps (`<= 4` pods, `<= $8/h`) prevent leasing an RTX 5090 pod.
- Probe result (2026-10-25, classed `instrument`): PASS. 39/39 `attention_audit` unit tests and 14/14 `attention_anchor` unit tests pass on host; `FIT_SHEET_SHA256` matches `fcb770e66d38f607d25e16dade14ac7a6223dd8af0ccc9ec22c80db71f2c3450`; `results/claude/a8-stage1-pod1-20261024-r2/a8s1-pod1.tar` (529.7 MB) is present in the HF store; `uor-pod status` reports 0/4 pods running and `$0/h` of `$8.00/h` with verified RTX 5090 offers available.

## Files in scope

- `crates/uor-r4-discrete/**`
- `docs/harness/prep/m1-a8-finite-cuda-qualification*`
- `docs/labs/m1-a8-finite-cuda-qualification/**`
- `docs/loop/tasks/m1-a8-finite-cuda-qualification/**`

## Done when

- All finite qualification controls (`synthetic_control`, `host_metadata_bookkeeping`, `canonical_initlock_c2`, `required_negatives`, `g_occ`, `sensitivity`, and `audited_restore_epoch_cursor` on both `relation` and `frozen-zero` arms) PASS on CUDA with restoration and timing recorded under the sheet-bound source `fd409fcdf3bc43d607e768bae6f8cf403aa75901` (`executable_sha256 = fb4493253d3e26dc7cc5fbcbcf7fa4dc882d93a5c7b02d340b25918a2d25b00c`), the sealed archive is uploaded to the HF store and verified, the committed summary artifacts and notebook row are recorded, and `docs/roadmap/items/m1-a8-finite-cuda-qualification.md` is set to `status: done`.

## Stop and ask if

- An owner-only boundary (`CHARTER.md`, §3 technical constraints, GPU caps, or `docs/harness/owner-paths.json`) would need to change.
- A pin mismatches or an exact-reproduction / finite-qualification check fails on CUDA, in which case classify the failure (`instrument` vs `mechanism`), record it in `REPORT.md`, and either repair the instrument in scope or close the configuration without waiting.
