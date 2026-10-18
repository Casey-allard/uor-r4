# Decode tokens/s against context — the optimization rung's baseline instrument (DRAFT)

- **Status:** DRAFT measurement plan. **Not registered**, and TEST FITNESS is not requested.
  - No source change, no compute, $0.
  - This is optimization-rung step 2 (`project-track.md` § Post-dose optimization path): "release-profile baselines … with tokens/s bars pre-registered on the same CUDA harness before any kernel work". It runs after the fast-path Dose 1 decision.
  - The PM measurement-plan seat is Kimi's. Kimi is offline, so Claude drafts this for Kimi to adopt or replace.
- **Line:** M1/M4 serving speed (#2029). **Lab:** claude.
- **Owner metric:** tokens/s, decode speed against context, with BPB secondary (owner, 12 October).
- **Rules:**
  - D24: GPU only. The host loop only moves token ids between GPU calls; no model arithmetic runs on the host.
  - D11: the measured artifact is the integer discrete model.
  - D23/D25/D32.

## 1. What is missing

- **No decode measurement exists for the discrete model.** The driver-cost lab measured target rows/s for forward and proposal, and disclaims decode explicitly (`docs/labs/discrete-driver-cost-2026-10-13/README.md`). The K16 timing reported padded-context tokens/s for training steps, not generation.
- **The model's cost thesis is architectural:** cache-resident addressed state and no per-token weight streaming. That thesis is untested until decode tokens/s is measured against context length.

## 2. Instrument

A release-profile CUDA binary, `decode-bench`, separate from the training driver so the driver's identity is unchanged. For a loaded checkpoint (the fast-path Dose 1 checkpoints), it runs:

- **Single-stream decode:** for context lengths `c ∈ {16, 32, 64, 128, 192, 243}`:
  1. Carry a window of `c` tokens: a fixed prompt prefix from DEV, never scored.
  2. Run `forward`, take `predictions[0]` and append it, sliding the window when it reaches the configured context.
  3. Repeat for `N = 256` generated tokens.

  It reports tokens/s as the median of 5 repeats after 1 warmup, with every call synchronized and timed in integer ns, plus per-token latency p50/p95.
- **Batched decode:** the same with 64 independent streams per call (`rows = 64`), reporting aggregate tokens/s and per-stream tokens/s.
- **Split of the cost:** carry (encode), forward (read plus writer) and host id movement, timed separately.
- **Identity in the report:** checkpoint hash, source, binary SHA, GPU and driver, and every `c`, `N` and repeat count.

**This is a timing instrument only.** It makes no quality claim, and generated text is not graded here.

## 3. Bar

Fixed at registration as the baseline record. It sets no pass/fail, because the first run is the reference every optimization-rung kernel change must beat. Each later kernel PR then pre-registers its own bar against this record:
- a tokens/s gain at the same `c` and rows;
- generated tokens identical to the baseline checkpoint's on the same prompt, so speed comes with no behavior change.

## 4. Base artifact

The fast-path Dose 1 treatment and frozen-zero final checkpoints, by hash, once produced. Nothing is bound in this draft.

## 5. Data split

Prompt prefixes come from DEV. Generation is not scored. No TRAIN, fresh or new data is used.

## 6. Steps

1. After the Dose 1 decision, implement `decode-bench` (Rust, opt-in, a separate binary).
2. Run finite controls:
   - generated ids equal across repeats, so decode is deterministic;
   - the batched stream 0 equals the single stream on the same prompt;
   - a sliding window at `c = 243` refuses nothing.
3. Run on one RTX 5090 and record the baseline.
4. Post it on #2029 as the optimization rung's reference.

## 7. Stop rule

- A control FAIL stops for diagnosis (D31).
- Owner caps and the disk floor are hard stops.
- No retry with changed code without review.

## 8. Cost

One 5090 for under 30 minutes, about $0.5.

## 9. Minimum test power

Timing only: 5 timed repeats per cell after a warmup, with the median and p95 reported, over 6 context lengths × 2 batch shapes × 2 checkpoints.

TEST FITNESS: PENDING
