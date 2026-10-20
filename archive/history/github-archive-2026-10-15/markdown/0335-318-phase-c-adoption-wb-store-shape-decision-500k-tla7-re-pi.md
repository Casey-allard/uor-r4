# #335 — #318 Phase C adoption: WB store-shape decision, 500k+TLA7 κ re-pin, mantissa fold row, BASELINE.md

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-08-01T21:37:57Z
- **Updated:** 2026-08-01T23:29:17Z
- **Closed:** 2026-08-01T23:29:17Z
- **Assignees:** auser
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/335
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Tracks the Phase C adoption items left over from #318 (dot-residual path, shipped as era TLA7 in PR #329). The #318 DoD itself is met; these adoption items were identified in the design note (docs/dot_residual_phase_b_design.md, "Phase C (adoption)" §90) and PR #329's deferred list but have no tracker — #327 closed via PR #332 having delivered only the corpus-refresh tooling, not the re-pin.

## Items

1. **WB/key-dispersion store-shape decision** — TLA7 WB 8.02 vs TLA6 7.15; the residual trade-off is real and needs the #244-harness decision (design note §62, §71 — the WB regression bound can still in principle veto shipping the residual path). Membership beam currently stays on the #281 rule pending this.
2. **κ re-pin: 500k corpus + TLA7 in one pin** — fixtures on main are still 150k/TLA6 (baseline_kappa.json re-pinned 2026-07-30 for #243 Phase C; container 1,051,916 B vs TLA7's 1,346,836 B). Follow the AGENTS.md re-pin procedure (dump_baseline_kappa → review diff → adopt → TLESS_REPIN_WRITE=1) with an era note. κ tests skip vacuously if the checkpoint is absent — verify /tmp/ref/out/model.bin first.
3. **Mantissa-bit fold refinement row** — flagged as "a live Phase C candidate at the pinned era" in #327's closing comment; no harness row exists.
4. **BASELINE.md + certifier-side TLA7 witness rows** — docs/transformerless/BASELINE.md has no TLA6/TLA7 entries; no TLA7 witness rows in the certifier.

## DoD

Store-shape decision recorded with harness evidence; fixtures re-pinned at 500k+TLA7 with era note; mantissa fold row run (positive or recorded negative); BASELINE.md and witness rows landed. All four local gates green per AGENTS.md.

## Comments (1)

### comment 5153965943 — 2026-08-01T23:22:42Z — auser

Phase C follow-up is implemented in PR #340 (ready, checks green, queued for protected auto-merge): https://github.com/UOR-Foundation/uor-r4/pull/340

500k/TLA7 decision evidence: shipped A 34.7/39.0/8.0249 WB/179,068 keys; cpy8 residual 35.3/39.6/8.5247/195,650 (quality gain but WB bound fails); mantissa-bit fold 34.6/38.9/8.3097/162,119 (WB improves but no quality gain). The persisted TLA7 witness is 512/512. The selected store shape remains single-key/query-beam; no centroid-width or mantissa refinement adoption.


---
_Comment/review ids in this file: 5153965943_

