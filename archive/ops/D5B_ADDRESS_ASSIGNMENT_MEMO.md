# D5-b — codebook address assignment as a QUBO (design memo, not run)

**Date:** 2026-10-18
**Status:** design memo only. Nothing here was run; it needs their memory-store data (AERM keys) which we do not have.
**Adoption note (2026-10-18):** the D5-a/D5-b split this memo proposes is **adopted as [D38](../../docs/integration/DECISIONS.md#d38--d5-split-d5-a-residency-and-d5-b-sparsity-owner-2026-10-18)**. The QUBO formulation below remains an unimplemented instrument proposal: the co-access matrix `A[k1,k2]` does not exist, the `D[c1,c2]` distance term is defined but never used in the objective, and the codec simulated-annealing solver (`xtask/src/qubo_codec.rs`) solves a different (per-weight rounding) problem — no D5-b assignment solver exists yet. `ops/braket_qaoa_toy_receipt.json` is a Bell-state parity check, not QAOA, and is unverified.
**Attribution:** the formulation is standard quadratic assignment; the QUBO solver used elsewhere in this engagement is classical simulated annealing (SIMULATED). No quantum device expedites this today.

## The problem

Their D5 invariant wants per-token parameter reads to drop below dense streaming — "selected access." One candidate their roadmap holds (`#1437`, NOT_RUN): address a learned sparse parameter memory by **fixed H4/E8 codebook cells** instead of learned keys. The open question is whether a fixed codebook can assign memory keys to cells with collisions as low as learned keys, at the same access budget.

## Formulation

Given `N` memory keys and `C` codebook cells (for 2I, `C = 120`; for E8, `C = 240`):

- **Variables:** `x[k, c] ∈ {0, 1}` — key `k` assigned to cell `c`.
- **Constraint:** every key to exactly one cell (`Σ_c x[k, c] = 1` for all `k`).
- **Objective:** minimize collisions weighted by co-access frequency. If `A[k1, k2]` is how often keys `k1, k2` are read in the same turn, and `D[c1, c2]` is the geometric distance between cells (Hamming for H4, shell distance for E8), the cost of a collision is high when frequently co-read keys land in the same cell:

```
min  Σ_{k1<k2}  A[k1, k2] · Σ_c x[k1, c] · x[k2, c]
```

This is a quadratic assignment problem, exactly a QUBO in the `x[k, c]` (the one-hot constraints become quadratic penalties).

## Why it stays a memo

1. It needs `A[k1, k2]` (their actual co-access statistics from the memory store's development panels) and the access budget. We have neither.
2. The solver is the same classical SA as the codec QUBO (`xtask/src/qubo_codec.rs`); there is no quantum advantage to chase here.
3. Their own D5-b candidate is parked behind "a served model answers at least half of development turns" (D18 re-entry). The formulation is ready for when that gate opens.

## What we hand over

- The formulation above (one page).
- The solver shape: one-hot QUBO, classical SA, per-key flip moves, deterministic seed.
- The measurement that would decide it: collision rate and co-access-weighted collision cost of the fixed codebook vs. their learned keys on the same development panel, at the same access budget — reported as a D12 evidence row, not a veto.
