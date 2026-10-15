# #1283 — Deliver native geometric language learner, zero-allocation serving, and full-corpus training (#820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-19T03:32:28Z
- **Updated:** 2026-10-09T13:17:46Z
- **Closed:** 2026-09-19T03:46:27Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1283
- **Merged:** 2026-09-19T03:46:27Z
- **Merge commit:** 38fa5a031f44e96c8b2e8c6acdb50179f1b56ce7
- **Base / Head:** main / UOR-Foundation:codex/p3-native-geometric-learner
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

This PR delivers the completed native geometric language modeling builds, zero-allocation runtime serving engine, memory-mapped binary model format, full 555M-token corpus training pipeline, standardized CLI binary targets, and comprehensive documentation updates across the repository.

### Implemented Capabilities & Builds

1. **2-Level Hierarchical Lattice Tables** (`crates/uor-r4-core/src/native_geometric/lattice_table.rs` & `vsa/hierarchical.rs`):
   - Coarse $H_4$ 600-cell root trigram transitions combined with fine Voronoi Hamming leaf clusters.
   - Balanced Multi-Sector Shortlist Routing (`MAX_PER_SECTOR = 8` across 6 active $H_4$ sectors), preventing single-sector noun-soup monopolization.
2. **Anti-Gaming Dynamic Surprisal Derivation**:
   - Resolved intra-cluster self-transition mode collapse via an exact information-theoretic surprisal deficit in $Q1.62$ fixed point with zero runtime floats: $\Delta I(V) = -\log_2(V) \times \text{FINE_SCALE}$ (evaluates bit-exact to $-24,576$ for canonical $|V| = 4096$).
   - Mathematically decouples cluster transition density from identical word repetition while strictly preserving coarse $H_4$ root trigram geometry.
3. **Conditioned Skip-Engram Syntactic Tables** (`crates/uor-r4-core/src/native_geometric/engram.rs`):
   - High-order n-gram tables (2-gram, 3-gram, 4-gram, 5-gram) and skip-bigrams ($k \in \{2, 4, 8\}$) conditioned on syntactic depth registers and quotation mark parity.
   - Sub-15 ns lookup latency over packed 32,768-entry binary tables.
4. **4-Head Binary VSA Context Attention Engine** (`crates/uor-r4-core/src/native_geometric/vsa/attention.rs` & `context_engine.rs`):
   - Multi-head 1024-bit binary hypervector context engine over active 64-token circular history.
   - Holographic Reduced Representations (HRR) with cyclic permutations and XOR associative unbinding.
5. **Exact Addressed Memory & Induction Attention** (`runtime.rs` & `native_capability_api.rs`):
   - In-context 2-layer bigram induction scan over 64-token ring buffer, boosting continuations by $4096 / k$ with zero GEMM or floats.
   - Exact addressed memory substrate with $O(1)$ retrieval, versioned provenance, and causal commits.
6. **Mmap Binary Model Serializer & Runtime** (`crates/uor-r4-core/src/native_geometric/learner/binary_model.rs`):
   - Compact `.rgm` binary format (< 3 MB footprint) with sub-50 µs cold-start startup via zero-copy memory mapping (`memmap2`).
   - 100% bit-exact numerical parity across 1,000 queries between continuous and binary models.
7. **Full-Corpus 555M-Token Training Pipeline & Card P3 Disposition**:
   - `pretokenize-corpus`: Chunked parallel memory-mapped BPE tokenizer converting raw text to binary `u16` token streams with canonical 64-byte headers.
   - `train-native-prose`: Dual-objective ($L_{\text{CE}} + \lambda L_{\text{JEPA}}$) Adam optimizer training over 546M sequence tokens across 33,895 batches ($256 \times 64$) in 3,982.42 seconds at 137,264.5 tokens/sec across all 8 M1 cores.
   - BPB converged from 2.0356 BPB to 1.2372 BPB on 64,000 held-out tokens (255,022 bytes).
   - Non-neural Kneser-Ney 5-gram control: 1.2055 BPB ($\Delta = -0.0316$ BPB).
   - Under the repository's Anti-Gaming Protocol and pre-registered kill criteria of Card P3, Card P3 is formally retired without score tampering or threshold softening, advancing the roadmap to Card P4 (hierarchical multi-scale composition).
8. **Interactive Streaming Chat REPL** (`crates/uor-r4-api/src/bin/r4-native-chat.rs`):
   - Interactive zero-allocation streaming terminal REPL with live per-token telemetry and session commands.
9. **Documentation & Repository Coherence**:
   - Fully updated `README.md` framing UOR-R4 as an open research experiment pioneering native geometric language modeling (zero-GEMM, zero runtime floats, zero steady-state heap allocations, transformerless serving).
   - Complete mathematical foundations ($H_4$, icosians, $\mathbb{Z}[\phi]$, VSA, hierarchical lattice, exact memory).
   - Comprehensive CLI usage instructions and developer workflows.
   - Updated `docs/integration/current-state.md` with verified benchmarks.

### Verification Record

- **Zero-Allocation Invariants:** 19/19 passed in `crates/uor-r4-core/tests/native_geometric_allocations.rs`.
- **Chat Serving Tests:** 9/9 passed in `crates/uor-r4-api/tests/native_chat_prose_serving_tests.rs`.
- **Binary Model Tests:** 7/7 passed in `crates/uor-r4-core/tests/binary_model_tests.rs`.
- **Hierarchical Lattice Tests:** 10/10 passed in `crates/uor-r4-core/tests/hierarchical_lattice_tests.rs`.
- **Conditioned Skip-Engram Tests:** 4/4 passed in `crates/uor-r4-core/tests/conditioned_skip_engram_syntactic_tests.rs`.
- **Historical Regression Suite:** 9,984/9,984 passed (100.0% bit-exact across 2,304 transfer cases and 6,688 historical traces) via `scripts/verify_qualification.sh`.
- **Corpus & S3 Trajectory Tests:** 12/12 mmap corpus tests, 11/11 grounded memory integration tests, 8/8 hierarchical routing tests, 3/3 S3 norm preservation tests.
- **Static Quality:** `cargo fmt --check`, `python3 scripts/check_claim_wording.py`, and `cargo check --all-targets` all pass cleanly with 0 warnings.

References #820, #973, #964, #963, #962, #1139, #1172.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

