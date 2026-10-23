# REPORT: legal-construction-exact-solve, claude-54 (a bounded-precision search, proved by the exact certificate)

Lab: claude. Order: [ORDER.md](ORDER.md). Item #2828 (`References`; the item stays open). Base: `1c728ff87`.

## What did not work, first

- **The root search did not reach an optimal basis within the 7,800 s cap.**
  - The run started 2026-10-23T03:08:21Z and ended 05:18:21Z with `timeout` exit 124: 48,025 pivots, all in phase 2, 37 bound flips.
  - The objective was negative from pivot 350 on, and its magnitude stayed in one power of two (`⌈log₂|z|⌉` = 5) from pivot 14,000 to the end.
  - No basis was proposed, so the certificate never ran. No optimum, and no sign of one, is claimed.
  - This is an instrument stop (D22), not a mechanism result.
- **The stall is a reading, not a measurement.** Degenerate stretches under Dantzig pricing with Bland's fallback would explain it, but `SearchStats` counts no degenerate steps or Bland switches. Counting them comes first in the next unit.
- **The run's binary predates the repair commit.** `root-search` at `2115180e1` runs search then certificate. The repair (`14a458c46`) is library code, tested on the whole suite, and the binary does not call it. That changes nothing here, because the search proposed no basis to repair.
- **The late pivot rate is confounded.** From about 6,500 s on, this session's own cargo builds shared the CPU. The rate fell from about 10 to about 2.5 pivots per second over the last 1,800 s, and that drop is not a property of the search.
- **The suite compares against `blp`, not `lp`.** Every exact objective in `tests/search_certify.rs` is checked against `blp::solve`. `tests/blp_agrees.rs` pins `blp` to `lp` exactly on the same controls, random suite and tightened-bound LPs. The ORDER's "equal to `lp::solve`" therefore holds through that link, not directly.

## What changed

| File | Change |
|---|---|
| `solver/src/dyadic.rs` | `Dy<P>`: an `i128` mantissa of at most `P` bits and an `i32` exponent, rounded to nearest once per operation; exact conversion to and from `BigRational`. |
| `solver/src/blp.rs` | `Basis` and `Pos`; `certify_basis` (exact Bareiss solve of `B·x_B = b − N·x_N` and `πᵀB = c_B`, then `blp`'s certificate); `solve_from_basis` (warm start); `solve_with_basis`; the `Start` enum. |
| `solver/src/search.rs` | `search::<P>` (the bounded-variable revised simplex on `Dy<P>`); `solve_search_certify`; `solve_search_certify_repair` with `RepairPath`. |
| `solver/src/bin/root-search.rs` | Runs the search on the saved case's root relaxation, then certifies. |
| `solver/tests/certify_basis.rs`, `solver/tests/search_certify.rs` | The controls and the planted faults (below). |
| `docs/labs/legal-construction-exact-solve/README.md` | The "Unit 3b" section. |
| `docs/research/notebook/legal-construction.md` | One row. |
| this folder | The ORDER (committed first, alone) and this REPORT. |

## Measurements

- **The root run:** the numbers above, with the full account in the lab record's Unit 3b.
  - The log is `local/claude/legal-case/root-search-run1.log` (1,921 progress lines; not committed).
  - `blp` ran 775 pivots in the same 7,800 s, so the search's pivots are about 60 times cheaper.
- **Tests** (debug, at `14a458c46`):

  | Suite | Passed |
  |---|---|
  | lib | 7 |
  | `blp_agrees` | 3 |
  | `case_loader` | 6 |
  | `case_verified` | 4 |
  | `certify_basis` | 3 |
  | `controls` | 11 |
  | `no_floats` | 1 |
  | `random` | 2 |
  | `search_certify` | 6 |

- **Done-when 1:** `p120_certifies_every_optimal_lp_exactly` certifies every optimal LP in the suite (more than 500) with the exact `blp` objective, and every infeasible one is an error.
- **Done-when 2:**
  - `planted_corrupted_basis_is_repaired_exactly` swaps one basic index, checks that the certificate rejects the result, and checks that `solve_from_basis` reaches `blp`'s exact optimum.
  - `p12_never_returns_a_wrong_optimum` shows that at 12-bit precision, where the search alone is unreliable, the certificate still never lets a wrong optimum through.
  - **Negative gate for the certificate, run at `8c69ea3cd`:** I made `solve_search_certify` return the search's own point and objective with no certificate (`search.rs:562`).
    - `p12_never_returns_a_wrong_optimum` and `p120_certifies_every_optimal_lp_exactly` both failed with "wrong optimum" on the first LP (0 passed, 2 failed).
    - Even at 120 bits the search's own objective is not the exact one, so the certificate carries the result.
    - I restored the line, and `git diff` is clean.
- **Done-when 4:** `no_floats` passes, so `f32` and `f64` appear only in `exact.rs`.

## Checks run at the head

- **Crate tests:** 43 passed, 0 failed, 1 lib test ignored by design. `cargo test --locked` in `solver/`.
- **Lint:** `cargo clippy --locked --all-targets -- -D warnings` is clean.
- **Format:** `cargo fmt -- --check` is clean. That needed one rustfmt-only commit: `root-search.rs`, `search.rs` and `search_certify.rs` had line breaks rustfmt wanted changed.
- **Loop:** `loop report --check --order` and `loop pr-check --local` (results in the PR checks).

## Skills used

- `solution-integrity`: a result is reported only with a passing exact certificate. When the search proposed nothing, the record says so and claims no sign.
- `scope-guard`: `lp` and `case.rs`'s mapping are untouched. The search, the certificate path and the repair live in the lab crate.

## Still open

- **Measure the stall:** count degenerate steps and Bland switches in `SearchStats`, then re-run the root search.
- **A degeneracy-aware rule inside the search:** bound perturbation or steepest-edge pricing on `Dy`. The certificate keeps the result exact either way.
- **Branch-and-bound** decides the item if the relaxation optimum is below 0, which both runs' negative phase-2 objectives point to without proving it.

## Plain summary

- A fast, float-free search now proposes a basis, and an exact certificate proves or rejects it. A rejected basis is repaired by exact pivots. On every test LP the answer equals the exact solver's.
- On the real case, the search did about 60 times as many pivots as the exact solver in the same 2 h 10 min, but still did not finish. The objective stayed negative and nearly flat, which suggests the search is stuck in degenerate steps. Nothing about the case's optimum is claimed.

References #2828
