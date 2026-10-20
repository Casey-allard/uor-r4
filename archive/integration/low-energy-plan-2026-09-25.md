# The plan I recommend: a local language model that stops wasting energy on matmul

2026-09-25 · Requested by the owner · References #820

**Status.** This is a proposal, not a decision record. The owner asked for the best plan with one goal: stop wasting energy on matrix multiplication. The earlier constraints (transformerless, no MoE, no sparse routing, M1-only training) were set aside for this plan. Where the plan relaxes one of them, it says so. The evidence comes from the [first-principles review](first-principles-review-2026-09-25.md) (§ references below point to it) and from papers retrieved today (arXiv ids).

Labels: **Measured** (run by the review), **Literature** (retrieved paper), **Derived** (arithmetic shown here), **Hypothesis** (untested).

## 0. The answer

The energy of a matmul is not spent in the multiplier. It is spent **moving weights and cached context from memory, and executing instructions** (§5 of the review: multipliers are at most about 0.4% of the DRAM-bound floor). So the way to stop wasting energy on matmul is to:
1. **store every weight in 1.58 bits (ternary)** and execute the linear maps with additions and table lookups, not multiplications;
2. **stop re-reading a growing context cache**: use a mostly recurrent backbone with a few attention layers whose cache is compressed;
3. **read the weights once per several tokens** by drafting tokens from exact memory (your "store and recall") and verifying them in one pass;
4. **shrink the output head**, which in small models is the largest single read;
5. **train in the cloud or by distillation, and serve on the M1.** Training energy is paid once. Serving energy is paid every token, and that is what this plan minimises.

The linear maps cannot disappear: they are where a model stores its knowledge, at about 2 bits per parameter (Literature 2404.05405). They can become ternary, add-only, and read far less often.

Geometry keeps four real jobs in this design (§6): compressing the context cache and memory keys, lookup-table attention scores, exact state-tracking lanes, and addressing for a lookup memory. The backbone itself is not geometric. The review measured that rotations add nothing on natural text (review §6.3).

## 1. Where the energy of one token goes

For batch-1 generation, every weight and every cached context entry is read once per token. At the review's DRAM cost of about 10 pJ/bit (80 mJ per GB; review §5.1), take a 2B-class model with the published BitNet b1.58 2B4T shape as the example. That shape is 30 layers, width 2,560, 5 key/value heads of 128, and a tied 128,256-token vocabulary (Literature 2504.12285, published config).

| What is read per generated token | Bytes | Energy at 80 mJ/GB | Label |
|---|---:|---:|---|
| Ternary body, 2.08B weights at 1.58–2 bits | 0.41–0.52 GB | 33–42 mJ | Derived |
| Output head, 328M weights, at 16 / 4 bits | 0.66 / 0.16 GB | 53 / 13 mJ | Derived |
| Attention cache at 4k context, 16-bit (77 KB per context token) | 0.31 GB | 25 mJ | Derived |
| Same cache with ¼ of the attention layers kept and 3-bit entries | 0.015 GB | 1.2 mJ | Derived |
| All multiply–accumulates, as int8 arithmetic | — | about 0.4–2 mJ | Derived (Horowitz costs) |

What follows from the table:
- **The arithmetic is a small fraction of the energy.** Removing multipliers alone saves little; removing bytes saves a lot.
- **After ternary weights, the head and the context cache dominate.** At 4k context they add 0.9–2.4× the body's bytes.
- **Instructions and time matter too.** A core that runs longer burns static power. The review estimated the current integer session, which emulates multiplication in software, at 11–23 mJ per token for a 1.68M-parameter model. That is the DRAM floor of a well-built 125–300M-parameter model (review §5.2).

## 2. What already exists, so we do not rebuild it

| System | What it shows | Label |
|---|---|---|
| **BitNet b1.58 2B4T** (Microsoft, open weights) | A natively ternary 2B model trained on 4T tokens, with 8-bit activations. On par with 1–2B full-precision models on average; weaker on knowledge (MMLU 53.2 vs 60.3 for Qwen2.5-1.5B), stronger on GSM8K (58.4 vs 56.8). Non-embedding memory 0.4 GB; 29 ms/token on CPU; estimated 0.028 J/token against 0.19–0.65 J for comparable models (the authors' estimate, not a wall-power measurement) | Literature 2504.12285 |
| **bitnet.cpp** | Ternary CPU kernels built on T-MAC's lookup-table method. Reported 55–70% less energy than llama.cpp on ARM (tested on an Apple M2), with 1.4–5.1× speedups | Literature 2410.16144 |
| **T-MAC** | Low-bit matrix products by table lookup, with no multiplications. Up to 70% less energy than llama.cpp; 30 tokens/s on one core for a 3B ternary model on an M2 Ultra | Literature 2407.00088 |
| **MatMul-free LM** | Ternary weights plus an element-wise gated recurrence (MLGRU), with no matrix multiplication anywhere. Comparable to a modern transformer baseline up to 2.7B, and the gap narrows with scale | Literature 2406.02528 |
| **Mamba in the Llama** | Distilled Llama-3-8B into a hybrid keeping ¼ of its attention layers, with 20B tokens. It matches the teacher on chat benchmarks | Literature 2408.15237 |
| **REST; prompt-lookup decoding** | Drafting tokens from retrieved or in-context text, then verifying them in one pass: 1.6–2.4× single-batch speedups (REST), 2–4× on input-grounded tasks (prompt lookup, reported) | Literature 2311.08252 |
| **Memory layers at scale** | Trainable key–value lookup memories add capacity without adding FLOPs. They beat dense models with twice the compute | Literature 2412.09764 |
| **Apple on-device model (2025)** | About 3B parameters on Apple silicon, with 2-bit quantization-aware training and a shared context cache | Literature (Apple 2025 report) |

**Consequence.** A ternary model with no floating-point matmul at serving time already runs on Apple CPUs today. The project's first job is to **measure it on the M1**, then beat it where the table in §1 says the bytes are.

## 3. The model I would build

| Part | Design | Why |
|---|---|---|
| Weights | Ternary (1.58-bit) in every linear map; 8-bit integer activations, 4-bit later | About 10× fewer bytes than 16-bit; the products become additions |
| Kernels | Rust ports of T-MAC / bitnet.cpp lookup-table kernels (NEON `tbl`). Use the hardware integer multiplier wherever it measures cheaper, for example for activation×activation products and scales. No floating point at serving time | Lookup kernels execute fewer instructions. Software-emulated multiply is 4.4× slower (review §3.4) |
| Backbone | About 3 gated **diagonal** recurrent layers (MLGRU / RG-LRU class) per attention layer | Constant-size state, so no cache growth. Diagonal decay was the best linear mixer on text in the review (§6.3), and it is element-wise |
| Attention layers | **Superseded by [geometric routed attention](geometric-attention-2026-09-26.md)** after the owner's objection: geometric codes admit candidates, trained chunk advertisements route, exact int8 scores rank the few admitted, and hyperbolic heads carry hierarchies. The earlier design was a sliding window with a 2–3-bit norm-plus-direction cache and lookup-table scores | Recall of facts in context needs some attention or exact memory. The geometric version reads about 12–35 KB per head per token at 4k context instead of 1 MB, if admission holds (its §11 gates) |
| Exact memory | Your store-and-recall, used twice: a copy/pointer distribution (already in D8) and **a draft source for speculative decoding** | Each pass verifies several drafted tokens, so every weight byte serves several tokens |
| Output head | 4-bit or ternary; a smaller vocabulary (32–50k; Apple's on-device model used 49k in 2024); a two-level head if it still dominates | In small models the head is the largest single read (62.7% of reads in D8; review §1) |
| Optional: tracking lanes | Separate exact 2I lanes: 1 byte of state, table reads, no arithmetic | Adds exact state tracking (review §6.2). Keep them only if they improve a code-tracing evaluation |
| Optional: scale beyond cache | A product-key memory layer, possibly with lattice (E8) cell keys | Adds capacity by lookup rather than by wider dense layers. *This relaxes the earlier "no sparse routing" rule*, as the owner allowed for this plan |

**If you want to stay strictly transformerless:** drop the attention layers and use MatMul-free LM's all-recurrent ternary backbone plus exact memory for recall. Expect weaker in-context recall (Hypothesis, consistent with the recall literature). Measure the difference before choosing.

**Small specialists fit in cache.** A ternary model up to about 80–100M parameters fits in the M1's 12 MB L2 plus 8 MB system cache. It then reads from on-chip memory, which is roughly an order of magnitude cheaper per byte than DRAM (Hypothesis until measured). This suits a code-completion or command model.

## 4. How to get good weights

- **An M1 alone** trains about 20–40M parameters per week from scratch (review §11.4). That is enough to choose an architecture, not to build an assistant.
- **Recommended:** distill in the cloud and serve on the M1.
  - Teacher options: BitNet b1.58 2B4T's released bf16 master weights, which share its tokenizer, or Qwen/Llama.
  - A 1–2B ternary hybrid on 10–20B tokens costs about 1–3 × 10²⁰ FLOPs (6ND for the student plus 2ND for the teacher). That is roughly 60–250 H100-hours at 40% of a 1 PFLOP/s bf16 peak (Derived). The Mamba-in-the-Llama recipe used 20B tokens for an 8B model.
- **Recipe.**
  1. Initialise the hybrid from the teacher, mapping attention weights into the recurrent layers as in 2408.15237.
  2. Train with ternary quantization from the start of distillation: absmean weights and a straight-through estimator, as in BitNet.
  3. Use logit distillation plus the language-model loss.
  4. Finish with a short fine-tune for chat and code.
- **Training may use floating point and matmul freely.** It is a one-time cost. The goal concerns serving.

## 5. Phases and gates

**Phase 0: measure the yardstick on your M1 (this week).** This container has no Apple hardware and no energy counters, so this step must run on your machine.

1. Baselines:
   - **bitnet.cpp with BitNet b1.58 2B4T:**
     ```bash
     git clone --recursive https://github.com/microsoft/BitNet
     huggingface-cli download microsoft/BitNet-b1.58-2B-4T-gguf --local-dir models/BitNet-b1.58-2B-4T
     python setup_env.py -md models/BitNet-b1.58-2B-4T -q i2_s
     ```
     `setup_env.py` is BitNet's own build helper, not a project dependency.
   - **llama.cpp** (`llama-bench`) with Llama-3.2-1B at Q4_K_M and at f16;
   - the project's current integer session.
2. For each, record joules per generated token at contexts of 512 and 4,096:
   - sample package power with `macmon pipe`, or with `sudo powermetrics`;
   - subtract idle power, integrate over the run, and divide by the tokens generated;
   - use the protocol in review §5.5.
3. Record tokens/s, peak memory, and perplexity on one fixed held-out text.
4. *Optional:* one Core ML model on the Neural Engine, to learn whether Apple's accelerator beats CPU lookup kernels in J/token.
5. *Gate:* the resulting table of J/token against quality is the yardstick. Every later claim is measured against it.

**Phase 1: a Rust ternary runtime (weeks 1–4).**
- Port the lookup-table and 2-bit kernels, load BitNet b1.58 2B4T, and match bitnet.cpp's outputs.
- *Gate:* J/token within 10% of bitnet.cpp on the same model and context.
- Retire the software-emulated multiply.

**Phase 2: cut the bytes beyond the weights (weeks 3–8), each measured alone.**
- **Speculative decoding from exact memory.** Draft from n-gram matches in the context and a local datastore. *Gate:* at least 1.5× fewer J/token on input-grounded tasks (code editing, summarisation, chat with history) at identical outputs.
- **Compressed attention cache.** 3-bit norm-plus-direction entries with norm rescaling. *Gate:* at most 0.02 nats of loss at 4k context.
- **4-bit head.** *Gate:* at most 0.02 nats of loss.

**Phase 3: our own model (months 2–4).** Distill the ternary hybrid of §3 in the cloud.
- *Gate:* teacher-level quality on a fixed suite (held-out perplexity, a small code and chat set) at no more than half the teacher's J/token at 4k context on the M1.
- *Stretch:* the same quality at 1B parameters.

**Phase 4: research, in parallel and small.** The geometric components of §6. Each enters the model only after it passes its own gate.

## 6. Where geometry lives in this plan

| Component | Geometric content | What it saves | Evidence |
|---|---|---|---|
| Context-cache and memory keys | Norm plus direction codes (D4/E8 lattices, 600-cell), with norm rescaling | About 5× fewer cache bytes, with ranking preserved | Review §6.1 (synthetic); HQMQ 2605.27646 (Llama-3-8B) |
| Attention scores by lookup | Codebook inner-product tables (2I has 9 values) | Removes activation products from scoring | Review §6.1: the integer scorer matched float |
| Exact tracking lanes | 2I as a 120-state automaton | Exact state tracking with 1 byte and two table reads | Review §6.2, toy scale |
| Lookup-memory addressing | Lattice cells as product keys | Capacity without FLOPs | Memory layers use learned keys; lattice keys are a Hypothesis |

The backbone is ternary diagonal recurrence plus a little attention. The review's measurements do not support a rotation backbone for language (§6.3).

## 7. What changes in the programme

- **The metric.** Measured J/token at matched quality replaces "no multiplier instruction".
- **Stop:**
  - software-emulated multiplication;
  - prime and zeta mechanisms in predictive paths;
  - authored 32-prompt panels as gates;
  - from-scratch M1 training for models meant to be useful.
- **Keep:**
  - the Rust stack and bit-exact integer serving;
  - learned low-bit rounding, now to ternary;
  - the pointer-copy read and exact memory, whose identities come from the UOR address layer (κ-labels);
  - the 2I lanes as an optional capability.

## 8. Risks

- **Ternary at small scale.** BitNet showed parity at 2B parameters and 4T tokens. Small ternary models lose more, and a distilled ternary hybrid is less proven than either part alone (Hypothesis).
- **Speculative gains depend on the task.** They are large when output copies input; smaller for open-ended text.
- **Estimates are not measurements.** Every energy number in §1 is derived, and M1 measurements may differ by 2× in either direction. Phase 0 exists to replace them.
