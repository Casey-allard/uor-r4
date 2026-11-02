# REPORT: finite quantization-aware shadow and export-gap control, deepseek-2

Lab: deepseek. Item: [cpu-quantization-exact-memory](../../../../roadmap/items/cpu-quantization-exact-memory.md) (#3503). Base: `454a90031`.

## What did not work, first

- **There is no trained FBS float snapshot available locally or in the public fixture.** The only public trained model in `local/deepseek/public-m1-parity` is the discrete 19-table D11 artifact; the repository's trained-FBS parity test is gated behind an external `UOR_FBS_SNAPSHOT`. This unit therefore proves the quantization/export mechanism on a paired synthetic snapshot and records trained-artifact, complete-reply and fresh-phrasing outcomes as `NOT_RUN`.
- **The quantized FBS reference is not an exact query oracle for the CPU/serving query.** On the first accepted fixture, the exact CPU and `IntegerStackSession` queries, tokens and kept cells matched, but the quantized reference differed on 20 of 64 Q12 query coordinates while retaining the same token and kept-cell decisions. The final evaluator keeps that reference gap as a measured field; the admission gate requires exact CPU/server parity rather than asserting the reference is identical.
- **A satisfied-margin no-op is not reachable on the convenient target-zero scan.** The first no-change fixture therefore failed to find an updateable/no-update row. The control now selects a target outside the admitted cells, the trainer's documented no-update path.
- **One-layer Hamming synthesis is invalid under the serving fixture contract.** The Hamming-out-of-scope control now uses a two-layer snapshot before attaching synthetic reads. This is a fixture correction, not a mechanism result.
- **No performance number is claimed.** The finite panel performs both reference forwards, exact CPU reads and serving-session reads to establish equality; its throughput is not a training rate.
- **The first adversarial review found five controls below the unit's contract.** The trainer checkpoint was not independently hash-bound, resume stopped at byte equality instead of proving the next checked step, `EXPORT_LIMITED` could be masked by a different exact metric, rollback fabricated the rejected reading, and baseline exact export parity was not fail-closed. All five were repaired before the replacement review.

## What changed

- Added `CpuQuantizationShadow`, which pairs one no-Hamming, no-memory native-writer `FbsTrunkSnapshot` with the exact Q12 `UORLUT01` artifact used by `CpuRankMarginTrainer`.
- After every trainer step, the accepted integer token-table delta is applied to the continuous shadow at exactly `delta / 4096`; the updated shadow must re-export byte-identically to the trainer artifact.
- Added a same-panel evaluator reporting:
  - exact CPU-versus-serving query, token and kept-cell mismatches;
  - plain-float shadow target admission, top-1 and top-8;
  - quantized-reference target admission, top-1 and top-8;
  - separate float/reference gaps for query coordinates, tokens, kept cells, admission, top-1 and top-8.
- Added a versioned checkpoint containing source snapshot and initial artifact SHA-256 values, current artifact and shadow hashes, trainer checkpoint bytes, training/panel samples, complete shadow state and panel reading. Restore validates all identities, re-exports the artifact, reparses the trainer, recomputes the panel and refuses malformed, truncated, stale or tampered input.
- The nested trainer state has its own SHA-256 and the canonical checkpoint envelope must match a caller-retained SHA-256. Restore also proves that uninterrupted and restored shadows choose and apply the same next step, then produce identical checkpoint identities.
- Added an admission gate with `Accept`, `NoChange`, `RejectAdmission`, `RejectExportGap` and `ExportLimited`. A rejected trial is dropped without assigning any trainer, shadow or cursor field.
- Added controls for exact Q12 re-export across repeated accepted/refused steps, whole-panel admission rollback, no-change state identity, float-only gain classification, tampered/trailing checkpoints and unsupported Hamming shadow state.

## Measurements

- The focused quantization-shadow suite passes 7/7.
- Accepted synthetic update: trainer table, shadow export and checkpoint artifact remained byte-identical after re-export; repeated checked steps preserved that invariant.
- Exact CPU/serving export: zero query-coordinate, token and kept-cell mismatches on the fixed panel.
- Quantized reference: zero token and kept-cell mismatches; its query-coordinate difference from serving is recorded rather than hidden.
- No-change control: model artifact and shadow bytes remained unchanged while the trainer clock advanced.
- Constructed float-only gain: classified `EXPORT_LIMITED`.
- A synthetic metric-mix control confirms a float top-1 gain cannot be masked by an exact top-8 gain.
- A real high-step trainer candidate that loses whole-panel admission through the normal checked-step path: classified `RejectAdmission`; the complete checkpoint bytes remained identical to the pre-trial state.
- Hamming/turn-memory saved-session restart is outside this PR and already landed separately in #3537 / `52ddbb0b0`.
- No corpus, trained-artifact, complete-reply, fresh-phrasing or language result occurred.

## Checks run at the head

- `cargo test --release --locked -p uor-r4-cpu-train` on merged `origin/main` through `13bbbe4ce`: PASS after the known duplicate-artifact retry, 105 passed and 4 ignored release benchmarks.
- Focused `quant_shadow` tests: PASS, 7/7.
- `cargo clippy --locked -p uor-r4-cpu-train --all-targets --no-deps -- -D warnings`: PASS; dependency crates emitted pre-existing dead-code warnings.
- `cargo fmt --package uor-r4-cpu-train -- --check`: PASS.
- `uor-harness ratchet d23`: PASS, 4,451 sites, baseline unchanged.
- `uor-harness ratchet d25`: PASS, 93 sites and zero tensor-core sites, baseline unchanged.
- `uor-harness ratchet d32`: PASS, 31 sites, baseline unchanged.
- `uor-harness loop pr-check --local`: PASS, including ORDER-first, scope, REPORT, owner-path, rule-source, rendered-lock and read-path checks.
- `uor-harness loop gates --full`: PASS, all 11 gates green.

## Skills used

No repository skill was invoked. The unit follows the owner-authorized D84 CPU program and the prior D37/FBS export references.

## Still open

- A trained FBS snapshot and matched query panel.
- Complete-reply and fresh-phrasing outcomes.
- Any claim that CPU training is useful at corpus scale.

## Plain summary

The CPU trainer now carries a lawful offline float shadow alongside its exact integer artifact, forces every accepted table update through both, measures where float and integer decisions diverge, and refuses a checkpoint or update that loses exact serving parity or panel admission. This is finite component evidence, not a trained-model or language result.
