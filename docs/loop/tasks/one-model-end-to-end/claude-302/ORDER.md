# ORDER: one-model-end-to-end, claude-302

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `826909863` (`origin/main`, 6 Nov 2026).

## Goal

Casey's one track (#2028 6675163438, "Yes, one track"): the trunk's no-memory top-1 must beat its TRAIN's unigram and its T2 on the human held-out panel, beyond the paired band, at every seed.

claude-301 (#3800, merged as `1f644d8acde8429e9b3c5bbc03f5089d34cfddcd`) met that bar with (b64L): width 256, batch 64 as four micro-batches of 16, 16,384 steps on TRAIN D3. The result reads "met; unseen ahead". The sheets' outcome map names the next unit: **width 1,024 on (b64L)'s recipe, with the unseen rows read again.** This unit pre-registers it and runs it, at seeds 1 and 2, and reads width 1,024 against claude-301's sealed (b64L) at equal steps and tokens.

## Claim check

- **(a) The step.** It is the outcome map's "an arm meets the bar" branch (claude-298's sheet, which claude-301's applies word for word).
- **(b) What it changes in the model.** It tests whether a four-times-wider trunk keeps the bar and gets more held-out human text right than width 256.

## Decision: the served-form check (§3.7)

claude-298's sheet says a bar-passing recipe "owes the served-form check (§3.7) in the next unit, before any serving claim". Casey parked serving, speed, integer and size work for claude in the one-track direction. This unit makes no serving claim. The check therefore stays owed, and it is carried in this unit's REPORT under "Still open", so that it runs before the first serving claim on this recipe. Casey can reverse this.

## Cancel probe (run first)

Run on 6 Nov, before any work. Class: instrument.
1. **No hold.** There is no `local/loop/STOP`, and #2028 has no `loop:pause` (`loop next`: kill switch off).
2. **The trainer since claude-301's pin** `ff5156bcc`. DeepSeek's #3803–#3806 add `query_cache.rs`, `fbs-query-cache` and 178 lines to `in_space_long.rs`, plus 32 to `train-in-space.rs`. The sheet must show that the training step at claude-301's recipe is unchanged by them, or pin `ff5156bcc` for training.
3. **Width 1,024 has never trained on this trainer.** It ran out of memory on a 32 GB 5090 at batch 16 (31,952 MiB) and batch 8 (29,456 MiB) in claude-295. `uor-pod` allows `--gpu pro6000` (96 GB, sm_120) when named.
4. **The caps:** `uor-pod status` shows 0 pods running.

Result: GO, with items 2 and 3 for the sheet to settle.

## Done when

1. The sheet is FIT by a fresh non-author review, and its FIT is posted on #3407.
2. The D67 pre-run review of the job and watch scripts passes in a fresh context, and is posted on #3407 before a pod goes up.
3. The runs, or the gates' NOT_RUNs, are sealed under `results/claude/one-model-end-to-end-w1024-2026-11-07/`, each with a `SHA256SUMS` that was read back. The pods are down.
4. The REPORT gives the negatives first, then: width 1,024 against D3's T2 and unigram by the Bar; the width lever against claude-301's (b64L); the spread; the gates; and one row in `docs/research/notebook/float-training.md`. It ends `References #3407`.

## Stop and ask if

"Ask" means decide, record it here and in the REPORT, and go on; nothing waits.
- **The sheet's stop rule.**
- **The cap:** the sheet's cost section sets it. When a watcher's stop fires on a healthy run, extend it and renew the lease; never cancel it.
- **Casey says "hold".**
- **The laptop's free disk falls under 5 GiB.**

## Files in scope

- `docs/harness/prep/one-model-end-to-end*`
- `docs/loop/tasks/one-model-end-to-end/**`
- `docs/research/notebook/float-training.md`
