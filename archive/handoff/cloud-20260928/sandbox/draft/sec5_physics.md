## 5. Physics stress test

### 5.1 Where the energy of a token goes

At batch-1 decoding, energy goes to **moving bits** and to **instruction overhead** on programmable cores, not to multiplier circuits. Reference figures:
- DRAM costs about 5–20 pJ/bit. The all-in figure for the M1, calibrated from AnandTech measurements, is at least about 9 pJ/bit.
- At 45 nm, a CPU instruction carries about 70 pJ of overhead, against about 3 pJ for a 32-bit integer multiply (Horowitz, ISSCC 2014).

The physics agent's model, at a DRAM cost of 10 pJ/bit (physics report F2):

| Model (M1) | Bytes/token | DRAM energy per token | Saving from removing all multiplies |
|---|---:|---:|---:|
| 1B, fp16 | 2.0 GB | 160 mJ | — |
| 1B, int4 | 562 MB | 45 mJ | 0.02% |
| 1B, ternary | 209 MB | 16.7 mJ | 0.36% |
| 1B, int4, 10% of weights touched | 56 MB | 4.5 mJ | 0.02% |
| 7B, int4 | 3.94 GB | 315 mJ | 0.02% |

In the DRAM-bound configurations, multipliers account for at most about 0.4% of the energy floor; for a cache-resident ternary model the figure is at most 3.4%. **Bit width, bytes touched per token and instruction count decide energy.**

Measured evidence has two parts:
- **Lookup-table kernels.** Multiplier-free LUT kernels *do* save energy when they remove instructions. T-MAC ran the *same* 4-bit and 2-bit models as llama.cpp and cut measured energy 21–61% by replacing dequantize-and-multiply work with table lookups (Literature 2407.00088 §5.4).
- **Fewer bytes.** Ternary models save more through fewer bytes. bitnet.cpp reported 55–70% on an M2 Ultra; its measurement method is not described (Literature 2410.16144).

The multiplier-free half of D0-b is sound when it takes T-MAC's form, a table lookup replacing a multiply. It is counterproductive when it *adds* instructions, as the current software emulation does.

### 5.2 The current serving path is about 10³× above its own floor

The 1.68M-parameter model fits in the M1's 12 MiB L2 cache, so steady-state DRAM traffic is near zero. Yet its 3.695 ms per call corresponds to an estimated **11–23 mJ/token**. That is roughly the DRAM-floor energy of a well-implemented 125–300M-parameter 4-bit model (physics report, Derived).

The cause is emulated arithmetic (§3.4):
- about 79% of step time is spent in software multiply and divide on x86;
- per product, software emulation is 45–310× slower than the hardware instruction.

On a general-purpose CPU, the multiplier circuit is present and powered whether or not it is used. Replacing one multiply instruction with dozens of instructions therefore very likely *raises* energy. This is Derived; it has not been measured on the M1.

### 5.3 "Least energy", "Hamiltonian", "phase transport", "field"

These terms are optimisation vocabulary or metaphor. The repository's current plan already says so (principal-attention-plan §4).
- "Least-energy routing or read" is argmin of a learned score, which is hard attention (§2.2).
- A rotation is multiply-accumulates unless it is drawn from a finite codebook, and then its saving is bytes and instructions.
- The router era's claims of 10–1000× lower energy per operation and 10–100× hardware gains were never measured.

**No joule has ever been measured in this repository.** The one earlier "measurement" was a hardcoded 3,500 mW constant, withdrawn on Sep 8 (recovery-2026-09-08.md).

Physically grounded ideas that do fit the constraints (physics report §3):
- **Modern Hopfield theory for the "least-energy read".** Attention is one energy-minimization step, and binary keys can be scored by XOR plus popcount.
- **Norm-preserving or oscillatory recurrent cores** (LinOSS, coRNN), with learned rather than fixed frequencies.
- **Reversible training.** It saves RAM on a 16 GB machine; it does not save serving energy.
- **Efficiency-core execution under DVFS.**

### 5.4 Zeta zeros

There is no physical or information-theoretic reason for zeta-zero phases to help a language model.
- **Spacing.** Their GUE-like level repulsion makes them a well-spread set of incommensurate frequencies. Log-spaced, low-discrepancy and learned sets have the same property.
- **Covering.** The golden-ratio rotation is the provably optimal U(1) covering generator.
- **Measured** (§4.2): as token codes they collide far more than a hash does; as frequencies they are no better than random; and at log-primes they encode primality.
- **Current use.** The current model contains no zeta component.

### 5.5 What "freeing computing from the GPU and overheating" physically requires

On an M1 the GPU is not the villain. Batch-1 decoding is bandwidth-bound or overhead-bound on either unit, and heat is simply average power. Four things are needed:

1. **Fewer bits per token.** Low-bit weights; compressed KV and event memory with a quantized norm, which is where the owner's "preserve the radius" instinct physically belongs; cache residency; and, for models larger than cache, sparse or blocked access.
2. **Kernels that finish sooner with fewer instructions.** T-MAC-class lookup-table kernels, not emulation.
3. **Fewer tokens per useful answer.** Quality is an energy lever.
4. **A measurement at matched quality.**

The physics report specifies a measurement protocol (§5):
- **Instruments.** `macmon` reads IOReport power without sudo, and so *may* remove the repository's recorded blockers; support on the owner's macOS 26 is unverified. Ground truth comes from a wall meter or battery integration.
- **Controls.** Thermal pre-heat, idle subtraction and interleaved ABAB runs.
- **Metrics.** Joules per output byte and per token, plotted against bits-per-byte on the same text.
- **Baselines.** llama.cpp, bitnet.cpp and MLX on the same machine.

Until that measurement exists, every energy statement is a hypothesis.
