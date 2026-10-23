# REPORT: legal-construction-exact-solve, deepseek-57 (exact single-coefficient census and a feasible legal change)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Item #2828. Base: `2035770c9`. `References #2828`; the item stays `ready`.

## What did not work, first

- **The prior instruments are still blocked and this unit does not repair them.** The exact dense tableau returned no result in 120 s; the bounded-variable revised simplex ran 7,800 s without a basis; the bounded-precision search took 48,025 pivots and returned no basis to certify. The recorded next step was a structural reduction, which this unit follows.
- **The first idea (dual certificate for the zero move) was dropped on evidence.** A nonnegative `y` with `J^T y = g` would prove `g.delta >= 0` for every feasible `delta` and make the zero move optimal. claude-50's phase-2 objective had already gone negative, and this unit exhibits a feasible `delta` with `g.delta < 0`, so no such certificate exists.
- **Single-coefficient moves barely exist.** The exact census shows 1,883 of 1,920 columns carry both signs, so only 37 columns permit any single-coefficient move at all; the joint pass is what produced the result.
- **The first joint version overclaimed a stop condition, and independent replication caught it.** It skipped already-chosen coefficients and then printed 'no improving move remains'; a fresh-context reviewer independently reproduced every number from the raw binary32 files with its own bit-exact rational conversion and its own encoding, confirmed the 41-move assignment's objective, guard minimum, zero-guard count and legality exactly, but showed 21 feasible improving re-destinations of already-moved coefficients existed (the best alone reached about -0.72195). The search now permits re-destinations and reports only a single-coefficient local optimum; the independent check now compares the full 380-entry guard vector, not just its minimum.

## What changed

| File | Change |
|---|---|
| `docs/labs/legal-construction-exact-solve/solver/src/bin/single-move.rs` | new exact instrument: legal-destination enumeration, column sign census, exact best single move, greedy joint pass with re-destinations and exact rational slack, per-column exact feasible interval, full-vector independent recomputation from `g` and `J`, assignment output; two unit tests |
| `docs/labs/legal-construction-exact-solve/README.md` | Unit 4 record: method, census, best single move, the corrected 48-move result on 40 coefficients, independent replication and the overclaim it fixed, limits (no optimality proof), re-run commands |
| `docs/research/notebook/legal-construction.md` | one result row for this unit (corrected numbers) |
| this folder | ORDER and REPORT |

## Measurements

- Load: 1,920 coefficients, 380 guards, 20 fractional coefficients, 384/384 archive hashes verified, 1.68 s.
- Census (0.11 s): 1,883 both-sign columns, 11 positive-only, 17 negative-only, 9 all-zero; 37 columns admit a single-coefficient move.
- Best single move: coefficient 959, code `q = 7`, `delta = 2/1`, objective change `-8791175/67108864` (about -0.131), exact guard minimum `0/1`.
- Greedy joint pass with re-destinations: **48 accepted moves on 40 distinct coefficients, no improving single-coefficient re-destination remains**, objective **-426088804491/549755813888 (about -0.775051)**, exact guard minimum `0/1`, **320 of 380 guards exactly active**, 48.0 s including the independent recomputation.
- Independent recomputation from `g` and `J`: objective and the **full guard vector** identical (`full_guard_vector_matches=true`); second run identical assignment and numbers.
- Independent replication of the earlier numbers by a fresh-context reviewer: exact match on objective, min guard, zero-guard count, moved count, legality of all moved (q, delta) pairs, census, best single move, and 384/384 receipts.
- Crate tests: all suites pass including the two new tests; build clean.

## Checks run at the head

`cargo test --locked` and `cargo build --locked --bin single-move` in the standalone solver crate; `single-move` on the saved case with the full-vector independent recomputation and a determinism rerun; independent reviewer replication from raw files; `uor-harness ratchet d23/d25/d32` (baseline); `loop report --check`; `loop pr-check --local`; counted non-author record needed (code PR).

## Skills used

- `solution-integrity`: the cheap dual-certificate idea was tested and discarded on evidence; a feasible move is reported as an upper bound, never as an optimum; the independent replication was treated as authoritative and its counterexample fixed the search; the assignment is recomputed over the full guard vector and determinism rerun.
- `scope-guard`: only the item's lab folder and notebook row changed; archived code untouched; no Python committed; no bar/status/hold touched.

## Still open

- **No optimality or infeasibility proof, so the item stays `ready`.** The greedy pass is a single-coefficient local search; a multi-coefficient joint improvement may exist. Next step: a branch-and-bound seeded by this incumbent with an exact LP/dual bound, or a face-reduction argument from the 320 active guards.
- The actual commit rule (native scoring, 17 reference cases, strict-loss lowering) is not evaluated here; this unit only solves the saved guard-constrained linear problem. Training and rental remain HELD.

## Plain summary

- An exact sign-test census plus a greedy exact-slack search with re-destinations found a legal move on the saved legal-construction case: 48 accepted moves on 40 coefficients, objective about -0.77505, every one of the 380 guard rows exactly satisfied (320 tight), full guard vector independently recomputed and deterministic. An independent reviewer replicated the raw-file arithmetic and caught the earlier version's false local-optimum claim, which is fixed. It is a feasible improving assignment and an upper bound on the discrete optimum, not an optimum; the item stays open.