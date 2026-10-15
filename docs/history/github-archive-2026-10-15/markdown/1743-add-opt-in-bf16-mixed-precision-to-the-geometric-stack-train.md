# #1743 — Add opt-in bf16 mixed precision to the geometric-stack trainer (Phase 1, gate running)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-05T15:16:02Z
- **Updated:** 2026-10-09T13:17:10Z
- **Closed:** 2026-10-05T19:13:40Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1743
- **Merged:** 2026-10-05T19:13:40Z
- **Merge commit:** 6a918a4349d5f2177a250fa34380c5a3628e387c
- **Base / Head:** main / UOR-Foundation:codex/bf16-phase1
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What

Phase 1 of the bf16 / tensor-core rewrite of the geometric-stack trainer: an opt-in `precision=bf16` for `geometric-stack train` and `dialogue-train`, with a parity gate frozen *before* the first run that decides adoption.

- `precision=bf16` (default `f32`, existing runs unchanged): the trunk's activations and every matmul operand are bf16 — Candle's CUDA bf16 GEMM, i.e. `CUDA_R_16BF` inputs with `CUBLAS_COMPUTE_32F` accumulation on `CUBLAS_GEMM_DEFAULT_TENSOR_OP` tensor cores. **Master weights, Adam moments, the optimiser, the gradient squared norm, the loss (f64 log-sum-exp) and evaluation stay f32; saved models stay f32 (D11 export untouched).**
- The custom CUDA kernels come from the *same* CUDA C source compiled twice by NVRTC (`UOR_STORAGE_BF16`): the bf16 module stores activations as `act_t` (bf16 bits) and converts on load/store, so every kernel still accumulates in f32/f64. `cuda_ops_bf16.rs` mirrors `cuda_ops.rs` op for op and launch for launch; the f32 module and the f32 source are untouched by the flag.
- f32 islands are named in `Precision`'s documentation and in `cuda_ops_bf16.rs`: unit-quaternion transport products and norms, RMSNorm statistics, the recurrence's carried state/drive/transition/`keep`, every read score/probability/lift/excess/distance, cross-entropy's log-sum-exp, the pointer scratch, all parameters, evaluation.
- `precision=bf16` is refused up front with `qat`, a transport snap, a flock selection, a product-key memory, a transformer arch, or a non-CUDA device.

## Gate result: both axes PASS → adopt bf16

Frozen rule `docs/compute/bf16-parity-gate.md`; runner `scripts/pod/bf16-parity-gate.sh`; tabulator `scripts/pod/bf16-parity-gate.py`. Report: **`docs/compute/bf16-phase1-report.md`**.

29M Arm A, 70,609 steps / 433.8M tokens, one binary for both arms:

| arm | seed | final dev NLL | tokens/s |
| --- | --- | --- | --- |
| f32 | 1 | 1.28536302 | 98,552 |
| f32 | 2 | 1.28911277 | 97,370 |
| bf16 | 1 | 1.28571284 | 121,530 |
| bf16 | 2 | 1.28920389 | 116,917 |

- f32 seed spread 0.00374975; |bf16−f32| = 0.00034982 / 0.00009112 → far inside the spread and the 0.01-nat bound. **NLL axis PASS.**
- Speed: **1.217×** (median 97,961 → 119,224 tok/s) at full length; 1.220× in 300-step alternating runs; 1.14× at a ~96M-shape single-GPU probe.
- D19 `log_recall=off` session of each Arm C fine-tune (1,075 turns): f32 843 / 855, bf16 **853 / 865**; |bf16−f32| = 10 turns on both seeds, inside the 12-turn seed noise. **Session axis PASS.**

## Verification

- **f32 unchanged, bit for bit**: pre-change binary vs this branch, same 60-step run — identical `train_loss` 6.841319410006205, `grad_norm` 2.3637522330593814, dev NLL 4.98604649385311.
- **CUDA op suite: 31 passed, 0 failed** with `UOR_REQUIRE_CUDA=1` (22 pre-existing + 9 new bf16 tests). The bf16 section includes an exact oracle (bf16-exact inputs must round exactly once — checked for the scan, the fused read and the recurrence; cross-entropy must be bit-identical), per-op tolerances with the measured values stated, a whole small model's forward/loss, and 1,000 bf16 training steps that must stay finite and track the f32 arm (final loss gap 1.3e-4).
- **Static audits**: 0 argument-kind mismatches over every kernel invocation of both modules against the CUDA declarations; launch geometry identical; every `act_t` use converted on load/store.
- **Delivered revision verified**: rebuilt from a clean clone of the PR head and compared with the archived gate binary on the same 60-step runs — `train_loss`/`grad_norm`/dev NLL identical to the last digit in both arms.

## Artifacts

`/workspace/uor-r4/bf16-gate/`: every sealed run root (860 MB tar without optimizer checkpoints, md5), `decision.txt`, the gate binary + SHA-256, the parity suite log, timing and bit-identity logs.

## Phase 2

`docs/compute/bf16-phase2-design.md`: a chunked tensor-core recurrence (associative Hamilton-product scan, 4-tap drive as a GEMM) and a flash-style bf16 read (online softmax, no `T²` buffers) — Phase 1's 1.2× is bounded by kernels that never touch tensor cores. Each phase gets its own gate; `f32` stays the default until then.

References #820. **Do not merge without the lab's queue/review.**


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 6000685668 — 2026-10-05T18:33:50Z — Casey-allard

**Requesting review and queue (no merge by the author).**

Head `7fcd6eec` (branch `codex/bf16-phase1`). Exact-head validation on the 2×RTX-4090 pod, all of it reproducible from the branch:

- `plugins`-free build at the head + `cargo test --release -p uor-r4-training --features cuda --test cuda_stack_ops_parity` with `UOR_REQUIRE_CUDA=1`: **31 passed, 0 failed, 1 ignored**.
- f32 invariance: the pre-change binary and this branch produce identical `train_loss` / `grad_norm` / dev NLL on the same 60-step 29M run.
- Frozen gate (`docs/compute/bf16-parity-gate.md`) run once, before any merge: NLL axis PASS (|bf16−f32| = 0.00035 / 0.00009 vs the 0.00375 f32 seed spread), D19 `log_recall=off` session axis PASS (10 turns of 1,075 on both seeds, inside the 12-turn noise), **adopt bf16**; 1.217× at the 29M rung, 1.14× at a ~96M-shape probe.
- The delivered revision was rebuilt from a clean checkout on the pod and reproduces the archived gate binary's numbers to the last digit in both arms.

Artifacts: `/workspace/uor-r4/bf16-gate/` (all sealed run roots without optimizer checkpoints + md5, `decision.txt`, gate binary + SHA-256, parity suite log, timing and bit-identity logs). Report `docs/compute/bf16-phase1-report.md`; Phase 2 plan `docs/compute/bf16-phase2-design.md`.

Phase 2 (chunked tensor-core recurrence + flash-style bf16 read) starts only after this lands, per its design note.


### comment 6001698711 — 2026-10-05T19:39:58Z — Casey-allard

Post-merge exact-head check at 44608ba2 on RTX 5090 (Claude lab): op suite **31 passed / 0 failed / 1 ignored** (incl. 9 bf16 tests, first run on sm_120); 96M 300-step dialogue-train from chat-100m2-balp-s1, same binary/data/seed: f32 0.0862 s/step vs **bf16 0.0689 s/step (1.25×)**, dev response NLL 0.09071 vs 0.09171 (within noise). Consistent with the frozen 29M gate. Note: this PR was merged before an exact-head review by a second lab was posted; the evidence above is recorded after the fact.


---
_Comment/review ids in this file: 6000685668, 6001698711_

