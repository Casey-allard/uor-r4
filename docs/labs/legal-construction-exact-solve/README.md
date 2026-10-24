# Legal-construction exact solve — lab record (#2828)

Item: [`legal-construction-exact-solve`](../../roadmap/items/legal-construction-exact-solve.md) (`#2828`). This record covers the exact-arithmetic host instrument built from the archived design note ([#2828 comment 6351533475](https://github.com/UOR-Foundation/uor-r4/issues/2828#issuecomment-6351533475)) and the saved-case run of 2026-10-22. No GPU, no model compute and no panel scoring is involved.

## Unit 2 — exact solver and finite controls (merged)

The standalone crate [`solver/`](solver/) is an exact rational two-phase tableau simplex with Bland's rule, inside depth-first branch-and-bound over the bounded integer codes `k` and the `l`/`r` binaries, starting from the incumbent `δ = 0`. The program is `minimise g·δ` subject to `J_j·δ ≥ 0` for every guard row, with `δ_i = k_i/4 + (l_i + r_i)(q0_i/4 − m_i)`. Every number is an arbitrary-precision rational; binary floating point appears only in the bit-exact f32 loader. The finite controls (tiny instances, brute-force enumeration, endpoints `q = ±7`, noop branch, all-zero and forbidding guards) pass and are mutation-tested.

## Unit 3 — case mapping and the dense-LP blocker (deepseek-33, 2026-10-22)

**What this unit settled.** The design note's open point 1 (which saved files are `g` and `m`) is resolved from the original source at commit `ebd6ecaa`, not inferred:

| Input | Source file (in `attempt2/`) | Shape |
|---|---|---|
| masters `m` | `coupled-prefix.coefficients-initial-master.f32le` then `coupled-generate.unary-initial-master.f32le` | 960 + 960 |
| objective `g` | `coupled-prefix.coefficients-gradient.f32le` then `coupled-generate.unary-gradient.f32le` | 960 + 960 |
| guard rows `J` | `protected-margin-{000..379}.f32le`, in guard-index order | 380 rows × 1,920 |

The order is the one used by `legal_proposals` (`protected_joint_vector.rs`) when it calls `protected_legal_set::run(&m, &g, &rows)`; every file is little-endian IEEE-754 binary32 and converts bit-exactly with `exact::f32_to_rational` (NaN/infinity refused). The 380 guard-row indices follow the guard population order in `protected-margin-receipt.json`.

**Verification.** The case loader is [`solver/src/case.rs`](solver/src/case.rs) (unit tests: expected shape, NaN refusal, wrong row length) with the run binary [`solver/src/bin/solve-case.rs`](solver/src/bin/solve-case.rs). On the saved case (extracted read-only under the gitignored `local/claude/legal-case/` from the verified iCloud archive):

- load succeeds: `n = 1,920`, `guards = 380`, `fractional = 20` (the 20 non-quarter-grid masters), in about `0.13 s`;
- the exact zero move is legal and every guard holds: `guard_value_min = 0/1`, `objective = 0/1`;
- the 31 per-term gradient files per family, summed in f32 order, are **bit-identical** to the corresponding `coupled-*-gradient.f32le` aggregate (both families), which is the original source's own replay check.

**The solve attempt, and why the item stays open.** The unit-2 solver is an exact rational dense tableau; its root LP at this size is a 1,960-variable, 440-row rational problem. This unit's exact root LP (`solve-case --stage lp`) did not return within a **120 s** cap and produced no objective/status line.

**Correction (deepseek-34, after a counted review).** An earlier draft of this section claimed an "independent dense run" in `local/claude/legal-case/root-lp-run1.log` had failed after 72+ minutes. That was wrong: the log is **claude-50's bounded-variable revised simplex** (`blp`, commit `95328fb27`, visible on PR #3015), which started at 2026-10-22T22:05:45Z under a declared **7,800 s cap**, was still running inside that cap when first cited, and whose objective had turned negative after ~250 degenerate pivots. This record uses only deepseek's own 120 s dense-tableau measurement and makes **no claim** about claude-50's method or outcome; that belongs to #3015's record.

The structural reason is visible in the data: the guard matrix is dense-ish (median 1,033 non-zeros per row; 380 rows over 1,920 columns) and simplex pivots on growing rationals. That is a **tooling/solver blocker, not a mechanism negative** (D22): the case has not been solved, so no optimality or infeasibility claim is made, and the item remains `ready`.

**What the next unit should try** (recorded, not done here): a bounded-variable or revised exact simplex with fraction-free updates, a sparse factorization, or a structural reduction that exploits the 380 constraints before any branch-and-bound — and it should time the root LP before building the tree, exactly as the unit-2 REPORT warned.

**Re-run commands** (from `docs/labs/legal-construction-exact-solve/solver/`, with the case extracted to its gitignored `local/` path):

```sh
CARGO_TARGET_DIR=/tmp/legal-case-target ~/.cargo/bin/cargo test --locked
CARGO_TARGET_DIR=/tmp/legal-case-target ~/.cargo/bin/cargo build --release --locked --bin solve-case
/tmp/legal-case-target/release/solve-case --case <attempt2 dir> --stage load
/tmp/legal-case-target/release/solve-case --case <attempt2 dir> --stage lp
```

## Unit 3a: verified inputs and the bounded-variable simplex (claude-53, 2026-10-23)

**Classification: instrument.** Kimi's PM ruling on this issue (14:15Z on 22 Oct, 6342969530) classes an exact host solve as an instrument. A run that does not finish is an instrument stop, not a mechanism negative (D22).

**Verified inputs.** `case::load_verified` checks every file against the sha256 the archive itself records, then calls `case::load`. On the saved case that is 384 of 384 files:
- `protected-margin-receipt.json` `terms[i].sha256` for the 380 guard rows;
- `coupled-gradient-receipt.json` `files[]` for the 4 master and gradient files.

`manifest.json` records blake3 only and is not used.

**The bounded-variable simplex (`blp`).** The unit-2 tableau writes every upper bound as a row, so at this size it is about 2,400 × 4,400. `blp` keeps the 440 real rows and handles bounds implicitly, with bound flips:
- Each row and the objective are scaled to integers. All data is dyadic.
- The basis inverse is kept as `M = det(B)·B⁻¹`. Its integer-preserving updates divide exactly, and the remainder is checked.
- Pricing is Dantzig's rule, falling back to Bland's after 50 degenerate pivots.
- Before returning `Optimal`, a certificate independent of `M` checks every input row and bound exactly, and every reduced cost's sign.
- `blp` agrees exactly with `lp` on the controls, the random suite, tightened-bound node LPs, and a case of 64 rows or more (the threaded path).

**The root-LP run (claude-50, `blp` at commit `95328fb27`).** This is an instrument stop:
- Started 2026-10-22T22:05:45Z under a declared 7,800 s cap. Ended 2026-10-23T00:15:46Z with `timeout` exit 124, and no result line.
- 775 pivots (the last progress line at 7,510 s), 2 bound flips, all in phase 2: the start point δ = 0 is feasible.
- `D = |det B|` grew 46–71 bits per pivot in each 25-pivot stretch up to pivot 250 (13,834 bits there). It then flattened, reaching at most 22,846 bits. The last pivots took about 20 s each.
- Progress lines come every 25 pivots. The objective was 0 at every line through pivot 275, and negative from pivot 300 on.
- To re-run: `cargo run --offline --locked --release --bin root-lp -- <attempt2 dir>`, with the case directory as in the commands above.

**What it establishes:** only that this exact search does not finish in the ORDER's time budget on this laptop.
- The negative phase-2 objective is an observation of the instrument. Phase 2 keeps iterates primal-feasible in exact arithmetic, so it points to a relaxation optimum below 0, in which case branch-and-bound decides the item.
- No certificate was produced, so no optimum, and no sign of one, is claimed.

**Next.** Claude's design note (6354250887) separates the search from the proof. A fixed-point integer search (no floats) finds a candidate basis, and `blp`'s exact certificate proves or rejects it. That is claude-54.

## Unit 3b: a fixed-width search, proved by the exact certificate (claude-54, 2026-10-23)

**Classification: instrument,** as in Unit 3a.
- Under Casey's standing direction (#2028, 6355948874), recorded on this issue (6355973721), the search counts as an exact-arithmetic host instrument.
- A result is reported only with a passing exact certificate. The search's numbers never enter the proof; only its basis does.

**The search (`search::search::<P>`).** It is `blp`'s bounded-variable revised simplex, run on `Dy<P>` (`src/dyadic.rs`):
- `Dy<P>` is an `i128` mantissa of at most `P` bits plus an `i32` exponent, rounded to nearest once per operation. `P = 120` here.
- There is no `f32` or `f64`, and `no_floats` still passes.
- It returns a proposed optimal basis: the basic indices, plus each nonbasic variable's position (`Lower` or `Upper`).

**The proof (`blp::certify_basis`).** From the basis it solves `B·x_B = b − N·x_N` and `πᵀB = c_B` exactly, by fraction-free (Bareiss) elimination on the integer-scaled data. It then runs `blp`'s certificate on every input row and bound, and on every reduced cost's sign.

**The repair (`blp::solve_from_basis`).** If the certificate fails, `blp` starts its exact pivots from that basis (a warm start) and certifies again. `blp::solve_search_certify_repair` runs search, proof and repair in turn.

**Controls** (`tests/search_certify.rs`, `tests/certify_basis.rs`):
- On the controls, the random suite and the tightened-bound node LPs:
  - search plus certificate gives the same status and the exactly equal objective as `lp::solve`, at `P = 120`;
  - at `P = 12` it never returns a wrong optimum;
  - search, certificate and repair equal `blp` on every LP, at both widths.
- A planted corrupted basis (one basic index swapped) is rejected by the certificate, and the warm start recovers the exact optimum.
- A planted wrong reduced-cost sign is rejected.
- Bad bases are errors, not panics.

**The root run (`root-search`, binary at `2115180e1`): an instrument stop.**
- **Input:** the saved case's root relaxation (1,960 variables, 440 rows), loaded through `case::load_verified` (384 of 384 sha256 checks).
- **Timing:** started 2026-10-23T03:08:21Z under the declared 7,800 s cap. Ended 05:18:21Z with `timeout` exit 124, no result line and an empty JSON.
- **Pivots:** 48,025, against `blp`'s 775 in the same budget. The last progress line was at 7,795 s. All were in phase 2 (δ = 0 is feasible), with 37 bound flips.
- **Objective:**
  - 0 at every progress line through pivot 325, and negative from pivot 350 on (`blp` went negative from pivot 300);
  - its magnitude (`⌈log₂|z|⌉`) reached 4 by pivot 1,825 and 5 by pivot 14,000, and was still 5 at the end.
- **Pivot rate**, in pivots per second:

  | Window | Rate |
  |---|---|
  | first 1,000 s | about 4.6 |
  | next 1,000 s | about 3.1 |
  | 2,000 s to 6,000 s | about 8–10 |
  | last 1,800 s | about 2.5 |

  From about 6,500 s on, this session's own cargo builds shared the CPU, so the late slowdown is not a property of the search.
- **The certificate never ran,** because the search returned no basis.
- **To re-run:** `cargo run --offline --locked --release --bin root-search -- <attempt2 dir>`.

**What it establishes:** only that this bounded-precision search does not reach an optimal basis on this laptop within the ORDER's time budget.
- No basis was proposed, so no certificate ran, and no optimum (nor its sign) is claimed.
- Each pivot is about 60 times cheaper than an exact one (48,025 against 775 in the same time), so pivot cost is no longer the bottleneck it was in unit 3a. The pivot count is.
- After pivot 14,000 the objective's magnitude stayed in one power of two for about 34,000 pivots. That is consistent with long degenerate stretches under Dantzig pricing with Bland's fallback (50 degenerate steps).
- That reading is not measured: `SearchStats` counts no degenerate steps.

**Next.** All of these stay inside the search, so the certificate keeps the result exact:
- count degenerate steps and Bland switches in `SearchStats`, so the stall is measured rather than inferred;
- then a degeneracy-aware rule inside the search: bound perturbation or steepest-edge pricing on `Dy`.

## Sources

- Design note: [#2828 comment 6351533475](https://github.com/UOR-Foundation/uor-r4/issues/2828#issuecomment-6351533475).
- Original source: commit `ebd6ecaa`, `crates/uor-r4-training/examples/geometric_frozen_map_fit/protected_joint_vector.rs` and `protected_legal_set.rs`.
- Saved case: `codex-direct-legal-centered-20261009` (`md5 5a965dade6ed7b04b8238b4d1e020be3`), read from the gitignored local extraction; it is not committed.
- Item: [`legal-construction-exact-solve`](../../roadmap/items/legal-construction-exact-solve.md); mechanism page: [`docs/mechanisms/legal-construction.md`](../../mechanisms/legal-construction.md).

## Unit 4: exact single-coefficient census and a feasible legal change (deepseek-57, 2026-10-23)

**Classification: instrument.** The PM ruling on #2828 (Kimi, 6342969530) classes an exact host solve as an instrument, and the owner direction recorded at 6355973721 keeps it there. This unit runs no GPU and no training; all decisions are arbitrary-precision rational arithmetic. It produces a **feasible improving legal change**, not an optimality or infeasibility proof, so the item stays `ready`.

**Method.** If only coefficient `i` moves by `delta_i`, every guard sees `J_j . delta = J[j][i] * delta_i`, so a positive `delta_i` is legal exactly when every entry of column `i` is `>= 0` and a negative one exactly when every entry is `<= 0` (exact sign test). The unit enumerates the legal destination set per coefficient from the design note's encoding, reports the exact best single move, then runs a greedy joint pass with exact rational slack: it derives the exact feasible delta interval per column from the current slack, picks the best improving destination inside it, applies it, and repeats until no improving single-coefficient re-destination remains. Every coefficient may be re-destined, so the stop is a single-coefficient local optimum, not a global one. Every accepted move is checked on all 380 guard rows; the returned assignment is recomputed from scratch from `g` and `J`, and the **full 380-entry guard vector** must equal the incremental one exactly.

**Result on the saved case** (input `attempt2`, `load_verified` 384 of 384 hashes, 1.68 s):

- Census: 1,920 columns; **1,883 columns have both signs** (no single-coefficient move possible), 11 positive-only, 17 negative-only, 9 all-zero; only **37 columns admit any single-coefficient move**.
- Exact best single move: coefficient 959 to code `q = 7`, `delta = 2/1`, objective change `-8791175/67108864` (about -0.131), all 380 guard rows exactly non-negative with minimum `0/1`.
- Greedy joint pass: **48 accepted moves on 40 distinct coefficients, no improving single-coefficient re-destination remains**; objective **-426088804491/549755813888 (about -0.775051)**; exact guard minimum `0/1`; **320 of 380 guards active (exactly zero)**; 48.0 s including the independent recomputation.
- Independent recomputation from `g` and `J` equals the incremental objective and **the full guard vector is identical** (`full_guard_vector_matches=true`); a second run produced the same assignment and numbers.

**Independent replication and the correction it forced.** A fresh-context reviewer independently reloaded the raw binary32 files with its own bit-exact rational conversion and its own encoding implementation: it confirmed the earlier 41-move assignment's objective, guard minimum, zero-guard count and legality exactly, and it found that the earlier version's claim of no improving move was false because the search never revisited already-chosen coefficients (21 feasible improving re-destinations existed; the best alone reached about -0.72195). The search now permits re-destinations, which produced the stronger result above, and the independent check now compares the full guard vector rather than only its minimum. That reviewer's finding is the reason this record does not claim a local optimum in the full assignment space.

**Returned assignment** (coefficient index `i`, final code `q`, exact `delta`; 40 distinct coefficients):

```json
{"i":27,"q":-7,"delta":"-3/2"},{"i":45,"q":-7,"delta":"-3/2"},{"i":56,"q":-7,"delta":"-3/2"},{"i":93,"q":-7,"delta":"-3/2"},{"i":195,"q":-7,"delta":"-3/2"},{"i":239,"q":-7,"delta":"-7/4"},{"i":248,"q":-3,"delta":"-1/2"},{"i":263,"q":-7,"delta":"-7/4"},{"i":269,"q":-7,"delta":"-3/2"},{"i":316,"q":-5,"delta":"-1/1"},{"i":318,"q":-7,"delta":"-7/4"},{"i":322,"q":-7,"delta":"-3/2"},{"i":349,"q":-7,"delta":"-3/2"},{"i":355,"q":7,"delta":"2/1"},{"i":359,"q":7,"delta":"2/1"},{"i":372,"q":-7,"delta":"-3/2"},{"i":466,"q":-7,"delta":"-3/2"},{"i":486,"q":7,"delta":"2/1"},{"i":498,"q":-7,"delta":"-3/2"},{"i":505,"q":7,"delta":"7/4"},{"i":521,"q":-7,"delta":"-3/2"},{"i":533,"q":2,"delta":"1/2"},{"i":548,"q":-7,"delta":"-3/2"},{"i":591,"q":-2,"delta":"-1/4"},{"i":592,"q":-7,"delta":"-3/2"},{"i":614,"q":-1,"delta":"-1/4"},{"i":625,"q":7,"delta":"2/1"},{"i":633,"q":-3,"delta":"-1/2"},{"i":646,"q":-7,"delta":"-3/2"},{"i":661,"q":-7,"delta":"-7/4"},{"i":671,"q":-7,"delta":"-7/4"},{"i":757,"q":-4,"delta":"-3/4"},{"i":777,"q":-7,"delta":"-3/2"},{"i":808,"q":-7,"delta":"-7/4"},{"i":834,"q":-7,"delta":"-3/2"},{"i":851,"q":7,"delta":"7/4"},{"i":874,"q":7,"delta":"7/4"},{"i":908,"q":-7,"delta":"-7/4"},{"i":917,"q":5,"delta":"3/2"},{"i":959,"q":7,"delta":"2/1"}
```

**What this establishes.** Prior units had never produced a legal move on the saved case: the exact dense tableau and the bounded-variable revised simplex did not finish, and the bounded-precision search returned no basis. This unit exhibits, in exact arithmetic, a legal code change that lowers the saved linearised objective (`g . delta < 0`) and satisfies every guard row exactly (`min J.delta = 0`). The discrete optimum is therefore **at most -426088804491/549755813888 (about -0.77505)**; it is not shown to be optimal. This is consistent with the negative phase-2 objective claude-50 observed on the relaxation.

**Limits.** The greedy pass stops when no single-coefficient re-destination improves the objective; a multi-coefficient joint improvement may still exist, so no optimality certificate, dual bound or infeasibility proof is produced. The actual commit rule (native scoring, 17 reference cases, strict-loss lowering) is not evaluated here; the unit only solves the saved guard-constrained linear problem. Training and rental remain HELD.

**Re-run** (from `docs/labs/legal-construction-exact-solve/solver/`, case extracted read-only):

```sh
CARGO_TARGET_DIR=/tmp/legal-target ~/.cargo/bin/cargo test --locked
CARGO_TARGET_DIR=/tmp/legal-target ~/.cargo/bin/cargo build --release --locked --bin single-move
/tmp/legal-target/release/single-move --case <attempt2 dir> --max-moves 256
```

## Unit 5: exact root-LP certificates, Lagrangian dual bound, and LP-guided discrete solve (antigravity-47, 2026-10-23)

**Classification: instrument.** As in Units 3a, 3b, and 4, this unit is an exact-arithmetic host instrument (no GPU, no training, no panel scoring, and no binary floats in the solver or certificate). Every bound and feasible assignment is verified in arbitrary-precision rational arithmetic (`BigRational`). Because a `0.01241883` (`0.059%`) gap remains between the certified discrete lower bound (`~ -21.02926767`) and the best exact guard-feasible discrete assignment (`~ -21.01684884`), the item stays `ready`.

**Why Unit 3b's primal search stalled, and how Unit 5 resolves it.**
1. **Degenerate primal vertex at `δ = 0`.** All 380 guard rows have right-hand side `b_j = 0`, so the initial feasible point `δ = 0` sits at the intersection of 380 homogeneous halfspaces. Adding `degenerate_pivots` and `bland_switches` counters to `SearchStats` and diagnosing the primal `Dy<120>` trajectory showed two distinct causes of the 48,025-pivot stall:
   - after each `REFACTOR_EVERY = 200` full Gaussian elimination in `Search::refactor`, `recompute_xb` left tiny `Dy<120>` rounding dust (`lo[bv] - x[bv] > 0` or `x[bv] - u[bv] > 0`) on basic variables, which `Search::ratio` treated as zero-step (`ratio = 0`) degenerate pivots without advancing nonbasic variables;
   - even with refactor dust clamped in `Search::ratio`, primal simplex from `δ = 0` traverses thousands of zero-RHS vertices, and `bb::relaxation`'s 60 fractional disjunction rows (`l_f + r_f <= 1`, `-k_{i_f} - 14 l_f <= 0`, `k_{i_f} + l_f <= 0`) share variables `k_{i_f}` that are zero at the start.
2. **Bounded-variable Dual Simplex with the Bound-Flipping Ratio Test (BFRT) + disjunction projection lifting (`search::search::<120>`).**
   - Setting each structural variable to its cost-optimal bound (`δ_i = u_i` when `g_i < 0`, `δ_i = l_i` when `g_i >= 0`) and making the 380 slacks basic yields a **dual-feasible starting basis in one step** with zero Phase-1 work (`y = 0 >= 0`).
   - Each Dual Simplex iteration selects the most primal-infeasible basic row, flips bounded nonbasic variables across their full interval `[l_j, u_j]` via BFRT while their dual ratio stays optimal, and pivots once (`search_dual::<P>`).
   - For `bb::relaxation` (440 rows, 1,960 variables), the 20 three-row fractional disjunction blocks at `q0_i = ±7` project exactly onto tightened box bounds on the 380-row structural LP (`try_project_disjunction_lp`), and any optimal basis of the 380-row projected LP where the 20 fractional columns are nonbasic at their bounds lifts algebraically to a 440-row basis of `bb::relaxation` (`lift_projected_found`).
3. **Fraction-free Bareiss speedup in `blp::certify_basis`.** Eliminating unit rows and unit columns while the Bareiss divisor is `1`, performing exact integer back-substitution before dividing by `det(B)` once per row, filtering non-zero dual multipliers in `col_pi`, and summing nonbasic columns before basic columns cut `blp::certify_basis` on the 440×1,960 root LP from `125.7 s` to `16.9 s`.
4. **Basis-free Lagrangian dual bound (`search::certify_lagrangian_dual_bound`) and warm-started LP-guided discrete solver (`search::lp_guided_discrete_solve::<120>`).**
   - For any non-negative dual multiplier vector `y >= 0` on `J·δ >= 0`, Weak Duality proves `g·δ >= Σ_i min_{q in -7..=7} (g_i - (J^T y)_i) δ_i(q)` for every guard-feasible discrete assignment in `O(m·n)` exact rational arithmetic without matrix inversion.
   - Starting from the `delta_hull_lp` optimal basis (`iter 0`), `lp_guided_discrete_solve::<120>` rounds each `δ_i` to its nearest legal discrete code `q_i in -7..=7`, tightens the LP right-hand side `b_r` on any guard violated by rounding error, warm-starts Dual Simplex (1–4 pivots per iteration, `260` total pivots across all iterations), polishes single coefficients in exact rational arithmetic, and verifies the resulting assignment by independent from-scratch recomputation over `BigRational`.

**Results on the saved case** (`attempt2`, `case::load_verified` 384 of 384 sha256 checks; two consecutive runs of `root-search` produced identical numbers):

- **Certified exact optimal objective of `bb::relaxation(&p)` (440 rows, 1,960 variables):**
  - `Dy<120>` search: **268 pivots, 285 bound flips, 2 degenerate pivots, 0 Bland switches in `1.99 s`** (`1,993 ms`).
  - Exact rational certificate (`blp::certify_basis`): **PASS (`certified = true`) in `16.89 s`** (`16,890 ms`).
  - Exact rational optimal objective (`~ -21.0301946857`):
    `-12265804370713109167117098919207832811683970906310184228788584982872931408529327572324431626879226631428910783652597928245619611202005471337022109155425987222940930071613354119049006854798921364124331237674084326964783771805503751150845734281676026286919946123547750040818742761806555404348484432065712166132599940679254061692725747668655373367482235461922167504582732358764778554452507282883432176125356000504993716631289710263040882116836745090144007395154853573663972878252605872959433047747484561927200761002521818794228451663390876567050119402992406851594702866986611921449076796626556914525363303620840659969097105593942576100279782164364048910147201693722781853976196281038880571819065984003709216192244462658975900429058037096454038922806518561394774081625770144714296000687837444419067614492662883704485119629481/583247304841584337206558341128916969183660190857073861307315979076639859111134748339476257812563142282528078285349222110234675622839366982213116632859894656784738584187432576005626112768138197953303126028624302101881730461477811675062444895913328989291262591847670146576153704185330616540587089858470094048372230770092375472025287368588777088823105435726604690399631157711975754364489876673165110953866622136345342047659798199742604887269702385042566831677169845365450978681803558611808472288368441868598653300745844071312564586126806351076967766624148860136499654510734805301080421216618042689882564801926930368346981716504546610562389247829036075222449675510577696762814021348271687863088147875599558261644894363382080287582633281389471276853370257152564630993954482933091369547023184297211587532344529650611510050816`
- **Certified exact optimal objective of `delta_hull_lp(&p)` (380 rows, 1,920 variables; convex hull of the discrete legal set per coefficient):**
  - Exact rational certificate (`blp::certify_basis`): **PASS (`certified = true`) in `14.97 s`** (`14,970 ms`).
  - Exact rational optimal objective (`~ -21.0292676715`):
    `-308714027835603580409423589115540466929534521403250577384074357521076317361000485880372720626823428525925713884474612758926037126591533080432002538811204827443181140443278829009616472192228008670573107561781226104171382677250507733902308990390237837872555159862627759640196000437754865014881293736011331272284657745590946804820664312984683291197038324856911899605852249612492342462205804570760380124721861875669719774191454132570880820061333567163116969603038813241021532519948010029316989658903223951232059281050203609926405352026828883553285483553118915075338289044948473079615426841568241556504292755022289298243841498550255447270490337262097444464751828552911306591249736400411934387999745406431026428833007335805138595460067177965915149145453190939144503688287661451935450673503989830610308460724978964911/14680208204107762102746214254048815142104420492789932855601783229541341736524903051756048160993442430835018239886705536653504256034236043808033061732941832791774454538373622345323848323127331529341182445450978072091974139664822076107893833915524030779421203913587559370245684504804134134083715332491291325996985425890952460399098263046203730776371248259143753071734795484313143592761472460865290487164209279016391880862082437606391689122148562848877266871234084494084679281287945012879493909457090181846027934325193433196983813346250139752514766165611906521006527374822200399752276854894016311390808966737072614291879232220550116765080647146739968805287108672132509770870928244762905674319366032953772137182958855934063238668589601038350791612240630569728440918457808617374700517619501577605473978393051529216`
- **Basis-free exact Lagrangian dual bound (`certify_lagrangian_dual_bound`):**
  - `-69207511589696303187478460303536414054189737500130664906640173451/3291009114642412084309938365114701009965471731267159726697218048` (`~ -21.0292676715`), confirming the `delta_hull_lp` lower bound without any basis inversion.
- **Warm-started `Dy<120>` LP-guided discrete assignment (`lp_guided_discrete_solve::<120>`):**
  - Runtime: **`3.78 s`** (`3,781 ms`, `260` total Dual Simplex pivots across all margin iterations).
  - Moves **`1,464` of `1,920` coefficients** to legal discrete destinations `q_i in -7..=7` (compared with 40 coefficients in Unit 4's single-coefficient pass).
  - Every guard row is **strictly positive** (`active_guards = 0` of `380`, `guard_min = 176121103777/281474976710656`, about `+0.00062571 > 0`).
  - Independent from-scratch recomputation over `BigRational` matches the incremental objective and full 380-entry guard vector (`full_guard_vector_matches = true`).
  - Exact rational objective: **`-94651472617665723/4503599627370496` (about `-21.01684884`)**, improving on Unit 4's `-0.775051` by `27.1×` and closing the discrete optimality bracket to **`[-21.02926767, -21.01684884]` (`0.059%` relative gap, `0.01241883` absolute gap)**.

**Re-run** (from `docs/labs/legal-construction-exact-solve/solver/`, case extracted read-only):

```sh
CARGO_TARGET_DIR=/tmp/legal-target ~/.cargo/bin/cargo test --locked
CARGO_TARGET_DIR=/tmp/legal-target ~/.cargo/bin/cargo build --release --locked --bin root-search
/tmp/legal-target/release/root-search <attempt2 dir>
```

## Unit 6 — Exact rational dual multiplier extraction, reduced-cost bound tightening, and multi-schedule + pair-polished discrete solve (`antigravity-48`, 2026-10-23)

Unit 5's basis-free Lagrangian dual bound `certify_lagrangian_dual_bound` evaluated $L(y)$ at a `Dy<120>`-approximated dual vector `d.dual_y` (`-21.029262371913034...`), losing `~5.08e-6` to `Dy<120>` rounding relative to the exact `delta_hull_lp` optimum (`-21.029267452019339...`), and had no exact domain-reduction step over the $1,920 \times 15 = 28,800$ discrete codes. Unit 6 closes the dual rounding loss to zero, certifies exact reduced-cost bound tightening (RCBT), and tightens the discrete primal incumbent:

1. **Exact rational dual multiplier extraction (`blp::certify_basis_with_dual`)**: returns both the certified primal `LpResult` and the exact unscaled rational simplex multiplier vector $\pi \in \mathbb{Q}^m$ ($\pi_r = \pi_{\text{scaled}, r} \cdot \text{row\_scale}_r / c_{\text{scale}}$). On `delta_hull_lp(p)`, setting $y = -\pi \ge 0$ gives an exact rational dual multiplier vector for which `search::certify_lagrangian_dual_bound(&p, &y)` equals the exact 1,070-digit rational optimum of `delta_hull_lp(p)` (`-21.029267452019339...`) with zero rounding loss (`"dual_bound_matches_delta_hull_lp": true`).
2. **Exact rational Reduced-Cost Bound Tightening (`search::certify_reduced_cost_fixing`)**: for any non-negative dual multiplier vector $y \ge 0$ and proven primal upper bound $U$, computes each coefficient's exact reduced cost $\bar{c}_i = g_i - (G^T y)_i$, minimum contribution $m_i = \min_{q \in -7..=7} \bar{c}_i \delta_i(q)$, and global lower bound $L(y) = \sum_i m_i$. Because every guard-feasible discrete assignment satisfies $\sum_i (\bar{c}_i \delta_i(q_i) - m_i) \le g \cdot \delta - L(y)$, any discrete code $q \in -7..=7$ with $\bar{c}_i \delta_i(q) - m_i > U - L(y)$ cannot participate in any guard-feasible solution with objective $\le U$. On `attempt2` with $U = -94662137592886459 / 4503599627370496$, RCBT provably eliminates **4,859 of 28,800 discrete codes (`16.87%`)**, fixes **81 coefficients** to a single proven optimal destination code (`|surviving_codes[i]| == 1`), and reduces **137 coefficients** to at most 2 surviving codes (`|surviving_codes[i]| <= 2`).
3. **Multi-schedule warm-started LP-guided discrete solve + 2-coefficient (pair) exact rational polish (`search::lp_guided_discrete_solve`)**: evaluates 4 progressive margin schedules (`(-17, 17/16)`, `(-16, 9/8)`, `(-15, 9/8)`, `(-13, 5/4)`) warm-started from the iteration-0 `delta_hull_lp` basis (`342` total pivots across all schedules) followed by steepest-descent single-coefficient and 2-coefficient (pair) exact rational polish. On `attempt2`, this improves the certified discrete incumbent from `-94651962270606439 / 4503599627370496` (`-21.016958...`) to **`-94662137592886459 / 4503599627370496` (`-21.019217408256145...`)**, with `1,467` moved coefficients, `0/380` active guards (`min J·δ = 145873908319 / 562949953421312 ≈ +0.0002591 > 0`), and `full_guard_vector_matches: true`. This shrinks the remaining primal-dual optimality gap $U - L(y)$ by **18.35%**, from `0.0123094...` (`0.0585%`) to **`0.010050043763194...` (`0.0478%`)**.
4. **Clippy hygiene**: resolved the 3 inherited `needless_range_loop` warnings in `src/bin/single-move.rs` so `cargo clippy --all-targets -- -D warnings` passes cleanly across the entire crate.

## Unit 7 — Warm-started Simplex Diving, multi-compensator exact rational repair polish, and Guard-Slack Slab certificate (`antigravity-49`, 2026-10-23)

Unit 6's discrete solver applied single-coefficient polish in coordinate order `0..n`, restricted pair polish to single-step (`±1`) moves with a single compensator `i2`, and rounded all `<= 380` fractional basic variables simultaneously in the LP-guided margin loop. Unit 7 upgrades both the primal discrete search and the exact rational Weak Duality certificate:

1. **Warm-started One-Sided Simplex Diving + Steepest-Descent Single-Coefficient and Multi-Compensator (`k <= 2` compensators, 3-coefficient) Exact Rational Polish (`search::lp_guided_discrete_solve`)**:
   - At the optimal basis of `delta_hull_lp`, at least $1,920 - 380 = 1,540$ nonbasic variables already sit at legal discrete endpoints $q_i = \pm 7$, while only $\le 380$ basic variables are fractional. Warm-started One-Sided Simplex Diving iteratively tightens one-sided bounds on batches of fractional basic variables closest to legal discrete values and re-optimizes the remaining basic variables with `dual_simplex_run` before running the margin loop.
   - `run_polish` now ranks single-coefficient moves across all $1,920$ coordinates in steepest-descent (`gain_dy` ascending) order and runs a multi-compensator (`(i1, i2)` pair and `(i1, i2, i3)` triple, step sizes `±1, ±2`) exact rational repair pass whenever a blocked improving column `i1` leaves 1–2 violated guards after a partial compensator `i2`.
   - On `attempt2`, this improves the certified discrete incumbent from `-94662137592886459 / 4503599627370496` (`-21.019217408256145...`) to **`-94666121428139195 / 4503599627370496` (`-21.02010200849505...`)**, with `1,465` moved coefficients, `0/380` active guards (`min J·δ = 247363824223 / 562949953421312 ≈ +0.0004394 > 0`), and `full_guard_vector_matches: true`.
   - The certified primal-dual optimality gap $U - L(y)$ drops below $10^{-2}$ for the first time, from `0.010050043763194...` (`0.0478%`) to **`0.009165443524291...` (`0.04358%` relative gap, an additional 8.80% gap reduction)**.
2. **Exact rational Guard-Slack Slab Certificate + tighter RCBT (`search::certify_reduced_cost_fixing`)**:
   - By the exact Weak Duality identity $\sum_{r=0}^{m-1} y_r (G_r \cdot \delta) + \sum_{i=0}^{n-1} (\bar{c}_i \delta_i(q_i) - m_i) = g \cdot \delta - L(y) \le U - L(y)$, every guard row $r$ with $y_r > 0$ at any guard-feasible assignment with objective $\le U$ satisfies $0 \le G_r \cdot \delta \le (U - L(y)) / y_r$. `ReducedCostFixing` now certifies `guard_max_slack[r] = Some((U - L(y)) / y_r)` and `constrained_guards = 67` on `attempt2`.
   - With the tighter upper bound $U = -94666121428139195 / 4503599627370496$, RCBT provably eliminates **5,194 of 28,800 discrete codes (`18.03%`, `+335` over Unit 6)**, fixes **89 of 1,920 coefficients** to a single proven optimal destination code (`+8` over Unit 6), and reduces **139 coefficients** to $\le 2$ surviving codes.

## Unit 8 — Deep Simplex Diving, full-guard multi-compensator polish, direct guard repair, and exact Lagrangian gap decomposition (`antigravity-50`, 2026-10-24)

Unit 7's multi-compensator polish checked remaining guard violations `rem_count` only on the guard subset `viol_rows` originally violated by `i1` (missing new guard violations introduced by `i2`), stopped Simplex Diving after 6 rounds of `dive_batch = 45` without state rollback on infeasible dive steps, and did not decompose the remaining primal-dual gap $U - L(y)$ into its per-coefficient reduced-cost loss and per-guard complementary-slackness loss. Unit 8 upgrades both the discrete solver and the exact rational certificate:

1. **Deep Warm-Started Simplex Diving with Rollback + Direct Guard-Violation Repair + Full-Guard Multi-Compensator (`k <= 3` compensators, 4-coefficient) Polish (`search::lp_guided_discrete_solve`)**:
   - Extends Warm-Started Simplex Diving to 12 rounds across finer dive batches (`dive_batch in {25, 30, 45}`) with full LP state rollback (`s_lo`, `s_up`, `s_basis`, `s_st`, `s_x`, `s_binv`, `s_pi`) whenever a dive batch renders the subproblem infeasible, plus direct greedy single-step guard-violation repair whenever a margin iteration has $1 \le \text{n\_viol} \le 15$ violated guards.
   - Fixes `PartialComp` in `run_polish` to scan all $m = 380$ guards (`after1_guard` and `after12_guard`) so `rem_r0` and `rem3_r0` always identify the true most-violated remaining guard row across all 380 guards, retains the top-2 partial 2-coefficient moves (`rem_count <= 3`), and adds 4-coefficient (`(i1, i2, i3, i4)`) exact rational repair when `rem3_count == 1`, using a two-tier schedule (2 passes per schedule to select the top-2 distinct candidates, followed by 8 deep passes on the winners).
   - On `attempt2` (`3,679` total Dual Simplex pivots across 12 schedules), this improves the certified discrete incumbent from `-94666121428139195 / 4503599627370496` (`-21.02010200849505...`) to **`-94671994445923515 / 4503599627370496` (`-21.021405604209846...`)**, with `1,461` moved coefficients, `0/380` active guards, minimum guard slack tightened `6.65×` to `guard_min = 18161425 / 274877906944` (`~ +0.00006607 > 0`), and `full_guard_vector_matches: true` (two consecutive runs identical).
   - The certified primal-dual optimality gap $U - L(y)$ shrinks by an additional **14.22%**, from `0.009165443524291...` (`0.04358%`) to **`0.007862069883520...` (`0.03739%` relative gap)**.
2. **Exact Rational Lagrangian Gap Decomposition (`search::certify_gap_decomposition`) and Tighter RCBT (`search::certify_reduced_cost_fixing`)**:
   - Certifies in exact rational arithmetic (`BigRational`) the algebraic split $g \cdot \delta(\text{codes}) - L(y) = \text{reduced\_cost\_loss} + \text{complementary\_slackness\_loss}$, where $\text{reduced\_cost\_loss} = \sum_{i=0}^{n-1} (\bar{c}_i \delta_i(q_i) - \min_{q'} \bar{c}_i \delta_i(q')) \ge 0$ and $\text{complementary\_slackness\_loss} = \sum_{r=0}^{m-1} y_r (G_r \cdot \delta) \ge 0$.
   - On `attempt2`, `gap_identity_holds` is `true`, **1,890 of 1,920 coefficients (`98.44%`) achieve exact zero reduced-cost penalty** (`zero_reduced_cost_coeffs = 1,890`; only 30 coefficients pay any reduced-cost penalty), `reduced_cost_loss ≈ 0.00229004` (`29.13%` of the gap), and `complementary_slackness_loss ≈ 0.00557203` (`70.87%` of the gap).
   - With the tighter upper bound $U = -94671994445923515 / 4503599627370496$, RCBT provably eliminates **5,821 of 28,800 discrete codes (`20.21%`, `+627` over Unit 7)**, fixes **102 of 1,920 coefficients** to a single proven optimal destination code (`+13` over Unit 7, crossing 100 fixed coefficients), reduces **152 coefficients** to $\le 2$ surviving codes (`+13` over Unit 7), and tightens the 67 active dual guard slabs $0 \le G_r \cdot \delta \le (U - L(y)) / y_r$.

## Unit 9 — Cooperative partial-repair multi-compensator polish, Disjunctive Split lower bound certificate, and Disjunctive Partition RCBT (`antigravity-51`, 2026-10-24)

Unit 8's multi-compensator polish rejected any first compensator `i2` that did not 100% repair the worst violated guard `r0` by itself before considering 3- or 4-coefficient repairs (`(i1, i2, i3)` and `(i1, i2, i3, i4)`), considered only `mult in [1, 2, 3, 4]` for `i1`, and certified only the root Lagrangian dual bound $L(y_{\text{root}})$ without branching on the 67 fractional basic variables of `delta_hull_lp`. Unit 9 upgrades both the primal discrete solver and the exact rational lower-bound and domain-reduction certificates:

1. **Cooperative Partial-Repair Multi-Compensator Polish + Full-Range `i1` Steps + Post-Sweep Deduplication (`search::lp_guided_discrete_solve`)**:
   - Allows partial `r0` repairs (`new_r0_s >= after1_guard[r0] / 2`, closing $\ge 50\%$ of `r0`'s deficit) into `top_partials` when no 1-compensator pair exists, unlocking cooperative 2- and 3-compensator repairs (`(i1, i2, i3)` and `(i1, i2, i3, i4)`) where two or three compensators jointly repair a deep guard violation on `r0`.
   - Expands primary candidate transitions to the full discrete step range `mult in 1..=14`, compensator steps to `step2 in -6..=6` and `step3 in -4..=4`, and direct greedy guard-violation repair to $1 \le \text{n\_viol} \le 5$ on $m \ge 64$ ($1 \le \text{n\_viol} \le 25$ on $m < 64$, up to 45 repair steps with `step in -3..=3`), while deduplicating post-`single_sweep` states across margin iterations and verifying `deltas == delta(codes)` from scratch over `BigRational`.
   - On `attempt2`, this improves the certified discrete incumbent from `-94671994445923515 / 4503599627370496` (`-21.021405604209846...`) to **`-94674498416698555 / 4503599627370496` (`-21.021961597411334...`)**, with `1,465` moved coefficients, `0/380` active guards (`guard_min = 3319399 / 17179869184 ≈ +0.00019321 > 0`), and `full_guard_vector_matches: true` (two consecutive runs identical).
   - The root primal-dual gap $U - L(y_{\text{root}})$ shrinks to **`0.007306079116978...` (`0.03474%`)**, with complementary-slackness loss $y^T (G \delta)$ reduced by **16.29%** (from `0.00557203` to `0.00466457`), root `eliminated_codes` raised to **`6,122 / 28,800` (`21.26%`, `+301` over Unit 8)**, and root `fixed_coeffs` raised to **`112 / 1,920` (`+10` over Unit 8)**.
2. **Exact Rational Domain-Restricted Dual Bound (`search::certify_domain_dual_bound`), Disjunctive-Split Tree Certificate, and Disjunctive Partition RCBT (`search::certify_disjunctive_split_bound_with_dual`)**:
   - `certify_domain_dual_bound(p, y, code_bounds)` certifies in exact rational arithmetic (`BigRational`) the basis-free Weak Duality lower bound $L(y, B) = \sum_{i=0}^{n-1} \min_{q_i \in [q_{\min, i}, q_{\max, i}]} (g_i - (J^T y)_i) \delta_i(q_i)$ over any domain-restricted discrete box $B = \prod_i [q_{\min, i}, q_{\max, i}]$.
   - `certify_disjunctive_split_bound_with_dual::<120>(p, &U, 24, Some(dual_y_ref))` evaluates the exact disjunctive split $(q_i \le q_{\text{down}}) \lor (q_i \ge q_{\text{down}} + 1)$ across all `67` fractional basic variables of `delta_hull_lp(p)`, warm-starts `dual_simplex_run::<120>` on each branch, certifies every leaf bound in exact `BigRational` arithmetic via `certify_domain_dual_bound`, and expands a 24-leaf best-first disjunctive branch-and-bound split tree.
   - On `attempt2`, splitting `coeff 809` (`q <= -1` vs `q >= 0`) certifies a single-split lower bound gain of `+0.00018493` (`-138413805965345675648607375215541291622139312426582589707502405201 / 6582018229284824168619876730229402019930943462534319453394436096 ≈ -21.02908274`), and the 24-leaf disjunctive tree certifies `tree_lower_bound = -69206643834381985661770419441753875297126864236382476036143436075 / 3291009114642412084309938365114701009965471731267159726697218048` (`~ -21.02900400`, `tree_gain ≈ +0.00026367`), shrinking the **tree-certified discrete optimality gap $U - L_{\text{tree}}$ by `10.43%` from Unit 8's `0.00786207` (`0.03739%`) to `0.00704240` (`0.03349%` relative gap)**.
   - By intersecting the exact domain reduced-cost test on each leaf $B^{(l)}$ under the shared root dual $y_{\text{root}} = \text{dual\_y\_ref}$ and the leaf dual $y^{(l)}$ across the exhaustive 24-leaf partition (`tree_surviving_codes[i] \subseteq rcbt(dual_y_ref).surviving_codes[i]` for every $i$), Disjunctive Partition RCBT provably eliminates **`6,254 of 28,800` discrete codes (`21.72%`, `+132` beyond same-dual root RCBT's `6,122` and `+433` over Unit 8)**, fixes **`112 of 1,920` coefficients** to a single proven optimal destination code (`+10` over Unit 8), and restricts **`179 of 1,920` coefficients (`+27` over Unit 8)** to at most 2 surviving codes.
