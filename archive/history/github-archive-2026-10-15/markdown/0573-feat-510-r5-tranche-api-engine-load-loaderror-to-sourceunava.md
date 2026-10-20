# #573 — feat(#510): R5 tranche — api engine load LoadError to SourceUnavailable (api 2->1)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-10T04:26:52Z
- **Updated:** 2026-10-09T13:09:03Z
- **Closed:** 2026-08-10T04:45:30Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/573
- **Merged:** 2026-08-10T04:45:30Z
- **Merge commit:** cd0d271d78179dd59abcdd90c10213d77eb2bfc1
- **Base / Head:** main / UOR-Foundation:r5/api-load-source-unavailable
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## R5 tranche (#510): api engine `load` `LoadError` → `SourceUnavailable`

`R4Engine::load` ingests four heterogeneous external byte-part sources — the R4G1
graph, the teacher artifact, the score report, and the tokenizer — into a
deployable engine. Every load-time failure means one of those source parts is not
usable, which is exactly `SourceUnavailable`; its free-form reason carries each
specific diagnostic the composite `LoadError` enum used to name. Replaces
`LoadError` with `Result<Self, SourceUnavailable>`:

- graph parse / `verify_cids` / missing HEAD / tokenizer_cid mismatch →
  `SourceUnavailable`
- teacher artifact / score-report JSON / tokenizer bytes invalid →
  `SourceUnavailable`
- teacher token table exceeding u32 → `SourceUnavailable`
- quality-gate digression (`validate_quality_report` `Option`, #571) →
  `SourceUnavailable(message)`

Failures on the already-parsed-and-CID-verified graph — the scorer rebuild, the
deployed step state, and R4G1 addressability — are **self-produced defects**, so
they now `.expect(...)` (panic) rather than fabricating a load error.

The `LoadError` enum and its `Display`/`Error` impls are removed; the crate now
re-exports `SourceUnavailable` (which `compile` already returns), so consumers
name the sanctioned type without depending on `uor-r4-model-source` directly.

### Callers adapted
- The root `r4g1` loader attributes teacher-part reasons to the teacher path and
  the rest to the graph path via the reason string.
- The api tests match on `error.reason` (a leaf `std::error::Error`, no source
  chain); the obsolete `LoadError` Display unit test is removed.
- `serving_eval` keeps its own `ServingEvalError::Load { message }` (it
  stringifies the error) until its R5 conversion.

### Audit
api audit-limits: **2 → 1** (only `serving_eval.rs` remains). `SourceUnavailable`
is sanctioned. After the next PR (serving_eval) api hits 0.

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

