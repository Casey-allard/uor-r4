# BVG-K Stage C2: block apply and schedule — 2026-10-18

- **Lab:** Claude, W3 (BVG-K design owner). **Parent design:** `docs/harness/prep/m1-batched-verified-coordinates-DRAFT-2026-10-18.md` §2 and §8 step 3.
- **Status:** source in progress, delivered in pieces. No GPU run yet, no training, and no hold changed. Each piece gets a fresh-context reviewer-subagent review before merge (owner direction, #2029 6255848646). The GPU controls run in a C2 qualifier (H4) after FIT.

C2 is split into four pieces:

| Piece | What it adds |
|---|---|
| H1 | Per-family cursor schedule, the `coordinate_block` config and cursor fields (host) |
| H2a | The appended `bvgk` CUDA module |
| H2b | The device API that launches it and computes `L(S_t)` on a scratch state |
| H3 | The driver block transition and checkpointing |
| H4 | The GPU qualifier for V1, V3–V6, V8 and V9 |

## H2a: appended `bvgk` CUDA module

`bvgk_source.rs` generates `bvgk.cu` as the generated `rowsk.cu` text plus three integer-only kernels. The rows and rowsk images are byte-identical: `discrete.cu`, `rows_source.rs` and `rowsk_source.rs` are unchanged.

- **`discrete_bvgk_choose`.**
  - It applies `discrete_choose`'s per-coordinate rule to the C1 scores: strict improvement against lane 0; Gate ties go to the lower ordinal, VectorBlock and Route ties to the lower resulting port.
  - A coordinate joins `S_t` only if its C1 status is 0, it is eligible and its index is in range.
  - It writes `[|S_t|, best, L(best)]`, where `best` has the lowest single-move loss and ties go to the lowest ordinal.
- **`discrete_bvgk_decide`.** It returns an empty set, `{best}`, or `S_t` only if `L_joint < L(best)` strictly. It zeroes the commit set before any refusal: status 4 for an inconsistent summary, status 17 when the joint loss is absent at |S_t| ≥ 2.
- **`discrete_bvgk_apply`.**
  - It validates every member before any write: 14 kind mismatch, 3 stale epoch, 9 gated mismatch, 4 out of range, 5 not improving or epoch at MAX, 15 duplicate index.
  - It writes in ordinal order and increments `++epoch` exactly once.
  - Status 16 (empty set) is a misuse code: the host skips apply for a no-op batch.
- **Build and D29.**
  - `build.rs` builds `bvgk.ptx` and `bvgk.cubin`, admitting exactly 8 PTX entries.
  - The D29 census now reads 11+1+1+3+5+8, with self-tests for planted float, `expf`, `tanhf` and `mma`, and for extra, missing and renamed entries.
  - Drift pins assert that the chooser rule, the move and proposal writes, the gate write and `resulting_port` match `discrete.cu` exactly.

**Review.** A fresh reviewer subagent returned PASS-WITH-FIXES with no correctness defect in the intended call order. Its fixes are applied: decide zeroes the commit set before refusing, status 16 is documented as a misuse code, and the drift pins are extended.

Two of its notes are carried to H4:
- **V3:** block apply ends at epoch +1, against +|S| for sequential single applies. V3 compares table, routes and gates, re-stamps each sequential proposal, and checks the epoch separately.
- **Lane-0 gate member:** apply refuses one, where `discrete_apply` returns silently. This cannot be reached through choose then decide.

**Checks (host).**
- `uor-r4-discrete` lib tests: 112 passed.
- Stub-nvcc `check --features cuda --all-targets`: exit 0.
- D29 self-test and repo: PASS.
- fmt: clean.
- Negative gate: removing decide's zeroing fails the new test.

`driver_source_sha256` is unchanged; no include-list file is touched.

## H1: per-family cursor schedule, `coordinate_block`, cursor fields (host)

- **Config.** `DriverConfig.coordinate_block` (K, 1..=64) is serde-skipped at 1. K > 1 requires `counterfactual_path = RowsV1`.
- **Cursor.**
  - `Cursor.block: Option<BlockCursor { o_vec, o_read, committed_moves }>` is serde-skipped when `None`. It must be `None` at K = 1 and `Some` at K > 1.
  - `(o_vec, o_read)` are absolute family visit counts and a pure function of `completed_batches`. Resume recomputes and checks them.
  - The invariant `accepted_updates ≤ committed_moves ≤ K·accepted_updates` is enforced, and each batch commits 0 moves, or 1..=block length when it commits.
- **Schedule.**
  - `block_at` follows today's batch-kind rule: under `InterleavedReadEvery8V1`, read iff `b % 8 == 7`; under `CanonicalVectorThenReadV1`, ceil(vectors/K) vector batches then ceil(reads/K) read batches.
  - Each family takes the next `min(K, remaining in its pass)` ordinals, so no block straddles a pass.
  - `advance_block` advances the cursor.
  - At K > 1, `coordinate_at` and `advance` refuse with a focused message.
- **Initial checkpoints.** Every initial-checkpoint writer and guard now uses the K-aware `driver.initial_cursor()`, which equals `Cursor::initial()` at K = 1.
- **Pinned evidence kept.** The pinned `stage-a-admission` instrument is untouched; its `..Cursor::initial()` literal compiles as is. The unpinned `fresh-scoring-eval` instrument gains `block: None` in one test literal.

**Tests (host):**
- V1: K = 1 `block_at` equals `coordinate_at` on the 24-step and C192 16,404-step fixtures, under both schedules.
- Visit ratio: exact per-family visit counts at K = 4 and 16.
- K = 3 truncation: exact block lists at pass boundaries under both schedules, plus a resume right after a truncated block.
- V6 refusals: focused messages.
- V9: resume mid-pass equals the uninterrupted sequence.
- V10: K = 1 config and cursor bytes are unchanged, and legacy JSON gives K = 1.
- A K = 4 initial checkpoint validates.

**Negative gates:** removing the cursor recompute fails the resume-refusal test, and removing the pass clamp fails both K = 3 tests.

**Review.** A fresh reviewer subagent returned PASS-WITH-FIXES with no defect on the K = 1 path or in the schedule arithmetic. Its fixes are applied: K-aware initial cursors, K = 3 truncation tests, `coordinate_at` refusing at K > 1, and the vacuous assertions replaced.

**Effect.** `driver_source_sha256` changes, because `training.rs` is in its include list. The running K16 roles use frozen build `21224b44` and are unaffected.

**Follow-up for H3.** These remaining `Cursor::initial()` uses still need K-aware routing once the block transition exists: `completion_baseline.rs` (two sites), `redirect_admission.rs` and one site in `controller.rs`. Today they refuse loudly at K > 1, not silently.
