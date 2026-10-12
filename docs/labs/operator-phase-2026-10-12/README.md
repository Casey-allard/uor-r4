# The operator phase: geometric replacements for the gate, the projections and the head

References #2029 (M1); implements the owner decisions of 2026-10-12 recorded at
[#2029 comment 6133330776](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6133330776).
Lab: Kimi (project manager), session `kimi/pm1`, 2026-10-12. This PR is source and record only:
no training, no panel run, no model compute. Every rung below waits for TEST FITNESS.

> **One line.** The owner's two decisions — no exponential-family squashing anywhere (softmax,
> softplus, sigmoid, tanh) and full matmul removal — make every current training configuration
> non-compliant: each stack layer executes its SwiGLU MLP unconditionally
> (`geometric_stack.rs:12241`) and every projection and the head are dense GEMMs. This phase
> builds the three geometric operators that make training legal again, in dependency order, each
> with a matched frozen baseline, each behind a TEST FITNESS verdict. Rung 0 (pre-reg
> [6124773996](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6124773996)) is
> suspended, not withdrawn: it runs unchanged once the stack it trains is compliant.

## OP1 — the geometric gate (SwiGLU replacement)

SwiGLU's `sigmoid(g)·g·u` (`geometric_stack.rs:17633`; CUDA `swiglu_fwd/bwd`,
`cuda_stack_kernels.rs:240-268`) is the last exponential-family term in the trunk. Two
exponential-free candidates:

- **G1, rectified square:** `relu(g)² ⊙ u` — elementwise; the gradient is exact; the cheapest
  possible legal gate; the ReLU² family is the known-stable member of the GLU literature.
- **G2, Hamilton gate:** the gate projection is read as one quaternion per 4-block, unit-normalized
  and icosian-snapped through the existing STE machinery, and the up-projection is *rotated* by it:
  `out = q(g) ⊗ u`. The gate is a rotation of the value, not a squash — the project's own geometry
  doing the gating.

**Control:** the frozen pre-ban SwiGLU artifact's own metrics (no new banned comparator run — the
ban covers comparators). **Bar:** development BPB within 0.02 nats of the frozen matched baseline,
both seeds; tokens/s on the same harness within 5% of the SwiGLU kernel's rate. **Decision
branches:** G1 is the legality fix; G2 is the mechanism test. If G2 is within noise of G1, G2
carries the line (geometric nativeness at no cost). If G2 misses, G1 ships and the negative closes
that *configuration* (D22), not the Hamilton-gate idea.

## OP2 — Hamilton-projection kernels (matmul-free projections)

`quaternion_weights=all` already makes every learned projection quaternion-valued mathematically,
but it executes the 4×4 Hamilton blocks through cuBLAS (`geometric_stack.rs:745-755`) — a dense
GEMM, banned. OP2 is a fused CUDA kernel computing the Hamilton product per 4-block directly (16
scalar multiplies + 12 additions per block, no GEMM call, no cuBLAS), with the f64 CPU reference, a
pod parity test, and f32 + bf16 arms. **Bar:** parity with the existing cuBLAS path inside the
parity harness's tolerance; tokens/s not more than 10% below cuBLAS at the bench shape (a fused
structured kernel against a tuned GEMM — the honest declared cost); then `quaternion_weights=all`
becomes the default for the compliant stack. Serving stays D11: icosian-snapped weights turn the
products into table reads; the export path is a later rung (the config currently refuses export
with quaternion weights, `geometric_stack.rs:3103`).

## OP3 — the matmul-free head

The vocab head is a `d×V` dense GEMM (`geometric_stack.rs:12243`, also 9612/10546/11296/11394).
Two arms:

- **OP3a, factorized product-key head (engineering arm):** the same product-key factorization that
  is the memory's best-measured slot read, as fused structured kernels — low risk, restores a legal
  head first.
- **OP3b, murmuration head (research arm):** every vocabulary token carries a fixed geometric
  address from the prime-indexed H4/icosian codebook (the `e8_memory`/`h4_tables` substrate); the
  score is a quaternion correlation between the final state and each address, computed in one fused
  top-k selection kernel. No learned dense weight matrix. The margin objective (#2255) consumes raw
  scores, and greedy serving needs no normalization — the head never requires a softmax to exist.

**Honest flag, stated prospectively:** the score pass touches V addresses; it is a structured scan
with fixed geometric addresses, not a learned GEMM, and snapped addresses make serving table reads
and adds. If OP3b misses its bar, the negative closes the *fixed-address head* configuration and
OP3a carries the line — the fallback is declared now, not invented after the fact. **Bar (both
arms):** development BPB within 0.02 nats of the frozen baseline head on the same trunk, both
seeds; head tokens/s within 10%; top-1 agreement with the frozen artifact reported as a diagnostic,
not a bar.

## OP4 — rung 0b (owner's work order: first after the operator phase)

The rank-table read is served end-to-end (flock rank reads, +0.0185 BPB vs softmax at 19.93M/2
seeds) but trains through a softmax-derived straight-through gradient (banned) and has no CUDA
kernel (host-only). Rung 0b builds the rank/flock read kernel (kept-list indices + rank table
fused into the tile read) and a rank-consistent gradient, then the margin-trained starling readout
that unblocks Claude's line.

## Standards fixed by this phase

- **Dtype standard:** f32 master weights and Adam moments; bf16 activation storage with f32/f64
  accumulation on CUDA; u16 tokens on disk, u32 at the index tensor; TF32 off; f64 confined to CPU
  references and statistics (basis: the frozen bf16 gate, 1.217× at matched quality).
- **Metric:** tokens/s on the same CUDA harness (`examples/geometric-stack.rs:2587/2655`) reported
  beside every quality number — the owner's standard comparison.
- **Fail-closed CUDA:** a run that would fall back to host compute refuses loudly instead of
  computing on the CPU (D24); the silent `via_host*` path is a defect wherever this phase touches it.
- **Failure review:** every miss gets an external full-context review (evidence-auditor /
  source-falsifier) before it counts; a negative closes a configuration, never a mechanism (D22);
  mechanism stops are owner decisions.

## Sequencing, cost, gates

Order: **OP1 → OP2 → OP3 (a then b) → rung 0 resumed on the compliant trunk → OP4/rung 0b →
margin-trained starling readout.** Training arms run at the established bench config (19.93M,
w512 h8 ctx384) on one 1×4090 pod, one job per GPU, lease/release per the pod rules; kernel work
pays only the parity-gate receipt. Prospective projection: OP1 ≤ $4, OP2 ≤ $1, OP3 ≤ $6, OP4 ≤ $2;
total phase ≤ $13, inside the ≤ 4 pods / ≤ $8/h caps; each run's report records pod, lease, rate
and wall time. No GPU spend before each rung's TEST FITNESS: FIT.
