# Physics & energy stress test — UOR-R4 (physics specialist report)

Scope: is "replace wasteful floating-point matmul with geometric/integer computation" a physically sound path to lower energy/heat for local LM inference on an M1-class laptop, and which physically grounded ideas could genuinely help? Repo read-only at `413a32f`. Artifacts: energy model `exp/physics/energy_model.py` → `exp/physics/energy_tables.md`; Rust micro-benchmark of the repo's serving kernels `exp/physics/kernelbench/` (outputs `run_*.txt`). Labels: [SOURCE file:line], [MEASURED], [LITERATURE url], [DERIVED], [HYPOTHESIS].

## 1. Executive verdict

- **The premise is physically wrong as stated.** At batch-1 decode, energy goes to moving bits (DRAM ≈5–20 pJ/bit) and to instruction overhead on programmable cores (Horowitz: 70 pJ/instruction vs "a few pJ" per operation, 45 nm), not to multipliers. In every DRAM-bound configuration I modelled, removing multipliers saves **≤0.4 %** of the physics-floor energy per token (≤3.4 % even for cache-resident ternary) [DERIVED, §2 F2].
- **What measurably cuts energy is fewer bytes and less time.** bitnet.cpp (ternary vs fp16) cut J/token 55–70 % on M2 Ultra while package power stayed at 27–73 W; T-MAC cut energy 21–61 % while power fell only 10–17 %. The savings came from finishing sooner by moving fewer bits, not from cheaper arithmetic [LITERATURE arXiv 2410.16144; 2407.00088; DERIVED].
- **The current served path is ~10³× above its own physics floor.** 3.695 ms per token for a 1.68M-parameter model that fits in L2 cache is an estimated 11–23 mJ/token. That is roughly the DRAM-floor energy of a well-implemented 125–300M 4-bit model. llama.cpp's 7B Q4_0 on M1 runs at 88 % of the memory-bandwidth ceiling, within ~1–2× of its memory floor [SOURCE integer-serving-result-2026-09-25.md:70; DERIVED].
- **D0-b's "no multiplier instruction" rule, as implemented, costs energy.** Activation×activation products use a software shift-and-add loop over u128, and normalisation uses software long division. I measured these at **42–310× slower per product** than the hardware multiplier and ~3–8× slower per division. By the repo's own profile arithmetic, ~75 % of post-optimisation step time lies outside the weight kernel. The integer step is ~6.5× slower than the matched F32 step [MEASURED; SOURCE; DERIVED].
- **The "Hamiltonian / least-energy / phase transport / field" vocabulary is metaphor or optimisation language, not physics.** The current plan already says so. **No joule has ever been measured in the repo.** One earlier power figure was a hardcoded 3500 mW constant and was withdrawn [SOURCE].
- **Zeta zeros have no physical reason to help an LM.** GUE statistics make them a well-spread set of incommensurate frequencies, a property that any low-discrepancy, log-spaced or learned set also has. The repo's only control points the other way (zeta-disabled 91/96 vs full 86/96) [SOURCE docs/native_geometric_recovery_973.md:304-316].
- **Where the physically grounded wins are:** bytes-first design (packed ≤2-bit weights; 2.5–4-bit KV/event memory with the norm kept, which is where the owner's "preserve the radius" instinct belongs; cache residency; block-structured access), efficient integer SIMD/LUT kernels, Hopfield-theoretic popcount admission, norm-preserving or oscillatory recurrent cores, reversible training for RAM, and E-core/DVFS execution.
- **An energy breakthrough can only come as quality per byte moved, not from arithmetic substitution.** Reaching 7B-class quality at 10× lower energy on M1 (~46 mJ/token) requires streaming ≤~0.4 GB/token, e.g. ternary plus ≤26 % block access of a 7B, or far better quality per parameter [DERIVED]. Only whole-system joules per useful output at matched quality can show it, and the project has never measured that.

## 2. Findings

**F1 — Physics-flavoured claims inventory** (grep counts: "energy" 669× in docs/integration, "Hamiltonian" 87×, "phase transport" 143× in router-research, "overheat"/"Landauer" 0×).

| Claim (where) | Class | Measured? |
|---|---|---|
| Goal "lower energy and wasted compute" on M1 [SOURCE AGENTS.md:5; README.md:3] | measurable engineering objective | No. "Energy savings require physical measurement" [SOURCE README.md:24] |
| D0-b "Energy measured, not estimated" [SOURCE docs/integration/DECISIONS.md:102] | measurement policy (sound) | Never executed; J/token UNAVAILABLE: sudo password and `CPU Power: 0 mW` under load [SOURCE docs/integration/handoff-2026-09-19.md:70-73,120-121] |
| `m1_profiler.rs` 3500 mW, "0% throttling", ">100× energy" [SOURCE docs/integration/recovery-2026-09-08.md:19] | fabricated constants, correctly withdrawn | — |
| 617 tok/s vs 36 tok/s Qwen2.5-1.5B Q4 [SOURCE README.md:58; EVIDENCE.md:218] | measured throughput (unequal quality) | Throughput yes; energy no |
| "Least-energy" routing/read = argmin of a declared score among admitted candidates [SOURCE docs/adr/0004-geometric-intelligence-route-hierarchy.md:296-298] | optimisation objective (Ising/EBM sense), not joules. `measure_energy` even includes a difference of prime atom numbers with no semantic metric [SOURCE docs/integration/principal-attention-computer-science-2026-09-24.md:17] | n/a |
| Three Hamilton concepts; "mathematical Hamiltonian energy [does not] measure electrical joules" [SOURCE docs/integration/principal-attention-plan-2026-09-24.md:71-77] | correct disambiguation | n/a |
| Phase rotations "extremely lightweight … could replace dense compute" [SOURCE router-research/phase_transport_hypothesis.md:47-61,122-133] | engineering claim, wrong as stated: a 2×2 rotation is 4 mul + 2 add. A quaternion lane is a 16-MAC 4×4 map; the only saving is parameters (bytes) | No |
| "10–1000× reduction in energy per operation" [SOURCE router-research/adaptive_field_computer_moonshot.md:161]; "10x to 100x compute or hardware" [SOURCE router-research/CORE_PROJECT_GOALS.md:20] | speculative hardware / aspirational target | No |
| "Oscillations emerge from spectral structure" (graph wave equation) [SOURCE router-research/minimal_theorem_for_spectral_emergence.md:18-32] | true linear-algebra fact; implies no computational advantage | n/a |
| External Allard corpus: "thermodynamic stability", "quantum translation layer", "70B-equivalent" [SOURCE docs/integration/review-2026-09-16/06-external-corpus.md:62-73] | metaphor/unsupported (already assessed) | No |

**F2 — First-principles energy model** (`energy_model.py`; full 60-row table in `energy_tables.md`).

Inputs:
- Per-operation energies at 45 nm (Horowitz): 8-bit add 0.03 pJ, 32-bit add 0.1 pJ, 8-bit mult 0.2 pJ, 32-bit mult 3 pJ, fp16 add 0.4 pJ, fp32 add 0.9 pJ, fp16 mult 1 pJ, fp32 mult 4 pJ; 64-bit cache reads 10 / 20 / 100 pJ (8 KB / 32 KB / 1 MB); DRAM 1.3–2.6 nJ; 70 pJ/instruction. DRAM I/O ">20 pJ/bit", "~10 pJ/bit even when the I/O is improved" [LITERATURE https://pdfs.semanticscholar.org/9476/20a1854655ed91a86b90d12695e05be85983.pdf ; https://pages.cs.wisc.edu/~markhill/restricted/isscc2014_horowitz_power_scaling.pdf].
- 7 nm arithmetic values used as the proxy for M1's TSMC N5 (±2× uncertainty): fp16 add/mul 0.16/0.34 pJ, int8 add/mul 0.007/0.07 pJ [LITERATURE arXiv 2310.11453 Table 2].
- SRAM scaled ×0.4 from 45 nm: L1 ≈0.09 pJ/bit, L2 ≈1.25 pJ/bit, SLC ≈1.9 pJ/bit [DERIVED/HYPOTHESIS].
- DRAM 5 / 10 / 20 pJ/bit scenarios, bracketed by HBM2 3.97 pJ/bit and GDDR5 14 pJ/bit [LITERATURE https://research.nvidia.com/sites/default/files/pubs/2017-10_Fine-Grained-DRAM%3A-Energy-Efficient/oconnor_and_chatterjee.micro2017.pdf]. M1 calibration: AnandTech measured 6.3 W active for single-thread compute vs 10.5 W for DRAM-heavy single-thread → ≥9 pJ/bit incremental at ≤58 GB/s [LITERATURE https://web.archive.org/web/20210123195200/https:/www.anandtech.com/show/16252/mac-mini-apple-m1-tested; DERIVED].
- M1: 68.25 GB/s spec [same AnandTech URL], 61.4 GB/s measured achievable [LITERATURE https://ziraph.com/blog/apples-to-apples-mlx-vs-llama-cpp-gemma-4]. M1 Pro: 200 GB/s spec [LITERATURE apple.com newsroom 2021-10]; ~172 GB/s effective, derived from llama.cpp F16 TG 12.75 t/s × 13.47 GB. P-cluster L2 12 MiB, SLC 8 MiB [LITERATURE doi:10.1109/mm.2022.3169245; https://github.com/open-mpi/hwloc/issues/507 (sysctl); https://en.wikipedia.org/wiki/Apple_M1 (SLC)].

Compact results (M1, DRAM 10 pJ/bit, batch 1, mJ/token; "practical" bits/weight includes packing scales):

| model | format | resident | bytes/token | M1 ceiling tok/s | M1 Pro ceiling tok/s | E_DRAM | E_SRAM | E_arith (mults) | saving from removing mults |
|---|---|---|---:|---:|---:|---:|---:|---:|---:|
| 1.68M UOR | int4 (4.5 b) | L2 | 0.94 MB | cache-resident | cache-resident | 0 | 0.010 | 0.00017 | 0.16 % |
| 30M | int4 | DRAM | 16.9 MB | 3,639 | cache-resident | 1.35 | 0.44 | 0.003 | 0.02 % |
| 125M | fp16 | DRAM | 250 MB | 246 | 687 | 20.0 | 6.4 | 0.063 | — |
| 125M | int4 | DRAM | 70 MB | 873 | 2,444 | 5.6 | 1.8 | 0.013 | 0.02 % |
| 1B | fp16 | DRAM | 2.0 GB | 31 | 86 | 160 | 51.5 | 0.50 | — |
| 1B | int4 | DRAM | 562 MB | 109 | 305 | 45 | 14.5 | 0.10 | 0.02 % |
| 1B | ternary (1.67 b) | DRAM | 209 MB | 294 | 823 | 16.7 | 5.4 | 0.10 | 0.36 % |
| 1B | int4, 10 % access | DRAM | 56 MB | 1,092 | 3,054 | 4.5 | 1.45 | 0.01 | 0.02 % |
| 3B | int4 | DRAM | 1.69 GB | 36 | 102 | 135 | 43.5 | 0.30 | 0.02 % |
| 7B | fp16 | DRAM | 14 GB | 4 | 12 | 1,120 | 360 | 3.5 | — |
| 7B | int4 | DRAM | 3.94 GB | 16 | 44 | 315 | 101 | 0.70 | 0.02 % |
| 7B | ternary | DRAM | 1.46 GB | 42 | 118 | 117 | 37.6 | 0.70 | 0.36 % |
| 7B | ternary, 10 % access | DRAM | 146 MB | 420 | 1,176 | 11.7 | 3.8 | 0.07 | 0.36 % |

**What each lever saves (1B dense, fp16 = 100 %):**

| Lever | Energy left | Saving |
|---|---:|---:|
| int8 | 53 % | 47 % |
| int4 | 28 % | 72 % |
| ternary | 10.5 % | 89.5 % |
| ternary + 10 % block access | 1.0 % | 99 % |
| Removing multipliers at any bit width | — | 0.00–0.36 % |

- **Sensitivity (DRAM 5 → 20 pJ/bit):** arithmetic stays 0.17–0.45 % of the total [DERIVED].
- **The model is consistent with measurement.** llama.cpp Llama-2-7B Q4_0 on M1 runs at 14.19 tok/s, i.e. 53.8 GB/s, 88 % of the measured ceiling [LITERATURE https://github.com/ggml-org/llama.cpp/discussions/4167]. With ~6.5 W GPU power (ziraph's check run) that is ~0.46 J/token = 15 pJ per streamed bit, against a modelled floor of 0.42 J [DERIVED].
- **Caveat on "10 % access":** it saves DRAM energy only if the selected weights form contiguous blocks at least one cache line or DRAM burst in size [HYPOTHESIS]. It is also a form of per-token routing, which conflicts with the owner's "no sparse routing" wish while D5 makes it terminal. **Flag, unresolved.**

**Compute-bound vs bandwidth-bound on M1.**
- **Models above ~20–30M parameters at 4-bit are bandwidth-bound at batch 1.** Their weights exceed L2+SLC, and derived peak int8 SIMD compute (4 P-cores × 4×128-bit NEON × 3.2 GHz ≈ 0.8 T MAC/s) is ~7× above the 61 GB/s streaming rate at 4.5 b/w [LITERATURE gts3.org M1 slides "4 128-bit SIMD units"; DERIVED].
- **The 1.68M model is not bandwidth-bound.** It is 0.80 MiB packed, but served as `Vec<i16>` = 3.2 MiB [SOURCE crates/uor-r4-integer/src/model.rs:29], plus a ≤64 KiB product table and ≤0.33 MB of event memory. All of it is L2-resident, so steady-state DRAM traffic is ~0 and energy = core power × time. Its physics floor is ~0.01 mJ/token [DERIVED].

**F3 — KV/event-memory traffic** (2 B/element, per generated token; MHA configs from bitnet.cpp App. A).

| config | ctx 256 | ctx 4096 | ctx-4096 KV vs 4-bit weights |
|---|---:|---:|---:|
| UOR joint (K64+V256, 1 layer) | 164 KB | 2.6 MB | 2.8× |
| 125M MHA | 9.4 MB | 151 MB | 2.2× |
| 1B MHA / GQA-512 | 50 / 13 MB | 805 / 201 MB | 1.4× / 0.36× |
| 7B MHA / GQA-1024 | 134 / 34 MB | 2.15 / 0.54 GB | 0.55× / 0.14× |

At long context, attention memory rivals or exceeds weight traffic for small models [DERIVED]. **This is the physically real case for bounded admission and exact addressed memory**: an O(1) addressed read in place of an O(T) scan cuts bytes per token. It is also where TurboQuant-class KV quantization pays, reaching quality neutrality at 3.5 bits/channel [LITERATURE arXiv 2504.19874].

TurboQuant also bears on the original "preserve the radius" idea:
- **TurboQuant already stores the vector norm separately.** It does not ablate the radius [LITERATURE same].
- **Its distortion is proven within ≈2.7× (MSE) of the Shannon lower bound**, with MSE ∝4^−b. So any better quantizer can gain at most log₄2.7 ≈ **0.7 bit/coordinate** at equal distortion on worst-case (rotated) inputs [DERIVED].

**F4 — The current integer serving path, physically.**
- **Weight path:** a per-coordinate 16-entry i64 table with a data-dependent gather per 4-bit weight stored as i16. That is ~10 B touched per weight term vs 0.5 B for packed 4-bit [SOURCE model.rs:85-120].
- **Q·K, A·V, gates and the vocabulary mixture:** computed with `product()`, a software shift/add loop over u128 [SOURCE model.rs:324-340; math.rs:58-82].
- **Softmax/normalisation/mixture divisions:** software long division, up to 128 iterations [SOURCE math.rs:110-147].
- **Micro-benchmark** [MEASURED, `kernelbench`: `taskset -c N ./target/release/kernelbench`]. Bit-exact equality checked; one core of a shared, loaded 2.8 GHz Xeon VM (load avg 5–11); min of 5 × 0.25 s. **These are ratios, not M1 absolute times.**

| kernel (ns per op) | baseline x86-64, runs 1–2 | AVX-512 native, runs 1–2 |
|---|---|---|
| repo weight gather (no MUL) | 0.91 / 0.92 | 2.43 / 3.91 |
| hardware MUL i16×i8→i32 (SIMD) | 0.53 / 0.51 | 0.08 / 0.15 |
| packed 4-bit + MUL | 0.67 / 0.68 | 0.16 / 0.23 |
| no-MUL branch-free bit-mask (SIMD) | 1.52 / 1.57 | 0.24 / 0.27 |
| Q·K software `checked_mul` | 127.6 / 125.2 | 64.0 / 70.0 |
| Q·K hardware MUL | 0.77 / 0.75 | 0.21 / 0.30 |
| A·V software `checked_mul` | 163.9 / 170.4 | 80.6 / 120.6 |
| A·V hardware i128 MUL | 3.86 / 3.86 | 1.80 / 2.69 |
| ÷1e8 software `div_round` | 502 / 265 | 109 / 149 |
| ÷1e8 compiler-rt u128 | 64 / 53 | 31 / 51 |

- **Where the post-optimisation time goes** [DERIVED]:
  - Before optimisation, `low_bit_dot` held 83.4 % of profile samples; the continuation then got 4.55× faster [SOURCE integer-serving-result-2026-09-25.md:57,64-67].
  - So ~75 % of the remaining step time is outside the weight kernel.
  - Op counts per full-window token: ~95K software products (81.6K of them in Q·K/A·V), ~9K software divisions and 1.67M weight gathers. At the measured costs above, software mul/div are ~90 % of the step on this VM.
  - **Conclusion:** the multiplier-free arithmetic is now the main energy cost of serving [HYPOTHESIS pending an M1 profile].
- **Integer vs F32:** the integer step is ~6.5× the matched F32 step [SOURCE integer-serving-result-2026-09-25.md:80-82]. At similar core power that is ~6.5× more joules per token [DERIVED].
- **Headroom:** T-MAC reaches ~90 G weight-terms/s per M2 core (BitNet-3B, 30 tok/s single-thread) [LITERATURE arXiv 2407.00088]. That is ~50× the UOR weight path's rate [DERIVED].

**F5 — Measured public results** (all retrieved this session).

| System | Result | Energy measurement |
|---|---|---|
| bitnet.cpp, M2 Ultra [arXiv 2410.16144] | J/token: 700M 0.314 → 0.140; 7B 3.013 → 1.068; 70B 28.02 → 8.42 (fp16 llama.cpp → ternary). tok/s 7B 15.6 → 52.4. Implied power 36 → 27 W (700M), 47 → 56 W (7B), 48 → 73 W (70B) | Method not described in the text [DERIVED power] |
| T-MAC, M2 Ultra [arXiv 2407.00088] | energy −20.6 / −61.2 / −51.3 % (7B-4bit / 7B-2bit / BitNet-3B); power only −10.3 / −10.3 / −17.3 %. BitNet-3B: 30 tok/s on 1 core, 71 on 8; 11 tok/s on Raspberry Pi 5 | powermetrics, 500 ms sampling, ≥120 s runs |
| T-MAC, Jetson AGX Orin, 7B-2bit | 2.12 J/token (CPU llama.cpp) vs 1.54 (GPU) vs 0.66 (CPU T-MAC). A good CPU LUT kernel beats the GPU | as above |
| MatMul-free LM [arXiv 2406.02528] | training memory 82 → 32 GB (−61 %); 13B inference 4.19 vs 48.5 GB (>2.7B uses random weights); Loihi-2 370M: 59.4 tok/s at 70.8 mJ/token; ternary Q/K attention failed to converge | Loihi multi-chip power estimated from single-block measurement; FPGA core dynamic power indistinguishable from the ~13.7 W board idle; throughput projected |
| llama.cpp M-series table [#4167] | Llama-2-7B TG on M1: Q8_0 7.92 / Q4_0 14.19 tok/s. On M1 Pro: F16 12.75 / Q8_0 22.34 / Q4_0 36.41 | none |
| ziraph, M1 16 GB [URL above] | Gemma-4-12B at ~4.9 b/w, 7–7.8 tok/s at 93–97 % of 61.4 GB/s ≈ **1.0–1.37 J/token** (≈17 pJ/streamed bit) | IOReport; author calls the energy leg the weakest |
| BitNet papers [arXiv 2310.11453, 2402.17764] | "71.4× arithmetic energy" | explicitly *estimated*, arithmetic only |

**F6 — Physics framings vs grounded mechanisms** (fit to: no float at serving, ≤4-bit additive/LUT maps, no MoE, transformerless).
- **Modern Hopfield networks** [LITERATURE arXiv 2008.02217]: energy E = −lse(β, Xᵀξ) + ½ξᵀξ + const; one CCCP step is transformer attention. They give exponential capacity, one-step retrieval, and averaging "metastable states" when patterns are not separated.
  - The project's "least-energy read" is exactly this with β→∞ over admitted candidates.
  - **This is the right theory** for setting temperature, separation and capacity.
  - Binary keys scored by XOR + popcount are multiplier-free and cheap [DERIVED].
  - Caveat: ternary Q/K failed in MatMul-free LM [LITERATURE].
- **Unitary/orthogonal RNNs** [LITERATURE arXiv 1511.06464]: |λ| = 1 bounds gradients. They solve copy tasks at T = 500 where LSTM fails, but lose to LSTM on local-structure MNIST.
  - This is the grounded meaning of "preserve the radial direction".
  - It predicts no quaternion-vs-Householder advantage, since both are orthogonal. That matches the repo result: quaternion 2.110368 vs Householder 2.085241 nats/token [SOURCE docs/integration/current-state.md:457-458].
- **Oscillatory recurrences** coRNN [LITERATURE arXiv 2010.00951] and LinOSS [LITERATURE arXiv 2410.03943]: forced harmonic oscillators with bounded energy and gradients.
  - LinOSS-IMEX is symplectic and time-reversible (|λ| = 1); LinOSS-IM is dissipative (forgetting).
  - The state matrix is diagonal, so the ops are cheap and element-wise, with parallel-scan training. LinOSS beats Mamba/LRU ~2× on a 50k-length task.
  - No language-model results yet. **Best physical home for "phases": learned oscillator frequencies.**
- **Hamiltonian/symplectic nets:** conservation conflicts with the forgetting that language needs.
  - Evidence: no-forgetting reversible RNNs fail even vanilla-RNN-level PTB language modelling [LITERATURE arXiv 1810.10999].
  - Low fit for serving.
- **Reversible computation** [same paper]: storing the bits that are forgotten gives 10–15× activation-memory savings in LM training with ~equal perplexity, at 2–3× training compute.
  - Useful for the 16 GB M1 training budget, not for inference energy.
  - Landauer's limit (kT ln2 ≈ 2.9×10⁻²¹ J) is ~10⁹× below today's energy per operation, so it is irrelevant here [LITERATURE https://en.wikipedia.org/wiki/Landauer%27s_principle].
- **Kuramoto / AKOrN** [LITERATURE arXiv 2410.13821]: synchronisation binding; Sudoku out-of-distribution 89.5 % with energy-based voting.
  - Symmetric (energy-guaranteed) variants did worse. There are no language results, and it needs T iterative steps per layer.
  - Low priority.
- **Holographic/VSA superposition:** capacity is linear in d.
  - Recall margin needs d ≳ 2(N−1) ln V, e.g. ~60 items at d = 1024, V = 4096 [DERIVED].
  - The repo measured its VSA layer "inert in scoring" [SOURCE EVIDENCE.md:216].
  - Exact addressed memory is the correct choice for exact recall.
- **Analog in-memory attention** [LITERATURE arXiv 2409.19315]: 6.1 nJ per token per head and up to 10⁵× lower attention energy than GPUs.
  - These are SPICE-simulated (28 nm), use ReLU instead of softmax, and adapt a GPT-2-level model.
  - **This is the genuine physics version of "field computing", but it needs custom silicon — out of scope for an M1.**

**F7 — Zeta zeros and physics.**
- **What is established:** Hilbert–Pólya is still conjectural. Montgomery's pair correlation, 1 − (sin πu / πu)², matches GUE and is supported by Odlyzko's numerics. Berry–Keating H = xp is "still quite far from being concrete" [LITERATURE https://en.wikipedia.org/wiki/Hilbert%E2%80%93P%C3%B3lya_conjecture ; …/Montgomery%27s_pair_correlation_conjecture].
- **The one real statistical property is level repulsion.** Pair correlation ≈ π²u²/3 as u → 0, so near-coincident frequencies are rare, unlike Poisson-random frequencies [DERIVED].
- **Deterministic low-discrepancy sets (equally spaced, golden-ratio) avoid near-coincidences entirely.** RoPE-style log spacing covers scales better: zero density grows only like log T/2π, so zeros are dense at high frequency and sparse at low [DERIVED].
- **Language statistics have no known link to prime distribution** [HYPOTHESIS].
- **Repo evidence:** zeta-disabled beats full (91/96 vs 86/96); the zeta-vs-random-phase control card was never run; zeta is retired from the critical path; the current D8 model path contains no prime/zeta component [SOURCE docs/native_geometric_recovery_973.md:314-316; EVIDENCE.md:252-254; native-core-transition-plan.md:262; crates/uor-r4-training/README.md:63].

**F8 — Measurement tooling in the repo.**
- `scripts/energy_per_token.py` has good hygiene: idle subtraction, a plausibility guard, auto-detected token counts [SOURCE scripts/energy_per_token.py:60-66,237].
- **Weakness 1:** Apple's own man page says powermetrics power values "are estimated … should not be used for any comparison between devices" [LITERATURE https://keith.github.io/xcode-man-pages/powermetrics.1.html].
- **Weakness 2:** the script's primary field, "Combined Power (CPU + GPU + ANE)", by its name excludes DRAM. Older M1 builds printed separate `DRAM Power` and `Package Power` lines [LITERATURE https://apple.stackexchange.com/questions/439304].
- **Weakness 3:** per-token denominators are not comparable across tokenizers.
- **Fix for both stated blockers (sudo password; `CPU Power: 0 mW`):** macmon, a sudoless Rust tool, reports cpu/gpu/ane/**ram_power** and **sys_power** (SMC PSTR) from IOReport [LITERATURE https://github.com/vladkens/macmon ; https://github.com/electricapp/power-monitor].

## 3. Genuinely strong vs weak

**Strong:**
- The repo's own claim discipline on energy: "UNAVAILABLE" rather than invented numbers, the constant-profiler withdrawal, the explicit Hamilton-concept disambiguation [SOURCE].
- Exact addressed/event memory with bounded admission. This is physically the right lever for long context, because it reduces bytes per token (F3).
- A tiny, cache-resident model is the one regime where a laptop can in principle run at ~0.1–1 mJ/token [DERIVED] — at its quality.

**Weak or wrong:**
- "Multiplier removal = energy saving" is wrong on commodity silicon (F2, F4).
- "Phase ops are cheap" is wrong in the sense intended: rotation costs are MACs (F1).
- Zeta phases (F7).
- "Hamiltonian/field" framing as a source of efficiency. It is vocabulary, not a mechanism.

**Irrelevant for M1 software:** Landauer, reversible logic for inference, analog/neuromorphic substrates.

## 4. Recommendations (ranked)

1. **Measure before anything else (1–2 days, no model work).**
   - Run macmon/power-monitor plus a wall meter or battery integration (protocol below).
   - Compare the current UOR integer session against llama.cpp (SmolLM2-135M / TinyStories-scale / Qwen2.5-0.5B), bitnet.cpp BitNet-b1.58-2B4T and MLX on the same M1.
   - Report J per output byte and bits-per-byte (BPB).
   - *Falsifies:* any energy claim, in either direction.
2. **Let the serving kernel use the hardware multiplier/divider, or a T-MAC-style NEON TBL LUT for ≤4-bit weights; pack codes to 4 bits; drop i128 on hot paths.**
   - Expected 3–30× lower J/token for bit-identical outputs [DERIVED from F4].
   - This conflicts with D0-b as written. Recommend the owner re-scope D0-b to "no floating point, ≤4-bit weights, integer MUL allowed", or at least measure the A/B.
   - *Falsify:* bit-exact A/B, J/token over ≥5 interleaved runs.
3. **Adopt bytes-first design targets.**
   - Packed ≤2-bit weights.
   - Event/KV memory quantized to 2.5–4 bits with the norm stored — the physical home of the "preserve the radius, 4-D" idea. Its maximum headroom over TurboQuant is ≈0.7 bit/coordinate [DERIVED].
   - Keep the whole working set L2/SLC-resident for small models; block-structured access for large ones.
   - *Falsify:* IOReport bandwidth counters (bytes/token) vs BPB.
4. **Use Hopfield theory for the admission/read.** β, separation and metastability give principled read temperatures; binary keys + popcount for multiplier-free candidate scoring, exact small re-rank.
   - *Falsify* on the D6 long-range probe vs float dot-product scoring.
5. **Recurrent core: test LinOSS-IM/IMEX or coRNN against the GRU-like core at equal parameters.** Arms: fixed zeta-initialised frequencies vs learned vs log-spaced.
   - *Falsify* with TinyStories NLL; expect the zeta arm ≈ random.
6. **Run the tiny model on E-cores / low DVFS and measure J/token.** Apple claims E-cores use one-tenth of P-core power [LITERATURE https://en.wikipedia.org/wiki/Apple_M1] [HYPOTHESIS: several-fold J/token gain for cache-resident models].
7. **Training-memory only:** reversible recurrence (RevGRU-style) for longer unrolls within 16 GB.
8. **Keep out of the critical path:** Hamiltonian/symplectic serving, AKOrN, VSA memory, zeta phases, analog/neuromorphic.

## 5. The physically honest "free computing from GPU and overheating" + M1 protocol

On an M1 the GPU is not the villain: batch-1 decode on CPU or GPU is DRAM-bound. Energy per token ≈ bytes/token × ~15 pJ/bit (all-in, measured-class) + active platform power × latency; heat is average power.

"Freeing" computing therefore means:
- fewer bits per token (low-bit weights/KV, cache residency, addressed/sparse access);
- finishing sooner and idling (efficient kernels);
- lower voltage and frequency where latency allows;
- fewer tokens per useful result, i.e. quality;
- proving it at matched quality.

Multiplier removal matters only in custom silicon (area), and even there memory dominates [LITERATURE Horowitz; O'Connor].

**Protocol:**
1. **Record the machine state:** model identifier, macOS build, RAM, AC vs battery; fixed brightness or clamshell; Wi-Fi/Spotlight/Time Machine off; Low Power Mode off.
2. **Instruments.**
   - (a) IOReport domains via macmon (cpu, gpu, ane, ram, sys = PSTR), 100–500 ms sampling.
   - (b) Ground truth: wall meter (Mac mini) or SMC battery V×I integrated at 1 Hz on battery.
   - Require (a) sys and (b) to agree within ±10 %.
3. **Thermal state:** pre-heat ≥5 min to steady state, then log SoC temperatures and cluster residency. Report both cold-start and steady-state runs.
4. **Idle baseline:** same state and duration, before and after. Report gross and net.
5. **Workload:** a fixed prompt set; greedy or fixed seed; fixed output length (e.g. 256 tokens); prefill and decode timed separately; plus one end-to-end task including load.
6. **Metrics:** J per output byte (tokenizer-neutral) and J per token; tok/s; bytes/s; peak and steady RSS; DRAM bytes/token from the AMC bandwidth counters (should confirm the bandwidth-bound regime); quality as held-out BPB and task success on the same text.
7. **Baselines on the same machine:** llama.cpp (Q4/Q8), bitnet.cpp (ternary), MLX, with ~quality-matched small models. Plot a J/byte vs BPB Pareto chart.
8. **Statistics:** ≥5 interleaved ABAB runs; report median and IQR; ≥60 s of load per run; reject runs with thermal throttling unless that is the declared condition.

## 6. Open questions

- The M1 actual per-instruction energy and L2 pJ/bit (my SRAM numbers are scaled estimates, ±2–3×).
- Whether IOReport's DRAM domain includes PHY/controller.
- Whether macmon's `sys_power`/`cpu_power` work on the owner's macOS 26 build, where powermetrics reports `CPU Power: 0 mW`.
- The true M1 time split of the UOR step. My ~75–90 % software-mul/div share needs an M1 profile.
- J/token on E-cores vs P-cores for the tiny model.
- The quality per byte moved of geometric vs ordinary arms at larger scale — the only place a real energy advantage could come from, and currently unmeasured (D4).

**Commands:** `python3 exp/physics/energy_model.py > exp/physics/energy_tables.md`; `cd exp/physics/kernelbench && CARGO_TARGET_DIR=./target cargo build --release -j2 && taskset -c 3 ./target/release/kernelbench` (add `RUSTFLAGS="-C target-cpu=native"` and `CARGO_TARGET_DIR=./target-native` for the AVX-512 run).
