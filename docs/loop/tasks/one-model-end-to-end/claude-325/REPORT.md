# REPORT: one-model-end-to-end, claude-325

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). ORDER: [ORDER.md](ORDER.md). Sheet: [one-model-end-to-end-b64l-d3mwcs-2026-11-08.md](../../../harness/prep/one-model-end-to-end-b64l-d3mwcs-2026-11-08.md), at pin `ff5156bcc`.

## What did not work, first

- **The panel bar is not met: 0 of 256 complete and 0 of 128 pairs, at both seeds.** Verdict: **negative**, for this configuration only: (b64L) on the d3mwcs mix, 16,384 steps (§4.1).
- **Outcome row (e) matches: no change from claude-322 in the reply form.** Rows 0a–0c, (a), (d), (b) and (c) do not match:
  - **Not (d):** neither seed collapsed (7 and 6 of the stopped rows, against 64), and stops are not below claude-322's at both seeds (115 against 109 at seed 1; 66 against 86 at seed 2).
  - **Not (b):** complete is 0 at both seeds.
  - **Not (c):** bare is 0 at both seeds.

  Row (e) reads that the current-state corpus's share is too small to move the reply form.
- **No reply is bare.** The 40,000 bare-answer documents did not move the opening frame:
  - Seed 1 stopped 115 replies. 56 are short, and none is bare. Among all 115 stopped replies, 43 open "Your friend's name is" and 17 open "Got it, your friend is".
  - Seed 2 stopped 66. 9 are short, and none is bare. Among all 66 stopped replies, 0 open "Your friend's name is" and 19 open "Got it, your friend is".
  - There is no bare reply to quote. The most common stopped replies at seed 1 are " Your friend's name is basalt loggia." (7), " Your friend's name is feldspar gallery." (6), " Your friend's name is syenite annex." (6) and " Got it, your dog is a dry cabin." (4).
- **The asked value is not chosen.** The first line holds the asked-for value in 27 and 0 rows, and the other value in 29 and 3 (claude-322: 14 / 11 and 2 / 6).
- **Stopping fell at seed 2,** from claude-322's 86 to 66; 189 of its 256 replies ran to the cap.
- **The reply's first token is a frame word,** whichever relation is asked. Teacher-forced, the reference's first token is top-1 on 0 of 256 rows at both seeds, under both renderings. The model's own top-1 at that position is " Your", " G", " I" or " It" on every row. This reading is claude-327's Part A, run on these artifacts.

## What changed

- The sheet (committed before the pods: draft, FIT round 1 fixes, FIT and pre-run review) and this REPORT.
- One notebook row in `docs/research/notebook/float-training.md`.
- No code, bar, status, scope or cap changed.

## Measurements

Both seeds ran at `ff5156bcc` on two RTX 5090s (vast-59750631 and vast-59748529): 7,948 s and 7,897 s of training, about 26,100 context tokens/s, $1.38 + $1.35 = $2.73. Each was sealed to `results/claude/one-model-end-to-end-b64l-d3mwcs-2026-11-08/runs/s{1,2}/`, read back, fetched to the laptop and checked against its `SHA256SUMS` (both OK).

| Reading | Seed 1 | Seed 2 | claude-322 (no corpus) |
|---|---|---|---|
| Complete / pairs | 0 / 0 | 0 / 0 | 0 / 0 |
| EOS stops of 256 (cap stops) | 115 (141) | 66 (189) | 109 / 86 |
| Collapse among stopped | 7 of 115 | 6 of 66 | 9 / 6 |
| Distinct texts; most common over all | 118; 12 | 116; 9 | — |
| Median reply length (tokens) | 64 | 64 | — |
| EOS stops per stratum (length2, 4, 8, reassert, update) | 34, 17, 20, 30, 14 | 16, 13, 29, 7, 1 | — |
| bare.jq: short / bare / span | 56 / 0 / 0 | 9 / 0 / 0 | 68 / 0 / 0 and 16 / 0 / 0 |
| Opening templates: "your friend's name is" / "got it, your friend is" | 43 / 17 | 0 / 19 | 73 / 28 and 1 / 36 |
| First line: asked / other / both | 27 / 29 / 0 | 0 / 3 / 0 | 14 / 11 / 0 and 2 / 6 / 0 |
| Role headers `Assistant:` / `User:` (stopped; all) | 0 / 0; 1 / 0 | 0 / 0; 0 / 0 | — |
| First target, teacher-forced, one-user-turn: top-1 / top-8 / top-32; median rank | 0 / 4 / 12; 2,335 | 0 / 3 / 8; 1,486 | 0 / 1 / 8; 1,282 and 0 / 0 / 0; 2,029 |
| Model's own first token, one-user-turn | " Your" 102, " G" 86, " I" 54, " It" 14 | " Your" 172, " G" 84 | " Your" 172, " G" 84 (s1; `local/claude/claude-327/a1/claude-322-s1-one.json`) |
| DEV nearest hits at step 16,384 | 7,061 | 6,396 | 7,041 / 6,958 |
| train_loss_mean / dev_loss at the last step | 0.853 / 2.350 | 0.914 / 2.354 | — |

**Held-out pairs** (`scale-probe grade` with `train=` claude-324's mix, then `pair`):
- **Deciding, against (b64L-D3):** `inside` at both seeds, d +1,682 and −48,178, mean −23,248. That is inside the ceiling of −58,212 and not `behind`.
- **Row (f), against claude-322's (b64L-d3mwcap6):** `inside` at both, d +17,138 and −22,551, mean −2,707. That is above −58,212, so row (f) does not match.
- **Descriptive:**
  - against t2: ahead, ahead;
  - against unigram: ahead, ahead;
  - against (b64L-mw): ahead, ahead;
  - against (b64L-d3mw): inside, inside;
  - against (b64L-d3mw3): ahead, inside;
  - against (b64L-d3mwcap): ahead, ahead.

DEV curves, every 1,024 steps, are in `local/claude/claude-325/dev.log`. Seed 1 rises to 7,061 and seed 2 to 6,396, beside claude-322's 7,041 and 6,958.

## Checks run at the head

- **On each pod:**
  - **I-0:** `cargo test --release --locked -p uor-r4-float-train --features cuda --lib`: 385 passed, 0 failed, 12 ignored, with the three micro-batch tests passing by name.
  - **I-p:** the job at the pin.
  - **I-3:** every input matches the Data split, and the windows read exactly 457,421 at both seeds.
  - **I-1:** smoke PASS.
  - **I-m:** PASS (26,768 of 32,607 MiB).
  - **I-f:** PASS.
  - **I-t:** recorded.
  - **I-5:** `same` on every split at both seeds.
  - **DEV agreement:** memory and no-memory DEV agree.
- **On the laptop:**
  - **I-xc:** ok, `1c9ae43c…`.
  - **I-c:** the negative and positive references' rows are equal (`read-325.sh ctrl`, exit 0).
  - **I-x:** artifacts `ac570514…` (s1) and `a3301596…` (s2).
  - `serve-s2.log`'s line "s1: no step-016384 checkpoint: NOT_RUN" is from a serve run made before seed 1's seal landed. It is not row 0a: `serve-s1.log` then scored both seeds.
  - `uor-harness loop report --check --order`: run at the head before review.

## Skills used

- None beyond the loop itself (ORDER, the sheet's FIT and pre-run review, the fresh review).

## Still open

- **Row (e)'s next step is a repeat factor for the corpus, under its own sheet. It is held:**
  - claude-326's lab decision allows no one-knob language run on this line until claude-327's Part A is read and its Part B has run.
  - Part A now says the first token is a frame word on these artifacts as well, so a larger share of bare replies is still the reply-form lever.
  - Part B's query-recall probe decides whether the read can choose the value at all.
- **Why 40,000 bare documents did not move the frame** is not measured. They are about 8.7% of the mix's 457,421 windows. The likeliest source of the frame is the capped copies' replies, which open with these templates in claude-322 as well; that is a hypothesis for the repeat-factor sheet to measure, not a reading taken here.

## Plain summary

We added 40,000 short "just the answer" replies to the training mix and trained the same model at two seeds. The model still opens every reply with a stock word ("Your…", "Got it…", "I…", "It…"), never with the bare value. It still never names the asked-for value more often than the other one, and none of its 256 replies is fully right. It did not get worse on held-out text. The next step, more of those short replies, waits until the probe that asks whether the model's memory can pick the right fact at all has run.
