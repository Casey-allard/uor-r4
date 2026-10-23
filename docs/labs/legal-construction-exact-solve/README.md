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

## Sources

- Design note: [#2828 comment 6351533475](https://github.com/UOR-Foundation/uor-r4/issues/2828#issuecomment-6351533475).
- Original source: commit `ebd6ecaa`, `crates/uor-r4-training/examples/geometric_frozen_map_fit/protected_joint_vector.rs` and `protected_legal_set.rs`.
- Saved case: `codex-direct-legal-centered-20261009` (`md5 5a965dade6ed7b04b8238b4d1e020be3`), read from the gitignored local extraction; it is not committed.
- Item: [`legal-construction-exact-solve`](../../roadmap/items/legal-construction-exact-solve.md); mechanism page: [`docs/mechanisms/legal-construction.md`](../../mechanisms/legal-construction.md).
