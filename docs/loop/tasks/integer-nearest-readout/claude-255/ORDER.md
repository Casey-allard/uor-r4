# ORDER: integer-nearest-readout, claude-255

Lab: claude. Item: [integer-nearest-readout](../../../../roadmap/items/integer-nearest-readout.md). Base: `05021840e`.

## Goal

The item's done-when reads:

> 1. **The artifact carries its readout,** with a negative gate. An artifact without it decodes byte for byte as at the base, and an engine without it refuses a nearest artifact.
> 2. **The nearest step** serves `stack_whole_table_nearest`'s rule under the field, with a test against a reference scan and a negative gate.
>    - The browser path (`wasm_serve`'s step) gives the same token as the session's nearest on one artifact, with a test.
>    - The serving audit (`audit_zero_matmul_serving.py --stack`) and the WASM build pass.
> 3. **End-to-end token parity passes its bar, under a prep sheet with a fresh FIT and a pre-run review.**
>    - Integer greedy of (r-drop) step 16,384, memory on, exported raw (`fbs-export --raw-gates --raw-transition`, R), against the float greedy decoded again on the GPU with the read's tie rule on (F′: claude-220's arm-G command at the unit's head), on DEV and p-w at seeds 1–3.
>      - claude-232's snapped export (S) and claude-220's stored replies (F) are kept as readings.
>      - Both changes are claude-243's [decision card](../../loop/tasks/integer-nearest-readout/claude-243/DECISION.md), logged on #3462; Casey can reverse it. "Done" applies to the raw export, and the item then names its recipe as the served artifact's.
>    - **The bar is claude-245's [decision card](../../loop/tasks/integer-nearest-readout/claude-245/DECISION.md),** logged on #3462; Casey can reverse it. It replaces claude-243's first-token ⌊0.03 n⌋, which was registered against claude-227's 0.33% residual of a snapped export. Q1, Q2 and Q4 are read under teacher forcing on claude-244's counted positions, and Q3 on claude-243's free decode. A LOSS of any class fails this bar for that artifact.
>      - R picks the reference token as often as F′ does: no loss beyond 3 row-clustered SE (Q1), and at most half a point (Q2);
>      - the same holds at the opening, on free decode (Q3);
>      - R's differences sit at its own near ties (Q4: κ ≥ 4/9).
>      - **Q5:** every clear-margin disagreement is also accounted for by an exact trace, under claude-247's [decision card](../../loop/tasks/integer-nearest-readout/claude-247/DECISION.md), logged on #3462; Casey can reverse it.
>      - First-token and whole-reply agreement stay reported as readings; claude-243's MISS on the old bar is recorded as it is.
>    - **The item is done only when the bar passes.** On a miss, the sheet's row is recorded, the item stays `ready`, and the next unit locates the gap.
>    - Integer decodes run on the laptop: product serving (CHARTER §3.6), under the owner's 31 Oct allowance for this chain. The float side runs on the GPU.
> 4. **Tokens per second,** with the nearest scan counted (4,096 rows a token), and the resident memory.
> 5. **One notebook row, and a REPORT with the negatives first.**

## Files in scope

- `crates/uor-r4-integer/src/fbs_snapshot.rs`
- `crates/uor-r4-integer/src/stack/format.rs`
- `crates/uor-r4-integer/src/stack/session.rs`
- `crates/uor-r4-integer/src/stack/kernels.rs`
- `crates/uor-r4-integer/src/stack/tests.rs`
- `crates/uor-r4-integer/src/stack/synthetic.rs`
- `crates/uor-r4-integer/src/bin/fbs-export.rs`
- `crates/uor-r4-integer/src/bin/fbs-toks.rs`
- `crates/uor-r4-float-train/src/fbs_export.rs`
- `crates/uor-r4-float-train/src/bin/fbs-snapshot.rs`
- `crates/uor-r4-integer/src/stack/hamming.rs`
- `crates/uor-r4-float-train/src/flock_read_hamming.rs`
- `crates/uor-r4-float-train/src/geometric_stack_native.rs`
- `crates/uor-r4-float-train/src/cuda_stack.cu`
- `crates/uor-r4-float-train/src/in_space_long.rs`
- `crates/uor-r4-float-train/src/flock_read_memory.rs`
- `crates/uor-r4-float-train/src/geometric_stack_native_cuda.rs`
- `crates/uor-r4-float-train/src/bin/train-in-space.rs`
- `docs/roadmap/items/integer-nearest-readout.md`
- `docs/harness/prep/integer-nearest-readout*`
- `docs/labs/integer-nearest-readout/**`
- `docs/loop/tasks/integer-nearest-readout/**`
- `docs/research/notebook/m4-serving.md`

## This unit

Q5 chain step 5, under claude-247's card: the serving-audit repair that done-when 2 needs before the replay driver (step 6). On main, the serving audit refuses two `fmov` instructions, both in `TypeHeads::mean_by` (`stack/hamming.rs`, from claude-249's forced reads). They are its two instances:
- `TypeHeads::mean`'s closure;
- `ServedHamming::type_mean`'s forced closure.

The disassembly (`local/claude/claude-254/typeheads-fmov-excerpt.txt`, from claude-254's build) shows LLVM copying the `choose` closure's captured environment with `ldr q0` / `str q0`, then taking its low word with `fmov x8, d0`. No arithmetic is involved: it is a capture copy through a SIMD register, the case session.rs's pool note avoids by giving every closure one pointer.

The fix, in `hamming.rs` only:
- `TypeHeads::mean_by` takes no closure;
- it takes the forced heads as an `Option`, and calls `choose` or `choose_forced` itself;
- the served arithmetic is unchanged.

## Cancel probe (run first)

Run `m5_full_path_cost_controls` with `--features serving-wasm` at base `05021840e`. If it passes, nothing needs repairing and this unit cancels.

**Result, 2 Nov:** it fails, 2 passed and 2 failed (`local/claude/claude-255/probe-base.log`).
- `serving_controls::test_parameter_breakdown_and_phase_timing_on_synthetic_stack` and `serving_controls::test_uor_r4_stack_cost_cli_emits_dynamic_breakdown_and_phase_latency` fail.
- Each names the same two findings: `fmov x8, d0` in `TypeHeads::mean_by::<ServedHamming::type_mean::{closure#0}>` and in `TypeHeads::mean_by::<TypeHeads::mean::{closure#0}>`.
- **Class: the object under test (the served code), not the instrument.** The audit is right that the stack's serving call graph holds a SIMD-to-general transfer, which D11's serving instruction set excludes. The unit goes ahead.

## Done when

1. `m5_full_path_cost_controls --features serving-wasm` passes at the head, all 4 tests, with a negative gate: with the `hamming.rs` fix set aside (diffed against the base), it fails again naming `fmov` in `TypeHeads::mean_by`.
2. `scripts/audit_zero_matmul_serving.py --stack` passes on the head's release `uor-r4-stack`.
3. The type mean is unchanged.
   - `uor-r4-integer`'s suite with `--features serving-wasm` fails no test beyond the base's own failures.
   - The forced-read tests from claude-249 and the hamming tests pass.
   - The base's and the head's release `fbs-toks` decode R's artifact (`a7r-s1.uorlut`) byte for byte alike, teacher-forced on claude-249's DEV s1 smoke prompts, plain and with its forced windows, with their decision traces.
   - **Amended, 2 Nov, during the work** (this text was written after the decodes ran; their script, `local/claude/claude-255/decodes.sh`, was written before), from "claude-254's `--decision-check` and self-grade on F's DEV s1 trace give the same fields". That check reads stored traces and tables and never runs the served session, so it could not see this change. R's artifact has type weights in both reads (256 nonzero each, in F's snapshot), so its decodes run `TypeHeads::mean`, and with `--force` they run `mean_by`'s forced branch.
4. A REPORT with the negatives first, and one notebook row in `docs/research/notebook/m4-serving.md`.

The item stays `ready`: Q5 is not read in this unit.

## Stop and ask if (stop and decide; never wait)

- **The fix needs a file outside the scope above** (the audit test or the audit script). Then the audit itself is the question: classify it, record it, and decide in a card.
- **Removing the closure does not remove the `fmov`.** Then try session.rs's pattern next: one pointer to a record, and a named `#[inline(never)]` function. If LLVM still emits it, record what the disassembly shows, and decide.
- **Any served value changes** (a test or reading in done-when 3 moves). Then the change is not a pure refactor; set it aside and find out why.
