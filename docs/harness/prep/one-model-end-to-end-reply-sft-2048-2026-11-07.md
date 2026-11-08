# Prep sheet: 2,048-step reply-SFT dose extension (one-model-end-to-end, deepseek-29)

Lab: deepseek. Item: [one-model-end-to-end](../../roadmap/items/one-model-end-to-end.md)
(#3407). Order: [deepseek-29](../loop/tasks/one-model-end-to-end/deepseek-29/ORDER.md).

TEST FITNESS: FIT - fresh non-author main-integrated review at `c020b4d2e5946703e43ac158cfee7d48622dd79e`, reviewed-runtime `b5aa959857f387bdc2329d0e68e243cb6473e88fa46f56553f95a47c884eaa80`, reviewer session `01a1b944-30a2-7ce0-aac4-8b01fc2cc6d2`.

PRE-RUN: GO - fresh D67 review at `c020b4d2e5946703e43ac158cfee7d48622dd79e`, reviewed-runtime `b5aa959857f387bdc2329d0e68e243cb6473e88fa46f56553f95a47c884eaa80`, reviewer session `01a1b944-30a2-7ce0-aac4-8b01fc2cc6d2`.

## Why

The reviewed 1,024-step reply-masked SFT selected step 512 with continuation
142/512, teacher-forced EOS top-1 0/256, top-8 256/256, median rank 2, free EOS
stops 23/256 and zero complete accepted replies. That is outcome row 6, partial
movement. The registered next step is one 2,048-step dose extension at a new
root.

## Question

With the same `(b64L)` seed-1 base, mixed data, reply mask, optimizer and
selector, but 2,048 steps instead of 1,024, does the selected checkpoint clear
an outcome row above row 6 while preserving continuation?

## Bar

This is a staged development run, not a milestone bar. The positive target is
original outcome row 3 at the selected checkpoint: EOS top-1 >= 16/256, median
EOS rank <= 3, free EOS stops >= 16/256, complete accepted rows >= 4/256, and
OASST2 continuation top-1 >= 93/512. The M2 bar remains >= 256/512 saved
questions, >= 52/128 complete-and-distinct pairs and >= 104/256
single-row-complete questions on the genuinely fresh panel. `m2-reply512`
remains unavailable, so this unit cannot claim an M2 pass.

## Base artifacts

Use the exact base, data, tokenizer, held-out guard, panel and hashes in
[the original sheet](./one-model-end-to-end-reply-sft-2026-11-07.md). The
1,024-step run and its seal remain immutable at:

`hf://datasets/caseyallard/uor-r4-store:results/deepseek/one-model-end-to-end-reply-sft-2026-11-07/runs/s1-20261108t005743z-a849820fba9cfbf4`

The fixed training base is the same seed-1 `(b64L)` checkpoint
`oh/run/b64L-s1/models/step-016384`. Its model, config and Hamming-read hashes,
the fresh optimizer/schedule requirement and the base-control readings are
normative in the original sheet and must be reproduced before step 0.

## Data split

No data is rebuilt. The input is the same OASST2 reply-masked TRAIN plus M-World
mix, tokenizer, DEV split, OASST2 held-out continuation guard and M2 panel fixed
by the original sheet. The mixed store and its hashes are unchanged:

| file | SHA-256 |
|---|---|
| `tokens.u16` | `3a2c397e4b1a13c488096f931cf377efaeed51d2de17c231ac72d4b6291899e3` |
| `response_mask.u8` | `67af9f09152510edba99795f3ff247a6f17eb80c3907836cd7c0af6d7cddb28b` |
| `manifest.json` | `1c7aee4dfaffdde07f9c64a5b887b504ee21355038b6ae00897a8a97407d9bbe` |

The OASST2 held-out continuation files, deterministic 512-row sample, 93/512
guard floor and four zero corpus-panel overlap counts are also inherited
unchanged. The pod must re-hash every named input before the first forward.

## Recipe

The original recipe with one change:

```text
--steps 2048
--eval-every 256
--seed 1
```

All other flags, hashes, base checkpoint/config/Hamming files, batch size,
micro-batch retry and scoring commands are unchanged. Training starts from the
same base at step 0 in a new root. It does not continue from the selected
step-512 checkpoint.

## Readings and selector

Record the same base controls and all checkpoint readings. Apply the selector
and `per-turn/v1` ordering from the original sheet unchanged across checkpoints
256 through 2,048.

## Outcome

Apply the original outcome rows 1-5 to the selected checkpoint. Row 2 may stop
the trainer at step 512 exactly as before. Because this run is the one row-6
extension, another row-6 result closes this dose; it does not authorize a second
extension. Row 7 also closes the dose.

- Row 1: continuation failure; no seed 2.
- Row 2: no ranking movement; stop and close this configuration.
- Row 3: positive; unlock seed 2 and a longer run from the selected checkpoint.
- Row 4: termination moved but complete content did not; diagnose first
  divergence, then a different answer-content intervention.
- Row 5: ranking moved but free generation did not; diagnose the mismatch.

No result claims M2, a mechanism effect, seed reproducibility or useful chat.

## Steps

1. Record fresh FIT and D67 reviews of the merged 2,048-step code and this
   sheet, including the exact reviewed commit and normalized runtime identity.
2. Check `uor-pod status`; use one DeepSeek-lab RTX 5090 under a two-hour lease
   while retaining room inside the four-pod and $8/hour project caps.
3. Build the exact merged pin, fetch and hash every inherited input, run the
   training-unit checks and reproduce the base rank, free-generation and
   continuation controls. Record the evaluation host's CPU model,
   architecture, logical processor count and `lscpu` output before grading.
4. Retrain from the base at step 0 for 2,048 steps. Export and score every
   256-step checkpoint before proceeding; at step 512, apply outcome row 2.
5. Apply outcome rows 1 and 3-5 to the selected checkpoint after all available
   checkpoints through 2,048. Seal, read back, then delete the owned pod.
6. Record the result as one notebook row. A second row 6 or row 7 closes this
   dose; row 3 is the only outcome that unlocks seed 2 and a longer run.

## Stop rule

- Instrument, pin, overlap, merge or input failure: train nothing; fix the
  instrument and use a new root.
- Out of memory: retry micro-batch 8 once; a second failure is `NOT_RUN`.
- Non-finite loss, missing checkpoint, failed export/reload, failed panel or
  changed input hash: stop the dose.
- Apply row 2 before continuing past step 512. Apply rows 1 and 3-5 to the
  selected checkpoint after all available checkpoints through step 2,048.
- Never cancel a healthy run for a self-set timer or budget; renew and record.
  Never touch Claude's pods, leases, files or GPU devices.

All original initialization, checkpoint-completion, progress, seal/readback and
teardown checks remain mandatory. Instrument failure trains nothing.

## Instrument checks

The D84 CPU-development grader must identify its host before the first grade.
The job writes `cpu.json` and `cpu.txt`; both enter the seal. Every base and
checkpoint grade must report substrate `cpu-development-research` and
qualification `gpu-required`. A missing CPU identity or an unlabelled grade
stops the dose before a result can be read. Each grade also gets a sidecar that
binds the grade's SHA-256 to the captured CPU model, architecture and logical
processor count.

## Minimum test power and cost

One seed is an exploratory dose reading. Seed 2 remains conditional on row 3 and
a fresh decision. The 1,024 run took 721 seconds of training and 1,457 seconds
of pod uptime. Eight checkpoints and 24 minutes of training project to 45-50
minutes and roughly $0.45-$0.55 at $0.563/hour. One OOM retry projects to under
$1.30. Before an OOM retry starts, bulky checkpoints, exports, panels and scores
from the failed attempt are replaced by a sealed SHA-256 file manifest; logs,
progress and attempt records remain. This keeps the final seal below 100 MiB.
The launcher remains inside four pods and $8/hour.
