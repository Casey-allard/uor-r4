# bf16 Phase 2b gate — fused-read rewrite (frozen decision rule)

> **Status: frozen 2026-10-05 before any Phase 2b run.** Companion to the Phase 1
> rule ([`bf16-parity-gate.md`](bf16-parity-gate.md)) and to the measured step
> profile ([`bf16-step-profile-2026-10-05.md`](bf16-step-profile-2026-10-05.md))
> that selects the read as Phase 2's target. The recipes, the acceptance rule and
> the measurement method below may not change after the first gate run starts; a
> later change is a new gate with its own record.

## 1. What the gate decides

The fused geometric read is **34.5%** of a 29M bf16 training step on an RTX 5090
(forward 8.9%, backward 25.6%; `read_dkv` 6.9%, `read_tile_inner` 5.9%,
`read_key_self` 5.0%, `read_dage` 4.5%, `read_row_grad_warp` 4.0%,
`read_dbeta` 2.8%). It materialises six `T x T` buffers per (batch, head) —
probabilities, the f64 excess, `dp`, the f64 `ds`, `inner_grad` and the
per-(index, j) `key_self` — and walks them with ten kernels forward and twelve
backward. The gate decides whether a flash-style dataflow (tiles over `(t, j)`,
an online softmax, scores recomputed in the backward, no materialised `T x T`
buffers) may replace it.

## 2. Frozen recipe

Identical to Phase 1's Arm A base recipe (`docs/compute/ladder-runbook.md` §3.3),
so the two gates are comparable:

```
geometric-stack train out=RUN seed=SEED lr=0.0005 precision=bf16 \
  train=$D/corpora/ts-train/tokens.u16,$D/corpora/td-train/tokens.u16,$D/chat-v0-p2/train/tokens.u16 \
  train_weights=0.6,0.15,0.25 valid=$D/corpora/ts-valid/tokens.u16 tokenizer=$D/tokenizer.json \
  arch=geometric width=576 heads=8 layers=10 pattern=rrarrarrar context=384 \
  read=l2 rotation=true key_shift=false \
  steps=70609 batch=16 warmup=200 min_lr=0.1 weight_decay=0.1 clip=1.0 \
  eval_every=5000 eval_windows=64 final_windows=512 checkpoint_every=10000 \
  device=cuda tf32=true data_parallel=1
```

- Four runs: the **current read** (reference) and the **flash read**, seeds 1 and
  2 each, one binary for all four; the arms differ only in the new
  `UOR_R4_CUDA_READ=flash` switch (or `read_path=flash` on the CLI).
- 433,821,696 target tokens per run; two runs per GPU in parallel.
- A run counts only at `completed_steps == 70609`; a `max_seconds`-capped root is
  resumed, never used.
- Evaluation is f32 in both arms (Phase 1's rule), so the comparison isolates the
  read's arithmetic.

## 3. Frozen acceptance rule

Let `NLL(arm, seed)` be a run's `final.nll`, `d_s = |NLL(flash, s) − NLL(current, s)|`
and `spread = |NLL(current, 1) − NLL(current, 2)|`.

**Adopt the flash read iff, for both seeds, `d_s ≤ spread` and `d_s < 0.01` nats.**

The Phase 1 gate measured `spread = 0.00374975` for this recipe; this gate
recomputes its own spread from its own two reference runs and does not inherit
that number.

## 4. Frozen op parity (before any gate run)

`cargo test --release -p uor-r4-training --features cuda --test
cuda_stack_ops_parity` with `UOR_REQUIRE_CUDA=1`, extended with a
`test_flash_read_parity` that compares the flash read against the current read on
the same inputs, for all three scores (Dot, Lorentz, L2), with and without the
NoRead slot and the age table:

- values: stated bound `1e-4 + 1e-3 * |reference|` on the read output, the query
  and key/value gradients and the auxiliary gradient (the forward's softmax order
  changes, so this is a tolerance, not bit-exactness);
- with a tile length of 1 (if the implementation has one) the flash path must
  match the current read bit for bit;
- determinism: the same run twice gives identical bits;
- the suite's existing 32 tests still pass.

## 5. Frozen speed measurement

- **29M**: the four gate runs' own `tokens_per_second`, plus an alternating
  300-step A/B (current, flash) x 3 rounds on one GPU, reporting the medians and
  the ratio.
- **96M**: the ladder's 96M shape (width 1024, heads 16, layers 14, context 384,
  batch 32 data-parallel over 2 GPUs, or batch 32 on one) as a 60-step probe,
  both arms, tokens/s.
- **The gate fails on speed alone** if the flash read is not at least 5% faster
  end to end at the 29M shape (the read is 34.5% of the step; a rewrite that does
  not move the step is not adopted, as Phase 2a's record shows).

## 6. Reporting

The profile table before and after (same `nsys` method as
[`bf16-step-profile-2026-10-05.md`](bf16-step-profile-2026-10-05.md)), the NLL
table with `d_s` and the spread, the parity table with measured maxima, the
tokens/s at both shapes, and every remaining f32 island the read keeps (the
lifts, the excess and the distances are f64; that does not change).
