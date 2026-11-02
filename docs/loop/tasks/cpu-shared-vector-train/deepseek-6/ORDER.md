# ORDER: write and verify the trained corpus artifact, deepseek-6

Lab: deepseek. Item: [cpu-shared-vector-train](../../../../roadmap/items/cpu-shared-vector-train.md) (#3505). Base: `45540f256`.

## Goal

Make every bounded CPU corpus run write its final trained `UORLUT01` artifact beside its checkpoint, then verify that the emitted bytes reload as the same serving model and match the trainer's artifact identity. This is the handoff required before the artifact can be passed to `uor-chat --stack` or a trained-artifact evaluator.

## Scope

- `crates/uor-r4-cpu-train/**`
- `docs/loop/tasks/cpu-shared-vector-train/deepseek-6/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-shared-vector-train.md`

No serving edit, nonzero training run, corpus-scale pilot, GPU run or language claim.

## Done when

- `cpu-corpus-driver` writes `<checkpoint-dir>/final.uorlut` and reports its path and SHA-256.
- The emitted artifact is reloaded with `IntegerStackModel::parse` before success.
- The emitted artifact, trainer artifact and final checkpoint payload agree exactly; a zero-step run reproduces the input artifact byte-for-byte.
- A failed, truncated or mismatched artifact write cannot produce a successful report.
- The artifact reload is exercised with a genuine public dual-c131 artifact without running an update.
- Full release tests, clippy, formatting, D23/D25/D32 ratchets, local PR check and all loop gates pass.
- A fresh exact-head adversarial review approves the change before merge-queue delivery.
