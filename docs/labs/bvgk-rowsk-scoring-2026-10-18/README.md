# BVG-K Stage C, piece 1: K-coordinate row-parallel scoring — 2026-10-18

- **Lab:** Claude, W3 (BVG-K design owner per #2029 6254942582).
- **Status:** opt-in source plus a GPU qualifier. The **GPU run is NOT_RUN**: it waits for TEST FITNESS on `docs/harness/prep/m1-bvgk-stagec-rowsk-qualification-2026-10-18.md` and the owner's compute release. No compute, $0 so far.

## Why

The accepted K16 timing record (`3efb4768…`) measures about **130 ms per proposal** at 64 rows, which is about 99.6% of each batch. The heavy `rows_v1` evaluation launch is `rows × 16 = 1,024` threads in blocks of 128. That is **8 thread blocks, so at most 8 of an RTX 5090's 170 SMs** do work.

BVG-K's hypothesis is that scoring K coordinates in one launch raises proposal throughput by up to about K×. This piece builds the scoring path and the instrument that measures it.

## Change

- **`rowsk_source.rs`** generates `rowsk.cu` as the **unchanged generated rows text** plus two entries:
  - `discrete_counterfactual_rowsk_eval`, one thread per (coordinate, row, lane);
  - `discrete_counterfactual_rowsk_reduce`, which replays the row-parallel lane loop **verbatim** per (coordinate, lane), with one status word per coordinate. A host test pins that the loop text is identical.
- **`build.rs`** builds `rowsk.ptx` and `rowsk.cubin` with their own 5-entry PTX admission.
- **D29:** `scripts/check_native_cuda_vendor.rs` adds the rows-K census and its negative self-tests.
- **`Device::score_rows_k`** (`rowsk_device.rs`) scores 1..=64 parameters of one kind:
  - one shared baseline launch, one eval launch and one reduce launch;
  - no chooser and no apply;
  - an unchanged-epoch check.
- **`rowsk-qualify`** runs V2 (per-coordinate parity with `propose_rows` over complete inventories, K ∈ {1, 4, 16}), V6 (block refusals) and the K latency curve.

**Unchanged:** the rows image, every default path, and `driver_source_sha256`. No file in its include list is touched.

## Checks (host)

- `uor-r4-discrete` lib tests pass 103/103, including 3 new `rowsk_source` tests.
- The stub-nvcc `--features cuda --all-targets` type check passes.
- Clippy (cuda lib plus the qualifier binary) is clean for the new files, and so is `fmt`.
- The D29 checker `--self-test` passes, and `--repo .` passes with the 11+1+1+3+5 entry census.

## Next

1. Get FIT on the Stage C sheet, then the owner's compute release.
2. Run `rowsk-qualify` on one 5090.
3. If V2 passes, the latency curve decides whether Stage T proceeds as designed: a K = 16 speedup of at least 2× is the threshold.
4. Then build piece 2: the batched chooser, block apply, cursor `o` and `coordinate_block`.

## Review

A non-author adversarial review returned APPROVE-WITH-NITS, with no correctness defect in the kernels or the host launch: offsets, buffer sizes, argument order, refusal order, shadowing and parameter validation were all checked. All three qualifier nits are applied:
- the racy-refusal retries now sit inside the unchanged-state check, and the record carries the observed set sizes;
- each V6 refusal must carry its expected message;
- V6 adds a Route-in-Relation case and a Gate-past-count case.

## FIT fixes (follow-up PR)

DeepSeek (6255083238, 6255181282) and AGY (6255142621, plus a piece-1 review at 21:13Z) returned FIT-WITH-FIXES. Applied:

- **Latency protocol.** The qualifier now makes **1 excluded warmup + 20 timed** launches per K and reports the median (`times[10]`) and the nearest-rank p95. All raw samples are kept in the receipt, and the single-proposal reference gets the same treatment. All K use the same seed-1, checkpoint-0 state on the same pod.
- **V0 identity preflight.** `rowsk-qualify` refuses to run unless the embedded `rowsk.cu` begins byte-for-byte with `rows.cu` (`rowsk_extends_rows_source`). The receipt records `driver_source_sha256()` and states the timing scope: host-callable calls, including allocation, copies and stream synchronization.
- **V7.** The scans and receipts are now explicit. The D29 checker's stdout and its SHA are saved beside the receipt.
- **Parent sheet.** Now has per-family cursors `(o_vec, o_read)`, explicit tie-breaks (lowest ordinal wins; S_t is committed only if `L(S_t) < L({best})`), V1 cursor reproduction, V4 tie fixtures and V10 opt-in identity. Stage C is split into C1 (scoring) and C2 (block apply). No Stage C result sets a Stage T bar.

## Subagent FIT review and fixes (owner direction, 18 October)

The owner directed that Claude's own fresh-context adversarial reviewer subagents replace cross-lab FIT for Claude lanes. Three read-only reviews (kernels, launch and parity; build, D29, V2 power, refusal text and driver hash; the fix diff) each returned FIT-WITH-FIXES with no blocker. Every finding is applied, and the sheet is **FIT**.

**Latency curve:**
- Every coordinate of each timed block must have status 0, checked before and after timing.
- The 2× threshold has a spread-aware reading: `above_2x`, `below_2x` or `inconclusive_within_spread`.
- The scope is end-to-end call throughput. The speedup is not attributed to occupancy over fixed overhead.

**V2 parity:**
- **Aliasing power:** at 64 rows, every K ≥ 4 configuration cell must contain a block whose coordinates' results differ.
- **Short blocks:** a deliberate short block per K > 1, so the partial-block path always runs.
- **Racy refusals:** both refusal sets are recorded in full.

**Sabotage control (V2-neg).** `sabotage-reducer-offset.patch` makes the reducer write coordinate c's losses into slot c − 1. On the pod, after the clean run, the patched build must make V2 FAIL. A PASS would show the instrument is powerless.

**V0 scope.** The prefix check proves only that `rowsk.cu` extends `rows.cu`. The rows image identity is compared against the frozen K16 build in the posted receipt.

## Result: GPU qualification run 1 — PASS (2026-10-18, one RTX 5090)

- **Source and pod:** merged head `b2d6c7c63cea8715e3d3be9f68a0b820fb78f65f` (#2710). Vast pod `vast-56628021`, RTX 5090, nvcc 12.8. Bootstrap parity PASS. About 17 min of pod time, about $0.27.
- **Status:** NOT_TRAINING; zero applied updates; no corpus.
- **Evidence:** `evidence/run1/`, with `SHA256SUMS`, the job script, the job log and both receipts. The pod workspace is also archived to the HF store under `vast-archive/vast-56628021/`, with read-back verified.

| Control | Outcome |
|---|---|
| V0 preflight | `rowsk.cu` extends `rows.cu`. The rows source, PTX and cubin hashes (`48d8e6e0…`, `c03752ab…`, `9d0f170b…`) equal every qualified #2630 rows-parity attempt; the fourth hash is the resource-report text, which differs between all of those attempts. `driver_source_sha256` = `f0dd75c1…` (unchanged by this work). |
| V2 scoring parity | **PASS. 5,130 / 5,130 coordinates** matched `propose_rows` exactly (losses and status), with 0 refusals and 0 racy cases. 132 short blocks ran, and the aliasing-power requirement held in every 64-row K ≥ 4 cell. |
| V6 block refusals | **PASS, 6 / 6**, each with its expected message and state unchanged. |
| V2-neg sabotage | **FAIL, as required** (`SABOTAGE_RC=1`): `Route(3): losses differ: rows [448 ×16] rows-K [0 ×16]`. That is the predicted signature: the last coordinate's slot is left unwritten. After the revert, the image is byte-identical to the clean image (`image-clean.txt` = `image-reverted.txt`). |
| D29 | Self-test and repo PASS. The output is identical for the clean and sabotaged sources. |
| Receipt | `clean/rowsk-qualification.json`, SHA-256 `d5ec5348d64e2abd74a334319d4f78e6ed7b174b5a2fc0bb381505dc46e09e56`. |

**Latency curve** (V4096/B4/C192/K16 Route objective A, seed 1, checkpoint 0, 64 rows; 1 warmup + 20 timed; end-to-end call latency):

| K | block median | block p95 | speedup ×1000 (low–high) | reading |
|---|---:|---:|---:|---|
| single `propose_rows` | 125.51 ms | 125.60 ms | — | — |
| 1 | 123.53 ms | 123.61 ms | 1,015 (1,015–1,016) | below_2x |
| 4 | 123.46 ms | 123.50 ms | 4,066 (4,065–4,069) | above_2x |
| 16 | 123.57 ms | 123.62 ms | **16,249 (16,243–16,262)** | **above_2x** |

**Reading.** At this shape, the cost of scoring is flat in K up to at least 16: 16 coordinates cost the same as one. Stage T's throughput hypothesis therefore holds at this shape. Per the sheet, this is end-to-end throughput: it does not say whether the flat cost is idle SM capacity or fixed per-call overhead. No Stage T bar, margin or K selection follows from it. **Next:** Stage C2, the block apply (cursors, tie-breaks and V1/V3–V5/V8–V10). Measuring K beyond 16, up to the cap of 64, is a candidate for C2's latency curve.
