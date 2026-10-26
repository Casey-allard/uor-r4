# ORDER: d11-exporter-float-trunk, claude-127

Lab: claude. Item: [d11-exporter-float-trunk](../../../../roadmap/items/d11-exporter-float-trunk.md). Base: `f33b44f74`.

## Goal

The item's done-when reads:

> A float-trunk snapshot exports to D11 with the audits green and the export gap measured inside the band. The trunk is a SpiralCore/FBS recurrence trunk with the Hamilton gate and the native vector writer head, as Stage 3 trains it. The synthesis (Stage 4) adds tokens per second, reported for decode and for context separately.

**This unit repairs the integer FBS runtime, then reads the Stage 4b verdict.**

Stage 4b (claude-126, #3265) was blocked. The exported artifact of the trained trunk disagreed with the D11 readout on 31% and 63% of positions on seed 1, and 1.4% and 1.6% on seed 2. A seed-1 diagnostic placed the departure in the integer artifact: the D11 GPU mode matched the artifact's quantized reference on 14,463 of 14,464 positions, and the integer session matched it on 6,641.

The integer FBS path was only ever tested on small, square trunks: claude-122's hand-built trunk and claude-125's random-init one, both width 32, MLP 32, 2 layers, and gains near 1. The trained trunk differs from them in four ways:
- the MLP is not square: width 256, MLP 384;
- it has 6 layers;
- its gains, taps and biases are larger, up to about 2.6;
- its decay offsets exceed 1, up to 3.5.

**The plan:**
1. **Localize.** A finite host control: the synthetic trunk of claude-122, made shape- and range-configurable, compared between the integer session and the quantized reference across its whole vocabulary. Vary one difference at a time to find which breaks agreement.
2. **Repair**, with that control at the breaking shape as the failing-first test. Show it fails with the fix set aside.
3. **Re-run `I-D11-Artifact-Agreement`** in a short GPU job on the stored Stage 4b snapshots and readouts (`results/claude/d11-fbs-export-gap-2026-10-26/`), under claude-126's FIT'd prep sheet. No retraining.
4. **Read the verdict** of `G-D11-ExportGap` from claude-126's stored receipt, together with the new parity.

## Cancel probe (run first)

Cancel if the integer FBS runtime was changed or repaired after `f33b44f74`, or another PR is working on it. To check:
- `git log f33b44f74..origin/main -- crates/uor-r4-integer/src/stack`;
- the open PRs.

Result: no commit touches `stack/` after `f33b44f74`, and no other open PR touches the FBS runtime. **Not cancelled (instrument: none needed).**

## Files in scope

- `crates/uor-r4-training/**`
- `crates/uor-r4-integer/**`
- `crates/uor-r4-float-train/**`
- `docs/harness/prep/d11-exporter-float-trunk*`
- `docs/labs/d11-exporter-float-trunk/**`
- `docs/loop/tasks/d11-exporter-float-trunk/**`
- `docs/research/notebook/m4-serving.md`

## Done when

1. **Localized.** The difference that breaks the integer session's agreement with its quantized reference is named, with the shape at which a finite host control first fails.
2. **Repaired.**
   - The control at that shape passes: integer tokens and kept cells equal the reference's, except near-tie exceptions as in claude-122's control, and the Q12 query is within 2 LSBs.
   - Set aside, the fix fails the control.
   - The existing FBS tests and the zero-multiply audits stay green.
3. **The pod parity is re-run** on the stored snapshots and readouts after a D67 pre-run review. `I-D11-Artifact-Agreement` is read against claude-126's bar. If it is green, `G-D11-ExportGap` is read from the stored receipt.
4. **A REPORT and a notebook row.**
   - If the gap is inside the band on both seeds, the item is done and the REPORT says `Closes #2820`.
   - Otherwise the REPORT says `References #2820`, and it records what the outcome decides.

## Stop and ask if

Stop and decide; never wait.
- **The departure is not one difference but several.** Repair each with its own control, or split the work into units.
- **The repair changes the artifact format.** It then needs a schema decision. Record it and keep the old schema refused.
