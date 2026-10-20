# #541 — feat(#510): R5 tranche 5a — graph-compiler quantum_cover + reference_ir to total

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-09T11:48:09Z
- **Updated:** 2026-10-09T13:09:00Z
- **Closed:** 2026-08-09T12:06:55Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/541
- **Merged:** 2026-08-09T12:06:55Z
- **Merge commit:** 512f1d791511d21ffbc0f8b75278ca4527d22f45
- **Base / Head:** main / UOR-Foundation:r5/gc-compute
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

First of several `uor-r4-graph-compiler` tranches. The crate is the true bottom-up leaf of the remaining R5 work (graph-certify wraps it; api wraps certify's scorer; graph-cli wraps all), so it converts before certify/api/cli. **87 → 82** sites.

## What changed
- **quantum_cover.rs** — remove `QuantumCoverError`. `DensityOperator::{max_entropy, from_weights, from_counts}` → `Option<Self>` (None = empty/degenerate distribution). Internal `member_entropy_nats` match and unit tests follow.
- **reference_compiler_ir.rs** — remove `ReferenceCompilerError`. `ReferenceCompilerPipeline::compile` → `Option<ReferenceGraphIr>` (None = empty corpus); `DifferentialCompilerHarness::compare` → `Option<f32>` (Some(delta) within tolerance, None past it — a measured bound crossed).

No external non-test callers of either error type; `bdd.rs` uses `.expect(...)` which still holds on `Option`.

## Verification
- `cargo build --workspace --all-targets` clean
- `cargo build -p uor-r4-graph-compiler --target wasm32-unknown-unknown` clean (neither module touches host I/O / `SourceUnavailable`)
- quantum_cover + reference_compiler_ir unit tests pass; `clippy -D warnings` clean; `cargo fmt --check` clean
- `audit-deferral` (R4) passes; `audit-limits` graph-compiler **87 → 82**

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

