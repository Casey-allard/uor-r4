# REPORT: written-state-count-model, claude-423 (the counter's phase profile)

Lab: claude. Order: [ORDER.md](ORDER.md).

## What did not work, first

- **Host work dominated the recorded order intervals** [claude run, codex readback]. Five host operations total **1,186.821 s (65.3%)** of **1,816.625 s**. Four intervals around CUDA-driving routines total **490.724 s (27.0%)**, including host allocations and launches, scalar reads during scans and waits for the stream. Device-to-host copies took 90.640 s. The remaining 48.440 s is unattributed inside the order timers. No device-event or kernel-only time was measured.
- **Selection repeatedly scans TRAIN.** Orders 0–9 each made 256 partition counts and spent 23.458–26.881 s in selection. Order 10 made 128 counts and spent 13.761 s. Selection took 260.940 s across orders, compared with 198.145 s around radix sort.
- **Extension counting repeats work.** At orders below N, counting order k+1 for extension runs took 374.996 s. Those outer intervals overlap the inner count phases.
- **Run provenance [codex].** The ORDER's procedure specified an N = 10 slice and G3 specified a full pass. The packet shows full DEV-tune. The prior [non-author FIT](../../../../harness/prep/r-star-sealed-2026-11-17.md) and the [second FIT](../../../../harness/prep/r-star-sealed-2-2026-11-17.md) cover claude-422. No contemporaneous FIT covering this additional `c423-profile` scoring pass was located in the reviewed records. Its Charter §4.5 provenance remains unverified.

## What changed

The original run [claude] recorded phase totals in integer nanoseconds and printed milliseconds per order to stderr. The implementation at `61d011ee5` inserted extra stream synchronizations after select, sort and emit for every ordinary nonempty partition count. The sink timed the assembler's `push` and the query filter's `keep` separately.

The review repair [codex] leaves profiling disabled on ordinary `NgramCounter::new` calls. `enable_phase_timing()` enables the timers and extra fences, and the reference instrument enables them only when invoked with `--profile-phases`. The CUDA regression compares ordinary and profiled order digests and checks that only the profiled counter accumulates timings. The archived run predates this repair.

## Measurements

The run is wick's 4090 at `61d011ee5`, budget 2^24, all of TRAIN against all of DEV-tune, with `--check-host` (job `c423-profile`, `run1/c423/`).
- **N = 10:**
  - wall 1,910 s;
  - counting 1,824 s;
  - GPU scoring 1.1 s;
  - host check 81 s;
  - peak resident memory 8,864,644 KiB.
- **N = 4:** wall 295 s.

The phase totals for N = 10, summed over orders 0–10:

| Phase | Measurement boundary | Seconds |
|---|---|---:|
| Query filter `keep` | Host | 378.940 |
| Selection | CUDA-driving routine | 260.940 |
| Extension runs | Host | 261.207 |
| Assembler `push` | Host validation, census, totals and hashing | 241.203 |
| Radix sort | CUDA-driving routine | 198.145 |
| Record building | Host | 185.150 |
| Adjust | Host | 120.321 |
| Device-to-host copies | Copy calls | 90.640 |
| Boundaries, emit, scans | CUDA-driving routine | 31.178 |
| Histogram | CUDA-driving routine | 0.461 |
| Unattributed inside the order timer | Remainder after summing logged milliseconds | 48.440 |

The order timer begins after `wanted.filter(k)` and stops before either `finish()` call. The enclosing own-count and extension-count intervals measured 395.469 s and 374.996 s respectively. Adding those times to the table would double-count work. The sort interval rose from 3.386 s at order 0 to 34.064 s at order 9. The five host operations grew from 1.633 s at order 1 to 228.912 s at order 9.

## What the numbers say

1. **Filtering before copies.** The DEV-tune query retained 1,842,098 of 1,286,910,428 order-10 triples (0.143%). The earlier sealed DEV-test query retained about 0.27%. Both figures describe final triples. The number of extension records needed to produce them is unknown. The five intervals for copies, record building, extension runs, adjust and keep total 1,036.258 s. Some of that work must still run to produce and validate the required output. The 241.203 s in `NgramAssembler::push` includes canonical ordering, partition-seam validation, census reconstruction, count-of-counts verification, totals and SHA-256. A proposed filter must preserve the full-order counts for discounts and both canonical digests. The profile does not isolate a saving for moving SHA-256 or establish that the combined 1,277.461 s can be removed.
2. **Bucket once per order.** Selection occupied 260.940 s in this implementation. Replacing repeated whole-stream selection could reduce that interval. The replacement's allocation, bucketing and transfer costs need measurement.
3. **Reuse counted orders.** Own-order and extension-order ranges share a partition token. Reuse may reduce the 374.996 s spent in extension counting. Its savings overlap the first proposal, and storing or re-partitioning reusable records also costs time.
4. **Reuse lower orders across the N sweep.** Orders 0–3 in the N = 4 and N = 10 logs have identical counts, discounts and digests. Order 4 has raw counts when it is the top order. A shared sweep would still need each top order's raw counts and verification.

The original **3–5 minute** count is an optimization hypothesis. The profile supplies neither an end-to-end bound nor a separate eight-minute estimate with the host digest retained. Extension volume, validation, data movement and the replacement implementation remain unmeasured.

## Checks run at the head

- **G1, laptop, at `61d011ee5`:** the stub-nvcc `cargo check -p uor-r4-discrete --features cuda --lib --tests` and the instrument's `cargo check --features cuda` are clean. The host `kn_` and `ngram_count` tests pass, 31 / 31.
- **G2, wick, at the original run head:** the CUDA tests `ngram_count_cuda kn_score_cuda kn_filter` pass, 14 / 14, including the new phase-totals test.
- **G3, wick:** the N = 10 JSON is `cmp`-identical to claude-422's `sweep-10.json` but for `masses_at_floor`. All pre-existing JSON fields agree for that historical run, and `masses_at_floor` is zero.
- **G4:** this REPORT.
- **Not run:** a negative of the new test. It asserts `extension_count_ns > 0` directly, so dropping that tick would fail it by construction. It was not run on wick.

**Post-review host checks [codex].** The repaired source passed ten `kn_` library tests, 21 `ngram_count` library tests and five standalone instrument tests, including the default-off/explicit-profile CLI case. With a local stub nvcc and `UOR_NVCC_ARCH=compute_89`, `cargo check --locked -p uor-r4-discrete --features cuda --lib --tests` and the instrument’s `cargo check --locked --features cuda` passed. These checks type-checked the paired CUDA regression. They did not compile a real CUDA image or run it. The native-space contract check passed. The stored N = 10 JSON agrees with every prior field after removing the added zero floor count, and the N = 4 and N = 10 records agree for orders 0–3. The original run packet is unchanged.

## Skills used

working-discipline.

## Still open

- A filtered implementation must reproduce whole-order totals, counts of counts, discounts, digests and query scores. Measure the required extension records and end-to-end cost before comparing it with bucketing or count reuse.
- The new opt-in and paired CUDA digest regression have no post-repair hardware run in this record. The full historical profile's FIT provenance remains open.

## Plain summary

Five host operations took about two thirds of the order timers. Moving the query filter may reduce that cost if the full counts and integrity checks survive. The profile gives engineers a place to start. A faster count and a trained written-state model remain separate, unmeasured outcomes.
