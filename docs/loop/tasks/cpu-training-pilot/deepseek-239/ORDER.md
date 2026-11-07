# ORDER: cpu-training-pilot, deepseek-239

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506), supporting [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Base: `195c217783fbecd58db70dd842129e90cff2a69c`.

## Goal

Take Claude-301's accepted b64L float checkpoints through a CPU-only snapshot and integer
export, then qualify the resulting `UORLUT01` artifacts on the CPU serving path. This closes
the local export bottleneck without changing the trainer or the accepted model recipe.

The existing `fbs-snapshot` binary opens CUDA even when it is only converting saved weights.
This host has no CUDA device. The new CPU path must read the saved tensors directly, reproduce
the deterministic SpiralCore/FBS route tables, validate the saved projection fingerprints and
Hamming read, apply the run's served contract, and write the same snapshot/artifact formats.

## Claim check

- The item is `ready`, has host done-class, and its scope includes the isolated CPU crate.
- Claude's open `one-model-end-to-end` ORDER owns the trainer path; this unit does not edit it.
- `loop claim` created `deepseek-239`, but its generated ORDER commit was refused by the current
  trailer hook before push. The failed claim rolled back. With the owner's explicit instruction
  to proceed, this ORDER and an isolated worktree are created manually.

## Cancel probe

Class: instrument.

1. No CUDA device is present (`nvidia-smi` is absent), so the existing GPU snapshot path cannot
   run on this host.
2. Both b64L seed archives are already present under
   `local/deepseek/centroid-retarget/inputs/`; their recorded SHA-256 values match.
3. The selected checkpoints contain `model.safetensors`, `config.json`,
   `hamming_read.safetensors`, and their run config.
4. No training or model-quality claim depends on this unit. The work unlocks the existing
   integer serving parity instruments on the accepted checkpoint.

Result: GO.

## Done when

1. A CPU-only exporter validates the run and model contracts, every tensor shape, the exact
   SpiralCore/FBS plan fingerprints, Hamming-read inventory and served contract before writing.
2. Unit tests reject malformed or foreign configs, missing/extra tensors, wrong projection
   bindings, non-finite values, and an unsafe existing output path.
3. Both b64L seeds export reproducibly to `uor-r4.lut-stack` artifacts without CUDA.
4. The artifacts pass `fbs-toks` execution and the independent `cpu-served-forward-audit` with
   zero query, token, kept-cell or route mismatches.
5. A REPORT records the artifact and input hashes, the exact CPU path, generated replies and
   all exclusions. No useful-chat or GPU-qualification claim is made.

## Stop and ask if

- The saved projection fingerprints cannot be reproduced from the pinned source.
- A required tensor or served read is not carried by the format.
- The artifact cannot be read by both CPU readers.
- Disk use would exceed the existing local evidence budget.

## Files in scope

- `crates/uor-r4-cpu-train/**`
- `Cargo.toml`
- `Cargo.lock`
- `docs/harness/prep/cpu-training-pilot*`
- `docs/labs/cpu-training-pilot-*`
- `docs/loop/tasks/cpu-training-pilot/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`
