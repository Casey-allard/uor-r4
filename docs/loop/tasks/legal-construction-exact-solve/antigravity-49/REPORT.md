# REPORT: antigravity-49, legal-construction-exact-solve

Lab: antigravity. Order: [ORDER.md](ORDER.md).
References #2828.

## What did not work, first

1. **Coordinate-order (`0..n`) single-coefficient polish and single-compensator (`k = 1`) pair polish alone.**
   - In Unit 6 (`antigravity-48`), single-coefficient polish iterated in coordinate order `i = 0..n` and applied moves immediately, allowing early coordinates with tiny objective gains to consume shared guard slack before higher-gain coordinates were visited; meanwhile, the pair polish only tried a single compensator `i2` with step size `±1`, rejecting improving moves `i1` whenever `i2` fixed the primary violated guard `r0` but left 1–2 other tight guards slightly negative.
   - Sorting single-coefficient candidates across all 1,920 coordinates by `gain_dy` ascending (steepest descent), expanding compensator steps to `±1, ±2`, and adding a 2-compensator (`(i1, i2, i3)`) exact rational repair pass whenever the best partial compensator `i2` leaves at most 2 violated rows, combined with warm-started One-Sided Simplex Diving across 6 rounds of 45 basic variables, improves the certified discrete incumbent from `-94662137592886459 / 4503599627370496` (`-21.01921741...`) to **`-94666121428139195 / 4503599627370496` (`-21.02010201...`)**.

## What changed

- **Code (`+385 / -84` lines across 2 files):**
  - [`docs/labs/legal-construction-exact-solve/solver/src/search.rs`](../../../../labs/legal-construction-exact-solve/solver/src/search.rs) (`+384 / -84` lines):
    - Extended `ReducedCostFixing` and `certify_reduced_cost_fixing` with exact rational Guard-Slack Slab bounds (`guard_max_slack: Vec<Option<BigRational>>` and `constrained_guards: usize`), proving from Weak Duality that every guard row $r$ with $y_r > 0$ at any guard-feasible assignment with objective $\le U$ satisfies $0 \le G_r \cdot \delta \le (U - L(y)) / y_r$.
    - Upgraded `lp_guided_discrete_solve::<P>` with warm-started One-Sided Simplex Diving (`dive_batch = 45` over 6 rounds before the margin loop), steepest-descent single-coefficient polish (`single_sweep`), and multi-compensator (`(i1, i2)` pair and `(i1, i2, i3)` triple) exact rational repair polish.
  - [`docs/labs/legal-construction-exact-solve/solver/src/bin/root-search.rs`](../../../../labs/legal-construction-exact-solve/solver/src/bin/root-search.rs) (`+1 / -0` lines):
    - Added `"constrained_guards"` to the `"reduced_cost_fixing"` JSON output.
- **Tests (`+22 / -0` lines across 1 file):**
  - [`docs/labs/legal-construction-exact-solve/solver/tests/search_certify.rs`](../../../../labs/legal-construction-exact-solve/solver/tests/search_certify.rs) (`+22 / -0` lines):
    - Verified `constrained_guards` and `guard_max_slack` on all 9 finite controls and 60 random instances, asserting that at the exact branch-and-bound optimal solution $\delta^*$ (`bb_res.assignment`), every guard row $r$ with `guard_max_slack[r] = Some(max_s)` satisfies $0 \le G_r \cdot \delta^* \le \text{max\_s}$.
- **Docs (`+14 / -0` lines across 2 files, plus `ORDER.md` and `REPORT.md`):**
  - [`docs/labs/legal-construction-exact-solve/README.md`](../../../../labs/legal-construction-exact-solve/README.md) (`+13 / -0` lines): added Unit 7 section.
  - [`docs/research/notebook/legal-construction.md`](../../../../research/notebook/legal-construction.md) (`+1 / -0` lines): added Unit 7 row.
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
  1. **Warm-started Simplex Diving + multi-compensator LP-guided discrete solve (`search::lp_guided_discrete_solve::<120>`):**
     - Solve time: **`23,588 ms` (`23.59 s`)**, **`2,274` total Dual Simplex pivots** across all schedules and diving rounds.
     - Moved coefficients: **`1,465` of `1,920`**.
     - Guard verification: **`active_guards = 0` of `380`**, **`guard_min = 247363824223 / 562949953421312` (`~ +0.00043941 > 0`)**, `full_guard_vector_matches = true`.
     - Exact rational objective: **`-94666121428139195 / 4503599627370496` (`~ -21.02010200849505`)** (improving on Unit 6's `-94662137592886459 / 4503599627370496` by `-0.00088460`).
  2. **Exact rational Reduced-Cost Bound Tightening & Guard-Slack Slab Certificate (`search::certify_reduced_cost_fixing`):**
     - Certified primal-dual optimality gap $U - L(y) \approx 0.009165443524291$ (`0.04358%` relative gap, breaking below `0.01` for the first time and down **8.80%** from Unit 6's `0.01005004`).
     - Eliminated codes: **`5,194` of `28,800` discrete codes (`18.03%`, `+335` over Unit 6)** provably eliminated.
     - Fixed coefficients: **`89` of `1,920` coefficients (`+8` over Unit 6)** provably fixed to a single optimal destination code (`|surviving_codes[i]| == 1`).
     - Binary-or-fixed coefficients: **`139` of `1,920` coefficients** reduced to at most 2 surviving codes (`|surviving_codes[i]| <= 2`).
     - Slab-constrained guards: **`67` of `380` guard rows** have $y_r > 0$ and are provably trapped in the exact rational slab $0 \le G_r \cdot \delta \le (U - L(y)) / y_r$.

## Checks run at the head

- `CARGO_TARGET_DIR=/tmp/antigravity-legal-target ~/.cargo/bin/cargo fmt -- --check`: PASS.
- `CARGO_TARGET_DIR=/tmp/antigravity-legal-target ~/.cargo/bin/cargo clippy --all-targets -- -D warnings`: PASS (`0` warnings across all targets).
- `CARGO_TARGET_DIR=/tmp/antigravity-legal-target ~/.cargo/bin/cargo test --locked`: PASS (all 13 test targets green, including `search_certify` and `no_floats`).
- `uor-harness loop report --check`: PASS.
- `uor-harness loop pr-check --local`: PASS.
- `uor-harness loop gates --full`: PASS (`11/11` green).

## Skills used

- `uor-research-architect` (`.agents/skills/uor-research-architect/SKILL.md`)
- `solution-integrity` (`/Users/casey.allard/.gemini/config/skills/solution-integrity/SKILL.md`)
- `scope-guard` (`/Users/casey.allard/.gemini/config/skills/scope-guard/SKILL.md`)

## Still open

- **Closing the remaining `0.00916544` (`0.04358%`) discrete optimality gap on `attempt2`:** Unit 7 pushes the certified discrete incumbent to `-94666121428139195 / 4503599627370496` (`~ -21.02010201`), eliminates `5,194 / 28,800` discrete codes (`18.03%`), fixes `89` coefficients to a single code, and traps `67 / 380` active dual guards in exact rational slabs $0 \le G_r \cdot \delta \le (U - L(y)) / y_r$, leaving a `0.00916544` (`0.04358%`) gap on `#2828` (`legal-construction-exact-solve`, status stays `ready`).

## Plain summary

At the continuous linear-programming optimum of the 1,920-variable, 380-guard convex-hull relaxation, at least 1,540 of the 1,920 coefficients already sit at legal discrete endpoints (`q = ±7`), while at most 380 basic coefficients take fractional values between legal quarter-steps. Rounding all 380 fractional coefficients at once adds up their rounding errors across the 380 guards.

In Unit 7, we added warm-started One-Sided Simplex Diving (which rounds batches of the 45 closest fractional basic coefficients at a time and re-optimizes the remaining basic coefficients along the guard boundaries) together with steepest-descent single-coefficient polish and a three-coefficient (`(i1, i2, i3)`) exact rational repair pass. On `attempt2`, this improves the certified legal discrete assignment to `-94666121428139195 / 4503599627370496` (`-21.02010201...`) with strictly positive slack on all 380 guards, bringing the remaining optimality gap below `0.01` for the first time (`0.009165`, or `0.04358%`). Using this tighter upper bound, our exact Reduced-Cost Bound Tightening certificate now eliminates **5,194 of the 28,800 discrete codes (`18.03%`)**, locks **89 of the 1,920 coefficients** to a single proven optimal value, and proves an exact upper bound $0 \le G_r \cdot \delta \le (U - L(y)) / y_r$ on the slack of all **67 active dual guards**.
