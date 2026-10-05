# bf16 tensor-core rewrite — session handoff (DeepSeek, 5 October 2026)

> **Scope of the session this hands off from:** the bf16 / tensor-core build only
> — Phase 1 (mixed precision, merged) and Phase 2a (chunked recurrence, measured
> and recorded as neutral). It did not run the Phase 2b implementation. Everything
> below is on GitHub and on the shared volume; nothing lives in a chat context.
>
> **For the next session:** read §1–§3, then start §3.4. Do not redo §2.

## 1. Repository state (exact heads)

| What | Where | Head |
| --- | --- | --- |
| Phase 1 bf16 mixed precision — **merged** | `main`, PR [#1743](https://github.com/UOR-Foundation/uor-r4/pull/1743) | merge `6a918a43`, branch `codex/bf16-phase1` `44608ba2` |
| Phase 2a chunked recurrence + step profile — **open PR** | `codex/bf16-phase2a-chunked-recurrence`, PR [#1757](https://github.com/UOR-Foundation/uor-r4/pull/1757) | `b2a14994` |
| Phase 2b flash-read **gate protocol only** | `codex/bf16-phase2b-flash-read` | `9bd02e34` (this handoff adds to it) |
| Shared pod protocol + tool — **open PR, not merged** | `claude/compute-protocol`, PR [#1758](https://github.com/UOR-Foundation/uor-r4/pull/1758) | read `docs/labs/compute.md`, run `scripts/pod/uor-pod` from that branch until merged |

Docs already written (read these first):

- [`docs/compute/bf16-phase1-report.md`](bf16-phase1-report.md) — Phase 1 implementation, evidence, gate result, f32 islands.
- [`docs/compute/bf16-parity-gate.md`](bf16-parity-gate.md) — the Phase 1 frozen rule (the template for every later gate).
- [`docs/compute/bf16-step-profile-2026-10-05.md`](bf16-step-profile-2026-10-05.md) — **the profile that decides what to optimise next.**
- [`docs/compute/bf16-phase2-design.md`](bf16-phase2-design.md) — Phase 2 plan, revised after the profile (read is first, RMSNorm second, recurrence only on evidence).
- [`docs/compute/bf16-phase2b-gate.md`](bf16-phase2b-gate.md) — Phase 2b's frozen gate (recipe, parity, acceptance, ≥5% speed floor).

## 2. Evidence already banked — do not redo

**Phase 1 gate (adopted bf16).** 29M Arm A, 70,609 steps, one binary, two seeds per arm: f32 `1.28536302` / `1.28911277`, bf16 `1.28571284` / `1.28920389`; f32 seed spread 0.00374975, |bf16−f32| = 0.00034982 / 0.00009112; D19 sessions `log_recall=off` 843/855 (f32) vs 853/865 (bf16). **1.217×** at 29M on a 4090 (97,961 → 119,224 tok/s). f32 path bit-identical to pre-change. Op suite 31/31.

**Phase 2a (chunked recurrence): parity clean, speed neutral — do not re-litigate.**
Parity 32/32 on the RTX 5090, max |chunked − serial| ≤ 4.8e-6 (bound 1e-4 + 1e-3 rel), bit-identical for single-tile shapes, deterministic. Speed: 29M bf16 alternating rounds serial 124,117 / 124,174 / 124,446 vs chunked 123,369 / 124,613 / 124,448 tok/s; ~96M shape 60,260 vs 59,522 (bf16). The chunked path stays opt-in (`UOR_R4_CUDA_RECURRENCE=chunked`, `UOR_R4_RECURRENCE_TILE`) and unadopted.

**Step profile (RTX 5090, 29M bf16, batch 16, ctx 384, `nsys`, 5 steps, 67 kernels):**

| family | share |
| --- | --- |
| **fused geometric read** | **34.5%** (forward 8.9%, **backward 25.6%**) |
| gemm (cuBLAS/cutlass bf16 tensor cores) | 17.4% |
| rms_norm (`bwd_dx` 9.2%, `fwd` 4.5%, `dw_partial` 2.8%) | 16.6% |
| Candle elementwise / bf16 casts (`ia_u32_f32` 4.0, `ucopy_bf16` 3.2, `badd_bf16` 2.7) | 11.5% |
| recurrence core (prep, carry, out, bwd, param partials) | 8.3% |
| cross_entropy (fwd+grad) | 4.3% |
| grad squared norm (`sq_lanes`+`sq_tree`, ~129 launches/step) | 3.8% |
| adam_update / other | 3.3% |

Read backward detail: `read_dkv` 6.9%, `read_tile_inner` (runs forward *and* backward) 5.9%, `read_key_self` 5.0%, `read_dage` 4.5%, `read_row_grad_warp` 4.0%, `read_dbeta` 2.8%.

## 3. The next work item: Phase 2b, the flash-style read

### 3.1 Why this and not something else

The read is a third of the step and three quarters of it is the backward, which
materialises and re-walks six `T x T` buffers per (batch, head). The recurrence is
8.3% — that is why Phase 2a could not pay. RMSNorm (16.6%) is the second target
after the read.

### 3.2 The frozen gate

[`bf16-phase2b-gate.md`](bf16-phase2b-gate.md), frozen before any run:

- recipe = Phase 1's Arm A base recipe with `precision=bf16`, 70,609 steps, seeds 1 and 2 for both the current read and the flash read, one binary, `UOR_R4_CUDA_READ=flash` (or `read_path=flash` on the CLI);
- adopt iff, for both seeds, `|NLL(flash,s) − NLL(current,s)| ≤ f32-arm seed spread` **and** `< 0.01` nats — the gate recomputes its own spread;
- op parity first: `test_flash_read_parity` over Dot/Lorentz/L2, NoRead and age on/off, bound `1e-4 + 1e-3·|ref|`; bit-identical at tile length 1 if the implementation has a tile knob; determinism; the existing 32 tests still pass;
- **speed floor: the flash read must be ≥5% faster end to end at 29M, else not adopted** — a neutral rewrite must not ship again (Phase 2a's lesson);
- report the before/after profile with the same `nsys` method, the NLL table, the parity table and 96M tokens/s.

### 3.3 Current implementation map (start here)

- `crates/uor-r4-training/src/cuda_stack_kernels.rs` §7 "General fused read": `read_lift`, `read_tile_inner`, `read_softmax_warp`, `read_mix`, `read_row_grad_warp`, `read_key_self`, `read_dq`, `read_dkv`, `read_dage`, `read_dbeta`; helper `read_distance` (f64 arcosh/sqrt), `read_age_offset`, `read_beta_offset`.
- `crates/uor-r4-training/src/geometric_stack.rs`: the `FusedRead` op (validation in `fused_read_selected`, `fused_aux_len`), scores `ReadScore::{Dot, Lorentz, L2}`.
- `crates/uor-r4-training/src/geometric_stack/cuda_ops.rs`: `FusedRead::cuda_pass` (builds probabilities/`null_probability`/lifts/`excess`), `cuda_fwd_impl`, `cuda_bwd` — the latter is the 12-kernel backward to replace.
- `crates/uor-r4-training/src/geometric_stack/cuda_ops_bf16.rs`: the bf16-storage mirror (`cuda_pass_bf`, `cuda_fwd_impl_bf`, `cuda_bwd_bf`) — **both modules must gain the flash path**, or the f32 path must fall back cleanly.
- Buffers today (per (batch, heads, time), square = rows x time): `probabilities` f32, `null_probability` f32, `query_lift`/`key_lift` f64, `excess` f64, `dp` f32, `d_scores` f64, `inner_grad` f32, `key_self` f64, `query_self`/`row_beta`/`row_offset` f64.

### 3.4 Target design (concrete)

- **Forward.** Tiles over `(t, j)` with `j <= t`: per tile compute scores in f32 from bf16-stored q/k (Dot: scaled inner product; Lorentz/L2: the f64 excess and `read_distance`), keep the online softmax (running max and sum) plus the NoRead logit, and accumulate the value read `sum_j p_j v_j` in the same pass. No `T x T` buffer. `read_lift` stays (per-row f64 lifts).
- **Backward.** Recompute each tile's scores and probabilities from the stored per-row `(max, sum)`; accumulate `dq` and `dkv` in registers/shared per tile and add them into the output gradients (no `ds`, `dp`, `inner_grad`, `key_self` matrices). Keep the per-row scalars the scores need: the NoRead logit gradient, `d_beta`/`d_offset` per head, the age-table gradient, and the Lorentz/L2 self terms (`query_self`, `key_self`) as per-row accumulators.
- **What must stay f64/f32** (same islands as Phase 1): the lifts, the Lorentz excess and distance, the score arithmetic in f32, the softmax accumulation in f32, all parameters.
- **Keep the refusals**: RoPE and a flock selection still run on the host in bf16 mode (`precision=bf16` refuses a selection today); the flash path may keep refusing them.
- **Launches**: aim for ≤4 kernels forward and ≤4 backward (today 10 and 12), with one kernel per direction if the tile loop fits.

### 3.5 First commands once a pod is up

```sh
# build (≈7 min cold), 5090 = sm_120
git clone -q --depth 1 --branch codex/bf16-phase2b-flash-read https://github.com/UOR-Foundation/uor-r4.git ds-uor-r4
cd ds-uor-r4 && export PATH=/usr/local/cuda-12.8/bin:$HOME/.cargo/bin:$PATH \
  LD_LIBRARY_PATH=/usr/local/cuda-12.8/lib64 CUDA_HOME=/usr/local/cuda-12.8 \
  CUDA_COMPUTE_CAP=120 CARGO_TARGET_DIR=/root/target-deepseek CARGO_INCREMENTAL=0
cargo build --release -p uor-r4-training --features cuda --example geometric-stack --example m-world
# parity (single-threaded: the selectors are process-global)
flock /root/gpu1.lock -c 'CUDA_VISIBLE_DEVICES=1 UOR_REQUIRE_CUDA=1 cargo test --release \
  -p uor-r4-training --features cuda --test cuda_stack_ops_parity -- --test-threads=1 --nocapture'
# 29M speed A/B (300 steps, alternate arms, 3 rounds) — the pattern is in the
# Phase 2a branch's /root/ds-bench2.sh equivalent; data under $D as below
$GS train out=/root/ds-bench/$name seed=7 lr=0.0005 precision=bf16 \
  train=$D/corpora/ts-train/tokens.u16,$D/corpora/td-train/tokens.u16,$D/chat-v0-p2/train/tokens.u16 \
  train_weights=0.6,0.15,0.25 valid=$D/corpora/ts-valid/tokens.u16 tokenizer=$D/tokenizer.json \
  arch=geometric width=576 heads=8 layers=10 pattern=rrarrarrar context=384 read=l2 rotation=true \
  key_shift=false steps=300 batch=16 warmup=200 min_lr=0.1 weight_decay=0.1 clip=1.0 \
  eval_every=300 eval_windows=8 final_windows=8 checkpoint_every=0 sample_tokens=0 \
  device=cuda tf32=true data_parallel=1 UOR_R4_CUDA_READ=flash
```

`$D=/root/data`; extract the corpora once with
`cd /root/data && tar -xf /workspace/uor-r4/data/step5-inputs.tar` (gives
`corpora/{ts-train,ts-valid,td-train}`, `chat-v0-p2/train`).

## 4. GPU protocol and environment

- **Rules:** `docs/labs/compute.md` on `claude/compute-protocol` (PR #1758, not yet merged). Use `scripts/pod/uor-pod` from that branch for `status/lease/run/renew/release/up/down`; **ask the owner on #820 before creating a pod** until the tool is merged. Caps: ≤4 pods, ≤$8/h, 2×5090 per pod, RTX 5090 only (`CUDA_COMPUTE_CAP=120`), never A100/H100 until Phase 2 passes its gate. Lease before use, renew ≤30 min, one job per GPU under `flock /root/gpuK.lock`, release when done, post one line on the [Compute board #1750](https://github.com/UOR-Foundation/uor-r4/issues/1750).
- **Live now:** `ic5zsemlme7moh` = `uor-deepseek-1005-2109`, 1×RTX 5090, $0.99/h, initializing (the new DeepSeek session's pod). Canonical volume `lmd1pfah3y` (`/workspace`); new non-canonical volumes `rfsx702p68` (EU-RO-1) and `v65f9luxbb` (EUR-IS-1) were just created for the placement order.
- **Storage:** everything durable on `/workspace`; `/root` dies with the pod. My artifacts are at `/workspace/uor-r4/deepseek/`: `split-29m-bf16-kern.csv`, `chunked-29m-bf16-kern.csv` (nsys tables), `29m-sweep.txt`, `29m-alternating.txt`, `96m-sweep.txt`, `parity-suite.log`, `chunked-parity-timings.log`. Archive new results the same way.
- **Cost so far for this work:** Phase 1 ≈ $5.2; Phase 2a ≈ 0.6 GPU-hours ≈ $0.6 on an already-running pod.
- A 2-GPU pod is worth having for the 2-seed gate (4 runs: ~1 h per run per GPU in parallel ≈ 2 h wall); 1 GPU makes it ≈ 4 h.

## 5. Gotchas learned the hard way

- `pattern=` must have one letter per layer: the 96M shape is `layers=14`, so omit `pattern=` (the CLI defaults to `"rra".repeat(layers/3) + "r".repeat(layers%3)`) or pass a 14-letter string. Passing `rrarrarrar` with `layers=14` fails validation.
- `UOR_CUDA_PROFILE=1` synchronises after every launch: it slows the step ~10× (12.6k vs 124k tok/s) and makes per-kernel shares meaningless. Use `nsys` (`/opt/nvidia/nsight-systems/2024.6.2/target-linux-x64/nsys profile --trace=cuda`, then `nsys stats --report cuda_gpu_kern_sum`); its "Total Time" column is **nanoseconds**.
- The CUDA parity suite must run `--test-threads=1`: `set_cuda_recurrence_kernels`, `set_recurrence_tile` and any read selector are process-global.
- bf16 islands that must not move: quaternion transport products and norms, RMSNorm statistics (f64), all read scores/probabilities/lifts/excesses/distances, the recurrence's carried state/drive/transition/`keep`, the loss's f64 log-sum-exp, parameters, optimiser, evaluation, saved models (D11 export).
- Builds: `CUDA_COMPUTE_CAP=120` for a 5090; use your own `CARGO_TARGET_DIR` (other labs' dirs are live); cold build ≈7 min.
- Leases: the ledger's `release` line for the 19:21Z deepseek lease was written by another actor, not by the leasing session — if a lease file vanishes, check `ledger.jsonl` and re-lease before assuming you still hold the GPUs.
- Never touch another lab's `/root` checkout or its material on `/workspace`.

## 6. Open decisions

1. **PR #1757**: merge as-is (profile + design revision + opt-in parity-tested chunked path) or keep the docs and drop the path? It is open and unqueued.
2. **Phase 2b scope**: the read alone (as frozen), or read + RMSNorm in one PR? The frozen gate covers the read only; RMSNorm (16.6%) deserves its own protocol if taken.
3. **Gate budget**: four 70,609-step runs on 2×5090 ≈ 2 h wall and ≈ $4, plus the parity and 96M probes.

## 7. Definition of done for the next session

- `test_flash_read_parity` green over Dot/Lorentz/L2 x NoRead x age, with the stated bounds, and the other 32 CUDA tests still green.
- A 29M bf16 A/B showing **≥5%** end-to-end at 29M on a 5090 (else record the negative and stop, as Phase 2a did).
- The frozen 2-seed NLL gate run and tabulated (adopt only if both seeds are within the spread and < 0.01 nats).
- 96M tokens/s for both arms.
- A PR with the exact-head evidence, `docs/compute/bf16-phase2b-report.md`, and one line on #820 plus the Compute board.
