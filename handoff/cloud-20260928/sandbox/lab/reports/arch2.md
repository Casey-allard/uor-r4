# arch2: the Geometric Chat Model (GCM), and the path from D8 (2026-09-26)

Labels: **Measured** (run here; scripts in `$S/lab/exp/arch2/`), **Derived** (arithmetic in `gcm_calc.py`, output `gcm_calc.json`), **Literature** (retrieved this session, arXiv id), **Hypothesis**. The plan is centred on today's owner decisions: weight transfer approved, hyperbolic geometry as the lead mechanism, and offline training in scope. **Environment fact:** huggingface.co is blocked by egress policy (403), so SmolLM2 weights could not be fetched here. Every real-teacher test must run on the owner's M1.

## 1. Verdict

- **Chat needs training tokens, and the M1 cannot supply them from scratch.**
  - Open small talk and basic instruction following appear at about 100–350M parameters, but only after 1–11T training tokens. MobileLLM-350M wins 47% on AlpacaEval against text-davinci-001; SmolLM2-360M scores 3.66 on MT-Bench; LFM2-350M scores 65 on IFEval.
  - Narrow-domain chat works at 15–30M (TinyStories).
  - An M1 trains about 0.2B tokens per week at 150M parameters (Derived).
  - **Converting an open instruct model is the only route to the 135–360M band, and it is now approved.**
- **Primary model (GCM-L): SmolLM2-135M-Instruct converted into a geometric hybrid.**
  - Embeddings and MLPs are kept, at 4 bits.
  - 8 of the 30 layers get exact hyperbolic (κ-Lorentz) reads.
  - 22 layers get a 64-token κ-Lorentz window plus a recurrent state, transported by the teacher's own RoPE rotations.
  - It reads about 70 MB per token at 2K context, against 316 MB for the teacher in bf16 (Derived).
- **New mechanism: a curvature homotopy that makes the conversion exact.**
  - Here κ is the curvature magnitude. The score is s_κ = [|k|² − d_κ(q,k)²]/(2√r), with d_κ the Lorentz distance.
  - As κ→0 it equals the teacher's dot-product softmax *exactly*. Measured: KL 1e-17 at κ=0 and 2–3e-6 at κ=1e-4, rising smoothly after that.
  - RoPE survives, because rotating only the spatial coordinates is a hyperbolic isometry (invariance error 5e-16).
  - The cycle-3 fixed-curvature "Dot-matched" start is **not** a copy of a trained head: with the best inverse temperature β it left 0.17 nats of KL on a diffuse head (Measured).
- **q·k can be served without a multiplier at int8, not at per-tensor 4 bits.**
  - Measured KL: at most 3e-4 nats for int8 q and k; up to 0.12 for per-tensor 4-bit.
  - The kernel is T-MAC's lookup-table trick transposed: per-query multiples tables over the int8 key bytes, one table read and one add per coordinate.
- **The KV cache must be hybrid.** Fully recurrent conversions lose heavily: MT-Bench 5.64 vs 7.35 in Mamba-in-the-Llama; LoLCATs MMLU 23.8 vs 52.8 without its window. The hybrid holds 8.1 MB at 2K, against 47 MB for the teacher in bf16.
- **Parameter memory: use deterministic n-gram addressing (Engram/L³), not product-key queries.** It gives D5-style per-token sparsity with no learned gating, it can be prefetched, and it fits in M1 DRAM up to about 1–2 GB.
- **The D8 throughput problem is solvable.** A parallel GCM-S-shaped stack reached about 29 GFLOP/s-equivalent on one contended thread, against about 2.7 for the D8 sequential unroll. Its scan equals the sequential recurrence to 1e-7 (Measured).
- **Compute is the real decision.** At an assumed 0.3 TFLOP/s the M1 needs about 31 h for the conversion pilot and 10–50 days for full 4-bit QAT distillation. A single H100 would take about 1 hour (Derived at an assumed 40% utilization).

## 2. Findings and proposals

### F1. What small efficient models reach (Literature)

| Model (arXiv) | Size / tokens | Result |
|---|---|---|
| TinyStories (2305.07759) | 1–80M | 28M/8 layers: 9/10 consistency; about 80M: near-perfect grammar and consistency. 1-layer models struggle with instructions; 2 layers are enough to some extent |
| DialoGPT (1911.00536) | 117–762M, 147M Reddit exchanges | 345M vs human replies on relevance: 45% vs 47%, p=0.055 |
| MobileLLM (2402.14905) | 125M / 350M, 1T tokens | MT-Bench 2.33 / 3.28; AlpacaEval against davinci-001: 24% / 47%. Distillation from Llama-2-7B: no gain, 2.6–3.2× slower |
| SmolLM2 (2502.02737 + model cards) | 135M (2T) / 360M (4T) | IFEval 29.9 / 41.0; 360M MT-Bench 3.66 (Qwen2.5-0.5B-Instruct: 4.16) |
| Qwen2.5-0.5B-Instruct (2412.15115) | 18T tokens | IFEval-strict 27.9, GSM8K 49.6 |
| LFM2-350M (2511.23404) | 10 short-convolution + 6 GQA layers; 11T tokens with Top-K logit distillation | IFEval 65.1, MMLU 43.4, 194 tok/s on a phone CPU. **A few attention layers plus cheap local mixers is enough** |
| Gemma 3 270M (Google blog) | 170M embedding + 100M transformer | Targeted at task fine-tuning |
| BitNet b1.58 2B4T (2504.12285) | ternary 2B, 4T tokens | MT-Bench 5.85, IFEval 53.5; 0.4 GB non-embedding |
| MatMul-free LM (2406.02528) | 370M / 15B tokens | Average 40.3 vs 41.1 for Transformer++. **Ternary Q/K attention failed to converge** |
| Mamba-2 (2405.21060) | 350M, 48 layers | Perplexity 8.60 pure, **8.26 with 6 attention layers**, 8.68 for Transformer++ |
| Gated DeltaNet (2412.06464) | 1.3B / 100B | WikiText perplexity 16.42 vs 18.53; recall 30.6 vs 37.0; hybrid 40.1 |
| RWKV-7 (2503.14456); xLSTM (2405.04517) | 0.19–2.9B; 125M–1.3B | RWKV-7 tracks S5 with a constant state; xLSTM is the best non-transformer on MQAR recall |
| Mamba-in-the-Llama (2408.15237) | Llama-3-8B-Instruct, 20B tokens | MT-Bench by attention share: 50% 7.35, 25% 6.86, 0% 5.64 (teacher 8.0). Without attention-weight initialization: 1.04 |
| MOHAWK (2408.10189) | Phi-1.5 | 3B tokens, no attention: 62.6 vs 64.9. Keeping 4 of 24 attention layers, 5B tokens: 66.0 vs 67.2. MLPs transfer frozen |
| LoLCATs (2410.10254) | Llama 3 8B, 40M tokens, LoRA | MMLU 52.8 vs 66.6. Without the window: 23.8. Keeping 50% softmax layers: 65.8 |

**Smallest usable conversational size.** About 15–30M for one narrow synthetic domain. About 100–350M for open small talk, only with trillion-token data or distillation. About 1–2B for a reliable assistant: MT-Bench is 5.4–6.6 for 1–2B instruct models (BitNet report).

### F2. The GCM at three sizes (Derived; quality is Hypothesis anchored on F1)

| | GCM-S (native) | GCM-M (native, distilled) | GCM-L (converted SmolLM2-135M-Instruct) |
|---|---|---|---|
| Parameters | 15.8M (d=320, 12 layers, V=4096) | 59.9M (d=512, 18 layers, V=8192) | 134.5M + 1.6M new (d=576, 30 layers, V=49,152) |
| Time-mixing | GeoLRU in 9 layers: ¾ diagonal-decay lanes, ¼ quaternion lanes with per-lane radial decay, short convolution k=4, optional 2I tracking lanes | GeoLRU in 14 layers | RoPE kept as an isometry. 22 layers carry a RoPE-transported recurrent state (F4) |
| Reads | 3 κ-Lorentz layers; 5 heads, 1 KV head; NoRead slot | 4 layers; 8 heads, 2 KV heads | 8 exact + 22 windowed (64-token) κ-Lorentz layers; 9 heads, 3 KV heads |
| Channel mixing | ReLU² MLP, 4× width, ternary (the square is one table read) | same | Teacher SwiGLU at 4 bits (ternary later) |
| Weight bytes per token | 3.6 MB + 0.66 MB head | 13.9 + 2.1 MB | 53.1 MB (26.5 if ternary) + 14.2 MB head |
| State / KV at 2K | 8.6 KB + 0.98 MB | 21.5 KB + 2.1 MB | 8.1 MB; 2.65 MB read per token (teacher bf16: 47.2 MB) |
| Work per token | about 10M lookup ops + 0.6M table multiplies | about 36M + 1.35M | about 110M + 7.4M |
| DRAM proxy (80 mJ/GB) | cache-resident | about 1.4 mJ | 5.6 mJ at 4 bits, 3.5 ternary (teacher 25.3 mJ) |
| Training | 1.7B tokens/week on M1 at an assumed 0.3 TFLOP/s | 0.46B tokens/week | Conversion (F6) |
| Expected quality | TinyStories-class narrow chat | Narrow chat; about DialoGPT-117M at M1 budgets | Teacher minus about 2–10% (conversion literature); IFEval ≈ 27–30 |

GeoLRU is h_t = r_t·(q_t⊗h_{t−1}) + (1−r_t)c_t, with q = 1 on diagonal lanes. It is the D8 quaternion transport made *linear*, so it trains with an associative scan. **Measured:** the scan equals the sequential recurrence to a relative error of 1.1e-7 (diagonal lanes) and 2.0e-7 (quaternion lanes) (`gcm_proto.py check`).

### F3. Curvature-homotopy conversion (Derived; Measured on synthetic heads)

**Construction.**
- **Lift and distance:** x ↦ (x₀, x) with x₀ = √(1/κ+|x|²), and d_κ = arcosh(κz)/√κ, where z = x₀y₀ − ⟨x,y⟩.
- **Stable form:** κz−1 = (κ/2)(|x−y|² − ((|x|²−|y|²)/(x₀+y₀))²).
- **Score:** s = [|k|² − d_κ(q,k)²]/(2√r). As κ→0, d_κ → |q−k|, so s → [⟨q,k⟩ − |q|²/2]/√r, whose softmax is the teacher's.
- **RoPE:** rotations fix x₀ and the Lorentz form, so d_κ(R_m q, R_n k) = d_κ(q, R_{n−m}k).
- **Intrinsic variant:** replace |k|² with d_κ(o,k)², the key's hyperbolic radius. This is the owner's "keep the radius" idea in its geometric role.

**Measured** (`kappa_homotopy.py`, `kappa_intrinsic.py`; three synthetic trained-like heads). KL to the dot-product head, in nats, for the diffuse / sharp / sink heads:

| κ | 0 | 1e-4 | 1e-2 | 0.1 | 1 |
|---|---|---|---|---|---|
| Euclidean-bias score | ~1e-17 | 2.3e-6 / 2.5e-6 / 0 | 0.014 / 0.005 / 0 | 0.22 / 0.05 / 0 | 0.50 / 0.38 / 0 |
| Intrinsic-radius score | ~1e-18 | 2.9e-6 / 1.2e-5 / 0 | 0.015 / 0.027 / 0 | 0.16 / 1.35 / 0.01 | 0.34 / 4.5 / 3.8 |
| Cycle-3 form (κ=1, best β) | — | — | — | — | 0.168 / 0.0068 / 0 |

Other measured results:
- The float32 stable form holds at κ=1e-6 (KL ≤ 8.7e-6).
- RoPE invariance error: 5.0e-16.

**Proposal.** Initialize every head at κ₀=1e-4 with the Euclidean-bias score, which departs from the teacher more gently. Initialize the NoRead logit at −30 so the start is exact. Learn log κ, β and NoRead during attention transfer, and test the intrinsic variant in E1.
- **Gain:** a hyperbolic model that begins as an exact copy of the teacher; curvature grows only where the loss rewards it.
- **Cost:** about 3 extra table reads per candidate.
- **Falsifier:** if fewer than 10% of heads reach κ > 1e-2 with a gain of at least 0.01 nats, the teacher's heads do not want curvature. Hyperbolic geometry then moves to keys, admission and memory (cycle 2).

### F4. KV cache: full Lorentz, recurrent, or hybrid (Derived + Literature)

| Option | Size at 2K | Read per token | Evidence |
|---|---|---|---|
| Full κ-Lorentz KV, all 30 layers (int8 + 4 B scalars per key) | 24.3 MB | 3.8 MB with admission | — |
| Pure recurrent state | about 1.5 MB, constant | — | MT-Bench 5.64 vs 7.35; Zephyr perplexity ratio 1.66 vs 1.03; LoLCATs without window MMLU 23.8 |
| **Hybrid (recommended)** | **8.1 MB** | **2.65 MB** with 320 admitted candidates | See below |

The hybrid:
- **8 exact layers**, spaced out (Mamba-2: position then matters little).
- **22 layers with a 64-token κ-Lorentz window** plus a 128-feature state per KV head, S_t = γR_θS_{t−1} + φ(k_t)v_tᵀ. Here R_θ is the teacher's RoPE rotation, φ is a feature map on the keys, v_t is the value and γ a decay.
  - With identity features this is exactly RoPE-relative linear attention (the frame argument from wave 1).
  - This is where R4/quaternion transport enters: paired RoPE planes can become learned 4-D lanes (Hypothesis).
  - An alternative feature map to test: Hypformer's Lorentz linear attention (2407.01290), which has no causal-LM evidence.
- **The share of exact layers is a dial.** The literature gives about parity at 50% (Zephyr 7.31 vs 7.34), −0.3 to −1.1 MT-Bench at 25%, and −1.2 average points at 17% (MOHAWK).

### F5. Activation-by-activation products without a multiplier (Measured + Derived)

- **q·k.** Store keys as int8. For each query, build 64 tables of 256 multiples each, by addition. Each candidate then costs 64 table reads and 64 adds.
  - |q−k|² comes from the same sums together with the stored |k|².
  - The correction term (…/(x₀+y₀))² is a per-query 1-D table over the quantized |k|².
  - arcosh² comes from one table read; |k|²/2 is stored per key at write time.
  - *Measured:* int8 per-tensor q/k gives KL ≤ 3.3e-4; 4-bit per-tensor gives 0.017 (diffuse) and 0.12 (sharp). 4-bit therefore needs per-group scales or the radius+direction code from cycle 2 (untested for scoring).
- **Value mixing, SwiGLU, gates, RoPE and norms** use quarter-square tables: ab = ⌊(a+b)²/4⌋−⌊(a−b)²/4⌋, 2 reads and 3 adds per product. Exp, rsqrt and arcosh² use sealed tables.
- **Count for GCM-L:** 7.4M table multiplies per token, against about 110M weight-lookup operations.

### F6. Offline cost, CPU vs M1 (Derived; one Measured rate)

**Attention transfer** (all layers trained from one teacher pass): 0.37 GFLOP/token.
- 20M tokens = 7.4 PFLOP.
- **M1** at an assumed 0.3 TFLOP/s (not yet measured in Candle-Metal): 6.8 h, about **14 min per layer**.
- **Here** at the measured 29 GFLOP/s on one thread: 71 h, and the teacher is unreachable.

**Distillation (Stage 2):** 1.3 GFLOP/token.

| Run | M1 (assumed 0.3 TFLOP/s) | 1 H100 (assumed 4e14 FLOP/s) |
|---|---|---|
| LoRA, 20M tokens | about 24 h | — |
| **Pilot = attention transfer + LoRA** | **≈ 31 h** | — |
| Full 4-bit QAT, 0.2B tokens | 10 days | 11 min |
| Full 4-bit QAT, 1B tokens | 50 days | 54 min |

Compute the teacher online: caching top-32 logits takes about 26 GB per 0.2B tokens.

### F7. Sparse geometric parameter memory (lead's addition)

**Literature:**
- **PKM (1907.05242):** 12 layers plus a memory beat 24 layers at about half the time.
- **Memory+ (2412.09764):** a 134M base with a 1M-row memory reached NQ 3.16 / TQA 18.8, against 0.91 / 7.7 for dense 134M and 2.58 / 17.7 for dense 373M. That is **at least 2.8× the dense parameters' worth on factual QA**. More than 3 memory layers hurts.
- **Engram (2601.07372):** deterministic hashed 2–3-gram addresses. It beats an MoE at matched parameters and FLOPs (MMLU +3.4, BBH +5.0). A 100B-parameter table in host DRAM cost under 3% overhead. Ablating it at inference keeps 29–44% of factual performance but 81–93% of reading comprehension.
- **L³ (2601.21461):** static token-ID routing. Perplexity went from 22.0 to 20.2 at 800M active parameters, with about 1 MB transferred per token at 2.6B.
- **TF-Engram (2607.07388):** SSD-backed, with prefetch: 460 vs 481 tok/s and 2.17 vs 2.08 ms/token.

**Assessment:**
- **D5 and gating.** Token-n-gram addressing gives per-token parameter sparsity without a learned router, so it fits D5 and the no-gating rule. PKM/Memory+ select top-k with a learned query, which is learned content-dependent selection. Geometric cells (E8/600-cell/radius shells) turn PKM into a fixed codebook. That is novel, but it depends on the hidden state, so it cannot be prefetched.
- **Bytes.** Two layers × 8 rows × 64 dims at 4 bits = 512 B per token, which is negligible. On an SSD it becomes 16 random 4 KB pages per token.
- **Where the table lives.** Up to about 2–4B parameters at 4 bits it fits in M1 DRAM; the SSD is needed only beyond that, with prefetch.
- **Proposal.** Add 1–2 early Engram layers to GCM-L during Stage 2. **Falsifier:** E5.

### F8. From the D8 learner to the GCM (Source + Measured)

**What must change:**
1. **`JointConfig::validate`** hard-codes vocabulary 4096, width 128/256, read width 64 and context ≤256. Freeze it for D8 and add a `GcmConfig` covering layer kinds, heads, KV heads, MLP kind, RoPE base, κ₀ and lane split.
2. **A new sequence-parallel `gcm_model.rs`.** `core_step` unrolls each position in about 120 small operations.
   - **Measured:** the parallel prototype runs at 311 tok/s at 15.65M parameters, about 29 GFLOP/s-equivalent on one thread under load 12 (`gcm_proto.py speed 320 12 8 256 3,6,9`). Cycle-3 D8: about 2.7 GFLOP/s-equivalent.
   - The comparison crosses frameworks (JAX vs Candle).
3. **The κ-score** in `lorentz_distance`/`read_scores`.
4. **A Llama safetensors importer**, plus the teacher's 49K BPE.
5. **Attention-transfer and KL-distillation modes**, with LoRA or full updates and QAT.
6. **The integer runtime:** NEON `tbl` lookup kernels, per-query tables, quarter-square tables, arcosh²/exp/rsqrt tables, and streaming KV. Retire software-emulated multiplication, which is 4.4× slower.
7. **A chat evaluation:** masked assistant-turn NLL, IFEval-style rule checks, multi-turn recall of user-stated facts, and J/token measured with macmon.

**What can be reused:**
- `report_output`, `NamedAdamW`, `batch_gradients`.
- `joint_quantization` (4-bit with straight-through estimation) and `joint_rounding`.
- `joint_admission`, with learned advertisements replacing its fixed sign tables.
- `read_dropout`, the NoRead slot, the pointer-copy mixture.
- The `joint-read-geometry` example as the template (checkpoint, resume, sealed reports).

### F9. Chat path and tokenizer

- **(b) Conversion: primary.**
  - It inherits the teacher's 2T tokens of knowledge, which sits in the MLPs (Engram's ablation; MOHAWK's frozen MLPs).
  - It is the only M1-feasible route to the 135–360M band.
  - Mamba-in-the-Llama showed that attention initialization is decisive, and the κ-homotopy makes it exact.
- **(a) Distillation** is Stage 2's training signal. It is exact logit KD, because the student keeps the teacher's tokenizer. For a smaller-vocabulary student, ALM (2503.20083) achieves cross-tokenizer distillation: 55.1 vs 51.6 for plain fine-tuning, against a teacher at 58.0.
- **(c) Synthetic dialogues** serve GCM-S/M and domain adaptation, but alone they cap at narrow chat.
- **Tokenizer.** GCM-L uses the 49K BPE; GCM-S keeps 4096; GCM-M uses 8–16K (N_v ∝ N_nv^0.83, 2407.13623).

## 3. Build plan and next experiments (ranked)

**M1 milestone. Exact, learnable κ-conversion (owner's M1, days).**
- Scope: F3 plus the importer.
- Tests:
  - (i) At κ=0 the converted SmolLM2-135M-Instruct logits match the teacher's to a relative error ≤1e-4.
  - (ii) After a 20M-token attention transfer (about 7 h), chat NLL is within 0.01 nats of the teacher, and the per-head κ histogram is reported. The F3 falsifier then decides whether scoring stays hyperbolic.

**M2 milestone. Hybrid KV (M1, ≈31 h).**
- Scope: 22 layers converted to window + RoPE-transported state, via LoLCATs Stage 1 plus 20M tokens of LoRA distillation.
- Test: at most a 5% relative IFEval-strict drop, chat NLL within 0.1 nats, KV ≤ 8.1 MB at 2K.
- Fallback: if the drop exceeds 10%, use 12–15 exact layers.

**M3 milestone. Multiplier-free integer GCM-L (M1, 5–10 days of 4-bit QAT).**
- Tests:
  - the integer session is bit-exact with the training hard path;
  - disassembly shows no multiply instruction in the declared kernel;
  - NLL within 0.05 nats of the float student;
  - lower measured J/token than llama.cpp running the teacher at Q4_0, at 2K context.

**Experiments runnable here (≤15 min, 1 core):**
- **E1.** Train a 2-layer d=128 attention LM on WikiText for 5 min, then continue for 5 min with learnable κ (both score variants) or as Dot, with paired seeds. Question: does κ grow, and does NLL improve?
- **E2.** Measure the KL of 4-bit per-group and radius+direction keys on E1's heads. Target: ≤0.01.
- **E5.** Add a hashed 2/3-gram table (2¹⁸×64) to the GCM-S prototype and compare with a dense control at equal active FLOPs.
- **E6** (M1). Measure the Candle-Metal throughput of the parallel GCM-S, to replace the assumed 0.3 TFLOP/s.

## 4. Open questions for the owner

1. **Paid compute.** Full distillation is 10–50 M1-days or about 1 H100-hour (rental cost not researched). Approve a small run?
2. **Teacher after the pilot.** SmolLM2-360M-Instruct (MT-Bench 3.66, about 2.7× the cost), or Qwen2.5-0.5B-Instruct, whose 151K vocabulary makes the head dominate the bytes read?
3. **"No transformer backbone."** Does a converted stack with κ-Lorentz reads and recurrent state satisfy the rule, or should it be restated as "no dot-product attention and no multiplier at serving"?
4. **Egress.** Allow huggingface.co for the lab, or keep real-teacher work on the M1?
5. **Precision.** 4-bit first, with ternary later (it needs far more QAT tokens)?
