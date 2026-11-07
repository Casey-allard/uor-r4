# REPORT: one-model-end-to-end, claude-308

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [ORDER.md](./ORDER.md). Laptop only; no GPU, no spend.

## What did not work, first

- **The quality-parity reading did not compare like with like, so it does not show served equals trained.** On the same oasst2 held-out panel (tokens sha `02a04e97…`, 184,542 response rows on both sides), the served artifact's nearest top-1 is **47,986** (seed 1) and **50,076** (seed 2). The float model's is 44,210 and 45,725. The served model scores higher, by 3,776 rows on seed 1 (4,351 on seed 2). Rounding is not expected to make a served copy beat its own weights by 8 %. It is an instrument difference. The two counts come from two scorers that give each row a different amount of preceding text:
  - The float scorer (`in_space_long.rs`, through `hamilton_sheet::build_document_windows`) cuts each document into back-to-back chunks of `context` targets. The first target of every chunk after the first reads one earlier token, and later targets read more, up to a full window.
  - `cpu-checkpoint-grade` reads each row through `CpuMaskedCorpus` with a full sliding prefix, `prefix_start = max(target − 192, doc_start)` (`corpus.rs:395-399`).

  So from the code alone, every row's prefix in the CPU grader is at least as long as in the float scorer. That fixes the direction of the gap. I did not measure each row's prefix on either side, so how much of the gap this explains is likely, not proven (§4.2). The CPU grader's `--baseline` takes only another integer artifact, so the trained float model cannot be scored with the same tool without new code.
- **Done-when 2 read 16 prompts per set, not DEV's 310.** The seal holds 16 float greedy replies per set (DEV and oasst2), and the I-3 reading compares against those. A 310-prompt reading needs the float model's replies on all 310, which needs a GPU forward. The ORDER rules that out.
- **One of the 27 differing replies is the 192-token cap, not a divergence.** The cap applies to prompt plus reply. Seed 2's DEV row 11 has a 176-token prompt, so the integer reply stops at 16 tokens (`end: "context"`), and those 16 equal the float reply's first 16. The other 26 are real divergences, first at reply offsets 0 to 20. Seed 1's DEV row 11 has the same prompt, but it diverges at offset 4, before the cap.

## What changed

- **No source change.** This unit adds this REPORT, the ORDER (with its amendment before any reading) and one notebook row.
- The exports, I-3 script and results, and grading outputs are in `local/claude/claude-308/` (not committed: artifacts and scratch).

## Measurements

**Export (done-when 1).** `cpu-fbs-export` (`724a3b78…`) from the sealed `step-016384` checkpoints:

| Seed | `UORLUT01` sha256 | Equals deepseek-239's |
|---|---|---|
| 1 | `9a370566cd9da163191c0196763c68ddb395927fcf61fdecc1736b04941a4bd9` | yes |
| 2 | `5039cadefd60633bcff5ac8511aff362e1174ef4cda20844a8febad625d2e304` | yes |

**I-3 (done-when 2).** `fbs-toks --replies` (`11cc0892…`) on each artifact, against the float model's 32-token greedy replies in the seal:

| Seed, set | First tokens differ | Whole replies differ | Order mismatches | First differing offsets (row:offset) |
|---|---|---|---|---|
| 1, DEV | 1 of 16 | 6 of 16 | 0 | 2:0, 6:8, 7:5, 8:1, 10:6, 11:4 |
| 1, oasst2 | 0 of 16 | 9 of 16 | 0 | 3:4, 4:3, 5:2, 6:6, 7:2, 9:6, 10:2, 11:9, 15:7 |
| 2, DEV | 0 of 16 | 8 of 16 (one the cap) | 0 | 0:3, 2:3, 4:7, 6:3, 9:14, 11:16 (cap), 12:4, 15:8 |
| 2, oasst2 | 0 of 16 | 4 of 16 | 0 | 1:3, 7:4, 9:20, 13:6 |

**Quality (done-when 3).** `cpu-checkpoint-grade --max-evaluation 184542` on the oasst2 held-out split; its audit found 0 token mismatches and 0 query-coordinate mismatches over its 192 checked positions:

| Seed | Served top-1 (CPU grader) | Float top-1 (float scorer, seal) | Served − float |
|---|---|---|---|
| 1 | 47,986 of 184,542 (26.0 %) | 44,210 (24.0 %) | +3,776 |
| 2 | 50,076 of 184,542 (27.1 %) | 45,725 (24.8 %) | +4,351 |

**Speed.** On the laptop, CPU:
- **Decode:** `fbs-toks` decoded 496 to 512 reply tokens in 1.02 to 1.21 s per set of 16, which is about **410 to 500 tokens/s** (about 460 on average), including the artifact load. The figures come from a re-run of `i3-308.sh` with its output saved (`i3-rerun/run.out`). That run's I-3 counts and offsets equal the first run's.
- **Grading:** the grader took 1,902 s (seed 1) and 1,683 s (seed 2) of wall time for 184,542 rows each (97 and 110 rows/s).

## Checks run at the head

- The four sha256 values above, recomputed with `shasum -a 256` (`i3/binaries.sha256`).
- Each grade JSON's `artifact_sha256` matches its export, and its `corpus.tokens_sha256` matches the float rows' `panel_sha256`.
- I recounted the float top-1 from `rows-oasst2-no-memory.json` (`targets` against `nearest`): 44,210 on 184,542 rows, matching `oasst2.score.out`.
- `loop report --check` on this REPORT.

## Skills used

None beyond the loop's own steps.

## Still open

- **Served against trained on one scorer.** The cheapest route is to make `cpu-checkpoint-grade` (or the float scorer) emit each row's pick and context length, then compare the two on rows where both read the same prefix. That needs a source change outside this ORDER's files.
- **No whole-reply score yet.** No tool scores a `UORLUT01` artifact's replies on `m2-reply512` or `m2-fresh256`. This is the next unit: a small CPU tool that renders each panel packet into a prompt, generates with the served artifact, and applies the panel's own acceptance rule.

## Plain summary

The trained model now exports, and the exported integer model serves, on the laptop at about 460 tokens/s. Its first words match the trained model's on 63 of 64 test prompts. Its whole replies drift apart a few words in, on 26 of 64; one more reply was cut short by the 192-token limit.

To check whether that drift costs quality, I scored the served model on 184,542 held-out answer tokens. It picked the right next token more often than the trained model's recorded score: 26 % against 24 % on seed 1. That is not a believable gain: an exported copy is not expected to beat its own weights. The two numbers come from two different scoring tools, which show the model different amounts of earlier text. So the path from training to serving runs end to end, but it does not yet prove the served model is as good as the trained one. That needs both models scored by the same tool. The next unit builds the scorer for whether a served model's replies are correct answers.
