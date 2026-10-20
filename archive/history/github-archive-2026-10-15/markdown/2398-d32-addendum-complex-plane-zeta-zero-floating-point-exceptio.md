# #2398 — D32 addendum: complex-plane zeta-zero floating-point exception (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-14T00:11:02Z
- **Updated:** 2026-10-14T00:13:16Z
- **Closed:** 2026-10-14T00:12:03Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2398
- **Merged:** 2026-10-14T00:12:02Z
- **Merge commit:** 1ef7ef415bd40c9f7f569cee29f023bddd126e15
- **Base / Head:** main / UOR-Foundation:kimi/d32-zeta-complex-plane-20261014
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Files the owner's 14 October confirmation as a D32 addendum: floating point is permitted **only** on the complex plane for Riemann zeta-zero imaginary parts (`rho_k = 1/2 + i·gamma_k`, prime log-phases `theta_k(p) = gamma_k ln p`) in the `C^4 -> R^4` / `C^8 -> R^8` maps used for prime routing. All downstream routing, lattice, FBS, reads, gates, writer and learning stay integer/exact; no hold is released; any sheet using the exception must declare the exact map and its integer boundary.

Source: owner confirmation to Kimi (PM) 2026-10-14, quoted formulation in #2028 comment 6157803441, confirmation request #2029 comment 6157856812. References #2028

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

