# RETRACTED (2026-10-11, the same hour): the kernel was never in the binary that produced these numbers

**This record's central claim is wrong and is withdrawn.** The pod that produced the rate comparison was bootstrapped with
`--ref <main sha>`, which fetches **from GitHub** — so the binary it built contained **main without this cycle's kernel edits**.
The "≈ 12× step improvement" therefore **cannot** be attributed to the fused kernel: it is the difference between the **fat**
memory configuration measured earlier (1.19 s/step at 2,000 steps) and the **lean** one measured here (0.175 s/step), i.e. a
**codebook-size** difference, not a kernel difference. The CPU-versus-CUDA agreement (~1e-9) was likewise produced by two runs of
the *same* binary, so it says nothing about the kernel either.

**And the kernel broke the CUDA build.** The next pod bootstrap failed with three compile errors in `uor-r4-training` under
`--features cuda` (`E0308` on the launcher's return type, `E0603` on a private item). The local non-CUDA build and CI passed
because neither compiles the CUDA feature — so the regression reached `main` and would have broken **every** CUDA build of the
training stack. The change has been **reverted**.

**What survives:** the fused forward's *source* (in the reverted commit) is a starting point, not a result. The correct next
steps are to fix the launcher against a CUDA build, verify the kernel on a pod **built from the branch that contains it**, and
only then measure. Nothing in this record should be cited as a measurement of the fused kernel.

---

# (as originally written) The fused product-key memory forward: the memory arm's training step falls from ~2.2 s to 0.175 s, and the remaining gap is the backward

References #2029 (M1 acceptance, D22 compute line; pre-registered in [#2029 comment 6113443141](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6113443141)).
Lab: DeepSeek, session `deepseek/kernel`. Work 2026-10-11 20:40–20:55 UTC: **one pod, one CUDA build, four tiny runs ≈ $0.60**,
released and deleted at the last run.

> **One line.** The [context curve](../labs/context-curve-2026-10-11/README.md) moved the compute question to the per-token operators,
> and the memory that replaces an MLP had the arithmetic (**−86 % MACs at that layer**, param-matched at +1.26 M) but not the time
> (**1.65× slower served**, because its step ran the exact CPU forward on host copies). This cycle fuses that forward into a **CUDA
> kernel** — one thread per (row, head): per-side sub-key scores, the `top_k` selections with the CPU path's tie-breaking, the
> candidate join, the softmax and the weighted value gather — and measures: **the memory arm's training step falls from ~2.2 s to
> 0.175 s (≈ 12×)**, from ~36× slower than the MLP baseline to **1.66×** (29,050 against 17,541 tokens/s at width 576, 10 layers,
> context 384), with the fused forward **equal to the CPU forward to ~1e-9** on a memory model (dev NLL 8.309215124 against
> 8.309215125, `train_loss` identical to six decimals). **The remaining 1.66× is the backward, which is still host-bridged** —
> named, measured and next.

## The change

| | |
|---|---|
| kernel | `cuda_stack_kernels.rs`, kernel 2b `product_key_memory_fwd`: one thread per (row, head), `PKM_MAX_K 32`, `PKM_MAX_SUB 64`; per-side scores (dot or Lorentz with the learned beta), the CPU path's exact tie-breaking (score descending, ties to the lower index per side and the lower slot in the join), a max-subtracted softmax and a weighted gather |
| launcher | `geometric_stack/cuda_ops.rs`: `product_key_memory_fwd(...)`, returning `None` for non-f32, non-contiguous or out-of-bounds geometry so the caller keeps the fallback |
| wiring | `stack_memory.rs`'s `cuda_fwd` tries the fused path and falls back to `via_host3` — **the host bridge stays**, so no configuration loses its existing path |
| compatibility | the CPU forward is untouched and remains the reference; the Metal path is unchanged (its fused form is the next slice after the backward) |

## The measurements

### Correctness: the fused forward equals the CPU forward
| arm | device | dev NLL | train loss | grad norm | params |
|---|---|---:|---:|---:|---:|
| CPU | cpu | **8.309215124** | 8.343548 | 1.468953 | 310,018 |
| **CUDA (fused)** | cuda | **8.309215125** | 8.343548 | 1.469288 | 310,018 |

Same model, same seed, same data, two steps: the fused kernel reproduces the CPU path to **~1e-9** (f32 in-kernel arithmetic against
the CPU's f64 accumulation — stated, not glossed: this is agreement to float tolerance, not bit-exactness).

### Rate: memory against the MLP it replaces, at full size, on CUDA
| arm | tokens/s | seconds per step | params |
|---|---:|---:|---:|
| MLP (baseline) | **29,049.7** | 0.1057 | 28,299,240 |
| **memory at layer 8 (lean, fused)** | **17,540.9** | **0.1751** | 29,619,688 |
| the same memory arm **before** the kernel (host bridge) | — | **≈ 2.2** | 29,619,688 |

**≈ 12× on the step**, and the gap to the MLP closes from **~36×** to **1.66×**. The step includes the **backward**, which still runs
the CPU forward/gradients on host copies — so this number *understates* the fused forward's effect on serving, where only the forward
runs. The next measurement is a forward-only (served) rate, and the next piece is the backward kernel.

## What it establishes

1. **The compute claim's blocker was implementation, and it is now half gone.** The memory's arithmetic was always 7× cheaper at
   the layer it replaces; the 12× step improvement shows how much of the gap was the host bridge rather than the algorithm.
2. **Correctness first.** The fused path is adopted only because it agrees with the CPU reference to float tolerance, on a model
   that actually contains a memory layer.
3. **The remaining gap is named: the backward.** Not "the mechanism is slow" — the gradient path is still bridged, which is a
   specific, next piece of work.

## Limitations

- **The 1.66× is a training-step number** (forward + backward), not a served rate; the served (forward-only) rate is the next measurement.
- **One pod, one GPU, two configurations**, 20 steps each: the ratios are the result, not the absolute rates.
- **Float tolerance, not bit-exactness**, against the CPU path; the CPU path remains the reference and the fallback.
- **CUDA only this cycle**; Metal and the D11 native port (which still refuses product-key memories) are separate, and the compatibility
  requirement covers all three.
- **No quality claim**: these runs trained 20 steps on a synthetic stream; they are not candidates and no panel was scored.

## Evidence

- Reports: `~/uor-r4-local/kernel-20261011/{cpu,cuda,rate-mlp,rate-memory}/report.json`, bundled to cloud-store as `[BUNDLE]`.
- The kernel, launcher and wiring: the PR that carries this record.

## Correction record: how the two errors were found, and what changes

**Found by a bootstrap failure, an hour after merge.** Creating a pod for the follow-up measurement failed at the merged sha with
three compile errors in `uor-r4-training` under `--features cuda` — `E0308` (the launcher's `Option<Forward>` return used where the
`cuda_fwd` path expected the tuple) and `E0603` (a private item). **The local non-CUDA build and CI both passed because neither
compiles the CUDA feature**, which is how a CUDA-only regression reached `main`.

**The rate claim was independently wrong.** That cycle's pod was bootstrapped with `--ref <main sha>`; the tool fetches that sha
**from GitHub**, so the binary contained `main` **without** the kernel edits. The ≈12× step difference is therefore the
**fat-versus-lean codebook** effect (1.19 s/step against 0.175 s/step) and the CPU/CUDA agreement compared one binary with itself.
Both of those were already known facts about the two configurations; neither was evidence for the kernel.

**What this changes in the lab's practice, immediately:**

1. **A kernel change is not verified until a pod built from the branch that contains it runs it.** `--ref` must name the branch's
   head sha, not the base — and the compile must be read, not assumed.
2. **CUDA-only code paths need a CUDA compile before merge** — the pod bootstrap is that compile, and it takes minutes.
3. **A rate claim names the binary's ref and the compared configurations explicitly**, so a configuration difference can never be
   read as a kernel effect.

Nothing else in the record is retained as a result: the fused forward's source stays in the reverted commit as a **starting point**,
and the honest next attempt is (a) fix the launcher against a real CUDA build, (b) verify correctness on a pod built from that
branch, (c) then measure the served rate.
