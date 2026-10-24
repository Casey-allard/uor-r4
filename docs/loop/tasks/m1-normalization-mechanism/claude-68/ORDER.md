# ORDER: m1-normalization-mechanism, claude-68 (trace scorer for the structural-norm bar term)

Lab: claude. Item: [m1-normalization-mechanism](../../../../roadmap/items/m1-normalization-mechanism.md) (#2842). Base: `origin/main`. Casey in Claude's chat, 2026-10-24: "build the trace-capture scorer for the structural-norm bar term (per-stage L1/L2/Linf and endpoint address distance on N_snap), as the next Claude unit; stop and ask if it needs a GPU-kernel change."

## Goal

Bar item 1 reads: "100% of stage outputs in `N_snap` satisfy ‖x̂‖₁ = 4, ‖x̂‖₂² = 8, ‖x̂‖∞ = 2, and endpoint `address_distance` ∈ {0, 4, 8, 12, 16}". #3119 reports it as `UNSCORED`.

No kernel change is needed. The Flock/Relation forward already traces, per (row, block):
- `rec[1]`, the final output root (`ABSENT` for `C_raw`);
- per level, `lv[2]`, the consensus root (`discrete.cu` 121–128, 225–268);
- `Device::forward_traced` and `flock_trace` return the trace on the host (`device.rs` 351, 1368);
- `normalization_mechanism::endpoint_address_distance` mirrors the kernel distance.

This unit adds:
1. **A pure, un-gated summary over trace words and outputs.** It checks every stage output, meaning each final output root and each level consensus root where `kept > 0`. Each must be a root index in [0, 16) whose vector has L1 = 4, L2² = 8 and L∞ = 2, and each row's endpoint distance to all 256 cells must lie in {0, 4, 8, 12, 16}. It reports counts, failures, the distance histogram and `met = (failures == 0)`.
2. **`normalization-dev-structure CONFIG CHECKPOINT OUT`** (CUDA; amended: no eval-report argument, since rows come from the DEV corpus as in `capture_dev`). It takes the A8-trunk config through `validate_normalization_config`, restores the checkpoint, runs `forward_traced` over the DEV rows, and emits the summary with mode, scope and pins.

The bar applies to `N_snap`. The other arms are reported, not judged.

## Cancel probe (run first)

At the base, does any command report per-stage root validity, norms, or endpoint distance over DEV? If one does, the task is void. Class: instrument.

## Files in scope

- `crates/uor-r4-discrete/**`
- This task folder

## Done when

1. **The summary catches bad outputs.** On hand-built traces it counts valid stage outputs, and it flags an `ABSENT` or out-of-range root and an off-set distance.
2. **The command is wired.** It refuses non-A8-trunk configs and is CUDA-gated with a non-cuda stub.
3. **Pinned by tests.** CPU tests pin 1, and one fails with its fix set aside.
4. **Checks pass:**
   - every discrete target and the `prime_zeta.rs` re-pin;
   - clippy on the changed lines;
   - `fmt`;
   - D23/D25/D32 at baseline;
   - `loop report --check` and `loop pr-check --local`.

## Stop and ask if

- Any GPU-kernel (`.cu`) or PTX change is needed.
- Anything outside `crates/uor-r4-discrete` must change.
- The trace cannot be tied to DEV rows without a new device path.
