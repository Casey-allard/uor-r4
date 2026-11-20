# Independent review of the 72-byte CUDA payload bound

This is a source-level allocation bound for the full-C count producer on the
reviewed single-stream path. Driver pool usage and measured peak VRAM are
separate. The [initial Wick resource HOLD](../FIT-FULL.md) records the original
80-byte estimate and exact admission deficit.

Let T be the full framed token count, n the admitted target count for a range,
g the distinct gram count and c the distinct context count. All phases enforce
c ≤ g ≤ n. Resident token IDs take 2T bytes and the cached predecessor-bucket
positions take at most 4T. The largest planned full-C range has
n ≤ 147,763,001.

| Phase | Variable device allocations above resident tokens and bucket positions |
|---|---|
| Token histogram and predecessor buckets | Predecessor histogram words use `4 × (4096 × ceil((T−1)/2^20) + 1)` bytes, with a separate 4 MiB allowance for scan or histogram auxiliaries. |
| Target select | At most 4n for the selected positions and a 256 MiB window of scan flags, plus the scan's small block sums and terminal words. |
| Radix sort | 12n for the two u32 position arrays and two u16 key arrays, plus 256 bytes per 4,096-target tile, a terminal word and block sums. |
| Boundary flags | At most 20n + 4g plus terminal words for sorted positions, gram flags, the flag clone, context flags, adjusted left flags and gram starts. |
| Gram emission | At most 12n + 46g plus fixed words. The 12n covers positions, context flags and left flags. The 46g covers gram starts, keys, next IDs, counts, context starts in gram space and two packed incidence arrays. |
| Context emission | At most 50g + 20c plus fixed words for retained gram data, the adjusted-count scan copy, context run starts and four u32 words per context. The cloned context flags are dropped before context record allocation. The two u64 incidence scans complete before context records and use at most 8 bytes per 2,048 grams plus a terminal word each. |

The count code returns and drops the earlier arrays before the later phases.
Both flag clones are real, deep device allocations in cudarc 0.19.10. A
synchronous free releases after the stream synchronizes. The asynchronous path
uses the same default stream for allocation, kernel use and free, which
orders reuse through the allocator. CUDA permits the reuse of a free from the
same stream and may keep reserved physical pages in its pool.
[CUDA 13.1 stream-ordered allocator](https://docs.nvidia.com/cuda/archive/13.1.0/cuda-programming-guide/04-special-topics/stream-ordered-memory-allocation.html)

For this exact full-C plan, the conservative 72n allowance plus the existing
256 MiB selector window and fixed word covers the phase maxima. Adding the
6T residency and the unchanged 512 MiB driver and allocator reserve requires
23,897,892,586 free bytes after context initialization. The exact C128 run's
idle device reading of 24,838,406,144 would leave 940,513,558 bytes over that
requirement. A revised producer must reproduce the finite split artifact and
measure post-context free bytes under its own lock. This derivation covers
logical allocations and the admission margin. The runtime must propagate
allocation failure, preserve any unique prefix and withhold a successful
full-run receipt unless all count, resource and artifact checks pass.
