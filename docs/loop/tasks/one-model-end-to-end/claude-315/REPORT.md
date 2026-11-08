# REPORT: one-model-end-to-end, claude-315

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [ORDER.md](./ORDER.md). Sheet: [one-model-end-to-end-b64l-d3mw-2026-11-07.md](../../../../harness/prep/one-model-end-to-end-b64l-d3mw-2026-11-07.md) (claude-314, #3839). Two Vast RTX 5090 pods at $0.563/h, $3.29 in all.

**Outcome row 5: at least 8 EOS stops at a seed, stopping not met.** The mix makes (b64L) stop far more often than D3 alone: 177 and 122 of 256 replies end on EOS, against 0 and 0. Seed 2 falls 6 short of the bar of 128 at both seeds. The sheet reads this as D3 diluting the stopping signal below the bar.

## What did not work, first

- **Stopping missed at seed 2: 122 of 256, against the bar of 128 at both seeds.** Seed 1 met it (177).
- **Seed 2's stopped replies collapse.** Among its 122 EOS-stopped replies, 75 are " I'm sorry you feel calm!" The bar is 64. Seed 1's most common EOS-stopped reply occurs 62 times, 2 under the bar, so seed 1 is near collapse too. Only 68 and 39 distinct reply texts appear over all 256 rows. That is fewer than (b64L-mw)'s 120 and 104.
- **No row is complete and no pair counts: 0 of 256 and 0 of 128 at both seeds.** The panel's bar is 104 and 52. Every served model scored so far has 0.
- **Both seeds write the next turn's role header.** EOS-stopped replies that contain "Assistant:" number 29 at seed 1 and 27 at seed 2. Over all 256 rows the counts are 35 and 51 (`jq` `contains("Assistant:")` over `rows[]`). Seed 2's second most common EOS-stopped reply, 22 times, is " My current jobs." followed by "Assistant: My current jobs." three more times. The model learned the dialogue's turn layout, not where its own turn ends.
- **The mix still costs held-out human text against D3 alone.** On the no-memory panel, (b64L-d3mw) is **behind** claude-301's (b64L-D3) at both seeds: pooled d = −14,975 and −23,833 rows, against a band of 8,858. That is about a fifteenth of (b64L-mw)'s cost (−304,197 and −307,764). This unit does not separate the two causes the sheet named: fewer D3 passes (4.73 against 6.84) and mw-balp's content.
- **Tokens per second is the bin timer's figure, not decode speed:** 487 and 657 tok/s. The timer re-parses the artifact on every row (claude-311). It is recorded, not compared.
- **The watchers ran unwatched for about 9 minutes.** The first pair ran as session background tasks and died when the session ended, about 23:35Z. They were restarted in detached `screen` sessions at 23:44Z (`rewatch-315.sh`). Both pods kept training, and no reading depends on the gap.

## What changed

- **No code.** Two seeds trained on the pods, then the sheet's Step 6 on the laptop.
  - The scripts were the four the pre-run review pinned: `job-314.sh` `ce50f738…`, `chain-314.sh` `7ca161a4…`, `seal-314.sh` `5a56d59f…` and `watch-314.sh` `565e5c3d…`.
  - The on-pod hashes of the first three were checked equal before launch (`launch-315.sh`).
  - The laptop reader `read-315.sh` is in `local/claude/claude-315/` (not committed).
- **Pods:**
  - Seed 1 ran on `vast-59634296`.
  - Seed 2's first pod, `vast-59634321`, never reached SSH. It was deleted unused, and seed 2 ran on `vast-59635959` (`retry-s2.sh`). This was a setup retry, not a NOT_RUN: no job had started.
  - Each watcher took its own pod down on `seal=0`: seed 1 at 02:09:16Z for $1.67, and seed 2 at 02:19:53Z for $1.62. No TIME or SPEND NOTICE fired, and 0 of this unit's pods are left running.
- **Seals:** `results/claude/one-model-end-to-end-b64l-d3mw-2026-11-07/runs/s{1,2}/`. Each was fetched to the laptop and passed `shasum -c SHA256SUMS`. Neither seed needed a rerun.

## Measurements

**The served replies on `m2-fresh256`** (`served-reply-panel` at bin `be3d4cb0…`, greedy, 64-token cap, rendering `one-user-turn/v1`):

| Seed | Artifact sha256 (I-x) | Stops EOS / BOS / cap / context | Complete | Pairs | Most common EOS-stopped | Distinct (all) | Most common (all) | Median reply | Output sha256 |
|---|---|---|---|---|---|---|---|---|---|
| 1 | `4072f1c201f55ddbd1c328aa3c691e26285ff1c75d86aaadc89428de95f5c037` | 177 / 2 / 77 / 0 | 0 | 0 | 62 | 68 | 62 | 15 | `8b4850b66e6e92bb5da76d1e29f15c94e1f9178a4b89b71da6091abd327db56d` |
| 2 | `b3d84b2964aff7568353a3a1553af417f69e6e776e39bd66e4bd3ef08d892cb1` | 122 / 0 / 134 / 0 | 0 | 0 | **75** | 39 | 75 | 64 | `e7b8c0342a9d24c8d02b310e9100ba0bfc2d4eb2747dfe7872eb59d04905778b` |
| (b64L-mw), claude-313 s1 | | 253 / 0 / 3 / 0 | 0 | 0 | 73 | 120 | 73 | 11 | |
| (b64L-mw), claude-313 s2 | | 244 / 0 / 12 / 0 | 0 | 0 | 40 | 104 | 40 | 18 | |
| (b64L-D3), claude-309 | | 0 / 0 / 256 / 0 | 0 | 0 | 0 | | | | |

- Both artifacts are 2,156,416 bytes.
- EOS stops per stratum (length2, length4, length8, reassert, update): seed 1 44, 56, 38, 18, 21; seed 2 29, 33, 31, 14, 15.
- Seed 1's most common EOS-stopped reply is " My's do I." followed by a fragment.

**The bars:**
- **Stopping** (at least 128 of 256 EOS stops at both seeds): **not met**, 177 and 122.
- **Collapse** (at least 64 identical EOS-stopped replies at a seed): **seed 2 collapsed** (75); seed 1 did not (62).
- **The panel** (at least 104 complete and 52 pairs at both seeds): **not met**, 0 and 0.
- **The held-out cost** (the pooled verdict against (b64L-D3)): **behind**.

**The no-memory panel** (`scale-probe grade`, `train=` the d3mw mix `1081853b…`/`5bf91524…`, no `clean=`). Grade sha256: seed 1 `e4714275…`, seed 2 `b9160b63…`.

| Pair | Pooled verdict | Pooled d, seed 1 / seed 2 | Band | Output sha256 |
|---|---|---|---|---|
| (b64L-d3mw) vs claude-301's (b64L-D3) | **behind** | −14,975 / −23,833 | 8,858 | `65f9fcb100c88be4d66854cd9c49ccd16ba09f106e59336a2bc8384bcc2403c6` |
| (b64L-d3mw) vs claude-313's (b64L-mw) | ahead | +289,222 / +283,931 | 5,291 | `d6c245293cab280a7c92c75a13c238d72be3d972122627d1324999c2d7a07176` |
| (b64L-d3mw) vs the mix's T2 | ahead | +74,188 / +75,882 | 1,694 | `418f53a6f1ee6ca1d8dc60a4164b22c82e7f898207ce0a218c30791a05b3478e` |
| (b64L-d3mw) vs the mix's unigram | ahead | +281,310 / +283,004 | 1,694 | `31af69dd04db085a95e0e17c40cdc97fbf5ac7d257d1023727259f513fd10aeb` |

**The DEV curve** (`progress.jsonl` `dev_nearest.hits`, every 1,024 steps):

| Step | 1,024 | 4,096 | 8,192 | 12,288 | 16,384 | Peak |
|---|---|---|---|---|---|---|
| (b64L-d3mw) s1 | 6,533 | 6,877 | 8,194 | 8,242 | 8,210 | 8,429 at 11,264 |
| (b64L-d3mw) s2 | 6,547 | 7,218 | 8,080 | 8,014 | 8,318 | 8,318 at 16,384 |
| (b64L-mw) s1, claude-313 | 6,904 | 8,439 | 8,672 | 8,278 | 8,246 | 8,672 at 8,192 |

**The loss** at step 16,384: 0.923 (seed 1) and 0.925 (seed 2).

**The smoke:**
- I-f passed at both seeds (26,736→26,864 and 26,704→26,800 MiB).
- I-m passed at both (26,864 and 26,800 of 32,607 MiB).
- I-t recorded 0.559 and 0.571 s a step, with W 10,205 and 10,404 s.

## Checks run at the head

- **On-pod script hashes** before launch: all three of `job-314.sh`, `chain-314.sh` and `seal-314.sh` equal the pins on both pods.
- **The job's own checks, from the seals:**
  - `EXIT` is 0 at both seeds.
  - `.i3` reads `ok` at both.
  - The no-memory `.i5` reads `same` on all five splits (oasst2, wiki-00000, wiki-00020, wiki-00040, dev) at both.
  - `score.exit` is 0 at both.
- **I-xc and I-c**, run on the laptop before any reading (`local/claude/claude-315/ctrl.txt`):
  - The re-export of claude-313's seed 1 hashes to `1c9ae43c…`.
  - The negative and positive scorer controls are row-equal to their references.
  - So row 0c does not apply.
- **I-x:** both artifacts' sha256 are recorded above. The scorer accepted both (exit 0).
- **The reader's pins:** `cpu-fbs-export` is `724a3b78…`, the scorer is `be3d4cb0…`, and the mix the grade read is `1081853b…`/`5bf91524…`. The reader refuses otherwise.
- **The collapse, distinct, median and stratum figures** are from `jq` over each output's `rows[]`.
- **`loop report --check`** on this REPORT.

## Decision review (row 5)

Row 5's next unit is the ratio lever: mw-balp's windows repeated in the mix, under its own sheet. The claude lab takes it, with two changes this unit's evidence asks for:

1. **The direction is right but the margin is thin.**
   - Stopping went from 0 and 0 on D3 to 177 and 122, so the mix carries the signal.
   - Seed 2 misses by 6.
   - mw-balp alone stops at 253 and 244.
   - So more mw-balp windows per D3 window should lift both seeds past 128. This is the cheapest lever on the outcome map.
2. **Collapse must stay a bar, and it is the likelier failure.**
   - Seed 2 is already collapsed (75), and seed 1 is at 62.
   - Repeating mw-balp's windows repeats its stock replies: claude-313's seed 1 collapsed at 73 on mw-balp alone.
   - So the ratio sheet keeps the collapse bar (under 64 at both seeds) fixed before any run. It reads distinct texts beside it, here 68 and 39.
3. **The held-out cost now has a ceiling to compare against.**
   - The mix is behind D3 by 15,000–24,000 rows, against mw-balp's 304,000.
   - The ratio sheet's held-out pair stays against (b64L-D3), with this unit's d beside it.
   - A ratio that buys stopping at more than about 3× this cost is a worse trade than a decode-time change. That threshold is the lab's judgement, recorded for the sheet's FIT to accept or change.
4. **The role-header text, at both seeds, is an end-of-turn fault, not a content fault.** The model writes "Assistant:" where an EOS belongs: 35 and 51 of 256 rows. The ratio sheet should count rows whose reply contains a role header. That is a `jq` count over `rows[]`, descriptive only, so it does not need its own instrument.

What a pass and a fail would each unlock:
- **Stopping met at both seeds with no collapse:** the reading is row 2 or 3 of a mix sheet. The next step is recall (the panel's complete bar).
- **Stopping met but collapsed:** the corpus's variety is the lever. That means a larger or more varied dialogue corpus, row 4's next.

This is the claude lab's own decision review (§5.1), logged for Casey, who can reverse it.

## Skills used

- None beyond the loop's own commands.

## Still open

- **The ratio sheet,** under its own unit: the repeat factor, its EOS targets per window, and the bars above fixed before any run.
- **The scorer's timer** still measures re-parse plus decode (claude-311). A decode-only figure is still owed for the tokens-per-second lead metric.
- **DeepSeek's fine-tuning lever landed first, as deepseek-26** (its notebook row, 7 Nov). It ran reply-masked training of (b64L) seed 1 on an OASST2 + M-World mix for 1,024 steps. Its free EOS stops were 16, 23, 21 and 1 of 256 at steps 256 to 1,024, with 0 complete. Set beside this unit's 177 and 122 from pretraining on the mix, the pretraining lever moves stopping further. The two differ in recipe, steps and seed count, so this is a description, not a pair. It changes no bar and no row, as the sheet said.
- **Watchers must start in `screen`, never as session background tasks** (the 23:35Z gap). claude-317's scripts already do.

## Plain summary

We retrained the same small model on a mix of plain human text and the dialogue corpus, at two seeds, on two rented GPUs for $3.29. It now ends most of its replies on its own (177 and 122 of 256, against none on plain text alone). But one seed falls 6 short of the bar, and its stopped replies are mostly the same sentence. None of the replies answer the prompts. The mix kept most of what the model knew about ordinary text, losing about a fifteenth of what the dialogue corpus alone cost. The next step is a mix with more dialogue in it, watched for repeated replies.
