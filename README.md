# UOR-R4 Geometric Language Model

**An open-source research experiment pioneering native geometric language modeling — achieving zero runtime matrix multiplication (zero-GEMM), zero runtime floating point, and zero steady-state heap allocations on consumer hardware.**

UOR-R4 investigates a fundamental alternative to transformer-based large language models: can language, context, memory, and reasoning be modeled through learned geometric state transitions, discrete algebraic codebooks, and exact addressed memory rather than dense tensor matrix products?

The research objective is local, low-energy language modeling on commercially available personal hardware — specifically Apple Silicon M1-class laptops — eliminating the massive energy, thermal, and memory overhead of dense neural backbones without compromising on exact knowledge retention or reasoning capability.

Technically, UOR-R4 is an **experimental autoregressive geometric state model with exact addressed memory and learned typed operators**, implemented entirely in Rust. It utilizes $H_4$ 600-cell root monoids, icosian group actions, exact $\mathbb{Z}[\phi]$ ring arithmetic, high-dimensional binary Vector Symbolic Architecture (VSA) hypervectors, discrete 2-level hierarchical lattice codebooks, conditioned skip-engram tables, and fixed zeta-zero spectral channels as primary computational mechanisms.

> [!NOTE]
> **Status: Pre-Alpha Open Research Experiment.**  
> UOR-R4 is an active scientific investigation. We adhere strictly to pre-registered hypotheses, empirical falsification, and honest metric reporting without score tampering, penalty masks, or threshold adjustments. See the [canonical plan](docs/integration/project-track.md) for roadmap responsibilities and [current state](docs/integration/current-state.md) for the latest measured artifacts.

---

## Table of Contents

- [Project Goals](#project-goals)
- [Runtime Serving Contract](#runtime-serving-contract)
- [Mathematical Foundations](#mathematical-foundations)
- [System Architecture](#system-architecture)
- [Latest Measured Empirical Results](#latest-measured-empirical-results)
- [CLI Usage & Developer Workflow](#cli-usage--developer-workflow)
- [Active Source Map](#active-source-map)
- [Repository & Documentation Map](#repository--documentation-map)
- [Roadmap & Next Steps](#roadmap--next-steps)
- [Contributing & Research Continuity](#contributing--research-continuity)

---

## Project Goals

1. **Transformerless, Zero-GEMM Serving**: Eliminate all mathematical matrix multiplications ($O(N^2)$ and $O(d^2)$ dense matrix-vector products) from final token generation. Replace dense projections with bounded $O(1)$ lattice table lookups, exact group composition, and binary hypervector operations.
2. **Zero Runtime Floating Point**: Execute the entire serving loop in fixed-point and integer arithmetic ($Q1.62$, $Q30$, $u32$, and binary bitwise operations). Completely prevent floating-point nondeterminism, precision drift, and GPU hardware requirements.
3. **Zero Steady-State Heap Allocations**: Ensure that once loaded, interactive autoregressive generation executes with zero allocations on the hot path (verified via counting allocators in CI). Working context resides in fixed-capacity stack arrays and bounded circular ring buffers.
4. **Exact Addressed Memory Substrate**: Decouple static learned generalization from dynamic episodic memory. Facts, names, versioned entities, and verbatim occurrences are stored in an exact addressed substrate with $O(1)$ retrieval, versioned provenance, and causal commits.
5. **Ultra-Low-Latency Local Inference**: Deliver sustained generation speeds exceeding 15,000–70,000 tokens/sec per single CPU core on Apple Silicon M1 with sub-50 µs cold-start latencies.

---

## Runtime Serving Contract

UOR-R4 enforces a rigorous boundary between offline learning and runtime serving:

| Boundary | Contract & Invariant |
| --- | --- |
| **Final Serving (Hot Path)** | **Zero GEMM, Zero Runtime Floats, Zero Steady-State Heap Allocations.** No matrix multiplication, no transformer attention heads, no multi-layer perceptron (MLP) blocks, no floating-point arithmetic, no heap allocation (`malloc`/`free`) during generation steps. |
| **Permitted Serving Operations** | Discrete hierarchical lattice lookups; Voronoi cell routing; binary VSA hypervector unbinding and bundle consensus; conditioned skip-engram n-gram lookups; integer add/subtract/shift/popcount; exact group composition by table lookup; $O(1)$ addressed memory retrieval and causal commit. |
| **Offline Preparation & Training** | Offline corpus tokenization, optimization, and parameter learning in Rust may utilize floating-point gradients, matrix multiplication, and standard optimizers (Adam). Final artifacts are serialized into discrete fixed-point tables and memory-mapped binary representations. |
| **No Teacher / Provider Serving** | Serving responses are generated strictly by the local native model. No external LLM API, remote teacher, or hidden dense backend is permitted at runtime. |
| **Hardware Target** | Consumer personal hardware (e.g., Apple Silicon M1 laptops, embedded CPU nodes) using standard CPU instructions without requiring discrete accelerators or CUDA GPUs. |

---

## Mathematical Foundations

### 1. $H_4$ 600-Cell Root Monoid & $S^3$ Hopf Fibration
The non-crystallographic Coxeter group $H_4$ describes the 120 quaternionic vertices of the 600-cell in 4-dimensional Euclidean space $\mathbb{R}^4$. 
- Each unit quaternion represents a point on the 3-sphere $S^3$.
- State transitions are modeled as discrete monoid actions on $S^3$, preserving directional orientation and chirality.
- Hopf fibration $S^3 \xrightarrow{\pi} S^2$ decomposes 4D state into a base spherical coordinate and an explicit fiber angle $S^1$, allowing dimensionality reduction while preserving fiber orientation without information loss.

### 2. Icosian Groups & Exact $\mathbb{Z}[\phi]$ Ring Arithmetic
To eliminate floating-point drift in geometric coordinate calculations, algebraic coordinates are expressed in the golden ring:
$$\mathbb{Z}[\phi] = \{ a + b\phi \mid a, b \in \mathbb{Z} \}, \quad \text{where } \phi = \frac{1 + \sqrt{5}}{2} \quad (\phi^2 = \phi + 1)$$
By operating over integer pairs $(a, b)$, icosian symmetries and root reflections are computed algebraically in closed form before projection onto fixed-point tables ($Q30$ and $Q1.62$).

### 3. Binary Vector Symbolic Architecture (VSA) & Holographic Representations
Contextual dependency over the active 64-token history is tracked via 1024-bit binary hypervectors:
- **Multi-Head Context Engine**: 4 independent VSA heads tracking distinct relational subspaces.
- **Holographic Reduced Representations (HRR)**: Transitions are bound into context hypervectors using cyclic coordinate permutation $\rho$ and XOR superposition $\oplus$:
  $$H_{\text{trans}} = \bigoplus_{i=1}^{t-1} E(w_i) \otimes \rho(E(w_{i+1}))$$
- **Associative Unbinding**: At step $t$, the expected continuation is queried via XOR unbinding:
  $$Q = H \otimes E(w_t)$$
  and ranked via bitwise Hamming distance ($\text{popcount}(Q \oplus E(w_{\text{cand}}))$).

### 4. 2-Level Hierarchical Lattice Tables ($L_{\text{hier}}$)
Vocabulary selection scales to large dictionaries without dense matrix multiplication through a 2-level hierarchical Voronoi lattice:
- **Level 1 (Coarse Sector)**: Evaluates $H_4$ root trigram transitions:
  $$\text{score}_{\text{coarse}} = \mathcal{T}_{\text{root}}(r_{t-1}, r_t, r_{\text{cand}})$$
- **Level 2 (Fine Cluster)**: Evaluates intra-cluster transition residuals:
  $$\text{score}_{\text{fine}} = \mathcal{T}_{\text{cluster}}(c_t, c_{\text{cand}})$$
- **Balanced Multi-Sector Shortlist Routing**: Enforces a strict quota (`MAX_PER_SECTOR = 8` across 6 active $H_4$ sectors), preventing single-cluster monopolization and ensuring diverse grammatical candidate generation.

### 5. Information-Theoretic Self-Transition Decoupling (Anti-Gaming Surprisal)
Because fine cluster residuals pool probabilities across all members of a cluster, naïve cluster transitions produce an artificial $+4,096$ bonus on identical token self-repeats ($t_{\text{cand}} == t_{\text{curr}}$), causing catastrophic word repetition loops ("the the", "years years").
UOR-R4 resolves this mathematically in $Q1.62$ fixed point with zero runtime floats via the exact surprisal deficit:
$$\Delta I(V) = -\log_2(V) \times \text{FINE\_SCALE}$$
For canonical $|V| = 4096$, this evaluates bit-exact to $-24,576$, decoupling cluster transition density from identical word repetition while preserving coarse $H_4$ root trigram geometry.

### 6. Conditioned Skip-Engram Syntactic Tables
To capture grammatical and syntactic regularities without self-attention:
- High-order exact tables for bigrams, trigrams, 4-grams, and 5-grams.
- Skip-bigram tables for strides $k \in \{2, 4, 8\}$ conditioned on dynamic syntactic depth registers and quotation mark parity.
- Sub-15 ns lookup latency over packed 32,768-entry binary tables.

### 7. Exact Addressed Memory Substrate & In-Context Induction
- **$O(1)$ Addressed Memory**: Stores named entities, user intents, and factual records with explicit provenance witnesses and versioned causal commits.
- **In-Context Induction Attention**: Implements a 2-layer bigram induction scan over the active 64-token ring buffer, boosting recurring sequence continuations by $4096 / k$ without matrix multiplications.

---

## System Architecture

```text
                               Raw Text Input
                                     │
                                     ▼
                      Byte-Level BPE Tokenizer
                                     │
                                     ▼
         ┌───────────────────────────────────────────────────────┐
         │              Active 64-Token Circular Context         │
         └───────────┬───────────────────────────────┬───────────┘
                     │                               │
                     ▼                               ▼
       ┌───────────────────────────┐   ┌───────────────────────────┐
       │   Geometric State ($S^3$) │   │  4-Head VSA Hypervectors  │
       │    Hopf Fibration Proj.   │   │    1024-bit HRR Memory    │
       └─────────────┬─────────────┘   └─────────────┬─────────────┘
                     │                               │
                     └───────────────┬───────────────┘
                                     │
                                     ▼
           ┌───────────────────────────────────────────────────┐
           │      Candidate Routing & Shortlist Pruning        │
           │  • Coarse H_4 Voronoi Sectors (Max 8/sector)      │
           │  • Fine Hamming Distance Codebook Clusters        │
           │  • Conditioned Skip-Engrams (2, 3, 4, 5-grams)    │
           │  • In-Context Induction Candidates                │
           └─────────────────────────┬─────────────────────────┘
                                     │ Top 64 Candidates
                                     ▼
           ┌───────────────────────────────────────────────────┐
           │          Zero-GEMM Fixed-Point Scoring            │
           │  • Coarse Root Trigram Score                      │
           │  • Fine Cluster Residual with Surprisal Decouple  │
           │  • VSA Associative Unbinding Hamming Alignment    │
           │  • Syntactic Register & Quotation Parity Gates    │
           │  • Exact Addressed Memory & Induction Boost       │
           └─────────────────────────┬─────────────────────────┘
                                     │
                                     ▼
                        Top-k / Temperature Sampler
                                     │
                                     ▼
                           Streamed UTF-8 Output
```

---

## Latest Measured Empirical Results

### 1. Full-Corpus 555M-Token Training Run (September 18, 2026)
Full-scale native geometric prose training was executed on the 555,385,505-token TinyStories corpus (`tinystories_train.u16`, 1,059.31 MB pre-tokenized binary token stream):
- **Optimization**: 546,644,574 sequence tokens across 33,895 batches ($256 \times 64$) with Adam optimizer.
- **Wall Time**: 3,982.42 seconds (~66.4 minutes).
- **Throughput**: Sustained **137,264.5 tokens/second** across all 8 cores on an Apple Silicon M1 (4 Firestorm + 4 Icestorm via Rayon).
- **Bits-Per-Byte (BPB) Convergence**: Evaluated on 64,000 held-out tokens (255,022 UTF-8 bytes):
  - Initial BPB: **2.0356 bits/byte**
  - Converged Final Geometric BPB: **1.2372 bits/byte** ($\Delta = -0.7985$ BPB).

### 2. Baseline Comparison & Honest Card P3 Disposition
Under the repository's strict Anti-Gaming Protocol ([AGENTS.md](AGENTS.md)), geometric predictions are evaluated against matched non-neural controls on the identical held-out test split:
- **Matched Non-Neural Control**: Kneser-Ney 5-gram (discount = 0.75, 103,415 bigram transitions) achieved **1.2055 bits/byte**.
- **Empirical Delta**: The native geometric model trailed the matched non-neural 5-gram control by **$-0.0316$ bits/byte**.
- **Pre-Registered Kill Criteria & Disposition**: Card P3 (`docs/integration/cards/card-p3-geometric-predictor-viability.md`) required a minimum predictive advantage of $\ge +0.3000$ BPB over the matched 5-gram baseline. In accordance with pre-registered criteria, **Card P3 was formally retired without score tampering, penalty masks, or threshold softening**. Flat single-scale geometric state transitions without hierarchical multi-scale composition do not overcome high-order count baselines, providing clear empirical motivation to advance the roadmap to Card P4 (hierarchical composition / multi-scale structure).

### 3. Serving Throughput & Allocation Verification
- **Single-Thread M1 Serving Speed**: **16,486 – 71,849 tokens/second** (latency ~13.9 – 60.6 µs/token).
- **Cold-Start Startup Time**: **< 50 µs** via zero-copy memory mapping (`memmap2`).
- **Binary Model Size**: **< 3 MB** (`.rgm` binary format).
- **Zero-Allocation Invariants**: **19/19 allocation tests PASS** (`native_geometric_allocations.rs`). Zero heap allocations on generation hot paths.
- **Numerical Parity**: 100% bit-exact parity across 1,000 queries between continuous and serialized binary models.
- **Historical Regression Retention**: **100.0% PASS on all 9,984 regression cases** (2,304 independent neighbor transfer cases + 6,688 historical traces bit-exact).
- **Serving Diversity**: All 9 prose serving tests pass with strict token diversity assertions ($\text{Distinct-1} \ge 0.50$, $\text{Distinct-2} \ge 0.70$, $\text{Max Token Frequency} \le 0.15$).

---

## CLI Usage & Developer Workflow

### 1. Interactive Streaming Chat (`r4-native-chat`)
Launch an interactive zero-allocation streaming session:

```bash
# Build the chat binary in release mode
cargo build --release -p uor-r4-api --bin r4-native-chat

# Run interactive streaming chat with live telemetry
cargo run --release -p uor-r4-api --bin r4-native-chat -- \
  --model models/geometric_prose_model.rgm \
  --tokenizer models/tokenizer.json \
  --temperature 0.8 \
  --top-k 10

# Single-prompt generation
cargo run --release -p uor-r4-api --bin r4-native-chat -- \
  --model models/geometric_prose_model.rgm \
  --tokenizer models/tokenizer.json \
  "Once upon a time, there was a little girl named Lily."
```

**Interactive REPL Commands**:
- `/reset`: Clear conversational history and reset VSA context buffers.
- `/stats`: Display memory usage, active records, and serving telemetry.
- `/exit`: Exit the interactive session.

### 2. Pre-Tokenizing Corpora (`pretokenize-corpus`)
Convert raw text datasets into memory-mapped, chunked 16-bit binary token streams with canonical 64-byte headers:

```bash
cargo run --release --bin pretokenize-corpus -- \
  --input data/TinyStoriesV2-GPT4-train.txt \
  --tokenizer models/tokenizer.json \
  --output data/tinystories_train.u16 \
  --batch-stories 4096
```

### 3. Full-Corpus Training (`train-native-prose`)
Train the native geometric model over binary token streams with real-time JEPA loss, cross-entropy loss, bits-per-byte tracking, and matched Kneser-Ney 5-gram baseline evaluation:

```bash
cargo run --release --bin train-native-prose -- \
  --corpus data/tinystories_train.u16 \
  --tokenizer models/tokenizer.json \
  --vocab-size 4096 \
  --context-window 64 \
  --batch-size 256 \
  --lr 0.001 \
  --output-model models/geometric_prose_model.rgm
```

### 4. Running the Complete Verification Suite
Execute static linting, claim wording checks, allocation invariants, and the 9,984-case regression suite:

```bash
# Full qualification script
bash scripts/verify_qualification.sh

# Zero-allocation verification
cargo test --manifest-path crates/uor-r4-core/Cargo.toml --test native_geometric_allocations

# Chat serving and diversity tests
cargo test --manifest-path crates/uor-r4-api/Cargo.toml --test native_chat_prose_serving_tests

# Binary serialization and mmap parity tests
cargo test --manifest-path crates/uor-r4-core/Cargo.toml --test binary_model_tests

# Hierarchical lattice codebook tests
cargo test --manifest-path crates/uor-r4-core/Cargo.toml --test hierarchical_lattice_tests

# Conditioned skip-engram tests
cargo test --manifest-path crates/uor-r4-core/Cargo.toml --test conditioned_skip_engram_syntactic_tests
```

---

## Active Source Map

All paths are relative to `crates/uor-r4-core/src/native_geometric/` unless otherwise indicated:

| Component | Key Source Files | Description |
| --- | --- | --- |
| **Hierarchical Lattice** | [`lattice_table.rs`](crates/uor-r4-core/src/native_geometric/lattice_table.rs), [`vsa/hierarchical.rs`](crates/uor-r4-core/src/native_geometric/vsa/hierarchical.rs) | 2-level Voronoi codebook; coarse $H_4$ root trigrams; fine cluster residuals; anti-gaming surprisal penalty. |
| **VSA Attention Engine** | [`vsa/attention.rs`](crates/uor-r4-core/src/native_geometric/vsa/attention.rs), [`vsa/context_engine.rs`](crates/uor-r4-core/src/native_geometric/vsa/context_engine.rs) | 4-head 1024-bit binary VSA context engine; HRR associative unbinding; circular ring buffers. |
| **Conditioned Skip-Engrams** | [`engram.rs`](crates/uor-r4-core/src/native_geometric/engram.rs) | High-order n-grams (2 to 5); skip-bigrams ($k \in \{2, 4, 8\}$); syntactic depth and quote parity tracking. |
| **Binary Model Serialization** | [`learner/binary_model.rs`](crates/uor-r4-core/src/native_geometric/learner/binary_model.rs) | Memory-mapped `.rgm` binary container; zero-allocation deserialization; CRC checksum validation. |
| **Corpus Pipeline** | [`mmap_corpus.rs`](crates/uor-r4-core/src/native_geometric/mmap_corpus.rs), [`bin/pretokenize-corpus.rs`](crates/uor-r4-core/src/bin/pretokenize-corpus.rs) | Streaming binary `u16` token corpus reader/writer with zero heap allocations. |
| **Learner & JEPA Trainer** | [`learner/jepa_trainer.rs`](crates/uor-r4-core/src/native_geometric/learner/jepa_trainer.rs), [`bin/train-native-prose.rs`](crates/uor-r4-core/src/bin/train-native-prose.rs) | Dual-objective $L_{\text{CE}} + \lambda L_{\text{JEPA}}$ training; Rayon parallel batches; Adam optimizer. |
| **Hopf & S3 Metric** | [`hopf_metric.rs`](crates/uor-r4-core/src/native_geometric/hopf_metric.rs) | Fixed-point $Q30$ unit quaternions on $S^3$; Hopf projection to $S^2$; integer Newton-Raphson normalization. |
| **Serving Runtime & API** | [`runtime.rs`](crates/uor-r4-core/src/native_geometric/runtime.rs), [`crates/uor-r4-api/src/native_capability_api.rs`](crates/uor-r4-api/src/native_capability_api.rs) | Complete serving session loop; induction attention; shortlist assembly; candidate scoring. |
| **Interactive Terminal Chat** | [`crates/uor-r4-api/src/bin/r4-native-chat.rs`](crates/uor-r4-api/src/bin/r4-native-chat.rs) | Streaming terminal REPL with per-token telemetry and session management. |

---

## Repository & Documentation Map

| Resource | Scope & Contents |
| --- | --- |
| [Canonical Project Plan](docs/integration/project-track.md) | Ordered programme responsibilities, milestone acceptance criteria, and roadmap progression. |
| [Current State](docs/integration/current-state.md) | Latest empirical results, verified benchmarks, and active implementation status. |
| [Agent Execution Policy](AGENTS.md) | Anti-gaming protocols, machine limits, non-negotiable architectural invariants, and delivery rules. |
| [Two-Pillar Architecture Review](docs/integration/review-2026-09-16/) | Comprehensive system architecture audit establishing Pillar 1 (Exact Memory) and Pillar 2 (Learned Generator). |
| [Experiment Cards (P1–P6)](docs/integration/cards/) | Pre-registered empirical research cards with explicit kill criteria and matched controls. |
| [Mathematical Foundations](docs/integration/geometric-attention-research-2026-09/mathematical-foundations.md) | Detailed derivations of $H_4$, icosian groups, $\mathbb{Z}[\phi]$, and spectral zeta channels. |
| [Formal Vocabulary](docs/formal_vocabulary.md) | Normative definitions for empirical criteria, proof status, and claim verification. |

---

## Roadmap & Next Steps

Following the completion of full-corpus 555M training and the formal retirement of Card P3 per pre-registered criteria, development proceeds along the Two-Pillar architecture:

1. **Card P4 (Hierarchical Multi-Scale Composition)**:
   - Investigate whether multi-scale geometric hierarchies (phrase-level and clause-level geometric state transitions) overcome the single-scale count-baseline bottleneck.
   - Implement matched random-phase and random-root controls to isolate the causal predictive contribution of zeta phases and $H_4$ root symmetries.
2. **Card P1 (Empirical M1 Hardware Ground Truth)**:
   - Establish rigorous comparative hardware benchmarks on Apple Silicon M1: measure tokens/sec, Joules/token via `powermetrics`, RSS memory, and held-out BPB against `bitnet.cpp` (BitNet b1.58 2B4T) and `llama.cpp` (SmolLM3 / Qwen3).
3. **Card P2 (Exact Memory Product Layer)**:
   - Expand the verified $O(1)$ addressed memory substrate to support production local assistance with grounded QA, conflict resolution, and principled abstention.
4. **Card P5 (Decodable Error-Correcting Codes for VSA)**:
   - Implement structured error-correcting clean-up codebooks to sharpen high-dimensional binary hypervector retrieval.

---

## Contributing & Research Continuity

UOR-R4 welcomes scientific collaboration. All contributions must adhere to the engineering invariants defined in [AGENTS.md](AGENTS.md):
- **Preserve Invariants**: No runtime matrix multiplication (zero-GEMM), no runtime floating point, and no steady-state heap allocations on serving hot paths.
- **Empirical Rigor**: Pre-register hypotheses and kill criteria. Never apply artificial score penalties, repetition bans, or heuristic masks to force favorable benchmark metrics. Compare against strong non-neural controls (e.g. Kneser-Ney 5-gram).
- **Focused Verification**: Validate changes with focused tests (`cargo test --test <name>`), allocation checks, `cargo fmt --check`, and `python3 scripts/check_claim_wording.py`.
- **Protected Delivery**: Submit changes via pull requests referencing relevant issues. Never direct-push to `main`.

---

## License

The UOR-R4 project is licensed under the [MIT License](LICENSE).
Third-party research sources and dependencies retain their respective original licenses and attributions.
