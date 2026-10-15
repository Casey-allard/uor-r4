# #518 — feat(#510): R5 tranche 1a — graph-format parse layer to the sanctioned surface

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-09T00:34:18Z
- **Updated:** 2026-10-09T13:08:52Z
- **Closed:** 2026-08-09T00:50:45Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/518
- **Merged:** 2026-08-09T00:50:45Z
- **Merge commit:** dd63742d9fcdfd86cd81b72be761800b8df215eb
- **Base / Head:** main / UOR-Foundation:feat/510-r5-t1a-graph-format
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

First tranche of the R5 full-rearchitecture (per the march plan on #510). Introduces the sanctioned failure vocabulary and moves graph-format's parse/CID layer onto it, **diagnostic-preserving**. graph-format R5 violations **31 → 12**; audit-limits total **352 → 333**.

**New `sanctioned.rs`:** `NotAProduct { object, reason }` (which object is absent + the preserved `FormatError` reason), `KappaError` (a CID did not reproduce), `ObservedBound` (a measured bound). The returned *contract* is regularized to the sanctioned set; `FormatError` stays as internal validation detail carried inside `NotAProduct`, so **no diagnostic is lost** and validation is unchanged.

- 17 parse/validate/build fns → `NotAProduct` (via `From<FormatError>`); the 2 CID checks → `KappaError`; r4g1 `RealizationError` gains `CidMismatch(KappaError)`.
- External callers (graph-runtime, api) bridged via the preserved `.reason` / `KappaError::as_format`; those crates convert in later tranches, removing the bridges.
- Tests updated to the sanctioned surface (the content-tamper realization test now expects `CidMismatch`).

std + no_std build, clippy clean on both, all graph-format + dependent runtime/api tests, the register meta-gate, and the 101-scenario BDD suite pass.

Remaining graph-format (tranche 1b): the other 5 error enums (12 sites). Refs #510.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

