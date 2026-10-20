# #1251 — Implement Hamming contextual read and refinement primitive

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-13T17:10:58Z
- **Updated:** 2026-10-09T13:17:35Z
- **Closed:** 2026-09-13T17:12:00Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1251
- **Merged:** 2026-09-13T17:12:00Z
- **Merge commit:** 7bd9c9b5078855cf94e795f6961d0fc4fb1d91cc
- **Base / Head:** main / UOR-Foundation:codex/hamming-context-refinement
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Implement bounded Hamming contextual reads that update working geometric state before querying again. The new primitive exposes all four contextual roots of a selected span start and its complete owned payload, preserving exact occurrence references and prior-hop receipts. Existing model sources and retained dispatch stay unchanged.

Seven focused release tests and one sealed report pass. Five authored cases show changed fourth-root context changes the second read; disabled/refined-context controls lose the dependency. Geometry census finds120distinct codes and856800single-root ranking comparisons with no angular/Hamming order or tie disagreements. This is finite untrained primitive evidence, not a language model or summed-two-lane equivalence claim.

Complete local build/report charge115442/240000ms. No fitting, model promotion or external compute. Next is parameter-bound query/update policy and emission integration with actual hard-path credit.

References #973.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

