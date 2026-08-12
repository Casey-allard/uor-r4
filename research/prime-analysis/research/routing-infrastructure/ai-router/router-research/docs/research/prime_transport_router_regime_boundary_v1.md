# Prime Transport Router — Regime Boundary v1

## Purpose

Determine whether the same recurrence (D=24, mathematically sequential)
benefits from parallel hardware at larger within-step work sizes.

## Sweep Grid

- D_HIDDEN: [32, 64, 128]
- batch_size: [256, 512, 1024]
- CPU threads: [1, 2, 4]
- Devices: ['cpu', 'mps']
- D = 24 (fixed), 200 steps per config
- torch 2.11.0, macOS-26.4-arm64-arm-64bit, cores=8

## Full Results

| Device | D_HIDDEN | Batch | Threads | Steps/sec | Fwd% | Bwd% | Total(s) |
|--------|----------|-------|---------|-----------|------|------|----------|
| cpu | 32 | 256 | 1 | **101.7** | 60.9% | 35.9% | 1.967 |
| cpu | 32 | 256 | 2 | **91.7** | 59.4% | 37.6% | 2.181 |
| cpu | 32 | 256 | 4 | **89.0** | 57.1% | 39.9% | 2.248 |
| cpu | 64 | 256 | 1 | **80.1** | 58.0% | 39.4% | 2.496 |
| cpu | 64 | 256 | 4 | **74.0** | 51.5% | 46.2% | 2.703 |
| cpu | 128 | 256 | 1 | **72.2** | 63.2% | 34.6% | 2.771 |
| cpu | 64 | 256 | 2 | **71.8** | 53.1% | 44.3% | 2.787 |
| cpu | 128 | 256 | 2 | **63.3** | 52.2% | 45.6% | 3.16 |
| cpu | 128 | 256 | 4 | **60.7** | 46.2% | 51.6% | 3.295 |
| cpu | 32 | 512 | 1 | **58.1** | 64.4% | 33.5% | 3.445 |
| cpu | 32 | 512 | 4 | **53.7** | 55.6% | 42.6% | 3.723 |
| cpu | 32 | 512 | 2 | **53.6** | 59.6% | 38.5% | 3.735 |
| cpu | 64 | 512 | 1 | **52.6** | 65.9% | 32.4% | 3.803 |
| cpu | 64 | 512 | 2 | **47.9** | 56.9% | 41.5% | 4.171 |
| cpu | 64 | 512 | 4 | **45.5** | 50.2% | 48.3% | 4.399 |
| mps | 32 | 256 | — | **43.2** | 44.3% | 47.6% | 4.626 |
| mps | 64 | 512 | — | **40.4** | 44.0% | 47.9% | 4.948 |
| cpu | 128 | 512 | 1 | **40.2** | 67.0% | 31.6% | 4.974 |
| mps | 64 | 256 | — | **39.9** | 43.5% | 47.2% | 5.012 |
| mps | 32 | 512 | — | **39.8** | 43.9% | 46.5% | 5.03 |
| mps | 128 | 256 | — | **39.8** | 43.9% | 48.1% | 5.02 |
| mps | 32 | 1024 | — | **39.5** | 43.9% | 48.2% | 5.063 |
| mps | 128 | 512 | — | **38.9** | 44.5% | 47.8% | 5.135 |
| cpu | 128 | 512 | 2 | **36.4** | 51.7% | 47.1% | 5.49 |
| mps | 64 | 1024 | — | **36.3** | 44.6% | 47.8% | 5.507 |
| mps | 128 | 1024 | — | **34.2** | 45.2% | 48.1% | 5.842 |
| cpu | 128 | 512 | 4 | **33.3** | 45.5% | 53.4% | 5.998 |
| cpu | 32 | 1024 | 1 | **30.9** | 65.2% | 33.5% | 6.462 |
| cpu | 32 | 1024 | 2 | **30.6** | 58.2% | 40.6% | 6.546 |
| cpu | 32 | 1024 | 4 | **29.9** | 53.6% | 45.2% | 6.69 |
| cpu | 64 | 1024 | 1 | **28.8** | 69.1% | 29.8% | 6.944 |
| cpu | 64 | 1024 | 2 | **27.4** | 56.5% | 42.4% | 7.292 |
| cpu | 64 | 1024 | 4 | **25.3** | 49.6% | 49.4% | 7.909 |
| cpu | 128 | 1024 | 1 | **20.1** | 65.0% | 34.0% | 9.957 |
| cpu | 128 | 1024 | 2 | **19.2** | 53.5% | 45.6% | 10.444 |
| cpu | 128 | 1024 | 4 | **18.5** | 47.5% | 51.6% | 10.812 |

## Question 1–2: Does Multi-Threading Help at Larger Regimes?

| D_HIDDEN | Batch | Best Threads | SPS@1thr | SPS@4thr | Multi-thread helps? |
|----------|-------|-------------|----------|----------|---------------------|
| 32 | 256 | 1 | 101.7 | 89.0 | no |
| 32 | 512 | 1 | 58.1 | 53.7 | no |
| 32 | 1024 | 1 | 30.9 | 29.9 | no |
| 64 | 256 | 1 | 80.1 | 74.0 | no |
| 64 | 512 | 1 | 52.6 | 45.5 | no |
| 64 | 1024 | 1 | 28.8 | 25.3 | no |
| 128 | 256 | 1 | 72.2 | 60.7 | no |
| 128 | 512 | 1 | 40.2 | 33.3 | no |
| 128 | 1024 | 1 | 20.1 | 18.5 | no |

`num_threads=1` remains optimal across all tested regimes.
The per-step ops may still be too small even at D_HIDDEN=128, B=1024.

## Question 3: When Does MPS Become Preferable?

| D_HIDDEN | Batch | CPU best (sps) | MPS (sps) | MPS speedup | MPS wins? |
|----------|-------|---------------|-----------|-------------|-----------|
| 32 | 256 | 101.7 (thr=1) | 43.2 | 0.42x | no |
| 32 | 512 | 58.1 (thr=1) | 39.8 | 0.69x | no |
| 32 | 1024 | 30.9 (thr=1) | 39.5 | 1.28x | **YES** |
| 64 | 256 | 80.1 (thr=1) | 39.9 | 0.5x | no |
| 64 | 512 | 52.6 (thr=1) | 40.4 | 0.77x | no |
| 64 | 1024 | 28.8 (thr=1) | 36.3 | 1.26x | **YES** |
| 128 | 256 | 72.2 (thr=1) | 39.8 | 0.55x | no |
| 128 | 512 | 40.2 (thr=1) | 38.9 | 0.97x | no |
| 128 | 1024 | 20.1 (thr=1) | 34.2 | 1.7x | **YES** |

**MPS becomes preferable at:** [(32, 1024), (64, 1024), (128, 1024)]

## Question 4: Does Larger Work Change the Forward/Backward Split?

| D_HIDDEN | Batch | Device | Fwd% | Bwd% |
|----------|-------|--------|------|------|
| 32 | 256 | cpu | 60.9% | 35.9% |
| 32 | 256 | mps | 44.3% | 47.6% |
| 32 | 512 | cpu | 64.4% | 33.5% |
| 32 | 512 | mps | 43.9% | 46.5% |
| 32 | 1024 | cpu | 65.2% | 33.5% |
| 32 | 1024 | mps | 43.9% | 48.2% |
| 64 | 256 | cpu | 58.0% | 39.4% |
| 64 | 256 | mps | 43.5% | 47.2% |
| 64 | 512 | cpu | 65.9% | 32.4% |
| 64 | 512 | mps | 44.0% | 47.9% |
| 64 | 1024 | cpu | 69.1% | 29.8% |
| 64 | 1024 | mps | 44.6% | 47.8% |
| 128 | 256 | cpu | 63.2% | 34.6% |
| 128 | 256 | mps | 43.9% | 48.1% |
| 128 | 512 | cpu | 67.0% | 31.6% |
| 128 | 512 | mps | 44.5% | 47.8% |
| 128 | 1024 | cpu | 65.0% | 34.0% |
| 128 | 1024 | mps | 45.2% | 48.1% |

## Question 5: Best Execution Substrate for Future Experiments

**Best overall config tested:** device=cpu, D_HIDDEN=32, batch=256, threads=1, **101.7 sps**

## Summary

### What is true only for the current tiny regime (D_HIDDEN=32, B=256)

- `num_threads=1` is optimal
- ~100% CPU on one core is expected and unavoidable
- Per-step ops are too small for threading overhead to amortize

### What appears true more generally across regimes

- Even at D_HIDDEN=128, B=1024, the per-step ops remain small enough
  that thread overhead dominates. The D=24 sequential loop limits the
  opportunity for intra-step parallelism.
- MPS becomes viable at larger regimes, amortizing dispatch overhead
- The D=24 sequential recurrence remains the structural depth constraint
  regardless of regime — but within-step throughput CAN scale with hardware
  if per-step work is large enough

