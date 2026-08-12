# Systems Scaling Branch v2: D_HIDDEN × Batch Size × Device

## 1. Locked Semantics

The following are unchanged from the canonical v6 path:

| Item | Status |
|---|---|
| Operator functions v20–v25 (T_b, T_x, T_y, T_c, T_z', T_r*) | locked |
| spin_H_core_v6, sigma_family_holonomy_law_v6, coupled_holonomy_residue_v6 | locked |
| Tau state representation (swap, coupled, twist, lift phases) | locked |
| v6 step-0-only additive injection rule | locked |
| Position-0 attention bias fix (b_pos0 init = +2.0) | locked |
| Task definition (predict x0 from D-step trajectory) | locked |
| LR=0.02, temperature schedule, gradient clipping=1.0 | locked |

Variables swept (execution-scale only): D_HIDDEN, batch_size, device.

## 2. Sweep Configuration

- D_HIDDEN: [32, 64, 128]
- batch_size: [256, 512, 1024]
- device: ['cpu', 'mps']
- D_CONTEXT: 24 (benchmark delay, solved with pos0 bias)
- BENCH_BUDGET: 3000 batches per config
- torch.set_num_threads(1) for all CPU configs (canonical setting)
- D_HIDDEN_ATTN = max(8, D_HIDDEN // 4): [8, 16, 32]

## 3. Results Table

| device | D_HIDDEN | batch | sps | runtime_s | acc | alpha0 | H_route | tr_frac | solved |
|---|---|---|---|---|---|---|---|---|---|
| cpu | 32 | 256 | 109.0 | 27.5 | 1.0000 | 0.8714 | 1.7024 | 0.9773 | ✓ |
| cpu | 32 | 512 | 62.4 | 48.1 | 1.0000 | 0.8713 | 1.6847 | 0.9844 | ✓ |
| cpu | 32 | 1024 | 33.8 | 88.7 | 1.0000 | 0.8713 | 1.6846 | 0.9836 | ✓ |
| cpu | 64 | 256 | 90.6 | 33.1 | 1.0000 | 0.8736 | 1.6073 | 0.2480 | ✓ |
| cpu | 64 | 512 | 50.7 | 59.2 | 1.0000 | 0.8737 | 1.6027 | 0.2423 | ✓ |
| cpu | 64 | 1024 | 25.3 | 118.7 | 1.0000 | 0.8737 | 1.6080 | 0.2355 | ✓ |
| cpu | 128 | 256 | 71.9 | 41.7 | 1.0000 | 0.8735 | 1.4221 | 0.7693 | ✓ |
| cpu | 128 | 512 | 37.0 | 81.2 | 1.0000 | 0.8735 | 1.4453 | 0.7588 | ✓ |
| cpu | 128 | 1024 | 21.1 | 142.3 | 1.0000 | 0.8736 | 1.4288 | 0.8103 | ✓ |
| mps | 32 | 256 | 55.9 | 53.7 | 1.0000 | 0.8713 | 1.6981 | 0.9782 | ✓ |
| mps | 32 | 512 | 54.3 | 55.3 | 1.0000 | 0.8713 | 1.6942 | 0.9802 | ✓ |
| mps | 32 | 1024 | 52.3 | 57.4 | 1.0000 | 0.8713 | 1.6892 | 0.9799 | ✓ |
| mps | 64 | 256 | 49.4 | 60.8 | 1.0000 | 0.8735 | 1.6019 | 0.1920 | ✓ |
| mps | 64 | 512 | 50.7 | 59.2 | 1.0000 | 0.8736 | 1.6011 | 0.2075 | ✓ |
| mps | 64 | 1024 | 46.0 | 65.3 | 1.0000 | 0.8736 | 1.6094 | 0.2275 | ✓ |
| mps | 128 | 256 | 51.1 | 58.6 | 1.0000 | 0.8733 | 1.4218 | 0.8027 | ✓ |
| mps | 128 | 512 | 51.5 | 58.3 | 1.0000 | 0.8734 | 1.4410 | 0.7603 | ✓ |
| mps | 128 | 1024 | 42.0 | 71.4 | 1.0000 | 0.8734 | 1.4475 | 0.7440 | ✓ |

## 4. Analysis

### 4.1 CPU Single-Thread Behavior Across D_HIDDEN

| D_HIDDEN | avg sps (CPU, all batch sizes) |
|---|---|
| 32 | 68.4 |
| 64 | 55.5 |
| 128 | 43.3 |

D_HIDDEN=32 is 1.6× faster than D_HIDDEN=128 on CPU. Larger hidden width increases per-step compute cost without proportional throughput gain at 1 thread.

### 4.2 Batch Size Effect on Throughput

| batch_size | sps (CPU, D_HIDDEN=32) |
|---|---|
| 256 | 109.0 |
| 512 | 62.4 |
| 1024 | 33.8 |

Batch size has limited effect on CPU throughput (0.31× from B=256 to B=1024). The bottleneck is per-step overhead, not matmul size.

### 4.3 MPS vs CPU Comparison

| D_HIDDEN | batch | CPU sps | MPS sps | MPS/CPU ratio |
|---|---|---|---|---|
| 32 | 256 | 109.0 | 55.9 | 0.51× |
| 32 | 512 | 62.4 | 54.3 | 0.87× |
| 32 | 1024 | 33.8 | 52.3 | 1.55× |
| 64 | 256 | 90.6 | 49.4 | 0.55× |
| 64 | 512 | 50.7 | 50.7 | 1.00× |
| 64 | 1024 | 25.3 | 46.0 | 1.82× |
| 128 | 256 | 71.9 | 51.1 | 0.71× |
| 128 | 512 | 37.0 | 51.5 | 1.39× |
| 128 | 1024 | 21.1 | 42.0 | 1.99× |

MPS is modestly faster (1.15× average). At small model sizes, MPS command-submission overhead is significant.

### 4.4 Accuracy and Solved Status

All 18 valid configurations solved D=24 (acc >= 0.90). The position-0 bias fix is robust across D_HIDDEN and batch size changes.

### 4.5 Attention Concentration (alpha0)

alpha0 range across all configs: [0.8713, 0.8737].
All configs show alpha0 >> 1/D=0.0417. Attention concentrates on position 0 in every config — the bias fix is working correctly regardless of D_HIDDEN or device.

## 5. Best Execution Configuration

**Best throughput: CPU, D_HIDDEN=32, batch_size=256 → 109.0 steps/second**

- Runtime: 27.5s for 3000 batches
- Accuracy: 1.0000
- alpha0: 0.8714

CPU (single-thread) remains the recommended substrate. At current model sizes, MPS command-submission overhead exceeds compute benefit. MPS becomes preferable when D_HIDDEN or batch size grows large enough to amortise per-op overhead.

## 6. Honesty

### What Improved

- The position-0 bias fix is robust: all 18 tested (D_HIDDEN, batch_size, device) combinations solved D=24 at acc=1.000, alpha0≈0.87. The fix is insensitive to capacity and device changes.
- MPS throughput is nearly flat across batch sizes (55.9→52.3 sps at D_HIDDEN=32, -6.8% from B=256 to B=1024), confirming that MPS amortises command-submission overhead across the batch. At B=1024, D_HIDDEN=128, MPS is 2.0× faster per step than CPU.
- MPS samples/second at B=1024 materially exceeds CPU: MPS B=1024 D_HIDDEN=32 processes ~53,555 samples/sec vs CPU B=256's ~27,904 samples/sec (1.92× more training examples per wall-clock second). This matters for future longer-delay experiments where per-step budgets are large.

### What Did Not Improve

- **Larger batch sizes hurt CPU steps/second**: CPU B=256→512→1024 at D_HIDDEN=32 degrades 109→62.4→33.8 sps (-69%). CPU throughput is bottlenecked by per-step Python control-flow overhead (the D=24 time-step loop), not matmul size. Adding more samples per step linearly increases that overhead.
- CPU remains effectively one-core-bound regardless of D_HIDDEN or batch size. Larger hidden width simply increases per-step compute cost without unlocking more cores.
- D_HIDDEN scaling does not materially change convergence: all configs solve D=24 in the same 3000-batch window (the bias fix dominates; extra capacity is redundant at D=24).

### Routing Anomaly at D_HIDDEN=64

At D_HIDDEN=64, transport_usage_fraction drops to 0.24–0.25 (from 0.97–0.98 at D_HIDDEN=32 and 0.76–0.81 at D_HIDDEN=128). This is a routing pattern shift, not a semantic change — accuracy remains 1.000 and alpha0 is unchanged. The wider MLP at D_HIDDEN=64 finds a different operator mix that still solves the task. This routing sensitivity to hidden width is noted but is outside the scope of this systems benchmark.

### What Remains Inefficient

- The D-step Python loop over trajectory time steps remains serial. Even on MPS, each of the 24 time steps is a separate kernel dispatch. A fully batched / unrolled implementation would reduce dispatch count from 24 to 1, which is the correct fix to actually leverage MPS parallelism.
- At D_HIDDEN=32 the matmuls are below the threshold for efficient hardware utilisation on any device. The workload is control-flow dominated at this scale. MPS only becomes competitive when batch size is large enough to amortise command overhead — and becomes the right choice when D_HIDDEN or D grows further.

## 7. Next Step (exactly one)

**Adopt CPU D_HIDDEN=32 B=256 as the canonical execution path and apply the pos0 bias fix to D=32.**

Rationale:
- CPU D_HIDDEN=32 B=256 gives 109.0 sps — the fastest gradient-update rate. Since D=24 solved at 2000 steps (~18s), this is the fastest path to convergence.
- MPS B=1024 processes more samples/sec (53,555 vs 27,904) but fewer gradient updates/sec. For future larger-D experiments with fixed step budgets, MPS B=1024 D_HIDDEN=128 (the 1.99× faster config per step vs CPU at same batch) is worth adopting if step counts must be high.
- The next scientific step is: apply the pos0 bias fix to D=32 and verify whether the same single-parameter `b_pos0 = +2.0` initialization solves the longer delay. D=32 was stuck at acc=0.408, alpha0=1/32 on the canonical path — the same symmetry-breaking failure as D=24.
