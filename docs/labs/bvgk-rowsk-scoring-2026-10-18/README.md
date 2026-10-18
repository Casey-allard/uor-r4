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
