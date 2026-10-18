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

- **V0 (identity preflight, AGY 6255181282 item 2).** `rowsk-qualify` refuses to run unless the embedded `rowsk.cu` begins byte-for-byte with the embedded `rows.cu`. The receipt records `driver_source_sha256()` (expected unchanged from main, since no include-list file is touched) and the `rows` and `rowsk` image identities. V0 proves only that `rowsk.cu` extends the `rows.cu` built alongside it. It does not prove the rows image is the one qualified in #2630. For that, the posted receipt compares `rows_image_sha256` with the rows image recorded for the frozen K16 build.
- **V2 (scoring parity). PASS requires every case to agree.** For every coordinate of every block, the K-coordinate launch's sixteen losses and status must equal an independent `propose_rows` for that coordinate on the same device, carried batch and targets. The sweep:
  - complete Route + VectorBlock inventories (Route, B2 and B4, objectives A and R), and Gate + VectorBlock (Relation `{2,1}`, B2);
  - V16, C12, K = 4 cells, model seeds 7 and 11;
  - row counts 1, 7 and 64;
  - blocks of K ∈ {1, 4, 16}.

  Racy refusals (lanes ending on different statuses, published by `atomicExch` on both paths) pass only if the observed status sets overlap across 4 re-runs. The receipt records each racy case's parameter and both observed sets in full. One mismatched loss or status is FAIL. Learned state and epoch must be unchanged around every call.

  **Aliasing power (subagent review, finding 1).** Parity can only expose a coordinate reading another coordinate's slot if coordinates in the same block hold different results. So V2 records, per configuration cell (read, blocks, objective, row count, kind, K), how many multi-coordinate blocks held two or more distinct (status, losses) results. **At 64 rows, every cell with K ≥ 4 must have at least one such block; otherwise V2 FAILs as blind.** Cells are pooled over model seeds 7 and 11. A kind with fewer than two coordinates forms no multi-coordinate block and is not a cell. A blind FAIL caused by the input, for example a 64-row batch that refuses at baseline so every coordinate shares one status, is an **instrument blocker** under D22, not a kernel result. It is repaired by a one-line amendment to the batch seed, with the reason posted, and V2 is rerun in a fresh report directory.

  **Partial blocks.** V2 also scores one deliberately short block (K − 1 coordinates) per K > 1, so the partial-block path always runs. The receipt counts short blocks, and the count must be > 0.
- **V2-neg (sabotage control, pre-registered).** After the clean run, the same pod applies `docs/labs/bvgk-rowsk-scoring-2026-10-18/sabotage-reducer-offset.patch`. The patch makes the reducer write coordinate c's losses into slot c − 1. The pod rebuilds and runs `rowsk-qualify` into a fresh report directory. **V2 must FAIL**, with a losses or status mismatch or the blindness check. The source is then reverted with `git apply -R`, and the clean image identity is re-checked. If the sabotaged run PASSes, the instrument is powerless: that is an instrument blocker, and the clean result is not accepted.
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
  - **Timing scope:** wall-clock around the host-callable `Device::score_rows_k` and `Device::propose_rows`, including per-call buffer allocation, host-to-device and device-to-host copies and stream synchronization. The single-proposal baseline also includes the chooser launch and its status synchronization, so this is end-to-end call throughput. A speedup may come from launch occupancy, amortized fixed overhead or both, and the curve does not attribute it to either. There is no host model computation. The receipt states this scope.
  - **Reported per K:**
    - median and p95 (nearest-rank) block latency;
    - per-coordinate latency;
    - `speedup_x1000` against K single proposals;
    - **all raw samples retained** in the receipt.
  - **Checks:** the first coordinate's losses equal `propose_rows`, and every coordinate in each timed block has status 0. This is checked on the same block and input once before timing and once after the last timed sample, which must also reproduce coordinate 0's losses. A refused coordinate would shrink the block's work and inflate the speedup.
  - **Spread-aware reading (per K):** `low = K × single median ÷ block p95` and `high = K × single p95 ÷ block median`, both ×1000.
    - The reading is `above_2x` if low ≥ 2000.
    - It is `below_2x` if high < 2000.
    - Otherwise it is `inconclusive_within_spread`.
  - **Use:** if K = 16 reads `below_2x`, the §1 throughput hypothesis fails at this shape. `inconclusive_within_spread` is reported as such and is not read as either side. That is reported, not converted into a mechanism negative or a K selection, and Stage T is re-scoped at its own registration.
- **Stage T separation.** No Stage C result sets a Stage T quality bar, non-inferiority margin or K selection. Stage T re-bases on the K16 outcome's configuration of record and fixes its margin from that dispersion at its own registration.

## Base artifact

None. Fresh `Device::new_with_objective` models with deterministic seeds; disposable fixtures.

## Data split

None. Deterministic integer identities and targets from a fixed generator; no corpus rows.

## Steps

1. Merge this source PR after exact-head review.
2. Get TEST FITNESS from non-author seats (AGY, DeepSeek), then PM dispatch, then the owner's compute release for this finite run. Local source and build work also needs the host above its 30 GiB disk floor.
3. On one RTX 5090 via `uor-pod`, build `--features cuda` at the merged head, run the D29 checker, then `rowsk-qualify`. Then run V2-neg on the same pod.
4. Post `rowsk-qualification.json` with its SHA, the `rowsk` and `rows` image identities, and the latency curve, then archive the run.

## Stop rule

- Any FAIL stops for diagnosis (D31). It is never a negative.
- A CUDA/build fault is an instrument blocker.
- Owner caps and the disk floor (host and pod) are hard stops.
- A healthy run is renewed, not cancelled, if it outlasts its estimate.

## Cost

Under 45 min on one RTX 5090, about $0.75 ($0.56–0.91/h offers):
- the build, about 10 min;
- V2, a few thousand small-shape launches;
- the latency curve: 3 × 21 block launches plus 21 single-proposal launches, each about 130 ms or less.
- V2-neg: one sabotaged rebuild (about 10 min) and one more V2 pass.

## Minimum test power

This is finite exact parity, so there is no statistical bar:
- V2 compares every coordinate of complete inventories, and a single disagreement fails it;
- the latency curve is descriptive and reported with its sample count.

TEST FITNESS: FIT (owner direction 2026-10-18 in the Claude chat: Claude's own fresh-context adversarial reviewer subagents replace cross-lab FIT for Claude lanes. Three read-only reviews — kernels/launch/parity, build/D29/V2-power/refusal text/driver hash, and the fix diff — returned FIT-WITH-FIXES; every finding is applied in this revision. Posted on #2029.)
