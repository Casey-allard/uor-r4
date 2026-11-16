# ORDER: written-state-count-model, claude-412 (an exact, deterministic CUDA n-gram counter over u16 token streams, the shared primitive of R\* and P1's count base)

Lab: claude. Item: [m4-lawful-likelihood-bpb](../../../../roadmap/items/m4-lawful-likelihood-bpb.md) (R\* is its sealed denominator; the track has no item of its own yet). Track: the written-state geometric count model (the PI decision of 16 Nov 2026, P0 item 4 and P1, "the GPU counting kernel"). Base: `origin/main` at `ef1f4a641`.

## Why this unit

The new track writes its model state by exact counting on the GPU instead of searching for it. Two pieces need the same primitive:
- the sealed reference R\*, modified Kneser-Ney over orders 1..N (P0 item 4);
- P1's count base, the S2 n-let rows inside `TrunkTables`.

That primitive is an exact GPU n-gram counter: for each order, every (context, next token) pair of the stream with its count, in one canonical order, with the per-context statistics modified-KN needs. CHARTER §3.5 is GPU-first, so the counts come from CUDA. The CPU code here is a test oracle on fixtures and nothing else.

This unit is a finite kernel test plus a throughput probe on an existing D87 slice. It is not a qualifying evaluation and trains nothing, so under §4.5 it needs no sheet and no FIT.

## Native-space items (D85)

1. **State schema.** One typed key, `PrimeRouteKey`: the ordered context t_{j-k..j-1} packed as 12-bit ids in a u128. Slot 1 (the latest token) sits highest, slot k lowest, and the order k sits in the top byte. It is the exact image of the positional-prime product of DECISION §3.2 (one prime per (token, slot)), so back-off is dropping slot k and shared-suffix admission is slot-wise equality. Per order, the counted state is a canonical stream of (key, next, count) triples, plus one record per context: total count, N1+(h•), N1(h•), N2(h•), N3+(h•). Each order also has its count-of-counts n1..n4.
2. **Shared operators.** One counting operator in its own CUDA module, `ngram_count.cu`, built ahead of time by nvcc into its own admitted PTX and cubin through `build.rs`. No NVRTC, and nothing is appended to `CUDA_STACK_SOURCE` or `discrete.cu`. The host side packs keys, plans partitions, assembles partitions and hashes; it does no counting.
3. **Replay oracle.** A test-only CPU oracle counts the same triples with an ordered map, independent of the GPU method. The D85 replay identity is the SHA-256 of the canonical triple stream per order. GPU and oracle must give the same bytes, and the SHA must not change across runs or partition sizes.
4. **Forward, gradient, update.** The update rule is "count": integer sums only. The kernels use no float, and no atomics whose order could reach the output. A planted single-count change must be caught.
5. **Serialization.** The lossless form is the canonical triple stream itself: 22 bytes per triple (key u128 LE, next u16 LE, count u32 LE) in ascending (key, next) order. The context stream is 40 bytes per record (key u128, total u64, then N1+, N1, N2, N3+ as u32). The frozen pruned form is P1's work.

## Design

- **Targets.** Order k (0..=10) counts each target j with k <= j < n, under the context t_{j-k..j-1}. Order 0 is the empty context (the unigram counts).
- **Sort.** On the GPU, positions j are sorted by their gram, with no key materialized. A stable LSD radix sort reads the token at each slot from the resident u16 stream, 6 bits per pass, so order k takes 2(k+1) passes. Ranks inside a warp come from `__match_any_sync`, so the scatter is stable and deterministic.
- **Reduce.** Run-length reduction uses boundary flags and exclusive scans: gram starts, counts, context starts and context statistics. A context's total count is a difference of gram starts. N1 and N2 come from scans of indicator words.
- **Out of core.** The stream is resident on the device (the full ~1.87e9-target corpus is ~3.7 GB of the 4090's 24 GB). The work is split into key-range partitions by the most significant sort token: slot 1 for k >= 1, the next token for k = 0. Each partition is a contiguous range of token values whose targets fit a position budget, and partitions arrive in canonical order. The merge across partitions is therefore a concatenation. The one context that can straddle a partition boundary, the empty context of order 0, is merged on the host by adding its statistics, which are additive over disjoint follower sets. The budget bounds the device buffers (about 70 bytes per position) and the host copy-out (22 bytes per gram), under the container's 16 GB and the card's 24 GB.

## Tests

Laptop (CPU, and a stub-`nvcc` check of the CUDA build):
1. Oracle on hand fixtures: the triples, the context records and the count-of-counts.
2. Key packing round-trip, back-off by dropping slot k, and refusal of an id of 4096 or more and of an order above 10.
3. Order preservation: "a b c" and "c b a" give different keys, and so do "the x the" and "x the". This test must fail on an order-free key (G2).
4. Partition assembly: the oracle counted per partition and assembled gives the same bytes and SHA as the whole, including the straddling order-0 context.

CUDA on wick's RTX 4090:
5. GPU triples, context records and count-of-counts equal the CPU oracle's, byte for byte, on random streams of 1e5, 1e6 and 1e7 tokens for orders 1..8 (and 0).
6. The triple-stream SHA is the same across two partition budgets and across two runs.
7. A planted single-count change is detected, both by the SHA and by the comparison.

Throughput probe: tokens/s per order on the D87 slice `/workspace/out/c404/stream/tokens.u16` (15.47M tokens), with the SHA at two budgets, and a projection to the full ~1.87e9-target corpus in GPU-minutes.

## Files in scope

- `crates/uor-r4-discrete/src/ngram_count.rs`: the key, the record formats, the hashing and the partition assembly;
- `crates/uor-r4-discrete/src/ngram_count.cu`: the kernels;
- `crates/uor-r4-discrete/src/ngram_count_device.rs`: the CUDA driver;
- `crates/uor-r4-discrete/src/ngram_count_oracle.rs`: the test-only CPU oracle;
- `crates/uor-r4-discrete/src/ngram_count_cuda.rs`: the CUDA tests;
- `crates/uor-r4-discrete/src/bin/ngram-count-probe.rs`: the throughput probe;
- `crates/uor-r4-discrete/build.rs`, `Cargo.toml` and `src/lib.rs`: the new module's build and registration only;
- `docs/loop/tasks/written-state-count-model/claude-412/**`;
- one row in `docs/research/notebook/m1-discrete-attention.md`.

No driver source listed in `driver_source_sha256` is edited, so the pin does not move.

## Stop list

- No training, no evaluation and no BPB. R\*'s modified-KN smoothing, continuation counts and the D87 TRAIN/DEV split are later units.
- No pruning, no frozen form and no `TrunkTables` change (P1).
- No pod; CUDA runs on wick only, in Claude's container. No other lab's GPU work is touched.

## Done when

- G1: the laptop oracle and key tests pass (`cargo test -p uor-r4-discrete --lib ngram_count`).
- G2: with an order-free key, the order test fails; restored and rebuilt.
- G3: the wick CUDA tests pass: GPU equals the oracle on every order and size, and the SHA is stable across budgets.
- G4: throughput is measured on the c404 slice, with the full-corpus projection.
- G5: this ORDER is the first commit of the unit and the REPORT the last.
