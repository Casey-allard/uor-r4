# ORDER — written-state-count-model (claude-421)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407).
Unit: claude-421, P0.5 (R\*), part 4 of 5. Parts:
- claude-418: the counts (merged);
- claude-419: the host scorer (merged);
- claude-420: the streams (merged);
- claude-421 (this unit): the counts filtered to the scored contexts, a GPU scorer equal to
  claude-419's host scorer, and the bits instrument;
- claude-422: the sheet, FIT, N tuned on DEV-tune, and the sealed DEV-test run.

## The seal

DEV-test stays sealed until claude-422's sheet and FIT. **No run in this unit scores a DEV-test
target.** Every real-data run here uses DEV-tune as the query stream. Synthetic fixtures are used
for everything else.

## What it builds

1. **The filter** (`uor-r4-discrete`, host). `KnQuery::from_stream(query, highest)` holds, for each
   order k in 0..=N:
   - the sorted, distinct context keys of every non-BOS query target j >= k;
   - the sorted, distinct (key, target) pairs of those targets.

   `count_order_adjusted` gets a sink form, so a caller sees each partition as it is assembled. The
   existing function keeps its behaviour by calling the sink form with `NgramAssembler::push`. The
   caller still hashes the whole order, so the digest, the SHA and n1..n4 (and so the discounts) are
   the whole order's. Only the records a query target reads are kept. Partitions arrive in key order,
   so the kept records stay in canonical order and `KnTables::new` accepts them.
2. **The GPU scorer** (`kn_score.cu`, `kn_score_device.rs`). It is built by nvcc in `build.rs` into
   its own module, never NVRTC and never appended to another source. It uploads one `KnTables` and
   scores a query stream: one thread per target, the claude-419 rule word for word (binary search,
   then `kept / A + gamma * P / M` in 128-bit integers). It returns the Q48 mass of every non-BOS
   target in stream order, as `KnTables::scored_masses` does, and refuses a zero mass or one past
   2^64.
3. **The bits instrument** (`docs/labs/kn-reference/instrument`, a standalone crate). Given TRAIN,
   a query stream, `id-bytes.u8`, an optional class mask (`novel.u8`), N and a partition budget, it:
   - counts TRAIN's adjusted orders 0..=N on the GPU through the filter;
   - scores the query on the GPU;
   - takes `lawful_likelihood::bits_q12` of each mass;
   - sums bits and bytes, over all targets and over the mask's class 1 (novel).

   It asserts at compile time that `kn_score::PROBABILITY_ONE == lawful_likelihood::PROBABILITY_ONE`.
   `--check-host` also scores on the host and refuses at the first mass that differs. It writes JSON:
   - each order's digest and kept-record counts;
   - targets, Q12 bits, bytes and BPB as an integer in micro-bits per byte, for all targets and for
     the novel class;
   - the sha256 of every input.

   It holds no float.
4. **A real-stream oracle test** (an ignored CUDA test). It takes the first 1% of TRAIN and the
   first 1% of DEV-tune, each cut at a record start. At N = 2 the GPU path's masses (counter, filter,
   GPU scorer) must equal claude-419's independent CPU oracle, `KnOracle`, at every query target.
   Bits are a pure function of the mass, so equal masses give equal BPB.

## Gates

- G1, laptop: `uor-r4-discrete`'s KN and n-gram host tests pass, old and new, and the cuda build
  checks with stub nvcc. New host tests:
  - filtered tables score every target of a query exactly as the unfiltered tables do, at N = 0..4,
    on a TRAIN fixture that differs from the query;
  - the sink form of `count_order_adjusted` is covered by the existing CUDA tests on wick.
- G2: set-aside negatives, each failing, then restored:
  - the filter drops the (key, target) pairs, so it keeps contexts only: the filtered-equals-
    unfiltered test fails;
  - the GPU scorer skips gamma at one order: the CUDA GPU-equals-host test fails;
  - the instrument's bits are taken of mass + 1: the host check is unchanged but the JSON's bits
    move. This is a manual check that the bits come from the masses scored.
- G3, wick's 4090:
  - CUDA tests: GPU masses equal `KnTables::scored_masses` at N = 0..4 on synthetic streams,
    including a query with BOS in every slot and a query unlike TRAIN; every existing counter test
    still passes;
  - the 1% oracle test passes;
  - the instrument on the 1% slices at N = 2 with `--check-host` passes, and two runs give
    `cmp`-identical JSON;
  - a scale run: whole TRAIN at N = 2, DEV-tune as the query, with time and peak device memory
    recorded. Its BPB is an engineering check only; claude-422's sheet sets the bar's N.
- G4: `uor-harness loop pr-check --local` reads `verdict: PASS`.
- G5: no stream, mask, count table or corpus text is committed. Only the JSON summaries and logs
  are.

## Out of scope

- choosing N; any DEV-test score; the sheet and FIT (claude-422);
- PPMd, which is a diagnostic only.
