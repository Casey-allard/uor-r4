# ORDER: d11-exporter-float-trunk, claude-132

Lab: claude. Item: [d11-exporter-float-trunk](../../../../roadmap/items/d11-exporter-float-trunk.md). Base: `7aedeea19`.

## Goal

The item's done-when reads:

> A float-trunk snapshot exports to D11 with the audits green and the export gap measured inside the band. The trunk is a SpiralCore/FBS recurrence trunk with the Hamilton gate and the native vector writer head, as Stage 3 trains it. The synthesis (Stage 4) adds tokens per second, reported for decode and for context separately.

**This unit is the residual check, the first step of claude-131's §4.4 pivot note** ([decision review](https://github.com/UOR-Foundation/uor-r4/issues/2820#issuecomment-6419387874)).
- **The question.** Three configurations (claude-129, -130, -131) passed the loss half and missed the top-1 half. On claude-131's seed 2, the export-mode panel (E) is 126 top-1 rows from the artifact (A).
  - E is not the artifact's emulation: it does not snap the transition or round the tables.
  - The emulation is the D11 panel (D, `evaluate_dev_panel_d11`), which none of the three runs paired against A.
  - So it is open whether A computes its own D11 forward, or departs from it. A departure would be an exporter defect, an instrument fault bearing on all three results (§4.2, L-007).
- **The check.** For each of the six saved models (three configurations × two seeds), on one GPU:
  - **F:** the plain-float panel, which must reproduce the stored predictions;
  - **E:** the export-mode panel on the tables as trained;
  - **D:** the D11 panel, with the tables rounded as exported;
  - **H:** the D11 head on the artifact's own Q12 queries;
  - **A:** the artifact's predictions, read from its stored, verified evaluator run. No artifact is re-run.
  - D against A is paired under the sheets' paired test. H against A, D against H, and the Q12 drift between D's and A's queries say where any departure sits: the head or the trunk.
- **New code:** a `d11-residual` mode in `spiralcore-fbs-sheet`, with its tests.
- **No training.** This is GPU evaluation of stored models, so it gets a prep sheet with a FIT and a D67 review (§4.5).

## Cancel probe (run first)

Cancel if a paired D11-panel-against-artifact check of these models exists or is in progress. To check:
- `git log 7aedeea19..origin/main -- crates/uor-r4-float-train docs/loop/tasks/d11-exporter-float-trunk docs/harness/prep/`;
- the open PRs;
- a search of `origin/main` for a residual mode or a D-against-A pairing.

Result (26 Oct):
- No commit after `7aedeea19` touches those paths.
- The other open PRs are #3283 (kimi), #3280 (antigravity) and #3169 (codex). None touches float-train or this item.
- `origin/main` has no `d11-residual` mode.
  - The artifact-gap scorer pairs F against A only.
  - Stage 4b compared D11 top-1 counts with claude-129's artifacts unpaired: 1,911 against 1,907, and 2,290 against 2,294.

**Not cancelled (instrument: none needed).**

## Files in scope

- `crates/uor-r4-training/**`
- `crates/uor-r4-integer/**`
- `crates/uor-r4-float-train/**`
- `docs/harness/prep/d11-exporter-float-trunk*`
- `docs/labs/d11-exporter-float-trunk/**`
- `docs/loop/tasks/d11-exporter-float-trunk/**`
- `docs/research/notebook/m4-serving.md`

## Done when

1. **The mode.** `spiralcore-fbs-sheet --mode d11-residual` writes one receipt per saved model, with:
   - the F, E, D, H and A top-1 counts;
   - the pairings D–A, H–A, D–H, E–A and F–D;
   - the Q12 drift between D's and A's queries;
   - the save-load, input-pin, verify and DEV-denominator checks.
   Host tests pin its parsing and the drift arithmetic; a GPU test pins H on a synthetic trunk.
2. **A prep sheet, its FIT, and a D67 pre-run review.** The sheet pins the classification rule:
   - **faithful:** D against A within the paired test on every seed of every configuration;
   - **departure:** otherwise.
   It also says what each outcome decides.
3. **The run:** one pod job over the six saved models, with every receipt in the store.
4. **A REPORT and a notebook row,** with the classification and what it decides under the pivot note. This unit cannot close #2820, so the REPORT says `References #2820`.

## Stop and ask if

Stop and decide; never wait.
- **An instrument check fails** (save-load, input pins or verify): classify it first. It is a repair, never a reading.
- **H differs from A on rows where the queries are identical:** that is a head-side exporter departure. Report it as such, and open a repair item; do not explain it away.
- **The comparator or the bar** (CHARTER §3.7): the owner's. It is already asked on #2028, so this unit changes neither.
