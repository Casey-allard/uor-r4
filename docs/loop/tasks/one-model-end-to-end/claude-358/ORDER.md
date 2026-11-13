# ORDER: one-model-end-to-end, claude-358

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `f694d162d9b04d1226fd17c0a81825f00e9da007`.

## Goal

**Run the retrain the sheet pre-registers: three seeds of the place-read model and one control, trained with the `longest-after-opening` copy rule on one pod, then read on the laptop as integer artifacts.**

**Why now.** [claude-357](../claude-357/REPORT.md) built the rule into the trainer and the artifact and wrote the sheet, [one-model-end-to-end-copy-longest-2026-11-12](../../../../harness/prep/one-model-end-to-end-copy-longest-2026-11-12.md), whose fresh fitness read ends FIT. Nothing has been trained with the rule. On 12 Nov 2026 (UTC), in chat, to "Start the training run on one pod now?" Casey answered **"Yes, run it"**.

## What is run

The sheet's order of work, unchanged. The pin is the base above, claude-357's merge commit.

| Step | Where | What |
|---|---|---|
| 1 | laptop | the pinned builds; I-o and I-w at the pin; the four comparison artifacts and the set-aside patch at their hashes |
| 2 | pod | I-p, I-g with its set-aside, I-3; three 64-step smokes (p and n with the flag, p without) with I-m, I-s, I-r, I-k, I-n and I-b; the scoring pass's arguments at full size on p's smoke |
| 3 | pod | p-s2, n-s2, p-s1, p-s3: stage 3's commands plus `--hamming-copy-longest 714,28`, each with its scoring pass, each sealed to the store on exit |
| 4 | pod | the pod's four checks, then the pod goes down |
| 5 | laptop | the exports with I-c, I-h and I-t; **I-f and I-u before any reply is read**; then the eight artifacts' replies on the seed 16 reading panel, R, O, D and M, and the sheet's row |
| 6 | laptop | the reported readings: the control, `--copy-rule latest-exact`, the nine-turn conversation word for word, the repeat-a-string rows, `fbs-toks --read` |

**Compute.** One RTX 5090 Vast pod at no more than $1.60/h, through `scripts/pod/uor-pod`. Expected 4 to 6 hours and $3.60 to $5.50; the sheet's ceiling is 10 hours and $16. A watcher is armed before anything ships (L-004). A run never writes into an existing root (L-002).

## Pre-run review (claude's own; logged for Casey, who can reverse it)

- **The chain:** claude-356 read the rule at serving on stage 3's models; claude-357 built it for training; this run trains it. The sheet's bars, rows and prediction were written before the run and are not changed here.
- **Tools and inputs:** the trainer, the exporter and the instruments are on `main` at the pin. TRAIN, the scoring panel, the kit, the tokenizer and DEV are in the store at the sheet's hashes; the reading panel and the four comparison artifacts are on the laptop.
- **What each outcome unlocks:** the sheet's rows 0 to 3. Row 0 trains nothing or reads nothing and names its repair. Level closes nothing and its routing choice goes to the owner.
- **The job script** is stage 3's with the flag and the sheet's new checks. A fresh reviewer who did not write it reads it against the sheet before the pod goes up; what they find, and what was changed, goes in the REPORT.
- **The lease:** one pod, inside the caps of 4 pods and $8/h across labs. `uor-pod status` is read first.

## What a pass is

| # | Gate | Check |
|---|---|---|
| G1 | The pod's step 2 passes every instrument check, or the run stops there with nothing trained and the reason on record. | the sealed `pre` stage |
| G2 | Every stage that ran is sealed in the store and read back before the pod goes down; the pod's four checks pass. | the watcher's log, the store's sums |
| G3 | I-f and I-u are read on all four retrained artifacts before any reply is read. | the logs' order and times |
| G4 | The REPORT gives R, O, D and M, the sheet's row, the model's replies word for word, and what did not work first; every figure is in a named log. | the REPORT |
| G5 | One notebook row carries the outcome. | the notebook |

## Not established by this unit

- Anything the sheet's rows do not say. A Level result closes nothing. No milestone moves on this run alone.
- Stopping after a copied string, questions about things never told, and conversation quality beyond the nine turns read.

## Cancel probe (run first)

- **What would make this unit unnecessary:** a model already trained with the rule.
- **Result (at claim):** none. claude-357 trained nothing; the notebook has no training with a fall-back copy rule.

## Claim

Written by hand, as claude-347's to claude-357's were: claude-346 (PR #3913, a draft under the hold) holds the item for `uor-harness loop claim`.

## Files in scope

- `docs/loop/tasks/one-model-end-to-end/claude-358/**`
- `docs/research/notebook/**` (one row)

## Inputs

- The sheet and the pin above.
- claude-352's pod kit and stage 3's four sealed snapshots and artifacts: `local/claude/claude-352/`.
- claude-357's reading panel and the four comparison artifacts: `local/claude/claude-357/`.
- This run's kit, not committed: `local/claude/claude-358/`.
