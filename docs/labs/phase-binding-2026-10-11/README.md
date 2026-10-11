# Phase binding: the geometric route to exact binding costs 2 M parameters and 13.4× the training time, and reads worse than the learned supervision

References #2029 (M1 acceptance, D22 order 4; mechanism: read binding, [brief](../mechanisms/read-binding.md)).
Lab: DeepSeek, session `deepseek/phasebind`. Runs 2026-10-11 06:49–07:17 UTC. **One pod, two fits ≈ 53 minutes of
GPU, ≈ $2.10**; panel scoring on the laptop CPU.

> **One line.** The read-binding brief names its own next step — *"Phase binding (Step 7d/9, #820; `ReadLineage`) is
> the geometric route to the same goal: values carry a fixed phase or lineage, so binding becomes exact rather than
> learned similarity."* D22 §2 says a KEEP is iterated, so this cycle trained that route on top of the supervision,
> at the same seeds and recipe as the keeper. It **costs 2.0 M more parameters (28.96 → 30.97 M) and 13.4× the
> wall time per training step (0.058 → 0.777 s)**, its bound/competing masses are **identical** (0.898/0.088 against
> the keeper's 0.896/0.090), and on the frozen panel it reads **worse on both matched pairs**: memory 19 → **17** and
> 24 → **20**, wrong-value 12 → **14** and 5 → **14**. **REJECT for this configuration**, keeper unchanged (read
> binding alone); the mechanism stays open (D22 §1).

## Design (pre-registered before compute)

| | |
|---|---|
| arms | **P1** (`data_seed=20261009`) and **P2** (`20261010`) at `read_binding_supervision=0.1` **+ `read_lineage=phase_bind`** (snapped phases) |
| matched baseline | **the keeper's own arms** S1/S2 (read binding, no lineage): same base, store, steps, recipe and seed — so the lineage flag is the only difference |
| everything else | `chat-29m-B-lr5e-4` base, adopted `mix-10` store with its `binding_labels.json[l]` sidecar (`source=recall`), `steps=2000 batch=16 lr=2e-4 warmup=100 protocol=2 context=384 device=cuda` |
| bar | ≥ 5 fewer wrong-value failures than the matched arm on ≥ 2 of 3 seeds, **or** ≥ +4 memory rows in aggregate, with no seed worse by more than the ±3-row noise; BPB within 0.01 |
| declared deviation | the pre-registration specified **three** seeds; **two** were run because the bar needs ≥ 2 of 3 to pass and both pairs came out **worse** than their matched arms, so a third could not change the decision. Named, not silently dropped |

## Results

### Quality, paired against the keeper
| pair | data order | phase binding memory | keeper memory | Δ | phase binding wrong-value | keeper wrong-value | Δ |
|---|---|---:|---:|---:|---:|---:|---:|
| **P1 vs S1** | 20261009 | 17/40 | 19/40 | **−2** | 14 | 12 | **+2** |
| **P2 vs S2** | 20261010 | 20/40 | **24/40** | **−4** | 14 | **5** | **+9** |

**Bar: NOT MET — the configuration is worse on both pairs**, on the failure mode it was supposed to improve.
The mechanism's own objective is unchanged: bound mass **0.898 / 0.897** against the keeper's **0.896 / 0.899**,
competing **0.088 / 0.088** against **0.090 / 0.086**.

### The compute ledger (the practice the [geometry audit](../integration/geometry-audit-2026-10-11/README.md) asks for, started here)
| | keeper (read binding) | **+ phase binding** | change |
|---|---:|---:|---:|
| parameters | 28,958,761 | **30,968,449** | **+2.0 M** (read 4.0 M → 6.0 M) |
| MACs/token counted analytically (context 384) | 30,203,136 | 30,203,136 counted terms | phase maps add parameters and term work the analytic count does not itemise |
| **wall time per training step** | **0.058 s** | **0.777 s** | **13.4×** |
| memory (matched seeds) | 19, 24 | 17, 20 | worse |
| wrong-value failures | 12, 5 | 14, 14 | worse |
| bound / competing mass | 0.896 / 0.090, 0.899 / 0.086 | 0.898 / 0.088, 0.897 / 0.088 | unchanged |

**Why the 13.4×**: the phase-binding read is not one of the configurations with a fused CUDA kernel, so its step runs
the exact CPU forward on host copies (the rule the stack's other kernel-less configurations follow). That is a
measured cost of the geometric route *as implemented*, not a property of the mathematics — a kernel could remove it,
and the quality result would still be the one above.

## What it establishes

1. **The learned supervision is not improved by making binding exact.** The geometric route reached the same bound
   mass and the same competing mass as the learned one and read worse on the panel. On this instrument, "exact rather
   than learned similarity" bought nothing over the supervision that already works — which is the brief's central
   open question answered in the negative for this form of it.
2. **A geometric mechanism measured with its cost beside its quality.** The keeper and this iteration differ by one
   flag, and the ledger says: +2 M parameters, 13.4× the training time, worse answers. Under a quality-only metric
   that trade would have been invisible; this is the first cycle in this lab that reports it.
3. **The keeper stands.** Read binding alone (wrong-value 47 → 30 over 120 rows, memory 57 → 65, three seeds against
   matched controls) remains the line's best result, and the next iteration should go after *what the supervision
   does not reach* rather than after exactness of the same quantity.

## Limitations

- **Two seeds, not three** (declared above), and 40 memory rows (±3 rows of noise) where the brief asks ≥ 120.
- **Phase binding is one form of the geometric route**: `phase_bind_free` (unsnapped phases), `rot` and `carrier` are
  variants that this cycle did not run; the negative is scoped to the snapped `phase_bind` configuration with
  supervision at W=0.1.
- **No served-cost measurement**: the export path's behaviour for a phase-binding model was not checked, so the
  13.4× is a training-time cost, not an inference claim.
- The bound/competing masses remain **train-side**, as in the powered cycle.

## Cost

| | |
|---|---|
| pod | `zs7da1n1i6p2w6`, 2 × RTX 5090: two phase-binding fits at **1,574 s and 1,619 s** (the keeper's were 115 s) ≈ 53 minutes of GPU ≈ $2.10 |
| laptop CPU | two v5 reply sweeps (64 rows, cap 64) |
| external | none beyond the pod; inside the ≤ 4 pods / ≤ $8/h caps |

## Evidence

- Arm reports and models: pod volume `/workspace/uor-r4/deepseek/phasebind-20261011/runs/{P1,P2}/`, pulled and bundled
  to cloud-store as `[BUNDLE]`; the ledger script is `/tmp/ledger.py`'s contents, reproduced in the bundle.
- Paired comparison against the keeper's arms: `readbind2-20261011/score/v5-{S1,S2}` (previous cycle) and
  `phasebind-20261011/score/v5-{P1,P2}`, classified with the same judge-free row checker.
- Pre-registration: [#2029 comment 6106356087](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6106356087).
