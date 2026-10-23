# ORDER: legal-construction-exact-solve, deepseek-57 (exact single-coefficient census and a feasible legal change)

Lab: deepseek. Item: [legal-construction-exact-solve](../../../../roadmap/items/legal-construction-exact-solve.md) (#2828). Base: `2035770c9`. Partial unit: `References #2828`; the item stays `ready` because no optimum or infeasibility proof is produced.

## Goal

The item's done-when is an exact optimal assignment or an infeasibility proof for the saved 1,960-variable case, which prior units could not reach (exact dense tableau >120 s; bounded-variable revised simplex 7,800 s with no basis; bounded-precision search no basis). This unit takes the recorded next step (structural reduction exploiting the guards) as far as it goes: exact single-coefficient legality is a sign test, so the unit reports the exact legal-direction census, the exact best single move, and a greedy joint pass with exact rational slack that re-verifies every guard and produces a feasible legal change if one exists.

## Cancel probe (run first)

Class instrument (PM ruling on #2828, 6342969530; owner direction 6355973721). Host-only, no GPU. At the base, the solver crate has no `single-move` binary and no exact statement about single-coefficient legal moves; a run of the existing instruments on the saved case returns no result within their caps. Task stands.

## Files in scope

- `docs/labs/legal-construction-exact-solve/**` (new `solver/src/bin/single-move.rs`, lab record), `docs/research/notebook/legal-construction.md`, this task folder. The archived code stays read-only; no Python is added.

## Done when

- `single-move` loads the saved case verified (384/384 hashes), computes the exact legal-direction census (columns allowing positive only / negative only / both / all-zero), and the exact best single move.
- It runs a greedy joint pass with exact rational slack, applies only moves whose full 380-row guard check is exactly non-negative at the current slack, and re-verifies every accepted move.
- The returned assignment is recomputed from scratch from `g` and `J`; the independent objective and guard minimum must equal the incremental ones exactly.
- The result is recorded honestly: a feasible legal change with its objective is progress and an upper bound (optimum <= that objective), not optimality or infeasibility; the item stays `ready` and training stays HELD.
- Unit tests for the destination set and the sign census; the crate's finite controls and tests still pass; ratchets stay at baseline; `loop report --check` and `loop pr-check --local` pass; a counted non-author record lands it.

## Stop and ask if

- Any file outside the item scope must change, any GPU work is needed, or the result would need a bar/status/hold change (decision card to the PM/owner, not this unit).