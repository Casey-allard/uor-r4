# #2064 — Align README and status figures with the code-confirmed geometry

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T17:31:15Z
- **Updated:** 2026-10-09T17:32:58Z
- **Closed:** 2026-10-09T17:32:30Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2064
- **Merged:** 2026-10-09T17:32:30Z
- **Merge commit:** d53d5dff3dcea3e5554facd39f08c043700a0cbb
- **Base / Head:** main / UOR-Foundation:claude/geometry-consistency
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The README's *How it works* section and two status figures (`architecture.svg`, `geometry-stack.svg`) predated the code-level formula check done for `docs/geometry.md` (#2059). This brings them in line:

- **Zeta zeros:** a fixed phase table γ·(log p − log q) over prime pairs, used by the native learner. They are not positional phases in the stack and not learned (README plus both figures).
- **Quaternions** drive the `r`-layer recurrence, h_t = λ_t(u_t·h_{t−1}) + √(1−λ_t²)a_t. The `a`-layer reads use Lorentz or dot scores with an age term and a NoRead slot.
- **Prime routing** admits records by gcd of prime products or by the longest ordered n-let.
- **The icosian snap** of u_t is an optional training-time step; which headline checkpoints used it isn't verified.
- **Z[φ]** is `ZPhi { a, b }`, and chirality and polarity are exact signs of those coordinates.

Dated research records that mention these mechanisms were checked and are consistent, and they stay as written. This is a documentation correction, not new research, so it adds no current-state entry.

Checks: claim wording passes; the SVGs parse; no "positional" zeta wording remains in README, STATUS, the figures or `docs/geometry.md`.

References #2028

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6085954117 — 2026-10-09T17:31:17Z — Casey-allard

Exact-head self-review at `d8c44fb04d8f3bfd41e95c001b49145ccdabc257`: each corrected sentence matches briefs/geometry-math.md second pass (code-confirmed); SVGs parse; claim wording passes.


---
_Comment/review ids in this file: 6085954117_

