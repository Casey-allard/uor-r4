# Learned attention link from an address-structured injective codebook (A8)

- **Status:** registered pre-registration, **TEST FITNESS: FIT** (see the end of the sheet). The file name has no `-DRAFT-` because code pins this path once the sheet is registered (see Source).
  - Training stays HELD. A run needs FIT, then the source registration below merged, then PM dispatch, then Casey's typed D40 release.
- **Line:** M1 learned attention (#2029). **Lab:** claude.
- **Parent:** `m1-attention-injective-init-DRAFT-2026-10-18.md`, the PrimeZeta injective sheet (FIT #2712, SHA-256 `e47ce5afb3966bda13cd04bd9b99849b401c028e078de958fa98b5e8b7ed4efb`).
  - **Exactly one variable changes: the pinned table.** PrimeZeta (`prime_zeta`, file `83c1380a…`, table `e9e03bcd…`) becomes `address_structured_v1` (file `6600bdb1306dbaea4d77f5bf021e717a6bef791e44c1ec7d7c9bb9f0e8cd3cd1`, table `4122cbab44bd88658743984fc1ab79059bf032c3e2fe526ffcf4ea5f95d9cd74`).
  - Every other section of the parent is inherited verbatim. Through the parent, that includes the grandparent `m1-learned-attention-link-2026-10-17.md`'s:
    - arms, schedule, read, objective and K;
    - §7.6 bars, §7.7 decision rule and §7.8 instrument validity;
    - §7.12.1 pre-flight, §7.12.2 Q1 gate, §7.12.3 fresh order and §7.12.5–§7.12.9.
- **Why this sheet exists:** the address-cells pre-flight (record `docs/labs/address-cells-preflight-2026-10-19/`, #2751) promoted A8 under its sheet's step 4. On A8's checkpoint 0:
  - B3_init(P_succ) **1,351 / 3,847**;
  - net G-Cap **1,351**, against a threshold of 500 under either reading of the control term. One reading uses 133 (A8's full-panel checkpoint-0 top-1, an upper bound for the P_succ term); the other uses 38 (P8's, per DeepSeek's post-merge erratum on #2751). This sheet uses the parent's definition, the FrozenZero arm's own P_succ value, recorded at the pre-run gate (§3);
  - G-Init's table check and both negatives, and G-Occ (256 cells, tie mass 0).
  - **Not run:** G-Init's comparison of routes, gates and epoch against an **unpinned** initializer. Only cross-arm equality was measured, so the erratum makes training conditional on that comparison or a waiver. This sheet makes it a hard pre-run gate (§3), so it runs before any update.
  
  On the PrimeZeta table the same gates had stopped the parent at 127 / 3,847.
- **Rules:** D22, D23/D25/D30 (no softmax, squashing or matmul), D24 (GPU only), D29, D31, D11, D40. The A8 table is integer host data preparation from TRAIN only. The driver, kernels and training are integer; there is no float anywhere on this path. Unlike the parent, it needs no D37 zeta exception: `address_cells.rs` and `address-cells-codebook` contain no `f32` / `f64` (grep, 0 matches).
- **D22 scope of this sheet (replaces the parent's):**
  - `pinned_table_v1` initialization with the A8 table above;
  - the parent's arms (Relation `LearnV1` vs `FrozenZeroV1`), read `Relation{8,1}`, `InterleavedReadEvery8V1`, `ActiveLeafV1`, objective A, K = 8, `rows_v1`, seed 1 first, and the Dose 1 data pins.
  - A negative here closes only this configuration, never the mechanism. The PrimeZeta configuration keeps its own sheet and scope.
- **Identity pins are all new** and are recorded in the release packet, not here: source `driver_source_sha256`, both configs, lineage, the table file SHA and the built binary. No receipt is inherited as PASS; the pre-flight values are the expected values, not substitutes.
  - As in the parent, the driver hash is not written in this sheet. The driver source will contain this sheet's SHA-256, so a driver hash here would be circular. The plan's `execution.driver_source_sha256` carries it.

## 1. Why: placement made the targets visible to the read

The parent removed aliasing (4,096 distinct vectors, a ceiling of 14,287, zero tie mass) but spread tokens uniformly over 256 cells. At K = 8 the writer admits the 8 occupied cells nearest `q`, so a uniform table admits about 8/256 of targets. That gave 427 / 14,287 overall and 127 / 3,847 on P_succ.

A8 places each token's cell near the cells of its TRAIN predecessors (greedy L1 on the writer's root metric, quota 16 per cell, still injective). Measured at checkpoint 0 with zero updates (#2751), on the same rows:

| | P8 (parent's table) | S8 (shuffled packing) | **A8** |
|---|---:|---:|---:|
| P_succ admitted (of 3,847) | 127 | 476 | **1,351** |
| Full panel admitted (of 14,287) | 427 | 1,733 | **4,244** |
| top-1 at checkpoint 0 (full) | 38 | 99 | 133 |

- A8 against P8 is a paired one-sided contrast: b − c = 1,224 > F ≈ 74.
- A8 against S8 is the same kind of contrast: b − c = 875 > F ≈ 81. So the gain is attributed to the predecessor relation, not to frequency packing.

**This characterizes the instrument at checkpoint 0. It is not evidence that the read learns.** Whether the Relation arm learns beyond the FrozenZero control is this sheet's question.

## 2. What changes from the parent

| Item | Parent | This sheet |
|---|---|---|
| Pinned table | PrimeZeta V4096 B4 (`83c1380a…` / `e9e03bcd…`) | **`address_structured_v1` V4096 B4** (`6600bdb1…` / `4122cbab…`), rebuilt from TRAIN by `address-cells-codebook` (#2742) |
| Registered sheet | the parent's SHA, bound for every pinned table | **this sheet's SHA, bound only when the pinned table file is `6600bdb1…`**. PrimeZeta keeps the parent's SHA, and any other pinned table is refused (§6). |
| Everything else | as listed under Parent | **unchanged** |

**Why objective A and K = 8 stay:**
- One variable keeps attribution clean.
- A8 already clears the K = 8 admission gates.
- K = 16 is outside the attention validator, and the pre-flight's K = 16 bracket decides nothing.

## 3. Bar

**Pre-run instrument gates:** the parent's §3 items 1–4, inherited with the table pins replaced by A8's, on both arms' checkpoint 0 at the new source head:
- the word digest must equal `4122cbab…`, and the table file `6600bdb1…`;
- negative (a) uses an edited A8 file with stale hashes;
- negative (b) uses a re-hashed, re-pinned A8 file whose changed word is a non-root word. It is rebuilt from A8, not reused from the pre-flight's PrimeZeta-derived input.

Old receipts are not inherited. A gate failure is `INSTRUMENT_UNREACHABLE` (D31), not a mechanism negative.
- G-Init includes the parent's comparison of routes, gates, seed and epoch against the unpinned initializer for the same config. The pre-flight checked these only across arms, so the pre-run gate runs the unpinned comparison.
- **Reproduction gate (new, D31):** on **both** arms' checkpoint 0, the values must equal the #2751 receipt exactly, because the inputs are byte-identical and the two arms share one checkpoint-0 model. The values are B3_init on P_succ 1,351 / 3,847, B3_init on the full panel 4,244 / 14,287, and top-1 on the full panel 133. Any difference is an instrument fault.
- **Bar (i)** is the parent's: `top1_control(P_succ, checkpoint 0) + 24`, from the FrozenZero arm's own zero-update DEV evaluation. The gate is net G-Cap > max(500, Bar (i)).

**Decision bars:** inherited verbatim from the grandparent §7.6 and §7.7, through the parent:
- `F_I = max(24, 2·sqrt(b+c))` on the matched P_succ set;
- the T2, complement, ablation and ITT floors;
- the §7.12.2 Q1 gate, with B3_init re-measured on this lineage;
- fresh transfer before any KEEP.

**No cross-sheet attribution.** The contrast is only Relation vs FrozenZero on the same A8 initialization. No claim compares these numbers with the PrimeZeta or seeded lineages. Table-vs-table comparisons are named follow-ups (§9).

**Known risk, stated before measurement:** codebook moves during learning can carry tokens out of their predecessor-structured cells. In the grandparent's seeded Dose 1, single-coordinate moves eroded P_succ admission from 2,034 to 225 / 205. The inherited Q1 gate measures whether admission survives on this lineage; it is not assumed.

## 4. Base artifact

None. Both arms build a fresh model through `PreparedDriver::new_model` with the pinned A8 table. Routes, gates, seed and epoch come from the device initializer, and only the codebook is replaced, through `Device::from_snapshot`'s GPU validation.

## 5. Data split

The parent's pins, unchanged:
- TRAIN / DEV `3dee858f…` / `0919e90f…`, and masks;
- fresh V2 tokens / mask / seal `0b7e754c…` / `a1cda529…` / `df5d074a…`.

The A8 table is built from TRAIN only. The fresh draw stays sealed until the paired DEV decision.

## 6. Source

The training path binds a registered sheet by codebook initialization (`registered_sheet` in `attention_learning.rs`). Today every `PinnedTableV1` maps to the parent's SHA. The change:
- **an exact mapping** by pinned table file SHA:
  - seeded keeps the grandparent sheet;
  - PrimeZeta `83c1380a…` maps to the parent sheet;
  - A8 `6600bdb1…` maps to this sheet;
  - **any other `PinnedTableV1` is refused.** Without this refusal, S8 or any other injective table would bind the PrimeZeta sheet outside its D22 scope. `registered_sheet` therefore returns a `Result`.
- tests:
  - this file's SHA and title prefix, next to `registered_sheet_constants_match_committed_sheets`;
  - an A8-pinned plan refuses the parent's sheet, and a PrimeZeta-pinned plan refuses this one;
  - an unregistered pinned table is refused, and that test fails with the refusal branch removed.
  - Test fixtures that use placeholder table SHAs change to registered ones. The rule does not change to fit them.

Re-pin rule: any change to this sheet, including the TEST FITNESS line, changes the constant, and so changes `driver_source_sha256`. The constant is set in the same PR, after FIT, and the release packet binds the driver hash through `execution.driver_source_sha256`.

## 7. Steps

1. FIT this sheet with a fresh-context adversarial reviewer subagent (owner direction 6255848646), posting every finding.
2. Land the Source change with its tests, after exact-head review.
3. Stage the two configs: Relation and FrozenZero, seed 1. Each is the parent's runbook config (`docs/labs/injective-attention-runbook-2026-10-19/evidence/injective-{relation,frozen-zero}-seed1.json`) with only the table pin changed to A8. The Relation config equals the pre-flight's `a8.json`.
4. PM dispatch with an exact costed plan, then Casey's typed D40 release.
5. At the new head:
   - re-run the parent's inherited pre-flight battery;
   - run the §3 gates, including the reproduction gate and the unpinned G-Init comparison;
   - re-measure per-batch cost.

   Record every pin and value in the release packet.
6. Run the Q1 gate, then the full dose, as in the parent.
7. Make the DEV decision per §7.7, then score fresh transfer only on H_geom.

## 8. Stop rule

- The parent's §7.9 and Q1 stops, unchanged, plus the §3 pre-run gates and the reproduction gate.
- A stop is for diagnosis (D31), never a mechanism negative.
- Owner caps and the 30 GiB disk floor are hard stops.

## 9. Cost and minimum test power

**Cost:** the parent's §8 assumption-based estimates apply unchanged, because the lineage differs only in table values. That is about $11.4 for the full dose (16,404 batches per arm, both arms in parallel) and about $2.8 for a Q1 stop at 4,096 batches, at the historical $1.894/h. They are planning figures, not measured costs and not caps. The release packet measures per-batch throughput and all-in cost at the new head.

**Minimum test power:** inherited:
- `B3_I ≥ 200` with the MDE rule (grandparent §7.12.6);
- the 14,287-row DEV panel (P_succ 3,847);
- fresh transfer for any KEEP.

Seed 2 is reproducibility only.

**Out of scope, as named follow-up sheets:**
- A8 vs S8 under training (relation vs packing after learning);
- A8 vs the PrimeZeta lineage at a matched arm;
- a K = 16 attention path, which would need source item (c) for the attention validator.

TEST FITNESS: FIT (owner direction 6255848646: a fresh-context adversarial reviewer subagent replaces cross-lab FIT for Claude lanes. The review of draft `6b4e9762` returned FIT-WITH-FIXES. All five findings are applied (a post-FIT wording correction, from DeepSeek's #2751 erratum, states that the unpinned G-Init comparison was not run and is a pre-run gate here; no bar changed): the exact sheet mapping that refuses unregistered pinned tables (§6); §3 items inherited with A8 pins, and negative (b) rebuilt from A8; the 133 full-panel upper-bound label and the FrozenZero P_succ control term; the reproduction gate on both arms with each value's row basis; and the float-free citation. Registration releases no training: the run needs the §6 source merged, then PM dispatch, then Casey's typed D40 release.)
