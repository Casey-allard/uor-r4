# Card P1 — Ground truth on the M1 (measurement only; no model work)
Owner (human): Casey        Drafted by: Antigravity        Date: 2026-09-17
Signed:

Hypothesis (one sentence, falsifiable):
None — this benchmark establishes the empirical ground-truth denominator (throughput, energy per token, memory footprint, bits-per-byte) on consumer Apple Silicon M1 hardware that every subsequent efficiency and capability claim divides by.

Why this and not something else (link to 08 §):
Links directly to 08 §4 R4, §6 P1, and §8 D0. Prior claims relied on estimated multiplier-free operations and profiler constants without empirical measurement against incumbent local runtimes. Establishing exact numbers for `bitnet.cpp` and `llama.cpp` defines the real competitive baseline before pursuing further architectural iterations.

Data:
Sealed held-out set: 2,000 TinyStories-V2 stories and 500 Simple-Wiki articles, hash-split by `BLAKE3(text) mod 1000 ∈ {0..9}`, never opened or used for training/development. Manifest and SHA-256/BLAKE3 digest stored under `docs/evidence/heldout-2026-09/manifest.json`. Authored externally (Eldan/Li; Wikimedia); opened-before: no.

Matched controls:
Incumbent open local model baselines evaluated on identical M1 hardware and identical test sequences:
1. `bitnet.cpp` + BitNet b1.58 2B4T (open ternary weights).
2. `llama.cpp` + SmolLM3-3B Q4_K_M.
3. `llama.cpp` + Qwen3-4B Q4_K_M.
4. `r4 geometric` + `15baec48` (native count model).
5. TLA/R4G1 `no_std` kernel bundle.
6. Optional: Kneser-Ney 5-gram (kenlm) fit on identical TinyStories data slice.

Primary metric + threshold:
Complete-request wall-clock throughput (load + encode + 256-token generation + persist) in tok/s, and energy efficiency via macOS `powermetrics` in Joules per token (J/token). Threshold: Establish pinned baseline table.

Secondary metrics:
Bits-per-byte (teacher-forced perplexity on held-out text), peak RSS (resident set size in MB), resident bytes per parameter/token, DRAM memory bandwidth saturation (GB/s).

Budget:
Engineering: ≤ 2 days. Machine: ≤ 3 h execution across 3 repeats per artifact. Storage: ≤ 6 GB (model weight snapshots). Wall-clock cap: 4 h.

Kill criterion (pre-registered):
N/A (Card P1 is an empirical measurement and calibration baseline, not a falsifiable mechanism test).

Preservation:
Identical pinned harness scripts and repeatability across 3 independent runs on the same session; temperature 0.0, fixed random seeds for generation passes.

Deliverables:
- Row per evaluated artifact in `docs/integration/EVIDENCE.md`.
- Comprehensive benchmark analysis and hardware telemetry in `docs/integration/cards/P1-ground-truth-m1-RESULT.md`.
- Denominator reference table in repository root `README.md` replacing historical estimated efficiency text.
- Telemetry raw logs and powermetrics outputs archived in `.uor-models/benchmarks/m1-ground-truth/`.
