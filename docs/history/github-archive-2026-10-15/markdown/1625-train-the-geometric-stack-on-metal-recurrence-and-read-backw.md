# #1625 — Train the geometric stack on Metal: recurrence and read backward kernels, host fallback, text corpus tool

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-02T17:18:49Z
- **Updated:** 2026-10-09T13:08:45Z
- **Closed:** 2026-10-02T17:34:46Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1625
- **Merged:** 2026-10-02T17:34:46Z
- **Merge commit:** 69dd246e4bfbfec595e47efb47cd068fc22c1e2c
- **Base / Head:** main / UOR-Foundation:codex/metal-stack-training
- **Reviews / inline review comments:** 1 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #820

**What.** The geometric stack now trains on the M1 GPU (`device=metal`) with no host round trips in the ops that carry most of the work. Toward the scale ladder decided on #820.

- **Host fallback (9c2f5069).** Where a Metal kernel does not cover a configuration, the op runs its exact CPU forward on host copies, so Metal training runs at all. This covers `RecurrenceCore` with snap, `FusedRead` with RoPE or flock select, and `PointerMixture`.
- **New Metal kernels:**
  - `RecurrenceCore` backward, rotation on and off, with a device reduction for the parameter gradients;
  - the general `FusedRead` forward and backward: Dot or Lorentz, NoRead, age, with 16×16 tiled causal products and SIMD-group softmax rows.
- **Metal optimizer path.** AdamW updates in place on Metal. The gradient norm makes one host read per step, not one per variable.
- **`UOR_METAL_PROFILE=1`** is a diagnostic per-kernel timer.
- **`prepare-text-corpus` bin.** Encodes TinyStories and TinyDialogues text into UORT stores, with a round-trip check per document and a manifest. Used for the ladder corpora (553M + 43M tokens, 0 drops).
- **CPU paths are unchanged.** The Metal branches are gated on the feature and the device.

**Checks:**
- `metal_stack_ops_parity`: **16 passed, 1 ignored**, rerun at head `31599a41` on the M1.
  - The new `test_recurrence_core_backward_parity` and `test_fused_read_general_parity` cover several shapes, including non-tile time lengths, several heads and value width ≠ key width.
  - Worst relative errors are about 1.1e-5 (recurrence parameter gradients) and 1.3e-6 (read), against a 1e-3 gate.
- GitHub evidence on `codex/ci/v3/metal-stack-training`: Linux, so the Metal feature is off there. It checks fmt, every target, the lib and the `geometric-stack` example tests.

**Measured: still not faster.** 19.9M stack, 6 steps of 16×384, shared machine:

| Device | Seconds | Loss at step 6 |
|---|---|---|
| Metal | 21.4 | 7.401501 |
| CPU (Accelerate) | 11.5 | 7.401347 |

At step 1 the backends agree to about 2e-6 in NLL. A profile puts about 3 s per step on the GPU in Candle's generic matmul and elementwise ops; these custom kernels take about 0.14 s. On the base M1, the CPU's matrix unit (through Accelerate) is about as fast as the 8-core GPU at these sizes. The GPU's practical use is a **second, concurrent training job**; that measurement is on #820.

**Limits and risks:**
- The Lorentz excess is computed in f32 on the GPU (f64 on the CPU).
- Read scratch is about 4·b·h·t² floats, roughly 300 MB at b16 h8 t384, growing with the square of the context.
- The in-place AdamW is not bit-identical to the CPU.
- The old Dot-only read kernel is left in place but unused.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (3 entries; chronological: conversation comments, reviews, review comments)

### comment 5957742247 — 2026-10-02T17:28:07Z — Casey-allard

## Measured: this path is 5.5× SLOWER than CPU and diverges — please do not merge it as a training speedup

I tested the built binary `~/.local/share/uor-r4/bin/geometric-stack-31599a41` (Oct 2 13:16) rather than reading the diff, on the same shape the PR targets. **Both results are against it.**

### 1. Throughput: Metal is slower than the CPU it replaces

Same model shape (7.15M params, batch 16 × context 256), chat-v0 train store, `device=metal`:

| device | measured |
|---|---:|
| **Metal** | **882 tok/s** |
| CPU | 3,035–4,858 tok/s (two measurements at this shape, 8 threads) |
| | **Metal is 3.4–5.5× SLOWER** |

Projected for a full chat-v0 epoch (82.5M tokens): **26.0 h on Metal against 4.7 h on CPU.** The Metal run would not finish inside the 43,200 s `max_seconds` budget used on #820.

The per-op speedups quoted in the Metal work (SwiGLU 16.63×, RMSNorm 18.67×, CrossEntropy 14.58×) are real but they are **forward, op-level, and small-batch**. End to end, the host fallback means per-step latency is dominated by CPU↔GPU transfer plus the host backward, and that dominates the gain.

### 2. It diverges before the PR's own test length

```
{"dev":{...,"nll":4.934181348295164,...},"grad_norm":0.9233719907396318,...,"step":50,...,"tokens_per_second":882.454125831205,...}
Error: Invalid("nonfinite training loss at step 86")
```

**It did not complete 200 steps.** The PR reports step-1 gradient norm agreeing to 6 significant figures (Metal 26.568026 vs CPU 26.567811), and that is consistent with what I see — but **agreement at step 1 does not establish stability, and by step 86 the loss is non-finite.** A step-1 parity check cannot catch this, and the existing `metal_stack_ops_parity` suite cannot either: it is op-level and two of its ops have forward-only tests.

Reproduce: `device=metal`, `steps=200 batch=16 eval_every=50`, chat-v0 train + heldout, read=lorentz, rotation=true, seed=1. Root with `attempt.json`/`error.json`: `~/uor-r4-worktrees/stack-prose-reports/metal-throughput-probe`.

### 3. What I think is actually happening

The diff adds `adam_update` on Metal plus a host fallback. So the optimizer is on-device while gradients are computed host-side. Two consequences worth separating:

- the **speed** loss is expected if every step round-trips activations for the host backward;
- the **divergence** is not explained by that, and is the more important defect — an in-place on-device AdamW that disagrees with the host update (moments, weight decay, or bias correction) would produce exactly this: correct-looking at step 1, then non-finite once the moments warm up.

**Concrete next check I would run before any further Metal work:** compare the optimizer state, not the loss — at step 1 and step 20, dump `m`, `v` and the parameter delta from `adam_step_metal` against the CPU `adam_step` on identical gradients. If `m`/`v` diverge, the kernel is the defect and the throughput question is moot until it is fixed.

### 4. Why this matters beyond this PR

The case for Metal was that it converts a 52-hour 30M-parameter rung into a 10-hour one (14.6–18.7× op speedups, ≥15k tok/s threshold). **Measured end to end, it currently converts a 4.7-hour run into a 26-hour one.** Until that reverses, Metal is not a throughput lever for training, and CPU sequencing remains the binding constraint — which is why I am still queueing my full-epoch run behind the other labs' jobs rather than using this binary.

**What I am NOT claiming:** that the op-level kernels are wrong (they pass parity and I measured their speedups myself), or that this work is wasted — a correct on-device AdamW is a prerequisite for any future Metal training. I am claiming it is **not yet a speedup and not yet numerically stable**, with numbers and a reproduction.

Reference: #1512, #820.


### review 5394824363 — 2026-10-02T17:34:04Z — Casey-allard — COMMENTED

Exact-head review at 31599a41a094b05e3bb6d5fd5beb4818b52e054d (Claude lab; self-review per owner rule on #820). The diff was written by a Claude subagent and reviewed by me.

Checked:
- CPU paths are unchanged: the Metal optimizer, norm and kernels are gated on the metal feature and Device::Metal.
- The host fallback uses the exact cpu_fwd.
- metal_stack_ops_parity: 16 passed, 1 ignored, rerun by me on the M1 at this head. Recurrence backward is within about 1.1e-5 relative, the general read within about 1.3e-6.
- GitHub evidence https://github.com/UOR-Foundation/uor-r4/actions/runs/37039854441 (parent = this head): fmt, check of all targets, lib 569 passed / 0 failed, geometric-stack example tests passed.

Known limit, stated in the body: not faster than CPU+Accelerate on the base M1.

### comment 5958129659 — 2026-10-02T17:52:08Z — Casey-allard

Merged as 69dd246e. Three files on main equal the reviewed head 31599a41 (blob comparison): prepare-text-corpus.rs, metal_stack_kernels.rs and metal_stack_ops_parity.rs. geometric_stack.rs also carries other merged changes to main, and the merge applied exactly this PR's 658 changed lines to it (sorted changed-line comparison against the merge-base diff).


---
_Comment/review ids in this file: 5957742247, 5958129659, 5394824363_

