# ORDER: d11-exporter-float-trunk, claude-125

Lab: claude. Item: [d11-exporter-float-trunk](../../../../roadmap/items/d11-exporter-float-trunk.md). Base: `509f1d80d`.

## Goal

The item's done-when reads:

> A float-trunk snapshot exports to D11 with the audits green and the export gap measured inside the band. The trunk is a SpiralCore/FBS recurrence trunk with the Hamilton gate and the native vector writer head, as Stage 3 trains it. The synthesis (Stage 4) adds tokens per second, reported for decode and for context separately.

The item puts the export gap on the GPU (CHARTER §3.5, §3.6): a panel-scored evaluation of the exported artifact. It needs either "a GPU evaluator of the container or a bit-exact parity proof against the sheets' export mode". No GPU evaluator of an FBS stack artifact exists. The sheets' export mode is not the artifact either (claude-124's attribution):
- it leaves the recurrence transition unsnapped;
- it reads every table at full precision.

**This unit is Stage 4a: a D11-equivalent evaluation mode on the GPU, with its parity to the integer session proven on finite controls.** With it, a DEV score taken on the GPU in that mode stands for the artifact's score, and Stage 4b's sheet can measure the export gap on the GPU.

**The design:**
- **Transition snap.** The float trunk's D11 mode also snaps the recurrence transition's rotation to the 120 icosians, as the integer path does (`format.rs` `RefRecurrence::step`, `FbsQuatMode::Icosian`).
- **Gates.** The gates are snapped to the 120 icosians directly, as the integer path snaps its own fixed-point gate, and as `HamiltonArm::HamiltonIcosianHard` does. They are not rounded to Q12 first, as the sheets' export mode rounds them. This was corrected before the run: the integer reference's `INTEGER` snaps do not round, and claude-124's attribution shows that the Q12 rounding alone moves queries by up to 0.536.
- **Rounded tables.** The trunk's tables are read rounded exactly as `export_fbs_trunk_uorlut01` writes them: grid-coded gains and taps, Q16 biases and decay offsets, and the Q12 embedding, which is also the writer table. Every value is rounded on the host once, and the GPU forward then runs on the rounded copy.
- **Parity proof.** On claude-124's random-init trunk:
  - the GPU in D11 mode against the converted trunk's quantized, fully snapped reference (`forward_f32_with(INTEGER, quantized = true)`);
  - the GPU in D11 mode against the integer session itself;
  - over all single tokens and the 8 full-context sequences.

This unit does not train a model and does not measure a gap.

## Cancel probe (run first)

Cancel if the float-train crate already has a mode that snaps the recurrence transition, or a GPU evaluator of an FBS stack artifact. To check: grep `crates/uor-r4-float-train/src` for an icosian snap of the transition in `composed_recurrence`, and for any reader of `uor-r4.lut-stack/4`.

Result at `509f1d80`:
- `composed_recurrence` has no snap or icosian step: its transition is `raw / sqrt(|raw|^2 + 1e-6) * lambda` only;
- nothing in `crates/uor-r4-float-train/src` names `lut-stack/4`, `STACK_FBS_SCHEMA` or `IntegerStackModel`.

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

1. **The D11 mode.** The float trunk has a D11 evaluation mode: gates as in export mode, the transition snapped, and tables rounded as exported. It is reachable from `fbs_float_readout` and from the sheets' panel scoring. Host tests pin each piece, and each piece's set-aside fails its test.
2. **The parity run, one GPU, on a random-init trunk.** Bars fixed here, before the run:
   - D11-mode GPU against the quantized, fully snapped reference: largest query gap at most `1e-3`, and tokens agree on at least 250 of 256 single tokens and 60 of 64 sequence positions;
   - D11-mode GPU token against the integer session's token: at least 250 of 256 and 60 of 64.

   A miss confined to at most 6 positions, each traced to a near-tie (an icosian snap, a rate index or a Q12 rounding), is an instrument miss. A mapping fault is repaired in this unit.

   Two planted faults must each break parity: tables left unrounded in D11 mode, and the transition snap turned off. "Breaks parity" means the set-aside misses at least one of the bars above, against the quantized reference or against the integer session. This definition is fixed after the pre-run review and before the run.

   The rounded trunk must also export to the same artifact bytes as the original: `cmp` in the job log.
3. Audits green, `loop gates --full` green, and a REPORT that hands Stage 4b a mode it can score on the GPU.

## Stop and ask if

Stop and decide; never wait.
- **The D11 mode cannot reach the integer session's tokens within the bar.** Classify the miss: an instrument fault, such as float-versus-fixed-point rounding near ties, or a semantic difference to repair.
- **The work outgrows one PR.** Land the D11 mode and its host tests first; the parity run then follows in the next unit.
