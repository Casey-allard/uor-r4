# ORDER: one-model-end-to-end, claude-395 (the first native training of the read's gates, on real states)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407).
Base: claude-393's head `b6138b4f5` (PR #4014, queued). It holds the sheet and the window-split credit kernel.

## Why this unit

D86 step 2 has its operator on host and device (claude-389 to claude-393), and the sheet [`one-model-end-to-end-native-read-train-2026-11-15`](../../../../harness/prep/one-model-end-to-end-native-read-train-2026-11-15.md) passed a non-author FIT. No gate of the native read has been trained on real states.

On 15 Nov 2026 Casey asked Claude to "do the steps you have recommended". The second and third steps are this unit:
- the native read becomes ADR-0007's I1 reference (claude-394, PR #4016);
- the Q31 objective moves off a synthetic fixture onto real states.

## What is built, then run

1. **TRAIN dumps with targets** (the sheet's item 1). `recall-payload-probe` gains `--native-train N`: the native replay and its dump read the first `N` TRAIN windows at the run's own seed. Beside every dump, held-out ones included, it writes:
   - `.ask.u32`, the asked fact's position;
   - `.other.u32`, the other fact's position;
   - `.random.u32`, a seeded earlier row (C2);
   - `.bucket.u8`, the query's far bucket.
2. **The native trainer** (the sheet's item 2). It is the `uor-r4-discrete` example `native_read_train`: the device step, readings as JSON lines, the saved read and its picks, and a host check on the first steps.
3. **The injected-pick decode** (the sheet's item 3). The probe gains `--native-picks FILE`: every pass with the read takes the native picks as the hard copy's first pick, so the ×16 pass decodes from them.
4. **The pre-run review** (§5.10). It covers the three things the FIT left unread: the device step code, `native_replay_pass` with `recall_payload_probe`, and the house-standard sheet.
5. **The run on wick**, as the sheet registers it:
   - outcome 0's instrument checks first;
   - then A-ext and A-4k at shifts {4, 8, 12};
   - then C0, C1, C2, C2′, C3 and the replay.

## Native-space five items (SKILL)

- **State:** the read's Q31 gates. Weights and gain are fixed (g = 4), as the sheet says.
- **Operator:** one definition, `Q31FirstPickRead::train_pick_step`, with its device twin.
- **Replay oracle:** `Q31ReadTrainStep` equality, host against device, on the first two steps. The replay arm's final read is byte-identical.
- **Forward, gradient, update:** integer only on the training path. The float trunk is the fixed source of the states, and the float decode in item 3 is diagnostic code (D85).
- **Artifact:** the saved read via `to_bytes`, sha256 recorded.

## Files in scope

- `crates/uor-r4-float-train/src/in_space_long.rs`, `crates/uor-r4-float-train/src/bin/recall-payload-probe.rs`, `crates/uor-r4-float-train/src/vector_reads.rs`
- `crates/uor-r4-discrete/examples/native_read_train*`
- `crates/uor-r4-native-vector/src/**`, only if the trainer needs an identity-gates constructor
- `docs/loop/tasks/one-model-end-to-end/claude-395/**`, `docs/research/notebook/float-training.md`

## Done when

1. The three tools pass their laptop tests and build with CUDA on wick, where the CUDA tests also run.
2. Outcome 0's checks pass, or a fault is reported and repaired.
3. Every registered arm ran its 512 steps, and the readings are in the REPORT, mapped to one outcome per (state, bucket) in the sheet's order.
4. The results, and the sha256 of the saved reads and targets, are in the HF store.
5. `loop pr-check --local` and a fresh-context review at the exact head pass.

## Stop and ask if

- a base state's sha256 differs from claude-389's ORDER;
- wick's GPU is in use by the D41 pilot's `llama-server` (it is never touched; wait or move the job's timing).
