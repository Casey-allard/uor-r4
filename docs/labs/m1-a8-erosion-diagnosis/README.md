# A8 reach erosion: host placement census, controls, Q1 and Q3

Item `m1-a8-erosion-diagnosis` (#2829). Lab: claude. The design is in [DESIGN.md](DESIGN.md) (#2992).

**A placement census is not admission: no reach or admission value is recomputed here.**

Nothing in this page says why reach fell. It reports where tokens sit and which writes moved them, on the retained checkpoints of the A8 dose to Q1 (batch 4,096).

## Run

```sh
cd docs/labs/m1-a8-erosion-diagnosis/census
CARGO_TARGET_DIR=<target> ~/.cargo/bin/cargo run --locked --release --bin census -- \
  <store>/vast-archive/vast-56670997/uor-r4/runs/claude  <out>/census.json
```

- **Output:** `census.json`, sha256 `8f8555e28df3cf42047a3e075e4bdb73899d7fb6ee21b444e42092582dfacc4a`. It runs host-only in about 56 s.
- **Inputs:** 65 checkpoints per arm (`checkpoint-00000` … `04096`), both `steps.jsonl`, and both GPU cell receipts.
  - The checkpoints and `steps.jsonl` were downloaded from the HF store `caseyallard/uor-r4-store`.
  - The binary re-verified each one against its run's `FILES.sha256`: 66 files per arm.
  - The receipts are hashed and recorded, because the gates folder carries no `FILES.sha256`.
  - No retained file was edited.
- **Reading the tables:** only `.checkpoint.model.table` is read (131,072 words = 4,096 tokens × 4 blocks × 8).
- **The cell rule:** the cell is the kernel's (`discrete.cu:286-290`), with ABSENT for a non-root end block. The roots come from `normalization_mechanism::orbit_root_coords`, because `geometry` is private.

## Controls (all must pass before Q1 and Q3)

| # | Control | Result |
|---|---|---|
| 1 | Checkpoint-0 tables byte-equal across arms | **PASS**: 0 of 131,072 words differ |
| 2 | G-Init: the checkpoint-0 words digest equals the A8 pin `4122cbab44bd88658743984fc1ab79059bf032c3e2fe526ffcf4ea5f95d9cd74` | **PASS**, both arms |
| 3 | Checkpoint 0: 256 occupied cells, max occupancy 16, 0 ABSENT; the host cells equal the GPU receipt `token_cells.u16` | **PASS**, both arms: 0 of 4,096 entries differ. Both receipts hash to `f249902d…`. |
| 4 | For every 64-batch interval, the changed table slots equal the union of applied `vector_block` indices; gate writes change no slot | **PASS**: 64 intervals per arm, 0 mismatches |
| 5 | The interval count | **64 intervals per arm**, from 65 contiguous checkpoints (0 to 4,096). The "66 intervals" in the attribution correction (6261186285) **does not hold**. The join it describes holds on all 64. |

## Q1: placement per checkpoint

`moved` counts the tokens whose cell differs from checkpoint 0.

| Batch | Relation: occupied / max / moved | FrozenZero: occupied / max / moved |
|---|---|---|
| 0 | 256 / 16 / 0 | 256 / 16 / 0 |
| 512 | 256 / 18 / 25 | 256 / 18 / 24 |
| 1,024 | 256 / 18 / 26 | 256 / 18 / 25 |
| 1,536 | 256 / 18 / 40 | 256 / 18 / 46 |
| 2,048 | 256 / 19 / 56 | 256 / 19 / 64 |
| 2,560 | 256 / 20 / 65 | 256 / 20 / 72 |
| 3,072 | 256 / 20 / 74 | 256 / 19 / 84 |
| 3,584 | 256 / 20 / 76 | 256 / 19 / 87 |
| **4,096 (Q1)** | **256 / 20 / 80** | **256 / 20 / 91** |

- No token is ever ABSENT, and **no cell is ever emptied or newly filled** in either arm: all 256 stay occupied at every checkpoint.
- At Q1, both arms hold 184–185 cells at exactly 16 tokens, and the rest range from 8 or 9 up to 20.

## Q3: writes to Q1

| Arm | Applied `vector_block` writes, by block 0 / 1 / 2 / 3 | Block 0 + 3 writes (the only ones that can change a cell) | Tokens whose cell changed | Applied gate writes |
|---|---|---|---|---|
| Relation | 192: 58 / 49 / 52 / 33 | **91** | **80** | 204 (117 combine, 87 relation) |
| FrozenZero | 207: 64 / 49 / 55 / 39 | **103** | **91** | 120 (combine only; 275 relation proposals refused) |

- **Relation reproduces the correction's count:** 91 cell-changing writes over 80 tokens.
- **The census's own "cell-changing writes" field is wider:** 142 for Relation and 160 for FrozenZero. It counts every applied write to a token whose cell changed within that interval, block 1/2 writes included, because receipts carry no values. The block 0 + 3 count is the direct one.
- The vector and gate totals equal the Q1 summaries (`evidence/*-q1-summary.json`).

## What this supports, and what it does not

**Supported, as placement facts:**
- **The occupied-set route is absent to Q1** (DESIGN §2, Q4 set iii). No cell was emptied or filled, so no row's admitted cells changed through the occupied set.
- **Whole-vocabulary placement churn is not higher in Relation.** FrozenZero moved more tokens (91 against 80) with more cell-changing writes (103 against 91), yet lost less reach (857 against 551).
- **What does separate the arms, among the writes counted here, is the gate writes.** Relation has 87 applied relation-gate writes; FrozenZero has none.

**Not supported:**
- Any statement that a given row lost admission.
- That the gate writes caused Relation's extra loss. That is co-occurrence: gates act through `q`, which only a device run can measure.
- That placement is cleared for the extra loss. Q4 sets (i) targets and (ii) TRAIN predecessors are not yet counted: Relation could move fewer tokens overall but more of the ones P_succ rows depend on.

No KEEP, paired-DEV or fresh claim is made.

## Next

**Q2, Q4 (i) and (ii), and Q5** need the P_succ target tokens and their TRAIN predecessors from the dose's DEV and TRAIN pins. That is the next unit. If Relation does not move more targets or predecessors either, the remaining suspect for its extra loss is the read. Testing the read needs a device run under its own FIT'd sheet.
