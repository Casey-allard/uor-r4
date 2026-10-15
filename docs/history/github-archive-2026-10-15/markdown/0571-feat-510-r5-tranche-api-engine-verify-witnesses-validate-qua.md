# #571 — feat(#510): R5 tranche — api engine verify_witnesses + validate_quality_report to Option (api 15->13)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-10T03:23:51Z
- **Updated:** 2026-10-09T13:09:03Z
- **Closed:** 2026-08-10T03:42:51Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/571
- **Merged:** 2026-08-10T03:42:51Z
- **Merge commit:** 64d2620c9a575282f8edc281814d53fc6e5a97f3
- **Base / Head:** main / UOR-Foundation:r5/api-witness-quality-option
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## R5 tranche (#510): api engine `verify_witnesses` + `validate_quality_report` → `Option`

Both `engine.rs` entry points are verdict/validator surfaces, so their `Result`
becomes a total `Option`:

- **`R4Engine::verify_witnesses` → `Option<WitnessVerificationError>`.** A witness
  replay either agrees (`None`) or names the first divergence (`Some(reason)`);
  every `WitnessVerificationError` variant is preserved as the `Some` payload. Its
  two internal `InferenceError`-returning calls (`check_window` /
  `score_sig_witness`) are matched inline — they stay `Result` until the
  `InferenceError` cluster converts.
- **`validate_quality_report` → `Option<String>`.** `None` means the report clears
  the declared quality basis; `Some(reason)` is the human-facing gate-failure
  message.

### Callers adapted
- `R4Engine::load` maps the quality `Option` to `LoadError::QualityGate` (load
  keeps its `LoadError` surface until its own R5 conversion).
- The root package's `r4g1` wrapper maps the witness `Option` back to its own
  `Result<(), WitnessVerificationError>` — the root crate is not audited, so this
  keeps its public signature and the server / status-policy callers unchanged.
- `tests/bdd.rs` drops the now-redundant `.err()` on `validate_quality_report`.

### Audit
api audit-limits: **15 → 13** (both sites were in `engine.rs`). No sanctioned
error types introduced; `WitnessVerificationError` is retained as an `Option`
payload, not a `Result` error. Remaining api: the `InferenceError` serving cluster
(11 fns), `LoadError`, and `serving_eval.rs`.

### Gauntlet
`cargo fmt --check`, `cargo build --workspace --all-targets`, `clippy` (api,
all-targets), the api lib + integration suites, `status_policy` (+census), and the
full root-package `bdd` suite (100 scenarios / 333 steps) — all green. (api is not
a wasm target.)

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

