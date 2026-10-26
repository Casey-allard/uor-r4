# ORDER: d11-exporter-float-trunk, claude-129

Lab: claude. Item: [d11-exporter-float-trunk](../../../../roadmap/items/d11-exporter-float-trunk.md). Base: `a605d033d`.

## Goal

The item's done-when reads:

> A float-trunk snapshot exports to D11 with the audits green and the export gap measured inside the band. The trunk is a SpiralCore/FBS recurrence trunk with the Hamilton gate and the native vector writer head, as Stage 3 trains it. The synthesis (Stage 4) adds tokens per second, reported for decode and for context separately.

**This unit measures the export gap of the exported artifact itself, and reads `G-D11-ExportGap`.**
- Stage 4b (claude-126) could not count its gap, because the D11 emulation did not match the artifact.
- claude-128 qualified a GPU evaluator of the artifact: bit-exact with the Rust integer session at 32,256 positions (#3274).
- Here that evaluator scores both stored Stage 4b artifacts over every one of the 428 DEV windows. Its tokens and Q12 queries at the 14,287 scored rows go into claude-126's bar, unchanged, on the GPU:
  - **the top-1 half:** b and c against the saved float model's panel;
  - **the loss half:** `MarginHinge(rivals = 8)` on the artifact's own queries, against the float loss and the two-seed noise band.

There is no emulation, and no retraining.

## Cancel probe (run first)

Cancel if the gap of these artifacts was measured after `a605d033d`, or another PR scores them. To check:
- `git log a605d033d..origin/main -- docs/loop/tasks/d11-exporter-float-trunk crates/uor-r4-float-train/src/spiralcore_fbs_sheet.rs`;
- the open PRs.

Result:
- No commit after `a605d033d` touches those paths.
- The open PRs are #3278 (this one), #3277 and #3276 (antigravity: m1-wider-code-carrier and r-vocab-4096-vs-16384), #3273 (kimi, m1-address-as-read) and #3169 (codex). None of the others touches this item.

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

1. **The scorer.** `spiralcore-fbs-sheet --mode d11-artifact-windows` writes the 428 DEV windows. `--mode d11-artifact-gap` scores one seed from `d11-fbs-eval run --verify` on them:
   - the artifact's prediction at each scored row is the evaluator's token at that offset;
   - its loss is the D11 head on the evaluator's Q12 queries, with the tables rounded as exported;
   - the float side is the saved model's own panel, checked against claude-126's stored predictions and loss.

   A host test pins the row mapping, and it fails if the mapping is set one position off.
2. **A prep sheet** amending Stage 4b, so that D is the artifact through the qualified evaluator. It carries the same bar, these instrument checks, a non-author FIT and a D67 pre-run review.
3. **The run on the pod.** `G-D11-ExportGap` is read on both seeds from the receipts: |b − c| ≤ F_I and |L_A − L_F| ≤ Δ_noise,L, with every instrument check green.
4. **A REPORT and a notebook row.**
   - If the gap is inside the band on both seeds, the item's done-when is met with claude-127's laptop tok/s, and the REPORT says `Closes #2820`.
   - Otherwise it says `References #2820`, and records what the sheet's FAIL row decides: for this trained configuration, direct post-training rounding into the artifact is disqualified (CHARTER §3.7), and the next configuration trains in the D11 mode before export.

## Stop and ask if

Stop and decide; never wait.
- **The evaluator disagrees with the Rust session** at any position of the 428 windows. That is an instrument fault: localize it. It is not a gap reading.
- **The saved float model does not reproduce claude-126's stored predictions.** That is an instrument fault, under `I-Save-Load`.
