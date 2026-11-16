# REPORT: written-state-count-model, claude-409 (decision card: the written-state track, the plan row, and the native-space items S0–S6)

Lab: claude. Doc-only. Base `origin/main` `ef1f4a641`.

## What did not work, first

- The card is not a row in `docs/integration/DECISIONS.md` [claude]. That file is an owner path: `loop pr-check` refuses it unless the PR carries an `## Owner approval` section quoting Casey with an ISO UTC time, and this unit has no such quote [claude]. The card lives at [DECISION.md](DECISION.md) in this task folder, where antigravity-114 also put its card [claude]. It proposes **D90** as the index row. That row is added only with Casey's words [claude].
- The review's subagents could not write REPORT.md: a hook refuses that file name to subagents [claude]. The top session wrote this REPORT [claude].
- `uor-harness check` gave no verdict: it printed nothing in 600 s at the base and was stopped [claude]. `loop pr-check --local` is the merge gate used here [claude].
- No `written-state-count-model` roadmap item exists yet, so the ORDER names `one-model-end-to-end`, the nearest existing item, because `pr-check` requires an existing one [claude].
- The PI review found that the current track cannot reach the goal as planned: per-coordinate scored training reaches about 0.5–1.1M coordinate trials per GPU-day, so 1e9 coordinates would take 900–2,000 GPU-days [claude, est, survey B2]. On ft/dev, every trained model scores below plain counts: bigram 6,300, best trained FBS arm 2,520 at its peak, M1-closing model 455 [claude, survey A1c].

## What changed

- `docs/research/reviews/pi-review-2026-11-16/`: the PI review record, ten files copied byte-for-byte (checked with `cmp`) plus a README index [claude]:
  - the surveys A1, A1b, A1c, A2, B and C, with C2's verification of every external number;
  - the PI synthesis;
  - red team R1 (technical) and red team R2 (charter and goal);
  - the PI's final DECISION.
- [DECISION.md](DECISION.md): the card [claude]. The written-state geometric count model track supersedes steps 3–6 of the owner's 16 Nov gap order (PLAN.md:38–39), and the card quotes the §2 bar verbatim [claude]. Step 1 (Pick) is done. Step 2 (conjunction memory) continues as P3's pointer conjunctions, and step 3 (readout) as the Pick/pointer readout [claude]. The card is logged for Casey, who can reverse it (§5.1, D68) [claude].
- `PLAN.md`:
  - rows 38–39 keep the owner's text and status, each with an appended note marking steps 3–6 superseded by the card [claude];
  - a new dated row names the track and says it is the lab's card, not an owner direction [claude];
  - the P0–P5 phases and their gates are tabled under Approved sequences [claude].
- [NATIVE-SPACE.md](NATIVE-SPACE.md): the D85 items for P1–P5 [claude]:
  - the state schema S0–S6;
  - the shared operator paths;
  - the replay oracle (the canonical triple-stream SHA-256);
  - the update rule (count, then integer Brier error-write);
  - the lossless format.

## Measurements

None. This unit is doc-only and ran nothing on a GPU [claude].

## What the numbers say

The bar the card adopts, quoted from DECISION §2 [claude]:
- **A:** ≤ 0.90 BPB on the pinned `chat-v0-p2` stream through PRLL.
- **Q (relative quality):** ≤ 0.80 × R\*, a sealed unpruned modified-KN, on D87 DEV-test.
- **R (replies):** the reply panels beat both the no-memory control and the shuffled-role control.
- **B (budget and speed):** ≥ 1e9 cells in ≤ 4 GB resident and ≥ 1,000 tok/s.

The label is "count-class, 1B-cell" until A, R and B all pass at once (DECISION §2). The honesty note adds that a miss on Q also keeps the count-class label [claude]. The first draft of this REPORT gave Q, R and B the wrong letters; the fresh review caught it. The top risk is that lawful counts with linear mixing stop near 0.90–0.95 × R\*, so the last ~10% rests on geometry, which P1's ceiling test and P2's equal-byte controls measure [claude, hyp].

## Checks run at the head

- G1: ORDER is the first commit (71b1e040e) and REPORT the last [claude].
- G2: `uor-harness check` gave no verdict within 600 s at the base [claude].
- G3: `loop pr-check --local` before this REPORT failed only on the missing REPORT. `order`, `owner-paths`, `banned-presence` and `rendered-lock` passed [claude]. Advisory warnings: the ORDER is over 4,096 B; RED-R2's quoted rule headings trip `rule-sources`; FILEMAP has stale rows [claude]. Rerun after the review fixes: verdict PASS. These checks passed: `order`, `banned-presence`, `report`, `owner-paths` and `rendered-lock`. The advisory warnings were `scope`, `commit-order`, `rule-sources`, `read-path` and `filemap`. The local `scope` list names files outside this diff, so it was measured against a lagging local base. The PR's own diff is 16 docs files [claude].
- G5: fresh review. An Opus reviewer in a fresh context returned APPROVE-WITH-LOW-ITEMS. The medium item, the mislabelled bar letters, is fixed. The low items are also fixed: NATIVE-SPACE now separates the exact replay stream from the declared one, and names the CPU oracle as a D84 parity check that qualifies no bar [claude].
- G4: a secret/email grep over `docs/research/reviews/` found no hits [claude]. A2 contains one absolute home path, which is not a secret; 111 other repo docs have the same kind of path [claude].

## Skills used

`native-space-cycle` (D85 items), the loop's ORDER/REPORT shape, and a PI review with two adversarial red teams run from the top session [claude].

## Still open

- The D90 index row in DECISIONS.md is added only with Casey's words [claude].
- A `written-state-count-model` roadmap item is not created yet [claude].
- In P0, still to land [claude]:
  - claude-410 (PRLL floor);
  - claude-411 (D87 split and 13-gram leak audit);
  - claude-412 (GPU exact n-gram counter);
  - claude-413 (R\*, scored on the GPU).
- PPMd is demoted to a non-gating diagnostic: a CPU run cannot qualify a bar (§3.5) [claude].
- In the owner's queue, as proposals that nothing waits on [claude]:
  - P6, more donor data;
  - a §3.2 ruling on a linear-domain calibration table;
  - the label;
  - a conditional §3.1 proposal, only if every P2 geometric arm loses.

## Plain summary

The lab reviewed the whole project against Casey's goal of a 1B-equivalent model, trained on wick and served on the laptop with no matmul and no softmax [claude]. Searching for a billion numbers one at a time would take years on one GPU, and counting tables already beat every trained model [claude]. So the track changes: write the model's state by exact counting inside the existing trunk tables, keyed by ordered primes and E8-snapped geometry, refine it on wick, and train only a small geometric core [claude]. This PR records that decision, the evidence and the plan [claude].
