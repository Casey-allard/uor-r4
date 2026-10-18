# Learned attention link from an injective codebook (PrimeZeta initialization) — DRAFT

- **Status:** DRAFT pre-registration. **Not registered.** TEST FITNESS is requested from non-author seats only after its source PR merges.
  - No compute, $0.
  - Training stays HELD. Running this needs FIT, PM dispatch and owner release.
- **Line:** M1 learned attention (#2029). **Lab:** claude, self-directed under D39 item 7 (intent 6253613139).
- **Parent:** `m1-learned-attention-link-2026-10-17.md` (FIT design, #2574).
  - **Exactly one variable changes: the initial codebook.** Every section of the parent not changed below is inherited verbatim. That includes:
    - arms, schedule, read, objective and K;
    - the §7.6 bars, the §7.7 decision rule and §7.8 instrument validity;
    - the §7.12.1 pre-flight, the §7.12.3 fresh order (#2582) and §7.12.5–§7.12.9;
    - the D22 scope.
- **This is also the deferred downstream sheet.** The PrimeZeta construction sheet (`m1-prime-zeta-complex-routing-2026-10-14.md`, FIT #2433) deferred "downstream learned/corpus attribution … to the first downstream training sheet". This is that sheet.
- **Rules:**
  - D22, D23/D25/D30 (no softmax, squashing or matmul), D24 (GPU only), D29, D11.
  - D37 and the October 14 zeta exception: f64 exists only in the offline constructor module, before the integer table. The driver, kernels and training are integer.
  - D39 items 2 and 3.
- **D22 scope of this sheet (replaces the parent's):** PrimeZeta V4096 / B4 `pinned_table_v1` initialization (`table_sha256 e9e03bcd…`), with the parent's arms, read (`Relation{8,1}`), schedule, objective A, K = 8 and `ActiveLeafV1` read initialization.
  - A negative here closes only this configuration, never the mechanism.
  - The parent's `+1-leaf-combine` initialization scope does not describe this configuration and is not reused.
- **Identity pins are all new** and are recorded in the release packet: source `driver_source_sha256 a328e63f9f73119328bccc2a0f9d2c036914ceba725d5afc18f129001741724c` (main at #2692), both configs, lineage, the table file SHA, and the built binary. No parent receipt is inherited as PASS.
- **Supersedes as the preferred next attention configuration:** the conditional Relation+R draft (`m1-learned-attention-link-R-DRAFT-2026-10-18.md`). That draft stays on file. It addresses admission through the objective; this sheet addresses the aliasing that precedes it.

## 1. Why: the read has never had distinguishable keys

The W0 alias census (D39 item 2) found that the default seeded initializer gives **16 distinct token vectors at checkpoint 0**: 256 tokens share each vector, and no token has its own identity.

The Relation read keeps the best context nodes by relation score over those payloads. At checkpoint 0 it can therefore separate only 16 classes of key. Every attention run so far had to diversify the codebook first, through single-coordinate moves: 2.4% of attempts were accepted, and the census reached 264 distinct vectors at a full Q4 dose. The same moves eroded admission (Dose 1 Q1: P_succ 2,034 → 225 / 205). **No run so far has isolated whether the read can learn.**

The qualified PrimeZeta constructor gives an injective codebook. Its census, measured with `coverage-census` (#2678) on the real 14,287-row DEV panel against the exported table:

| | Seeded checkpoint 0 (parent) | PrimeZeta initialization (this sheet) |
|---|---|---|
| Distinct vectors | 16 | **4,096** |
| Occupied writer cells | ≤ 16 | **256 / 256** |
| DEV alias ceiling (G-Cap) | 1,986 | **14,287 / 14,287** |
| DEV targets that can win a tie | 4 / 674 | **674 / 674** |
| Exact tie mass (rows) | 12,301 | **0** |

Table: `prime-zeta-codebook` V4096 B4. `table_sha256` is `e9e03bcd95b78632a7f18d1b27c07d37f0ba82c52eee692c37da011eadac8804`, bit-exact for seeds 1 and 2.

**This comparison characterizes the instrument; it is not evidence about trained read behaviour.** The seeded column is the parent lineage's checkpoint 0, and the PrimeZeta column is table-level. The artifact-level values are measured again on the actual checkpoint-0 snapshot by the §3 gates.

## 2. What changes from the parent

| Item | Parent | This sheet |
|---|---|---|
| `codebook_initialization` | seeded device default | **`pinned_table_v1`**: the PrimeZeta V4096 B4 table above, pinned by file SHA-256 and bound into the lineage |
| Everything else | Relation LearnV1 vs FrozenZeroV1; `InterleavedReadEvery8V1`; `ActiveLeafV1`; `Relation{8,1}`; objective A; K = 8; `rows_v1`; seed 1 first; Dose 1 data pins | **unchanged** |

**Why objective A and K = 8 stay.**
- Changing one variable keeps attribution clean.
- The parent's collapse came from occupancy growth: 16 → 189 cells under A. Here occupancy starts at 256, so that growth dynamic is absent.
- Whether admission survives is measured, not assumed, by the new pre-run gates in §3.

## 3. Bar

**Pre-run instrument gates.** Zero updates, run on the GPU at the new source head, after the parent's inherited pre-flight battery has been **re-run at that head**. Old receipts are not inherited. A gate failure is `INSTRUMENT_UNREACHABLE` (D31), not a mechanism negative. All values below are recorded in the release packet.

1. **G-Init.** On the actual checkpoint-0 artifact of each arm:
   - the snapshot's `table` equals the pinned file's table word for word;
   - its word digest equals `e9e03bcd…`;
   - GPU snapshot validation (`discrete_validate`) passes;
   - routes, gates, seed and epoch equal the unpinned initializer's for the same config;
   - a deliberately corrupted copy of the table (one non-root word) is refused with status 6. This is the negative control.
2. **G-Occ (artifact level).** The GPU token-cell export (`attention-export-token-cells`, which admits these Relation configs) runs on each checkpoint-0 artifact, and `coverage-census --token-cells` reports:
   - `occupied_cells` (must exceed 16; 256 is expected);
   - the occupancy histogram;
   - crowding;
   - **within-cell tie mass** (must be 0, with no split alias group).

   These are measured on the artifact, never inferred from the table.
3. **G-Cap, raw and net of admission (D39 item 2).**
   - Raw: `dev_alias_ceiling_rows` from `coverage-census` on the artifact.
   - Net: P_succ rows whose target is an alias representative **and** whose target cell is admitted at checkpoint 0. The admission term comes from the zero-update checkpoint-0 DEV evaluation's per-row admitted cells, the same evaluation that yields B3_init.
   - Both values are reported. Net G-Cap must be at least the Q1 evaluability floor (500 P_succ rows), or the run is `INSTRUMENT_UNREACHABLE`.
4. **B3_init(P_succ)** is measured on the GPU and recorded before dispatch. If it is below 500, the run is not dispatched, and a successor decision is needed: with 256 occupied cells, admission at K = 8 may be low from the start. This is the parent's hard gate, run on this lineage.

**Decision bars:** inherited verbatim from the parent §7.6 and §7.7:
- `F_I = max(24, 2·sqrt(b+c))`;
- the T2, complement, ablation and ITT floors;
- the Q1 gate (§7.12.2), with `B3_init` re-measured on this lineage;
- fresh transfer before any KEEP.

**No cross-sheet attribution.** No claim compares this sheet's numbers with the parent's Dose 1 numbers. The contrast is only Relation vs FrozenZero on the same initialization. Initialization vs initialization is a separate question; see §9.

## 4. Base artifact

None. Both arms build a fresh model through `PreparedDriver::new_model` with the pinned table. Routes, gates, seed and epoch come from the device initializer, and only the codebook is replaced, through `Device::from_snapshot`'s GPU validation.

## 5. Data split

The parent's pins, unchanged:
- train/DEV `3dee858f…` / `0919e90f…`, and masks;
- fresh V2 tokens/mask/seal `0b7e754c…` / `a1cda529…` / `df5d074a…`.

The fresh draw stays sealed until the paired DEV decision.

## 6. Steps

1. Merge the source (pinned initialization and `prime-zeta-codebook`) after exact-head review. Note: this changes `driver_source_sha256` on main; the K16 packet stays bound to its own build.
2. Write the two configs (Relation and FrozenZero, seed 1). Then run TEST FITNESS with non-author seats, PM dispatch and owner release.
3. At the new head, re-run the parent's inherited pre-flight battery (old receipts are not reused). Then run G-Init (with its negative control), G-Occ, G-Cap (raw and net) and B3_init (§3). Then re-measure the per-batch cost. Record every pin and value in the release packet.
4. Run the Q1 gate, then the full dose, as in the parent.
5. Make the DEV decision per the parent §7.7, then score fresh transfer only on H_geom.

## 7. Stop rule

- The parent's §7.9 and Q1 stops, unchanged, plus the §3 pre-run gates.
- A stop is for diagnosis (D31), never a mechanism negative.
- Owner caps and the disk floor are hard stops.

## 8. Cost

**Assumption-based planning estimates; changed-lineage throughput and cost remain unmeasured.** The #2658 0.07 s figure is a row-parity proposal microbenchmark (`timing.qualified=false`), not a training batch, and it is **not** used.

The historical [second paired `rows_v1` Q1 attempt (Dose2)](../../labs/attention-rows-q1-diagnosis-2026-10-18/README.md), source `20d158ae` / ELF `e7cb6197`, used Relation `{8,1}`, K = 8, `InterleavedReadEvery8V1`, `ActiveLeafV1` and 64 rows per batch. It is distinct from the original owner-stopped 904/1,065-batch attempt and from this sheet's new injective lineage:

- both historical arms reached 4,096 batches, in parallel on one 2×5090 pod;
- **5,361 s was the resource lifetime**, covering setup/build, model work, custody and rental, not a measured batch-timing series;
- [retained timing clarification6253995798](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6253995798) distinguishes dispatch-to-terminal-observation wall envelopes from GPU/per-batch timings. It establishes no measured upper bound for this changed lineage.

Dividing that resource lifetime by 4,096 gives about **1.31 s per batch only as a planning assumption**. The following extrapolations assume the new lineage has the same amortized resource lifetime per batch, both arms remain parallel, and the historical quoted pod rate of $1.894/h applies. Setup, occupancy, evaluation and custody costs need not scale this way; these are neither measured costs nor upper bounds.

| Run | Assumption-based pod-time estimate | Assumption-based cost estimate at $1.894/h |
|---|---|---|
| Full dose, 16,404 batches per arm, both arms in parallel | about 6.0 h | **about $11.4** |
| A Q1 stop at 4,096 batches | about 1.5 h | **about $2.8** |
| Seed 2, if released | assumes the same additional run time | assumes the same additional run cost |

The release packet must measure per-batch throughput and all-in cost for the new head and lineage rather than inherit these estimates. Registered owner caps and Q1 stops are unchanged; these planning figures grant no budget or compute release and are not stop rules. Extensions follow the standing owner rule.

## 9. Minimum test power

- **Inherited:** `B3_I ≥ 200` with the MDE rule (parent §7.12.6), the 14,287-row DEV panel (P_succ 3,847), and fresh transfer for any KEEP.
- Seed 2 is reproducibility only.
- **Out of scope, as a named follow-up sheet:** the PrimeZeta-vs-null attribution (tangent and normal nulls, both exportable with `prime-zeta-codebook`), and injective vs seeded initialization at a matched arm.

TEST FITNESS: PENDING
