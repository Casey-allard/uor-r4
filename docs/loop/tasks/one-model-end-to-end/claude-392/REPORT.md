# REPORT: one-model-end-to-end, claude-392 (the CUDA twin of the Q31 read's gate update)

Lab: claude. Item: one-model-end-to-end (#3407). ORDER: [ORDER.md](ORDER.md). PR #4011.
Commits:
- `9f2258238`: the ORDER;
- `56fcad199`: the kernels, the host wrapper and the parity tests;
- `804622560`: this REPORT and the notebook row;
- `3a9685d07`: two test lines rewritten, because the D32 lexer read `)?.1` (a tuple field) as a float literal;
- the last commit: the review's margin `u64::MAX` case, gates 5 and 6, and this REPORT.

## What did not work, first

1. **The local PR check failed once on a lexer reading, not on banned code.** `banned-presence` counted two D32 sites in the test file: `train_chain(...)?.1` reads to the lexer as the float literal `.1`. The lines now name the step count, and the check passes.
2. **Nothing failed at the first run.** The laptop build (stub nvcc), clippy, and the first wick run at `56fcad199` were each green the first time. So the parity tests' strength rests on the negative gates below, not on a bug they caught.
3. **The device step is not fast yet.** Each gate-word thread runs over every row of the batch in order: `2 * width` threads, `rows` iterations each. That is fine at test size (128 rows), and it is the simplest exact layout. A real run at many thousands of rows needs a per-window partial sum first. Because every sum is exact in `__int128`, a split cannot change a word.
4. **The device step does more passes than it needs.** It runs `first_pick_read` twice (picks before and after) and the address and loss kernels twice (read and candidate). The pass before could share its addresses with the read (claude-391 Still open 6). Not done here.

## What changed

- **`native_vector.cu`, two integer kernels.**
  - `native_q31_read_pick_loss`: one thread per row. It finds the rival with the read's tie rule (`<=`, the target skipped) and computes the hinge with the margin raised to 1 when the rival is later, exactly in `__int128`. It writes the loss as two `u64` words and the three sign residuals (query, target key, rival key). The residuals are zero when the hinge is zero.
  - `native_q31_read_gate_credit`: one thread per gate word, the query gates first, then the key gates. It sums the exact conjugate products `residual (x) conj(block * 2^credit_shift)` over the rows in `__int128` and writes `sat(gate − clamp(credit >> (step_shift + 1), ±2^32))`. It multiplies by `2^shift` rather than left-shifting a negative `__int128`, which C++17 leaves undefined.
- **`build.rs`:** `NATIVE_VECTOR_ENTRIES` admits the two entries (now six). The integer PTX inventory check still rejects any float or matrix token.
- **`native_vector_device.rs`:** `NativeVectorEngine::read_train_pick_step` and its `_reference`. The host keeps:
  - the refusals, in the host rule's order: the device `first_pick_read` gives `apply`'s refusals, then targets length, step shift and target bounds, with the same messages;
  - `q31_read_credit_shift`;
  - the `u128` loss sum;
  - the strict-drop accept;
  - the re-admission of the candidate through `Q31FirstPickRead::new`.

  An empty batch returns the host rule's no-step result without a launch.
- **`tests/native_vector_cuda.rs`:** three tests, comparing every field of `Q31ReadTrainStep`, device against host.

The host rule, the read, the session and the artifact are unchanged.

## Measurements

wick (RTX 4090, `sm_89`, CUDA 12.8 container), `--release`, at `56fcad199`. Finite tests, no training run.

| Test | Device against host |
|---|---|
| claude-391's fixtures, chained step by step | equal at every step: 4 accepted steps at shift 4 and 49 at shift 8 (scale 2^18); 4 at 2^10 and 2^28; picks `[0, 1, 2]` after training |
| A margin-0 tie with a later rival | loss 1 on both, then trained to the right pick |
| An exact tie | rival row 2 (the latest) on both |
| A margin on a right pick | loss `S/2` before, accepted, on both |
| Two swapped windows, shift 0 | loss 131,072 → 393,216, rejected, on both |
| Seeded batches: width 64 and 256, scale 2^18 and full range, margins 0, 2^20 and `u64::MAX`, shifts 0, 4 and 31, three chained steps each | 108 steps equal, field for field; both accepted and rejected steps occur (asserted); at margin `u64::MAX` every hinge uses the high loss word |
| Five refusals and an empty batch | each refusal names its rule on both; the empty batch is no step on both |

The whole `native_vector_cuda` suite (10 tests) took 1.13 s on wick.

## Checks run at the head

- **Laptop:** a stub-nvcc `cargo check -p uor-r4-discrete --features cuda --lib --test native_vector_cuda` is clean. `cargo clippy` on the same targets shows nothing on the changed lines; two existing warnings in this test file (lines 297 and 298) are on main. Only the changed lines were reformatted: both edited files are rustfmt-clean on main.
- **wick, at `56fcad199` and again at `3a9685d07`** (the second run checks the container copy holds the rewritten lines): `cargo test --release --locked -p uor-r4-native-vector` passed. `UOR_REQUIRE_CUDA=1 cargo test --release --locked -p uor-r4-discrete --features cuda --test native_vector_cuda -- --test-threads=1`: 10 passed, 0 failed. The build's PTX admission passed with the two new entries.
- **Negative gates on wick.** Each sabotage was applied to the container's copy of `native_vector.cu`. Each was confirmed applied with `cmp`/`diff` and confirmed to compile. Then the three update tests ran, and afterwards the file was restored and re-run:
  1. **The rival's tie to the earliest row** (`<` for `<=` in `native_q31_read_pick_loss`): the fixtures test fails ("rivals differ").
  2. **No minimum margin for a later rival** (`m = margin`): the fixtures and seeded tests fail ("losses or accept differ").
  3. **No conjugate** (the minus signs on block words 1 to 3 removed): the fixtures and seeded tests fail.
  4. **No `+ 1` in the step's shift** (`credit >> step_shift`): the fixtures and seeded tests fail.

  5. **The high loss word dropped** (`loss[2 * item + 1] = 0`): the seeded test fails. It passed before the review's case with margin `u64::MAX` was added, because no row's hinge reached 2^64.
  6. **The clamp removed:** the tests still pass, and that is expected. A step of `2^32` or more already moves any gate word past `±2^31`, so `q31_saturate` gives the same word with or without the clamp. The clamp only keeps the cast to `s64` from wrapping, which needs `|credit| >= 2^64`: more than about 2^29 aligned rows in one batch. No finite test reaches that, so this guard is argued by arithmetic here, not pinned by a test.

  After each restore, `cmp` matched the original and the three tests passed (3 passed, 0 failed). The full suite at the final test file passed (10 passed).
- `uor-harness loop pr-check --local`: see the PR record.

## Skills used

`native-space-cycle`: the ORDER's five items. One operator definition (the host `train_pick_step`) is compared against its device twin on every output word. Both are integer only, and the PTX admission rejects float tokens.

## Still open

1. **A pre-registered training run** of the read's gates on served states with target picks, judged on held-out picks. It needs a sheet with a non-author FIT (§4.5). This twin is its GPU path.
2. **Speed.** The gate-credit kernel is serial over the rows (What did not work 3), and the step repeats forward passes (What did not work 4). Both need fixing before a run at scale. A window-partial sum keeps the result exact.
3. **From claude-391:**
   - Still open 3: the weights' update;
   - Still open 4: a step-size schedule;
   - Still open 5: one credit shift per batch.

   These stand unchanged.

## Plain summary

The rule that teaches the read which earlier fact to copy now also runs on the GPU. On every test case, including random batches at real scale and full range, the GPU gives the same integers as the laptop: every gate word, the loss, the accept decision and the picks. Breaking any one of four details on the GPU (the tie rule, the minimum margin, the conjugate, the step's divisor) makes a test fail. The GPU version is still simple and slow, one thread per gate word over all rows. That is enough to check correctness, and it needs a faster split before a real training run.
