# REPORT: d11-exporter-float-trunk, claude-123

Lab: claude. Order: [ORDER.md](ORDER.md). Pre-run review (D67): #2820 comment `6408456837`, PRE-RUN: GO.

References #2820.

## What did not work, first

- **The float GPU trunk and the converted trunk disagree on most tokens, and this unit does not say why.**
  - On a random-init trunk (vocabulary 256, width 32, MLP 32, 2 layers), the float GPU model picks the converted reference's token on 67 of 256 single tokens. The largest gap between the GPU's final hidden vector and the reference's query is 2.825.
  - Part of this is probably the known export gap, though no run here measured how much. The integer path, and so the reference, snaps the Hamilton gates and the recurrence rotations to the 120 icosians, while the plain float GPU path keeps them continuous.
  - This run cannot separate that gap from a fault in how the converter maps the copied tables. Separating the two is Stage 4's parity proof against the sheets' export mode, so it is now the first thing Stage 4 owes.
  - What is proven exactly is the routes (below). The other tables were checked by reading the float forward, not by a test:
    - the embedding is a plain row lookup, with no scale;
    - the convolution weights are `[tap][channel]`, with tap 0 on the current position;
    - the gate bias holds the decay gates, then the rotations;
    - `rec.in` holds the drive, then the output gate;
    - the decay offset is added before the clamp;
    - the RMS epsilon is `1e-5`, the CUDA kernel's `RMS_EPSILON`.
- **A regression of mine reached main.** claude-122 (#3255) left `stack_fbs_gather` loading through SIMD registers.
  - The crate's own step-disassembly verifier refuses those registers. So `tests/m5_full_path_cost_controls` (2 of 3 tests) and `uor-r4-stack cost` have failed on main since `611cfb05`.
  - The Python audit allows plain SIMD loads, and CI does not run that integration test, so neither caught it. I found it here, in this unit's full integer suite.
  - It is fixed in this PR: `black_box` on each stored lane, the pattern `kernels.rs` already uses. My first attempt, `black_box` on each load, failed the verifier, because the compiler spilled each value and reloaded it into an `h` register.
  - Noted on #2028 (comment `6408584391`).
- **The design changed after the ORDER was written.** The integer crate's `stack` module compiles only under `serving-wasm`, and that feature also compiles `model`, `sampling` and `tables`. D29's presence rule rests on the feature staying off in the native graph. So:
  - The snapshot types moved to an ungated `fbs_snapshot` module.
  - The converter writes JSON. A second bin, in the serving build, exports that JSON to the artifact.
  - The new path dependency adds one line to `Cargo.lock`. The item's scope omits that file, so pr-check's scope rule (advisory) WARNs on it.
- **My first boundary still let the serving build in, and the fresh review caught it (round 1).**
  - I had given float-train a test-only `serving-wasm` dev-dependency. Resolver 2 keeps that out of normal builds. It does not keep it out of workspace-wide runs that build float-train's tests: CI's `cargo check --workspace --all-targets` and `cargo test --workspace --lib` would then have compiled the serving build for every crate in the run. They would also have stopped testing the integer crate's no-feature configuration.
  - The dev-dependency is gone. The GPU test now proves the routes and writes `snapshot.json` and `readout.json`. A serving-build bin, `fbs-compare`, exports, parses and serves them, and prints the reading.
  - `cargo tree --workspace -e features -i uor-r4-integer` now shows only `default`.
- **The other round-1 findings, all fixed:**
  - a sheet arm left set on the thread would have been exported as plain routes, with no record;
  - a table that is not finite would have reached the JSON as `null`;
  - a failed write could leave a partial output file that blocked the retry;
  - the GPU compare did not check output length;
  - one sample did not prove that `f32` values survive JSON exactly.
- **Two of my own checks were wrong at first.**
  - A "scaled value" refusal case halved a coded value. That can land on another atom's slot, which the decoder correctly reports as two atoms. It is now a fractional value.
  - My first set-aside of the two-atom check did not compile, so it proved nothing. A compiling set-aside then failed the test as required.
- **Failures in the integer crate's full suite that are not this PR's:**
  - In a run at this branch before the gather repair, 38 tests failed: my 2 step-audit tests, then 35 that need fixture files a clean clone lacks (bundles and `tokenizer.json`), and one more.
  - That one, `m7_parity_library_cli_browser`, fails because `wasm-pack` calls a bare `cargo`, which Homebrew's `cargo` shadows on this Mac.
- **Pages not updated.** `docs/model/serving-and-export.md` and `docs/mechanisms/lut-serving.md` should name the converter and the two bins. Both are outside this item's scope, and `lut-serving.md` waits for the umbrella (claude-121's note).

## What changed

Size against `origin/main` at `398edb88`:
- code +661/−69: the code parts of `fbs_export.rs` (lines 1–315) and `fbs_snapshot.rs` (lines 1–127), the three bins, the other code files and `Cargo.lock`;
- tests +409: the test modules of `fbs_export.rs` (322 lines) and `fbs_snapshot.rs` (64), and `stack/tests.rs` (23);
- docs: the ORDER, this REPORT and the notebook row (+181 at the code head; the REPORT grew after it, with review notes only).

pr-check counts whole files, so it reports the in-file test modules as code. The ORDER gave no line estimate.

- **`uor-r4-integer`:**
  - `fbs_snapshot` (ungated) holds `FbsRoutes`, `FbsTrunkLayer` and `FbsTrunkSnapshot`, plus the float readout types `FbsFloatReadout` and `FbsFloatPosition`. Their serde refuses unknown fields, and `stack::format` re-exports the snapshot types.
  - `fbs_snapshot` also has `write_new_file`. It writes through `<path>.partial` and a hard link, so it never replaces a file or leaves a partial one.
  - `discrete_d11` adds `spiralcore_signed_axes()` and `spiralcore_orbit_roots()`, both read-only.
  - Two new serving-build bins:
    - `fbs-export SNAPSHOT_JSON OUT_PATH` parses the artifact back before writing it, and prints the sha256.
    - `fbs-compare SNAPSHOT_JSON READOUT_JSON` exports, parses and serves every readout position, and prints one JSON line of counts. It is the instrument for Stage 4's comparison.
  - The `stack_fbs_gather` repair.
- **`uor-r4-float-train`, `fbs_export`:**
  - `fbs_trunk_snapshot(&StackModel)` reads each projection's routes back through the model's own GPU projection, `linear`, now `pub(crate)`. It feeds one coded carrier, where slot `i` holds `i + 1`.
  - It refuses a thread with a sheet arm set, and refuses any copied table with a value that is not finite.
  - `decode_coded_routes` refuses any block that is not a one-atom signed gather with a state in the integer Cl(0,6) table.
  - `check_geometry_tables` holds the float geometry's 64×8 signed axes and 16 orbit roots to the integer crate's, in order.
  - `check_fbs_export_config` refuses each unservable configuration for its own reason.
  - `fbs_float_readout` records the float GPU trunk's hidden vector and next token for every single token.
  - A new GPU bin, `fbs-snapshot MODEL_DIR OUT_JSON [READOUT_JSON]`.
  - `uor-r4-integer` is a dependency without features, and never `serving-wasm`.

## Measurements

| Measurement | Value | Where |
|---|---|---|
| Route read-back on the GPU | bit-identical to the model's projection on 3 random carriers, for all 6 projections in both layers, output lengths checked | `gpu_converted_trunk_reproduces_the_projection`, RTX 5090, at the code head `398edb88` |
| GPU set-asides | the GPU test fails at its bit-equality assert (`fbs_export.rs:603`) with the decode's sign bit dropped, and with a valid but wrong state | same pod, `tests-setaside-{1,2}.log` |
| Sheet arm | conversion is refused while an arm is set | same test |
| Export, parse, serve | `fbs-compare` and `fbs-export` exit 0. The integer session matches the quantized reference's token on 256 of 256. Artifact sha256 `0621616ab3d6a9786be8e1e82569d6bf6539a854fbe8775cd5c47d6622c9447a` | `compare.json`, `out/trunk.uorlut` |
| Float GPU against the converted reference (a reading, no bar) | 67 of 256 tokens agree; largest hidden gap 2.825. The same at `d93b4977` and at `398edb88` | `compare.json` |
| Tables identical | the float signed axes and roots equal the integer crate's | `geometry_tables_match_the_integer_crate` (host) |
| Host converter tests | 5 pass. Set aside the root check, the two-atom check, the state lookup or the U(1) refusal, and each fails its test | `cargo test -p uor-r4-float-train --lib fbs_export` |
| Snapshot JSON | round-trips bit-exactly and exports to the same bytes; an unknown field is refused. 200,000 random finite `f32` values plus edge values (signed zeros, subnormals, extremes) keep their bits through JSON | `fbs_trunk_snapshot_json_round_trips_exactly`, `json_f32_round_trips_bit_exactly` |
| Feature boundary | `serving-wasm` appears 0 times in `cargo tree --workspace -e features -i uor-r4-integer` | host |
| Step-audit repair | `m5_full_path_cost_controls` passes 3 of 3; with the fix set aside, 1 of 3 | integration test, host |
| Zero-multiply audit | `FULL PASS`: rlib 56 symbol ranges and 41 call-graph functions; `uor-r4-stack` 44 and 90 | `scripts/audit_zero_matmul_serving.py --stack` |
| Suites | float-train 31 passed; integer serving-build library 316; integer no-feature library 54; `memory_shape` 3 | host |
| GPU cost | two RTX 5090 pods, about 5 minutes each, at $1.09/h | Vast `vast-57766451` and `vast-57768703`, both deleted |

The GPU evidence is in the HF store under `results/claude/d11-fbs-converter-2026-10-26/`, with a `SHA256SUMS` for each run. The first run, at `d93b4977`, is at the folder's top level. The run at the code head is under `final-398edb88/`.

## Checks run at the head

See the exact-head review record on the PR:
- the converter's host tests, and the float-train suite;
- the integer serving-build library, and `m5_full_path_cost_controls`;
- the stub-nvcc `cargo check --features cuda --all-targets`;
- the GPU test at the final head;
- clippy (no findings in new files; the eight in `discrete_d11.rs` are pre-existing);
- `cargo fmt --check`;
- both audits;
- `loop report --check`, `loop pr-check --local` and `loop gates --full`.

## Skills used

- **holding-the-goal.** When the feature boundary appeared, I re-planned the unit around it rather than enabling `serving-wasm`.
- **writing-meaningful-tests.** Every new test was checked against a set-aside of the code it pins, including the GPU test, with two set-asides.

## Still open (Stage 4 and after)

- **The parity proof,** first: the float GPU in the sheets' D11 export mode against the converted trunk, to split the export gap from any mapping fault in the copied tables.
- **The Stage 4 sheet:** a fresh non-author FIT; a two-seed retrain saved with `StackModel::save`; the export gap measured on the trained trunk; laptop tok/s, with decode and context reported separately.
- **`fbs-compare` should refuse a readout that does not cover every token exactly once.** Today an empty or repeated readout still exits 0 (round-2 review, low). The GPU run's readout covered all 256 tokens once, so its counts stand. Fix it before Stage 4 relies on the tool.
- **Serving front ends:** native-head paths in the `uor-r4-stack` CLI and the WebAssembly front end.
- **The pages:** `serving-and-export.md` and the `lut-serving.md` row.

## Plain summary

This unit builds the bridge from a float-trained SpiralCore/FBS model to the integer serving file.
- The tricky part is each projection's routing choice, which the GPU makes. The converter reads it back by pushing one numbered input through the GPU and reading which numbers come out where, so nothing is guessed. On a GPU this reproduced every projection exactly.
- The converted model exports to the integer file, loads, and runs.
- What is not yet known is why the plain float model and the converted one often pick different next tokens on an untrained model. Part of the reason is that the integer version rounds its rotations to a fixed set. The next unit has to measure how much.
- This unit also fixes a code-generation slip of mine from the previous unit, which had broken one serving test on main.
