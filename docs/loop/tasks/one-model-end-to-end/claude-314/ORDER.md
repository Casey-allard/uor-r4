# ORDER: one-model-end-to-end, claude-314

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `4845939bf`.

## Goal

**Prepare the run claude-313's decision review named:** (b64L) on a **D3 + `mw-balp/train` mix**, at claude-301's recipe and pin `ff5156bcc`, at seeds 1 and 2. The deliverables:
- the mixed TRAIN, built on the laptop and put in the store;
- a pre-registered sheet with a non-author FIT;
- the four run scripts, gate-tested;
- a pre-run review.

The run itself is the next unit.

Claim check: (a) the step is "pipeline first" (#2028, 6709947526). claude-313 (#3834) found that mw-balp makes (b64L) stop: 253 and 244 of 256. But it costs the held-out human text: behind (b64L-D3) on every split, pooled d about −304,000. (b) The mix is the one change from both arms: TRAIN alone. So stopping, collapse and the held-out cost can each be read against both (b64L-D3) and (b64L-mw).

## Harness exception, recorded

`loop claim` refused: deepseek-25's PR #3838 holds the item's claim. That PR waits on DeepSeek's reply-SFT job: a 1,024-step fine-tune from the (b64L-D3) checkpoint on an OASST2 + M-World mix. It is a different lever from this one, a from-scratch pretraining mix. This ORDER is written by hand, as claude-313's was, under Casey's "i approve you to make harness exceptions as needed". The sheet reads deepseek-25's result if it lands first.

## Cancel probe (run first)

1. No open PR or claim trains (b64L) from scratch on a D3 + mw-balp mix. #3838 fine-tunes; it does not pretrain.
2. D3's and mw-balp's stored shas are as claude-301 and claude-312 recorded.
3. `mix-chat-corpus` (or the path the sheet names) can read both stores. mw-balp has no `manifest.json` (claude-312's sheet).

## Steps

1. Choose the mix:
   - the ratio, by scored targets and by documents;
   - the EOS rate per million;
   - the passes over each part at 16,384 steps.
   Build it on the laptop and verify its shas; put it in the store read-only.
2. Write the sheet. Fix before any run:
   - the bars carried from claude-312: stopping at least 128 of 256 at both seeds, collapse under 64 at both, the panel 104 and 52;
   - the held-out pair against (b64L-D3);
   - the outcome map.
3. A fresh non-author FIT, at most two rounds.
4. Adapt claude-312's four scripts to the mix's TRAIN, gate-test them, and pre-run review them.

## Files in scope

- `docs/loop/tasks/one-model-end-to-end/claude-314/**`
- `docs/harness/prep/one-model-end-to-end-b64l-d3mw-*`
- `docs/research/notebook/float-training.md`
- `crates/uor-r4-training/src/bin/mix-chat-corpus.rs`, only if it must read a store without a manifest, with a test.

Scripts and scratch live in `local/claude/claude-314/`, and are not committed.

## Done when

1. The mixed TRAIN is in the store, with its shas, ratio and EOS rate recorded.
2. The sheet is merged, with its FIT and pre-run review.
3. The REPORT is written, negatives first, with one notebook row.

## Stop and ask if

Nothing waits on Casey (§5.1). Laptop only, no spend. Bound: one unit.
