# REPORT: antigravity-48, legal-construction-exact-solve

Lab: antigravity. Order: [ORDER.md](ORDER.md).
References #2828.

## What did not work, first

1. **Evaluating `certify_lagrangian_dual_bound` on `Dy<120>`-rounded dual multipliers (`d.dual_y`).**
   - In Unit 5 (`antigravity-47`), `certify_lagrangian_dual_bound` evaluated the Lagrangian dual bound $L(y)$ at the `Dy<120>`-computed dual vector `d.dual_y` (`-21.029262371913034...`), which lost `~5.08e-6` relative to the exact `delta_hull_lp` optimum (`-21.029267452019339...`) due to 120-bit fixed-point rounding in `B⁻¹`.
   - Extracting the exact unscaled rational simplex multiplier vector $\pi \in \mathbb{Q}^m$ directly from `blp::certify_basis_with_dual(&delta_lp, &d.delta_lp_basis)` ($\pi_r = \pi_{\text{scaled}, r} \cdot \text{row\_scale}_r / c_{\text{scale}}$) and setting $y = -\pi \ge 0$ eliminates 100% of that rounding loss: `certify_lagrangian_dual_bound(&p, &y)` matches the exact 1,070-digit rational optimum of `delta_hull_lp(p)` byte-for-byte (`dual_bound_matches_delta_hull_lp: true`).
2. **Single-schedule margin tightening (`(-13, 5/4)`) with coordinate-order single-coefficient polish.**
   - Unit 5's single margin schedule (`bump_pad = 2^-13`, `bump_scale = 5/4`) with coordinate-order `0..n` polish stopped at `-94651962270606439 / 4503599627370496` (`-21.016958...`) because early coordinates consumed shared guard slack before higher-gain coordinates were visited, and single-coefficient moves could not trade a small guard relaxation on one coefficient for a larger objective gain on another.
   - Replacing coordinate order with steepest-descent single-coefficient polish plus a 2-coefficient (pair) exact rational polish pass and evaluating 4 warm-started progressive margin schedules (`(-17, 17/16)`, `(-16, 9/8)`, `(-15, 9/8)`, `(-13, 5/4)`) improves the certified discrete incumbent to **`-94662137592886459 / 4503599627370496` (`-21.019217408256145...`)**, reducing the remaining primal-dual optimality gap $U - L(y)$ by **18.35%** (`0.0123094...` -> `0.010050043763194...`).

## What changed

- **Code (`+609 / -239` lines across 4 files):**
  - [`docs/labs/legal-construction-exact-solve/solver/src/blp.rs`](../../../../labs/legal-construction-exact-solve/solver/src/blp.rs) (`+38 / -3` lines):
    - Added `certify_basis_with_dual(lp, basis) -> Result<(LpResult, Vec<BigRational>), String>`, returning both the certified primal `LpResult` and the exact rational unscaled dual multiplier vector $\pi \in \mathbb{Q}^m$.
  - [`docs/labs/legal-construction-exact-solve/solver/src/search.rs`](../../../../labs/legal-construction-exact-solve/solver/src/search.rs) (`+525 / -211` lines):
    - Added `ReducedCostFixing` and `certify_reduced_cost_fixing(p, y, upper_bound)`, which certifies over `BigRational` that $y \ge 0$ and $L(y) \le U$ and provably eliminates every discrete code $q \in -7..=7$ whose exact reduced-cost penalty $(g_i - (G^T y)_i)\delta_i(q) - m_i$ exceeds $U - L(y)$.
    - Upgraded `lp_guided_discrete_solve::<P>` with a 4-schedule warm-started Dual Simplex margin loop (`342` total pivots across all 4 schedules on `attempt2`) and steepest-descent single-coefficient + 2-coefficient (pair) exact rational polish.
  - [`docs/labs/legal-construction-exact-solve/solver/src/bin/root-search.rs`](../../../../labs/legal-construction-exact-solve/solver/src/bin/root-search.rs) (`+32 / -11` lines):
    - Updated `root-search` to extract the exact rational dual multiplier vector $y = -\pi$ via `blp::certify_basis_with_dual`, certify `lagrangian_dual_bound` at $y$, and emit `"reduced_cost_fixing"` (`fixed_coeffs`, `binary_or_fixed_coeffs`, `eliminated_codes`, `total_codes`, `gap`, `dual_bound_matches_delta_hull_lp`) in the JSON summary.
  - [`docs/labs/legal-construction-exact-solve/solver/src/bin/single-move.rs`](../../../../labs/legal-construction-exact-solve/solver/src/bin/single-move.rs) (`+14 / -14` lines):
    - Resolved the 3 inherited `clippy::needless_range_loop` warnings so `cargo clippy --all-targets -- -D warnings` passes cleanly across the entire crate.
- **Tests (`+45 / -0` lines across 1 file):**
  - [`docs/labs/legal-construction-exact-solve/solver/tests/search_certify.rs`](../../../../labs/legal-construction-exact-solve/solver/tests/search_certify.rs) (`+45 / -0` lines):
    - Added `reduced_cost_fixing_rejects_negative_multiplier_and_invalid_upper_bound` (deliberate break: planted negative multiplier $y_r < 0$, wrong-length $y$, and invalid upper bound $U < L(y)$ are rejected).
    - Extended `dual_search_and_lp_guided_discrete_solve_respect_exact_bounds` across all 9 finite controls and 60 random instances to verify that `certify_lagrangian_dual_bound(p, &exact_y)` equals `delta_hull_lp`'s exact optimal objective with zero gap and that the exact branch-and-bound optimal assignment `bb_res.assignment` lies inside `rcbt.surviving_codes[i]` for every coefficient $i$.
- **Docs (`+10 / -0` lines across 2 files, plus `ORDER.md` and `REPORT.md`):**
  - [`docs/labs/legal-construction-exact-solve/README.md`](../../../../labs/legal-construction-exact-solve/README.md) (`+9 / -0` lines): added Unit 6 section.
  - [`docs/research/notebook/legal-construction.md`](../../../../research/notebook/legal-construction.md) (`+1 / -0` lines): added Unit 6 row.
- **Generated / Other:** `0` lines.

## Measurements

All measurements were taken on the laptop (`macOS`, release profile for `root-search`, sha256-verified read-only `attempt2` directory at `local/claude/legal-case/codex-direct-legal-centered-20261009/attempt2`, 384/384 files verified):

- **Command:**
  ```sh
  cd docs/labs/legal-construction-exact-solve/solver
  CARGO_TARGET_DIR=/tmp/antigravity-legal-target ~/.cargo/bin/cargo build --release --locked --bin root-search
  /tmp/antigravity-legal-target/release/root-search /Users/casey.allard/uor-r4/local/claude/legal-case/codex-direct-legal-centered-20261009/attempt2
  ```
- **Results (two consecutive runs produced identical non-timing output):**
  1. **Exact rational dual multiplier extraction & Lagrangian bound (`blp::certify_basis_with_dual` + `search::certify_lagrangian_dual_bound`):**
     - `delta_hull_lp` (`380` rows, `1,920` variables) certified in `19,053 ms` (`19.05 s`).
     - `lagrangian_dual_bound` evaluated at $y = -\pi \ge 0$ equals the exact 1,070-digit rational `delta_hull_lp` optimum identically (`dual_bound_matches_delta_hull_lp = true`, `-21.029267452019339...`).
  2. **Multi-schedule warm-started LP-guided discrete solve + pair polish (`search::lp_guided_discrete_solve::<120>`):**
     - Solve time: **`8,929 ms` (`8.93 s`)**, **`342` total Dual Simplex pivots** across 4 schedules.
     - Moved coefficients: **`1,467` of `1,920`**.
     - Guard verification: **`active_guards = 0` of `380`**, **`guard_min = 145873908319 / 562949953421312` (`~ +0.00025912 > 0`)**, `full_guard_vector_matches = true`.
     - Exact rational objective: **`-94662137592886459 / 4503599627370496` (`~ -21.019217408256145`)** (improving on Unit 5's `-94651962270606439 / 4503599627370496` by `-0.00225937`).
  3. **Exact rational Reduced-Cost Bound Tightening (`search::certify_reduced_cost_fixing`):**
     - Certified primal-dual optimality gap $U - L(y) \approx 0.010050043763194$ (`0.0478%` relative gap, down **18.35%** from Unit 5's `0.0123094...`).
     - Eliminated codes: **`4,859` of `28,800` discrete codes (`16.87%`)** provably eliminated.
     - Fixed coefficients: **`81` of `1,920` coefficients** provably fixed to a single optimal destination code (`|surviving_codes[i]| == 1`).
     - Binary-or-fixed coefficients: **`137` of `1,920` coefficients** reduced to at most 2 surviving codes (`|surviving_codes[i]| <= 2`).

## Checks run at the head

- `CARGO_TARGET_DIR=/tmp/antigravity-legal-target ~/.cargo/bin/cargo fmt -- --check`: PASS.
- `CARGO_TARGET_DIR=/tmp/antigravity-legal-target ~/.cargo/bin/cargo clippy --all-targets -- -D warnings`: PASS (`0` warnings across all targets).
- `CARGO_TARGET_DIR=/tmp/antigravity-legal-target ~/.cargo/bin/cargo test --locked`: PASS (all 13 test targets green, including `reduced_cost_fixing_rejects_negative_multiplier_and_invalid_upper_bound`, `dual_search_and_lp_guided_discrete_solve_respect_exact_bounds`, and `no_floats`).
- `uor-harness loop report --check`: PASS.
- `uor-harness loop pr-check --local`: PASS.
- `uor-harness loop gates --full`: PASS (`11/11` green).

## Skills used

- `uor-research-architect` (`.agents/skills/uor-research-architect/SKILL.md`)
- `solution-integrity` (`/Users/casey.allard/.gemini/config/skills/solution-integrity/SKILL.md`)
- `scope-guard` (`/Users/casey.allard/.gemini/config/skills/scope-guard/SKILL.md`)

## Still open

- **Closing the remaining `0.01005004` (`0.0478%`) discrete optimality gap on `attempt2`:** Unit 6 eliminates 100% of the dual rounding loss between `certify_lagrangian_dual_bound` and `delta_hull_lp`, provably eliminates 4,859 of 28,800 discrete codes (fixing 81 coefficients to a single code), and improves the exact discrete incumbent to `-94662137592886459 / 4503599627370496` (`~ -21.01921741`), leaving a `0.01005004` (`0.0478%`) gap on `#2828` (`legal-construction-exact-solve`, status stays `ready`).

## Plain summary

In Unit 5, our basis-free Lagrangian lower-bound certificate evaluated its bound using 120-bit fixed-point dual multipliers, losing about `0.000005` to rounding compared to the exact convex-hull linear program, and our discrete rounding solver stopped at an objective of `-21.016958`, leaving a `0.012309` (`0.0585%`) gap between the best known discrete move and the theoretical lower bound.

In Unit 6, we first extracted the exact rational dual multipliers directly from the certified basis (`blp::certify_basis_with_dual`), which makes the basis-free Lagrangian lower bound match the 1,070-digit rational convex-hull LP optimum identically (`-21.02926745...`) with zero rounding loss. Second, we upgraded the LP-guided discrete solver to run four warm-started margin schedules followed by steepest-descent single-coefficient and two-coefficient (pair) exact rational polish, improving the certified discrete move to `-94662137592886459 / 4503599627370496` (`-21.01921741...`) with strictly positive slack on all 380 guards and shrinking the remaining optimality gap by 18.35% to `0.010050` (`0.0478%`). Third, using the exact rational dual multipliers and the new discrete upper bound, we added an exact Reduced-Cost Bound Tightening certificate (`search::certify_reduced_cost_fixing`) that provably rules out **4,859 of the 28,800 discrete destination codes (16.87%)** and locks **81 of the 1,920 coefficients** to their single proven optimal discrete value.
