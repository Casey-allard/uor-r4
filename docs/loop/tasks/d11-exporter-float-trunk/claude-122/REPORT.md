# REPORT: d11-exporter-float-trunk, claude-122

Lab: claude. Order: [ORDER.md](ORDER.md). Design note: #2820 comment `6407186781`.

References #2820.

## What did not work, first

- **The converter is not built (done-when item 3 is partial).**
  - The `crates/uor-r4-float-train` side does not exist yet: saving a trained `StackModel` and converting it to an `FbsTrunkSnapshot`.
  - The ORDER's stop rule applies: the work outgrew one PR, so this lands the container, the float reference, the integer session and the controls. The converter follows in the next unit.
  - The converter's route read-back runs `ProjectionPlan::forward`, which is GPU-only, so it pairs with the Stage 4 retrain.
- **No trained model and no export gap yet.** Everything here is finite controls on a hand-built trunk. There is no measured export gap and no tokens per second.
- **I set a bar the reference could not meet.** The plain and quantized float references agree on 224 of 256 single tokens. I had first written a bar of 250, with no basis for it.
  - At vocabulary 256 each cell holds one token, so two kept cells leave two candidates.
  - On this untrained random trunk, the 4-bit grid-coded gains and taps flip near-tied cells.
  - The test is now a floor against gross faults (192). The measured 224 is recorded in the test and the commit.
  - The integer session is held to the quantized reference instead, and matches it exactly (below).
- **The fresh review found weak spots in my tests and two holes in the container rules (fixed in review round 1).**
  - The near-tie exception took the smaller of the token margin and the cell-boundary margin, whatever had differed. A wrong token with agreeing kept cells could have passed on a near-tied cell boundary. It never ran (0 exceptions), so this was a latent flaw, not a hidden failure.
  - The export-refusal test checked only that some error occurred. The kept-cell test would have passed a writer that kept the worst cells.
  - An FBS shape could carry a read selection or read weights. A writer-table value outside Q12 in i16 saturated silently. `rms_eps` was never checked.
  - My REPORT's size line added additions to deletions, and its plain summary claimed "every possible input".
- **The first validation test could not fail.** It asserted only that some shape error occurred: with the projection rule set aside, a later rule still refused the shape. It now asserts each refusal's own reason, and fails with that rule set aside.
- **The zero-multiply audit first failed on the binary.** `stack_native_writer` showed `fmov x9, d0` twice in `uor-r4-stack`, but not in the rlib: the compiler had vectorized its abs-difference sums. `black_box` on each loaded value fixed it, the same way `stack_max_abs` does, and both audits now pass.
- **Four builder dispatches handed back with no code.** The causes, each logged in `local/claude/FRICTION.md`:
  - subagents start near 150k of injected context;
  - a suborder's `## Scope` globs must be backtick-quoted;
  - the guard binds the lowest-numbered free suborder file.

  Slice 1a, part of 1c and all of slice 2 were then written in the main session.
- **A comment I wrote earlier broke a check on main.** The D23 scanner counted a banned word in a comment that claude-119 (#3252) left in `spiralcore_fbs_sheet.rs`. Kimi repaired it in #3259, and this branch merged that. This branch adds no new banned word: the one rewritten doc line in `format.rs` keeps its count at 11.
- **Gaps in this PR's served API:**
  - `step()` on an FBS model returns an empty slice, because the native writer chooses the token itself. The `uor-r4-stack` CLI and the WebAssembly front end do not serve FBS models yet.
  - `measure_step_phases` refuses FBS models.
  - `next_token_scores` is not meaningful for FBS models.

## What changed

Size at the review-round head: code +1,180/−160 (format.rs, session.rs, discrete_d11.rs, mod.rs), tests +517 (tests.rs, tests/memory_shape.rs), docs +174 (ORDER, this REPORT, the notebook row). The ORDER gave no line estimate.

- **The container (`format.rs`).**
  - `StackShape` gains `projection`, `head` and `kept_cells`. They serialize only when set, so existing containers are byte-identical.
  - An FBS shape takes `uor-r4.lut-stack/4`.
  - Validation refuses unlawful FBS shapes, each for its own reason (16 cases in the test). An FBS shape serves recurrence layers only, with no read selection or read weights. The group rule (multiples of 32) gives every route table whole 8-blocks.
  - `FbsRoutes`, `FbsTrunkLayer` and `FbsTrunkSnapshot`, with `export_fbs_trunk_uorlut01`, which is byte-deterministic. It refuses a writer-table value that does not fit Q12 in i16 and an `rms_eps` that is not finite and positive. It writes:
    - one `u32` route entry per output block (`atom | state << 16`);
    - grid-coded gains and conv taps;
    - Q16 biases and decay offsets;
    - the Q12 `writer.table`;
    - no matrices.
- **The float reference (`format.rs`).** `FbsTrunkSnapshot::forward_f32` is the test oracle:
  - every projection is an FBS gather through the integer crate's own Cl(0,6) codes (`cl06_axes`, a new `pub(crate)` accessor in `discrete_d11.rs`);
  - the recurrence and Hamilton math are the dense reference's, moved into shared helpers;
  - the head is the native vector writer;
  - it returns the scored query.
- **The integer session (`session.rs`).**
  - `IntegerStackModel::parse` loads an FBS container into an `FbsModel`. It refuses route entries outside their input or the 64 states, and needs no `exp` table.
  - `stack_fbs_gather` is a gather and a sign. `stack_norm_gain` applies grid codes by shift and add.
  - `stack_native_writer` reads only the kept cells' rows, and its row offsets use `stack_mul_u64`.
  - `stack_step_fbs` reuses the dense recurrence's own core: the conv, lanes and output gate are factored into `stack_recurrence_core` over a `RecurrenceTables` view.
  - `IntegerStackSession::step` dispatches FBS models to it, so the audit's call-graph root covers the new path. `step_native` returns a `NativeStep { token, kept_cells, query_q12 }`.
- **Tests.**
  - The shape rules.
  - Export determinism and parsing, and export refusals, each pinned to its own error.
  - The reference: kept cells only, no left-out cell scoring above a kept one, the gather pinned to `cl06_action`, and the quantized floor.
  - The full-vocabulary control.
  - Parse refusals of corrupt route bytes.

## Measurements

All of this ran on the laptop, with `CARGO_TARGET_DIR=local/target`, on the hand-built trunk `synthetic_fbs_trunk(20261026)`: vocabulary 256, width 32, MLP 32, context 8, 2 layers, 2 kept cells.

| Measurement | Value | Command |
|---|---|---|
| Integer session against the quantized reference | 256 of 256 single tokens and 8 of 8 sequence positions agree on kept cells, token, and the Q12 query within 2 LSBs; 0 near-tie exceptions | `cargo test -p uor-r4-integer --features serving-wasm --lib fbs_trunk_export_matches -- --nocapture` |
| Set-asides | gather sign dropped: fails (query 718 against 5,335); gain dropped: fails (3,831 against 5,335) | same test, each change applied and restored |
| Review-round set-asides | each fails its test: the read-path rule, the Q12 range refusal, the `rms_eps` check, and a reference that keeps the worst cells | `fbs_shape_takes…`, `fbs_trunk_export_refuses…`, `fbs_trunk_reference_reads_only…`, each change applied and restored |
| Plain against quantized float reference | 224 of 256 single tokens agree | `fbs_trunk_reference_quantized_agrees` |
| Zero-multiply audit | `FULL PASS`: rlib 56 symbol ranges and 40 call-graph functions; `uor-r4-stack` 44 symbol ranges and 97 functions; 0 forbidden instructions | `python3 scripts/audit_zero_matmul_serving.py local/target/release/{libuor_r4_integer.rlib,uor-r4-stack} --stack` |
| FBS helpers in the binary | `IntegerStackSession::step` calls `stack_fbs_gather` (two forms), `stack_norm_gain` and `stack_native_writer` directly. None holds a forbidden instruction. `stack_norm_gain` uses no SIMD register. The gathers move data through SIMD registers by plain loads and stores only, which the audit allows | `otool -tvV` and `objdump -d` on `uor-r4-stack` |
| Integer library suite | 313 passed, 0 failed, including every dense-stack test | `cargo test -p uor-r4-integer --features serving-wasm --lib --no-fail-fast` |

## Checks run at the head

See the exact-head review record on the PR:
- the integer library suite and `tests/memory_shape`;
- clippy, with no findings in changed code (the findings at `tests.rs:871` and `:1801`, `kernels.rs:1092` and `:1117`, and the eight in `discrete_d11.rs` are pre-existing);
- `cargo fmt --check`;
- both audits;
- `loop report --check`, `loop pr-check --local` and `loop gates --full`.

## Skills used

- **holding-the-goal.** Each builder hand-back was read for what it learned, and fed to the next brief or done here.
- **writing-meaningful-tests.** Every new test was checked against a set-aside of the code it pins.

## Still open

- **The converter** in `crates/uor-r4-float-train`: save a trained model, read routes back from the GPU kernel by a coded-carrier forward, check the codes against the integer crate's `AXES`, and convert to `FbsTrunkSnapshot`. It needs a GPU test.
- **Stage 4:** a new one-page sheet with a fresh non-author FIT that names the exported arm; a two-seed retrain with saving under a D67 pre-run review; a GPU evaluator of the container or a bit-exact parity proof; laptop tokens per second for decode and context.
- **Serving front ends:** the `uor-r4-stack` CLI and the WebAssembly front end need a native-head path for FBS models.
- **The `docs/mechanisms/lut-serving.md` row,** written by an infra-loop-harness unit once the umbrella is free (claude-121's scope note).

## Plain summary

The project's float-trained models use SpiralCore/FBS projections: each output block takes one input block and flips and reorders its numbers, with no weights at all. This PR teaches the integer serving engine to load and run such a model:
- a new file format version that records each block's choice;
- the integer code that runs those choices as a lookup and a sign;
- the model's own head, which picks the next token from a short list of candidates rather than scoring every word.

On a small hand-built model, the integer engine picks exactly the same next token as a floating-point reference that rounds its tables the same way. It did so for every single-token input and for each position of one 8-token sequence. A check of the compiled code finds no multiply, divide or floating-point instruction in the served path. What is not done yet is the piece that takes a real trained model from the training code into this format, and the measurement on a trained model.
