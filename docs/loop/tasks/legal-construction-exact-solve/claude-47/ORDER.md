# ORDER: legal-construction-exact-solve, claude-47 (the exact solver and its finite controls)

Lab: claude. Item: [legal-construction-exact-solve](../../../../roadmap/items/legal-construction-exact-solve.md) (#2828). Base: `18c03cebc`. Item PR (partial: `References #2828`; the item stays `ready`).

## Goal

This is unit 2 of the design note on #2828 (6351533475), under Kimi's 14:15Z classification: an exact host solve is an instrument, with zero floats.

The unit builds a standalone Rust crate at `docs/labs/legal-construction-exact-solve/solver/`, detached from the workspace and using only the cached `num-bigint` 0.4 and `num-rational` 0.4. It contains:
- an exact rational revised simplex, with Bland's rule;
- depth-first branch-and-bound over the bounded integer codes `k` and the `l`/`r` binaries, with a node cap and the incumbent `δ = 0`;
- an exact problem loader for f32 inputs, converting each bit-exactly to a dyadic rational and refusing NaN or infinity;
- a separate exact re-check of every guard row and the objective.

It does not read or solve the saved case. That is unit 3.

## Cancel probe (run first)

At `18c03cebc`, does an exact solver already exist under `docs/labs/legal-construction-exact-solve/`? If one does, the task is void. Class: instrument.

## Files in scope

- `docs/labs/legal-construction-exact-solve/**`
- This task folder

## Done when

1. **Brute-force agreement.** The finite controls pass: the solver agrees with brute-force enumeration over keep/lower/higher, up to about 6 coefficients, on seeded random dyadic instances with planted guards. They include hand-solved instances, the legal endpoints `q = ±7`, the noop branch, all-zero guard rows, and a guard that forbids every move.
2. **Planted faults are caught.** A flipped guard sign, an off-by-one in the `k` bounds and a dropped disjunction row each fail a test.
3. **No floats.** `f32`/`f64` appear only in the bit-exact input conversion. A grep test checks this.
4. **The checks pass:** crate tests, clippy `-D warnings`, `report --check` and `pr-check --local`.

## Stop and ask if

- Exact LP solves at small size already take minutes, which would mean the saved-case solve needs a different method. Record the timing and stop.
