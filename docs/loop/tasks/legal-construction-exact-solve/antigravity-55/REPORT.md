# REPORT: antigravity-55, legal-construction-exact-solve

Lab: antigravity. Order: [ORDER.md](ORDER.md).
References #2828.

Written by claude on 24 Oct, finishing #3152 under the owner's close-out direction. Antigravity's speed-up was uncommitted work in the worktree; I committed it as found (`2b463ff89`). I then fixed the candidate sort that Kimi found, added the two tests, and ran both replays.

## What did not work, first

1. **The Unit 11 candidate sort visited unscored candidates first.** In `antigravity-54`, the comparator returned `Less` for `(Some, None)`. At deeper tree levels, the fractional variables with no root split score therefore filled the 16 evaluated slots before the scored ones: the reverse of what its report claimed. Kimi found it in review of #3152. `split_candidate_order` now puts scored candidates first, by descending score, then unscored ones. Because Unit 11's rational outputs came from the buggy order, the ORDER's "bit-for-bit identical Unit 11 outputs" clause and its "fix the sort" clause cannot both hold at one commit. The ORDER's dated note records that they are met in sequence.
2. **Unit 11's pool tightening made `disjunctive_ms` slower.** It went from `1,116,550 ms` (Unit 10) to `1,809,808 ms`. Each of the 24 leaves re-summed all 1,920 `BigRational` terms for each of up to 160 pool duals, with a GCD on every term.

## What changed

- **Code** (`search.rs`, `+268 / -99` lines from `f455de54f` to the head):
  - `2b463ff89` is antigravity's speed-up. `dyadic_to_rational` strips shared factors of two with `trailing_zeros` and builds the result with `BigRational::new_raw`, so there is no GCD. `sum_rationals` adds dyadic terms by shift-and-add. `dual_rc_pool` is a `Vec<(Vec<Dy<P>>, Rc<CachedRc>)>`, where a local `struct CachedRc { rc, root_terms, root_lb, root_span }` caches each dual's terms. The ORDER's plan called this `DualPoolEntry`. Leaf bounds and `tighten_leaf_mask` update incrementally over the split coordinates only, at most 23 of 1,920. `tighten_leaf_mask` skips a coefficient in O(1) when `root_span[i] <= leaf_slack`.
  - `0096186e9` adds `pub fn split_candidate_order(score_a, score_b)` and uses it for the tree-level sort: scored candidates first, by descending score, then unscored.
  - `82074e3ec` factors the certifier into `disjunctive_split_bound(.., use_dual_pool)`. It adds a `#[doc(hidden)]` ablation, `certify_disjunctive_split_bound_without_pool`, which skips the pool in both places it is read, so a test can pin the pool's effect.
- **Tests** (`tests/search_certify.rs`, `+72 / -0` lines in this unit). The PR's `+74` also includes antigravity-54's two `split66` assertions. Each test fails when its fault is planted and passes once the fault is restored:
  - `split_candidate_sort_visits_scored_before_unscored` has 24 candidates, 16 scored and 8 unscored, and asserts that the first 16 visited are exactly the scored ones in descending order. It fails with the old comparator. It calls `split_candidate_order` directly, so it pins the comparator, not the call site. Removing the `cand_splits.sort_by(..)` call, or swapping its arguments, would leave it green. Only the `attempt2` replay below exercises the call site: its outputs differ from the speed-up-only run exactly because of the sort.
  - `dual_pool_strictly_tightens_tree_bound_and_eliminations`, on seed 689, asserts that with the pool the tree lower bound is strictly higher and strictly more codes are eliminated than without it. It fails when the pool read is disabled.
- **Docs:**
  - [`README.md`](../../../../labs/legal-construction-exact-solve/README.md): a Unit 12 section.
  - [notebook](../../../../research/notebook/legal-construction.md): one row.
  - `antigravity-54/REPORT.md`: corrected against its commit.
  - This unit's `ORDER.md`: trimmed under the read-path budget, with a dated note.

## Measurements

Both runs are of `root-search` on `attempt2` (`local/claude/legal-case/codex-direct-legal-centered-20261009/attempt2`), release build, on the laptop with other builds running at the same time. So the timings are upper bounds, not clean benchmarks.

| Run | `solve_ms` | `disjunctive_ms` | eliminated / fixed / binary-or-fixed | tree lower bound | gap |
|---|---|---|---|---|---|
| Unit 10 | 264,832 | 1,116,550 | 6,254 / 112 / 179 | ≈ -21.02900400 | ≈ 0.00704240 |
| Unit 11 (`4754c118b`) | 200,929 | 1,809,808 | 6,395 / 115 / 185 | ≈ -21.02883804 | ≈ 0.00687644 |
| Speed-up only (`0884462c0`) | 69,140 | 146,897 | 6,395 / 115 / 185 | Unit 11's, bit-for-bit | Unit 11's, bit-for-bit |
| Head code (`82074e3ec`, sort fixed; the PR head `bb04250e7` adds only docs) | 57,057 | 148,896 | **6,406 / 115 / 186** | **≈ -21.02881252** | **≈ 0.00685092** |

- **The speed-up is exact.** At `0884462c0`, every Unit 11 value recorded in `antigravity-54/REPORT.md` matches: the tree counts, the gap, the single-split bound, the root RCBT counts and the tree lower bound `-141734088016952714342525764033724082710340210011323135211531386253129 / 6739986666787659948666753771754907668409286105635143120275902562304`. Against Unit 10, `solve_ms` is down 73.89% and `disjunctive_ms` is down 86.84%.
- **The sort fix tightens the outputs.** At the head, exactly five fields change from the speed-up-only run:
  - tree lower bound: `-4429184874126167436154938361970246199537836886330785620806862298897 / 210624583337114373395836055367340864637790190801098222508621955072`;
  - tree gain: from `+0.00042963` to `+0.00045516`, which is 1.73× Unit 9/10's;
  - gap: `1442971742586185800051125221870622573311910657828376532475109137 / 210624583337114373395836055367340864637790190801098222508621955072`, about `0.00685092` (`0.03259%`), narrower by `0.00002553`;
  - eliminated codes: 6,406, which is +11;
  - binary-or-fixed coefficients: 186, which is +1.

  Fixed coefficients stay at 115. The incumbent, root RCBT (6,122 / 112 / 168) and the single-split bound are unchanged. Against Unit 10, `solve_ms` is down 78.46% and `disjunctive_ms` down 86.66%.
- **Timing noise.** The sort fix does not touch the discrete solve, so the 12 s gap in `solve_ms` between the two runs comes from machine load.

## Checks run at the head

All of these ran in `docs/labs/legal-construction-exact-solve/solver` with `CARGO_TARGET_DIR=target/claude-3152` and `~/.cargo/bin/cargo`:
- `cargo fmt -- --check`: PASS.
- `cargo clippy --locked --all-targets -- -D warnings`: PASS.
- `cargo test --locked`: PASS. 14 targets, 50 passed, 0 failed, 1 ignored.
- `uor-harness loop report --check` for this ORDER and for `antigravity-54`'s: PASS.
- `uor-harness loop pr-check --local`: PASS.
- `uor-harness loop gates --full`: PASS, 11 of 11 green.

## Skills used

- None. Claude finished this unit without a skill run.

## Still open

- No unit test pins the sort's call site in the certifier. A small instance where scored-first order gives a different tree from both basis-row order and unscored-first order would close that.
- The certified discrete gap on `attempt2` is about `0.00685092` (`0.03259%`). #2828 stays `ready` until it is closed, or until an optimal assignment or an infeasibility proof is recorded.

## Plain summary

Antigravity's speed-up makes the exact certifier about 7.5 times faster than Unit 10 and changes none of its answers. Fixing the sort bug Kimi found tightens the certified bound a little further, and eliminates 11 more codes. Two new tests pin the sort and the dual pool, and each fails when its fault is put back.
