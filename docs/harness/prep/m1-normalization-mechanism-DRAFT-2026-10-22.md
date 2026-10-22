# Scale-matched E8 orbit-root normalization on the discrete M1 trunk (`OrbitRootSnapNormV1`)

- **Status:** draft pre-registration, `TEST FITNESS: PENDING` (see the end of the sheet).
  - Training and GPU evaluation remain HELD. A run needs a non-author `TEST FITNESS: FIT` review (CHARTER §4.5), the owner's bar confirmation on `#2028` (`https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6338596024`), `m1-trunk-assembly-and-training`, PM dispatch, and Casey's typed D40 release.
- **Line:** M1 discrete normalization and trunk stability (#2029, item `m1-normalization-mechanism` #2842). **Lab:** antigravity.
- **Parent:** [`m1-attention-a8-address-cells-learning-2026-10-19.md`](m1-attention-a8-address-cells-learning-2026-10-19.md) (the A8 address-structured cells sheet).
  - **What changes from the parent:** evaluates scale-matched integer E8 orbit-root normalization `OrbitRootSnapNormV1` (`crates/uor-r4-discrete/src/normalization_mechanism.rs`, `NormalizationMode::ScaleMatchedSnapV1`) against two matched controls (`UnitSnapControlV1` and `UnnormalizedControlV1`) at equal parameter count.
- **Applicable owner decisions:** D11, D21, D22, D23, D24, D25, D28, D29, D31, D32, D37, D40.
  - Geometry only (`Cl(0,6)` 64-frame signed-permutation group acting on the 16-root $E_8$ orbit $\mathcal{O}_{16} \subset \{-2, 0, +2\}^8$).
  - Zero normalized exponentials or squashing functions (CHARTER §3.2, D23).
  - Zero dense matrix multiplications or GEMM (CHARTER §3.3, D25).
  - Zero floating-point types or literals on the discrete path (D32).
  - Exports to D11 as 4-bit root indices (`u8` in `0..16`, `4 * B` bits total across $B$ blocks) and optional per-block frame alignment candidates (`NormFrameTable::export_words()`), requiring zero multiply instructions at serving time.

## 1. Why: multi-term integer residual accumulation drifts in norm and distorts un-scaled snapping

In the discrete M1 substrate (`crates/uor-r4-discrete`), each 8D block root $\rho_r \in \mathcal{O}_{16}$ ($r \in \{0, \dots, 15\}$) is a signed permutation of $(2, 2, 0, 0, 0, 0, 0, 0)$ with exact invariants $\|\rho_r\|_1 = 4$, $\|\rho_r\|_2^2 = 8$, $\|\rho_r\|_\infty = 2$, and support size $2$. When $n \ge 1$ signed root contributions are summed across tree aggregation or residual stages ($x_b = \sum_{k=1}^n s_k \rho_{r_k}$, $s_k \in \{-1, +1\}$):
- Without normalization (`UnnormalizedControlV1`), block norms grow up to $\|x_b\|_1 \le 4n$, $\|x_b\|_2^2 \le 8n^2$, and $\|x_b\|_\infty \le 2n$, pushing endpoint cell distances (`address_distance`) outside the canonical 5-value set $\{0, 4, 8, 12, 16\}$.
- With un-scaled `snap_1` (`UnitSnapControlV1`), comparing a $4$-term sum $3\rho_{r_0} + \rho_{r_{\text{orth}}}$ against un-scaled reference roots $1 \cdot \rho_r$ ties on $L_1$ distance ($12 = 12$) and selects the 1-vote minority root $r_{\text{orth}}$ by Hamming distance ($2 < 4$), whereas multiplicity-matched `snap_4` ($n = 4$) strictly selects the 3-vote majority root $r_0$ ($L_1 = 8 < 24$, verified in `scale_equivariance_on_orbit_multiples`).
- Separately, the norm-preserving Hamilton gate (`#2879`) satisfies $|q \otimes u| = |u|$ outside its $10^{-12}$ division-by-zero guard core and therefore cannot rescale an accumulated residual sum back to unit block norm.

`OrbitRootSnapNormV1` (`crates/uor-r4-discrete/src/normalization_mechanism.rs`) enforces exact unit-root block invariants after every aggregation or residual stage using integer arithmetic only:
$$C_n(x, r) = \bigl(d_1(x, n\rho_r),\; d_H(x, n\rho_r),\; r\bigr), \qquad r^\star = \arg\min_{r \in \{0,\dots,15\}} C_n(x, r), \qquad \hat{x}_b = A_{a_b}\rho_{r^\star} \in \mathcal{O}_{16}.$$

## 2. Construction and matched controls

Three arms share the identical A8 initialization (`address_structured_v1`, file SHA-256 `6600bdb1306dbaea4d77f5bf021e717a6bef791e44c1ec7d7c9bb9f0e8cd3cd1`), `Spec` ($V=4096$, $B=4$, $C=192$, $K=8$), `ReadSpec::Relation { kept: 8, sweeps: 1 }`, objective A (`NearestAddressMarginV1`), and proposal schedule:

1. **Treatment (`N_snap` — `NormalizationMode::ScaleMatchedSnapV1`):** Applies multiplicity-matched `snap_n` projection onto $\mathcal{O}_{16}$ after each tree aggregation (`n = 3` or `found`) and residual stage (`n = terms`), keeping $\|\hat{x}_b\|_1 = 4$, $\|\hat{x}_b\|_2^2 = 8$, $\|\hat{x}_b\|_\infty = 2$, and `address_distance` $\in \{0, 4, 8, 12, 16\}$.
2. **Un-scaled snap control (`C_unscaled` — `NormalizationMode::UnitSnapControlV1`):** Applies `snap_1` ($n = 1$) after multi-term accumulation, keeping the output on $\mathcal{O}_{16}$ while omitting multiplicity scaling.
3. **Un-normalized accumulation control (`C_raw` — `NormalizationMode::UnnormalizedControlV1`):** Passes the raw integer residual sum $u + \sum_l g_l m_l \in \mathbb{Z}^8$ directly to `address_distance` and `token_distance` without snapping back to $\mathcal{O}_{16}$.

## 3. Bar

- **Pre-run finite controls (CPU, landed in `crates/uor-r4-discrete/src/normalization_mechanism.rs`):**
  1. Exact invariant norms and bounded coordinate range (`L1 = 4`, `L2^2 = 8`, `L_infty = 2`, `support = 2`) across arbitrary inputs and multiplicities (`norm_stability_and_bounded_range`).
  2. Idempotence / two-pass stability $\operatorname{Norm}_1(\operatorname{Norm}_n(x)) = \operatorname{Norm}_n(x)$ (`idempotence_two_pass_stability`).
  3. Scale equivariance $\operatorname{Norm}_n(n\rho_r) = \rho_r$ and majority-vote preservation on $3\rho_0 + \rho_{\text{orth}}$ where un-scaled `snap_1` flips to the minority root (`scale_equivariance_on_orbit_multiples`).
  4. Deterministic tie-breaking on $x = \mathbf{0}$ and across all $\binom{16}{2} = 120$ root-pair sums (`deterministic_tie_breaking_on_zero_and_midpoints`).
  5. Exact equivariance under all 64 $\mathrm{Cl}(0,6)$ signed-axis frames and 4-bit-per-block (`16`-bit at $B=4$) D11 export (`cl06_frame_equivariance_across_all_64_actions`).
  6. Multi-stage residual stability across $S = 16$ stages vs. linear norm growth (`L1 = 68`) and address-distance drift (`> 16`) in `UnnormalizedControlV1` (`multistage_residual_accumulation_vs_unnormalized_drift`).
- **Pre-registered trunk bar (proposed via decision card on `#2028`, CHARTER §5.2):**
  - On the 14,287-row DEV panel (`P_succ` 3,847 copy-successor rows), across $\ge 2$ seeds:
    1. **Structural norm stability:** 100% of stage outputs in `N_snap` satisfy $\|\hat{x}_b\|_1 = 4$, $\|\hat{x}_b\|_2^2 = 8$, $\|\hat{x}_b\|_\infty = 2$, and endpoint `address_distance` $\in \{0, 4, 8, 12, 16\}$.
    2. **Top-1 & admission contrast:** `N_snap` beats `C_raw` (and is non-inferior to or beats `C_unscaled`) on paired one-sided McNemar top-1 contrast by $b - c \ge F_I = \max(24, \lceil 2\sqrt{b + c}\rceil)$ in both seeds while passing the Q1 admission floor ($\ge 1{,}351 / 3{,}847$ on `P_succ`).
    3. **Fresh transfer:** Any DEV `KEEP` must replicate on the sealed fresh V2 draw (`0b7e754c…`) before mechanism promotion.

## 4. Base artifact

- **Trunk / proxy:** Because the full Stage 3 trunk is not yet assembled (`m1-trunk-assembly-and-training`), this test runs on the active discrete M1 proxy trunk (`crates/uor-r4-discrete`, `ReadSpec::Relation { kept: 8, sweeps: 1 }`, A8 pinned codebook `6600bdb1…`), and is re-executed unchanged once `m1-trunk-assembly-and-training` lands.

## 5. Data split

- Inherits the pinned M1 splits from `m1-attention-a8-address-cells-learning-2026-10-19.md`:
  - TRAIN / DEV tokens `3dee858f…` / `0919e90f…` and masks;
  - Sealed fresh V2 draw tokens / mask / seal `0b7e754c…` / `a1cda529…` / `df5d074a…`.
- Training never touches DEV or fresh V2 rows (CHARTER §3.10, D22 §4).

## 6. Steps

1. Land `crates/uor-r4-discrete/src/normalization_mechanism.rs` with its 6 finite CPU controls (`cargo test -p uor-r4-discrete --lib`).
2. Obtain a non-author `TEST FITNESS: FIT` review on this sheet and owner confirmation of the §3 numeric bar on `#2028`.
3. Wire `NormalizationMode` (`ScaleMatchedSnapV1`, `UnitSnapControlV1`, `UnnormalizedControlV1`) into the GPU read/trunk kernel (`discrete.cu`) and qualify its PTX admission (`build.rs` `admit_ptx`).
4. After PM dispatch and Casey's typed D40 release, run `N_snap`, `C_unscaled`, and `C_raw` across seeds 1 and 2 on GPU.
5. Score the paired contrasts on DEV (`P_succ` and all 14,287 rows) and, if DEV passes, on the sealed fresh V2 draw.

## 7. Stop conditions

- Any PTX admission failure, checkpoint-0 reproduction mismatch, or out-of-range state is an `INSTRUMENT_UNREACHABLE` stop for diagnosis (D31), never a mechanism negative.
- Self-set step or wall-clock estimates are planning figures, not stop triggers; extend and renew rather than cancel a healthy run.
- Hard stops are the owner spend/pod caps ($\le 4$ pods, $\le \$8.00/\text{h}$ across all labs), the 30 GiB / 128 MiB disk floor, or a Q1 admission collapse gate failure.

## 8. Cost and minimum test power

- **Cost (planning estimate at $\$1.894/\text{h}$ on a single leased GPU via `scripts/pod/uor-pod`):**
  - `preparation`: $\$0.00$ (host metadata and finite CPU controls on laptop).
  - `run`: $\approx \$11.40$ per seed pair (`N_snap` + controls, 16,404 batches per arm, $\approx 6\text{ h}$ wall time per seed pair; $\approx \$22.80$ for 2 seeds).
  - `controls`: included in the paired `C_unscaled` and `C_raw` runs above.
  - `evaluation`: $\approx \$0.40$ (DEV panel checkpoints plus sealed fresh V2 draw on `KEEP`).
  - `retries`: $\approx \$2.80$ (one Q1 diagnostic retry allowance within owner caps).
  - `storage`: $< 64\text{ MiB}$ per retained snapshot set, respecting the 128 MiB / 30 GiB stop margin.
- **Minimum test power (D22 §3):**
  - Seeds: $2$ paired seeds (seeds 1 and 2).
  - Rows: $14{,}287$ DEV rows ($3{,}847$ `P_succ` rows) plus sealed fresh V2 transfer draw ($\ge 200$ rows).
  - Decisive run: YES ($\ge 2$ seeds and $\ge 200$ fresh rows, trained in from step 0 rather than bolted on).

TEST FITNESS: PENDING
