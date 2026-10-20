# Roadmap draft: from here to a chat-capable geometric LM on an M1 (lab lead, 2026-09-26)

Inputs: expert reports `$S/lab/reports/{math2,arch2,train2,sys2,exp2}.md`; lead experiments `$S/lab/exp/lead_conv/`;
committed Rust tool `crates/uor-r4-training/{src/kappa_llama.rs,examples/kappa-conversion.rs}` (branch commit c67e230).
Labels: M Measured, D Derived, L Literature (retrieved this session by the named expert), H Hypothesis.

## 1. Thesis

1. **Chat quality is bought with training tokens, and the owner's compute cannot buy them from scratch** (arch2 F1,
   train2 §6; L+D). Open small talk appears at ~100–350M parameters after 1–11T tokens; an M1 trains ~0.2B tokens/week
   at 150M. The only route to the 135–360M band inside the budget is **converting an open instruct model**
   (SmolLM2-135M-Instruct first, 360M next; Apache-2.0), which the owner approved on condition of no runtime matmul.
2. **Geometry's job is where it measurably pays, not everywhere** (math2 §1, §3; exp2 F1–F4; M). Hyperbolic structure
   gives large wins on sparse hierarchy-determined retrieval (code-scope tree: hyperbolic keys 61–90% vs dot 0–11%)
   and on admission indexes (radius-aware cells, Busemann landmarks), and small wins on mean next-token loss
   (cycle 3: ~0.02 nats, concentrated on enclosing-scope copies at −0.06 nats/token in 3/3 seeds; a perfect scope
   oracle is worth ≤0.02 nats at ≤2k tokens). Context length is the big lever (0.40 nats from 128 → 2,048 in the oracle),
   so **hyperbolic geometry is most valuable as the long-context index and memory, not as a per-token score.**
3. **Energy is dominated by the weight path, not the multiplier** (sys2; D from published constants, not M1-measured).
   At ≥85% of J/token, ternary/4-bit table-driven weights on efficiency cores are the main saving (projected 4–7 mJ/token
   for a converted 135M vs ~44 mJ fp16, ~27 mJ Q4_0). The hyperbolic read with compact keys costs ~7% of step time and
   removes most KV bandwidth at 2K context.

## 2. What exists today

| Piece | State | Evidence |
|---|---|---|
| D8 joint learner (Rust/Candle), Lorentz read option, checkpoint/resume | committed | cycle 3 (M) |
| Exact dot→hyperbolic conversions (three constructions) | derived, measured on real and synthetic heads | arch2 F3 (κ-homotopy, key-norm bias), lead (intrinsic polarization), math2 §4 (norm completion) |
| `kappa_llama` + `kappa-conversion` (tokenize / probe / train / sample) for HF Llama checkpoints | committed c67e230, 6 tests, e2e on a synthetic bf16 GQA checkpoint | flat-limit max logit error 8e-6 at κ≈4e-11 (M) |
| Stand-in teacher conversion (3-layer 0.6M attention LM, WikiText bytes, 1.990 bpb) | done | table below (M) |
| Admission indexes: content cells, cascade, radius-aware core+direction | scratch | exp2 (M) |
| Integer kernels (packed ternary LUT, per-query q·k tables, quarter-square) | scratch, x86-timed, NEON compiled not run | sys2 (M ratios; D energies) |
| SmolLM2 weights | not reachable here (HF egress blocked); on the owner's Mac | train2 §1 (M) |

Stand-in conversion (bits/byte on WikiText-2 valid; teacher 1.9897; 400 KD steps on Q/K + temperature only; M):

| score | zero-shot | after Q/K KD |
|---|---:|---:|
| dot (drift floor) | 1.9897 | 1.9909 |
| intrinsic polarization (exact at κ→0) | 1.9897 | 1.9905 |
| cosine (fitted β) | 2.051 | 1.9908 |
| Gromov product (q\|k)_o, fitted scale (tree: LCA depth) | 2.049 | 1.9907 |
| Euclidean distance | 3.93 | 1.9929 |
| committed Lorentz read β(δ−d) | 4.01 | 1.9946 |

Reading: swapping a trained model's score geometry is cheap — every variant recovers to within 0.004 bpb of the teacher
with ~1.6 MB of distillation text updating only Q/K. Exact constructions start there. On real teacher heads the
intrinsic variant drifts from the teacher *more gently* than arch2's key-norm variant as κ grows (mean row KL at log ε=−3:
0.06–0.09 vs 0.24–0.53 on layers 1–2; M) — the reverse of arch2's synthetic-head result.

## 3. The model: GCM-L (converted), stated as mechanisms

1. **Backbone:** SmolLM2-135M-Instruct weights (embeddings, MLPs, projections) — 4-bit QAT first, ternary later
   (train2 §5: ParetoQ ~10B tokens to saturate 4-bit, ~30B for ternary; L).
2. **Attention score:** κ-hyperbolic per head, initialized exactly at the teacher (κ≈1e-4 or smaller), κ learned under
   a data objective or a stronger teacher (360M). Heads whose κ stays ≈0 snap to int8 dot tables; heads with κ>0 keep
   Lorentz tables (math2 §5b). **Correction (math2 E11/E11b, M, toy code-tree heads):** an exact conversion fine-tuned
   with learnable curvature *stays in the flat basin and keeps the teacher's blind spot* (non-root 0.4% → 0.4%);
   annealing curvature up lifts it to 31%, a direct curved conversion to 52.5% (after a transient collapse), a fresh
   hyperbolic head gets 85.5%. So arch2's falsifier ("<10% of heads reach κ>1e-2") would be a false negative on an
   exact start. The test must use **annealed arms** (tool: `anneal_to`, commit 50bee85) against the dot control, judged
   on held-out KL/NLL *and* hierarchy-dependent events (enclosing-scope copies, entity/scope resolution).
3. **Context:** hybrid KV (arch2 F4): a few exact layers plus windowed layers with RoPE-transported recurrent state;
   long-range reads through a content-addressed index (exp2 P1/P2 cascade: cells → 256-bit codes → exact), radius-aware
   core+direction cells for hierarchical heads (exp2 P3), Busemann-landmark admission (math2 §5d).
4. **Parameter memory (gated on owner D5 decision):** deterministic n-gram (Engram-style) or product-key memory with
   E8/sign sub-codebooks (math2 §6, arch2 F7) for factual recall.
5. **Serving:** integer and table-driven only: T-MAC-style LUT GEMV for ≤4-bit weights, per-query product tables for
   q·k, quarter-square tables for activation products, sealed exp/rsqrt/arcosh² tables (sys2, arch2 F5).

## 4. Milestones

| # | Where | Work | Decision it makes |
|---|---|---|---|
| M1a | owner's M1, <1 h | `kappa-conversion mode=probe` on real SmolLM2-135M-Instruct; `mode=sample` sanity | Is the flat-limit conversion exact on the real model (max logit error, KL, top-1 agreement vs κ)? |
| M1b | owner's M1, hours | `mode=train` KD from SmolLM2-360M-Instruct into the converted 135M, `trainable=query_key`, paired data order, 2 seeds: `dot`; `intrinsic` learnable; `intrinsic` annealed (`anneal_to` ∈ {−3, −2, −1}); `key_norm` annealed | Do annealed curved heads match or beat the dot control on held-out KL/NLL, and do they win on hierarchy-dependent events? Per-head κ histograms |
| M1c | owner's M1, 1 h | Baselines: llama.cpp f16/Q4_0 SmolLM2-135M J/token (sys2 §R1 procedure); Candle-Metal throughput (train2 cbench) | Replace every assumed M1 constant |
| M2 | M1 days, or ~1 GPU-hour if approved | 4-bit QAT distillation of the converted student on chat data (smol-smoltalk, everyday-conversations; Apache-2.0) | Chat quality at 4 bits (masked assistant NLL, IFEval subset, multi-turn fact recall) |
| M3 | here + M1 | Integer serving of the converted 4-bit model: LUT kernels, table softmax/norms, κ tables; chat CLI | Measured J/token vs llama.cpp (claim holds at ≥2× below Q4_0 on E-cores; sys2) |
| M4 | here + M1 | Hybrid KV + admission cascade at 2K–8K; event-level evaluation (enclosing-scope copies, entity recall) | Does hyperbolic admission keep ≥99% of attention mass at ≤2% scored on real heads? |
| M5 | M1 | Parameter memory arms (none / learned product keys / E8 / sign / Engram) | ≥0.02 nats or factual-recall gain at equal active parameters |

Runnable here meanwhile (each ≤15 min, 1 core): (a) curvature under a data objective on the stand-in (running:
`ft.py`); (b) exp2 #1 real-q/k admission on saved D8 heads; (c) math2 §5a two-level hyperbolic memory gate;
(d) context-256 Dot vs flat Lorentz D8 (running, 4 runs).

## 5. Risks (lead's own)

1. **Conversion quality at 4 bits** is the dominant risk to chat; hyperbolic scoring is secondary to it.
2. **The curvature test may come back null** on a teacher-matching objective; then hyperbolic geometry's role is the
   index/memory, which must be shown on real heads (M4), not assumed from synthetic MQAR.
3. **"No transformer backbone"**: the converted stack is transformer-derived (Llama blocks with replaced attention and
   LUT-executed ≤4-bit maps). AGENTS.md forbids a transformer backbone at serving; the owner's weight-transfer approval
   may or may not cover this reading.
4. **M1 constants are assumed** (0.3 TFLOP/s Candle-Metal, energy per byte of caches); every cost figure moves with M1c.
5. **`unsafe`**: NEON table kernels need SIMD intrinsics, which the portable runtime crates forbid.

## 6. Owner decisions

1. Enable huggingface.co (+ its download hosts) for this environment, or keep real-teacher work on the M1.
2. Paid compute: ~1 H100-hour covers the 135M 4-bit conversion; ternary needs far more. Approve a small budget?
3. Does a converted stack (Llama-derived blocks, κ-hyperbolic attention, LUT ≤4-bit maps, integer serving) satisfy
   "no transformer backbone at serving", or should the rule be restated as "no dense float matmul and no multiplier at
   serving"?
4. Allow one small audited SIMD crate with `unsafe` for NEON table kernels (outside the frozen `forbid(unsafe_code)`
   crates)?
5. D5: are geometric-code-addressed memory layers (sparse parameter access without a learned gate) allowed?
6. Evaluation: judge by event-level metrics (scope/entity resolution, long-memory recall) alongside mean NLL?
