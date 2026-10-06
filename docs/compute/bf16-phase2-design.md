# Phase 2 design note — chunked tensor-core recurrence and a flash-style bf16 read

> Written at the end of Phase 1 (bf16 mixed precision, `precision=bf16`,
> `docs/compute/bf16-parity-gate.md`). Phase 2 starts only if the Phase 1 gate
> passes; this note freezes its plan and is revised only by that outcome.

## 1. What Phase 1 measured

On one RTX 4090 with the 29M recipe (width 576, 10 layers, context 384, batch
16, 433.8M tokens per run), the same binary with and without `precision=bf16`:

| arm | tokens/s (two runs) |
| --- | --- |
| f32 (TF32 matmuls) | 98,410 / 98,294 |
| bf16 | 120,103 / 119,781 |

≈ **1.22×**. The gain is bounded by the step's composition, not by the GEMMs:
Phase 1's own profile of the f32 step attributes ~14% to matmul, ~18% to the
quaternion recurrence scan, ~14% to the fused read's L2 kernels, with the rest
in RMSNorm/SwiGLU/cross-entropy, memsets and launches. Phase 1's bf16 storage
halves the *bytes* those kernels move, but their arithmetic is still scalar f32
with one to four lanes per thread and no tensor-core use at all. At the 96M rung
(width 1024, 14 layers) the matmul share is larger and Phase 1's ratio should
rise; Phase 2 must measure it.

Phase 1 also fixed the numerical boundary that Phase 2 inherits: quaternion
products and norms, RMSNorm statistics, the recurrence's carried state, drive,
transition and `keep`, every read score, probability, lift, excess and distance,
the loss and the optimiser are f32/f64 and must stay so. Only storage and the
matmul operands are bf16.

## 2. Phase 2a — chunked tensor-core recurrence

### 2.1 The mathematics

One lane's recurrence is

```
h_t = q_t ⊗ h_{t-1} + keep_t · c_t        c_t = b + Σ_{s<4} tap_s ⊙ a_{t-s}
```

with `q_t` a unit quaternion (S3), `⊗` the Hamilton product and `keep_t` the
decay factor. [`recurrence_core_fwd`] runs it serially: one thread per
(window, lane) walks `t = 0..T`. At T = 384 that is 384 dependent steps of a
4-component product — the longest chain in the model.

`⊗` is associative, so a chunk of `k` positions has one transition
`Q = q_{t} ⊗ … ⊗ q_{t-k+1}` and the scan composes chunks instead of positions:

```
H_{i+1} = Q_{tile i} ⊗ H_i + C_{tile i},     C = the chunk's drive folded through its own transitions
```

That is a two-level scan: within a tile (all lanes in parallel), then across
tiles (O(T/k) dependent steps instead of T). The drive is a 4-tap convolution
over the width and is a small GEMM: `[B·T, 4] × [4, width]` shifting the
activation rows, run in bf16 with f32 accumulation. The elementwise output gate
(`held · gelu(branch)`) stays elementwise.

### 2.2 What Phase 2a must not break

- The composed chunk transition is a product of up to `k` unit quaternions; it
  is composed and renormalized in **f32** (Phase 1 measured that the parity of
  the transport needs f32), and the carried state itself stays f32.
- The association order changes the f32 rounding. The chunked path is therefore
  a *new* op with its own tolerance, not a claimed bit-identical rewrite of the
  serial scan; where the tile size is 1 it must reproduce the serial scan
  exactly, and that is a test.
- The backward recomputes the chunk states instead of storing them, as
  `CudaRecurrenceKernels::Split` already does, so the memory win is real.

### 2.3 Interfaces and tests

- New kernels `recurrence_chunk_prep`, `recurrence_chunk_scan`, `recurrence_chunk_out`
  and their backward, selected by `CudaRecurrenceKernels::Chunked` beside the
  existing `Single`/`Split` (same `set_cuda_recurrence_kernels` /
  `UOR_R4_CUDA_RECURRENCE` switch). `RecurrenceCore`'s signature does not change.
- Tests: (i) tile size 1 equals the serial scan; (ii) chunked bf16 against the
  Phase 1 f32 scan with stated tolerances on random inputs; (iii) drift over a
  4,096-position sequence against the serial f32 scan; (iv) the 1,000-step
  finiteness test Phase 1 added.
- Gate: the Phase 1 parity rule on the 29M recipe, plus a tokens/s comparison
  on the 4090 and, if the rung is available, the 96M shape.

## 3. Phase 2b — flash-style bf16 read

### 3.1 Current cost

The fused read materializes, per (batch, head): the score matrix `[T, T]`, the
softmax probabilities, the f64 excess matrix, the f64 `ds`, the f32 `dp` and
`inner_grad`, and the per-(index, j) `key_self` — six `T²` buffers, written and
read by 10 kernels forward and 12 backward. At T = 384, one index's score
matrix is 295 KB in bf16 but the *passes* over it, each with its own launch, are
the cost.

### 3.2 Plan

- Tile over `(t, j)`: compute each tile's scores with bf16 tensor-core GEMMs
  (query/key are already bf16 storage), carry the online softmax (running max
  and running sum) as flash attention does, and accumulate the value read in the
  same pass. The `T²` matrices are never stored.
- The geometric scores are not plain dot products: Lorentz and L2 need the
  per-row lifts `sqrt(1+|x|²)`/`|x|²` in f64 (Phase 1 keeps them), the age table
  and the NoRead logit are per-row scalars, and `beta`/`offset` are per-head.
  All of these are available before the tiles, so the tiling is the same
  dataflow with a per-tile fold of the excess.
- Backward recomputes the tile scores from the stored per-row `(max, sum)` (no
  `ds`/`dp`/`inner_grad` matrices) and scatters the query/key/value gradients
  with the same tiling.
- The online softmax reorders the row's additions. That is a *stated* change:
  the current kernels already sum the row in a warp order, and the flash
  variant's order is different again. The test states the reordering and
  measures it against the Phase 1 f32 read on identical inputs, like the
  existing tiled-vs-ascending test does.

### 3.3 Risks and unknowns

- The flock selection (`select=`) prunes sources by score before the softmax; it
  needs the full row's scores, so a selected read either keeps a score pass or
  the tiling must run twice. Phase 1 refuses bf16 with a selection for exactly
  this reason; Phase 2 may keep that refusal.
- The NoRead slot and the age table break the "tile-local" property; both are
  cheap enough to fold per row.
- The Lorentz distance's f64 `arcosh` is per (t, j): a tile computes it in f64
  before the bf16 accumulate.

## 4. Order of work and gates

> **Revision, 5 October 2026 (measured).** Phase 2a was implemented, tested and
> measured before Phase 2b started: it is parity-clean but **speed-neutral**
> (+0.2% at 29M, −1.2% at ~96M on an RTX 5090). The `nsys` kernel-family
> profile in [`bf16-step-profile-2026-10-05.md`](bf16-step-profile-2026-10-05.md)
> shows why: the recurrence is **8.3%** of a 29M bf16 step (the serial carry
> ~2%), while the **fused read is 34.5%** (its backward alone 25.6%) and
> RMSNorm is 16.6%. The order below is therefore reversed, and Phase 2a is
> recorded as a negative result rather than a shipped path.

1. **Phase 2b first (the read).** One third of the step is the read and three
   quarters of that is its backward; the flash-style dataflow (tiles, online
   softmax, recomputed scores, no `ds`/`dp`/`inner_grad`/`excess`/`key_self`
   matrices) is where the remaining step time is.
2. **RMSNorm second** (16.6%; `rms_norm_bwd_dx` alone 9.2%): one pass for the
   row statistics shared by `dx` and `dw` instead of a `row_r` buffer read back
   by a second kernel.
3. **Phase 2a (chunked recurrence) last, and only on evidence**: the kernel is
   8.3% of the step, so a rewrite can win at most a few percent; revisit it if
   the profile at a longer context or larger batch shows the carry growing.
4. Each phase carries its own parity gate on the 29M recipe under the Phase 1
   rule, its own speed measurement, and its own PR. `precision=f32` and the
   existing kernels stay the default until each gate passes. A phase that
   measures neutral or negative is recorded as such and does not ship as a
   default (Phase 2a's record is above).
4. The 96M-rung speed measurement (where the matmul share is larger) is part of
   Phase 2's report; Phase 1 reported the 29M rung only.

## 5. Out of scope for Phase 2

- The D11 serving path: saved models stay f32 and serving is the native
  integer/table runtime; nothing here changes it.
- Loss scaling: Phase 1's gate ran bf16 without it and stayed finite; if a gate
  ever shows nonfinite activations, the mitigation is a stated loss scale, not a
  silent one.
- Any change to the read's scoring semantics: the same scores, the same
  per-head parameters, the same tolerances.
