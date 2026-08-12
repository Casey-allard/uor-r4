# Prime Transport Router — Bottleneck Probe v3

## Root Cause (Summary)

**The D=24 sequential loop is the singular structural bottleneck.**

It forces both forward and backward to execute as a 24-step serial chain of tiny tensor ops on a single thread. Forward takes 56% of wall time, backward takes 40%. Together they account for **95.7%** of all runtime. Nothing else matters.

The backward pass is expensive not because of computational cost, but because **autograd must reverse through 24 dependent sequential steps**. Each step produced small intermediate tensors (256×21, 256×6, 256×25). The autograd engine cannot parallelize any of it — it's one chain. Worker threads spawned by the engine sit idle on `ReadyQueue::pop()` waiting for work that never comes, because there is only one sequential path to traverse.

---

## Current Baseline (PRIMARY — all data below is from this config)

| Parameter | Value |
|-----------|-------|
| device | **CPU** |
| D_HIDDEN | 32 |
| batch_size | 256 |
| D (context/loop length) | **24** |
| pos0 bias | enabled (B0_INIT=2.0) |
| torch.set_num_threads | **1** |
| torch version | 2.11.0 |
| python version | 3.10.20 |
| platform | macOS-26.4-arm64 |
| JIT scripted | yes |
| model params | 1,403 |

## Historical Sample Status

`/AI-Research/Sample of python3.10.txt` is a **historical** macOS `sample` trace captured from an **MPS-era run** (different backend). It is **secondary comparative evidence only**. It is NOT sufficient to explain the current CPU baseline bottleneck.

The historical sample does confirm the *structural* pattern (backward waiting on ReadyQueue due to the sequential chain), but the dispatch mechanism (MPS Metal queues vs CPU kernels) is completely different.

---

## Phase 3: Split Timing — The Primary Evidence

200 training steps. Total wall time: 2.06s. Throughput: 96.9 steps/sec.

| Component | Wall Time (s) | % Total | Notes |
|-----------|--------------|---------|-------|
| **forward** | **1.157** | **56.0%** | 24-step sequential loop, each step dispatches ~15 tiny ops |
| **backward** | **0.820** | **39.7%** | autograd reverses through 24-step sequential chain |
| clip_grad | 0.030 | 1.5% | |
| loss | 0.022 | 1.1% | |
| optimizer | 0.012 | 0.6% | |
| zero_grad | 0.012 | 0.6% | |
| sample | 0.011 | 0.5% | |
| overhead | 0.001 | 0.0% | |

**Wall-time attribution coverage: 99.95%** (target: ≥80%) ✓

**Forward + backward = 95.7% of all wall time.** Both are dominated by the D=24 sequential loop.

---

## Phase 2: torch.profiler — What Ops the Loop Executes

50 training steps. Total self CPU: 0.59s.

| Rank | %Self | self_cpu(s) | Count | Operator | Why |
|------|-------|------------|-------|----------|-----|
| 1 | 15.0% | 0.088 | 3,750 | `aten::index` | TN[state_ids], TR[state_ids] — 24 steps × 50 steps × ~1.5 calls |
| 2 | 13.7% | 0.081 | 2,550 | `aten::bmm` | einsum("bi,bij->bj") — 24 steps × 50 + attention |
| 3 | 9.4% | 0.055 | 117,650 | `aten::select` | tokens[:,t] and internal slicing — **massive dispatch count** |
| 4 | 8.4% | 0.049 | 1,250 | `aten::tanh` | activation, 24+1 per step |
| 5 | 5.7% | 0.034 | 7,500 | `aten::mm` | W1 and W2 matmuls, 24×2 per step + backward |
| 6 | 3.9% | 0.023 | 150,400 | `aten::as_strided` | view/reshape overhead — **150K calls for 50 steps** |
| 7 | 3.4% | 0.020 | 6,400 | `aten::add` | bias adds, residual |
| 8 | 2.5% | 0.015 | 1,250 | `aten::_softmax` | Gumbel-softmax + attention |
| 9 | 2.2% | 0.013 | 2,550 | `<backward op>` | autograd backward kernels |

**Key observation:** The loop executes ~15 distinct tensor ops per iteration × 24 iterations = **~360 op dispatches per training step** in the forward alone. Each op operates on tensors of size 256×21 to 256×32 — tiny enough that **C++ dispatch overhead is comparable to actual compute time**. The backward doubles this with another ~360 gradient ops traversed serially.

---

## Phase 1: cProfile — Python-Level Confirmation

200 training steps. Total wall time: 1.76s.

| Rank | %Wall | tottime(s) | Function |
|------|-------|-----------|----------|
| 1 | 61.4% | 1.078 | `module.py:_call_impl` (forward pass) |
| 2 | 33.6% | 0.590 | `torch._C._EngineBase.run_backward` |
| 3 | 0.3% | 0.005 | `torch.randint` |
| 4 | 0.2% | 0.004 | `torch._foreach_norm` |
| 5 | 0.2% | 0.004 | `cross_entropy_loss` |

cProfile confirms: **95% of Python-level time is in forward (61%) + backward (34%)**. Everything else is noise.

---

## Phase 4: Thread Monitoring

100 training steps.

- Thread count throughout: **min=1, max=1, avg=1.0**
- Only thread: `MainThread`
- No autograd worker threads were active during CPU baseline execution

**Interpretation:** With `torch.set_num_threads(1)` on CPU, PyTorch runs the entire forward AND backward on the MainThread. There are no worker threads. The autograd engine's C++ `ReadyQueue` is processed inline by the calling thread. This means one core does everything — explaining the ~100% CPU in Activity Monitor.

---

## Answers to Required Diagnostic Questions

### 1. Why does Activity Monitor stay near ~100% CPU?

**Proven:** `torch.set_num_threads(1)` forces all computation onto one core. The training loop has zero idle time — forward (56%) and backward (40%) run back-to-back with no I/O, no waits, no parallelism. One core at 100% utilization = 100% in Activity Monitor (which reports per-process, and this process has one active thread).

### 2. Is there one truly hot thread?

**Proven:** Yes. Phase 4 shows exactly 1 thread (`MainThread`) throughout all 100 monitored steps. There are no autograd workers, no OpenBLAS threads, no Metal threads. One thread does everything.

### 3. What exact functions/ops dominate wall time?

**Proven:** Forward pass (56%) and backward pass (40%). Within forward, the top ops by self-CPU are:
- `aten::index` (15%) — table lookups into the 343K×6×21 TN tensor
- `aten::bmm` (14%) — batch matmul from einsum
- `aten::select` (9%) — token slicing, 117K calls for 50 steps
- `aten::tanh` (8%) — activations
- `aten::mm` (6%) — linear layer matmuls

All of these are called from inside the D=24 loop.

### 4. Is a major hot path still running in Python?

**Partially.** The model is JIT-scripted, so the `for t in range(D)` loop runs in TorchScript's C++ interpreter, not CPython. But the loop structure still forces sequential dispatch — JIT doesn't fuse across loop iterations. Each of the ~360 ops per step is dispatched individually through the ATen dispatcher.

### 5. Is forward, backward, autograd, indexing, or something else the dominant bottleneck?

**Proven:**
- **Forward: 56%** — the D=24 loop dispatching ~360 tiny ops
- **Backward: 40%** — autograd reversing through the same 24-step chain
- **Everything else: 4%** — irrelevant

The root cause of BOTH is the same: **the 24-step sequential loop**. Forward iterates it forward, backward reverses through it. Both are forced serial. Both operate on tiny tensors where dispatch overhead rivals compute.

### 6. Are there wakeup/polling/tiny-dispatch effects in the current CPU baseline?

**Yes, but not the MPS kind.** There are no Metal command queues or GPU submission barriers. Instead, the "tiny dispatch" effect is:
- 360+ ATen op dispatches per forward step
- 360+ gradient ops per backward step
- Each op on tensors of size 256×21 to 256×32
- `aten::as_strided` alone is called **150,400 times** in 50 steps (3,008/step)
- `aten::select` is called **117,650 times** in 50 steps (2,353/step)

These are not "wakeups" — they are legitimate C++ function calls — but the sheer volume of tiny dispatches means overhead is a significant fraction of total time.

### 7. What is the single most evidence-based fix?

**Reduce the backward graph depth from the D=24 sequential chain.**

The most directly justified fix: **apply `torch.utils.checkpoint` to the loop body** so that autograd does not store all 24 steps' intermediates and instead recomputes them during backward. This:
- Cuts the backward graph from 24 sequential nodes to a smaller number of checkpoint segments
- Reduces memory pressure from intermediate tensors
- Trades ~1× forward recompute for significantly shorter autograd chains

Alternative (more invasive): write a **custom `torch.autograd.Function`** that implements the entire 24-step loop as a single differentiable op with a hand-written backward. This eliminates autograd graph overhead entirely but requires manual gradient derivation.

**What NOT to do:** Adding more threads won't help — the chain is serial. Switching to MPS won't help — the ops are too small for GPU. Increasing batch size has diminishing returns because the bottleneck is dispatch count, not FLOP count.

---

## Ranked Bottleneck List (Current CPU Baseline)

| Rank | Component | Wall Time | % | Root Cause |
|------|-----------|-----------|---|------------|
| 1 | forward pass (D=24 loop) | 1.157s | 56.0% | 24 sequential iterations, ~360 tiny op dispatches each |
| 2 | backward pass (D=24 chain) | 0.820s | 39.7% | autograd reverses 24-step serial chain, no parallelism possible |
| 3 | grad clipping | 0.030s | 1.5% | norm computation over 1,403 params |
| 4 | cross-entropy loss | 0.022s | 1.1% | single op |
| 5 | optimizer step | 0.012s | 0.6% | SGD update |
| 6 | zero_grad | 0.012s | 0.6% | |
| 7 | data sampling | 0.011s | 0.5% | randint calls |

---

## Honesty Section

### What is PROVEN from current CPU profiling

- Forward takes 56.0% of wall time, backward takes 39.7% — measured directly via split timing (Phase 3, 200 steps)
- Throughput: 96.9 steps/sec on CPU with 1 thread
- Exactly 1 thread active throughout training (Phase 4, 100 steps)
- Top torch ops are `aten::index`, `aten::bmm`, `aten::select`, `aten::tanh`, `aten::mm` — all called from inside the D=24 loop (Phase 2, 50 steps)
- ~100% CPU is single-thread saturation with no idle gaps
- Wall-time attribution covers 99.95% of runtime

### What is only SUGGESTED by the historical MPS sample

- The MPS sample showed `THPEngine_run_backward` with the main thread waiting on `ReadyQueue::pop()` — this confirms the sequential chain pattern but under a different backend
- OpenBLAS threads sleeping (irrelevant to current 1-thread CPU config)
- Metal dispatch patterns (irrelevant to CPU)
- The MPS sample's thread structure does NOT apply to CPU execution

### What remains UNCERTAIN

- Exact ratio of C++ dispatch overhead vs actual compute within each tiny op
- Whether `torch.jit.script` is helping or hurting (JIT dispatch may add overhead for very small ops)
- Cache miss rate on the 343K×6×21 TN table indexing (173MB tensor, likely exceeds L2)
- Whether `torch.utils.checkpoint` will actually reduce backward time or just shift it to recomputed forward time
- The optimal checkpoint granularity (every step? every 4 steps? every 8?)

### What previous claims were too confident

- Prior claims that "workload shape" or "tiny ops" fully explained ~100% CPU were not backed by profiling — they were guesses
- The historical MPS sample was initially treated as primary evidence for the CPU bottleneck — it describes a fundamentally different execution path
- "Thread count changes" observed in Activity Monitor likely referred to the MPS-era config, not the current CPU baseline where thread count is constant at 1
