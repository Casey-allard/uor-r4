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

**Pre-run instrument gates.** Zero updates, run on the GPU in the parent's pre-flight battery. A failure is `INSTRUMENT_UNREACHABLE` (D31), not a mechanism negative.
1. **G-Cap (D39 item 2):** the coverage census on the actual checkpoint-0 artifact reports a DEV alias ceiling above Bar (i). The table-level ceiling above is 14,287.
2. **G-Init:** the checkpoint-0 snapshot's table equals the pinned table word for word, and GPU snapshot validation passes.
3. **B3_init(P_succ) is reported before training.** If it is below 500, the Q1 gate's first predicate cannot be met, and the run is not dispatched without a successor decision. With 256 occupied cells, admission at K = 8 must be measured rather than assumed.

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
3. Run the parent's pre-flight battery, plus G-Cap, G-Init and B3_init (§3).
4. Run the Q1 gate, then the full dose, as in the parent.
5. Make the DEV decision per the parent §7.7, then score fresh transfer only on H_geom.

## 7. Stop rule

- The parent's §7.9 and Q1 stops, unchanged, plus the §3 pre-run gates.
- A stop is for diagnosis (D31), never a mechanism negative.
- Owner caps and the disk floor are hard stops.

## 8. Cost

On `rows_v1`, Relation at K = 8 takes about 0.07 s per proposal (#2658 timing). 16,404 batches is therefore about 20 min per arm, plus the parent's evaluations. Both arms on one 2×5090 pod take about 45 min, about $2. Seed 2, if released, doubles that.

## 9. Minimum test power

- **Inherited:** `B3_I ≥ 200` with the MDE rule (parent §7.12.6), the 14,287-row DEV panel (P_succ 3,847), and fresh transfer for any KEEP.
- Seed 2 is reproducibility only.
- **Out of scope, as a named follow-up sheet:** the PrimeZeta-vs-null attribution (tangent and normal nulls, both exportable with `prime-zeta-codebook`), and injective vs seeded initialization at a matched arm.

TEST FITNESS: PENDING
