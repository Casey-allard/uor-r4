# ORDER: d11-exporter-float-trunk, claude-126

Lab: claude. Item: [d11-exporter-float-trunk](../../../../roadmap/items/d11-exporter-float-trunk.md). Base: `7670779dd`.

## Goal

The item's done-when reads:

> A float-trunk snapshot exports to D11 with the audits green and the export gap measured inside the band. The trunk is a SpiralCore/FBS recurrence trunk with the Hamilton gate and the native vector writer head, as Stage 3 trains it. The synthesis (Stage 4) adds tokens per second, reported for decode and for context separately.

**This unit is Stage 4b: the export gap of a trained trunk, scored on the GPU in the artifact's own computation, and the artifact's tokens per second on the laptop.**

Stage 4a (claude-125, #3264) gave the float trunk a D11 mode that computes what the artifact computes, and proved its parity on a random-init trunk. Stage 3 (claude-119, #2891) trained this trunk, but saved no checkpoint, and its "integer-export" mode snapped only the gates and the writer: it was not the artifact.

**The design:**
- **Runner.** A new mode `d11-export-gap` of `spiralcore-fbs-sheet`, on Stage 3's inputs and recipe.
  - It trains the four Stage 3 arms on two seeds. The band keeps Stage 3's FIT'd definition, the largest two-seed DEV loss difference over the arms.
  - It saves the exported arm, `Fbs_1Atom_Active`, with `StackModel::save`. Only one-atom routes convert, and Stage 3's capacity gate kept one atom.
  - It reloads the saved model and checks that it scores the plain panel exactly as the trained model did.
  - It rounds the reloaded copy as exported, then scores the DEV panel with `evaluate_dev_panel_d11` and a D11 loss.
  - It computes the paired float-versus-D11 top-1 discordance, `b` and `c`, and writes the snapshot and the plain, export and D11 readouts for a laptop parity check.
- **Laptop.** `fbs-export` writes the trained trunk's artifact. `fbs-compare` holds the D11 readout against the integer session. A new `fbs-toks` bin times the artifact's integer session, for context (prefill) and decode separately.

The bars, the instrument checks and the decision table are fixed in a one-page prep sheet with a fresh non-author FIT before any GPU run.

## Cancel probe (run first)

Cancel if a trained SpiralCore/FBS checkpoint has since been saved, or the export gap of a trained FBS trunk in the artifact's computation has been measured. To check:
- list the store's `results/` for an FBS checkpoint;
- grep `crates/uor-r4-float-train/src` for a runner that calls `StackModel::save`, and for a caller of `evaluate_dev_panel_d11`;
- list the open PRs for overlapping work.

Result at `7670779dd`:
- the store holds no FBS model checkpoint;
- only the writer test calls `save`;
- nothing calls `evaluate_dev_panel_d11`;
- no other open PR touches this work.

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

1. **The runner mode and `fbs-toks` exist.**
   - The D11 loss refuses an unrounded trunk, as the D11 panel does.
   - Host tests pin the paired-gap arithmetic, the save-and-reload check and the `fbs-toks` statistics, and each piece's set-aside fails its test.
2. **The prep sheet** `docs/harness/prep/d11-exporter-float-trunk-stage4b-2026-10-26.md` has a fresh non-author `TEST FITNESS: FIT` on #2820, and a D67 pre-run review is posted on #2820 before the pod.
3. **The run, on one GPU, after the probe ladder.**
   - The ladder: the synthetic probe, then the throughput probe, then the run.
   - The gap is measured against the sheet's bars, with its instrument checks reported.
   - The evidence goes to the HF store with a `SHA256SUMS`, and the pod is down.
4. **The laptop half.**
   - The trained trunk's artifact exports.
   - Both zero-multiply audits are green.
   - `fbs-compare` reports the D11 readout against the integer session.
   - `fbs-toks` reports decode and context tokens per second.
5. **A REPORT and a notebook row.**
   - If the gap is inside the band on both seeds with every instrument check green, the item's status is done, and the REPORT says `Closes #2820`.
   - Otherwise it says `References #2820`, and it records what the outcome decides.

## Stop and ask if

Stop and decide; never wait.
- **An instrument check fails** (the save-and-reload check, the D11 parity, the DEV denominator, the Q12 oracle). Classify it instrument or object. Repair an instrument fault in this unit; it never counts as a gap.
- **The work outgrows one PR.** Land the runner mode, `fbs-toks` and the sheet first; the run then follows in the next unit.
