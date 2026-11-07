# REPORT: one-model-end-to-end, claude-304

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [ORDER.md](./ORDER.md). Sheet: [`one-model-end-to-end-w1024-speed-2026-11-07.md`](../../../../harness/prep/one-model-end-to-end-w1024-speed-2026-11-07.md), which amends [claude-302's](../../../../harness/prep/one-model-end-to-end-w1024-2026-11-07.md).

## What did not work, first

- **There is no width-1,024 reading, and this unit will not produce one.** I stopped seed 1's training at step 2,048 of 16,384, by a decision review (below), after Casey asked whether the run's up-to-$88 cost would carry forward. This is a stop by decision, not a measurement: it closes nothing about width (§4.1, L-007).
- **Seed 2 never started.** Three RTX PRO 6000 pods, `vast-59497270`, `-59499851` and `-59502786`, never reached SSH, in 1,800, 1,800 and 1,200 s. Each was deleted. The pods' machine ids were read from Vast's live instance listing at the time, which no saved log keeps. Machine 48282 (driver 580.95.05) is recorded in `local/claude/FRICTION.md`. The later offer watcher's log shows the same driver on machine 48276. Treat the driver-580.95.05 pattern as a lead, not a finding.
  - `uor-pod`'s circuit breaker refused a fourth try, and `--force-retry` with a 0.99 reliability filter picked the same host again.
  - `uor-pod up` defaults to `--ref main`, which had moved past the job's pin. The third seed-2 pod (`vast-59502786`) booted at `195c2177…`, not the pin, so its run would have been off-pin even had SSH come up. A later pod needs `--ref 2578cd6b4d10d2d2beae24dcdc6692424d56e1f0`.
- **Micro-batch 64 does not fit.** The whole batch of 64 in one forward ran out of memory on the 97,887 MiB card.
- **Micro-batch 32 is barely faster than 16:** 3.516 against 3.641 s a step, about 3.4%. Halving the forwards a step saves little, so the step time is not mostly per-forward overhead.

## The decision review (the stop)

I read the chain before deciding: claude-302's question and outcome map, claude-301's result, and claude-295's width reading. Casey can reverse this.

- **The one-track bar is already met.** claude-301's (b64L) met it at width 256 ("met; unseen ahead"). This run could not add to that.
- **Only one outcome moves the trunk off width 256.** In claude-302's outcome map, only "(w1024) meets the bar and the width lever is `ahead`" makes the next unit width 1,024. A lever `inside` or `mixed`, a `behind` reading, and (w1024) `mixed` or `inside` against T2 each name "more human text at width 256". The fifth case, ahead of T2 but not of the unigram, is a grade check with no outcome drawn. The map's "anything else" case lets the lever and the spreads pick the next unit. For a NOT_RUN, the map reruns each missing seed under the sheet. This unit is a NOT_RUN at both seeds, and this decision review declines that rerun, for the reasons here; Casey can reverse it.
- **The evidence leans toward data, not width.**
  - claude-295's one width reading, 512 against 256, was `inside`, while its data lever (D3 against D1) was `ahead`.
  - claude-301's tokens lever was `ahead`.
  - Width 1,024 has about 16× the parameters at the same ~200M tokens, with an untuned learning rate. A clear `ahead` was the least likely outcome.
- **One seed reads nothing.** The bar needs both seeds, and seed 2 had no usable host.

**Decision:** stop, and keep the costs measured. The next unit is more human text at width 256, the outcome map's default.

## The readings

None. The sheet's Readings, Bar and width lever are unread.

## What this unlocks (sheet, outcome map)

- The outcome map's default: more human text at width 256, on RTX 5090s at about $0.56/h.
- Width 1,024 stays available if a later unit needs it, at the measured cost below: micro-batch 32, 3.52 s a step, about 18.4 h for 16,384 steps on an RTX PRO 6000.

## What changed

- **The sheet** (FIT WITH FIXES, all seven findings applied) and **the ORDER**.
- **The job, chain, seal and watch scripts** in `local/claude/claude-304/` (D67: GO WITH FIXES, all applied):
  - smokes at 64, 32 and 16;
  - an 80% memory bound at 64 and 32;
  - a missing reading drops a size, and a smoke run with no size read is an instrument fault;
  - the watcher logs its time and spend limits and keeps watching, to a 27 h hard stop.
- **No source change.**

## Measurements

Seed 1, `vast-59497268`, RTX PRO 6000 (97,887 MiB), pin `2578cd6b4d10d2d2beae24dcdc6692424d56e1f0`:

| Check | Result |
|---|---|
| I-0 | passed |
| I-e | PASS against 1e-5. Micro 4 and micro 2 within 1.76 × 10⁻⁷ of whole at batch 16 (step 2, micro4). The read-off control moved 1.07 × 10⁻⁶ at step 4, UNDERPOWERED (recorded, not gated). Memory: whole 36,957, micro4 29,661, micro2 29,629, read0 11,581 MiB |
| Smoke m64 | out of memory |
| Smoke m32 | I-1 PASS; I-m 76,125 of 97,887 MiB (under 80%); I-f 76,125 then 76,061; 3.515759 s a step |
| Smoke m16 | 3.640717 s a step (passed) |
| I-t | micro 32; used 2,165 s; projected wall 66,247 s (bound 72,000) |
| Training | stopped at step 2,048 by PID, after 7,857 s: the trainer exited 143 (`not-run.txt`: "training failed (exit 143, ok)"), and the job exited 6 (run incomplete). `train_loss_mean` 1.00014 at step 1,024 and 0.98842 at step 2,048. `dev_top1_hits` 218 and 213 |

**Seal:** `results/claude/one-model-end-to-end-w1024-speed-2026-11-07/runs/s1/`
- `w1024s-s1.tar.gz` `78cf4a1f61ff81eb6b0b03c15ce7d78a824c55b5028b8a0ac2de767961299a79`, 249.9 MB;
- `w1024s-s1.list` `371c9bc3a781faebae324347a3caf853e85320f33b192e17e5b2baab581f6577`;
- read back on the pod (`seal=0`), then the pod went down.

**Spend:** about $7.50 in all:
- seed 1: $5.13;
- the three failed seed 2 pods: about $2.40 at $1.789/h.

## Checks run at the head

- The job's laptop gate tests: 81 ok, 0 FAIL. Six mutations were each caught (exit 1).
- `bash -n` on every script.
- `preprules::` on the sheet: 17 passed.
- `loop report --check` on this REPORT: PASS.
- Every number in Measurements was checked against the sealed tarball, fetched back from the store and re-hashed to `78cf4a1f…` (`ie.txt`, `ie-mem.txt`, `im.txt`, `if.txt`, `it.txt`, `micro.txt`, `not-run.txt`, `EXIT`, `w1024-s1/progress.jsonl`).

## Skills used

None beyond the loop's own steps.

## Still open

- Width 1,024 has no quality reading. This unit's stop closes nothing about it.
- The Vast host problem (driver-580.95.05 machines that never reach SSH), and `uor-pod`'s lack of a host exclusion. Both are in `local/claude/FRICTION.md`.
- The served-form check (§3.7) is still owed on the adopted width-256 recipe.

## Plain summary

This unit set out to train a model four times wider than the one that already passed Casey's bar. When Casey asked whether the cost would pay off, I checked. Most possible results would have sent us back to the narrower model anyway, earlier evidence suggested more data helps more than more width, and the second seed could not get a working machine. So I stopped the run after about $7.50 instead of up to $88. I kept what it measured, mainly that the wider model fits only at micro-batch 32 and takes about 18 hours per run. Next comes more human text on the smaller model, which is cheaper and better supported by the evidence.
