# sys2: energy and time per token on an M1, and the Rust serving design closest to the floor

2026-09-26 · Systems and energy engineer · the repository was read-only · scratch outputs are in `$S/lab/exp/sys2/`

**Labels.** **Measured** means I ran it in this sandbox: an x86 Xeon at 2.1 GHz, one thread, 128-bit SSE only, on a host shared with other agents (the process got about 40% of a core). Treat those numbers as ratios; absolute times are inflated about 2.5×. **Measured (project)** means the repository's own M1 measurement. **Literature** means a source retrieved this session. **Derived** means my arithmetic, in `energy_model.py`. **Hypothesis** means untested.

**Scope change from the lead (owner decision).** The likely served model is a converted SmolLM2-135M or 360M:
- ternary or 4-bit linear maps executed with lookup-table (LUT) kernels;
- a Lorentz (hyperbolic) attention read over a quantized cache of 36 bits per key;
- no matrix product that uses a multiplier.

§4–§5 cost exactly that model; §1–§3 give the generic sizes.

## Verdict

- **The multiplier is not where the energy goes.** At 7 nm an int8 multiply costs 0.07 pJ (Literature, Jouppi et al., ISCA 2021). An M1 P-core spends about 340 pJ per retired instruction (Derived). Replacing multipliers is worth ~0%. Removing instructions, bytes, and P-core time is worth everything.
- **After ternarization, the CPU core dominates, not DRAM, unless you move to the E-cores** (Derived):

  | Cost of one ternary weight | Energy |
  |---|---:|
  | Fetch from DRAM | ≈18 pJ |
  | Process on a P-core | ≈90 pJ |
  | Process on the E-cores at 2 GHz | ≈23 pJ |
  | Process on the E-cores at ~1 GHz (background QoS) | ≈7 pJ |

  For comparison, an fp16 weight costs ≈144 pJ from DRAM plus ≈128 pJ of stalled P-core time.
- **Converted SmolLM2-135M at 2K context** (projected, 1 P-core unless noted; Derived):

  | Serving path | J/token | tok/s |
  |---|---:|---:|
  | fp16, llama.cpp-class | ≈44 mJ | 175 |
  | Q4_0 with fp16 KV cache | ≈27 mJ | 207 |
  | Ternary LUT with Lorentz-36 read | ≈19 mJ | 228 |
  | Same, on 4 E-cores | ≈7 mJ | 291 |
  | Same, E-cores at background QoS | ≈4.3 mJ | 143 |
  | Plus drafting from exact memory | ≈2.8 mJ | ~146 |

  The best case is ~15× below fp16 on a P-core. For 360M the fp16 path is ≈111 mJ and the best case ≈6.8 mJ. These projections are not M1 measurements.
- **The Lorentz read over a 36-bit cache is not the energy bottleneck.** For 135M at 2K it replaces a 47 MB fp16 KV stream per token with ~1–2 MB of codes and value lines (−3.3 mJ). It costs ~7% of step time. The weight path is ≥85% of the energy.
- **q·k without a multiplier: use 4-bit product tables (asymmetric lookup scoring), not sign-code popcount.**
  - Both cost about 1 ns per key here (1.06 vs 0.75 ns, Measured).
  - Popcount discards the radius, and cycle 2 showed the radius is what carries the hierarchy.
  - Every step of the integer Lorentz read is table reads, adds and shifts, and it matches its exact definition to 0.989 top-32 overlap with 16-bit tables (Measured).
- **The existing integer runtime is where the waste is** (Measured here, same core):

  | Kernel | Existing path | Proposed path | Ratio |
  |---|---:|---:|---:|
  | Weight kernel, per weight | 767–1032 ps (i16 codes, scalar gather) | 56–66 ps (packed ternary LUT) | 12–16× |
  | q·k, per key | 7.4 µs (64 software shift-add products) | 1.06 ns (36-bit table scan) | ~7,000× |
  | Element-wise product | 156 ns (software) | 3.5 ns (quarter-square table) | ~44× |

- **Speculative decoding from exact memory mainly buys latency.** It divides only the DRAM term. On LUT kernels, verifying k+1 drafts costs ~0.65(k+1) steps (Derived from kernel instruction sharing; Hypothesis until measured), so on P-cores it saves ~10%, and on low-clock E-cores ~35%. It needs position-parallel verification: causal attention supports it; a recurrence with a dense state→gate matrix (today's D8) does not.
- **The single decisive measurement** (§R1): M1 J/token of this Rust stand-in (P- and E-cores) against llama.cpp f16 and Q4_0 SmolLM2-135M at depth 2048. Supported if the E-core stand-in is ≥3× below f16; falsified if <2× below Q4_0.

## Findings and proposals

### 1. An energy model of M1 decode

**Constants (Literature unless marked):**

| Quantity | Value | Source |
|---|---|---|
| DRAM, all-in on M1 | +4.2 W for a DRAM-heavy single thread at ≤58 GB/s gives **≥72 pJ/B (9 pJ/bit)** (Derived) | AnandTech M1 Mac mini, web.archive.org/web/20210123195200/https:/www.anandtech.com/show/16252/mac-mini-apple-m1-tested |
| Single-core read bandwidth | ~58 GB/s from one Firestorm core; STREAM 59 GB/s on the CPU | AnandTech (same); Hübner et al., arXiv 2502.05317 |
| DRAM input/output only | DDR3/4 1,300 pJ per 64-bit access; HBM2 250–450; GDDR6 350–480 | Jouppi et al., ISCA 2021, Table 2 (gwern.net/doc/ai/scaling/hardware/2021-jouppi.pdf) |
| LPDDR4X | Energy is "dominated by the core, not I/O" | JEDEC (Gans) forum slides |
| SRAM read, 7 nm | 8 KB 7.5, 32 KB 8.5, 1 MB 14 pJ per 64-bit, i.e. **0.94 / 1.06 / 1.75 pJ/B** | Jouppi Table 2 |
| M1 L2 (12 MB) and system cache (8 MB) | **~2–5 and ~5–15 pJ/B** | Hypothesis, extrapolated |
| Arithmetic, 7 nm | int8 multiply 0.07 pJ; int32 add 0.03; fp16 multiply 0.34; fp32 multiply 1.31 | Jouppi |
| P-core power | M1 single-core package power 3.68–3.75 W (Cinebench R23) | notebookcheck.net, M2 SoC analysis |
| E-core power | 4 E-cores at 2,064 MHz: <1.5 W. Background QoS runs them at ~1 GHz for 160–170 mW. The same FP thread used 205 mJ on an E-core against 1,280 mJ on a P-core | T. Kaiser, github.com/ThomasKaiser/Knowledge; eclecticlight.co 2022-01-03 and 2023-11-07 |
| Core resources | Firestorm: 4 SIMD pipes; `tbl`, `zip`, `saddw` and `add` at 4 per cycle; 3 loads per cycle. Icestorm: 2 SIMD pipes, 2 per cycle | dougallj.github.io/applecpu |

**Energy per retired instruction** (Derived: power ÷ the instruction rate of the LUT kernels below):

| Core | pJ per instruction |
|---|---:|
| P-core | ≈340 |
| E-core at 2.06 GHz | ≈90 |
| E-core at ~1 GHz | ≈25 |

That is 10³–10⁴× the arithmetic energy. It is Horowitz's instruction-overhead point, now quantified per core type.

**What dominates** (Derived, full table in `energy_tables.txt`):
- **≤15M, ternary:** weights (4.7 MB) sit in the P-cluster L2; energy is ~100% core time.
- **60M (16 MB):** core 84% on a P-core; DRAM 75% on E-cores at background QoS.
- **135M–360M:** core 80–85% on P-cores; DRAM ~70% on E-cores at ~1 GHz, where drafting starts to pay.
- **fp16 transformers:** about half DRAM (weights plus KV cache), half core time stalled on memory.
- **Sanity check.** The bandwidth side reproduces llama.cpp on the M1 GPU: 7B Q4_0 at 14.19 tok/s is 3.56 GiB × 14.19 ≈ 54 GB/s (Literature: llama.cpp discussion #4167).

**Generic sizes at 2K context** (vocab 8,192, multi-head attention, fp16 KV cache; Derived; energy is mJ/token):

| Size | (a) fp16 transformer, P-core | (b) ternary LUT transformer + fp16 KV, P-core | (c) geometric, P-core | (c) 4 E-cores | (c) E-cores, ~1 GHz |
|---|---:|---:|---:|---:|---:|
| 15M | 7.5 mJ · 1,005 tok/s | 5.3 · 1,092 | 2.0 · 1,857 (L2-resident) | 0.9 · 2,282 | 0.5 · 1,221 |
| 60M | 24.2 · 321 | 15.1 · 377 | 7.3 · 603 | 2.7 · 761 | 1.6 · 381 |
| 150M | 56.5 · 139 | 32.5 · 171 | 17.6 · 252 | 6.6 · 321 | 3.9 · 157 |

- **Measured (project):** today's 1.7M D8 integer step takes 3.695 ms per full-window call on the owner's M1, i.e. ≈13.7 mJ/token at 3.7 W (Derived).
- That is about **8 nJ per weight**, ~90× the proposed ternary LUT cost.
- **A 135M model with proper kernels would cost about what today's 1.7M model costs per token.**

### 2. LUT kernels and D0-b

**Literature.**
- **T-MAC** (arXiv 2407.00088).
  - Groups of g=4 weights, one bit-plane at a time.
  - A 16-entry table per activation group lives in one NEON register and is read with `vqtbl1q_u8`.
  - Table entries are rounded to int8; "mirror consolidation" halves the table.
  - Results: BitNet-b1.58-3B at 30 tok/s on one M2-Ultra core and 71 on eight; energy −20.6% / −61.2% / −51.3% against llama.cpp (powermetrics at 500 ms, runs ≥120 s).
- **bitnet.cpp** (arXiv 2502.11880).
  - TL1 maps 2 ternary weights to a 4-bit index (9 combinations).
  - TL2 maps 3 weights to a 1-bit sign plus a 4-bit index, 1.67 bits per weight; the sign is applied with XOR and ADD.
  - TL\*_1 keeps exact int16 tables by doing two lookups per entry.
  - I2_S is multiply-add based (`sdot`).
  - M2 Ultra tok/s at 700M: f16 110.65, Q4_0 197.38, T-MAC 220.22, TL2_0 229.21, I2_S 238.16. At 7B: 14.87, 39.47, 53.37, 55.42 and 54.74.

**D0-b mapping (Derived).**

| Step | D0-b status |
|---|---|
| Table construction | Adds and subtracts only (I use ±x₀±x₁, built by adds) |
| Lookup | `tbl` |
| Accumulation | Widening adds |
| Sign (TL2) | XOR and ADD |
| Combining bit-planes | Shifts |
| I2_S, Q4_0 and Q8_0 in llama.cpp | Excluded: they use `sdot` or `mul` |
| T-MAC float scales and bitnet.cpp's absmax activation division | Must become power-of-two scales, which the project's per-row exponents already are, plus leading-zero normalization |
| T-MAC "fast aggregation" (`vrhaddq`) | Allowed (additive) but lossy |

**Measured (`geo-bench`, bit-exact against a multiplication reference; ps per weight, 1536×576 / 2560×960):**

| Kernel | ps per weight |
|---|---:|
| Existing i16 gather | 767 / 1032 |
| **TL1 ternary, exact int16 tables** | **56 / 66** |
| Element-wise signed-4-bit LUT | 130 / 165 |
| f32 multiply-add | 456 / 503 |
| int8 multiply-accumulate (not D0-b) | 191 / 209 |

At equal 128-bit width, the ternary LUT kernel is 3× faster than int8 multiply-accumulate.

**M1 rates (Derived: 14 SIMD operations per 64 weights for TL1 and per 32 for 4-bit, × dougallj throughputs, × 0.7):**

| | Ternary | 4-bit |
|---|---:|---:|
| P-core | ≈41 G weights/s | ≈20 G weights/s |
| One E-core | ≈13 G weights/s | ≈6.6 G weights/s |

Rounded int8 tables (TL1_0-style) should reach ~70 G/s on a P-core, consistent with T-MAC's ~90 G/s on M2-Ultra.

**Proposal P1: replace `low_bit_dot` with packed-nibble TL1/TL2 kernels and a 4-bit element-wise kernel for the head.**
- Expected gain: 12–16× on the weight path (Measured ratio).
- Cost: ~2 weeks.
- Falsifier: bit-exact parity plus M1 timing below 1.5 ns per weight.

**Blocker (Measured, compile check).** NEON intrinsics need `unsafe`, for the loads and for calls from outside `#[target_feature]` functions. They cannot live under the crate's `#![forbid(unsafe_code)]`. It needs an owner decision (question 1 at the end).

### 3. Geometric read cost, 36-bit keys and drafting

**The integer Lorentz read as specified in cycle 3 §6** costs 64 products per candidate. With D0-b's software products that is **7.4 µs per key (Measured), or 15 ms per query head at 2K.**

**The multiplier-free version I built and measured:**
- **Keys.** 8 × 4-bit direction plus a 4-bit radius code, stored in fast-scan blocks of 32 keys, **bucketed by radius code**.
- **Direction score s = ⟨q,u⟩.** Query table rows come from an offline 256×16 product table (no per-query multiplies); `pshufb`/`tbl` then scans 16 keys per instruction.
- **Radius.** The radius levels have ≤2 set bits (0.125…24), so r·s is two shifts and an add. q₀k₀(r) takes 16 quarter-square products per query head.
- **Selection.** A two-pass bound (the k-th best of the per-block best) filters candidates in s-space with one i16 compare; exact z is computed for the survivors (~194 of 2,048).
- **Top-32 weights.** arcosh and exp come from tables.
- **Values.** 7-bit mass × 4-bit value products are read with `tbl` from an offline 128×16 table.

**Measured at T=2,048 per query head:**

| Scoring | Time |
|---|---:|
| Full read | 12.9 µs (8-bit tables) · 15.5 µs (16-bit) |
| Lookup scan alone | 2.16 µs (1.06 ns/key) |
| Sign-code popcount | 1.55 µs (0.75 ns/key) |
| Hardware int16 dot (poorly vectorized comparator) | 39 µs |

**Accuracy (Measured).** Top-32 overlap with the exact function on the same codes is 0.935 with 8-bit tables and **0.989 with 16-bit tables** (0.976 / 0.997 at T=256). The 36-bit code itself against full-precision keys is 0.836 / 0.941, cycle 2's loss. On M1 a P-core should do the scan in ~0.3 µs and the full read in ~0.6–1.2 µs per query head (Derived).

**Popcount against product tables.** Popcount on sign codes costs the same per key but drops the radius. Cycle 2 found radius-less sign codes score below the trivial rule on hyperbolic keys, and cycle 1 measured +0.014 bits/byte for Hamming scores used as final scores. Keep popcount for admission only, and only at ≥32K contexts, where a 64-bit admission scan is cheaper than scoring 36-bit keys.

**E8 block codes.** Their 240-entry tables exceed one `tbl` (up to 64 bytes), so expect 2–4× the 4-bit lookup cost (Derived).

**Cache footprint at 2K for SmolLM2-135M** (90 layer × KV-head pairs; Derived):

| Format | Size |
|---|---:|
| 36-bit keys | 0.83 MB |
| fp16 keys | 23.6 MB (28×) |
| 4-bit values + exponent | 6.1 MB |
| fp16 values | 23.6 MB |
| **Total, fp16** | **47.2 MB** |
| **Total, codes** | **6.9 MB**, L2-resident if weights are streamed with non-temporal loads |

Per head at 2K: 9.2 KB of 36-bit keys against 32 KB (fp16, 8 dimensions) or 256 KB (fp16, 64 dimensions).

**Drafting from exact memory (Literature):**

| Method | Result |
|---|---|
| REST (arXiv 2311.08252) | Mean accepted length 1.96–2.65 on HumanEval (0.9–27 GB datastore); 1.62–2.36× speedups; retrieval <1 ms |
| Prompt-lookup decoding (github.com/apoorvumang/prompt-lookup-decoding) | ~2.4× on summarization and context QA; small gains on the first chat turn |
| SuffixDecoding (arXiv 2411.04975) | 6.3–7.8 accepted tokens per step on agentic and SWE tasks; prompt lookup 2.4–3.2 there; open-ended Spec-Bench 1.66× for suffix-only drafting; ~20 µs per draft token on CPU |

**Proposal P2: an adaptive-length drafter over the session's token tape** (a suffix automaton, drafting max length α·match length).
- Energy gain: DRAM term ÷M; −20 to −35% J/token on E-cores. Latency gain ~M× only if verification runs on spare cores.
- Cost: ~1 week.
- Falsifier: acceptance M < 1.3 on the owner's chat logs.
- Recurrent hybrids: keep any added geometric state linear in h (input-only gates) so verification stays position-parallel.

### 4. The Rust serving design

**Data layout.**
- **Weight tiles.** Blocks of 32 rows. Each step is one 16-byte vector: the low nibble covers rows 0–15, the high nibble rows 16–31. Tiles are in execution order, 64-byte aligned, and streamed.
- **Activations.** int8 with a power-of-two exponent. The TL1 tables (9 sums in int16 byte planes, 32 B per pair) are built once per input and shared by q/k/v and by gate/up.
- **Memory per layer × KV head.** Radius-bucketed key blocks, u16 positions, 33-byte values, the u16 token tape, and the suffix automaton.
- **Sealed tables** (≈0.5 MB, hash-bound like today's): query-by-level product table 4 KB, 16-bit variant 8 KB, mass-by-value 4 KB, quarter squares 128 KB, arcosh 128 KB, exp 16 KB, k₀(r), SiLU and 1/√x.

**Per-token sequence** (converted layer):
1. RMSNorm: squares from a table, a 1/√x table, and the gain folded into the next map's row exponents.
2. Quantize to int8 and build the TL1 tables.
3. q/k/v with TL1.
4. Encode k (radius by table; direction by 16-level compares against radius-scaled thresholds, which are shifts) and append it to its bucket.
5. Run 9 (or 15) Lorentz reads.
6. o with TL1, then the residual add.
7. MLP: gate and up with TL1, SiLU by table, gate⊙up by quarter squares, down with TL1.

After 30 (or 32) layers: the 4-bit head (49,152×576, 21% of the weights), then the existing exact Q48 softmax and integer sampler. With drafts, steps 1–7 run for k+1 positions per layer pass.

**Threads.**
- Default ("eco"): E-cores at background QoS, 4 threads, row blocks split per map with one barrier per map.
- "Fast": one P-core.
- No allocation in the steady state.

**What `uor-r4-integer` already provides:** sealed bundles and hashes, the Hugging Face byte-level BPE engine (ceiling 2²⁰ tokens), text sessions with the exact occurrence tape, integer tables, exact Q48 normalization, integer greedy and categorical sampling, signed-4-bit codes with per-row exponents, and report roots.

**What must be added:**
- Packed tiles and the NEON kernels, with scalar parity tests.
- The int8 interface. Today the activations are 16-bit Q8 codes held in i32.
- A general layer graph (grouped-query attention, SwiGLU; `validate` currently fixes vocab 4,096, width 256, context 256).
- The Lorentz-36 store and read.
- Quarter-square tables in place of `checked_mul`/`div_round` on hot paths.
- Multi-position verification and the drafter.
- QoS and thread control.
- The energy harness.

**Measured stand-in.** `geo-bench --step135/--step360` runs the complete multiplier-free decode step with synthetic weights:
- all ternary projections, the 9/15 grouped-query Lorentz reads over 2,048 keys, and the 4-bit 49,152-row head;
- 40.7 / 102 MB packed weights and 6.7 / 12 MB of cache.

Here it took 46.7 / 117.6 ms/token wall at ~40% CPU share, i.e. ≈19 / 44 ms of CPU per token. Context 256 against 2,048 changes the time by only 8%. My M1 P-core projection is 4.4 / 10.6 ms, a plausible 4× from clock, `tbl` width and contention.

**Projection for the converted models** (Derived; J/token · tok/s, context 2K):

| | 135M, 1 P-core | 135M, 4 E | 135M, E ~1 GHz | 360M, 1 P-core | 360M, 4 E | 360M, E ~1 GHz |
|---|---:|---:|---:|---:|---:|---:|
| (a) fp16, llama.cpp-class | 43.9 · 175 | 35.5 · 94 | — | 111 · 70 | 90.5 · 37 | — |
| (b) Q4_0 + fp16 KV | 26.7 · 207 | 14.3 · 218 | — | 64.8 · 84 | 33.6 · 93 | — |
| (b) ternary LUT + fp16 KV | 25.0 · 198 | 12.0 · 210 | — | 57.1 · 85 | 26.2 · 94 | — |
| (c′) ternary + Lorentz-36 | 19.3 · 228 | 7.2 · 291 | 4.3 · 143 | 47.0 · 94 | 17.6 · 120 | 10.5 · 58 |
| (c′) + drafts, M=2 | — | 5.6 · 298 | 2.8 · 146 | — | 13.7 · 123 | 6.8 · 60 |
| (c′) + drafts, M=5 (code editing) | — | — | 2.0 · 137 | — | — | 4.7 · 56 |

- **Baseline anchors (Literature):** llama2.c stories15M, fp32, ~110 tok/s on an M1 Air (github.com/karpathy/llama2.c), ~9× below my optimized-fp16 row; M4-Max llama.cpp Qwen2.5-0.5B Q4_K_M 297 tok/s (github.com/john-rocky/apple-silicon-llm-bench), far below its bandwidth ceiling. Implementations vary ~10×; only measurement decides.
- **Floor for 135M on M1 without routing:** ≈2–3 mJ/token = weight DRAM ÷M + low-clock E-core compute.
- **Going lower requires fewer weight bytes per token.** TL2 saves 17%. A two-level head cuts up to ~10 of the 40.7 MB/token (≈25%), but is lossy. Conditional computation is against current policy.
- **Latency is not the constraint.** Every row is above 50 tok/s, far faster than reading speed.

### 5. Proposals, ranked by joules saved per week of work

| # | Proposal | Expected gain | Cost | Cheapest falsifier |
|---|---|---|---|---|
| P1 | Packed TL1/TL2 kernels plus element-wise 4-bit head (§2) | 12–16× on the weight path | ~2 weeks | Parity; M1 below 1.5 ns/weight |
| P3 | Run on E-cores at background QoS | 3–6× J/token (Hypothesis, from Literature powers) | 1 day | §R1 run with `taskpolicy` |
| P4 | Quarter-square tables in place of software `checked_mul`/`div_round` | 44× per product (Measured) | 2 days | Bit-exact test |
| P5 | Lorentz-36 cache replacing the fp16 KV cache | −3.3 mJ (135M) / −6 mJ (360M) per token | 2 weeks | Top-32 overlap ≥0.98 on converted-model queries |
| P2 | Drafts from the token tape | −20–35% J/token on E-cores | 1 week | M < 1.3 |

## Recommended next experiments (ranked)

**R1 (owner's M1): the single most important measurement.** Measure idle-subtracted CPU and DRAM joules per token for the Rust stand-in against llama.cpp, at SmolLM2-135M size and 2K depth. It tests the claim's dominant terms: per-weight core energy by core type, DRAM energy per byte, and the size of the context term.

1. **Prepare.** AC power, low fixed brightness, other apps closed, `sudo mdutil -a -i off`. Build `$S/lab/exp/sys2/geo-bench` with `cargo build --release` (its NEON path type-checks for `aarch64-apple-darwin` but has not been run on an M1). Build llama.cpp and download the f16 and Q4_0 GGUF files of SmolLM2-135M-Instruct.
2. **Log power.** For every run, in a second terminal: `sudo powermetrics -i 200 -s cpu_power,gpu_power,ane_power --show-process-ipc -o run_X.txt`. Keep "CPU/GPU/ANE Power", plus "DRAM Power" and "Package Power" where your macOS prints them, plus per-process instructions and cycles. If `sudo` or "CPU Power: 0 mW" blocks this, use `macmon pipe -i 200 > run_X.json` (sudoless). Apple's man page says these power values are estimates, so compare only on the same machine.
3. **Differential runs.** Run each configuration at two lengths so that setup and prefill cancel out, three times each, in random order, 30 s apart:

   | Configuration | Command, run at two lengths |
   |---|---|
   | Stand-in, P-core | `./geo-bench --step135 --seconds 20` and `--seconds 80` |
   | Stand-in, E-cores | the same pair, prefixed with `taskpolicy -c background` |
   | Context sensitivity | the same pair with `--ctx 256` |
   | llama.cpp, CPU | `llama-bench -m smollm2-135m-{f16,q4_0}.gguf -t 1 -ngl 0 -p 0 -d 2048 -r 1 -n 256` and `-n 1024` |
   | llama.cpp, GPU | the same with `-ngl 99` |
   | Current project session | one `uor-r4-integer generate` run for reference |

4. **Compute.** J/token = [(E₂ − E₁) − P_idle·(t₂ − t₁)] / (n₂ − n₁), where E is the integral of the reported powers and P_idle comes from a 60 s idle log. Also record instructions per token; the model predicts ≈0.27 instructions per ternary weight.
5. **Decision.** The energy claim holds if the E-core stand-in is ≥3× below llama.cpp f16 on CPU and ≥2× below Q4_0. It is falsified if it is <2× below Q4_0. Projected: 7 / 44 / 27 mJ.

   The 360M run and the 256-context variant fit pJ/B (the slope in bytes) and pJ per weight (the intercept), which recalibrate every row above.

**Further experiments:**
- **R2 (here, 1 day).** Rounded int8-table TL1_0 and TL2 in `geo-bench`: speed against the exact int16 tables.
- **R3 (here).** Lorentz-36 top-32 overlap on real converted-model q/k from SmolLM2 layers, once the conversion has keys.
- **R4 (M1).** Four-thread E-cluster scaling of `--step135`. The E-cluster DRAM bandwidth (30 GB/s assumed) is unmeasured.
- **R5 (here).** Draft acceptance M on the project's own chat and code transcripts, replaying the drafter against logged greedy outputs.

## Open questions for the owner

1. **May one small, audited SIMD kernel crate contain `unsafe`**, with scalar-reference parity tests, so that the portable crates keep `forbid(unsafe_code)`? Without it the NEON LUT kernels cannot exist on stable Rust (Measured compile check), and serving stays 12–16× slower on weights.
2. **Should the default serving mode be "eco"** (E-cores at background QoS, ~140–290 tok/s for 135M) with a "fast" P-core mode on request?
3. **Are rounded (int8) LUT tables acceptable** as a serving option (~1.7× faster, small loss), or must serving stay bit-exact with int16 tables?
4. **D0-b and element-wise products.** Does D0-b allow the hardware integer multiplier for element-wise activation products (SwiGLU gate, norms, masses), or must those use quarter-square tables? Tables cost 1.8× hardware (Measured) and are negligible either way; they are 44× cheaper than today's software loop.

**Artifacts** (in `$S/lab/exp/sys2/`):
- `geo-bench/`: Rust, x86 and NEON paths, with exactness assertions.
- Results: `bench_full.txt`, `step135*.txt`, `step360.txt`.
- `energy_model.py` and its output `energy_tables.txt`.
- `safecheck/`: the `forbid(unsafe_code)` compile test.
