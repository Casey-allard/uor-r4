# Phase 1 report — bf16 mixed precision in the geometric-stack trainer

> Phase 1 of the bf16 / tensor-core rewrite: `precision=bf16`, opt-in, with the
> frozen parity gate of [`bf16-parity-gate.md`](bf16-parity-gate.md) deciding
> adoption. Branch `codex/bf16-phase1`, PR #1743.
>
> **Outcome: the gate passes on both axes and both seeds — adopt bf16
> (1.217× at the 29M rung on a 4090, dev NLL within 0.0004 nats of f32, D19
> sessions within 10 of 1,075 turns).**

## 1. What landed

`precision=f32|bf16` (default `f32`) on `geometric-stack train` and
`geometric-stack dialogue-train`.

**bf16.** The trunk's activations and the operands of every matrix product are
bf16. Candle's CUDA path then runs `cublasGemmStridedBatchedEx` with
`CUDA_R_16BF` inputs, `CUBLAS_COMPUTE_32F` accumulation (not
`..._FAST_16BF`) and `CUBLAS_GEMM_DEFAULT_TENSOR_OP`, and the result is written
as a bf16 activation — tensor cores where the device has them, f32 in the
accumulator.

**Still f32 (the islands).** Every parameter and its gradient, the Adam moments,
the whole optimiser step and the gradient squared norm; the loss's f64
log-sum-exp and the loss value; evaluation (dev/final NLL) and sampling; the
saved model and the D11 export. Inside the kernels: RMSNorm statistics (f64),
every read score, probability, lift, excess and distance (f32/f64), the
recurrence's carried state, drive, transition and `keep`, the pointer's
attention scratch, and — deliberately — the unit-quaternion transport products
and norms, which are computed in f32 after reading the transition from bf16
storage. `Precision`'s doc comment and `cuda_ops_bf16.rs`'s module comment name
each of these where it lives.

**Kernels.** The custom kernels are not rewritten: the same CUDA C source is
compiled twice by NVRTC (the bf16 build defines `UOR_STORAGE_BF16`), and in that
build `act_t` is a 16-bit bf16 storage whose loads and stores convert
(round-to-nearest-even) while every accumulation stays f32/f64. The f32 build's
`act_to_f`/`act_from_f` are the identity and its `ld4_act`/`st4_act` call
`ld4`/`st4`, so its machine code is unchanged. `cuda_ops_bf16.rs` (1,575 lines)
mirrors `cuda_ops.rs` op for op, launch for launch; the f32 module gains only
the dtype dispatchers.

**Coverage is refused up front** (a clear error, not a silent host fallback):
`precision=bf16` with a non-CUDA device, a transformer arch, `qat`, a transport
snap, a flock selection, or a product-key memory.

## 2. Evidence

### 2.1 f32 is unchanged, bit for bit

The pre-change binary (`6bab0baa`, built on the pod) and this branch, the same
60-step 29M run (seed 1, `tf32=true`):

| quantity | before | after |
| --- | --- | --- |
| `train_loss` | 6.841319410006205 | 6.841319410006205 |
| `grad_norm` | 2.3637522330593814 | 2.3637522330593814 |
| dev NLL (8 windows) | 4.98604649385311 | 4.98604649385311 |

The 22 pre-existing `cuda_stack_ops_parity` tests still pass (the pod also ran
the throughput test alone: 87.5× SwiGLU, 9.0× RMSNorm, 15.1× cross-entropy
against the CPU reference).

### 2.2 Static audits of the bf16 mirror

- **Launch arguments.** Every kernel invocation in both modules was compared
  against the CUDA declaration's parameter types: `act_t*` ↔ `Arg::B`, `float*`
  ↔ `Arg::F`/`Arg::f`, `double*` ↔ `Arg::D`/`Arg::d`, `uint*` ↔ `Arg::U`,
  structs by value. **0 mismatches** over all launch sites of both modules.
- **Launch geometry.** Grid and block dimensions are identical to the f32
  module's for every mirrored launch (the one intentional difference, a
  defensive `.max(32)` on the pointer row count, was removed).
- **Conversions.** Every `act_t*` parameter's uses are wrapped in
  `act_to_f`/`act_from_f` (or the vector `ld4_act`/`st4_act`), and no conversion
  is applied to an f32 buffer. Straight-through is a bit copy by construction.

### 2.3 bf16 tests (in `tests/cuda_stack_ops_parity.rs`)

- **Exact oracle.** With bf16-exact inputs (multiples of 2⁻⁷), the arithmetic
  inside the kernels is the same f32/f64 in both builds, so each output must be
  the f32 output rounded once to bf16 — bit for bit. Checked for the quaternion
  scan, the fused read (L2), the recurrence core and cross-entropy (whose f64
  loss must be *identical*). A missing or wrong conversion cannot pass this.
- **Suite result:** `cargo test --release -p uor-r4-training --features cuda
  --test cuda_stack_ops_parity` on the pod with `UOR_REQUIRE_CUDA=1`:
  **31 passed, 0 failed, 1 ignored** (the 22 pre-existing tests plus the 9 new
  bf16 ones).
- **Per-op tolerances** (bf16 storage vs f32 storage, same inputs, CUDA both
  sides; stated as `abs + rel·|f32|`): straight-through 1e-4 + 2e-3; quaternion
  scan 5e-3 + 5e-3 (the scan's state is a sum of bf16-rounded drives, so a
  near-cancelling step has a tiny reference value and still carries the input
  scale's rounding — measured 6.3e-3 on the pod, hence an absolute term at the
  input scale); fused read (Dot/Lorentz/L2) 1e-3 + 6e-3 (measured 2.4–2.8e-3);
  recurrence core
  (rotation on/off, both kernel paths) 3e-3 + 8e-3 (measured 3.3–3.6e-4);
  cross-entropy 1e-3 + 5e-3 of the loss (measured 4.2e-4); pointer mixture
  (Dot/Lorentz) 2e-3 + 1e-2 (measured 2.6e-4); a whole small geometric model's
  logits 5e-2 + 2e-2 and its loss 5e-2 + 2e-2 (measured 3.8e-3 logits, 2.9e-5
  loss).
- **1,000-step finiteness.** A small model trained 1,000 steps in bf16: every
  loss and gradient norm finite, every parameter finite, the loss strictly
  reduced, and within 0.2 of the f32 arm's final loss on the same windows.

## 3. Speed (RTX 4090)

**29M recipe, full runs** (70,609 steps, 433.8M tokens, one binary, one process
per GPU; `tokens_per_second` from each run's own report):

| arm | seed | tokens/s |
| --- | --- | --- |
| f32 | 1 | 98,552 |
| f32 | 2 | 97,370 |
| bf16 | 1 | 121,530 |
| bf16 | 2 | 116,917 |

Median **f32 97,961 → bf16 119,224 tok/s: 1.217×**. A 300-step alternating
measurement on the same pod gave the same ratio (1.220× / 1.219×), so the win is
not a warm-up artifact.

**~96M shape** (width 1024, heads 16, layers 14, context 384, batch 32 on one
4090, 60 steps, seed 3): f32 **41,994** tok/s → bf16 **47,854** tok/s → **1.14×**.
Note the shape differs from the ladder's 96M run, which splits batch 32 over two
GPUs (`data_parallel=2`); this is a single-GPU probe, not a ladder result.

**No A100 or H100 was available** in the owner's account for this phase (both
authorized pods are 2× RTX 4090). The A100 profile that motivated the work —
matmul ~14% of a step, tensor cores barely used — is unchanged and untested
here; the 4090 numbers above are the only measured ones, and a bf16 run on an
A100/H100 is the first thing Phase 2 should measure when one is available.

Why only 1.14–1.22×: at these shapes the matmul share of a step is ~14%, so even
an infinitely fast GEMM cannot give more. The recurrence scan (~18%) and the
fused read (~14%) do not use tensor cores at all — Phase 1 only halves the bytes
they move, and their scalar f32 arithmetic dominates. That is the case
[`bf16-phase2-design.md`](bf16-phase2-design.md) makes for the chunked
recurrence and the flash-style read.

## 4. Parity gate

Frozen rule: [`bf16-parity-gate.md`](bf16-parity-gate.md). Four 29M Arm A base
runs (f32/bf16 × seeds 1, 2; 70,609 steps; 433.8M tokens each; one binary),
then the Arm C fine-tune from each and a D19 `log_recall=off` session of each
fine-tune.

**Base runs (final dev NLL, f32 scoring, 512 windows).**

| arm | seed | steps | final dev NLL | tokens/s |
| --- | --- | --- | --- | --- |
| f32 | 1 | 70,609 | 1.28536302 | 98,552 |
| f32 | 2 | 70,609 | 1.28911277 | 97,370 |
| bf16 | 1 | 70,609 | 1.28571284 | 121,530 |
| bf16 | 2 | 70,609 | 1.28920389 | 116,917 |

- f32 seed spread: **0.00374975** nats.
- seed 1: |bf16 − f32| = **0.00034982** nats (0.09× the spread).
- seed 2: |bf16 − f32| = **0.00009112** nats (0.02× the spread).
- Both are also far below the 0.01-nat absolute bound. **NLL axis: PASS.**
- Speed at full length: median bf16 119,223 tok/s vs median f32 97,961 tok/s →
  **1.217×**, matching the 300-step measurement (1.220×).

**Fine-tunes** (Arm C, 4,000 steps each, `init=` the matching base): all four
completed at 4,000 steps, two per GPU, ≈4 min each.

**D19 sessions (`log_recall=off`) of each fine-tune** (1,075 scored turns):

| arm | seed | scored | of |
| --- | --- | --- | --- |
| f32 | 1 | 843 | 1,075 |
| f32 | 2 | 855 | 1,075 |
| bf16 | 1 | 853 | 1,075 |
| bf16 | 2 | 865 | 1,075 |

- f32 seed noise: 12 turns; the frozen floor is 12; allowed difference 12.
- seed 1: |bf16 − f32| = **10** turns.
- seed 2: |bf16 − f32| = **10** turns.
- Both bf16 sessions score *above* their f32 counterpart by 10 turns, i.e. the
  difference is inside the seed noise and in the direction of the arm that is
  1.2× faster. **Session axis: PASS.**

**Frozen decision: ADOPT bf16** — both axes pass for both seeds. The tabulator's
complete output is archived with the runs (see below).

**The delivered revision reproduces the gate binary.** After the gate, the
branch's head was checked out on the pod (`git checkout -f` of the pushed head,
clean tree), rebuilt, and compared against the archived gate binary
(`gate-binary.sha256`, SHA-256
`8e3889391d34b75bef21e05157e80cf7e55c37d76135782098c0c129f04fe8de`) on the same
60-step run in both arms: `train_loss`, `grad_norm` and dev NLL are identical to
the last digit in both (f32 6.87116785844167 / 2.5489016467485293 /
4.993935182165907; bf16 6.870017512639364 / 2.6023508887764213 /
4.996839917106195), and the op suite reports 31 passed / 0 failed at the same
head. So the gate's numbers are the numbers of the delivered revision; the
only source changes after the gate were test-side (fixtures and tolerances) and
two public wrappers' dtype validation, and they are covered by this
reproduction.

**Archived artifacts** (pod volume, `/workspace/uor-r4/bf16-gate/`):
`bf16-gate-runs.tar.gz` (860 MB, every sealed run root without optimizer
checkpoints, md5 in `MD5SUMS`), `decision.txt`, `binaries.sha256`,
`gate-binary-geometric-stack` + `gate-binary.sha256`, `parity-suite.log`
(31 tests), `timing-29m.log`, `timing-96m-{f32,bf16}.log`,
`bit-identity-f32-{base,new}.log`, `headcheck-{f32,bf16}.log` (the delivered
revision's reproduction of the gate binary), `parity-suite.log` (31 tests), and
`bf16-phase1-src.tar.gz` (a source snapshot taken when the gate ended; the
delivered revision is the branch head).


## 5. Cost

| item | amount |
| --- | --- |
| 2 × RTX 4090 pod (`$1.48/h`), gate incl. builds and exploration | ≈ 3.5 h ≈ **$5.2** |
| laptop | edits only, no model build or run |
| new storage | run roots on the pod volume (≈1.2 GB), no new laptop storage |

## 6. Limitations and next steps

- Phase 1's speed win is bounded by a step whose non-matmul kernels are scalar
  f32; Phase 2 addresses exactly that (design note above).
- `precision=bf16` covers the geometric float arms. Product-key memory, the
  transformer control arm, RoPE/selection reads and QAT runs are refused rather
  than silently approximated; each would need its own bf16 kernels.
- Evaluation is always f32. A bf16 *inference* path would be a separate change
  with its own parity question (and the served model is f32 by D11 anyway).
- The gate's downstream criterion uses the D19 session of a fine-tune, which is
  the mission-relevant readout; the session itself still runs f32 on a saved f32
  model.
