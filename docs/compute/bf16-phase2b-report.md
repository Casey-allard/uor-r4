# bf16 Phase 2b report — the flash-style fused read

> **Scope:** the Phase 2b work item of
> [`bf16-phase2-handoff.md`](bf16-phase2-handoff.md) §3.4 — replace the fused
> geometric read's materialised `T x T` dataflow with a flash-style one. Frozen
> rule: [`bf16-phase2b-gate.md`](bf16-phase2b-gate.md) (Gate 1), superseded on the
> acceptance rule only by [`bf16-phase2b-gate-2.md`](bf16-phase2b-gate-2.md)
> (Gate 2).

## 1. Outcome

**ADOPTED.** The flash read is the default for the CUDA read forward path as of
PR **#1827**; `UOR_R4_CUDA_READ=fused` selects the previous path explicitly. The
fused path is retained, so the comparison stays reproducible.

Implementation delivered as PR **#1809** (`claude/bf16-flash-read`): ~1,939 lines
across `cuda_stack_kernels.rs`, `geometric_stack.rs`,
`geometric_stack/cuda_ops.rs`, `geometric_stack/cuda_ops_bf16.rs` and
`tests/cuda_stack_ops_parity.rs`.

## 2. NLL gate (Gate 1's four runs, read under Gate 2's rule)

Frozen recipe: `geometric-stack train`, `precision=bf16`, 70,609 steps,
`batch=16`, `context=384`, `width=576`, `heads=8`, `layers=10`,
`pattern=rrarrarrar`, `read=l2`, `rotation=true`, `key_shift=false`,
`lr=0.0005`, `device=cuda`, `tf32=true`, `data_parallel=1`. Evaluation f32 in
both arms. One binary for all four runs. 433,821,696 target tokens per run. All
four reached `completed_steps == 70609`; `stopped_early_at_max_seconds` was never
set.

| arm | seed | steps | final.nll | tok/s |
|---|---|---|---|---|
| current | 1 | 70609 | 1.28798791 | 135740 |
| current | 2 | 70609 | 1.28781354 | 137903 |
| **flash** | **1** | 70609 | **1.28536636** | 150229 |
| **flash** | **2** | 70609 | **1.28342084** | 151137 |

```
spread = |NLL(current,1) − NLL(current,2)| = 0.00017437

seed   d_s = flash − current    d_s <= spread    |d_s| < 0.01
1      −0.00262155              ✓                ✓
2      −0.00439270              ✓                ✓
```

**The flash read improves NLL on both seeds** — a mean improvement of
**0.00351 nats**, with both seeds agreeing in sign. Both are well inside the
0.01-nat materiality bound.

**On Gate 1's rule this was reported as `NLL axis: FAIL` / `DO NOT ADOPT`**, because
that rule tests `|d_s| <= spread` and is therefore **symmetric**: it rejects
improvement exactly as it rejects harm. `spread` is also a two-sample range, and
at n=2 it came out **0.00017437 nats — 21× tighter than Phase 1's 0.00374975**.
Gate 2 replaces the test with a one-sided one (`d_s <= spread`, signed) keeping
the two-sided 0.01 materiality bound; see
[`bf16-phase2b-gate-2.md`](bf16-phase2b-gate-2.md) §1 for the full argument.
**Gate 1's freeze clause provides for exactly this: a later change to the
acceptance rule is a new gate with its own record.**

## 3. Speed

### 29M — the shape the 5% floor applies to

Alternating 300-step A/B, 3 rounds, one GPU:

| arm | rounds (tok/s) | median |
|---|---|---|
| current | 110316, 109598, 111196 | **110316** |
| flash | 119118, 116464, 119223 | **119118** |

**ratio 1.0798 (+8.0%)** — clears the frozen floor of 1.05. Consistent with the
independent +6.7% reported on the same shape by `claude/flashread`, and with the
gate runs' own per-run throughputs (flash 150–151k vs current 136–138k tok/s).

### 96M — the ladder shape, reported but not thresholded

Ladder §3.4 shape: `width=1024`, `heads=16`, `layers=14`, `context=384`,
`batch=32`, `data_parallel=1`, 60 steps, both arms, `pattern` left to the default
(`rrarrarrarrarr`, one letter per layer), **95,957,184 parameters** in both arms.

| arm | tok/s | steps |
|---|---|---|
| current | **56,309.10** | 60 |
| flash | **56,178.65** | 60 |

**ratio 0.9977 — the flash read is neutral at 96M (−0.2%).**

**The 29M speed gain does not carry to the 96M shape.** The frozen floor is
stated at 29M, where it passes, so this does not change the verdict — but it is
a real limitation of the result and is reported rather than omitted: the read's
share of the step at 96M is evidently not where the flash dataflow's advantage
lives, and a shape-dependent win is a weaker claim than a general one.

## 4. Op parity

`cargo test --release -p uor-r4-training --features cuda --test
cuda_stack_ops_parity -- --test-threads=1`, run as part of the gate:

```
test result: ok. 37 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out
```

This includes `test_flash_read_parity` over Dot/Lorentz/L2 with and without the
NoRead slot and the age table, at the frozen bound `1e-4 + 1e-3 * |reference|`.

## 5. UNAVAILABLE

Two items the reporting rule asks for were not obtainable, and are marked rather
than approximated:

- **The `nsys` before/after kernel-family profile.** `nsys` (Nsight Systems) is
  **not present on the pod image**
  (`runpod/pytorch:1.0.2-cu1281-torch280-ubuntu2404`): `/opt/nvidia` carries
  `nsight-compute` only, and `apt-get install nsight-systems` finds no such
  package. The method is fixed by
  [`bf16-step-profile-2026-10-05.md`](bf16-step-profile-2026-10-05.md) —
  `nsys profile --trace=cuda` over 5 steps, summarised with
  `nsys stats --report cuda_gpu_kern_sum`. **The "before" profile the comparison
  needs is on trunk** (Phase 2a, PR #1757); only the "after" half is missing, and
  it needs an image with Nsight Systems installed.
- **The parity table's measured maxima.** The suite passes 37/0 at the frozen
  bound, but the per-case maximum deviations were not captured; the test asserts
  the bound rather than reporting the headroom.

## 6. f32/f64 islands the read keeps

Unchanged by Phase 2b, as §3.4 required: the per-row **lifts** (f64), the
**Lorentz excess and distance** (f64 arcosh/sqrt), the **score arithmetic and
softmax accumulation** (f32), and **all parameters**. The flash dataflow removes
the materialised `T x T` buffers — probabilities, the f64 excess, `dp`, the f64
`ds`, `inner_grad` and the per-(index, j) `key_self` — and recomputes per-tile
scores in the backward from the stored per-row `(max, sum)`. It does not change
which quantities are computed in which precision.

## 7. Evidence

- Gate 1 runs: `/workspace/uor-r4/deepseek/p2bgate-20261007/phase2b-gate` —
  `GATE.txt` plus the four run roots (1.2 GB).
- 96M probe: `/workspace/uor-r4/deepseek/p2b-profile-20261007` (1.5 GB).
- Salvaged reference pair from the pod whose GPUs failed mid-run:
  `/workspace/uor-r4/deepseek/p2bgate-20261006/phase2b-gate`.

**Determinism cross-check:** the reference pair reproduced on a second pod to 16
significant figures (1.2879879063628172 / 1.2878135411133271) after the first
pod's GPUs died with `CUDA_ERROR_NO_DEVICE` during the flash pair. The four runs
in §2 are all from the second pod, on one binary.

## 8. Operational notes

Running this gate needed three fixes, all now in the tooling (PR **#1828**):
`bf16-phase2b-gate.sh` now resolves its own `REPO` (a bootstrapped pod checks out
to `/root/build/src-<sha>/`, not `/root/ds-uor-r4`), puts cargo on `PATH` for a
non-interactive shell, and stages the corpora from the step5/tokenizer tars
rather than assuming `$D` is populated. Separately, **four pod draws stalled in
`awaiting_container`** — the host rented but the container never started — before
one came up; deleting and redrawing is the fix, and `uor-pod up` now detects the
stall at 180 s instead of waiting the full 900 s.
