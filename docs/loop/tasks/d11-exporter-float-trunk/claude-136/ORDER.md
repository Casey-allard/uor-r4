# ORDER: d11-exporter-float-trunk, claude-136

Lab: claude. Item: [d11-exporter-float-trunk](../../../../roadmap/items/d11-exporter-float-trunk.md). Base: `4b77eb7f5`.

## Goal

The item's done-when reads:

> A float-trunk snapshot exports to D11 with the audits green and the export gap measured inside the band. The trunk is a SpiralCore/FBS recurrence trunk with the Hamilton gate and the native vector writer head, as Stage 3 trains it. The synthesis (Stage 4) adds tokens per second, reported for decode and for context separately.

**This unit is the decision-relevance bound.** The decision review after [claude-135](../claude-135/REPORT.md) required it (#2820 owner log, comment 6426588305, change 4):

> Report how many net top-1 rows the leftover could move, beside F_I and the three FAILs' margins. If the leftover cannot turn any FAIL into a pass, the REPORT says so: the exporter question then cannot change the item's verdict.

- **The bound needs no new run.** claude-132's six stored, verified receipts already pair the plain float panel (F) with the D11 forward (D) and with the artifact (A), each with its own paired F_I.
- **F against D is the gap a perfectly faithful exporter (A = D) would show.** If it is outside F_I on every model, no exporter correction can turn a FAIL into a pass.
- **This is grading of stored GPU outputs, on the laptop** (run file: counting and grading of stored GPU outputs), with no model compute.
- **The decision it feeds.** If the bound holds on all six:
  - the exporter-faithfulness investigation (claude-132 to claude-135) ends without the continuous-localisation run;
  - the D-against-A leftover is reported as part of the export gap;
  - the next unit is claude-131's pivot note's decisive run, under a fresh decision review recorded as an owner log.

## Cancel probe (run first)

Cancel if the bound is already recorded, or if another unit on this item is open. To check:
- the item's tasks on `origin/main`;
- the open PRs;
- the notebook `docs/research/notebook/m4-serving.md`.

Result (26 Oct, `origin/main` at `4b77eb7f5`):
- No commit after `4b77eb7f5` touches this item's tasks or the notebook.
- The only open PR on this item is this unit's, #3305. The others are #3304 (antigravity, m1-close-saved-model), #3302 (kimi) and #3169 (codex).
- The notebook records no decision-relevance bound.

**Not cancelled (instrument: none needed).**

## Files in scope

- `docs/loop/tasks/d11-exporter-float-trunk/**`
- `docs/research/notebook/m4-serving.md`

## Done when

1. **The bound, per model, from claude-132's receipts** (`results/claude/d11-fbs-residual-2026-10-26/resid/`), checked against their `SHA256SUMS`:
   - F against A, its net and F_I;
   - F against D, its net and F_I;
   - D against A, its net;
   - whether F against D is inside its F_I.
2. **A fresh decision review of the next step, recorded on #2820 as an owner log.**
3. **A REPORT and a notebook row,** with `References #2820`. The item stays open.

## Stop and ask if

Stop and decide; never wait.
- **Any model's F against D is inside its F_I:** the exporter question can change that model's verdict, and the continuous localisation goes ahead as the decision review set it.
- **The bar or the comparator** (CHARTER §3.7): the owner's, already asked on #2028. Nothing waits on it.
