# ORDER: d11-exporter-float-trunk, claude-124

Lab: claude. Item: [d11-exporter-float-trunk](../../../../roadmap/items/d11-exporter-float-trunk.md). Base: `482a192cc`.

## Goal

The item's done-when reads:

> A float-trunk snapshot exports to D11 with the audits green and the export gap measured inside the band. The trunk is a SpiralCore/FBS recurrence trunk with the Hamilton gate and the native vector writer head, as Stage 3 trains it. The synthesis (Stage 4) adds tokens per second, reported for decode and for context separately.

claude-123 (#3261, merged as `482a192c`) built the converter. On a random-init trunk, the float GPU picked the converted reference's token on only 67 of 256 tokens, and that unit could not say why. **This unit is Stage 4's first step, the parity proof.** It separates two things:
- the export gap, which comes from icosian snaps that the float path does not apply;
- any fault in how the converter maps the copied tables.

A trained-model export gap needs that split first.

**The design.** The converted trunk's float reference (`FbsTrunkSnapshot::forward_f32`, a test oracle in `uor-r4-integer`) snaps three things to the 120 icosians: the recurrence transition, the recurrence output gate and the feed-forward gate. It gains switches that turn each snap off and normalize instead, exactly as the float CUDA kernels do:
- the Hamilton gates use `g / sqrt(|g|^2 + 1e-12)` (`cuda_stack.cu` `hamilton_gate_fwd`);
- the transition uses `raw / sqrt(|raw|^2 + 1e-6) * lambda` (`composed_recurrence`).

The gates also get the sheets' export-mode form, `snap(round(g * 4096))` (`hamilton_sheet.rs` `try_hamilton_gate_fwd_cuda`). The existing `forward_f32` keeps every snap on and is unchanged.

`fbs-compare` then holds each float GPU readout against the matching reference:
- **Mapping proof:** the plain float GPU against the reference with no snaps, and the export-mode float GPU against the reference with gates in export form and the transition unsnapped. If the copied tables are mapped right, each pair agrees to within float rounding.
- **Gap attribution:** the integer session against the fully snapped, quantized reference, as before, plus each reference variant against the next, which needs no GPU. Together they say how much of the 67/256 each snap explains.

`fbs-compare` also refuses a readout that does not cover every token exactly once (claude-123's round-2 finding).

## Cancel probe (run first)

Cancel if the float CUDA path and the integer reference already agree within float rounding, which would leave nothing to split. Result: claude-123's two GPU runs both read 67 of 256 tokens, with a largest hidden gap of 2.825. **Not cancelled.**

Cancel if the reference cannot copy the float normalizations exactly, for example a different epsilon or rate table. Result, read at `482a192c`:
- the Hamilton gate epsilon is `1e-12` (`cuda_stack.cu:183`);
- the transition epsilon is `1e-6` (`geometric_stack_native.rs`, `composed_recurrence`);
- the rate table is `lambda = 2^-(i+1)` with `keep = sqrt(1 - lambda^2)` on both sides (`cuda_stack.cu:231-258`; `format.rs` `RefRecurrence::step`), so they differ only by the CUDA side's f32 rounding of `keep`.

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

1. **Reference variants.** The reference gains snap switches. Its default stays bit-identical to `forward_f32`, held by a test. Host tests check each switch on hand-computed blocks, and a set-aside of each switch fails its test.
2. **Readouts.** `fbs_float_readout` can read out in the sheets' export mode as well as plain. The GPU test writes both readouts.
3. **`fbs-compare`.** It reports each pairing above, and refuses an incomplete or repeated readout (host-tested).
4. **The GPU run, on a random-init trunk.** Bars fixed here, before the run, and amended once, before the run, after the pre-run review (#2820):
   - **Single tokens.** Plain float GPU against the reference with no snaps: largest hidden gap at most `1e-3`, and tokens agree on at least 250 of 256. The same bar holds for the export-mode readout against its matching reference.
   - **Sequences (the amendment).** A single token from a zero state never reaches the transition rotation, convolution taps 1 to 3 or the carried state, so one position proves none of them. The readouts therefore also hold 8 fixed sequences of the full context, 64 positions. Their bar: largest hidden gap at most `1e-3`, and tokens agree on at least 60 of 64, for each readout against its matching reference.
   - **Near-ties.** A miss confined to at most 6 positions, each traced to one icosian snap or rate-index flip at a near-tie, is an instrument miss, not a mapping fault. The gap bar takes the worst position, so one flip fails it.
   - **f32 only.** The model runs in f32: `hidden()` refuses any other precision (`params()`).

   A miss is classified: a mapping fault in the converter or the reference is a real finding and gets repaired, while a bar set too tight is an instrument fault. The gap attribution is a reading with no bar.
5. Audits green, `loop gates --full` green, and a REPORT that says what Stage 4 still owes.

## Stop and ask if

Stop and decide; never wait.
- **The mapping bar fails.** Find which table or step differs, using a per-layer comparison if needed, and fix it in this unit when it is the converter's or the reference's fault.
- **The float export mode does not snap what the integer path snaps.** Record the difference. It shapes the Stage 4 sheet.
- **The work outgrows one PR.** Land the reference variants and `fbs-compare` first. The GPU run then follows in the next unit.
