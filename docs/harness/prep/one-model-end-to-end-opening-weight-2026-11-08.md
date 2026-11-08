# Prep sheet: opening-weighted reply SFT (one-model-end-to-end, deepseek-35)

Lab: deepseek. Item: [one-model-end-to-end](../../roadmap/items/one-model-end-to-end.md)
(#3407). Order: [deepseek-35](../../loop/tasks/one-model-end-to-end/deepseek-35/ORDER.md).

TEST FITNESS: FIT - fresh-context DeepSeek exact-head review of this sheet and the opening-weight implementation; weighted rows and micro-batch normalization are covered, default behavior is unchanged, and no FIT blocker remains.

PRE-RUN: GO WITH ADMISSION GATE - fresh D67 review passed; the three stored input bytes were downloaded and matched the pinned hashes, the runner re-hashes them, the pod build runs the CUDA weighted micro-batch test, and initialization is approved only after the run config records weight 8. No DeepSeek GPU was free at review and no lease was taken; launch only after admission, from the merged pin, without touching another lab's pod.

## Why

The closed 1,024-step reply-SFT run and its content-ratio successor both fail at
the first answer token. On the sealed legacy step-512 checkpoint, the first
target is 0/256 at top-1, top-8 and top-32; rank is min/p50/max 59/894/4,041.
After a correct opening, 1,580 later targets are top-1 and 2,566 are top-8 out
of 2,576. The content-ratio step-512 checkpoint is the same qualitative result.

This tests one objective-side intervention: repeat only scored reply-opening
rows eight times in the existing geometric nearest loss. Serving, architecture,
optimizer, data, selector and model dimensions remain unchanged.

## Question

At 1,024 steps from the same `(b64L)` seed-1 base and original OASST2-heavy
reply-masked mix, does an opening weight of 8 move first-target ranking and
complete replies while retaining the continuation and EOS selectors?

## Bar

This is a staged development run, not a milestone bar.

- Positive content row: first-target top-1 >= 16/256 and top-8 >= 64/256;
  EOS top-1 >= 16/256; free EOS stops >= 16/256; complete accepted rows
  >= 4/256; continuation >= 93/512.
- Opening movement row: first-target top-1 >= 8/256 and top-8 >= 32/256, but
  the positive content row is not met.
- Partial movement row: any first-target top-1 > 0 or top-8 >= 16/256.
- No movement: first-target top-1 is 0 and top-8 is below 16/256.

The first-target bars are above the observed legacy baseline of 0/256 and
0/256. No M2 claim is possible because `m2-reply512` remains unavailable.

## Base artifacts

Use the original 1,024-step sheet's exact seed-1 `(b64L)` checkpoint,
tokenizer, held-out guard, panel and hashes. The checkpoint is:

`oh/run/b64L-s1/models/step-016384`.

The matched legacy comparator is deepseek-26 selected step 512 on the same
data and dose: continuation 142/512, EOS top-1 0/256, EOS top-8 256/256,
median EOS rank 2, free EOS stops 23/256, complete rows 0, first-target ranks
min/p50/max 59/894/4,041.

## Data split

No data is rebuilt. Use the original sealed store:

`results/deepseek/one-model-end-to-end-reply-sft-2026-11-07/inputs/`

| file | SHA-256 |
|---|---|
| `tokens.u16` | `3a2c397e4b1a13c488096f931cf377efaeed51d2de17c231ac72d4b6291899e3` |
| `response_mask.u8` | `67af9f09152510edba99795f3ff247a6f17eb80c3907836cd7c0af6d7cddb28b` |
| `manifest.json` | `1c7aee4dfaffdde07f9c64a5b887b504ee21355038b6ae00897a8a97407d9bbe` |

The split has 25,071 documents, 4,059,876 scored targets and 79,679 scored
replies. Opening targets are 1.96% of scored targets; weight 8 multiplies their
loss contribution without changing any input token or serving row.

## Recipe

The original 1,024-step recipe with one opt-in flag:

```text
--opening-weight 8
--steps 1024
--eval-every 256
--seed 1
```

Training starts at step 0 from the same base with a fresh optimizer and
schedule. Geometry, width, context, batch, micro-batch retry, training-as-served,
checkpoint scoring and selector settings are unchanged.

## Readings and selector

Record the base controls and checkpoints 256, 512, 768 and 1,024. Apply the
unchanged deterministic `per-turn/v1` selector. Every teacher-forced report
records first-target top-1/top-8/top-32, rank min/median/max and the later-target
totals. Those fields are diagnostics only in this unit; the existing selector
ordering does not change.

## Outcome

Apply this first-match map to the selected checkpoint:

1. Continuation `< 93/512`: continuation failure; close without seed 2.
2. Every positive-content threshold is met: unlock seed 2 and a longer
   opening-weighted run.
3. First-target top-1 `>= 8/256` and top-8 `>= 32/256`, but positive content is
   not met: opening moved; diagnose the remaining suffix, EOS or free-generation
   gap before another dose.
4. First-target top-1 `> 0` or top-8 `>= 16/256`: partial opening movement;
   close this weight/data/dose configuration.
5. Otherwise: no opening movement; close this objective configuration and do
   not try a weight sweep.

The original EOS, free-stop and complete-row readings are recorded alongside
the opening row so the content gap can be classified, but they do not preempt
this opening-specific first-match map.

This is one fixed weight, not a hyperparameter sweep.

## Steps

1. Record fresh FIT and D67 review of this sheet, the exact runtime and the
   opening-weight implementation.
2. Check pod admission and use one DeepSeek-lab RTX 5090 inside project caps.
3. Fetch and re-hash the original source, base, panel and tokenizer; reproduce
   the registered base controls before step 0.
4. Train for 1,024 steps with `--opening-weight 8`, scoring every 256-step
   checkpoint with the unchanged CPU grade and panel chain.
5. Apply the original and opening-specific outcome rows, seal, read back and
   delete the owned pod.
6. Record one result. No seed 2 unless the positive content row is reached.

## Stop rule

- Instrument, pin, input, panel or merge failure: train nothing.
- OOM: retry micro-batch 8 once; a second failure is `NOT_RUN`.
- Non-finite loss, missing checkpoint, changed hash, failed export or panel:
  stop the run.
- Never run a weight sweep or another ratio/dose extension from this sheet.
- Never cancel a healthy run for a self-set timer. Never touch another lab's
  pods, leases, files or GPU devices.

## Minimum test power and cost

One seed is an exploratory first-read against a matched weight-1 run. The prior
1,024-step run took 721 seconds of training and 1,457 seconds of pod uptime.
Four checkpoints and scoring project to roughly 24 minutes and $0.23, with one
OOM retry under $0.65.

## Instrument checks

The CPU host identity, per-grade hardware sidecars, source hashes, running
config with `opening_weight: 8`, first-target diagnostics, seal limit and store
readback all remain mandatory. A missing or defaulted opening weight stops the
run before training. The pod build must also pass the CUDA test that weighted
whole-batch and micro-batched gradients agree.
