# ORDER: legal-construction-exact-solve, claude-54 (a bounded-precision search, proved by the exact certificate)

Lab: claude. Item: [legal-construction-exact-solve](../../../../roadmap/items/legal-construction-exact-solve.md) (#2828). Base: `1c728ff87`. Item PR (partial: `References #2828`).

## Goal

claude-50's exact simplex (`blp`) spent 7,800 s and 775 pivots on the root relaxation without finishing. Its pivots slowed as `det(B)` grew to about 22,800 bits. This unit separates the search from the proof (design: #2828, 6354250887).

The classification is settled. The search counts as an exact-arithmetic host instrument under Casey's standing direction (#2028, 6355948874), recorded on #2828 (6355973721). The result is reported only with a passing exact certificate.

1. **Search.** Run the bounded-variable revised simplex in fixed-width integer arithmetic: a dyadic with an integer mantissa of at most 128 bits and an `i32` exponent, rounding to nearest. There is no f32/f64, and `no_floats` still holds. The search proposes an optimal basis, meaning the set of basic indices plus each nonbasic variable's position.
2. **Proof.** From that basis, solve `B·x_B = b − N·x_N` and `πᵀB = c_B` exactly, by fraction-free elimination on the integer-scaled data. Then run `blp`'s certificate checks: every input row and bound, and the sign of every reduced cost.
3. **Repair.** If the certificate fails, `blp` runs exact pivots from that basis (a warm start) and re-certifies.
4. **Run** the saved case's root relaxation, loaded through `case::load_verified`, under a 7,800 s cap. Record the certified optimum and the wall time, or the timing stop.

## Cancel probe (run first)

At the base, does the crate already have a non-BigInt search, or a warm-start entry into `blp`? If it does, the task is void. Class: instrument.

## Files in scope

- `docs/labs/legal-construction-exact-solve/**`
- `docs/research/notebook/legal-construction.md`
- This task folder

## Done when

1. On the controls, the random suite and the tightened-bound LPs, the search plus certificate gives the same status and the exactly equal objective as `lp::solve`.
2. **Planted faults:**
   - a corrupted search basis (one basic index swapped) is rejected by the certificate, and the warm start recovers the exact optimum;
   - a search result with the certificate skipped fails a test.
3. **Real case:** the certified root optimum (exact rational and its sign) with its wall time, or a recorded timing stop under the cap. This goes in the lab README, with one notebook row.
4. Crate tests and clippy `-D warnings` pass. f32/f64 appear only in `exact.rs`. `report --check` and `pr-check --local` pass.

## Stop and ask if

- The search cannot reach an optimal basis on the controls at 128-bit precision.
- The certificate path would need a change to `lp` or to `case.rs`'s mapping.
