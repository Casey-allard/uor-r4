# REPORT: one-model-end-to-end, claude-389 (the native Q31 first-pick read operator)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [ORDER.md](ORDER.md). PR #4001. Base `origin/main` at `e5a32f458`, which holds claude-388 (#3997). The runs used head `d1ea1039b`.

**Reading map letter: P.** `q31_first_pick_read` is an integer-only operator. On claude-386's saved Flock-g states, it reproduces the float read's definition, both the choice and the write:

| State | Picks equal to the f64 reference | Worst out-word miss (×1, bound 3 LSB) | Clipped words |
|---|---|---|---|
| Seed 1, 16,384 steps (barred) | **1,024 / 1,024** | 1.02 LSB | 0 |
| Seed 2, 16,384 steps (barred) | **1,024 / 1,024** | 1.02 LSB | 0 |
| Seed 1, 4,096 steps (reported) | 1,024 / 1,024 | 1.16 LSB | 0 |
| Seed 2, 4,096 steps (reported) | 1,024 / 1,024 | 1.10 LSB | 0 |

The bar was at least 1,014 of 1,024 picks, every out word within its bound, and no clip, at both barred states. Every condition holds, with no pick mismatch at all. All four controls and the structural check pass, so neither D nor N2 applies. The trained gain shift is `g = 0` at every state.

The CUDA twin agrees with the Rust operator bit for bit on the first barred state's real converted read: 49,152 rows and 12,582,912 out words, with 0 picks and 0 words differing.

So D85's native read exists as one typed operator with a device twin, and its picks agree with the f32 read claude-388 measured on every row. The `R_ask` claude-388 found therefore carries over unchanged: at ×1 its hard copy misses the bar, and at ×16 it passes.

## What did not work, first

1. **This is forward only.** The operator is a pure function. It has no gradient and no update for its fields, so by the native-space SKILL this is an **operator unit, not a training run**. The read is not yet trained in the native space.
2. **No session calls it.** `IntegerStackSession` and the lut-stack artifact are unchanged (the stop list). The operator runs on float states converted by a reference-code diagnostic. Nothing it makes is served.
3. **The trained gain is still too small to pass the bar on its own.** Every state converts at `g = 0`, the gain the float read learned. At that gain claude-388's hard copy gives `R_ask` 0.631 / 0.643 and 0.096 / 0.145, below the bar. The ×16 out-word check at `g = 4` passes too (worst 2.6 to 5.1 LSB, bound 18), so a fixed 16-fold gain works in Q31. But no trained parameter supplies that gain yet.
4. **There was no near-tie to test the `2B` allowance.** Every state has 0 rows whose f64 top two scores lie within `2B` (256 LSB). So the run never tested the rounding allowance at a tie; the fixtures do instead (the tie rule and the 1-LSB control).
5. **The first wick job failed on the kit, not the code.** Job c389 stopped with rc=3 before any test ran. `uor-r4-discrete` reads `docs/labs/rung0-pilot-release-2026-10-14/PILOT_PLAN.json` with `include_str!`, and the kit's `sync` copies no `docs/`. I copied the tracked `docs/` files into the container (`git ls-files -z -- docs | tar`) and re-ran the same script as job c389b, which passed.
6. **The laptop cannot type-check the float crate's cuda code.** The stub nvcc compiles `uor-r4-discrete`. It fails on `candle-kernels`, whose macro-made entries (`affine_*`) fail the D29 export admission. So the capture's cuda code was first compiled on wick.

## What changed

- **`crates/uor-r4-native-vector/src/read.rs`** (new): `q31_first_pick_read` and `Q31FirstPickRead`, as the ORDER describes them:
  - address: one Hamilton rotation per block, by the query gate and by the key gate, then the 16-group fold in exact i64;
  - score: the L1 distance in u64 over earlier rows, the least one winning and a tie going to the latest row;
  - payload: `p = sat((w·e + r) >> (31 − g))`, then `out = sat(x + p)`, with `r = 2^(30−g)`, and 0 at `g = 31`;
  - refusals: a word outside `-2^31..=2^31`, a width that is not a multiple of 16, mismatched lengths, an id outside the table, or `g > 31`.

  It has fixture tests for the tie rule, the fold, saturation, the gain and every refusal. The fields serialize as raw little-endian words with a version byte, and a round trip is tested.
- **`crates/uor-r4-native-vector/src/read_dump.rs`** (new, outside the ORDER's file list but inside its crate): `Q31ReadDump` holds one converted read, its windows, ids and table, written as `UORQ31RD` little-endian bytes. It exists so control 2 can run the CUDA twin on a barred state's real input. It has 2 tests.
- **The CUDA twin:**
  - `native_vector.cu` has two integer-only kernels, `native_q31_read_address` and `native_q31_first_pick_read`, with one thread per row;
  - `build.rs` admits 4 `NATIVE_VECTOR_ENTRIES`;
  - `NativeVectorEngine` has `first_pick_read` and `first_pick_read_reference`;
  - `tests/native_vector_cuda.rs` has 3 tests: a seeded fixture; ties, saturation and refusals; and the 1-LSB key-gate control (control 3).
- **`crates/uor-r4-discrete/examples/first_pick_read_parity.rs`** (new, outside the ORDER's file list): control 2. It loads a dump, runs the CPU operator and the CUDA twin, prints one JSON line, and exits non-zero on any difference.
- **`crates/uor-r4-float-train`:**
  - `vector_reads.rs`: `capturing_replay(f)` holds a thread-local copy of each Flock forward's inputs. A cuda test checks that the forward stays bit for bit with the capture on.
  - `native_replay.rs` (new): the conversion rule, the f64 reference, the bar, controls 1 and 4, the structural check and the report lines. It has 7 tests, among them `t2_swapped_reference_gates_fail_control_4_and_the_bar`, the negative that a wrong capture fails.
  - `in_space_long.rs`: `native_replay_pass`, one extra hard-copy pass at scale 1.
  - `recall-payload-probe` gains two flags:
    - `--native-replay`, which needs `--first-only`;
    - `--native-dump PATH`, which needs `--native-replay` and refuses to write over a file.

    Both are tested.
  - `Cargo.toml` gains the dependency-free `uor-r4-native-vector`.
- **Float training and the float read's default forward are unchanged.** The capture is off by default and only copies inputs.

## Measurements

The job, c389b, ran on wick (RTX 4090, `sm_89`), with the tree synced at `d1ea1039b`. Every state has width 256, no refusal, 256 held-out windows of 192 rows, and 1,024 query rows (512 per far bucket).

**Conversion, as recorded in each JSON** (`e_x`, `e_t`, `e_w` are the log2 scales):

| State | `e_x` / `e_t` / `e_w` | `g` | `max|x|` | `max|w|` |
|---|---|---|---|---|
| ext-s1 | 25 / 32 / 24 | 0 | 30.60 | 6.21 |
| ext-s2 | 24 / 32 / 23 | 0 | 35.06 | 5.88 |
| arm-s1 | 24 / 33 / 22 | 0 | 36.20 | 0.61 |
| arm-s2 | 25 / 33 / 23 | 0 | 25.99 | 0.64 |

**The bar and the checks.** Each state is checked on 262,144 out words.

| State | Picks = ref | Mismatches > `2B` | Near ties | Worst out word ×1 (bound 3) | Worst out word ×16 (bound 18) | Clips: in / lane / out | Structural |
|---|---|---|---|---|---|---|---|
| ext-s1 | 1,024 | 0 | 0 | 1.021 | 2.613 | 0 / 0 / 0 | 256 / 256 |
| ext-s2 | 1,024 | 0 | 0 | 1.020 | 2.590 | 0 / 0 / 0 | 256 / 256 |
| arm-s1 | 1,024 | 0 | 0 | 1.156 | 4.917 | 0 / 0 / 0 | 256 / 256 |
| arm-s2 | 1,024 | 0 | 0 | 1.097 | 5.071 | 0 / 0 / 0 | 256 / 256 |

**Controls:**

| State | Control 1 (key gates replaced by `[2^30, 2^30, 2^30, 2^30]`): mismatches > `2B` / agree with unsubstituted | Control 4: f32-vs-f64 pick disagreements / f32 out words over bound | `B_f` |
|---|---|---|---|
| ext-s1 | 0 / **4** of 1,024 | 0 / 0 | 0.00747 |
| ext-s2 | 0 / **51** | 0 / 0 | 0.00856 |
| arm-s1 | 0 / **45** | 0 / 0 | 0.00884 |
| arm-s2 | 0 / **6** | 0 / 0 | 0.00635 |

- **Control 1** passes (a) and (b) at every state. The rotation moves the native picks on 95% to 99.6% of rows, and native and reference still agree under it. So the replay can fail, and it follows the definition rather than copying a pick.
- **Control 2**, CPU against CUDA on `dump-ext-s1.q31`:
  - batch 256, time 192, 49,152 rows and 12,582,912 out words;
  - `picks_differ` 0, `out_words_differ` 0, `bit_identical` true;
  - CPU 93.7 ms, CUDA 58.1 ms, as one-shot wall times including the copies.
- **Control 3**, the 1-LSB key-gate change in the near-tie fixture, changes a pick: `cuda_first_pick_read_tracks_a_one_lsb_key_gate_change` passed.
- **Control 4** passes at every state: the capture is the read that ran.

**Report lines (no bar):**
- Native picks agree with the f32 first picks claude-388 read on 1,024 of 1,024 rows at every state; f32 and f64 disagree on 0.
- So the native first-right counts equal claude-388's f32 ones: 1,000 and 990 at the barred states, 418 and 474 at the 4,096-step states.
- The scale-1 hard-copy `R_ask` in the same JSONs repeats claude-388's ×1 numbers: 0.631 / 0.643 (ext-s1), 0.096 / 0.145 (ext-s2), and 0.002 / 0.000 at the 4,096-step states.

**Results.** These are in the HF store at `caseyallard/uor-r4-store:results/claude/native-q31-read-389-2026-11-15/`: the four replay JSONs, control 2's output, `SHA256SUMS` (sha256 `9643f22b…5e4fb72d`), and `dump-ext-s1.q31`, the input control 2 ran on (104 MB). The other three dumps stay on wick under `/workspace/out/c389/`, with their sha256 recorded in `SHA256SUMS`. Every file was checked on the laptop with `sha256sum -c`.

| File | sha256 |
|---|---|
| `replay-ext-s1.json` | `e3361f47…4c76fbc8` |
| `replay-ext-s2.json` | `0998381c…557f3210` |
| `replay-arm-s1.json` | `8c0c7aec…ca0d1f31` |
| `replay-arm-s2.json` | `a074da16…4afa3382` |
| `control-2-ext-s1.json` | `957ff628…3e81cd00` (the JSON line between the job's two `== control-2` log lines) |
| `dump-ext-s1.q31` | `2e28e594…a4ed7a32` |

The four states' `model.safetensors` and `vector_read.safetensors` sha256 equal the ORDER's.

## Checks run at the head

- **Laptop (`aarch64-apple-darwin`), at `d1ea1039b`:**
  - `cargo test -p uor-r4-native-vector`: 23 passed.
  - `cargo build -p uor-r4-native-vector --target wasm32-unknown-unknown`: built.
  - `cargo clippy -p uor-r4-native-vector`: 0 findings.
  - `cargo test -p uor-r4-float-train --lib native_replay`: 7 passed.
  - `cargo test -p uor-r4-float-train --bin recall-payload-probe`: 3 passed.
  - `rustfmt --edition 2021 --check` on the four new files: clean.
- **wick (`sm_89`), job c389b, the tree synced at `d1ea1039b`:**
  - `cargo +1.97.1 test --release --locked -p uor-r4-native-vector`: 23 passed.
  - `… -p uor-r4-discrete --features cuda --test native_vector_cuda -- --test-threads=1`: 7 passed, among them the three read tests and control 3.
  - `… -p uor-r4-float-train --features cuda --lib vector_read`: 27 passed, among them `the_replay_capture_holds_the_forward_and_leaves_it_bit_for_bit`.
  - `… --lib native_replay`: 7 passed. `… --bin recall-payload-probe`: 3 passed.
  - Binary sha256s: `recall-payload-probe` `57abc48c0ba1cdf117e7bcc0b1c794c5853049015eb3c7d496dfc0098169c6f7`, and `first_pick_read_parity` `c77ac616e16c412dc8e9771318875391a027375e1b5c95fcad9d02fbe8e085af`.
- **The sheet's FIT:** four fresh-context non-author reviews. The fourth returned `TEST FITNESS: FIT` at `9d9b960ce`, and its minor points were folded in at `f86ca898f`, before the job.
- **The code review, at `481911260`:** four fresh-context reviews, one per area: the operator, the CUDA twin, the replay's bar logic, and the capture with the flags. Each returned APPROVE with minor findings only. The follow-up commit after `481911260` folds in three of them:
  - A negative gate showed that the replay tests passed with the f64 reference's tie flipped to the earliest row. The cause: the test helper fills the f32 fields from the same reference. The new test, `the_reference_sends_an_exact_tie_to_the_latest_row`, asserts the reference's picks directly. It fails with the tie flipped (7 passed, 1 failed) and passes as written.
  - `--native-dump` and `--out` spelled the same way are now refused. The dump is opened with `create_new`, so it never writes over a file, including one that appears during the run. A different spelling of the same file (`./o`, an absolute path, a symlink) still gets past the check: `--out` is written last and would replace the dump.
  - `Q31ReadDump`'s size products use `checked_mul`. A header with `batch = time = u32::MAX` now returns an overflow error. On 64-bit the error comes from `rows × width`; `batch × time` alone still fits. Without the check, the extended `a_damaged_dump_is_refused` panics.

  None of the three changes the operator, the kernels or the replay's arithmetic on valid input, so wick's results at `d1ea1039b` stand. Laptop, after the commit: native-vector 23 passed, native_replay 8 passed, the bin's 3 passed, the wasm32 build is clean, clippy has 0 findings, and rustfmt on the new files is clean.

## Skills used

- `native-space-cycle`: the five items are in the ORDER. The state schema, the shared operator path (one Rust definition, one CUDA twin, a bit-identical differential), the replay oracle and serialization are met for the read's forward. Forward, gradient and update are not: this is forward only.
- The loop's ORDER/REPORT form, with a one-page sheet and four FIT rounds.
- The wick kit (`local/claude/wick/wick.sh`: sync, job, wait, get), with a completion watcher armed.
- `uor-store` for the results.

## Still open

1. **Attach the operator to `IntegerStackSession`.** It is the session unit, and the read's fields go into the lut-stack artifact beside the stack's state. Its first check is this run's picks, reproduced from the artifact.
2. **The native update rule.** The read's gates and weights need a native gradient and update on the same Q31 words: the training half of D85's one space. That includes the gain. `g = 0` is what the float read learned, and the bar needs about ×16. Either a fixed `g = 4` or a trained gain under a target size; the update-rule unit decides.
3. **The wick kit's `sync` skips `docs/`.** `uor-r4-discrete` and others `include_str!` files under `docs/`, so any job building them fails until the tracked `docs/` files are copied in. That fix belongs to the kit (`local/claude/wick/wick.sh`).
4. **No test catches a fold that permutes query and key alike.** Such a permutation leaves every L1 distance unchanged. During the build, a sabotage gate that changed the fold to `(c+1) % 16` on both sides still passed; only a collapse to `c % 4` failed. A test that pins the fold's order would compare the address words themselves against a fixture.
5. **The stub nvcc cannot type-check candle's cuda code** (`affine_*` against the D29 admission). So a float-train cuda change compiles first on wick.
6. **The D41 pilot** (`d41-driver-math-pilot`) runs in its own session on wick.
7. **The review's minor findings, not folded in.** Each would change a CUDA-only file, or the operator, after the wick run, so they belong with the session unit's next device run:
   - **The id refusal is narrower than the ORDER says.** The operator refuses an out-of-table id only where a pick reads it. For example, `ids = [0, 1, 99]` passes when row 2 picks row 0. CPU and CUDA agree on this rule, and a test asserts it. The session unit either checks ids `1..time` of every window up front or states the narrower rule.
   - `first_pick_read_parity` reports `bit_identical: true` on a dump with no row past row 0. It should refuse `rows == 0` or `time < 2`. This run's dump had 49,152 rows.
   - The device tests use gain shifts 0 and 4 only, so the `g = 31` path (round 0, shift 0) does not run on the GPU.
   - No test pins the window offset on the keys (`keys[first + s]`) with a window-1 row whose correct pick differs from window 0's.
   - The probe writes the dump before the cut pass. An error after the dump leaves the dump on disk without its JSON.
   - Pre-existing in `q31_hamilton_rotate` (`lib.rs:349-373`): the sum wraps in u64 before the clamp. Admitted non-unit gates such as `[2^31, 2^31, 2^31, 2^31]` can therefore wrap instead of saturating. The read inherits this; the replay's unit gates and headroom rule stay clear of it.

## Plain summary

The model's look-back read now exists as integer code, the form the project trains and serves in. It also has a GPU twin. On the four saved models it chose the same earlier position as the float read on every one of 1,024 test rows. The numbers it wrote back were within about one unit in the last place, and nothing overflowed. The GPU twin gave exactly the same bits as the CPU code on a real model's full input. So the read that recalled the asked fact 96.5% to 98.4% of the time in claude-388 now works in integers. Two steps remain: plug it into the integer serving session, and train its settings in integers, including the 16-fold gain the copy needs.
