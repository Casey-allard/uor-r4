# bf16 Phase 1 parity gate — frozen decision rule

> **Status: frozen 2026-10-05 before any gate run started.** This document is the
> decision rule for adopting `precision=bf16` in the geometric-stack trainer.
> `scripts/pod/bf16-parity-gate.sh` executes it; `scripts/pod/bf16-parity-gate.py`
> tabulates it. Neither the recipes below nor the acceptance rule may be changed
> after the first base run starts; a later change is a new gate with its own
> record.

## 1. What the gate decides

Phase 1 of the bf16 rewrite adds an opt-in `precision=bf16` to
`geometric-stack train` and `dialogue-train`: the trunk's activations and every
matmul operand become bf16 (Candle's CUDA bf16 GEMM, `CUBLAS_COMPUTE_32F`
accumulation on tensor cores), and the CUDA custom kernels of the active
geometric path (recurrence core, fused read, pointer mixture, RMSNorm, SwiGLU,
cross-entropy) read and write bf16 activation storage while accumulating in
f32/f64. Master weights, Adam moments, the optimiser, the loss, evaluation and
saved models stay f32 (see `crates/uor-r4-training/src/geometric_stack.rs`,
`Precision`).

The gate decides **adoption of bf16 as the default training precision for the
next scale rungs**. It is a parity question, not a speed question: bf16 may only
be adopted if it trains at least as well as f32 within seed noise.

## 2. Frozen recipe (both arms)

The 29M rung of the ladder runbook (`docs/compute/ladder-runbook.md` §3.3), Arm
A (the `lr5e-4` base mix), exactly as `scripts/pod/step5-knowledge.sh:stage_base`
trains it — same data files, same steps, same hyperparameters, `tf32=true` in
both arms (the f32 arm's matmuls are TF32, the bf16 arm's are bf16):

```
geometric-stack train out=RUN seed=SEED lr=0.0005 \
  train=$D/corpora/ts-train/tokens.u16,$D/corpora/td-train/tokens.u16,$D/chat-v0-p2/train/tokens.u16 \
  train_weights=0.6,0.15,0.25 valid=$D/corpora/ts-valid/tokens.u16 tokenizer=$D/tokenizer.json \
  arch=geometric width=576 heads=8 layers=10 pattern=rrarrarrar context=384 \
  read=l2 rotation=true key_shift=false \
  steps=70609 batch=16 warmup=200 min_lr=0.1 weight_decay=0.1 clip=1.0 \
  eval_every=5000 eval_windows=64 final_windows=512 checkpoint_every=10000 \
  device=cuda tf32=true data_parallel=1
```

- Four base runs: `f32-s1`, `f32-s2`, `bf16-s1`, `bf16-s2`; the bf16 runs add
  `precision=bf16`. **One binary** builds all four: the arms differ only in that
  flag.
- 433,821,696 target tokens per run, 2 runs per GPU in parallel.
- A run counts only at `completed_steps == 70609` (`stopped_early == false`);
  a `max_seconds`-capped root is moved aside and resumed, never used.
- The report's `final.nll` (f32 scoring, 512 windows, `ts-valid`) is the
  comparison quantity. Evaluation is always f32 in both arms, so the comparison
  isolates the training-step arithmetic.

## 3. Frozen fine-tune and session recipe (both arms)

Recipe C of `scripts/pod/step5-knowledge.sh:stage_ft` from each base, at its
base's precision:

```
geometric-stack dialogue-train out=FT tokenizer=$D/tokenizer.json \
  train_tokens=$D/ft/mixed-c/tokens.u16 train_mask=$D/ft/mixed-c/response_mask.u8 \
  train_manifest=$D/ft/mixed-c/manifest.json \
  dev_tokens=$D/ft/dev/tokens.u16 dev_mask=$D/ft/dev/response_mask.u8 \
  dev_manifest=$D/ft/dev/manifest.json init=BASE/model \
  pointer=32 protocol=2 context=384 policy=full_prefix data_seed=SEED \
  steps=4000 batch=16 lr=0.0003 warmup=100 min_lr=0.1 weight_decay=0.1 clip=1.0 \
  eval_every=500 checkpoint_every=4000 dev_seed=20260930 dev_per_source=32 \
  max_seconds=10800 device=cuda tf32=true
```

and the D19 grounded session (`log_recall=off`, the unaugmented-state baseline)
of each fine-tune:

```
m-world session out=SESSION world=v2 model_root=FT tokenizer=$D/tokenizer.json \
  compiler=$D/sieve/compiler-save-op-v25-rawtable/compiler.json \
  trunk=$D/sieve/op-model-v25/model op_policy=unless_query \
  log_recall=off max_new_tokens=64 arms=default reload=0
```

Session score = the `default` arm's summed `by_category.pass` out of 1,075
scored turns.

## 4. Frozen acceptance rule

Let `NLL(arm, seed)` be a base run's `final.nll`, let
`d_s = |NLL(bf16, s) − NLL(f32, s)|` and `spread = |NLL(f32, 1) − NLL(f32, 2)|`.

**NLL criterion.** bf16 passes on the NLL axis iff **for both seeds**
`d_s ≤ spread` **and** `d_s < 0.01` nats.

Let `S(arm, seed)` be the session score, `e_s = |S(bf16, s) − S(f32, s)|` and
`noise = |S(f32, 1) − S(f32, 2)|`.

**Downstream criterion.** bf16 passes on the session axis iff **for both seeds**
`e_s ≤ max(noise, 12)` scored turns. (The floor is the seed spread measured for
this exact recipe at the same scale: Arm A seeds 1 and 2 scored 843 and 855 of
1,075 in `/root/runs/step5/session-off-A-s{1,2}`, a spread of 12. Two f32 seeds
can tie exactly, which would make a floor-less rule zero-width.)

**Decision.** bf16 is adopted as the ladder's training precision iff both
criteria pass for both seeds, and the fine-tunes complete 4,000 steps. Every
number is reported either way, including a failed gate.

## 5. Reporting

- tokens/s of both arms on the same RTX 4090 from each run's own
  `tokens_per_second` (`target_visits / train_seconds`), plus the per-round
  median and the ratio.
- The NLL table (`f32`, `bf16` × seeds), `d_s`, `spread`.
- The session table (`f32`, `bf16` × seeds), `e_s`, `noise`.
- Every remaining f32 island (the `Precision` documentation plus this gate's
  observed coverage), and the Phase 2 design note.

## 6. Cost projection (recorded before the runs)

- Base: 4 × 433.8M tokens. Measured before the gate on the same pod and recipe
  (300-step runs, seed 7): f32 98,350 tok/s, bf16 119,940 tok/s → 1.22×.
  Projection: 2 f32 runs in parallel ≈ 1.24 h, 2 bf16 runs in parallel ≈ 1.01 h,
  ≈ 2.3 h wall with 2 × RTX 4090 (`$1.48/h/pod`) ≈ **$3.4** of GPU time.
- Fine-tunes: 4 × 24.6M tokens ≈ 4 × 4 min ≈ 0.3 h. Sessions: 4 × ~25 min CPU
  ≈ 0.5 h wall, GPU idle.
- Total: **≈ 3 h wall, ≈ $4.5**, plus one build (~3 min).
