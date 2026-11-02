# ORDER: finite quantization-aware shadow and export-gap control, deepseek-2

Lab: deepseek. Item: [cpu-quantization-exact-memory](../../../../roadmap/items/cpu-quantization-exact-memory.md) (#3503). Base: `454a90031` from fresh `origin/main`.

## Goal

Add one isolated, lawful quantization-aware checkpoint control for the CPU shared-vector trainer. Keep the offline float source shadow and the exact Q12 integer artifact synchronized through accepted table-coordinate updates, measure the float-to-integer score gap on one fixed finite panel, and prove that a restored checkpoint reproduces the artifact and readings exactly.

## Scope

- `crates/uor-r4-cpu-train/**`
- `Cargo.toml`
- `Cargo.lock`
- `docs/loop/tasks/cpu-quantization-exact-memory/deepseek-2/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-quantization-exact-memory.md`

No serving edit, Hamming/turn-memory session restart (a separate #3503 unit), corpus run, GPU run or language claim.

## Cancel probe

Use one no-Hamming, no-memory, native-writer synthetic `FbsTrunkSnapshot`. If its exported artifact cannot be reproduced exactly after applying the same accepted Q12 token-table delta to the shadow, or if plain-float and exact-integer outputs cannot be scored on the same finite panel without adding a serving dependency or banned operation, stop and preserve the first mismatch. No rounded approximation may stand in for exact artifact agreement.

## Done when

- A versioned quantization-shadow checkpoint binds the snapshot and artifact hashes, trainer checkpoint, panel fingerprint, panel readings, substrate and exact-integer/export eligibility.
- The float shadow carries the continuous `embed` source while the trainer artifact carries its Q12 table. Applying an accepted integer coordinate delta to the shadow at exactly `+/-1/4096` re-exports byte-identically to the trainer artifact.
- The same fixed panel is scored by the plain-float shadow, the quantized FBS reference, the exact CPU writer and `IntegerStackSession`. Exact CPU/server query, token and kept-cell mismatches must be zero; float-to-server token, kept-cell, target-admission and Q12 query-gap counts are recorded separately as measured values.
- A whole-panel target-admission gate accepts a strict improving checkpoint and an identical no-change control, and rolls back an admission-losing update byte-identically.
- Restore validates every schema, hash, shape, cursor, panel and artifact invariant before construction, then reproduces the checkpoint bytes and next-step readings; malformed, truncated, inconsistent and tampered envelopes are refused.
- A constructed candidate whose float gain changes or disappears in the exact integer readings is labelled `EXPORT_LIMITED` rather than accepted.
- Tests, a focused REPORT, one notebook row and the roadmap progress distinguish fit, admission, exact-export and float-shadow outcomes. Trained-artifact, complete-reply and fresh-phrasing outcomes remain `NOT_RUN`.
- Full release tests, strict clippy, formatting, D23/D25/D32 ratchets, local PR check and all loop gates pass, followed by a fresh exact-head adversarial review.

## Stop list

- Do not add Candle, `uor-r4-training`, dense products, squashing, normalized exponentials or serving-source dependencies.
- Do not claim that a synthetic component proves trained-model quality, useful learning, complete replies, fresh phrasing or CPU qualification.
- Do not include Hamming/turn-memory session persistence in this unit.
- Do not change a bar, status, rule or owner boundary. Record a genuine ambiguity for the parent instead.
