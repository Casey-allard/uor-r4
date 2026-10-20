# #561 — feat(#510): R5 tranche — graph-certify fmm.rs scorer to total

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-09T23:16:10Z
- **Updated:** 2026-10-09T14:17:24Z
- **Closed:** 2026-08-09T23:30:57Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/561
- **Merged:** 2026-08-09T23:30:57Z
- **Merge commit:** 200bfddf7af963c5658f6568abac542555ec2b85
- **Base / Head:** main / UOR-Foundation:r5/certify-fmm
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

R5 rearchitecture (#510): convert `fmm.rs` (the FMM low-rank far-field scorer) off its eight `Result<_, String>` sites. All become total `Option`, except one spurious `Result` that becomes infallible. **The module is now fully sanctioned (0 audit sites).**

## Surfaces
- `FmmConfig::validate -> Option<Self>` — `None` on a zero max_rank, a tolerance outside (0,1], or a zero dimension.
- `from_graph_parts -> Option<Self>` — `None` when the graph parts do not yield a scorer (empty/misaligned regions, zero vocab, inconsistent signature widths, zero spectral energy, no candidates). The internal `validate`/`symmetric_eigen` `?` chains carry through unchanged as `Option`.
- `fixed_point -> FmmFixedCandidateScorer` (was `Result`) — every element is rounded and clamped into range, so quantization cannot fail; the `Result` was spurious, now infallible.
- `score` / fixed `score` / `select_into -> Option<...>` — `None` on a query signature width mismatch, an undersized projection scratch, or an empty candidate set.
- `to_packed_section -> Option<Vec<u8>>` — `None` when the table cannot be packed (dimension/rank/token-count exceed the packed integer widths, empty dim/rank/candidates, inconsistent factor dims, or a coefficient exceeds i32).
- `symmetric_eigen -> Option<(Vec<f64>, Vec<f64>)>` — `None` on invalid matrix dimensions; the Jacobi-pivot invariant carries through as `?`.

## Ripple
`score_runtime::fmm_candidate` keeps its own `Result<_, String>` surface (converted in a later score_runtime PR) and maps the new `Option` via `ok_or_else`. `tests/bdd.rs`: `load_fmm_candidate` unchanged (`fmm_candidate` still `Result`, `.ok()` works); `fmm.score(...).ok()?` → `fmm.score(...)?`; `candidate.fixed_point().ok()` → `candidate.fixed_point()` via `.map`. fmm's own tests: `fixed_point` `.expect` dropped; the fail-closed test asserts `.is_none()` instead of `expect_err`/`contains`.

## Verification
audit-limits graph-certify fmm.rs now 0 sites. Full workspace build + wasm32 graph-compiler + clippy + fmt clean; graph-certify suite green (fmm tests 2/2), BDD 100/100, audit-deferral clean.

Part of the #510 bottom-up sweep — graph-certify scorer core.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

