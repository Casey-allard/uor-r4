# #2459 — docs/labs: rung-0 completion OwnerReleasePlan (per-seed)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-14T08:52:56Z
- **Updated:** 2026-10-14T08:57:19Z
- **Closed:** 2026-10-14T08:56:49Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2459
- **Merged:** 2026-10-14T08:56:49Z
- **Merge commit:** 788be6dee840bfd4ea24d5fb5163a6467e40c87d
- **Base / Head:** main / UOR-Foundation:kimi/completion-release-plan
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Draft for Kimi PM review: the rung-0 completion OwnerReleasePlan packet, modeled on `docs/labs/rung0-pilot-release-2026-10-14/`, as two per-seed plans. Non-authorizing: this packet releases nothing, writes no owner record, and runs no compute. Training and M2 remain HELD; D33 does not extend to the completion.

## Contents
- `docs/labs/rung0-completion-release-2026-10-14/README.md` — packet record with the per-seed identity table, scope, bars/guard/B3/diagnostics, release path, resource admission and open items.
- `SEED1_PLAN.json` / `SEED2_PLAN.json` — `uor-r4.completion-owner-release-plan/1` plans (status `HELD_OWNER_ACTION_REQUIRED`): seed 1 resumes from checkpoint-1640 (`46e8ed5e…`, old-lineage baseline, guard ≈3.67% ≈525 strict wins; batches 1,640..16,403 = 14,764), seed 2 is a fresh-init full pass 0..16,404 with `u` measured at its checkpoint 0, explicitly not chained to seed 1.
- `seed1-config.json` / `seed2-config.json` — standalone `DriverConfig`s (V4096/B4/C192/K8, batch 64, schedule `canonical_vector_then_read_v1`) matching the pilot's config with only the seed changed.
- `sheet.md` — byte copy of the completion sheet at merge `4efd3f95b` (FIT (design), Amendments 1–3), sha256 `4c317f1e…`.

## Identity binding
- Frozen discrete source `33564b75e02e945ec5d5c94740c5741362289236` (Claude APPROVE at #2454 head `c8bf51909`, comment 6164246420); `driver_source_sha256` `cb090e85…` reconstructed from those exact bytes (method validated against the pilot driver `236e7fb6…`). #2454 merge sha recorded at delivery (open item a).
- source/kernel/geometry retained exactly from the pilot pins (the executing code refuses alternatives; the qualified build must compile the frozen source unmodified — Claude N3).
- config/lineage reconstructed with serde declaration-order encoding, method validated by reproducing the published seed-7 lineage `2cebef05…` and the pilot's cross-confirmed seed-1 `config 6252b613…` / `lineage 6db5ecf5…` exactly. Seed-1 completion lineage `94330d90…`; seed-2 `config 029965ae…` / `lineage 36bd0c03…` — to be cross-confirmed via `completion-build-identity` (open item b).
- Two placeholder fields per plan (`executable_sha256`, `claude_review.sha256`) are filled only from the qualified build and Claude's binding `APPROVE_A2_COMPLETION_ADMISSION_DIFF` records (N4; open items b/c). `completion-check-plan` fails closed on placeholders, so the packet is deliberately not executable in this state.

Open items: (a) #2454 merge sha; (b) qualified build pins + cross-confirmation; (c) Claude's binding review records; (d) the owner's two owner-personal release decisions (no seed-1→seed-2 chain); (e) Claude's one-line Amendment-3 re-confirmation.

References #2029

Line: evaluation integrity · count 1/3 · headline 37/232 → 37/232 (unchanged)

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

