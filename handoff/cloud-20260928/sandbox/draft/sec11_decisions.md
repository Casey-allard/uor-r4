## 11. Decisions only the owner can make

Each decision below is stated with its options, their consequences, and the review's recommendation where one is justified. Nothing changes until the owner rules. As noted in the Method, the shared briefing asked every agent to flag the tension in §11.1, so its repetition across reports is not independent evidence.

### 11.1 "No sparse routing" vs D5 (owner-ratified, per-token parameter sparsity)

The owner's words in this request: "no traditional matmul in runtime (floating point), moe, or sparse-routing". D5 (DECISIONS.md:241, owner-ratified on Sep 24) makes per-token parameter sparsity the terminal serving invariant.

Facts that bear on the choice:
- **Sparse access matters only above cache size.** For models too large for cache, touching fewer parameters is the largest energy lever. For example, touching 10% of a ternary 7B model's weights means 146 MB/token instead of 1.46 GB (physics report).
- **Below cache size, it buys little.** A ternary model of up to about 18–23M parameters (4.5–5.7 MB) fits in the M1's L2 cache (arch report).
- **Evidence exists only for learned routers.** The best evidence that sparse access preserves knowledge (32 experts, 8.8% of parameters touched, about 1.3× capacity loss) comes from a *learned* router (2404.05405), which the owner has excluded.
- **Some access is always selective.** Every model selects one embedding row per token, so "no per-token selection" cannot be literal.

Options:

| Option | Meaning | Consequence |
|---|---|---|
| **A** | Forbid learned gating (MoE routers, learned top-k memory). Allow *deterministic* addressing: a learned state is quantized to a fixed geometric codebook cell (600-cell/E8), and that cell selects table rows. Note that this is **top-1 access with a fixed codebook over learned states**, which the owner may still regard as MoE-like. | D5 remains achievable for models larger than cache. It must beat LSH, k-means and PQ addressing at matched bytes touched to count as a geometric win. |
| **B** | Forbid content-dependent selection of parameter subsets beyond the token's own embedding row. | Retire D5 as a terminal invariant. Serving is dense and low-bit. The levers become bits per parameter, cache residency and kernel efficiency. This is practical up to about 20M parameters at ternary, and caps efficiency above that. |
| **C** | Defer. Keep D5 as an aspiration, but scope it to models above cache size and revisit when one exists. | Removes the tension from present work. The current and near-term models are cache-resident. |

*Review recommendation:* **C now**, then A or B once a model larger than cache exists.

### 11.2 D0-b and the multiplier

D0-b states its objective as "no multiplier, tiny RAM, no GPU, local, measured energy … the multiplier constraint stays" (DECISIONS.md:113-114). The owner's wording in this request targets *floating-point* matmul.

Facts:
- **Cost of the current path.** Emulating integer multiplication in software makes the integer path 4.4× slower on x86 (bit-identical output), and very likely costs more energy on a CPU that has a multiplier. This is not yet measured on the M1.
- **Multiplier-free can save energy.** T-MAC showed that multiplier-free *lookup-table* kernels save energy when they remove instructions.

Options:

| Option | Meaning | Consequence |
|---|---|---|
| **A** | Amend D0-b so that *integer* activation products may use the hardware integer multiplier. Keep: no floating point, weights of 4 bits or fewer, weight maps executed by add or lookup table. | This changes an owner-stated objective. It gives the simplest and fastest path; energy is then judged by measurement. |
| **B** | Keep D0-b and remove activation×activation products **by design**: codebook-quantized keys with lookup-table scores; snapped rotation lanes; low-bit activations where they train; the analytic uniform floor. | The geometric way to honour D0-b. It needs engineering, and training risk for low-bit activations (ternary Q/K failed in MatMul-free LM). |
| **C** | Keep D0-b with software emulation (status quo). | Measured slow, and probably energy-negative. |

*Review recommendation:* decide **after the Phase-0 joule measurement** (§9.2 item 3). If hardware multiply is lower in J/token, prefer **A** for activation products and **B** wherever the geometry provides it naturally.

### 11.3 How may an existing model be used?

Current policy allows an offline teacher as a *training source*, but not as the author of served responses. The levels of use:

| Level | Use | Status |
|---|---|---|
| 1 | **Data**: more TinyStories (itself model-generated) plus open synthetic dialogue and code corpora | Allowed |
| 2 | **Logit distillation** from #1017 or an open model | Allowed. Helps most early and at small student compute |
| 3 | **Weight transfer or architecture conversion**: keep a pretrained model's channel-mixing weights (as ternary or 4-bit maps) and replace its attention with the geometric recurrence | Needs an explicit decision. The served model would contain transformer-derived weights, though no attention. Honest costs: LoLCATs-style 40M-token conversion keeps softmax attention windows, so it is not transformerless. Fully attention-free conversion used about 3B tokens in the literature (MOHAWK), which is months on an M1 at these sizes. |

*Review recommendation:* use levels 1 and 2 now. Treat level 3 as an optional, reduced-token experiment only if the owner wants the data point.

### 11.4 Compute scope

M1-only is the current authority. It caps from-scratch training at roughly 20–40M parameters on about 1–2.5B tokens per week, and only if the throughput fix delivers its assumed rate. That means TinyStories-class coherence, not general chat.

Modest rented GPU time would raise the feasible scale by 10–100×. That is an owner policy choice (currently "no paid compute"). It is listed only because it is the main lever this constraint rules out.

### 11.5 Process

Options: adopt the lightweight process in §10.2, or keep the current governance. The audit found that most D8 cycles spent more wall-clock time on process than on model computation. It also found that principal reviews caught real errors, and those reviews should be kept.
