# REPORT: one-model-end-to-end, claude-278 (P3, cache unit 1: the trunk's exact step cache)

Lab: claude. [ORDER](./ORDER.md). PR #3672, item #3407. Plan step: Casey's plan of 4 November, P3 (faster scoring), cache unit 1 of 2. Base `14a47d998`. The gates ran at the code commit `2d34cba131d592c0d92bf34a126b25690adc2531` on one RTX 5090: Vast `vast-59133839`, Czechia, driver 595.99.02. The pod cost about $0.23.

**In one paragraph.** The GPU float model's trunk now has a one-position step with carried state, and on the GPU it equals today's whole-window forward bit for bit.
- **The compare:** 48 cases of 12 steps each. Every step's points and every layer's carried state (the scan state and the convolution's last three inputs) equal the whole window's.
- **The window edge** is exact, and the step past it is refused.
- **The negatives** are caught: one flipped state bit, or one zeroed input row.
- **The kernel:** the scan's step runs the existing CUDA scan kernel, unchanged. No kernel was added.
- **The suite:** the crate's whole CUDA suite passes, 336 passed and 8 ignored.
- **Nothing calls the step yet.** Unit 2 adds the Hamming read's step and switches the scorer's decode loops to it.

## Tokens per second first

**None measured.** This unit builds and proves the step, and no decode loop calls it yet, so it has no tok/s of its own.
- Today's figure, which the cache is built to beat: a query decodes at 15.2 tok/s at a 2,048 window, where every decode step re-runs the whole window, and decode is 99% of a scoring pass ([claude-277](../claude-277/REPORT.md)).
- Unit 2 switches the scorer to the step where the window fits and measures decode tok/s on the same files.

## What did not work

1. **The first pod run's negatives test (G3) failed,** at `887274c24`. Flipping the lowest bit of one value of layer 0's carried scan state left all three steps' points bit-equal to the clean run's.
   - **Classified as instrument.** That run's compare looked at the points only, so it could not see the state itself.
   - **The likely cause (a hypothesis, not measured):** the change rounds away before the points. A 1-ulp change in one state component enters the next state through `T ⊗ held`, scaled by components of `T`, and mixes with the other lanes later, through the output projection.
     - Component 0's value was not recorded. If it was zero or subnormal, a flush to zero downstream would erase the flip just as well.
     - Rerunning the old flip under the new carry compare would settle it (Still open, item 2).
   - **Not a fault in the cache:** the same run's step-equals-full gate (G1) passed in all 48 cases.
   - **The fix,** `2d34cba13`, has two parts:
     - the compare now also checks every layer's carries against the ones a prefill of that whole window captures, bit for bit;
     - G3 flips the top mantissa bit (bit 22) of the largest value in layer 0's carried state.
   - **The rerun's G3 passes.** Run 2 does not show which of the two parts made it pass, because the test adds point and carry differences into one count. The lowest-bit flip is no longer a test case: a flip that small can round away within one step, so its being unseen would say nothing about the compare.
2. **The first run's G1 proved the points equal, not the carried state.** Equal points do not imply an equal state, for the reason in item 1. The rerun's G1 proves both.
3. **Refused, so not tested** (each is a typed refusal, never a silent second path):
   - bf16 storage;
   - a step that would slide the window (unit 2 re-runs the whole window there);
   - every read hook but turn memory: the flock, content, Hamming, opening and prompt-register reads. The Hamming read's step is unit 2;
   - a turn-memory opening row;
   - the rate instruments, gate telemetry, a D11 snap capture or replay, an FBS projection arm;
   - an id outside the vocabulary;
   - a prompt shorter than the convolution's width (4) or longer than the window.
4. **Not refused, by review:** a step counts turn-memory reply rows in its own rows (row `b` is batch row `b`). A caller who passes whole-window numbers (`b * time + t`) gets either an "outside the rows" error or the reply on the wrong row. The doc comment says so; unit 2's scorer must name step rows.
5. **The retention reading is one setting for the whole process.** G1 switches it, so the CUDA tests must run one at a time; the pod job runs them with `--test-threads=1`. Under a default multi-threaded `cargo test --features cuda`, G1 can race with other tests that read it. The pattern predates this unit (`in_space_long.rs`'s tests switch it too), and it is not fixed here.
6. **The −0 corner is pinned only at its entry.** The CPU test checks the entry rule: a carried −0 enters the kernel as +0. It does not test what that does at the second position, where it can change a bit only when every term of a component of the Hamilton product is zero and the drive component beside it is exactly −0. G1 saw no difference in 48 random cases.

## What worked

- **G1, step equals full,** in all 48 cases. The cases cross:
  - retention on and off;
  - the served surrogate contract on and off;
  - the turn memory's reply table on and off;
  - prompts of 4, 7 and 33 positions;
  - batch 1 and batch 3.

  Each case takes 12 steps on a random 6-layer model, at width 256. At every step, the step's points are compared with the whole-window forward's at that position, as u32 bits, for every row. Every layer's carries are compared with a whole-window prefill's, also as bits.
- **G2, the window edge:** at a context of 16, a prompt of 4 steps up to exactly 16 of 16 positions, bit-equal. The next step is refused, and it leaves the cache unchanged.
- **G3, the negatives:** a flipped top mantissa bit in a carried scan state, and a zeroed newest convolution input, each make the compare fail.
- **The two-position scan through the CUDA kernel** equals the whole scan's next position, bit for bit, on random transitions and drives (batch 3, 9 positions, 5 lanes, at every position 1 to 8).
- **The CPU twin's identity test** and **the refusal test** pass, on the laptop and on the pod.
- **Training's arithmetic is unchanged.** The whole-window path runs the same ops in the same order; two fresh reviews checked this. The existing golden CUDA tests pass in both runs.
- **The reason the step reuses the scan kernel is visible in its PTX.** `quaternion_scan_fwd` compiles to 18 fused `fma.rn.f32` and 60 other f32 multiplies and adds. nvcc fuses multiply-adds, so a step kernel compiled on its own could fuse differently and break bit equality.

## What changed

- **`crates/uor-r4-float-train/src/geometric_stack_native.rs`** (+772 −17 against the base):
  - `DecodeCache` and `LayerCarry`: per layer, the scan state at f32 and the convolution's last three inputs. Their format mirrors the integer session's `LayerState`.
  - `CacheRefusal`, a typed refusal for each case in What did not work, item 3.
  - `prefill_cached`: the unchanged whole-window forward, which fills the cache from its own tensors.
  - `step_cached`: one position per row. It commits the new carries only after the whole step succeeds.
  - `quaternion_scan_step`: the existing scan kernel run over two positions.
  - `hidden` now calls `trunk`, which threads an optional carry through `composed_recurrence`.
  - Six tests in `decode_cache_tests`.
- **`crates/uor-r4-float-train/src/hamilton_sheet.rs`** (+7): `is_instrumenting_snaps`, so the step can refuse a D11 snap capture or replay.
- **No kernel changed.** The 26-kernel extract, D29's compiled-module boundary and its pins stay as they were.

## Measurements

| check | run 1, at `887274c24` | run 2, at `2d34cba13` |
|---|---|---|
| decode-cache tests | 5 passed, 1 failed (G3); 38.77 s | 6 passed; 38.50 s |
| the crate's CUDA lib suite, one thread | 335 passed, 1 failed (G3), 8 ignored; 83.33 s | 336 passed, 0 failed, 8 ignored; 96.21 s |
| `quaternion_scan_fwd` in the f32 PTX | 18 `fma.rn.f32`, 60 other f32 mul/add/sub | the same; the PTX's SHA-256 `678be8583cd4…` is the same in both runs |
| sealed files | 10; `SHA256SUMS.files` `f6135ceac71a…` | 10 and `DONE`; `SHA256SUMS.files` `06c20cfbc8e4…` |
| job exit | 5 (the gates failed) | 0 |

The laptop gates, at `887274c24`:
- the stub-nvcc `--features cuda --all-targets` check of the crate: 0 errors;
- the crate's whole CPU suite: 275 passed, 0 failed;
- rustfmt: clean on the changed file.

The later commits change doc comments and `cfg(feature = "cuda")` test code only. At `2d34cba13` the stub-nvcc check passed again.

## Checks run at the head

- **The head adds only this REPORT, the notebook row and the item line** to the gate commit `2d34cba13`.
- **The seals,** checked twice: on the pod, and on the laptop after a full read-back from the store, 10 of 10 OK for each run.
  - Run 1's `SHA256SUMS.files` has SHA-256 `f6135ceac71a9ba652debb4007118d1b89c27fe196932a7e8134660e259e3c40`.
  - Run 2's has `06c20cfbc8e4bdf8ca1edff7f16493b5249150138c01b14caf0b01bf23262014`.
- **The store:** `caseyallard/uor-r4-store:results/claude/one-model-end-to-end-decode-cache-2026-11-04/` holds `run1/` (13 files: the 10 sealed, `SHA256SUMS.files`, `SHA256SUMS.err` and `EXIT`) and `run2/` (14 files: the same, plus `DONE`).
- **The pod** was deleted after the read-back. It was up from 16:54:38Z to 17:08:44Z: 14.1 minutes at $0.986/h.
- **`loop report --check` and `loop pr-check --local`** were run at the head before review.
- **Reviews, each in a fresh context:**
  - two code reviews of `8d0f0ffa1..887274c24`, split in half: APPROVE WITH CHANGES and APPROVE. Their doc findings are folded into `d1560863d`; the others are What did not work, items 4 and 5;
  - a third review of the test change `2d34cba13` and of this REPORT: APPROVE WITH CHANGES. Its wording findings are folded in, and its one code finding is Still open, item 2. It is recorded on PR #3672 by `loop review`.

## Skills used

None applied.

## Still open

1. **Cache unit 2:**
   - the Hamming read's step;
   - the switch of the scorer's decode loops to the step where the window fits, with a full re-run when the window slides;
   - reply rows named in step rows;
   - decode tok/s against today's files, byte for byte.
2. **Two test checks for unit 2's pod, at no extra pod:**
   - rerun the old lowest-bit flip under the carry compare, to settle What did not work, item 1;
   - assert that the stepped and captured caches hold the same number of layers and input rows before the carry compare zips them. A truncated cache would otherwise compare fewer items without error.
3. **The process-wide retention reading** and multi-threaded CUDA tests: a test lock, or one test thread enforced by the harness (What did not work, item 5).
4. **Then, as #2028 orders:** P2b's classification, the real-text corpus, P4, P5 and P6.

## Plain summary

When the model writes an answer, it adds one word at a time. Today, for every new word, it re-reads the whole conversation from the start. That is why scoring is slow: at long windows almost all the time goes there.

This unit gives the model a memory of where it was. After reading the prompt once, it keeps a small summary of each layer, and each new word only updates that summary. The rule was that this shortcut must give exactly the same numbers as re-reading everything, down to the last bit, so that no score can change.

On a graphics card it did, in all 48 cases tried, for every word and every layer. The first test run also showed that a check of the outputs alone could miss a tiny error inside the summary. So the check now compares the summaries themselves too.

Nothing uses the shortcut yet; the next unit switches scoring over to it and measures the speed. The test runs cost about $0.23.

References #3407.
