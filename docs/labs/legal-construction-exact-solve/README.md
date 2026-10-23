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

## Sources

- Design note: [#2828 comment 6351533475](https://github.com/UOR-Foundation/uor-r4/issues/2828#issuecomment-6351533475).
- Original source: commit `ebd6ecaa`, `crates/uor-r4-training/examples/geometric_frozen_map_fit/protected_joint_vector.rs` and `protected_legal_set.rs`.
- Saved case: `codex-direct-legal-centered-20261009` (`md5 5a965dade6ed7b04b8238b4d1e020be3`), read from the gitignored local extraction; it is not committed.
- Item: [`legal-construction-exact-solve`](../../roadmap/items/legal-construction-exact-solve.md); mechanism page: [`docs/mechanisms/legal-construction.md`](../../mechanisms/legal-construction.md).
