# #413 — fix(core): #407 — CTX_SAMPLE 6_000 -> 50_000; fifth kappa re-pin (era artifact)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-04T18:15:45Z
- **Updated:** 2026-10-09T14:17:05Z
- **Closed:** 2026-08-04T18:35:33Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/413
- **Merged:** 2026-08-04T18:35:32Z
- **Merge commit:** 1cb5421df2f2748c9b7e2a228746a4a4d70623bf
- **Base / Head:** main / UOR-Foundation:i407-ctx-sample
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Closes #407.

**Fix (per sweep attribution on the issue / PR #411):** `CTX_SAMPLE` 6_000 → 50_000 (~23 → ~195 samples/centroid across 4 stages × 256); `CTX_ITERS` stays 6 (iteration count measured as noise).

**Fifth kappa re-pin**, canonical_deterministic mode (`TLESS_CANONICAL_DETERMINISTIC=1`, per #265 D2): container κ `blake3:ef6a20f3…` → `blake3:8fbf3f68…` (1,346,836 bytes). Token-side pins and thresholds byte-identical to the previous baseline. Container κ verified mode-invariant on linux/x86_64. `kappa_reproduction` passes bit-identical against the adopted fixtures (canonical env, full internal compile, 84.9s).

**Record row on the new era artifact** (anchor_infill, fixture 500k/2507 stories): store 34.9% | product fusion 43.2% (50.1% @off3) | store×tri×fwd 43.5% | bits: product 6.800, store 8.513 — statistically equivalent to the previous artifact's row (34.7 / 43.4 / 43.6 / 6.803). The sample-size term is small on the fixture, as the sweep predicted; the fix buys era self-consistency and compile hygiene.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

