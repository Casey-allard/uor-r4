# Addressed geometric attention: complete forward integration — #973

**PASS_ADDRESSED_ATTENTION_FORWARD_INTEGRATION_GATE.** September 13, 2026. The optimized Rust gate passes **29 focused tests**, then the separately invoked report driver passes one four-particle × 64-position frozen-parameter forward/backward dry-run. There are **zero optimizer updates, language fits or promotions**. Retain `15baec48`. This implements the next step from the [primitive gate](native_geometric_addressed_attention_primitives_973.md) under the unchanged [addressed-attention specification](native_geometric_addressed_attention_spec_973.md).

## Complete implemented path

The isolated [addressed_attention module](../crates/uor-r4-core/src/native_geometric/addressed_attention/mod.rs) now connects QUERY_A → EXTENT_A → QUERY_B → EXTENT_B → CONTROL → EMIT → OBSERVE → KEY. Each phase invokes the shared 345-gate LUT4 circuit over 1024 causal bits. Null reads skip their extent phase. The two scans select from exact prior occurrences, committed results and Null using two signed H4 query/key lanes. Extents resolve owned spans only after source selection. Exact identities remain distinct from geometry. No hash-distance attention or source-byte output override is introduced.

The selected exact byte, hemisphere predicates and prepared active/result byte condition EMIT. Numeric validity is available after both reads; at most two selected scalars are decoded once per byte. Overflow legality uses bounded comparisons, and at most the chosen Add/Sub executes once. The prepared operation is reused for emission and offer caching. Matching actual observations acknowledge a lease; mismatches cancel it. OBSERVE incorporates the actual byte into the fixed zeta phase channels and updates four geometric roots; KEY publishes the new occurrence and any final acknowledged result in the same call. Input ingestion runs OBSERVE/KEY without a target-dependent prediction. EOS creates no byte occurrence and ends the response until the host resumes it.

The runtime uses integer/table operations, exact object access and finite circuits. Offline parameter initialization, interpretation, sampling, CE and gradient accumulation use floating point. No matrix products or transformer backbone are added to serving. Transactional state cloning copies fixed state. Artifact/snapshot serialization and offline policies allocate. Steady-state allocation has not been measured; this is not a measured allocation-free implementation of the separate frozen R4G1 kernel.

## Artifact and continuation binding

A separately versioned **86,417-byte initialized model artifact** is constructed, written, loaded from its actual file and verified. Its ID is `ac5cf4addfeca7d4346d126dcd4e3b301da2f6b597ec1b349c954023952f5719`. It is an untrained experimental model, not the retained model or a normal CLI dispatch promotion.

The envelope binds fixed wiring seed `0x973`, compiled gates/heads, all canonical prime/token/phase rows, signed H4 products/inverses/ranks and hemisphere predicates, construction identity, implementation digest, parameter/data/configuration digests, seed and optional parent. Byte `b` uses native token `b+2`; BOS/EOS keep their reserved rows. The runtime consumes the first four of the native eight phase channels, while the envelope binds all eight. Unknown versions, noncanonical topology/geometry, malformed lengths/preferences, trailing bytes and incompatible implementation identities reject. Checksums establish corruption detection, not authentication.

An independent parameter interpreter and the loaded deterministic export produce identical complete offers, selected references, phase contexts, observed state and pending-snapshot continuation on the authored trace. This establishes implementation parity for the executed cases, not parity between stochastic training quality and deterministic generation quality. Snapshot checks cover full-ring eviction, pending state, object roots, reconstructible phases, bad IDs, turn/reset and EOS behavior. Host epochs remain non-reused namespaces; caller-owned RNG/score state must be checkpointed by the host if a policy returns an error. Runtime transactional rollback does not roll back external policy state.

## Actual frozen-gradient result

The report uses the first 64 bytes of an explicit mixed prose/code instrumentation string. Four fresh trajectories share 665,216 frozen parameters, initialized uniformly in `[-.25,.25)` with seed 7341. Event seed is 973. Each circuit/head invocation samples independently from its counter-keyed distribution. Current target affects only the direct conditional CE after EMIT context selection; a paired-target test leaves the offered symbol, runtime state and path scores identical. Every visited gate/head row contributes a separate score, including offered-symbol effects on future acknowledgment. The full mean trajectory loss supplies serial leave-one-out credit. No optimizer is called.

| Measurement | Executed value |
|---|---:|
| Trajectories / positions each | 4 / 64 |
| Complete circuit invocations | 2,001 |
| Gate events | 690,345 |
| Head events | 4,561 |
| Signed candidate scores | 16,640 |
| Selected reads / scalar decode attempts | 465 / 465 |
| Bytes executing all eight phases | 215 / 256 |
| Selected arithmetic operations | 0 |
| Initial mean conditional CE | 5.5523084622 nats |
| Finite nonzero gradient entries | 282,887 / 665,216 |
| Gradient L2 norm | 2.4943658190 |
| Forward/backward elapsed time | 309,230 µs |
| Preparation/export/reload time | 98,992 µs |
| Total report work before sealing | 420,702 µs |
| Six dense f64 arrays, counted by capacity | 31,930,368 bytes |
| Stored gate-event tape | 0 bytes |
| Runtime session type size | 14,520 bytes |
| Largest snapshot in this dry-run | 3,155 bytes |

Parameter digest before and after is identical: `d29bb3f055f015161c782015ae5113872d47effa93744d9363f4d3f5fe72891b`. Per-byte checks enforce at most eight circuit calls, 530 candidate scores, two scalar decodes and one selected arithmetic operation. The random initialized paths perform no arithmetic and do not exercise a saturated 256-record scan. Separate focused tests exercise ordered subtraction, overflow boundaries, same-KEY publication, full-ring resume and retained primitives. Those tests must not be described as learned arithmetic.

The memory figure counts parameters, three LOO accumulators and two path score/direct arrays. Temporary categorical, serialization, report and interpreter allocations are additional. The dry-run completed before the one-second RSS sampling interval, so its peak RSS is **UNAVAILABLE**, not zero. The 0.309-second measurement includes compiled-payload binding, four trajectories, score accumulation, snapshots and gradient finishing. It is one small CPU profile, not steady-state serving latency, a worst-case cost, or an energy result.

## Validation and preservation

The first release command completed successfully:

```text
cargo test --offline --release -p uor-r4-core --lib addressed_attention -- --test-threads=1 --nocapture
29 passed; 0 failed; 1 intentionally ignored report driver; 538 filtered out
```

The exact retained test executable then ran `offline::tests::frozen_forward_backward_report` once, with an exclusive report directory: **1 passed; 0 failed; 0 ignored**. That attempt was sealed and its complete file set verified. The report driver is excluded from ordinary test runs so builds cannot accidentally launch or overwrite the bounded measurement. The existing two unused test-fixture constant warnings remain.

Build/test charge was 111,728 ms; separate report process charged 614 ms. Total **112,342 / 180,000 ms**. Shared ledger **122,861,673 / 132,950,000 ms**; parent cycle **3,111,100 / 3,200,000 ms**. Peak sampled build/test RSS was 2,564,980,736 bytes, within 4 GiB. Measured storage growth before documentation/delivery was 20,910,080 bytes within 128 MiB; final storage/engineering receipt appends locally. No allowance extension, paid compute or cleanup occurred.

Both original checkout heads/statuses and retained model hash verify. All **21 prior sealed roots / 733 files** verify unchanged; 140/141 older shared-core source bindings remain unchanged, with only the already introduced native module declaration differing. No retained dispatch/loader change or V3–V7 replay occurred. The [tracked evidence](evidence/native_geometric_addressed_attention_forward_973.json) binds all current source files, exact executable, logs, review, projection, artifact and sealed report files. Local receipts remain in the established handoff `shared-core-first-step/addressed-attention-forward-1/`. GitHub compatibility acknowledgements execute no tests; local execution supplies validation.

## Remaining work

This result establishes a complete implemented route and a measured finite-gradient execution. It establishes **no learned conversation, memory, coding, generalization or deterministic generation advantage**. Corpus preparation, optimizer/checkpoint support, learned EOS behavior, a training dose and source-separated acceptance remain unfinished. Paired-H4 companion transport, durable learned revision writing and Hamming comparison retain their explicit specification scope. Existing literature and UOR/Prism/NEMESIS/Spiralcore research remain available; this gate required no new research survey or parameter sweep.

**Next:** Freeze one small source-separated conversation/coding learning pilot, its independent acceptance and matched context-disabled control before fitting. Implement the Rust corpus/optimizer/checkpoint driver against this complete path, retain the initialized artifact as the baseline, and qualify checkpoint/export and deterministic generation separately from stochastic CE. Use the measured 0.309-second four-particle/64-position cost only as an initial estimate; it excludes optimizer/checkpoint costs and does not measure longer/saturated contexts. Record the complete build/preparation/fit/evaluation/correction/storage projection and any necessary preauthorized parent allowance extension before execution. Select one fixed training recipe and a finite update cap; do not start a sweep, fit against opened evaluation or infer a useful learning dose merely from throughput. Actual changed-source responses and retained controls are the learning gate; falling construction CE alone is insufficient.

Proposed next ceiling: **64 optimizer updates**, each with four particles over 64 positions, one fixed recipe and no seed/rate grid. The present profile implies about19.8 seconds for64 frozen batches before optimizer/data/checkpoint/evaluation overhead; it is not a promise of convergence or a full fit timing. Proposed complete allowance240,000 ms:140,000 build/preparation/checks,40,000 fit,30,000 evaluation/export/generation,30,000 correction; two build threads, one model process,4 GiB RAM,128 MiB new storage and128 MiB margin. Only88,900 ms remains in the current parent allowance, so refresh and record a necessary preauthorized extension before that successor. No successor allowance or fit is consumed by this integration gate.
