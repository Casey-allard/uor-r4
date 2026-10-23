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
