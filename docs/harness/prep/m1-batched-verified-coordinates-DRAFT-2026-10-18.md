# Batched verified coordinates (BVG-K): score K coordinates per batch, commit a verified set (DRAFT)

- **Status:** DRAFT design, revised after one adversarial reviewer-subagent round (findings 1–10 folded in below).
  - **Not registered**, and TEST FITNESS is not requested. No source change, no compute, $0.
  - This is a **learning-law change** (W3 in `NOW.md`; D39 item 4: registered before any float sheet). It enters only through FIT'd pre-registration, and never mid-run.
  - **Two stages (§8):** construction (Stage C: source plus finite controls and the K latency curve, configuration-agnostic) and training arms (Stage T). Stage T is held until the K16 redirect outcome fixes the configuration of record.
- **Line:** M1 discrete learning infrastructure (#2029). **Lab:** claude.
- **Authority and rules:**
  - Owner path directive (#2029 6244066845): "batched-coordinate scoring … only as a new FIT'd pre-registration after the dose".
  - D22, D23/D25/D32 (integer CUDA, no softmax or matmul), D24 (GPU only), D29 (scanned modules).
  - Owner goal (D37): train fast and run fast.
- **Basis:** the row-parallel exact counterfactual (#2630; record `docs/labs/counterfactual-rows-parity-2026-10-18/`) and `counterfactual_path = rows_v1` (#2631).

## 1. The current law, and the gap

`PreparedDriver::transition` (`training.rs`) is exact cyclic coordinate descent:
- Each batch scores **one** coordinate's 16 virtual moves (`coordinate_at`, from `completed_batches`).
- It commits the chosen move only on strict improvement. The tie rule is the lower ordinal for Gate, and the lower resulting port for VectorBlock/Route (`discrete_choose`).
- Under `InterleavedReadEvery8V1`, every eighth batch is a read (Gate) coordinate and the rest are VectorBlock coordinates.
- The Dose 1 configuration has 16,384 VectorBlock coordinates (V4096 × B4) and a 16,404-batch schedule cycle (`schedule_cycle`). Each epoch step is one accepted update.

**Measured cost, plus a launch shape taken from source (bound 2026-10-18).**
- **Measured.** The accepted K16 qualification's registered timing record (`3efb4768ec37458f8ad1f585eca595f8cb1e94dbf489ccf8e9222786a4db8712`; 2 checkpoint-0 cells, 100 timed batches each, `rows_v1`, 64 rows) gives these per-proposal means:

  | Proposal | Objective A | Objective R |
  |---|---|---|
  | Route | 130.24 ms | 130.25 ms |
  | VectorBlock | 129.50 ms | 129.51 ms |

  Carry costs 0.47 ms. The proposal is about 99.6% of the batch.
- **Launch shape (from source).** The heavy evaluation kernel (`rows_device.rs`, `discrete_counterfactual_rows_eval`) launches `rows × 16 = 1,024` threads. `launch_config` (`device.rs`) clamps blocks to 128 threads, which gives **8 thread blocks**. An RTX 5090 has 170 SMs, so **at most 8 SMs (about 4.7%) do work** during the roughly 130 ms that dominates every batch. Each thread runs the full serial writer for its (row, lane).
- **Hypothesis (to be measured by the construction stage, §8 step 3).** Scoring K coordinates in one launch (`rows × 16 × K` threads) raises proposal throughput by up to about K× at similar latency, until SMs, registers or memory saturate. The construction stage measures the K = 1, 4, 16 per-launch latency directly; the hypothesis is not assumed.

## 2. The law: BVG-K

**The schedule keeps the read share per coordinate, not per batch.** The cursor gains a **coordinate ordinal** `o`, separate from `completed_batches`. The existing interleaved order is walked by `o`, exactly as today's `coordinate_at(o)`.
- Each batch takes the next **K coordinates of one kind**: K VectorBlock coordinates, or K read coordinates when the order reaches a read run.
- Each kind's run is bounded by the next kind change.
- So vector and read coordinates are visited in the **same proportions and order per pass** as K = 1. The read visits per vector pass are equal for both arms, and both are reported.

**For each batch:**
1. **Score.** One launch computes the baseline once per row, the virtual moves per `(row, coordinate, lane)`, and one ordered reduction per `(coordinate, lane)`. This is the #2630 replay plus a coordinate axis, using the **same row-eligibility mask as the `rows_v1` counterfactual**. The per-coordinate chooser is unchanged, with the same tie rules as above.
2. **Propose.** `S_t` is the coordinates whose chosen move strictly improves, ordered by coordinate ordinal. Let `best` be the member with the lowest single-move loss.
3. **Commit:**
   - `S_t` empty: commit nothing (a no-op batch), and advance `o` by the block.
   - `|S_t| = 1`: commit that move. **There is no verify forward**, so K = 1 pays no extra launch.
   - `|S_t| ≥ 2`: one forward on the same batch, with all of `S_t` applied virtually, gives `L(S_t)`. **Commit whichever of `S_t` and `{best}` has the lower batch loss**, both scored on the same pre-commit state. Every committed state therefore strictly lowers this batch's loss. The joint verify is **required**, not optional, as section 2a explains.
4. **Accounting:**
   - `epoch` advances **+1 per batch with a commit**, which is unchanged at K = 1.
   - A new `committed_moves` counter counts moves.
   - `accepted_updates` keeps its meaning as committed batches, so `epoch == accepted_updates` still holds.
   - The cursor records `o`. Checkpoint, resume and admission validate `o`, `completed_batches`, `accepted_updates` and `committed_moves` together.
5. **Records** per batch: K, `|S_t|`, whether a joint or single commit happened, `L_before`, `L(S_t)`, `L(best)`, the committed loss and `o`.

### 2a. Why moves interact, so verification is needed

- **Storage is disjoint:** VectorBlock `index` writes `table[index*8..index*8+8]`.
- **Losses are not additive**, for three reasons:
  - Consecutive indices group 4 tokens × 4 blocks.
  - A token's address cell reads its blocks 0 and B−1 together (`token_cell`).
  - The writer's occupancy and crowded-cell rivals are shared across tokens (`writer`).

One joint verify against the same pre-commit state is enough for monotonicity, given the commit rule in step 3.

### 2b. K = 1 is the current law

With K = 1, `S_t` is empty or one move, there is no verify, the epoch and cursor accounting are unchanged, and `o == completed_batches`. **Rollback identity:** the state, epoch and accepted-update traces equal the current `rows_v1` law bitwise (control V1).

**Not changed:** the objective (`NearestAddressMarginV1`), the read, the move sets, the chooser rule, the batch rows, the data and the integer arithmetic.

**FrozenZeroV1:** the block path calls `apply_refused_by_policy` on read coordinates exactly as `transition` does today, so frozen relation trits are never committed. Control V8 covers this, with the refused-slot ledger re-derived as a function of K.

## 3. Implementation sketch (for the registration, not this draft)

**New, all opt-in:**
- A generated rows-K module, which is #2630's with a coordinate dimension; the baseline is shared across K.
- A batched chooser.
- A **block-apply kernel** committing a set under one epoch step, because `discrete_apply` refuses a stale epoch (status 3).
- The cursor field `o` and the counter `committed_moves`.
- `DriverConfig.coordinate_block: K`, serde-skipped at 1 so existing config bytes are unchanged.

**Interlocks:** a D29 entry census. Nothing selects it implicitly.

## 4. Finite controls (to be derived at registration)

| ID | Control | Expectation |
|---|---|---|
| V1 | K = 1 identity | Over N batches, the state hash, epoch, accepted-update and cursor traces equal the current `rows_v1` law bitwise |
| V2 | Scoring parity | For each coordinate in a block, the 16 losses equal an independent `propose_rows` for that coordinate on the same state |
| V3 | Block apply = forced sequential apply | Committing `S` by block-apply leaves the same table, routes and gates as a **test-path forced apply of the same fixed ordinals**, one coordinate at a time. A re-proposal is not used, because it could choose differently or be refused with status 5 |
| V4 | Joint-verify selection | A fixture with **two blocks of the same token** where each move improves alone but the pair worsens the batch: the single best is committed. A second fixture where the pair improves more than the best: the pair is committed |
| V5 | Monotonicity | On every committing batch of a short run, the committed loss is below `L_before` |
| V6 | Refusals | `K ∉ 1..64`, a mixed-kind block, a stale epoch on block-apply, and inconsistent cursor counters on resume are refused with typed statuses and no state change |
| V7 | D29 | The generated module carries no float, squash or matmul tokens and an exact entry list |
| V8 | FrozenZero | At K = 16, every relation trit stays 0 over a schedule reaching every read coordinate, and the refused-slot count matches the re-derived ledger |
| V9 | Checkpoint and resume | A run checkpointed mid-pass and resumed equals an uninterrupted run bitwise |

## 5. Bar

Fixed numerically at registration, before any result is visible.

- **Primary axis: equal training wall clock** (the owner's goal, D37), on identical pods.
- **Secondary axes, both reported:** equal coordinate evaluations (passes) and equal target rows consumed.
- **Lead metric:** training **target-tokens/s** (rows × context positions scored per second).
- **Second metric:** coordinate evaluations/s.
- **Quality, non-inferiority at equal wall clock:** DEV `P_succ` conditional top-1 and the full DEV loss. The **numeric margin** is fixed at registration from the fast-path Dose 1 dispersion. Seeds vary only the initialization and the corpus order is common to all seeds, so the margin is not "two-seed noise" by name.
- **KEEP:** better quality at equal wall clock, with fresh transfer reported (D22).
- **Per pass:** DEV loss per pass is reported for every arm to expose overfitting. K-move commits fitted and verified on the same 64 rows can raise in-sample per-batch fit.
- **Diagnostic only, never a stop:** joint-commit rate, mean `|S_t|`, and no-op batches.
- **Q1 and the per-family ledger are re-derived as functions of K.** For example, Dose 1's Q1@4096 batches equals 3,584 vector and 512 read coordinate evaluations at K = 1, but 16× more evaluations at K = 16. The checkpoint is therefore placed on the same axis as the primary comparison.

## 6. Base artifact

None is bound in this draft. Stage T arms start from the identical initialization of the **configuration of record that the K16 redirect outcome selects**, with the seed list fixed at registration. Stage C uses disposable fixtures and the K16 checkpoint-0 state, with zero corpus updates. There is no checkpoint promotion.

## 7. Data split

The Dose 1 TRAIN/DEV corpora and fresh-draw protocol, pinned exactly as in the fast-path packet: train/DEV `3dee858f…`/`0919e90f…`, with fresh V2 sealed until the paired DEV decision. No new data.

## 8. Steps (two stages, registered and released separately)

**Stage C: construction.** Source plus finite GPU controls, with zero corpus updates. It is configuration-agnostic, so it can proceed while the K16 redirect runs.
1. The §1 hypothesis is now bound to the measured K16 profile (done above).
2. Implement §3 as opt-in source. The default K = 1 is byte-identical, and `coordinate_block` is serde-skipped at 1.
3. Run controls V1–V9 on one GPU. Also record a **latency curve**: per-launch latency at K ∈ {1, 4, 16} on the same state, the direct test of the §1 hypothesis. The finite controls need their own TEST FITNESS and owner release; they are not a training dose.

**Stage T: training arms.** These are held until the K16 redirect outcome fixes the configuration of record (Route K16 A or R, or a successor).
4. Re-base this sheet's configuration on the K16 outcome's configuration of record, and re-derive the Q1 placement, the per-family ledger and the margin from its dispersion.
5. Run arms **K = 1, K = 4 and K = 16**, each ≥ 2 seeds, on identical pods with the pre-registered bars. K = 4 separates "K helps" from "K = 16 overfits".
6. Score on DEV, with fresh transfer for any KEEP.

## 9. Stop rule

- Any control FAIL stops the line for diagnosis (D31). It is never a negative.
- A throughput miss with quality non-inferior is KEEP-and-iterate on K. A quality loss beyond the registered margin closes only that `(K, schedule)` configuration (D22).
- Owner caps and the disk floor are hard stops. A healthy run is renewed, not cancelled.

## 10. Cost

- **Stage C (implementation, controls and the K latency curve):** under 30 min on one 5090, **about $0.50**. One K = 1 launch is about 130 ms (measured); the K latency curve is a handful of launches per K.
- **Three arms × 2 seeds:** each arm should be under an hour on the fast path, about $12 in all on a 2×5090 pod. The projection is refined at registration from the fast-path Dose 1's measured per-batch cost.

## 11. Minimum test power

≥ 2 seeds per arm on the 14,287-row Dose 1 DEV panel (`P_succ` 3,847), well above the D22 floor of 200 fresh rows. The registered numeric non-inferiority margin is set no narrower than the measured dispersion. Fresh transfer is reported for every KEEP.

TEST FITNESS: PENDING
