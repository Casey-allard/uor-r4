# [codex] D33 parity-only direct count oracle

This leg tests identity of complete count words on the original record-aligned D29 C1 TRAIN input against the repaired D31 sm_120 census. It uses Wick (AMD Ryzen 9 9950X), one process, no accelerators, and zero tolerance. The parent run decision controls C1 dispatch. Its non-author FIT must bind this executable and the CPU parity scope. GPU evidence remains the qualification authority. The process has no held inputs, optimizer, fitted weights, scoring, model loading, or token prediction.

The independent executable checks two pinned input and GPU receipt pairs and supports only C128 and C1. It verifies the full UORT digest, 64-byte header, payload length, u32 ordinal bound, 12-bit ids, explicit BOS=0/EOS=1 frame, and registered totals. It reads each record including EOS, admitting every non-BOS target with a full context drawn from that record. Windows at record offset k have oldest token BOS for k>0 and lack a left extension. At order zero, every target has an in-record predecessor, so the adjusted unigram value is the distinct immediate predecessor count, including BOS where observed. At order 10 all adjusted counts preserve occurrence counts. At orders 1–9 BOS-start grams preserve occurrence counts and interior grams count distinct immediate predecessors. The 12-bit positional encoding matches the archived PrimeRouteKey format. Sorting one order by (packed context, next id, predecessor) makes all equal triples and all equal predecessor values consecutive. Group lengths yield raw counts and predecessor-run lengths yield adjusted counts. The result is exact by construction regardless of occurrence sort stability.

The executable hashes every 22-byte little-endian triple row and 40-byte little-endian context row in full canonical order, with the exact stream domain followed by the one-byte order. It compares every per-order target, unique triple, unique context, count-of-counts and both digests in both domains. The C1 comparison uses D31's actual raw order-10 digest. It stops immediately at the first divergent order/domain/field and retains its computed row and diagnostics. A digest discrepancy requires a raw or adjusted canonical word comparison for localization before attribution. The oracle does not independently load the D11 carrier. That is the ordinary serving loader leg of D33.

One 32-byte occurrence array at a time bounds the C1 working set attributable to tokens and occurrence rows at 705,646,234 bytes before small fixed buffers and process overhead. Libraries are pinned by Cargo.lock. The normal dependency graph contains only SHA-256 and JSON support. No production implementation is imported. Estimated dispatch envelope is one CPU core, 1 GiB private RSS, no new rented compute, and under five minutes as a planning estimate.

## Pinned identities and Wick controls

| Item | Identity |
|---|---|
| Wick private root | `/mnt/plexpool/uor-r4-corpus/runs/20261120-count-identity-33/oracle/d31` |
| Executable `target/release/d33-count-oracle` | `e7690f58e0a7ddbe29edd4303d584940d6c2ef7ec60822dec0bb5040e94e2373` |
| Rust source | `e625a667e3cc9aaaae27ca9494852fcaea5ddfdc6810f7b92f3fd9b12a4218a0` |
| Cargo.toml | `d88f343d9d598e53633e8bb1b765b2efd6e7f54be02aa5414eff840dec740c03` |
| Cargo.lock | `6e3b55a7f1c539bc0ce81e13266bffdabb72f139015f30c9bbe73e5a89be3f73` |
| C128 GPU receipt | `af7d8c9ee99b0fda69ec4c258051928ae9103ec4df7597d6212a94235deaa5e2` |
| D31 C1 GPU receipt | `34e3cd6e9d4ae35a78284a36911547f4ca4109b52e0e85e09fb3af54630f3df7` |

Wick built with rustc 1.93.1 and `cargo build --offline --locked --release`. `cargo test --offline --locked --release` passed all three finite controls. The fixtures demonstrate repeated versus distinct predecessors, BOS-start preservation, EOS/BOS separation, literal order-10 key and raw/adjusted domain distinction, rejection of a corrupted digest, changed hash on a planted count word and malformed record or ordinal rejection.

The registered C128 input reproduced all 22 GPU rows in 0.08 s wall and 6,108 KiB peak RSS. The C128 tape SHA-256 is `7710f5fb0b35b6a80554e194a0843e78803f4aca3e7affd8be5fd0180293a7ec`. Full C1 remains pending the D33 decision. Equality would establish the CPU leg of count identity on this pinned slice. A discrepancy would stop D33's identity conclusion at the first failed dependency. Neither outcome measures language quality, changes P1/P2 bars, or qualifies another count representation.

The Wick build root also retains `exact.sha256`, `dependency-tree.txt`, `runtime-needed.txt`, and `source-scan.txt`. The forbidden-operation name scan found zero hits in the oracle code, locked crate names and normal dependency graph. ELF dependencies are the system C runtime, compiler runtime, and dynamic loader. The earlier C128 tape in this directory used the D30 baseline receipt. The exact D31 release build produced `c128-d31.jsonl` and `c128-d31.time`.
