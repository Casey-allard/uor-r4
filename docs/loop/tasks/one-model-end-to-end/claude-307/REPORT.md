# REPORT: one-model-end-to-end, claude-307

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [ORDER.md](./ORDER.md). Sheet: [`one-model-end-to-end-d3-tokens-2026-11-07.md`](../../../../harness/prep/one-model-end-to-end-d3-tokens-2026-11-07.md).

## What did not work, first

- **There is no 32,768-step reading, and this unit will not produce one.** The unit was stopped by the owner's decision before any training ran. Two RTX 5090 pods, `vast-59596793` (seed 1) and `vast-59596829` (seed 2), came up at $0.563/h each. Seed 1 had finished its bootstrap (`up-s1.log`: parity PASS, SSH ready) and seed 2 was still waiting for SSH. Both were deleted before any script was copied or any job started. This is a stop by decision, not a measurement: it closes nothing about training length (§4.1, L-007).
- **The training-to-serving pipeline is not closed, and this unit's runs would not have closed it.** No (b64L) checkpoint has been exported or served. The served-form check that claude-301 named as the next step never ran, because claude-302 to claude-304 each stopped early. In the sheet's row 4 ("twice the steps buy nothing measurable"), this unit's checkpoint would have gone unused.
- **D67 minor 6 was not done:** `gt/test-seal.sh` does not cover the store's empty-root probe, because the job hard-codes `/root/uor-store`. It is moot here, since nothing was sealed.

## The decision (the stop)

In Claude's chat, Casey asked whether every gap from training to serving is closed, and whether these runs teach the model anything or are guess-and-check tests. Casey said they would rather have everything else done before dedicating time to training. Claude answered no on both counts and asked: "Redirect claude to close the pipeline end to end before any more training runs?" Casey answered **"Yes, pipeline first"** (recorded verbatim on #2028, 6709947526).

- The pods were taken down before the answer, so nothing billed while Casey decided. A new pod bootstraps in a few minutes, so the stop could be reversed cheaply.
- What this lifts and what comes next are recorded on #2028. For claude, the parking of serving and integer work (6675163438) is lifted for this purpose. claude-301's sealed (b64L) `step-016384` checkpoint goes through export, integer serving, a served-equals-trained check and the reply panel, mostly on the laptop. No new training run starts until that path works.
- Casey's ~$9 approval for this unit (6709527956) is not spent.

## What this unlocks

- The next claude unit on this item: (b64L) end to end, export → serve → parity → reply panel, under its own ORDER.
- This sheet and its scripts stay ready: re-running it is one launch, under a fresh approval, once the pipeline works.

## What changed

- **The ORDER and the sheet:** FIT round 1 FIT WITH CHANGES; round 2 FIT; D67 GO WITH CHANGES, majors 1–2 and minors 1–5 applied.
- **The scripts** in `local/claude/claude-307/` (job, chain, seal, watch, read). Gate tests: 30 ok; 13 of 13 mutations caught; seal tests green.
- **No source change.**

## Measurements

None from training: the sheet's Readings, Bar, tokens lever and mid-run reading are unread. I-r ran on the laptop before the launch (7 Nov): PASS, 120 fields equal (`local/claude/claude-307/ir/result.txt`).

Spend: two pods at $0.563/h. The compute ledger (`~/.local/share/uor-r4/compute`) records uptimes of 356 s (seed 1, down 16:33:36Z) and 350 s (seed 2, down 16:33:52Z): about $0.11 together.

## Checks run at the head

- The job's laptop gate tests: 30 ok, 0 FAIL (`gt/test-gates.txt`). 13 of 13 mutations caught (exit 1 on each).
- The seal tests: green, with 2 of 2 seal mutations caught.
- `bash -n` on every script.
- `loop report --check` on this REPORT.

## Skills used

None beyond the loop's own steps.

## Still open

- No (b64L) checkpoint has been exported, served in integers, checked served-equals-trained, or scored on the reply panel. That is the next unit.
- The 32,768-step tokens lever has no reading. This sheet can be re-run under a fresh approval once the pipeline works.
- D67 minor 6: the seal test does not cover the store's empty-root probe.

## Plain summary

This unit was about to spend about $9 training the current best model for twice as long. Casey asked whether that would actually feed the model, or whether it was guess-and-check while the path from training to a usable, served model was still unfinished. The honest answer was that the path is unfinished: no trained model of this kind has been exported or served yet. So I stopped before any training ran, spending about $0.11. Next, the existing trained model goes through export, serving and the reply tests first, so later training lands on a path that works.
