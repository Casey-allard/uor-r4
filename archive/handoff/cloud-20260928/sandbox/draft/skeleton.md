# Skeleton + verified inserts (lead working draft)

## Verified inserts so far
- Model: vocab 4096 BPE (TinyStories), d=256, read 64, ctx 256; ~1.68M params, 1.05M tied embedding, ~0.63M non-emb [lit measured from config.rs:59-85].
- #1017 reference transformer: 7,155,360 params, 6 layers, width 288, 150M tokens, 1.574 nats; current learners 2.085–2.110; 5-gram+cache 2.392.
- TinyStories (2305.07759): consistency 1M/8L 1–3/10, 2.5M 3–6/10, 8.3M 5–8/10, 28M 8–9/10.
- Premise correction: TurboQuant/PolarQuant/QJL all STORE the norm; they remove per-block scale/zero-point.
- HQMQ (2605.27646, MIT/IBM, May 2026) = 4-D quaternion KV chunks, quantized radius (3–6 bits), direction = 2T·q_s; ~fp16 at ~5 bits; beats TurboQuant-style at ~3 bits on Llama-3-8B (+0.745 vs +1.118 ppl); E8 variant underperformed 24-cell.
- Lit measurement (Gaussian, MSE/dim @2 bits): scalar 0.117, 2-D polar 0.119, 600-cell gain–shape 0.107, best learned 4-D shape 0.106, 4-D VQ 0.098, E8P 0.089, trellis 0.069, Shannon 0.0625. @3 bits 600-cell 0.071 worse than scalar 0.035. fp16 radius/block: 5.73 bits/dim, MSE 0.071.
- State tracking: Illusion of State (2404.08819): S4/Mamba in TC0; A5 canonical. Grazzi 2411.12537: DeltaNet 1.3B WikiText ppl 18.57 vs 18.54 (LM unchanged), gains on code/math.
- Automaton Underneath (2609.18966): additive input path = parasitic attractor; −b learns exact automaton to 16× length; reflection-length law: A5/S5 need 4 reflections/token in defining rep; quaternion left mult (isoclinic) = 4 reflections; control H(v)H(e0) ≤2 reflections, plane containing e0 [SOURCE joint_model.rs:1741-1777 per lit].
- Allen-Zhu & Li 2404.05405: 2 bits/param; int8 no loss; post-training int4 → 0.7 bit/param; MoE 32 experts using 8.8% params per token loses only 1.3×; junk data hurts.
- MOHAWK 2408.10189: Phi-1.5 (1.3B) → Phi-Mamba with 3B tokens, reusing MLPs/emb/head (frozen MLP OK); avg 62.6 vs teacher 64.9 vs Mamba-2 (315B tok) 59.6.
- LoLCATs 2410.10254: Llama-3-8B linearized with 40M tokens (attention transfer MSE + LoRA), ~6.5 A100-hours; also 1B models with 40M tokens.
- T-MAC 2407.00088: LUT bit-serial; powermetrics energy −20.6/−61.2/−51.3% vs llama.cpp; BitNet-3B 30/71 tok/s on 1/8 M2 Ultra cores.
- BitNet 2B4T: 54.19 avg vs Qwen2.5-1.5B 55.23; 29 ms/token CPU; 0.028 J/token is an estimate.
- MatMul-free LM: ternary Q/K attention failed to converge -> element-wise GRU token mixer; 1.3B 46.2 vs 48.0.
- Intelligence per Watt 2511.07885: B200 1.6–2.3× more intelligence/J than M4 Max at batch 1.
- llama.cpp #4167: M1 14.2 tok/s LLaMA-7B Q4_0; M1 Pro 35.5–36.4.
- Golden gates (1704.02106): icosahedral C60 + T=(2+φ)i+j+k, norm 7+5φ (N=59): optimal almost-covering asymptotically, efficient navigation; icosians form an E8 lattice copy; golden rotation optimal U(1) generator.
  LEAD MEASURED: level-1 = exactly 3600 rotations, exact ½Z[φ] coords after √(7+5φ) scaling; covering NOT better than random at equal N (mean 9.32° vs 8.83°, max 30.5° vs 22.8°); level-2 sample ≈ random; near-identity hole (T-count≤1 mean 13.4° vs random 10.0°; 2I smallest rotation 72°).
- KBS 1310.4150: exact synthesis over Z[ω]/Z[τ], O(log 1/ε), Gauss complexity = sum of Galois-embedding norms.
- Integer small-rotation codebook (lead): q=(2^k,a,b,c), a,b,c∈{−1,0,1}, k=0..4, plus half-turns: 157 rotations, finest 7.15°; shift+add application; norm folded into radial decay.

## Sections
0 Bottom line
1 What exists (facts)
2 Lineage and verdict per idea
3 CS stress test [arch, verify]
4 Math stress test [math, statetrack]
5 Physics stress test [physics]
6 Experiments [quant, statetrack, lead text LM, golden gates]
7 Diagnosis [audit]
8 Theory
9 Path
10 Ideas that ease research
11 Owner decisions
App: novelty map, evidence, refs
