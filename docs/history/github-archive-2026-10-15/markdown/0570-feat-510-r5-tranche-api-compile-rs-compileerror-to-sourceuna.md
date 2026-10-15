# #570 — feat(#510): R5 tranche — api compile.rs CompileError to SourceUnavailable (api 20->15)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-10T02:59:14Z
- **Updated:** 2026-10-09T13:09:03Z
- **Closed:** 2026-08-10T03:18:21Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/570
- **Merged:** 2026-08-10T03:18:21Z
- **Merge commit:** adb1ad7289a6fe161b3b1c9adddad5dba11bea80
- **Base / Head:** main / UOR-Foundation:r5/api-compile-source-unavailable
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## R5 tranche (#510): api `compile.rs` `CompileError` → `SourceUnavailable`

First api-crate tranche. The typed compile orchestrator is host-source machinery:
it validates a local Hugging Face source directory, prepares a work directory,
drives the three compiler stages over the filesystem, and collects their byte
outputs. Every failure is *"the local source or work directory could not produce
a model"* — exactly `SourceUnavailable`. Replaces the composite `CompileError`
enum with it:

- `compile` → `Result<CompileOutcome, SourceUnavailable>`
- `validate_source` / `corpus_complete` / `utf8` / `read_stage_file` →
  `Result<_, SourceUnavailable>`

The former variants fold into the single sanctioned reason string, keeping their
operator-facing detail: `SourceInvalid` → `"invalid source: …"`, `Io` →
`"<stage> stage I/O: …"`, `Stage` → `"<stage> stage: …"` (graph-cover already
yields `SourceUnavailable` from `graph-compiler::compile`), `NonUtf8Path` →
`"<stage> stage: path is not UTF-8: …"`. The `CompileError` enum and its
`Display`/`Error` impls are removed; the crate re-export drops it.

api is a **host-only crate** (it does not build for `wasm32` — its compiler
dependencies are unavailable there), so naming the wasm-gated `SourceUnavailable`
is sound. Adds the `uor-r4-model-source` path dependency.

### Tests
The api integration suite's three `CompileError::SourceInvalid` matches become
`Err(error)` with `error.reason.contains(...)` assertions.

### Audit
api audit-limits: **20 → 15** (compile.rs at 0). With graph-compiler and
graph-certify already at 0, the remaining workspace is api 15 + graph-cli 42.
Remaining api sites are the engine.rs serving surfaces
(`InferenceError` / `LoadError` / `WitnessVerificationError`,
`validate_quality_report`) and `serving_eval.rs`, converted next.

### Gauntlet
`cargo fmt --check`, `cargo build --workspace --all-targets`, `clippy` (api,
all-targets), and the full api lib + integration test suites — all green.
(api is not a wasm target, so no wasm build applies.)

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

