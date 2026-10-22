# A8 reach erosion: host placement census, controls, Q1 to Q5

Item `m1-a8-erosion-diagnosis` (#2829). Lab: claude. The design is in [DESIGN.md](DESIGN.md) (#2992).

**A placement census is not admission: no reach or admission value is recomputed here.**

Nothing in this page says why reach fell. It reports where tokens sit and which writes moved them, on the retained checkpoints of the A8 dose to Q1 (batch 4,096).

## Run

```sh
cd docs/labs/m1-a8-erosion-diagnosis/census
CARGO_TARGET_DIR=<target> ~/.cargo/bin/cargo run --locked --release --bin census -- \
  <store>/vast-archive/vast-56670997/uor-r4/runs/claude \
  <corpus>  <out>/census-targets.json
```

- **Output:** `census-targets.json`, sha256 `789e3aa848d2b00e746d439da43ab79f385d88af4c35ba12d4f40acdef6a261d`. It runs host-only in about 6 s (release build).
- **Corpus inputs (`<corpus>`):** `mw-balp/train/{tokens.u16,response_mask.u8}` and `ft/dev/{tokens.u16,response_mask.u8}`, loaded through `PreparedCorpus::load`, which refuses any file whose sha256 differs from the A8 config pins (TRAIN `3dee858f…` / `b5e7352d…`, DEV `0919e90f…` / `a838bb1a…`).
- **Inputs:** 65 checkpoints per arm (`checkpoint-00000` … `04096`), both `steps.jsonl`, and both GPU cell receipts.
  - The checkpoints and `steps.jsonl` were downloaded from the HF store `caseyallard/uor-r4-store`.
  - The binary re-verified each one against its run's `FILES.sha256`: 66 files per arm.
  - The receipts are hashed and recorded, because the gates folder carries no `FILES.sha256`.
  - No retained file was edited.
- **Reading the tables:** only `.checkpoint.model.table` is read (131,072 words = 4,096 tokens × 4 blocks × 8).
- **The cell rule:** the cell is the kernel's (`discrete.cu:286-290`), with ABSENT for a non-root end block. The roots come from `normalization_mechanism::orbit_root_coords`, because `geometry` is private.

## Controls (c1–c5 gate Q1 and Q3; c6–c7 gate Q2, Q4 and Q5)

| # | Control | Result |
|---|---|---|
| 1 | Checkpoint-0 tables byte-equal across arms | **PASS**: 0 of 131,072 words differ |
| 2 | G-Init: the checkpoint-0 words digest equals the A8 pin `4122cbab44bd88658743984fc1ab79059bf032c3e2fe526ffcf4ea5f95d9cd74` | **PASS**, both arms |
| 3 | Checkpoint 0: 256 occupied cells, max occupancy 16, 0 ABSENT; the host cells equal the GPU receipt `token_cells.u16` | **PASS**, both arms: 0 of 4,096 entries differ. Both receipts hash to `f249902d…`. |
| 4 | For every 64-batch interval, the changed table slots equal the union of applied `vector_block` indices; gate writes change no slot | **PASS**: 64 intervals per arm, 0 mismatches |
| 5 | The interval count | **64 intervals per arm**, from 65 contiguous checkpoints (0 to 4,096). The "66 intervals" in the attribution correction (6261186285) **does not hold**. The join it describes holds on all 64. |
| 6 | The P_succ copy rule (copied from `attention_evidence.rs:7-20`), over the DEV rows enumerated as `dev_row_identities` does (context 192, 64 rows per batch), gives 14,287 rows and 3,847 P_succ | **PASS**: 14,287 rows, 3,847 P_succ |
| 7 | TRAIN payload 7,512,797 tokens and 2,425,925 eligible targets; DEV 14,287 eligible targets | **PASS**: all three exact (TRAIN has 60,000 documents) |

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

## P_succ targets and TRAIN predecessors

- **P_succ rows:** the 3,847 DEV rows of control 6. They have **448 distinct target tokens**.
- **TRAIN predecessors:** for each target t, every s with a nonzero TRAIN bigram count s → t (`BigramCounts::count` over the 2,425,925 eligible TRAIN positions). Their union over the 448 targets is **1,325 tokens**. Every P_succ row's target has at least one predecessor.
- **Cell distance:** the writer's root metric (`discrete.cu:274-276`, `address_distance` over `root_cost`). It is the L1 over the 8 words of the two block-0 roots plus the L1 over the 8 words of the two block-3 roots. A unit test checks it against `endpoint_address_distance`.

## Q2: P_succ targets per checkpoint

- `moved` counts the distinct P_succ targets whose cell differs from checkpoint 0.
- `rows` counts the P_succ rows whose target is moved.
- `pred. dist.` is the mean over all 3,847 P_succ rows of the target's predecessor distance. That is the bigram-weighted mean, over the target's TRAIN predecessors, of the cell distance between target and predecessor. A rise means placement drifted away from the bigram relation A8 built on.

| Batch | Relation: moved / rows / pred. dist. | FrozenZero: moved / rows / pred. dist. |
|---|---|---|
| 0 | 0 / 0 / 8.564 | 0 / 0 / 8.564 |
| 512 | 20 / 884 / 9.038 | 19 / 700 / 8.632 |
| 1,024 | 20 / 884 / 9.038 | 19 / 700 / 8.632 |
| 1,536 | 32 / 1,018 / 9.300 | 39 / 934 / 8.958 |
| 2,048 | 47 / 1,306 / 9.832 | 54 / 1,182 / 9.407 |
| 2,560 | 52 / 1,365 / 9.933 | 59 / 1,241 / 9.542 |
| 3,072 | 57 / 1,401 / 10.096 | 64 / 1,277 / 9.732 |
| 3,584 | 59 / 1,444 / 10.111 | 67 / 1,321 / 9.739 |
| **4,096 (Q1)** | **63 / 1,461 / 10.123** | **71 / 1,369 / 9.799** |

## Q4: cell-changing writes on the sets P_succ rows depend on

These are applied block-0/3 `vector_block` writes, to Q1, whose token's cell changed in that interval.

| Set | Relation | FrozenZero |
|---|---|---|
| (i) P_succ target tokens (448) | **69** writes on 63 targets | **77** writes on 71 targets |
| (ii) TRAIN predecessors of P_succ targets (1,325) | **81** | **94** |
| (iii) all tokens, for reference (= Q3) | 91 | 103 |
| P_succ rows whose target took such a write | **1,461** | **1,369** |

Sets (i) and (ii) overlap, so their counts do not add.

## Q5: applied gate writes per 64-batch interval

The timeline is listed in the output as `targets.q5_applied_gate_writes_per_interval`, one combine/relation pair per interval, 64 intervals per arm. Gate indices 0–19 are combine and 20–44 are relation.

- **Relation:** 117 combine and 87 relation. Relation-gate writes start in the third interval (batches 128–192) and recur throughout to Q1.
- **FrozenZero:** 120 combine and 0 relation.

These totals equal Q3's. Q5 is a timeline only; it says nothing about what a gate write did to `q`.

## What this supports, and what it does not

**Supported, as placement facts:**
- **The occupied-set route is absent to Q1** (DESIGN §2, Q4 set iii). No cell was emptied or filled, so no row's admitted cells changed through the occupied set.
- **By token count, Relation moves fewer of the tokens P_succ rows depend on.** Relation has fewer cell-changing writes than FrozenZero on P_succ targets (69 against 77) and on their TRAIN predecessors (81 against 94). It also moves fewer distinct targets (63 against 71).
- **By row weight, Relation's placement damage is larger.** The targets Relation moves carry more P_succ rows (1,461 against 1,369, at Q1). The mean predecessor distance rose more in Relation: by 1.559, against 1.235 for FrozenZero, from the same 8.564 at checkpoint 0. Relation's lead is already present at batch 512 (884 against 700 rows).
- **Among the writes counted here, the arms still differ in gate writes.** Relation has 87 applied relation-gate writes; FrozenZero has none.

**What this means for the suspects:**
- **Placement remains a suspect** for Relation's extra loss, on the row-weighted measures, though not on the token counts. Relation moved fewer target and predecessor tokens, but they were the heavier ones for P_succ rows, and the target-to-predecessor relation drifted further.
- **The read is not cleared either.** The row-weighted gap is moderate: 92 rows (about 2.4% of the P_succ rows) and 0.32 in mean distance. Whether that covers the reach gap between the arms is not something placement counts can say.

**Not supported:**
- Any statement that a given row lost admission. A moved target or a larger predecessor distance is not a lost admission.
- That the gate writes, or the placement drift, caused Relation's extra loss. Both are co-occurrence. Gates act through `q`, and admission depends on `q` and the cells together, which only a device run can measure.
- That "the read" means gate writes alone. `q` is computed over the context window's token vectors, so a table write to any block of a context or query token also changes `q`, not only a gate write.

No KEEP, paired-DEV or fresh claim is made.

## Next

Separating placement from the read needs admission itself. One option is a device run under its own FIT'd sheet: replay Relation's Q1 table with FrozenZero's gates, and the reverse.
