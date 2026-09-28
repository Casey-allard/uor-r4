# train2 — Training and data strategy: the fastest credible route to a geometric model we can chat with

2026-09-26 · Agent `train2` · Scratch: `$S/lab/exp/train2/` (benchmarks, logs, configs; 180 KB) · Repo untouched.
Labels: **Measured** (run here; logs in `exp/train2/*.log`), **Literature** (retrieved this session), **Repo record** (a repo document read this session), **Derived**, **Hypothesis**.

Re-centred after the lead's 16:40 message. The owner approved weight transfer, hyperbolic attention as the lead mechanism, and offline training as a first-class concern. The plan below therefore converts SmolLM2-135M-Instruct first and SmolLM2-360M-Instruct second.

## 1. Verdict

1. **The fastest credible route is conversion, not from-scratch training.** Take SmolLM2-135M-Instruct, which is Apache-2.0 and which the repo's own Llama adapter accepts (Measured).
   - Swap each softmax attention for a **Lorentz-score softmax attention**. Keep the KV cache and add a per-head sink/NoRead logit.
   - Train it by per-layer attention transfer, then logit distillation (KD) on chat data, with ≤4-bit weights.
   - Published conversions changed the attention for **20–100M tokens of attention transfer plus 250–700M tokens of KD** (Literature: LoLCATs, RADLADS). The fully recurrent MOHAWK conversion needed 3B tokens.
   - Total cost is about 3×10¹⁷ FLOPs, the same as training a 15M model from scratch. The result inherits 2T tokens of pretraining plus SFT/DPO instead of TinyStories-level fluency (Derived).
2. **From scratch, M1-class compute buys narrow, child-level fluency at 5–15M parameters, not chat.** A 60M model needs 42–417 M1-days and a 150M model needs years (Derived, §F4).
3. **Ternary MLPs are not feasible on the M1.** QAT from a pretrained model saturates at about 30B tokens for ternary or 2-bit, and about 10B for 3–4-bit (Literature: ParetoQ). **Go 4-bit first**, which D0-b permits via lookup-table kernels. Leave ternary for when external compute is authorized.
4. **(a) This sandbox cannot download the teacher or any Hugging Face data (Measured).**
   - Every Hugging Face host answers CONNECT 403 at the egress proxy. No local copy exists.
   - GitHub raw, PyPI and crates.io work. From GitHub raw we can fetch Taskmaster-1 (CC BY 4.0), MultiWOZ (MIT), SGD (CC BY-SA 4.0) and the 541 IFEval prompts.
5. **(b) Measured cost here, one core, Candle, SmolLM2-135M shape:**

   | Step | Tokens/s | 24 core-hours buys | Share of the literature budget |
   |---|---:|---:|---|
   | Teacher forward | 129 | — | — |
   | Per-layer Lorentz attention transfer | 46.8 | ≈4.0M tokens | ≈20% of LoLCATs' transfer stage |
   | Full student forward and backward | 32.5 | ≈2.2M KD tokens | <1% of RADLADS' KD |

   So the sandbox is for **attention transfer and evaluation**. KD and 4-bit QAT belong on the M1.
6. **The project's sequential D8 learner uses about 5% of a core's matmul throughput (Measured).**
   - It runs at about 2–3 GFLOP/s. Candle matmul on the same core runs at 42–78 GFLOP/s.
   - A time-parallel model of similar size trains 4–6× more tokens per second (2,885 against 483–683).
   - Any from-scratch geometric backbone must therefore be trainable with a parallel scan or in parallel over time.
7. **(c) The smallest convincing demo here (about 24 core-hours, once Hugging Face is reachable):** SmolLM2-135M-Instruct with all 30 attentions replaced by Lorentz attention and the MLPs untouched.
   - Evaluate teacher-relative fidelity, IFEval, multi-turn memory probes and a blind side-by-side.
   - If Hugging Face stays blocked, fall back to a Taskmaster-1 order-taking chat model trained from scratch.
8. **M1 plan: about 2–5 weeks of M1 time for 135M conversion, KD and 4-bit QAT (Hypothesis).** This depends on Candle reaching 0.1–0.3 TFLOP/s on the M1, which nobody has measured. Measure it first; the benchmark exists. 360M is second, at about 2× the cost, with IFEval 41.0 against 29.9.

## 2. Findings and proposals

### F1. Network access (Measured, 16:30–16:52 UTC, curl through the session proxy)

| Result | Hosts |
|---|---|
| **Proxy policy denial (CONNECT 403)** | huggingface.co, hf.co, datasets-server.huggingface.co, cdn-lfs.huggingface.co, cdn-lfs-us-1.hf.co, cas-bridge.xethub.hf.co, arxiv.org, export.arxiv.org, kaggle.com, zenodo.org, osf.io, gutenberg.org, dl.fbaipublicfiles.com, zissou.infosci.cornell.edu, modelscope.cn |
| Session GitHub gate (403, "use add_repo") | github.com, codeload.github.com, api.github.com |
| **Works** | raw.githubusercontent.com (35 KB in 0.41 s; a 65.7 MB Taskmaster file downloaded), pypi.org (3.67 MB JSON in 0.11 s), files.pythonhosted.org, index.crates.io, static.crates.io, crates.io API (with a User-Agent), storage.googleapis.com, s3.amazonaws.com |

- **No SmolLM2 weights or tokenizer exist on this disk.** I searched every `*.safetensors`, `tokenizer.json` and `*.gguf`.
- The teacher file is 269,060,552 bytes (Repo record: `models/smollm2-135m-instruct.json`). That exceeds my 200 MB cap, so the lead must approve it; it uses 0.27 GB of the 26 GB free.
- I did not try mirrors. The proxy's rules forbid routing around a policy denial.
- **Remedy:** the owner opens Network access for this environment and adds huggingface.co with its download hosts (or picks a broader level): cloud environment menu in the session title bar → Edit. Otherwise Stage 1 runs on the Mac.
- A Hugging Face MCP connector appeared mid-session, but its tools were not exposed to this agent. It would not move 270 MB into the sandbox in any case.

### F2. Data (Literature/URL via Hub API and cards unless marked)

| Dataset | Size | License | Generated by | Role |
|---|---|---|---|---|
| **smol-smoltalk** | 460,341 conversations (1.81 GB Arrow) | Apache-2.0 | Llama-3.1-405B (Magpie-Ultra), Qwen2.5-72B and others | The exact SFT set of SmolLM2-135M/360M-Instruct → **primary conversion and KD data** |
| **everyday-conversations-llama3.1-2k** | 2,260 train + 119 test; 2.17 MB | Apache-2.0 | Llama-3.1-70B | Greetings, "who are you", simple science; **test split for evaluation** |
| SmolTalk (full) | 1.1M | no license tag in card metadata | mixed | Not needed |
| TinyStories | 2,141,709 rows; 7.62 GB repo | CDLA-Sharing-1.0 | GPT-3.5/4 | From-scratch fluency; share-alike applies to data |
| TinyStoriesInstruct | 21,974,061 rows; 2.69 GB | no dataset card, so license unstated | GPT-3.5/4 | Instruction → story |
| TinyDialogues | ≈130K conversations, ≈29M words | MIT | GPT-4 | Child-directed dialogue (ages 2–15). Synthetic beat natural CHILDES for small LMs (2408.03617). Hosted on Hugging Face only |
| SODA | 1.19M / 146K / 149K dialogues | CC BY 4.0 | InstructGPT | Social chit-chat |
| UltraChat-200k | filtered from 1.4M | MIT | ChatGPT | General multi-turn |
| OpenHermes-2.5 | ≈1M | no license tag | GPT-4 mixture | Not recommended |
| Magpie-Pro-300K-Filtered | 300K; 563 MB | Llama 3 license | Llama-3-70B | Not recommended |
| **Taskmaster-1 (reachable here)** | 13,215 dialogs. Measured on the written set: 7,708 dialogs, 169,469 utterances, 1.45M words (≈1.85M tokens), 6 domains of 200–270K words, ≈20K slot-annotated segments per domain | CC BY 4.0 | Humans | Narrow task chat with API-argument labels, so slot fidelity can be scored automatically |
| MultiWOZ, SGD, IFEval prompts (541, 207 KB) | reachable via GitHub raw | MIT / CC BY-SA 4.0 / — | Humans | Fallback data and evaluation |

**Which data lets a 5–50M model hold simple conversations?**
- From scratch, the enabling factor is restricted vocabulary and register (Literature, TinyStories 2305.07759):
  - 1–33M models became fluent in ≤30 V100-hours each.
  - Consistency emerged at width ≥128.
  - Instruction following needed ≥2 layers.
- A plausible from-scratch chat mix: TinyDialogues, everyday-conversations, SODA filtered to simple vocabulary, and TinyStories-style assistant chats. **Hypothesis:** 5–15M parameters on 0.3–3B tokens gives child-level chit-chat. I found no evidence of a 5–50M model holding open-domain chat.
- For conversion, use smol-smoltalk plus everyday-conversations, the teacher's own SFT distribution.
- **Licensing:** avoid OpenAI-generated sets (UltraChat, SODA, OpenHermes) and Llama-licensed ones (Magpie) unless the owner accepts their terms.

### F3. Teachers, and what the repo's Rust can run

I ran the repo's own `AdapterFeatures::{huggingface_llama, huggingface_gpt2}().validate_config` on the retrieved `config.json` files (**Measured**; binary `cbench/src/bin/adapter.rs`, log `adapter_check.log`).

| Teacher | License | Shape | Repo adapter | Notes |
|---|---|---|---|---|
| **SmolLM2-135M-Instruct** | Apache-2.0 | Llama; d576, 30 layers, 9 query / 3 KV heads, feed-forward width 1536, vocabulary 49,152 tied, eps 1e-5, RoPE θ 1e5, no RoPE scaling | **ACCEPTED** | IFEval 29.9. Byte-level BPE with Digits+ByteLevel, exactly the shape `uor-r4-tokenizer` requires. Already run by the repo: 114–128 tokens/s of CPU teacher generation in its compile pipeline (Repo record, `docs/smollm2_teacher_baseline_320.md`) |
| **SmolLM2-360M-Instruct** | Apache-2.0 | d960, 32 layers, 15/5 heads, feed-forward width 2560 | **ACCEPTED** | IFEval 41.0, MT-Bench 3.66. 74.5 tokens/s on macOS Accelerate (Repo record) |
| TinyLlama-1.1B-Chat-v1.0 | Apache-2.0 | Llama; vocabulary 32,000 untied | ACCEPTED (config only) | Its SentencePiece-style tokenizer lacks the `ByteLevel` pre-tokenizer the repo parser requires (Derived from code, untested). 8× the cost of 135M |
| Qwen2.5-0.5B-Instruct | Apache-2.0 | qwen2; vocabulary 151,936 | **REJECTED**: `model_type`; relabelled, then `rms_norm_eps` 1e-6 | Stronger GSM8K (26.8) and MT-Bench (4.16), but the head alone is 136M parameters (Derived) |
| Llama-3.2-1B-Instruct, gemma-3-270m-it | Llama 3.2 license (gated); Gemma terms | `gemma3_text` | Not supported | — |

- The repo's teacher oracle is an inference executor. It supports batched decode (`BatchedTeacher::forward_batch_into`), so it can supply teacher logits and serve as a **parity reference**.
- Training the student needs a Candle implementation of Llama. `cbench/src/bin/smol.rs` is that skeleton; it still lacks RoPE and weight loading.

**Which kind of distillation (Literature)?**
- **Logit KD** is the natural loss for conversion; MOHAWK stage 3 and RADLADS step 2 both use it.
- Distillation beats supervised training when a teacher already exists and the student's token budget is small (Distillation Scaling Laws, 2502.08606). Both hold here.
- **Teacher logits can be cached.** Keeping only the top-k logits is biased. Sampling about 12 tokens per position matched full KD (2503.16870).
  - Storage: about 36 B/token, so about 9 GB for 250M tokens on the Mac.
  - This drops the teacher forward, which is 20% of a KD step here (7.8 of 38.5 ms).
- **Sequence-level KD** from the 135M teacher's own generations is weak (IFEval 29.9). Prefer text generated by 70–405B models (smol-smoltalk), scored with the 135M/360M teacher's logits.
- **For tiny from-scratch students, logit KD from SmolLM2 is expensive.** It forces a 49,152-way head: 12.6M parameters at d=256 (Derived). Use sequence-level data instead.

### F4. Compute

**Measured here** (one thread, `nice 10`, Candle 0.9.2 f32; CPU-time basis because other agents held the load average at 6–23; `cbench`, logs `mm_bench.log`, `lm_bench.log`, `smol135_bench.log`):

| Workload | Tokens/s (CPU time) | Achieved |
|---|---:|---:|
| Matmul 16×256×768 (one recurrent step's shape) / 4096×256×768 / 4096×1024×4096 | — | 57.7 / 78.4 / 65.7 GFLOP/s |
| D8 learner, 0.69M parameters, width 128, T=128, batch 16 (finished run `c3/runs/full/code_dot_s1`: 4.096M visits in 8,486 s) | 483 (brief: ≈683) | ≈2–3 GFLOP/s |
| Time-parallel causal LM, forward+backward+SGD, V=4096: 0.93M / 4.3M / 12.3M parameters | 2,885 / 818 / 341 | 17.0 / 23.2 / 27.3 GFLOP/s |
| SmolLM2-135M shape, T=256: teacher forward with 49K head | 129 | ≈37 GFLOP/s |
| — attention transfer: teacher forward + 30 Lorentz-student layers forward/backward | 46.8 | — |
| — full student forward+backward+SGD, Lorentz attention, batch 2×256 | 32.5 | ≈28 GFLOP/s |
| SmolLM2-360M shape: teacher forward | 66.1 | — |

- A 4×256 student batch was **OOM-killed at 8.9 GB RSS**.
- The shell's memory cgroup is **14.35 GB, shared by every agent's processes**. Conversion runs here need micro-batches of 2×256 or gradient checkpointing.

**M1 anchors (Repo record).** The first-principles review measured:
- the #1014 transformer on PyTorch MPS at ≈0.32 TFLOP/s (12.5% of peak);
- the D8 learner at 1,362 tokens/s per arm on the M1 CPU;
- the D8 learner slower on Candle-Metal than on CPU.

Candle's rate for a matmul-heavy student on the M1 is unmeasured. I plan with 0.1–0.3 TFLOP/s (**Hypothesis**).

**Budgets (Derived: C = 6ND, the Chinchilla approximation; ≈20 tokens/parameter at compute-optimum, 2203.15556).**
- Token anchors (Literature and Repo record):
  - the #1017 transformer: 6M non-embedding parameters on 150M tokens → 1.57 nats;
  - llama2.c: stories15M trained "in a few hours" on 4×A100; the 110M model ≈26B tokens (131K tokens/update × 200K iterations).
- Sandbox rate: 25 GFLOP/s per core. A100 at 125 TFLOP/s effective is a reference only; it is not authorized.

| Model | Tokens for useful quality (Hyp.) | FLOPs | Sandbox, one core | M1 at 0.1–0.3 TFLOP/s | One A100 |
|---|---|---:|---:|---:|---:|
| 5M from scratch | 0.3–1B | 0.9–3×10¹⁶ | 4–14 days (24 core-h ≈ 70M tokens) | 8–83 h | 1–4 min |
| 15M from scratch | 1–3B | 0.9–2.7×10¹⁷ | 42–125 days | 3.5–31 days | 12–36 min |
| 60M from scratch | 3–10B | 1.1–3.6×10¹⁸ | years | 42–417 days | 2.4–8 h |
| 150M from scratch | 10–30B (SmolLM2-135M used 2T) | 0.9–2.7×10¹⁹ | — | 1–8.5 years | 20–60 h |
| **SmolLM2-135M conversion** (20–100M transfer + 250M KD) | — | ≈3×10¹⁷ | 2,700+ core-hours | **11–37 days** | ≈45 min |

### F5. Conversion budgets and what fits where (Literature → Derived)

| Method | Tokens | Result |
|---|---|---|
| LoLCATs (2410.10254): linear + sliding-window attention; output-MSE attention transfer, then LoRA; T=1024 | ≈20M transfer + ≈20M LoRA | Llama-3.2-1B: average without MMLU 59.6 vs 60.0; MMLU 27.3 vs 31.9. Keeping half the layers softmax, transfer only (20M): MMLU 65.8 vs 66.6 (8B) |
| MOHAWK (2408.10189): Phi-1.5 → Mamba-2 | 80M + 160M + 2.76B | Average 62.6 vs teacher 64.9. Stage 3 can freeze the MLPs |
| RADLADS (2505.03005): Qwen2.5 → RWKV variants | 100M alignment + 250–700M KD + 100M context | 92–104% relative scores. The 7B conversion took 7.25 h on 8×MI300X. **Also converted Qwen3-8B to Softpick attention (a changed softmax score, KV cache kept) "rivaling or exceeding" the teacher** |
| ParetoQ (2502.02631): QAT from pretrained models, 125M–8B | ≈10B saturates 3–4-bit; ≈30B ternary/2-bit | 4-bit is near-lossless. At 125M, 3-bit kept average accuracy within 0.6 but WikiText-2 perplexity rose from 14.9 to 21.6 |
| BitDistill (2510.13998): Qwen3 0.6–4B → ternary, task-specific | 10B continued pretraining + logit/attention KD | Direct ternary fine-tuning: MNLI 74.1 vs 88.0 (FP16). With their KD: 88.0–88.2. Task results only, not general chat |
| HELM (2505.24722): fully hyperbolic LMs from scratch, 115M/1B, 5B tokens | — | ≈1.5× training time. All models score near chance, so its +1–4-point "gains" are weak evidence |

**Derived: why a Lorentz-attention conversion should sit in the cheap LoLCATs/Softpick regime.**
1. Only the score function changes.
2. RoPE applied before the Lorentz lift is a rotation, so it preserves ‖q‖ and ‖k‖. Hence q₀ and k₀ are unchanged and z = q₀k₀ − ⟨R_i q, R_j k⟩ keeps its relative-position structure; HELM's HoPE makes the same argument.
3. Near the operating point, −arcosh z ≈ const + ⟨q,k⟩/sinh δ₀ minus a per-key norm term. The committed cycle-3 initialisation (β₀, δ₀) makes the score a first-order copy of the dot product; the two correlated at 0.81 at initialisation in cycle 3.
4. Initialising W_q and W_k from the teacher therefore starts the student close to it.

A constant-state recurrent student would sit in the MOHAWK regime instead (billions of tokens), with weaker in-context recall.

| Stage | Literature tokens | Sandbox, 24 core-h | Fraction | M1 at 0.1–0.3 TFLOP/s (Hyp.) |
|---|---|---:|---:|---|
| Per-layer attention transfer (≈0.33 GFLOP/token) | 20M (LoLCATs) – 100M (RADLADS) | **4.0M** | 20% / 4% | 6–18 h for 20M; 1.3–3.8 days for 100M |
| End-to-end KD (≈1.15 GFLOP/token) | 250–700M | 2.2M | ≤0.9% | 11–33 days for 250M |
| 4-bit QAT to saturation | ≈10B | — | 0.02% | years → instead PTQ plus 100–300M tokens of KD-QAT (4–40 days) |
| Ternary QAT | ≈30B | — | 0.007% | not feasible |

- Cached sampled logits save ≈20% of a KD step. Freezing the MLPs early in KD, as MOHAWK did, saves roughly 25% more (Derived).

### F6. Proposals: a staged plan

**P1 — Stage 1, sandbox, needs Hugging Face: "Lorentz-attention SmolLM2-135M-Instruct" (≈24 core-hours).**
- **1a. Parity (0.5 core-h).**
  - Add RoPE and safetensors loading to `smol.rs`.
  - Check the Candle forward against the repo's `HuggingFaceLlamaOracle` logits on 20 chat prompts, with max |Δlogit| < 1e-3.
- **1b. Zero-training swap (1 core-h).** Replace one layer at a time, then all 30, using the cycle-3 initialisation, and record the change in NLL on held-out assistant tokens.
- **1c. Per-layer transfer (16 core-h: 4 processes × 4 h, ≈3–4M tokens).**
  - Data: everyday-conversations plus one smol-smoltalk parquet shard, in ChatML format, T=256–512.
  - Train W_q, W_k, β, δ and a sink logit per head; keep W_v and W_o frozen.
  - Measure the relative MSE curve per layer.
- **1d. Evaluation (≈5 core-h):** G1–G3 below plus 30 saved sample chats.
- *Expected gain:* the first chat-capable model whose attention is hyperbolic, with MLP knowledge inherited intact. It is not yet D0-b-serving: MLPs remain float.
- *Cost:* 0.27 GB download, <1 GB outputs.
- *Cheapest falsifier:* 1b plus a 50K-token single-layer transfer. **Refuted if** relative MSE stays above 10% after 1M tokens, or the all-layer swap after transfer is more than 0.5 nats worse than the teacher. Then try squared-Lorentz scores (HELM) or keep one softmax layer in four.

**P2 — Stage 2, M1, 1–3 weeks: 4-bit everywhere plus KD.**
- Start with group-wise 4-bit post-training quantization and measure the change in NLL.
- Then run LSQ-style QAT under KD for 100–300M tokens, using cached sampled teacher logits.
- Extend attention transfer to 20–50M tokens.
- *Falsifier:* if 4-bit PTQ costs more than 0.3 nats and 50M tokens of KD-QAT recover less than half of it, ask the owner about 8-bit attention or head weights.

**P3 — Stage 3, M1: integer serving.**
- T-MAC-style 4-bit table matvec, which is D0-b-legal.
- Lorentz scores from keys coded by cycle 2's radius+direction quantizer (36 bits/key); per-query codebook tables; exp and arcosh tables.
- Measure tokens/s, bytes/token and J/token against llama.cpp running SmolLM2-135M at Q4.
- **Derived:** 4-bit weights put about 67 MB behind every token, 21% of it the tied head.

**P4 — Stage 4:** 360M, at about 2× the cost (Measured teacher ratio 1.95×).

**P5 — Fallback if Hugging Face stays blocked: a Taskmaster-1 order-taker (sandbox, 24 core-hours).**
- Model: a ≈4M time-parallel model with a Lorentz read, about 70M tokens (≈35 epochs).
- *Falsifier:* slot fidelity on held-out dialogs must beat both a nearest-response retrieval baseline and a count/cache baseline.
- This demonstrates narrow real conversation only, not general chat.

**Evaluation that shows "we can chat with it".** The owner should freeze thresholds before any run; the values below are Hypotheses.
- **G1 fidelity.** On 119 everyday-conversations test chats and 300 held-out smol-smoltalk chats:
  - NLL on assistant tokens ≤ teacher + 0.15 nats;
  - mean KL(teacher‖student) ≤ 0.15;
  - greedy top-1 agreement ≥ 75%.
- **G2 IFEval** (541 programmatic prompts, fetchable here): ≥ 25. The teacher's reported 29.9 must be re-measured in our harness.
- **G3 memory.** 100 multi-turn probes where a fact stated earlier is asked 2–6 turns later, scored by exact or keyword match: ≥ teacher − 5 points.
- **G4 blind side-by-side.** 60 prompts; the owner rates student against the 4-bit teacher; wins plus ties ≥ 50%.
- **G5 cost on the M1.** No float or multiplier in the serving kernel; J/token and bytes/token reported.

## 3. Recommended next experiments (ranked)

1. **[M1, 10 min] Measure Candle on the M1 for the SmolLM2 shape.** Every M1 estimate above depends on it. This needs an edited copy of `cbench`:
   - copy `lab/exp/train2/cbench` (3 source files) and point its path dependencies at the Mac's checkout;
   - run `cargo build --release --features metal --bin smol`, then `TRAIN2_DEVICE=metal smol {teacher|xfer|student} 49152 576 30 9 3 1536 4 256 3`;
   - repeat with `--features accelerate` and no `TRAIN2_DEVICE`.
2. **[Owner action, then sandbox, 30 min] Allowlist Hugging Face and run the P1a parity check.**
3. **[Sandbox, 15 min] P1b zero-training Lorentz swap:** the change in NLL per layer.
4. **[Sandbox, 15 min] Single-layer transfer pilot, 50K tokens:** MSE against tokens for Lorentz-arcosh vs squared-Lorentz vs dot (control).
5. **[Sandbox, 15 min] 4-bit group-32 round-to-nearest PTQ of all linear maps:** the change in NLL on held-out chat. This decides whether PTQ suffices for the first demo.
6. **[Sandbox, only if Hugging Face stays blocked] P5 pilot:** tokenise Taskmaster-1 and run a 15-minute training check.

## 4. Open questions for the owner

1. **Allow huggingface.co and its download hosts for this environment, and approve the 0.27 GB teacher download**, which exceeds the 200 MB cap? Or should Stage 1 run on the Mac?
2. Is **4-bit (not ternary)** acceptable for the first chat model? Ternary needs about 30B QAT tokens, which means external compute.
3. Is **Lorentz softmax attention with a compressed KV cache** acceptable as the geometric attention? A constant-state recurrent student costs roughly 10–100× more conversion tokens and recalls less.
4. Which **data licenses** are acceptable? The Apache-2.0 SmolLM2 sets are clean. OpenAI-generated sets carry terms of use; Llama-licensed and CDLA-Sharing sets carry their own conditions.
5. One rented GPU-hour would cover the whole 135M conversion compute (Derived). Is that still not authorized?
