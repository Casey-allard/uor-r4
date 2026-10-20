# #2430 — Rung-0 pilot OwnerReleasePlan — pinned, non-authorizing (R15) (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-14T03:47:41Z
- **Updated:** 2026-10-14T03:49:30Z
- **Closed:** 2026-10-14T03:48:41Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2430
- **Merged:** 2026-10-14T03:48:41Z
- **Merge commit:** f410a77c18c474ca1f694e0908454f82fdc5a4f6
- **Base / Head:** main / UOR-Foundation:kimi/rung0-pilot-plan-20261014
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Kimi gate 3 of the rung-0 pilot release chain: the complete OwnerReleasePlan packet at docs/labs/rung0-pilot-release-2026-10-14/.

Contents:
- PILOT_PLAN.json (schema uor-r4.pilot-owner-release-plan/1, status HELD_OWNER_ACTION_REQUIRED) with the full pinned execution identity: base e7d926fbe3, reviewed commit cbeeee084d5563dd87cd0e8e4850b65d2ea540c0 (Claude APPROVE head of #2421, crate byte-identical to qualified 35a48f5863), source/kernel/geometry/executable/driver-source hashes, config_sha256 6252b613c6bf, lineage_sha256 6db5ecf5d6e1, sheet_sha256 1029c3b92771.
- pilot-config.json: the D2 config with seed 7→1; passes every validate_pilot_config pin statically (verified by re-deriving serde-compact encoding; method validated against the published seed-7 lineage hash).
- sheet.md: byte copy of the FIT(design) rung-0 sheet at c1f920ca7 with both owner_release markers.
- review.json: Claude's ReviewRecord (APPROVE_A2_ADMISSION_DIFF, scope m1-a2-seed1-vector0..1639-diagnostic-pilot, receipt #2421 comment 6160178151).
- README.md: provenance, pin table, pilot scope, exact owner command.

Static verification: all owner_release admission checks (controller.rs) replicated offline — 36/36 PASS (one self-test literal corrected; the plan itself is consistent). owner-release itself was NOT run (R15: Casey-only, TTY-guarded).

This packet authorizes nothing by itself. Next: Codex cross-confirms config/lineage hashes with one build-identity run on the CUDA binary; then Casey's D4 decision.

References #2029

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

