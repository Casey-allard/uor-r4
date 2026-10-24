# REPORT: antigravity-50, legal-construction-exact-solve

Lab: antigravity. Order: [ORDER.md](ORDER.md).
References #2828.

## What did not work, first

1. **Checking remaining guard violations (`rem_count`) only on `viol_rows` in multi-compensator polish and running 6 dive rounds without infeasible-batch rollback.**
   - In Unit 7 (`antigravity-49`), `PartialComp` in `run_polish` counted remaining violated guards after `(i1, i2)` only across the subset `viol_rows` originally violated by `i1`, missing any tight guard row $r' \notin \text{viol\_rows}$ pushed below zero by `i2` itself; consequently, the selected `rem_r0` for the 3-coefficient (`i3`) repair pass rarely named the true bottleneck guard row. Furthermore, Simplex Diving stopped after 6 rounds of `dive_batch = 45` and did not snapshot/restore the LP state when a dive batch became infeasible.
   - Precomputing `after1_guard` and `after12_guard` across all $m = 380$ guards so `rem_r0` and `rem3_r0` always identify the true most-violated remaining guard row across all 380 guards, retaining the top-2 partial 2-coefficient moves, adding 4-coefficient (`(i1, i2, i3, i4)`) repair, adding direct greedy guard-violation repair when $1 \le \text{n\_viol} \le 15$, and extending Simplex Diving to 12 rounds (`dive_batch in {25, 30, 45}`) with full state rollback improves the certified discrete incumbent from `-94666121428139195 / 4503599627370496` (`-21.02010201...`) to **`-94671994445923515 / 4503599627370496` (`-21.02140560...`)**.

## What changed

- **Code (`+621 / -126` lines across 2 files):**
  - [`docs/labs/legal-construction-exact-solve/solver/src/search.rs`](../../../../labs/legal-construction-exact-solve/solver/src/search.rs) (`+616 / -126` lines):
    - Added `GapDecomposition` and `certify_gap_decomposition(p, y, codes)` to certify in exact rational arithmetic (`BigRational`) the algebraic split $g \cdot \delta(\text{codes}) - L(y) = \text{reduced\_cost\_loss} + \text{complementary\_slackness\_loss}$ and count `zero_reduced_cost_coeffs`.
    - Upgraded `lp_guided_discrete_solve::<P>` with 12-round Warm-Started Simplex Diving with infeasible-batch rollback across `dive_batch in {25, 30, 45}`, direct greedy single-step guard-violation repair when $1 \le \text{n\_viol} \le 15$, and a two-tier full-guard-scan multi-compensator (`k = 1` pair, `k = 2` triple, `k = 3` quad) exact rational repair polish.
  - [`docs/labs/legal-construction-exact-solve/solver/src/bin/root-search.rs`](../../../../labs/legal-construction-exact-solve/solver/src/bin/root-search.rs) (`+5 / -0` lines):
    - Added `"reduced_cost_loss"`, `"complementary_slackness_loss"`, `"zero_reduced_cost_coeffs"`, and `"gap_identity_holds"` to the `"reduced_cost_fixing"` JSON output.
- **Tests (`+18 / -0` lines across 1 file):**
  - [`docs/labs/legal-construction-exact-solve/solver/tests/search_certify.rs`](../../../../labs/legal-construction-exact-solve/solver/tests/search_certify.rs) (`+18 / -0` lines):
    - Verified `certify_gap_decomposition` rejection of invalid inputs and exact identity `gap == reduced_cost_loss + complementary_slackness_loss` across all 9 finite controls and 60 random instances.
- **Docs (`+15 / -0` lines across 2 files, plus `ORDER.md` and `REPORT.md`):**
  - [`docs/labs/legal-construction-exact-solve/README.md`](../../../../labs/legal-construction-exact-solve/README.md) (`+14 / -0` lines): added Unit 8 section.
  - [`docs/research/notebook/legal-construction.md`](../../../../research/notebook/legal-construction.md) (`+1 / -0` lines): added Unit 8 row.
- **Generated / Other:** `0` lines.

## Measurements

All measurements were taken on the laptop (`macOS`, release profile for `root-search`, sha256-verified read-only `attempt2` directory at `local/claude/legal-case/codex-direct-legal-centered-20261009/attempt2`, 384/384 files verified):

- **Command:**
  ```sh
  cd docs/labs/legal-construction-exact-solve/solver
  CARGO_TARGET_DIR=/tmp/antigravity-legal-unit8 ~/.cargo/bin/cargo build --release --locked --bin root-search
  /tmp/antigravity-legal-unit8/release/root-search /Users/casey.allard/uor-r4/local/claude/legal-case/codex-direct-legal-centered-20261009/attempt2
  ```
- **Results (two consecutive runs produced identical non-timing output):**
  1. **Deep Warm-Started Simplex Diving + full-guard 4-coefficient LP-guided discrete solve (`search::lp_guided_discrete_solve::<120>`):**
     - Pivots: **`3,679` total Dual Simplex pivots** across all 12 schedules and diving rounds (`409.4 s`).
     - Moved coefficients: **`1,461` of `1,920`**.
     - Guard verification: **`active_guards = 0` of `380`**, **`guard_min = 18161425 / 274877906944` (`~ +0.00006607 > 0`, tightened `6.65×` from Unit 7's `+0.00043941`)**, `full_guard_vector_matches = true`.
     - Exact rational objective: **`-94671994445923515 / 4503599627370496` (`~ -21.021405604209846`)** (improving on Unit 7's `-94666121428139195 / 4503599627370496` by `-0.00130360`).
  2. **Exact rational Lagrangian Gap Decomposition (`search::certify_gap_decomposition`) and RCBT (`search::certify_reduced_cost_fixing`):**
     - Certified primal-dual optimality gap $U - L(y) \approx 0.007862069883520$ (`0.03739%` relative gap, down **14.22%** from Unit 7's `0.00916544`).
     - Exact gap split (`gap_identity_holds = true`): **`reduced_cost_loss ≈ 0.002290041445924` (`29.13%` of gap)** + **`complementary_slackness_loss ≈ 0.005572028437596` (`70.87%` of gap)**, with **`1,890` of `1,920` coefficients (`98.44%`) at exact zero reduced-cost penalty** (`zero_reduced_cost_coeffs = 1,890`; only 30 coefficients pay any reduced-cost penalty).
     - Eliminated codes: **`5,821` of `28,800` discrete codes (`20.21%`, `+627` over Unit 7)** provably eliminated.
     - Fixed coefficients: **`102` of `1,920` coefficients (`+13` over Unit 7)** provably fixed to a single optimal destination code (`|surviving_codes[i]| == 1`).
     - Binary-or-fixed coefficients: **`152` of `1,920` coefficients (`+13` over Unit 7)** reduced to at most 2 surviving codes (`|surviving_codes[i]| <= 2`).
     - Slab-constrained guards: **`67` of `380` guard rows** have $y_r > 0$ and are provably trapped in the tighter exact rational slab $0 \le G_r \cdot \delta \le (U - L(y)) / y_r$.

## Checks run at the head

- `CARGO_TARGET_DIR=/tmp/antigravity-legal-unit8 ~/.cargo/bin/cargo fmt -- --check`: PASS.
- `CARGO_TARGET_DIR=/tmp/antigravity-legal-unit8 ~/.cargo/bin/cargo clippy --all-targets -- -D warnings`: PASS (`0` warnings across all targets).
- `CARGO_TARGET_DIR=/tmp/antigravity-legal-unit8 ~/.cargo/bin/cargo test --locked`: PASS (all 13 test targets green, including `search_certify` and `no_floats`).
- `uor-harness loop report --check`: PASS.
- `uor-harness loop pr-check --local`: PASS.
- `uor-harness loop gates --full`: PASS (`11/11` green).

## Skills used

- `uor-research-architect` (`.agents/skills/uor-research-architect/SKILL.md`)
- `solution-integrity` (`/Users/casey.allard/.gemini/config/skills/solution-integrity/SKILL.md`)
- `scope-guard` (`/Users/casey.allard/.gemini/config/skills/scope-guard/SKILL.md`)

## Still open

- **Closing the remaining `0.00786207` (`0.03739%`) discrete optimality gap on `attempt2`:** Unit 8 pushes the certified discrete incumbent to `-94671994445923515 / 4503599627370496` (`~ -21.02140560`), eliminates `5,821 / 28,800` discrete codes (`20.21%`), fixes `102` coefficients to a single code, and proves that `1,890 / 1,920` coefficients (`98.44%`) are already at zero reduced cost while `70.87%` (`0.00557203`) of the remaining `0.00786207` gap resides in guard complementary slackness $y^T (G \delta)$ on the 67 active dual guards, leaving `#2828` (`legal-construction-exact-solve`) at status `ready`.

## Plain summary

In Unit 7, when a candidate move `i1` was partially repaired by a second coefficient `i2`, our search only checked whether `i2` fixed the guards that `i1` had violated—not whether `i2` itself pushed another tight guard below zero. In Unit 8, we fixed the multi-compensator polish to scan all 380 guards after each partial move, extended it up to 4-coefficient (`(i1, i2, i3, i4)`) exact rational repairs, added direct guard-violation repair when a margin iteration has at most 15 violated guards, and extended Warm-Started Simplex Diving to 12 rounds with full rollback whenever a dive batch becomes infeasible.

We also added an exact rational Lagrangian gap decomposition certificate (`certify_gap_decomposition`) that splits the remaining primal-dual gap $U - L(y)$ into per-coefficient reduced-cost loss and per-guard complementary-slackness loss. On `attempt2`, the certified discrete incumbent improves to `-94671994445923515 / 4503599627370496` (`-21.02140560...`) with minimum guard slack tightened `6.65×` to `+0.00006607`, shrinking the remaining optimality gap by **14.22%** to **`0.007862` (`0.03739%`)**. Our exact certificate proves that **1,890 of the 1,920 coefficients (`98.44%`) now pay zero reduced-cost penalty**, eliminates **5,821 of the 28,800 discrete codes (`20.21%`)**, and locks **102 coefficients** to a single proven optimal destination code.
