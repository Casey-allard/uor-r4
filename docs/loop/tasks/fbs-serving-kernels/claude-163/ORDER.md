# ORDER: fbs-serving-kernels, claude-163 (the split)

Lab: claude. Item: [fbs-serving-kernels](../../../../roadmap/items/fbs-serving-kernels.md). Base: `cc24ae3a7`.

## Goal

The owner, 28 Oct 2026, in claude's chat: "I don't care what we serve at long as it is cheap". On the M1, a served token is 3.73 ms, and its six layers are 98.6% of it (claude-158). Each layer's two blocks are timed whole, so which part of a block to make faster is not known.

The item's done-when reads:

> 1. **The split:** each layer's gathers, core, gated product and requantization timed on the M1, by extending `measure_fbs_step_stages` and `fbs-toks --stages`. The parts sum to the layer within 10%.
> 2. **A vector kernel for the largest part,** written for D11 (CHARTER §3.6: integer, no multiply instruction; products through its tables), in `stack/fbs_simd.rs`, behind a runtime switch (no new cargo feature), with the scalar path kept for WASM and other targets.
>    - A host test holds every token, kept cell, query and session state equal to the scalar `step_native` over a synthetic trunk and a stored artifact.
>    - The kernel is in the audit's kernel list, and `scripts/audit_zero_matmul_serving.py --stack` passes on the release binary.
> 3. **Speed:** `fbs-toks` decode tokens/s on the M1 with and without the kernel, on claude-126's export, with the same replies.
> 4. One notebook row and a REPORT. This is host and serving work: no GPU and no sheet (CHARTER §4.5).

**This unit is done-when 1, with its notebook row and REPORT.** The split decides which part the kernel (done-when 2) is written for, so the kernel is the next unit's. The item stays `ready`.

## Cancel probe (run first)

Run at `cc24ae3a7`:
- **No open PR** touches `stack/session.rs`, `bin/fbs-toks.rs` or the audit script.
- **No split exists:** `session.rs` has no per-part clock (`measure_fbs_step_parts`: 0 matches).
- **The artifact is present:** claude-126's seed-1 export, fetched from `results/claude/d11-fbs-export-gap-2026-10-26/d11gap/save/seed-1/artifact.uorlut`, sha256 `314c3909c66b58522a641c3f5b51c25dda4bda5ab2274bcfd0b5ca5038fdb305`. That is claude-158's `314c3909…fdb305`.

All three held. **Not cancelled** (instrument: none needed).

## The work

- **`session.rs`:** `stack_step_fbs_timed` gains a `parts` flag. With it off, its laps are exactly claude-158's. With it on, each layer is timed in nine parts:
  - recurrence: norm (with its gain), the two input gathers, the core, and the output gather with the residual add;
  - feed-forward: norm (with its gain), the two input gathers, the gated product, the requantization, and the output gather with the residual add.
  - A new `measure_fbs_step_parts` calls it with the flag on. `stack_step_fbs` and `step` stay untouched for the audit.
  - A host test holds the parts-timed step equal to `step_native` (token, kept cells, query, session state), with `9 × layers + 6` times that tile the step.
- **`fbs-toks`:** `--parts` runs the protocol a third time through `measure_fbs_step_parts`. It reports each part's median, and each layer's parts summed against that block's median from the `--stages` run, per mille. It also gains a host test of that arithmetic. `--parts` implies `--stages`.
- **The run, on this M1:** `fbs-toks <artifact> --parts`, release build, binary sha256 recorded. Outputs sealed to `results/claude/fbs-serving-kernels-split-2026-10-28/` with a `SHA256SUMS`.

## Files in scope

- `crates/uor-r4-integer/src/stack/session.rs`
- `crates/uor-r4-integer/src/bin/fbs-toks.rs`
- `docs/loop/tasks/fbs-serving-kernels/**`
- `docs/research/notebook/m4-serving.md`
- `docs/roadmap/items/fbs-serving-kernels.md` (a note on what this unit met)

## Done when

1. Each layer's nine parts are timed on the M1 at the release build, and each block's parts sum to within 10% of that block's `--stages` median (900–1,100 per mille), at every layer.
2. The parts-timed step equals `step_native`, by host test.
3. `fbs-toks --parts`' arithmetic has a host test, and the plain and staged replies match.
4. One row in `docs/research/notebook/m4-serving.md` and a REPORT naming the largest part, the one the kernel unit takes.

## Stop and decide if

- **A block's parts miss the 10% band.** Then the clock is intruding: Instant reads are about 40 ns, so 54 laps should cost about 2 µs of a 3.7 ms token. Classify it as instrument, coarsen the split where the parts are shortest, and rerun. Never report a split outside the band as a measurement.
- **Replies differ between runs.** That is a determinism fault in the timed twin, so it stops the unit and is fixed first.

No step waits on Casey.
