# A8 reach erosion: design of the host-only placement census

Item: `m1-a8-erosion-diagnosis` (#2829). Lab: claude. Base: `77fedb347`.

**Status:** design note. No census has run yet, and no admission is recomputed anywhere in this work.

## 1. What fell

The two A8 dose arms start from byte-equal checkpoint-0 models (`docs/labs/a8-dose-2026-10-19/README.md:26`) and share one reach at checkpoint 0: B3_init, the admitted P_succ target positions, is **1,351 / 3,847** (`:24`, `:45-48`). At Q1 (batch 4,096):

| Arm | P_succ reach at Q1 | Change | Accepted updates to Q1 (vector / gate-combine / gate-relation) |
|---|---|---|---|
| Relation (`LearnV1`) | **551** | −59% | 192 / 117 / 87 |
| FrozenZero | **857** | −37% | 207 / 120 / 0 (275 relation proposals refused by policy) |

- The counts come from `evidence/relation-q1-summary.json` and `evidence/frozen-zero-q1-summary.json`.
- Relation stopped on the half-init rule: 551 < 675.5. It was not below 500 (`README:47`), and its status is `Q1_STOP_FOR_DIAGNOSIS`.
- FrozenZero ran all 16,404 batches.

**The control eroded too, with no relation update applied.** So the relation updates are not necessary for erosion. The question splits in two:
- (a) why both arms lose reach;
- (b) why Relation loses about 300 more positions.

## 2. Three suspects

| Suspect | How it could cut reach | Host-observable? |
|---|---|---|
| **Update moves** (accepted `VectorBlock` writes) | A write to block 0 or block B−1 changes the token's address cell. It can move a P_succ target out of the cells its predecessors' reads admit. It can also change the **occupied-cell set**: the reader admits the K = 8 *occupied* cells nearest `q` (`docs/mechanisms/address-cells.md:23`). So a move of **any** token, target or not, that empties a cell or fills a new one can change which cells a row admits. A write to block 1 or 2 changes only rank data inside the cell. | **Yes.** Cells are rebuilt from each checkpoint's table, and the receipts name each written slot. |
| **The read** (accepted `Gate` writes: combine, and relation for Relation only) | Gate values change the read output `q`, so a different set of cells is admitted for the same row even with the table unchanged. | **No.** `q` and admission need device evaluation, which this item excludes. The host can only count and time these writes. |
| **The instrument** | The Q1 evaluation differs from the checkpoint-0 one; the wrong checkpoint is scored; custody or join errors. | **Partly.** Checkpoint identity, hashes and the receipt-to-checkpoint join are checkable. The Q1 evaluator itself is not re-run. |

## 3. Questions the host data can answer, with what each supports

| # | Question | Supported claim | Not supported |
|---|---|---|---|
| Q1 | Per arm and per 64-batch checkpoint: the token→cell map, occupied cells, occupancy histogram, max occupancy, and the count of tokens whose cell differs from checkpoint 0 | How much **placement** changed, and when | Any admission or reach value |
| Q2 | For the target tokens of the 3,847 P_succ rows: how many changed cell by each checkpoint, and the root-metric L1 distance from each target's cell to its TRAIN predecessors' cells (the relation A8 placed on) | Whether the **A8 placement property eroded**, per arm, and how fast | That a target lost admission; admission depends on `q`, not only on cells |
| Q3 | Per interval and arm: which accepted moves changed a cell (`VectorBlock` with `index % 4` ∈ {0, 3}) and which only ranks (∈ {1, 2}) | The **direct cell-changing write rate** per arm. The correction found 91 such writes over 80 tokens for Relation (6261186285). | That those writes, rather than the gate writes, caused the loss |
| Q4 | Do the arms differ in cell-changing writes on three sets: (i) the P_succ target tokens, (ii) their TRAIN predecessors, (iii) any token whose move empties a cell or fills a new one (the occupied-cell set)? | If Relation exceeds FrozenZero on **none** of the three, the extra loss (b) is unlikely to come from placement, which **points to** the read, as a hypothesis. If it exceeds on any of them, placement stays a live suspect for (b). | Proof that the read caused it; that needs a device run under its own FIT'd sheet. Writes to blocks 1–2 of context tokens may also act on `q` through the read; the host cannot see that. |
| Q5 | Gate writes per interval: count and timing against the reach curve, where checkpoints carry B3 | Co-occurrence only, as the dose README already words it (`:68-69`) | Attribution |

**A placement census is not admission.** No result from this work can be called KEEP, a paired DEV verdict or a fresh result, and fresh stays sealed.

## 4. Controls the census must reproduce before any claim

1. **Byte-equal checkpoint 0.** The two arms' `checkpoint-00000.json` tables are identical word for word.
2. **G-Init.** The checkpoint-0 table equals the A8 pin word for word: table file `6600bdb1306dbaea4d77f5bf021e717a6bef791e44c1ec7d7c9bb9f0e8cd3cd1` (`attention_learning.rs:33`) and table digest `4122cbab…`.
3. **The A8 placement facts at checkpoint 0.** 256 occupied cells and max occupancy 16 (`docs/labs/address-cells-preflight-2026-10-19/README.md:40`). The host-rebuilt cell map must equal the GPU receipt `cells/token_cells.u16` in the gates archive (8,192 B, 4,096 entries) entry for entry.
4. **Receipt join.**
   - Between consecutive checkpoints, the changed-slot set must equal the union of accepted `VectorBlock` indices in `steps.jsonl` over that interval. The correction reports 0 mismatches for Relation.
   - The census repeats this for FrozenZero.
   - Gate and route writes must change no table slot.
5. **The interval count.** The correction says "all 66 64-batch intervals (0→4,096)". The archive index holds 65 Relation checkpoints (00000…04096), which is 64 intervals. The census states which is right before using either.

If any control fails, the result is an instrument finding, recorded as such, not a mechanism result.

## 5. Data (read-only, verified copies)

- **Where it lives:** HF store `vast-archive/vast-56670997/uor-r4/runs/claude/`. A live listing on 22 Oct 2026 at about 19:25Z matched the 2026-10-20 index (`archive/storage/hf-store-index-2026-10-20.tsv`). It shows 65 Relation checkpoints, 00000…04096, so 64 intervals, and both gates receipts at 8,192 B.
- **Relation:**
  - 65 checkpoints, `a8-dose-relation-seed1/checkpoint-{00000..04096}.json`, about 6.97 MB each, 453 MB in all;
  - `steps.jsonl` (4.0 MB);
  - `FILES.sha256`.
- **FrozenZero:**
  - the same 65 checkpoint indices, 00000…04096, so the arms match at Q1;
  - `steps.jsonl` (17 MB);
  - `FILES.sha256`.
  - Its later 193 checkpoints (to 16,404) are read only if the trend past Q1 is needed.
- **Gates:** `a8-dose-gates-20261019/*/cells/token_cells.u16` per arm, for control 3.
- **Checking and cleanup:** every downloaded file is checked against its run's `FILES.sha256` before use. The custody `FILES.sha256` self-entry is stale (`README:83`), so the per-run lists are used. Downloads live under `local/` and are deleted after the run; nothing retained is edited.
- **Each `steps.jsonl` line** is one `StepReceipt` (`crates/uor-r4-discrete/src/training.rs:2530`). It holds the coordinate (`kind` ∈ {`vector_block`, `route`, `gate`}, plus `index`), `target_positions`, `candidate`, `loss_before`/`loss_after`, `applied` and `any_candidate_cost_differs_from_identity`.
  - For a `vector_block`, token = `index / 4` and block = `index % 4`; that is the correction's slot rule, with B = 4.
  - Values before and after come from the bracketing checkpoints.

## 6. The tool

- **What it is:** a Rust host binary in `docs/labs/m1-a8-erosion-diagnosis/census/`, an instrument crate like the one in `docs/labs/m2-fresh-heldout-panel/instrument`. It reads only `.model.table` from each checkpoint JSON (`Snapshot.table`, `vocab × blocks × 8` words, `snapshot.rs:12`). It needs no driver, corpus or GPU.
- **Cell rule:** it rebuilds each token's cell with the kernel's rule (`token_cell`, `discrete.cu:286-290`): find blocks 0 and B−1 among `geometry::ROOTS`, then `cell = root(block 0)·16 + root(block B−1)`. If either block is not one of the 16 roots, the token is **ABSENT**. A write that takes a block off the 16-root orbit therefore makes the token ABSENT; it never yields a cell off the 256 grid. The host test inverse in `address_cells.rs:286-298` uses the same rule but has no ABSENT branch, so the census adds one.
- **The GPU receipt for control 3:** `token_cells.u16` is written by `export_token_cells` (`attention_preflight.rs`, over `discrete_export_token_cells`, `discrete.cu:367-372`). It holds one little-endian u16 per token, 4,096 entries (`coverage_census.rs:186-192`). The export refuses a table with an ABSENT token (status 8, `discrete.cu:370`), so its entries are 0–255.
- **Not reused:** the census does not reuse `coverage_census::run`, which needs `PreparedDriver` and a corpus. It only reads `DriverCheckpoint.model` as JSON (`training.rs:409`), down to `.table`.
- **P_succ targets and predecessors:** these come from the dose's DEV panel and TRAIN pins, read the same way the A8 builder reads them.
- **Order of testing:** controls 1–3 first, on checkpoint 0 only; then the join (control 4); then Q1–Q5.
- **Output:** one JSON per arm and checkpoint, plus a joined summary. The result is one row in `docs/research/notebook/m1-discrete-attention.md`.

## 7. What this cannot decide, and the next step if needed

If Q4 points at the read, the next step is a device run that recomputes admission per checkpoint. It needs its own one-page sheet and a non-author FIT, and it is out of scope here. The protected-reach controller (`docs/labs/admission-anchor-controller-2026-10-19/README.md`) is built, and its GPU qualification is NOT_RUN. This census can inform its design but cannot qualify it.
