# redteam2 (FEASIBILITY/ENGINEERING): adversarial review of ROADMAP_DRAFT (2026-09-26)

Target: `$S/lab/draft/ROADMAP_DRAFT.md` (the version with annealed M1b), reports arch2/train2/sys2/math2/exp2, repo HEAD
50bee85 (`kappa_llama.rs`, `kappa-conversion.rs`, read-only). Labels: **M** Measured here (files in
`$S/lab/exp/redteam2_feas/`), **D** Derived, **L** Literature retrieved this session, **H** Hypothesis.

Memory runs used the committed release binary (`mode=train trainable=query_key`, KD from a same-shape teacher, one
thread) on synthetic bf16 checkpoints with SmolLM2-135M width, heads and vocab at 1 and 3 layers. Peak RSS is in
`mem_runs.jsonl`.

## CRITICAL

**C1. M1b cannot run on an M1 as specified (out of memory).**

*Location:* roadmap §4 M1b ("owner's M1, hours"). Tool defaults `batch=4 time=256` (`kappa-conversion.rs:383–384`).

*Cause:* each layer keeps about 30 full `(B,H,T,T)` intermediates for backward (`head_scores` + `arcosh1p_squared`,
`kappa_llama.rs:294–355`). The 49,152-wide KL also keeps its log-softmax tensors alive (`:702`).

*Evidence (M):*

| Configuration | Per layer | Intercept (head/KL + embeddings) |
|---|---:|---:|
| intrinsic, 4×256 | 771 MB (736 MB with a fixed mmap threshold, so this is live memory, not heap retention) | 4.3 GB |
| dot, 4×256 | 275 MB | 4.35 GB |
| intrinsic, 1×256 | 240 MB | 1.44 GB |

*Projection (D):* 30 layers plus the real 360M teacher (1.45 GB in f32):

| Arm | Peak |
|---|---:|
| Curved arms, defaults | ≈27 GB |
| Dot control, defaults | ≈13.5 GB |
| Curved, batch 1 | ≈9.5 GB |

- No arm fits an 8 GB M1 at the defaults.
- On 16 GB only batch 1 has a chance. Metal caps the GPU working set below physical RAM (H: about ⅔–¾).
- train2's measured OOM at 8.9 GB with 4×256 is consistent.

*Fix:*
1. Before scheduling anything, run `steps=1 batch=1` on the Mac and read the peak.
2. Add gradient accumulation (the tool has none).
3. Cache sampled or top-k teacher logits offline, so the 360M teacher and its 49K-wide log-softmax are not resident.
4. Chunk the head/KL computation.
5. Fuse `arcosh1p_squared` into a custom op with an analytic backward, or recompute it per layer. This removes about
   460 MB/layer at 4×256.

**C2. The energy thesis compares unlike configurations. For the model the plan actually builds (4-bit), the lab's own
model predicts parity with llama.cpp.**

*Location:* §1.3 ("4–7 mJ … vs ~44 mJ fp16, ~27 mJ Q4_0") and the M3 test ("≥2× below Q4_0 on E-cores").

*Evidence (D):*
- The 4–7 mJ rows are sys2's **ternary** path on **E-cores**; the 44/27 mJ baselines run on **one P-core**.
- sys2's own rates make the D0-b 4-bit LUT slower per weight than Q4_0 `sdot`: 20.5 vs 35.8 G weights/s on a P-core.
- I re-ran `energy_model.py` unchanged except for 4-bit weights and matched core/QoS (`energy_recheck.py`), 135M at 2K:

| Comparison | P-core | 4 E-cores | E-cores ≈1 GHz |
|---|---:|---:|---:|
| 4-bit LUT + 36-bit read vs Q4_0 + fp16 KV | 30.8 vs 26.7 mJ (**worse**) | 11.6 vs 14.3 | 6.9 vs 10.2 |
| Same, with int8 keys, vs Q4_0 + llama.cpp q8_0 KV | 31.6 vs 25.1 | 12.4 vs 11.9 | 7.7 vs 8.6 (**≈parity**) |

- The int8-key row is the fair one. sys2's 36-bit code was measured on 8-dim keys; SmolLM2 heads are 64-dim, and arch2
  F5 measured KL up to 0.12 for 4-bit per-tensor keys. Only int8 keys were shown to be near-lossless.

*Literature agrees on magnitude (L 2407.00088):* T-MAC's energy saving over llama.cpp is 20.6% for the 4-bit model. The
61.2% and 51.3% savings are for the 2-bit model and BitNet.

*Ternary* reaches 2.4× at matched QoS (D). But it needs about 30B QAT tokens (L ParetoQ 2502.02631), which is ≈4.6e19
FLOP with the 360M teacher, or **≈1.3 H100-days per attempt** at arch2's assumed 40% utilization (D). That is paid
compute.

*Where the geometry does pay (D):* at 8K context with 3% admission, 7.1 vs 14.8 mJ (E-cores ≈1 GHz) and 11.7 vs 20.3 mJ
(4 E-cores). KV reads dominate there.

*Fix:*
1. Restate thesis 3 as: "≈parity at 4 bits and 2K; about 2× at ≥8K, contingent on M4 admission, or with ternary."
2. Make M4, not M3, the energy gate.
3. Put the ternary paid-compute number in front of the owner now.

## MAJOR

**M1. M1b's decision metrics are not interpretable as written.**

- *(a) Raw κ is not scale-free.* `score(cq, ck; ε) = c²·score(q, k; cε)` (D, from `:309–355`).
  - Measured on the lead's tiny checkpoint: scaling q/k by 3× moves the KL at the same κ from 1.5e-6 to 1.5e-2
    (log ε = −4), and from 7.3e-5 to 0.22 (log ε = −3) (M, `probe_tiny_*`).
  - So "per-head κ histograms", a global `init_log_eps` and a global anneal floor (`floor_log_eps`, `:673–681`) hit
    heads with strength that varies by orders of magnitude.
  - With `trainable=query_key`, κ is not even identifiable: q/k norm, ε and β can trade off against each other.
  - *Fix:* report and anneal a dimensionless curvature per head (κ·median|k|², or the mean realized z). Add per-head
    KL-vs-κ to `probe`.
- *(b) "Hierarchy-dependent events" have no implementation for SmolLM2.* Event labels exist only for the D8 4,096-BPE
  code set, and the tool emits only mean valid NLL and KL.
  - *Fix:* before M1b, tokenize the repo's Rust code with SmolLM2's tokenizer, reuse math2's scope labelling, and add
    per-event NLL to the tool.
- *(c) "Match or beat" will pass trivially.* Only q/k are trainable, so the KL gap to the 360M teacher is mostly MLP
  knowledge. In the stand-in, every variant landed within 0.004 bpb.
  - *Fix:* pre-register the minimum detectable effect from the two dot seeds.
  - Add an `anneal_hold=false` release arm. It is the only arm where the data, not the floor, sets κ.

**M2. Gate on "no transformer backbone" before M2, not in parallel.**
- At serving, GCM-L is Llama blocks (RMSNorm, QKV/O maps, SwiGLU) run through 4-bit LUT GEMV.
- AGENTS.md names "a dense transformer hidden behind lookup" as not the target.
- The draft asks the owner (§6.3) but schedules weeks of M2/M3 compute regardless.

**M3. M2 omits a serving requirement.**
- D0-b serving needs integer activations with power-of-two scales, table SiLU/exp/rsqrt/arcosh², and a ≤4-bit
  49,152-row head.
- The cited QAT budgets are weight-only with float activations. ParetoQ quantizes "all weights except for the embedding
  and output layers" (L).
- *Fix:* QAT must train against the exact integer forward. First run a forward-only integer PTQ baseline (<1 h).

**M4. `sample` silently loads the wrong model.**
- `sample` defaults to `score=dot trainable=scalars` (`kappa-conversion.rs:635–650`) and restores only the variables
  the fresh model declares.
- Saved `log_eps` and trained q/k weights are therefore ignored without error. A default sample after an
  intrinsic/query_key run shows a teacher-like model: a false sanity pass for M1a/M1b.
- Derived from the code, not executed (the tiny checkpoint has no tokenizer).
- *Fix:* require the saved name set to equal the model's variable set, or read `score` and `trainable` from
  `report.json`.

**M5. The tool is confined to short windows.**
- Every layer materializes about 30 T×T tensors, plus a full −∞ tensor per forward (`:567`).
- At T = 2,048 and batch 1, each one is 151 MB per layer (D). So M4's 2K–8K work, and serving at 8K with κ learned at
  T = 256, are untested.
- *Fix:* a chunked, online-softmax implementation before M4.

## MINOR

- **m1. κ is per query head** (`grid=(layers, heads)`, `:466`). Under GQA the lifted key and radius code then differ
  across the 3 query heads sharing a KV head, while sys2 costs one code per KV head. Tie κ per KV head.
- **m2. No checkpoint or resume in `train`.** A multi-hour arm dies if the Mac sleeps. Save variables periodically.
- **m3. The chat template omits SmolLM2's default system turn** ("You are a helpful AI assistant named SmolLM…", from
  `tokenizer_config.json`, L).
- **m4. Latent NaN.** The series branch is evaluated for all z. z⁴ overflows f32 at z ≳ 3e9, and 0·∞ in the
  `where_cond` backward then gives NaN. It only happens at very large κ|q||k|. Clamp z before the series.
- **m5. The tool's f32 KL floor is about 1e-8** (the lead's probe logged KL = −2.9e-9).
- **m6. Cost figures.**
  - The arithmetic reproduces for arch2 F6 (7.4 PFLOP → 6.8 h; 0.2B-token QAT → 10 days) and train2 F4/F5 (3e17 FLOP →
    11–37 days).
  - The load-bearing 0.3 TFLOP/s Candle-Metal rate is unmeasured. The project record has D8 slower on Metal than on the
    CPU.
  - The FLOP counts omit the curved score's roughly 30 memory-bound passes per layer.
  - "≈1 GPU-hour" omits setup, retries and evaluation. H: budget ≥10× for a first rental.
- **m7. Literature labels (L; the core numbers were verified):**
  - Mamba-in-the-Llama (2408.15237): "0% attention: 5.64" is the Mamba2-Llama3 row. "Without attention init: 1.04" is
    Zephyr-Mamba 50% (Table 8), not Llama-3. These runs used 20B tokens and about 5 days on 8×A100.
  - math2's "T-MAC up to 70% less energy" is the 2-bit best case; for 4-bit it is 20.6%.
  - ParetoQ at 125M, 3-bit is quoted correctly (47.9 vs 48.5; Wiki2 21.6 vs 14.9). But its 4-bit row (Wiki2 20.4 vs 9.2
    for plain rounding) shows the small-model perplexity column is not a usable planning anchor.
  - LoLCATs verified: 52.8 vs 66.6; 23.8 without the window; 65.8 with half the layers softmax; Llama-3.2-1B 59.6/60.0
    and 27.3/31.9.

## Checked and correct (do not redo)

- **GQA and RoPE (D):** match HF `repeat_kv` and half-split Llama RoPE.
- **Real checkpoints (L+D):** the tied head and strict inventory match the real SmolLM2-135M/360M configs and file sizes
  (134,515,008 / 361,821,120 bf16 parameters; no `lm_head`).
- **Tokenizer and mask:** `<|im_end|>` is a single stop token. The −∞/`where_cond` mask is safe.
- **Metal:** every op/dtype pair has a kernel in the vendored candle, and the lockfile has the Metal crates. Not
  executed on Metal.
- **f32 precision:** adequate at κ₀ = 1e-4. Below κ ≈ 1e-9 the log ε gradients are rounding noise.
- **M1a** (forward-only) fits on 8 GB (D).

## Steelman (feasibility)

"Serve SmolLM2-135M today with llama.cpp Q4_0 and a q8_0 KV cache, pinned to E-cores with background QoS.
- By the lab's own model that is ≈8.6 mJ/token at 2K, with teacher-quality chat and no new code.
- The 4-bit geometric path lands at ≈7.7 mJ and loses quality to integer-only activations.
- Curvature on a teacher-matching objective has, so far, only matched the flat limit."

It is right at 2K and wrong at ≥8K, where admitted reads roughly halve the energy (D). Multiplier-free, float-free
serving is also a goal in itself.

So the geometric value lies in long-context memory and admission (M4). That matches thesis 2; it does not match thesis 3
or the M3 criterion.

## What is right and should not change

- Conversion first, from approved Apache-2.0 weights.
- The exact κ-homotopy start with a matched dot control and paired data order.
- The anneal arms. The anneal code is correct (projected Adam, logged floor, dot refused).
- Geometry moved to the long-context index and memory role.
- Strict config/inventory refusal and sealed report roots.
- M1c measuring before trusting constants, and the owner questions.

## Single most likely failure, and the one test to run first

**Likely failure.** Weeks of M1 time produce a 4-bit, integer-only converted SmolLM2 that:
- chats worse than the teacher;
- has J/token ≈ llama.cpp Q4_0 on the same cores;
- shows no measurable geometric contribution at chat-length contexts.

**First test (owner's M1, about 1 h, no new code).** Extend M1c:
1. Run `llama-bench` on SmolLM2-135M-Instruct Q4_0 with `-ctk q8_0 -ctv q8_0 -p 0 -d 2048` and `-d 8192`.
   - Once on one P-core.
   - Once under `taskpolicy -c background`.
2. Measure J/token with powermetrics, using sys2 §R1's differential method.
3. Run it next to `geo-bench --step135` and an int8-key 4-bit variant of it.

**Decision rule.**
- If Q4_0 on E-cores is within 1.5× of the stand-in at 2K: rewrite thesis 3 and M3, and bring the ternary paid-compute
  decision forward, before any M2 compute.
- If the 8K gap is ≥2×: M4 becomes the energy gate.

In the same session, run `kappa-conversion mode=train steps=1 batch=1` to confirm the C1 memory figure before scheduling
M1b.
