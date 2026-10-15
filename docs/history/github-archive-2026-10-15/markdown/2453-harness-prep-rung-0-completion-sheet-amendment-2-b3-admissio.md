# #2453 — harness/prep: rung-0 completion sheet Amendment 2 (B3 admission ceiling)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-14T08:06:35Z
- **Updated:** 2026-10-14T08:09:17Z
- **Closed:** 2026-10-14T08:08:33Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2453
- **Merged:** 2026-10-14T08:08:33Z
- **Merge commit:** c109b2109d1fdfd80a8cb5bd94d382033f2f8f38
- **Base / Head:** main / UOR-Foundation:kimi/rung0-completion-sheet-amend2
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Amendment 2 to `docs/harness/prep/m1-discrete-rung0-completion-2026-10-14.md`, registering reviewer item B3 from Claude's conditional FIT ([#2029 comment 6163704400](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6163704400)). Sheet text only: no bar, guard, law, data, evaluator, dose or code change.

Line: evaluation integrity · count 1/3 · headline 37/232 → 37/232 (unchanged — no model compute in this PR). Decision: NOT YET PROMOTED (registration only).

- **B3a — header:** amendment 2 (2026-10-14) bullet after amendment 1, registering the admission ceiling on the rank bars per the conditional FIT.
- **B3b — bars/guard/evaluator:** `discrete_evaluate` ranks top-1/top-8 only over identities in admitted cells, so top-1 and top-8 hits ≤ target-cell admitted rows = `admitted_cell_count_sum − address_rival_count_sum`; pilot receipts baseline 114,296 − 108,616 = 5,680, pilot end 114,296 − 112,116 = 2,180 (below bar (ii)'s 4,775, just above bar (i)'s 1,941); bars remain attainable in principle because admission changes as the codebook moves; the measure is reported at every checkpoint alongside top-1/top-8 with each seed's checkpoint-0 value as its baseline; the receipts already emit both sums — no code change.
- **B3c — outcome decisions:** if bar (i) or (ii) is missed while target-cell admitted rows at 100% are below that bar's count, the FIRST attribution (alongside B2's schedule step) is writer admission — the address-objective trade-off — an instrument/configuration step before any mechanism language; the matching named successor is an admission/address-objective configuration for Codex's analysis to specify, NOT adopted here.
- **B3d — steps:** step 7's per-checkpoint report list gains target-cell admitted rows (checkpoint-0 baselines per seed).
- **B3e — verdict:** TEST FITNESS: FIT (design) per Claude's conditional FIT (6163704400); Codex source check on D5 when posted; a FIT releases nothing — training remains HELD.

Checks run at this exact head (`134cf52a6`): `UOR_LAB=kimi ./local/harness-target/debug/uor-harness check` → exit 0, 0 errors (warnings only; this sheet's `prep.fit_before_run` PENDING warning clears). `python3 scripts/check_claim_wording.py` → pass. `python3 scripts/harness/cli.py lint check --body-file <this body>` → 0 errors, 1 warn (no dated record path — not applicable: registration-only change with no research result, same delivery shape as #2451).

References #2029


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

