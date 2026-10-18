# BVG-K Stage C, piece 1: K-coordinate scoring parity and latency curve — pre-registration

- **Line:** M1 W3 BVG-K (#2029). **Lab:** claude. **Parent design:** `m1-batched-verified-coordinates-DRAFT-2026-10-18.md`, §8 Stage C.
- **What runs:** `rowsk-qualify REPORT_DIR` from the opt-in source in this PR. It is finite GPU controls only:
  - no corpus;
  - no training, and **zero applied updates**;
  - nothing that any driver path calls.
- **Rules:**
  - D22, D23/D25/D30 (integer CUDA only, with the PTX inventory admitted and the D29 census extended to `rowsk.cu`);
  - D24 (GPU only; host code compares results only);
  - D29;
  - D40 (no owner phrase is involved; this is not a training release).

## Bar

- **V2 (scoring parity). PASS requires every case to agree.** For every coordinate of every block, the K-coordinate launch's sixteen losses and status must equal an independent `propose_rows` for that coordinate on the same device, carried batch and targets. The sweep:
  - complete Route + VectorBlock inventories (Route, B2 and B4, objectives A and R), and Gate + VectorBlock (Relation `{2,1}`, B2);
  - V16, C12, K = 4 cells, model seeds 7 and 11;
  - row counts 1, 7 and 64;
  - blocks of K ∈ {1, 4, 16}.

  Racy refusals (lanes ending on different statuses, published by `atomicExch` on both paths) pass only if the observed status sets overlap across 4 re-runs. One mismatched loss or status is FAIL. Learned state and epoch must be unchanged around every call.
- **V6 (block refusals).** Empty, over-long (65), mixed-kind and out-of-inventory blocks are refused before any launch, with state unchanged.
- **Latency curve.** It is reported and does not gate. At V4096 / B4 / C192 / K16 Route objective A, 64 rows:
  - the block median latency at K ∈ {1, 4, 16} (1 warmup + 5 timed);
  - the per-coordinate latency;
  - `speedup_x1000` against K single `propose_rows`;
  - a check that the first coordinate's losses equal `propose_rows`.

  **Decision use:** if the K = 16 speedup is below 2000 (2×), the §1 occupancy hypothesis fails at this shape. Stage T is then re-scoped before any training sheet; a low speedup is not a mechanism negative.

## Base artifact

None. Fresh `Device::new_with_objective` models with deterministic seeds; disposable fixtures.

## Data split

None. Deterministic integer identities and targets from a fixed generator; no corpus rows.

## Steps

1. Merge this source PR after exact-head review.
2. Get TEST FITNESS from non-author seats (AGY, DeepSeek), then the owner's compute release for this finite run.
3. On one RTX 5090 via `uor-pod`, build `--features cuda` at the merged head and run the D29 checker, then `rowsk-qualify`.
4. Post `rowsk-qualification.json` with its SHA, the `rowsk` and `rows` image identities, and the latency curve, then archive the run.

## Stop rule

- Any FAIL stops for diagnosis (D31). It is never a negative.
- A CUDA/build fault is an instrument blocker.
- Owner caps and the disk floor are hard stops.

## Cost

Under 30 min on one 5090, about $0.50:
- the build, about 10 min;
- V2, a few thousand small-shape launches;
- the latency curve, about 3 × 6 launches at about 130 ms or less each.

## Minimum test power

This is finite exact parity, so there is no statistical bar:
- V2 compares every coordinate of complete inventories, and a single disagreement fails it;
- the latency curve is descriptive and reported with its sample count.

TEST FITNESS: PENDING
