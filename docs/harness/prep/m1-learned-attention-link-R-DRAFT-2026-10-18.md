# Learned attention link under the rank-hinge objective (Relation + R) — successor DRAFT

- **Status:** DRAFT successor pre-registration. **Not registered**, and TEST FITNESS is not requested.
  - No source change, no compute, $0.
  - It is registered **only if** the fast-path admission redirect (K16-A vs K16-R, `m1-discrete-rung0-admission-redirect-2026-10-15.md`, owner #2029 6250410397) shows **K16-R passing `G-Adm` (`target_admitted_rows ≥ 4,894`) and `G-Occ` on both seeds at cursor 16,404**, under that sheet's definitions. Bar (i), Bar (ii) and G-Guard are **not** required for this sheet's registration: they are ranking bars on Route, and this sheet asks a different question.
  - **Withdrawal branches:**
    - `ADMISSION_STILL_CAPPED_AT_K16` or `OCCUPANCY_COLLAPSE_TRIGGERED`: withdrawn in favour of the redirect's named successor, the full-vector address metric.
    - `K16_A_KEEP_R_NULL_AT_SCOPE`: withdrawn. If A also keeps admission at K16, the evidence points to width, not R, and this sheet's choice of R is unsupported. A width-only successor (Relation + A at K16) would be drafted instead.
  - **Route admission does not transfer to Relation.** The redirect runs Route with `CanonicalVectorThenReadV1`; this sheet runs Relation with `InterleavedReadEvery8V1`. A redirect pass is a precondition for drafting, not evidence that admission survives here. That is why this sheet keeps its own Q1 gate and occupancy guard (§1).
- **Line:** M1 learned attention (#2029). **Lab:** claude, drafting under the shared track #2029 6250966192.
- **Parent:** `m1-learned-attention-link-2026-10-17.md` (FIT design, #2574).
  - Its fast-path run stopped at the registered Q1 gate in both arms: P_succ admission 2,034 → 225 (Relation) / 205 (frozen-zero). The outcome-seat verdict (6248429872) calls this a configured admission collapse under objective A.
  - **Every section of the parent not changed below is inherited verbatim**, including §7.6 bars, §7.7 decision rule, §7.8 instrument validity, §7.12.1 pre-flight, §7.12.3 fresh order, §7.12.5–§7.12.9 and the D22 scope.
- **Authority:** the parent §7.12.4 names this exact successor: "If the Q1 gate shows admission collapse, the next configuration is **R = `OccupiedCellRankHingeV1`**, not a larger A dose. That is a new pre-registration/owner decision."
- **Rules:** D22, D23/D25/D32 (integer, no softmax or matmul), D24 (GPU only), D29, D11. D37 is not used; this sheet runs no float.

## 1. What changes from the parent

| Item | Parent (Dose 1) | This successor |
|---|---|---|
| Objective (both arms) | `NearestAddressMarginV1` (A) | **`OccupiedCellRankHingeV1` (R)**: writer `loss = 8·T + width·R` with `R = max(0, rank − kept_cells)` over occupied cells (`discrete.cu:329–340`) |
| Writer admission width `spec.kept_cells` (`s[3]`) | 8 | **The redirect's passing K, fixed at registration.** The default is K = 16, the redirect's tested width. K = 8 is admissible only if the redirect shows R preserving admission at 8, which it does not test |
| Read width `Relation { kept, sweeps }` (`s[9]`, `s[10]`) | `kept: 8, sweeps: 1` | **Unchanged** |
| Counterfactual path | `rows_v1` (fast-path rerun) | `rows_v1` (exact parity under R: #2658 attempt 5, active R > 0, 134/134) |
| Admission trajectory | Q1@4096 only | **Plus** a zero-apply B3 evaluation every 256 batches on DEV (rules in §1.3) |
| Arms, seeds, schedule, init, data, fresh protocol | Relation LearnV1 vs FrozenZeroV1; seed 1 first; `InterleavedReadEvery8V1`; `ActiveLeafV1`; Dose 1 pins | **Unchanged** |

**The two widths are independent.** The source check is `discrete.cu:125–247` and `307–333`:
- The writer's `s[3]` admits the K address-nearest **occupied cells** for token prediction.
- The Relation read's `s[9]` keeps the 8 best **context nodes** (leaves strictly before the query), ordered by relation score, then frame distance, then Hamming, then recency, then node id.

So a target cell at writer rank 9–16 is **not hidden from the read**. The read never selects writer cells; it changes the query payload `q`, which the writer then scores. The writer's prediction still picks by token distance across all K admitted cells.

### 1.1 Q1 gate (§7.12.2), plus an occupancy guard

- **Predicates (unchanged).** Stop for diagnosis if any of these holds:
  - `B3_Q1(P_succ) < 500`;
  - `B3_Q1(P_succ) < 0.5 × B3_init`;
  - zero GateRelation and GateCombine sensitivity.
- **`B3_init` is re-measured** at the registered K and reported before training, together with `occupied_cells` and mean admitted tokens per row. At K = 16 with ≤ 16 occupied cells at initialization, `B3_init(P_succ)` may equal all 3,847 rows. The half-initial threshold is then about 1,924.
- **Occupancy guard (new; it mirrors the redirect's G-Occ).**
  - **Why it is needed.** R is zero on every row whose target cell is admitted. A move that packs the vocabulary into ≤ K occupied cells is therefore free under R, and B3 reaches 3,847 trivially.
  - **The guard.** At Q1 and at the final cursor 16,404, a B3 value is **INVALID** if either:
    - `occupied_cells ≤ K`; or
    - `occupied_cells < max(2K, ⌈0.5 × occ_init⌉)`, which is `max(32, …)` at K = 16.
  - **Effect of INVALID.**
    - It counts as `Q1_STOP_FOR_DIAGNOSIS` (D31), never a pass, whatever the B3 value.
    - Mean admitted tokens per row is reported alongside, so that admission which survives only through crowding is visible.
  - **Why R needs it.** R pushes back only after admission is lost (target rank > K); it does not prevent the loss. Until then, training is token-only. The guard is what makes a B3 pass under R non-vacuous.

### 1.2 Per-family ledger at Q1

Inherited from the parent: VectorBlock 3,584, GateCombine 237, GateRelation 275, because the schedule is unchanged.

### 1.3 Trajectory rules (diagnostic only)

- **What it is.** A zero-apply DEV evaluation every 256 batches. It reports `B3(P_succ / P_comp / full)`, `target_admitted_rows`, `occupied_cells`, mean admitted tokens per row, and the target cell's writer rank bucket (1–8, 9–K, > K).
- **No zero-apply evaluation changes state.** For every trajectory point (and for Q1 and the final evaluation), the hashes of table, routes, gates, cursor and RNG state match before and after the evaluation. A mismatch is an instrument fault (D31).
- **It never selects.** The final checkpoint is fixed at cursor 16,404. The trajectory never feeds:
  - the power-up dose (B = 32,808);
  - the seed-2 release;
  - the choice of K;
  - any stop.

  The registered stops are the parent's §7.9 stops and the Q1 gate above.
- **It never touches the fresh seal.** The trajectory reads DEV only. The fresh draw stays sealed per the parent §7.12.3 (#2582).

### 1.4 No cross-sheet attribution

- No claim compares this sheet's numbers with the parent's Dose 1 numbers.
- No survival of admission is attributed to R alone: K and the objective change together, so the K change is confounded.
- Attribution between R and width is the redirect's question (`KEEP_K16_R` vs `K16_A_KEEP_R_NULL_AT_SCOPE`), not this sheet's.
- This sheet's comparison is only **Relation vs FrozenZero at the same (R, K)**.

## 2. Why this is the right next test

- **The collapse is shared, not relational.** Both parent arms collapsed together, so it comes from the **shared** vector and combine learning under A, not from relation learning.
- **R targets the collapsed quantity.** It penalizes the target cell's occupied-cell rank beyond `kept_cells`, exactly what collapsed. The occupancy guard (§1.1) closes the degenerate way of satisfying it.
- **The question is the parent's.** Is the learned Relation link visible **once admission survives**? It is asked at a configuration where the parent's evaluability precondition can hold.
- **One paired contrast.** Relation vs FrozenZero at the same (R, K) answers that question. No A@K16 arm is needed here, because the redirect supplies the width-vs-objective contrast.

## 3. Source changes required (separately reviewed before registration)

1. **`attention_learning.rs` `validate_config` and `scope`.** These currently admit only `kept_cells == 8` and objective A. They need two explicit scope strings, one per arm, admitting exactly this sheet's (R, K) pair:
   - `m1-attention-learning-relation-r-k16-seed1-batches0..16404`;
   - `m1-attention-learning-relation-frozen-zero-r-k16-seed1-batches0..16404`.

   Nothing else is admitted.
2. **Trajectory hook.**
   - It writes the zero-apply DEV evaluation of §1.3 every 256 batches as sealed per-checkpoint receipts, with the before/after state hashes.
   - It has no checkpoint-resume semantics.
3. **Occupancy telemetry.** `occupied_cells` and `admitted_token_count_sum` are emitted at initialization, Q1, every trajectory point and the final cursor. The writer already computes occupancy under R (`discrete.cu:329–341`, `telemetry[2]`); this item surfaces it in the receipts.
4. **Consumer and scorer.** AGY's paired-DEV scorer accepts objective R and the registered K, and applies the §1.1 INVALID rule. Its bars and floors are unchanged.

**Dependencies:**
- The parent's §7.12.3 deferred fresh scoring has already merged (#2582).
- **Redirect receipts C-R1..C-R6 are a prerequisite at Relation/ActiveLeaf.**
  - The #2658 row-path parity proves that `rows_v1` replays the lane loop exactly under R at Relation. It does **not** cover:
    - the K16 baseline-guard widening (C-R6);
    - the K16 objective-R driver admission.
  - Those receipts are re-run at this sheet's configuration as part of the parent's pre-flight battery (§7 step 4).

## 4. Bar

- **Inherited verbatim** from the parent §7.6 and §7.7: `F_I = max(24, 2·sqrt(b+c))`, the T2, complement, ablation and ITT floors, and fresh transfer before KEEP.
- **Change:** a B3 value marked INVALID by §1.1 cannot satisfy any bar.
- The trajectory adds no bar.

## 5. Base artifact

None. Both arms use a fresh `Device::new_with_objective(spec_K, Relation{8,1}, R, seed, 0)` with `ActiveLeafV1`, built identically.

## 6. Data split

- **The parent's pins, unchanged:**
  - train/DEV `3dee858f…`/`0919e90f…`, and masks;
  - fresh V2 tokens/mask/seal `0b7e754c…`/`a1cda529…`/`df5d074a…`.
- The fresh draw stays sealed until the paired DEV decision.

## 7. Steps

1. The redirect reports. Register only on K16-R `G-Adm` + `G-Occ` on both seeds (see Status). Otherwise take the named withdrawal branch.
2. Fix K. Implement §3 and review it at the exact head.
3. TEST FITNESS by non-author seats. With Kimi offline, these are AGY and DeepSeek. Then owner release.
4. Run the parent's pre-flight battery at the new configuration, including the C-R1..C-R6 receipts.
5. Report `B3_init` and occupancy, then run the Q1 gate with the occupancy guard, then the full dose, as in the parent.
6. Make the DEV decision per the parent §7.7, then score fresh transfer only on H_geom.

## 8. Stop rule

- The parent's §7.9 and Q1 stops are unchanged, plus the §1.1 INVALID rule at Q1.
- A stop is for diagnosis (D31), never a mechanism negative.
- Owner caps and the disk floor are hard stops.

## 9. Cost

- **Training:** on `rows_v1`, Relation at K16 takes about 0.13 s per proposal (#2658 timing, K16). Over 16,404 proposals that is about 36 min per arm.
- **Trajectory:** about 64 evaluations of about 9 s each (224 DEV forwards each), or about 10 min.
- **Total:** both arms on a 2×5090 pod take about 50 min, about $2.
- Seed 2, if released, doubles that.

## 10. Minimum test power

- **Inherited:** `B3_I ≥ 200` with the MDE rule (parent §7.12.6), the 14,287-row DEV panel (P_succ 3,847), and fresh transfer for any KEEP.
- Seed 2 is reproducibility only.
- A B3 value marked INVALID by §1.1 does not count toward `B3_I`.

TEST FITNESS: PENDING
