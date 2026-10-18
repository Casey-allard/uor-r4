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

- **V0 (identity preflight, AGY 6255181282 item 2).** `rowsk-qualify` refuses to run unless the embedded `rowsk.cu` begins byte-for-byte with the embedded `rows.cu`. The receipt records `driver_source_sha256()` (expected unchanged from main, since no include-list file is touched) and the `rows` and `rowsk` image identities.
- **V2 (scoring parity). PASS requires every case to agree.** For every coordinate of every block, the K-coordinate launch's sixteen losses and status must equal an independent `propose_rows` for that coordinate on the same device, carried batch and targets. The sweep:
  - complete Route + VectorBlock inventories (Route, B2 and B4, objectives A and R), and Gate + VectorBlock (Relation `{2,1}`, B2);
  - V16, C12, K = 4 cells, model seeds 7 and 11;
  - row counts 1, 7 and 64;
  - blocks of K ∈ {1, 4, 16}.

  Racy refusals (lanes ending on different statuses, published by `atomicExch` on both paths) pass only if the observed status sets overlap across 4 re-runs. One mismatched loss or status is FAIL. Learned state and epoch must be unchanged around every call.
- **V6 (block refusals).** Six blocks must each be refused before any launch, with its own expected message and with state unchanged: empty, over-long (65), mixed-kind, out-of-inventory, a Route parameter in a Relation model, and a Gate past the gate count.
- **V7 (compliance scans, explicit).**
  - The D29 checker (`scripts/check_native_cuda_vendor.rs`, `--self-test` and `--repo .`) must PASS at the run head. That covers the D23/D25/D30 token scan (no float, squash or matmul tokens) and the exact 5-entry `rowsk.cu` census.
  - The build's `admit_ptx_inventory` must admit the 5-entry PTX.
  - The receipt records `applied_updates: 0`, zero corpus updates and no checkpoint promotion.
  - The D29 checker's full stdout and its SHA-256 are saved beside `rowsk-qualification.json` in the report directory, so the run's compliance pass is part of the receipt.
- **Latency-curve protocol (pre-registered; diagnostic for Stage T, never a gate).**
  - **Shape:** V4096 / B4 / C192 / K16, Route, objective A, model seed 1, freshly initialized at checkpoint 0. The same model state and the same pod are used for every K. 64 deterministic rows.
  - **Blocks:** K ∈ {1, 4, 16}, with block = `VectorBlock(0..K)`.
  - **Samples:** per K, **1 excluded warmup + 20 timed** synchronized launches. The reference is the same for single `propose_rows(VectorBlock(0))`.
  - **Timing scope:** wall-clock around the host-callable `Device::score_rows_k` and `Device::propose_rows`, including per-call buffer allocation, host-to-device and device-to-host copies and stream synchronization. There is no host model computation. The receipt states this scope.
  - **Reported per K:**
    - median and p95 (nearest-rank) block latency;
    - per-coordinate latency;
    - `speedup_x1000` against K single proposals;
    - **all raw samples retained** in the receipt.
  - **Check:** the first coordinate's losses equal `propose_rows`.
  - **Use:** if the K = 16 speedup is below 2000 (2×), the §1 occupancy hypothesis fails at this shape. That is reported, not converted into a mechanism negative or a K selection, and Stage T is re-scoped at its own registration.
- **Stage T separation.** No Stage C result sets a Stage T quality bar, non-inferiority margin or K selection. Stage T re-bases on the K16 outcome's configuration of record and fixes its margin from that dispersion at its own registration.

## Base artifact

None. Fresh `Device::new_with_objective` models with deterministic seeds; disposable fixtures.

## Data split

None. Deterministic integer identities and targets from a fixed generator; no corpus rows.

## Steps

1. Merge this source PR after exact-head review.
2. Get TEST FITNESS from non-author seats (AGY, DeepSeek), then PM dispatch, then the owner's compute release for this finite run. Local source and build work also needs the host above its 30 GiB disk floor.
3. On one RTX 5090 via `uor-pod`, build `--features cuda` at the merged head and run the D29 checker, then `rowsk-qualify`.
4. Post `rowsk-qualification.json` with its SHA, the `rowsk` and `rows` image identities, and the latency curve, then archive the run.

## Stop rule

- Any FAIL stops for diagnosis (D31). It is never a negative.
- A CUDA/build fault is an instrument blocker.
- Owner caps and the disk floor (host and pod) are hard stops.
- A healthy run is renewed, not cancelled, if it outlasts its estimate.

## Cost

Under 30 min on one RTX 5090, about $0.50:
- the build, about 10 min;
- V2, a few thousand small-shape launches;
- the latency curve: 3 × 21 block launches plus 21 single-proposal launches, each about 130 ms or less.

## Minimum test power

This is finite exact parity, so there is no statistical bar:
- V2 compares every coordinate of complete inventories, and a single disagreement fails it;
- the latency curve is descriptive and reported with its sample count.

TEST FITNESS: PENDING
