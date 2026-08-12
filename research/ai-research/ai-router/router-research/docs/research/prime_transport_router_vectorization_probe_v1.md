# Prime Transport Router — Vectorization Probe v1

## Baseline Config

CPU, D_HIDDEN=32, batch=256, D=24, pos0_bias=enabled, threads=1
torch=2.11.0, python=3.10.20, macOS-26.4-arm64-arm-64bit

## Loop Decomposition: Sub-Operation Classification

| Sub-operation | Depends on τ[t-1] or s[t]? | Classification | Vectorized? | % of loop time |
|---------------|---------------------------|----------------|-------------|----------------|
| A. token_select | No | trivially_vectorizable | yes | 0.9% |
| B. embedding_lookup | No | trivially_vectorizable | yes | 2.0% |
| C. cat | Yes (τ) | eliminable (split matmul) | yes | 1.7% |
| D. linear1_mm+tanh | Yes (τ) | partial (emb part vectorizable, τ part sequential) | partial | 20.9% |
| E. linear2_mm | Yes | sequential | no | 2.7% |
| F. gumbel_noise | No | trivially_vectorizable | yes | 9.3% |
| G. softmax | Yes | sequential | no | 9.1% |
| H. TN_lookup | Yes (s[t]) | sequential (coupled recurrence) | no | 26.9% |
| I. einsum_blend | Yes | sequential | no | 14.5% |
| J. injection | No (t=0 only) | trivially_vectorizable (mask) | yes | 0.3% |
| K. argmax | Yes | sequential (non-diff) | no | 3.9% |
| L. TR_lookup | Yes (s[t]) | sequential (non-diff) | no | 5.8% |
| M. gather_route | Yes | sequential (non-diff) | no | 1.9% |

**Vectorizable fraction of loop time: 35.2%**
**Mathematically sequential fraction: 64.8%**

## Why the Core Recurrence is Mathematically Sequential

The recurrence is:
```
τ[t] = einsum(softmax(MLP(emb_proj[t] + τ[t-1] @ W1_tau) / temp), TN[s[t]])
s[t+1] = TR[s[t]].gather(argmax(w[t]))
```
where MLP includes tanh (nonlinear) and softmax (nonlinear).

This is a **nonlinear coupled recurrence**:
- τ[t] depends on τ[t-1] through tanh and softmax — NOT a linear recurrence
- s[t+1] depends on s[t] through argmax — NOT even continuous
- The two are coupled: τ[t] needs TN[s[t]], and s[t+1] needs argmax(w(τ[t-1]))

**Parallel prefix/scan** requires an associative binary operator over the recurrence.
For linear recurrences (y[t] = A[t]·y[t-1] + b[t]), the operator (A,b)∘(C,d) = (AC, Ab+d)
is associative, enabling O(log n) parallel evaluation (used in S4, Mamba, etc.).

The current recurrence is **not linear**: it involves tanh, softmax, and state-dependent
table lookup. There is no known associative composition for this class of recurrence.
It is **mathematically sequential**, not accidentally sequential due to implementation style.

## What Was Vectorized

The following operations were hoisted out of the D=24 loop:

1. **Token embeddings** — `W_emb[tokens]` computed once as (B, D, D_EMB)
2. **Embedding projection** — `embs_all @ W1[:D_EMB,:]` computed once as (B, D, D_HIDDEN)
3. **Gumbel noise** — pre-generated once as (B, D, N_OPS)
4. **Token injection** — `W_tok_inject[x0]` computed once as (B, D_TAU)
5. **cat elimination** — replaced `cat([embs, τ]) @ W1` with `emb_proj[t] + τ @ W1[D_EMB:,:]`

What remains in the loop (irreducible sequential core):

- `τ @ W1_tau + b1` — matmul (B,21)×(21,32), depends on τ[t-1]
- `tanh(...)` — nonlinear activation
- `h @ W2 + b2` — logits, depends on h
- `softmax(...)` — nonlinear, depends on logits
- `TN[state_ids]` — table lookup, depends on s[t]
- `einsum(w, tn)` — blend, depends on w and TN lookup
- `argmax + TR lookup + gather` — hard routing, depends on w and s[t]

## Before/After Runtime Comparison

| Metric | Baseline | Vectorized | Change |
|--------|----------|------------|--------|
| Steps/sec | 119.3 | 93.0 | 0.78x |
| Total wall (300 steps) | 2.51s | 3.23s | +28.3% |
| Forward time | 1.571s (62.5%) | 1.795s (55.6%) | +14.3% |
| Backward time | 0.833s (33.1%) | 1.312s (40.7%) | +57.4% |
| Forward-only sps | 219.9 | 203.8 | 0.93x |

**Semantic equivalence: max output diff = 4.47e-08** (PASS)
**JIT scripted: baseline=yes, vectorized=no**

## Did the Bottleneck Move?

**No. The vectorized version was strictly worse: 0.78x training throughput.**

- Forward: +14.3% slower (1.571s → 1.795s)
- Backward: +57.4% slower (0.833s → 1.312s)
- Forward-only (no backward): 0.93x — also slower

The pre-computed `(B, D, D_HIDDEN)` embedding projection created a larger intermediate
tensor in the autograd graph. The backward must now propagate gradients through this
(256, 24, 32) tensor aggregated across all D steps, which is MORE expensive than 24
individual small matmul backwards in the baseline.

**JIT confound:** The vectorized model failed TorchScript (global constant in slice).
Baseline ran JIT-scripted, vectorized ran eager. This accounts for some of the gap,
but even in forward-only (eval, no backward, no JIT advantage for either), the
vectorized version was 0.93x — confirming the pre-computation doesn't help.

**The bottleneck did not move. It is structural: the 24-step nonlinear recurrence.**

## Is the Remaining Sequential Portion Fundamental?

**Yes.** The sequential core is a nonlinear coupled recurrence:

- τ[t] depends on τ[t-1] via `tanh(... + τ[t-1] @ W_tau ...)` — nonlinear
- s[t+1] depends on s[t] and argmax(softmax(MLP(τ[t-1]))) — discrete + nonlinear
- TN[s[t]] lookup couples the continuous path (τ) to the discrete path (s)

This is NOT a linear recurrence. It cannot be reformulated as a parallel scan.
The sequentiality is **mathematical**, not an implementation artifact.

To make this parallelizable would require **changing the model architecture** to a
linear recurrence (like S4/S5/Mamba), which would change the operator semantics —
explicitly prohibited by constraints.

## Honesty Section

### What was fully vectorized

- Token embedding lookup: all D steps computed in one `W_emb[tokens]` op
- Embedding projection: `embs @ W1[:4,:]` for all D steps in one matmul
- Gumbel noise: pre-generated for all D steps in one `rand` + `log` op
- Token injection: computed once, applied conditionally
- cat elimination: replaced with split matmul (no allocation)
- These represent **35.2%** of inner loop time

### What resisted vectorization

- The τ-recurrence MLP (tanh + matmul chain): **nonlinear, depends on τ[t-1]**
- State table lookups (TN, TR): **depend on state_ids which changes each step**
- Softmax + einsum blend: **depend on the MLP output**
- Hard routing (argmax + gather): **discrete, coupled to τ path**
- These represent **64.8%** of inner loop time — the dominant cost

### What still remains the dominant bottleneck

The D=24 sequential recurrence is the bottleneck. It forces both forward and backward
into a 24-step serial chain. The vectorized pre-computation reduces per-step dispatch
overhead but does not change the fundamental O(D) sequential depth of the computation.

## Thread Count Analysis

The `torch.set_num_threads(1)` setting was tested against alternatives:

| Threads | Steps/sec | Forward (s) | Backward (s) | Total (s) |
|---------|-----------|-------------|--------------|-----------|
| **1** | **98.2** | 1.32 | 0.66 | 2.04 |
| 2 | 91.1 | 1.38 | 0.75 | 2.20 |
| 4 | 84.8 | 1.46 | 0.83 | 2.36 |
| 6 | 82.3 | 1.50 | 0.86 | 2.43 |
| 8 | 78.1 | 1.58 | 0.92 | 2.56 |
| 10 | 78.0 | 1.59 | 0.92 | 2.56 |

**`num_threads=1` is optimal for this workload.** More threads makes it strictly worse
because the per-step tensor ops (256×25×32 matmul, etc.) are too small for thread
synchronization overhead to amortize.

This means the workload is **fundamentally capped at 1 core**. The 100% CPU in Activity
Monitor is correct: one thread doing continuous work on one core. Adding threads wastes
time on futex/spinlock synchronization for each tiny op.

This also explains the original observation of "thread count changes 8→13 but total CPU
stays ~100%" — if that was without `set_num_threads(1)`, PyTorch spawned 8+ worker
threads per op, but they spent their time synchronizing, not computing. The real work
was always one core's worth.

The only paths to eliminate the throughput ceiling are:
1. **Architectural change** — replace the nonlinear recurrence with a linear one (changes semantics)
2. **Reduce D** — fewer steps means shorter chain (changes task if D < 24)
3. **Increase op size** — larger batch_size or D_HIDDEN to amortize threading overhead (locked by config)
4. **torch.compile** — fuse ops within each sequential step (doesn't change depth, reduces constant factor)

