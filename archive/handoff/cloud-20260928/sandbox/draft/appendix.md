## Appendix A. Novelty map

Compiled from the literature report, which retrieved every source listed.

| Project idea | Closest prior work | Novelty | Leverage for the project |
|---|---|---|---|
| Keep the radius and quantize the direction | TurboQuant 2504.19874, PolarQuant 2502.02617 and QJL 2406.03482 all store the norm | Already done | High |
| 4-D quaternion chunks with a polytope shape code | HQMQ 2605.27646 (2026), IsoQuant 2603.28430 (2026), HIGGS p=4 2411.17525, QuIP# D4 2402.04396 | Already done (concurrent) | High |
| 600-cell or E8 = H4 ⊕ φH4 as a codebook | QuIP# E8P, LLVQ Leech lattice 2603.11021, 600-cell coding theory | Incremental | Medium |
| Per-lane quaternion state rotation | QRNN 1806.04418, PaTH 2505.16381, DeltaProduct 2502.10297, Mamba-3 2603.15569 | Incremental | Medium |
| Continuous training, then snapping to 2I, then serving as an exact integer automaton, inside a language model | Illusion of State 2404.08819; negative eigenvalues 2411.12537; DeltaProduct 2502.10297; The Automaton Underneath 2609.18966 | The capability is published; the snap-and-serve pipeline was not found (hypothesis). Measured at toy scale in this review | High |
| Quaternion-frame linear recurrence equals decayed linear attention with "quaternionic RoPE" | Mamba-2 SSD; RetNet; PaTH; Mamba-3 (complex case) | Incremental; the non-commutative case is not found in the literature | High (training kernels) |
| ≤4-bit additive/LUT serving (D0-b) | T-MAC 2407.00088, BitNet b1.58 2402.17764, bitnet.cpp 2410.16144, MatMul-free LM 2406.02528 | Already done | Very high |
| Soft read with a NoRead slot; copy gate | Pointer-sentinel mixture 1609.07843; attention sinks 2309.17453 | Already done | Low |
| Exact addressed memory | kNN-LM 1911.00172, RETRO 2112.04426, Engram, memory layers 2412.09764 | Already done (the learned sparse variants conflict with "no sparse routing") | High |
| Prime addresses | Prime Fourier Embeddings 2606.23044 (helps arithmetic only) | Incremental | Low–medium |
| Fixed zeta-zero phases | None found | Novel, unmotivated, and measured as decorative | Low |
| Hyperbolic / least-energy manifold routing | HELM 2505.24722 (near chance; best variant is MoE), RiLM 2609.10305, Hopfield 2008.02217 | Incremental | Low–medium |
| Prime-norm icosian "golden gates" for rotation codebooks | Parzanchevski–Sarnak 1704.02106; Kliuchnikov–Bocharov–Svore 1310.4150 (quantum compiling) | Novel in ML (hypothesis). No covering advantage at practical sizes; the theorem predicts holes (§6.4) | Low |
| Transformer-to-recurrent conversion | MOHAWK 2408.10189 (attention-free, about 3B tokens), LoLCATs 2410.10254 (40M tokens, but keeps 64-token softmax windows) | Already done | Medium: costly on an M1 if fully attention-free |

## Appendix B. Evidence produced by this review

The specialist reports and scripts lived in the review sandbox. Their load-bearing numbers are reproduced in this document with labels. The reports were:

| Report | Main experiments or checks |
|---|---|
| Mathematics | e1: 2I/E8 closure, lattice and cost; e2: transport drift; e3/e3b: zeta discrepancy, Landau bias, RoPE aliasing, identity collisions; e4: S³ codebooks and lattice second moments; e6/e6m: A5 learnability; e7: Householder representations; e8: octonions; e9: precision horizon |
| Physics | Energy model (60 configurations); Rust micro-benchmark of the repo's serving kernels; M1 measurement protocol |
| Architecture | Exact parameter and FLOP counts; M1 training budgets; quaternion-recurrence equivalence checks (errors 5e-15 to 2e-13); polar 600-cell attention-key codebook; multiply micro-benchmarks, including the quarter-square LUT |
| Literature | Seventy-plus retrieved papers; a 4-D gain–shape measurement |
| Audit | GitHub API data for 974 PRs; per-PR file statistics for the 369 PRs merged in the last 30 days; compute-versus-orchestration accounting |
| Verification | Builds and tests; x86 disassembly census; a random-weight timing harness comparing software and hardware multiply (bit-identical); a correctness review |
| Quantization | Matched-bit rate-distortion comparisons, inner-product error and NN recall, and the codebook-serving variant |
| State tracking | A5/Z60 word problems; quaternion, diagonal, complex and GRU models; exact 2I table serving |
| Lead | 2I closure; golden-gate covering (level-1 3,600 rotations, exact ½ℤ[φ] coordinates, covering against random); a WikiText-2 byte-level time-mixing comparison (six variants) |

The Python scratch code used by the agents is **not** committed, in keeping with the project's no-Python-model policy. The figures quoted here come from those runs. Any mechanism adopted from this review should be re-implemented and re-measured in Rust inside the project's own harness.
