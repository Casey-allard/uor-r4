# ORDER: one-model-end-to-end, claude-310

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `08c27ec2f`.

## Goal

**How often did (b64L) see EOS as a scored target?** claude-309 (#3828) found the served (b64L) never ends a reply: 256 of 256 panel replies ran to the cap, and the float model emitted EOS on 0 of 16 DEV prompts. The dialogue encoder writes EOS after every assistant reply and masks it as reply (`crates/uor-r4-tokenizer/src/dialogue.rs:11-14`). D3, (b64L)'s TRAIN, is 3.77M oasst2 tokens and about 22M Wikipedia tokens (its `manifest.json`). This unit counts the EOS targets the trainer scored, per source root. It is a laptop count with no model and no GPU (§4.5 needs no sheet).

Claim check: (a) the step is "pipeline first" (#2028, 6709947526): the score is closed, and the next training run must move it. (b) It changes nothing in the model. It says whether the stop is a data-exposure gap, which a corpus or loss-weight change can fix, before any spend.

## Cancel probe (run first)

1. No open PR or claim counts EOS targets in D3.
2. D3's `tokens.u16` and `response_mask.u8` from the store hash to `ed8f2537…` and `ec6e520c…`.

## The count, pinned before any reading

- **Mask meaning:** read from `train-in-space` at the pin `ff5156bcc` (how `response_mask.u8` selects scored targets, and whether the target at position t is token t or t+1). The REPORT quotes the lines.
- **Per root**, in manifest order, with each root's token range checked against `tokens_taken`: positions, scored targets, EOS tokens, EOS among scored targets, and BOS.
- **For oasst2:** scored reply targets between consecutive EOS (mean and median), and the share of scored targets that are EOS.
- **Reference:** the same counts on claude-261's `ft/dev`, the DEV split.

## Files in scope

- `docs/labs/one-model-end-to-end/instrument/**` (a new bin, `corpus-eos-exposure`, and its tests)
- `docs/loop/tasks/one-model-end-to-end/claude-310/**`
- `docs/research/notebook/float-training.md`

## Done when

1. The bin builds `--offline --locked`. Tests pin the mask convention and the per-root split on hand-made arrays.
2. D3 and DEV are counted, and the JSON outputs and their shas are recorded.
3. The REPORT, negatives first, and one notebook row. It says what the counts allow: whether an EOS-weight or corpus change is the next lever.

## Stop and ask if

Nothing waits on Casey (§5.1). Laptop only, no spend. Bound: one unit.
