# Prime Transport Router — Live Path Verification v1

> **Objective**: Binary determination of whether any graph traversal
> (BFS or equivalent) executes AFTER cache load and BEFORE/DURING training.

---

## 1. Phase Timeline

All timestamps are relative to `CACHE_LOAD_START` (t=0).

| Marker | Relative Time | Absolute perf_counter |
|--------|---------------|----------------------|
| CACHE_LOAD_START | +0.000000s | 916328.218912 |
| CACHE_LOAD_END | +0.386137s | 916328.605049 |
| TRAINING_START | +0.483991s | 916328.702903 |
| TRAINING_END | +1.683633s | 916329.902545 |

| Duration | Value |
|----------|-------|
| Cache load (`torch.load`) | 0.386112s |
| CACHE_LOAD_END → TRAINING_START | 0.097854s |
| Training (50 steps) | 1.199642s |

## 2. Phase Markers

| Marker | Value |
|--------|-------|
| BFS_ENTERED | False |
| RECONCILIATION_ENTERED | False |
| SERIAL_TRAVERSAL_ENTERED | False |

## 3. Functions Executed: CACHE_LOAD_END → TRAINING_START

These are ALL functions executed between the two phase markers.

### 3a. Explicitly instrumented functions

| Function | Python Loop Iters | Wall Time (s) | Note |
|----------|-------------------|---------------|------|
| `convert_onehot_to_angular[TN_oh]` | 4 | 0.077238 | input=(343665, 6, 21) |
| `convert_onehot_to_angular[tau0_oh]` | 4 | 0.015877 | input=(343665, 21) |
| `torch.cat[tau0_hyb]` | 0 | 0.000000 | no loop |
| `RouterAngularHybrid.__init__` | 0 | 0.001297 | model parameter init |

### 3b. sys.settrace captured functions (top 30 by cumulative time)

| Function | Call Count | Loop Iters | Cumulative Time (s) | Serial? |
|----------|-----------|-----------|-------------------|---------|
| `run_router_live_path_verification_v1.py::instrumented_convert_onehot_to_angular` | 2 | 46 | 0.093340 | no |
| `run_router_live_path_verification_v1.py::__init__` | 1 | 0 | 0.001266 | no |
| `module.py::__setattr__` | 12 | 12 | 0.000659 | no |
| `run_router_live_path_verification_v1.py::_emit` | 9 | 0 | 0.000410 | no |
| `module.py::register_parameter` | 12 | 0 | 0.000330 | no |
| `module.py::__getattr__` | 17 | 17 | 0.000244 | no |
| `module.py::register_buffer` | 5 | 5 | 0.000172 | no |
| `run_router_live_path_verification_v1.py::rp` | 7 | 0 | 0.000123 | no |
| `module.py::remove_from` | 12 | 84 | 0.000115 | no |
| `run_router_live_path_verification_v1.py::<genexpr>` | 34 | 0 | 0.000088 | no |
| `parameter.py::__new__` | 12 | 0 | 0.000077 | no |
| `module.py::__init__` | 1 | 0 | 0.000048 | no |
| `run_router_live_path_verification_v1.py::zp` | 4 | 0 | 0.000040 | no |
| `run_router_live_path_verification_v1.py::stop_trace` | 1 | 0 | 0.000000 | no |

## 4. Traversal Analysis

### Loop functions between CACHE_LOAD_END and TRAINING_START

**States in loaded cache:** 343,665
**Serial traversal threshold:** 1,000 Python-level loop iterations

#### `convert_onehot_to_angular` — called twice (TN_oh, tau0_oh)

```python
for s, e, m in PHASE_BLOCKS:   # PHASE_BLOCKS has exactly 4 elements
    k = onehot[..., s:e].argmax(dim=-1).float()  # vectorized over N states
    angle = 2.0 * math.pi * k / float(m)         # vectorized
    out[..., ai]     = torch.cos(angle)           # vectorized
    out[..., ai + 1] = torch.sin(angle)           # vectorized
```

- **Python loop iterations: 4** (constant — `len(PHASE_BLOCKS) = 4`)
- **N_states: 343,665** — processed simultaneously via vectorized PyTorch ops
- **Classification: NOT a serial state traversal**
  - Outer Python loop: 4 iterations (constant)
  - Inner work: fully vectorized batch tensor operations
  - Each `argmax`, `cos`, `sin` processes all N states in a single op

#### `RouterAngularHybrid.__init__`

- No iteration over states
- Only parameter/buffer initialization

## 5. Final Classification

```
POST-CACHE TRAVERSAL: NONE
```

> No serial traversal detected. The path from cache load to training
> executes only constant-iteration loops (4 iterations of PHASE_BLOCKS)
> with all per-state work done via vectorized PyTorch tensor operations.
> BFS is fully eliminated in the cache-load path.

---

## Appendix: Full Phase Event Log

```
+0.000000  [CACHE_LOAD_START] None  
+0.386137  [CACHE_LOAD_END] None  load_time_s=0.3861  n_states=343665  TN_shape=(343665, 6, 21)  TR_shape=(343665, 6)
+0.386173  [BFS_ENTERED] False  
+0.386180  [RECONCILIATION_ENTERED] False  
+0.386183  [SERIAL_TRAVERSAL_ENTERED] False  
+0.386218  [FN_ENTER] None  fn=prepare_hybrid_tables
+0.386251  [FN_ENTER] None  fn=convert_onehot_to_angular[TN_oh]  input_shape=(343665, 6, 21)
+0.463556  [FN_LOOP] None  fn=convert_onehot_to_angular[TN_oh]  iters=4  wall_s=0.077238  input_shape=(343665, 6, 21)  serial=False
+0.464037  [FN_ENTER] None  fn=convert_onehot_to_angular[tau0_oh]  input_shape=(343665, 21)
+0.479982  [FN_LOOP] None  fn=convert_onehot_to_angular[tau0_oh]  iters=4  wall_s=0.015877  input_shape=(343665, 21)  serial=False
+0.482547  [FN_LOOP] None  fn=torch.cat[tau0_hyb]  iters=0  wall_s=0  input_shape=(343665, 8)  serial=False
+0.482620  [FN_EXIT] None  fn=prepare_hybrid_tables  wall_s=0.096418
+0.482645  [FN_ENTER] None  fn=RouterAngularHybrid.__init__
+0.483946  [FN_EXIT] None  fn=RouterAngularHybrid.__init__  wall_s=0.001297
+0.483991  [TRAINING_START] None  steps=50
+1.683633  [TRAINING_END] None  wall_s=0.5825
```
