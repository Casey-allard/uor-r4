# REPORT: antigravity-47, legal-construction-exact-solve

Lab: antigravity. Order: [ORDER.md](ORDER.md).
References #2828.

## What did not work, first

1. **Primal `Dy<120>` simplex from `δ = 0` (Unit 3b baseline and refactor-dust fix alone).**
   - Adding `degenerate_pivots` and `bland_switches` counters to `SearchStats` exposed why Unit 3b's primal `Dy<120>` search stalled past 48,025 pivots in 7,800 s: every 200 pivots, `Search::refactor` recomputed `x_B = B⁻¹(b - N x_N)` from scratch, leaving tiny `Dy<120>` rounding dust (`lo[bv] - x[bv] > 0` or `x[bv] - u[bv] > 0`) on basic variables. `Search::ratio` treated those negative numerators as zero-step (`ratio = 0`) blocking pivots, causing long stretches of degenerate basis swaps without moving nonbasic variables.
   - Clamping refactor rounding dust in `Search::ratio` (`if x[bv] <= lo[bv] { Dy::ZERO } else { ... }`) eliminated the artificial zero-step refactor stalls on primal simplex, but primal simplex from `δ = 0` still had to traverse thousands of degenerate zero-RHS vertices because all 380 guard rows have `b_j = 0` (`J·0 = 0`) and `bb::relaxation`'s 60 fractional disjunction rows share `k_{i_f} = 0` at the origin.
2. **Recomputing `x_B` from scratch inside the Dual Simplex BFRT candidate loop or gating bound flips on `ratio < next_ratio`.**
   - An initial Dual Simplex BFRT prototype recomputed `x_B` from scratch inside the candidate flip loop and gated bound flips on `ratio < next_ratio`, which caused dual cycling when two candidates tied on dual ratio (`ratio = 0`).
   - Replacing that with incremental basic-variable updates on each bound flip (`x[basis[k]] -= B⁻¹[k][i] * a_{ij} * Δx_j`) and flipping every bounded candidate whose violation reduction satisfies `|α_{rj}|(u_j - l_j) + tol < rem_viol` resolved the cycling completely: the 380-row projected LP converges in **268 pivots and 285 bound flips (`1.99 s`)** with only **2 degenerate pivots** and **0 Bland switches**.
3. **Dividing by the Bareiss pivot divisor inside the back-substitution loop in `blp::certify_basis`.**
   - Dividing intermediate back-substitution terms by the Bareiss divisor inside the inner loop caused intermediate rational denominators to blow up on the 440×440 basis solve (`125.66 s` per certificate call).
   - Performing exact integer back-substitution (`z_perm[i] = (det(B)·r[i] - Σ_{j>i} U[i][j]·z_perm[j]) / U[i][i]`) and dividing by `det(B)` once per row at the end, combined with unit-row/unit-column elimination while the Bareiss divisor is `1`, non-zero dual filtering in `col_pi`, and summing nonbasic columns before basic columns in `certify_basis`, cut the 440×1,960 exact rational certificate time from `125.66 s` to **`16.89 s`** (`7.4×` speedup).
4. **Unchecked positive objective after margin-tightening on tiny random instances.**
   - On one synthetic random instance in `dual_search_and_lp_guided_discrete_solve_respect_exact_bounds`, tightening `b_r` on violated guards pushed the rounded discrete candidate to a positive objective (`g·δ > 0`), which was caught by the test assertion `disc.objective <= 0`.
   - When the post-polish discrete objective is positive, `lp_guided_discrete_solve` now resets to the legal zero move `δ = 0` (`g·0 = 0`, `J·0 = 0`) and runs the exact rational single-coefficient polish pass from `δ = 0`, guaranteeing `objective <= 0` on every instance.

## What changed

- **Code (`+1,422 / -39` lines across 3 files):**
  - [`docs/labs/legal-construction-exact-solve/solver/src/search.rs`](../../../../labs/legal-construction-exact-solve/solver/src/search.rs) (`+1,218 / -19` lines):
    - Added `degenerate_pivots` and `bland_switches` counters to `SearchStats` and clamped refactor rounding dust in `Search::ratio`.
    - Added float-free `Dy<P>` bounded-variable Dual Simplex with the Bound-Flipping Ratio Test (`search_dual::<P>`, `dual_simplex_run::<P>`), which starts at the cost-optimal nonbasic bounds (`Upper` when `c_j < 0`, `Lower` when `c_j >= 0`) with basic slacks (`π = 0 >= 0`), requiring zero Phase-1 work.
    - Added exact algebraic projection of `bb::relaxation`'s 3-row fractional disjunction blocks at `q0_i = ±7` (`try_project_disjunction_lp`) and exact basis lifting from the 380×1,920 projected LP back to the 440×1,960 root LP (`lift_projected_found`).
    - Added `delta_hull_lp(p)` (the 380×1,920 convex-hull `δ`-relaxation), `certify_lagrangian_dual_bound(p, y)` (basis-free `O(m·n)` exact rational Weak Duality lower-bound certificate), and `lp_guided_discrete_solve::<P>(p)` (warm-started `Dy<P>` Dual Simplex margin-tightening + exact rational single-coefficient polish + independent from-scratch `BigRational` guard/objective verification).
  - [`docs/labs/legal-construction-exact-solve/solver/src/blp.rs`](../../../../labs/legal-construction-exact-solve/solver/src/blp.rs) (`+118 / -12` lines):
    - Optimized `bareiss_solve` with unit-row and unit-column elimination while the Bareiss divisor is `1` and exact integer back-substitution before a single final division by `det(B)` per row; optimized `certify_basis` by filtering non-zero dual multipliers in `col_pi` and summing nonbasic columns before basic columns.
  - [`docs/labs/legal-construction-exact-solve/solver/src/bin/root-search.rs`](../../../../labs/legal-construction-exact-solve/solver/src/bin/root-search.rs) (`+86 / -8` lines):
    - Extended `root-search` to report `search_degenerate_pivots`, `search_bland_switches`, the certified `delta_hull_lp` objective (`blp::certify_basis`), the basis-free `lagrangian_dual_bound`, and the `lp_guided_discrete_solve` assignment summary in its JSON output.
- **Tests (`+82 / -0` lines across 1 file):**
  - [`docs/labs/legal-construction-exact-solve/solver/tests/search_certify.rs`](../../../../labs/legal-construction-exact-solve/solver/tests/search_certify.rs) (`+82 / -0` lines):
    - Added `lagrangian_dual_bound_rejects_negative_multiplier_and_wrong_length` (deliberate break: planted negative dual multiplier `y_r < 0` and wrong-length multiplier vector are rejected).
    - Added `dual_search_and_lp_guided_discrete_solve_respect_exact_bounds` across all 9 finite controls and 60 random instances, verifying that `search_dual::<120>` + `blp::certify_basis` matches `blp::solve` on `delta_hull_lp`, that both `delta_hull_lp` and `certify_lagrangian_dual_bound` bound the exact branch-and-bound optimum from below, and that `lp_guided_discrete_solve::<120>` produces an exact guard-feasible assignment (`full_guard_vector_matches = true`, `guard_min >= 0`, `bb_res.objective <= disc.objective <= 0`).
- **Docs (`+46 / -0` lines in repo docs, plus `ORDER.md` `+40` and `REPORT.md`):**
  - [`docs/labs/legal-construction-exact-solve/README.md`](../../../../labs/legal-construction-exact-solve/README.md) (`+45 / -0` lines): recorded Unit 5 methods, root-cause diagnosis of Unit 3b's primal stall, exact rational certificates, Lagrangian dual bound, and LP-guided discrete solve on `attempt2`.
  - [`docs/research/notebook/legal-construction.md`](../../../../research/notebook/legal-construction.md) (`+1 / -0` lines): added one row for Unit 5.
- **Generated / Other:** `0` lines.
- **Size against ORDER estimate:** `1,550` net lines (`+1,550 / -39` excluding `ORDER.md`/`REPORT.md`) across the exact 7 scoped files against the ORDER's `~420`-line estimate (`3.69×` ratio). The increase comes from implementing the complete `Dy<P>` Dual Simplex + BFRT engine, the 60-row disjunction projection and basis lifting, the basis-free Lagrangian dual bound, and the warm-started LP-guided discrete solver inside `search.rs` so both `bb::relaxation` and `delta_hull_lp` certify in seconds and the discrete optimality gap closes from `[-21.03, -0.775]` to `[-21.02926767, -21.01684884]` (`0.059%`).

## Measurements

All measurements were taken on the laptop (`macOS`, release profile for `root-search`, sha256-verified read-only `attempt2` directory at `local/claude/legal-case/codex-direct-legal-centered-20261009/attempt2`, 384/384 files verified):

- **Command:**
  ```sh
  cd docs/labs/legal-construction-exact-solve/solver
  CARGO_TARGET_DIR=/tmp/antigravity-legal-target ~/.cargo/bin/cargo build --release --locked --bin root-search
  /tmp/antigravity-legal-target/release/root-search /Users/casey.allard/uor-r4/local/claude/legal-case/codex-direct-legal-centered-20261009/attempt2
  ```
- **Results (two consecutive runs produced identical numbers):**
  1. **Root LP relaxation (`bb::relaxation`, 440 rows, 1,960 variables):**
     - `Dy<120>` search (`search::search::<120>`): **`268` pivots, `285` bound flips, `2` degenerate pivots, `0` Bland switches in `1,993 ms` (`1.99 s`)** (vs. Unit 3b's timeout after `48,025` pivots in `7,800 s`).
     - Exact rational basis certificate (`blp::certify_basis`): **`certified = true` in `16,890 ms` (`16.89 s`)**.
     - Certified exact rational optimal objective (`~ -21.0301946857`):
       `-12265804370713109167117098919207832811683970906310184228788584982872931408529327572324431626879226631428910783652597928245619611202005471337022109155425987222940930071613354119049006854798921364124331237674084326964783771805503751150845734281676026286919946123547750040818742761806555404348484432065712166132599940679254061692725747668655373367482235461922167504582732358764778554452507282883432176125356000504993716631289710263040882116836745090144007395154853573663972878252605872959433047747484561927200761002521818794228451663390876567050119402992406851594702866986611921449076796626556914525363303620840659969097105593942576100279782164364048910147201693722781853976196281038880571819065984003709216192244462658975900429058037096454038922806518561394774081625770144714296000687837444419067614492662883704485119629481/583247304841584337206558341128916969183660190857073861307315979076639859111134748339476257812563142282528078285349222110234675622839366982213116632859894656784738584187432576005626112768138197953303126028624302101881730461477811675062444895913328989291262591847670146576153704185330616540587089858470094048372230770092375472025287368588777088823105435726604690399631157711975754364489876673165110953866622136345342047659798199742604887269702385042566831677169845365450978681803558611808472288368441868598653300745844071312564586126806351076967766624148860136499654510734805301080421216618042689882564801926930368346981716504546610562389247829036075222449675510577696762814021348271687863088147875599558261644894363382080287582633281389471276853370257152564630993954482933091369547023184297211587532344529650611510050816`
  2. **Convex-hull `δ`-relaxation (`search::delta_hull_lp`, 380 rows, 1,920 variables):**
     - Exact rational basis certificate (`blp::certify_basis`): **`certified = true` in `14,970 ms` (`14.97 s`)**.
     - Certified exact rational optimal objective (`~ -21.0292676715`):
       `-308714027835603580409423589115540466929534521403250577384074357521076317361000485880372720626823428525925713884474612758926037126591533080432002538811204827443181140443278829009616472192228008670573107561781226104171382677250507733902308990390237837872555159862627759640196000437754865014881293736011331272284657745590946804820664312984683291197038324856911899605852249612492342462205804570760380124721861875669719774191454132570880820061333567163116969603038813241021532519948010029316989658903223951232059281050203609926405352026828883553285483553118915075338289044948473079615426841568241556504292755022289298243841498550255447270490337262097444464751828552911306591249736400411934387999745406431026428833007335805138595460067177965915149145453190939144503688287661451935450673503989830610308460724978964911/14680208204107762102746214254048815142104420492789932855601783229541341736524903051756048160993442430835018239886705536653504256034236043808033061732941832791774454538373622345323848323127331529341182445450978072091974139664822076107893833915524030779421203913587559370245684504804134134083715332491291325996985425890952460399098263046203730776371248259143753071734795484313143592761472460865290487164209279016391880862082437606391689122148562848877266871234084494084679281287945012879493909457090181846027934325193433196983813346250139752514766165611906521006527374822200399752276854894016311390808966737072614291879232220550116765080647146739968805287108672132509770870928244762905674319366032953772137182958855934063238668589601038350791612240630569728440918457808617374700517619501577605473978393051529216`
  3. **Basis-free exact Lagrangian dual bound (`search::certify_lagrangian_dual_bound`):**
     - `-69207511589696303187478460303536414054189737500130664906640173451/3291009114642412084309938365114701009965471731267159726697218048` (`~ -21.0292676715`).
  4. **Warm-started `Dy<120>` LP-guided discrete assignment (`search::lp_guided_discrete_solve::<120>`):**
     - Solve time: **`3,781 ms` (`3.78 s`)**, **`260` total Dual Simplex pivots** across all margin iterations.
     - Moved coefficients: **`1,464` of `1,920`**.
     - Guard verification: **`active_guards = 0` of `380`** (every guard strictly positive), **`guard_min = 176121103777/281474976710656` (`~ +0.00062571 > 0`)**, `full_guard_vector_matches = true`.
     - Exact rational objective: **`-94651472617665723/4503599627370496` (`~ -21.01684884`)**, within **`0.059%` (`0.01241883`)** of the certified `-21.02926767` discrete lower bound.

## Checks run at the head

- `CARGO_TARGET_DIR=/tmp/antigravity-legal-target ~/.cargo/bin/cargo test --locked` (in `docs/labs/legal-construction-exact-solve/solver/`): PASS (all 11 test targets green: `lib`, `root-lp`, `root-search`, `single-move`, `solve-case`, `blp_agrees`, `case_loader`, `case_verified`, `certify_basis`, `controls`, `no_floats`, `random`, `search_certify`).
- `CARGO_TARGET_DIR=/tmp/antigravity-legal-target ~/.cargo/bin/cargo clippy --lib --bin root-search --test search_certify -- -D warnings`: PASS (`0` warnings).
- `rustfmt --edition 2021 --check src/search.rs src/blp.rs src/bin/root-search.rs tests/search_certify.rs`: PASS.
- `uor-harness loop report --check`: PASS.
- `uor-harness loop pr-check --local`: PASS (except the expected pre-scope `scope` warning).
- `uor-harness loop gates --full`: PASS (`7/7` passed).

## Skills used

- `uor-research-architect` (`.agents/skills/uor-research-architect/SKILL.md`)
- `solution-integrity` (`/Users/casey.allard/.gemini/config/skills/solution-integrity/SKILL.md`)
- `scope-guard` (`/Users/casey.allard/.gemini/config/skills/scope-guard/SKILL.md`)

## Still open

- **Closing the remaining `0.01241883` (`0.059%`) discrete optimality gap on `attempt2`:** Unit 5 certifies the exact continuous lower bounds (`~ -21.03019469` on `bb::relaxation` and `~ -21.02926767` on `delta_hull_lp` and the Lagrangian dual bound) and exhibits an exact guard-feasible discrete assignment at `-94651472617665723/4503599627370496` (`~ -21.01684884`) with strictly positive margin on all 380 guards (`min J·δ ≈ +0.000626 > 0`), leaving a `0.01241883` gap between the discrete lower and upper bounds. Closing that remaining `0.059%` gap (or proving that no discrete point in the gap satisfies `J·δ >= 0`) is tracked on `#2828` (`legal-construction-exact-solve`, status stays `ready`).

## Plain summary

The legal-construction problem asks how much we can lower a saved linear objective (`g·δ`) by moving 1,920 model coefficients (`δ`) to legal quarter-grid destinations (`q in -7..=7`) without making any of 380 linear guard margins (`J·δ >= 0`) negative. In earlier units (Units 3a and 3b), trying to solve even the continuous linear-programming relaxation from the starting point `δ = 0` timed out after more than two hours (48,025 steps) because all 380 guards pass through zero at `δ = 0`, trapping primal simplex at a degenerate corner.

In this unit (Unit 5), we first added step counters that confirmed the degeneracy stall and then implemented a float-free fixed-point Dual Simplex solver (`Dy<120>`, using 128-bit integer mantissas and no binary floating point) with the Bound-Flipping Ratio Test (which flips bounded variables across their full interval in one step) and an algebraic projection of the 20 fractional coefficient constraints. Instead of starting at the degenerate `δ = 0` corner, Dual Simplex starts at the cost-optimal corner of the bounding box where dual feasibility holds immediately. It finds the exact optimal basis of the 1,960-variable, 440-row root relaxation in **268 pivots (`1.99 s`)**, and our optimized fraction-free rational verifier (`blp::certify_basis`) proves that basis optimal in **`16.89 s`** with exact rational objective `~ -21.03019469` (and `~ -21.02926767` on the tighter 380-row convex-hull relaxation, confirmed independently by a basis-free Lagrangian dual bound). Finally, warm-starting Dual Simplex to iteratively tighten any guard violated by discrete rounding produces an exact, independently verified legal discrete assignment across 1,464 coefficients in **`3.78 s`** with objective `-94651472617665723/4503599627370496` (`~ -21.01684884`) and strictly positive slack on all 380 guards—improving on Unit 4's `-0.77505` move by `27×` and narrowing the gap to the theoretical optimum to `0.059%`.
