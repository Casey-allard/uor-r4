# ORDER: m4-export-uorlut01-cli, claude-259

Lab: claude. Item: [m4-export-uorlut01-cli](../../../../roadmap/items/m4-export-uorlut01-cli.md). Base: `22fecde5a`.

## Goal

The item's done-when reads:

> M4 bar: 'One model exported as an integer file under the serving contract (D11), usable from a command line, with held-out quality at or below 0.90 bits per byte'. In addition:
>
> - The export gap is reported (D37).
> - The static audits are green.
> - A fresh-process restore is bit-identical.

This unit does one part of it: "usable from a command line", for the trained model.

Casey, 2 Nov 2026: "your goal is to get us to a working chat model using the training and serving methods we have described - you need to figure out how to stitch all of that together". Today nobody can chat with the trained model. `uor-chat --stack a7r-s1.uorlut` (the exported FBS model of claude-243, `uor-r4.lut-stack/7`) fails before its first reply with "the engine returned 0 scores for a vocabulary of 4096".

There are two causes, both in `crates/uor-r4-integer/src/stack/chat.rs`:

1. `StackChat` reads `IntegerStackSession::step`, which returns an empty slice for an FBS model. An FBS model's token is the whole-table nearest pick, which `serve_step` returns for both kinds of model.
2. `StackChat` never calls `mark_reply_start`, so the turn memory's reply table (claude-194, the part that carries a reply past its first word) never fires in chat. `fbs-toks` marks it before the prompt's last token.

The fix: `StackChat` takes every token from `serve_step`, and marks the reply start before it steps the history's last token. Then this unit records real transcripts of a7r-s1, a7r-s2 and a7r-s3 from the command line.

**Claim check (run-claude.txt, every unit).**

- **(a) Step.** This advances PLACE.md's 2 Nov path, step 1: "Chat runs on the trained model". In the 29 Oct order it is step 3, the text path (R7), and it feeds step 4 (one model served and scored end to end).
- **(b) What it changes.** The serving path: the chat CLI serves the trained FBS model's own pick, with its turn memory, where today it serves nothing. No training change.

## Cancel probe (run first)

1. **Is the work already done or in flight?** No open PR touches `stack/chat.rs`, `stack/tests.rs` or `bin/uor-chat.rs`. Checked 2 Nov over every open PR's files: #3559 (this unit), #3558 (claude, one-model-end-to-end) and #3541 (kimi, Studio) list none of them. `git diff origin/main` is empty for both files.
2. **Does the failure reproduce?** Yes. `printf 'User: hello\n/quit\n' | uor-chat --stack .../claude-243/R/a7/a7r-s1.uorlut --tokenizer .../w2-inputs/2026-10-25/driver-data/tokenizer.json --max-tokens 16` prints `error: the engine returned 0 scores for a vocabulary of 4096`.
3. **Class: instrument.** The fault is in the chat front end (it reads a score row FBS models never fill), not in the model or its export. `fbs-toks` decodes the same artifact through `step_native` at about 1,200 tok/s (claude-243). So this is a repair to the serving instrument, and it says nothing about the model's quality.

## Files in scope

- `crates/uor-r4-integer/**`
- `crates/uor-r4-discrete/**`
- `crates/uor-r4-lut/**`
- `docs/mechanisms/lut-serving.md`
- `docs/model/serving-and-export.md`
- `docs/harness/prep/m4-export-uorlut01-cli*`
- `docs/labs/m4-export-uorlut01-cli/**`
- `docs/loop/tasks/m4-export-uorlut01-cli/**`
- `docs/research/notebook/m4-serving.md`

This unit edits only `crates/uor-r4-integer/src/stack/{chat,tests}.rs`, `crates/uor-r4-integer/src/bin/uor-chat.rs` (if its prompt-ids path reads the empty score row too), the task folder and the notebook.

## Done when

1. `StackChat` serves FBS artifacts. Each token comes from `serve_step`, and the reply start is marked before the history's last token is stepped. Dense models keep their current replies, because `serve_step` is the same `stack_argmax` for them.
2. These tests pass in `stack/tests.rs`, and the first two fail with the fix set aside:
   - **T1.** On a synthetic `/7` FBS artifact with a 512-id tokenizer, a reply equals a reference built from a fresh session: the reply start marked before the history's last token, then `serve_step` picks, with the EOS and short-cycle stops. Without the fix: `ScoreWidth`.
   - **T2.** Over the synthetic opening keys, at least one history gives different replies with and without the mark, and chat gives the marked one. Without the mark: it fails.
   - **T3.** A second turn that extends the session equals the fresh-session marked reference.
3. The existing `StackChat` tests and the crate's full test suite pass with `--no-fail-fast`.
4. Real transcripts of a7r-s1, a7r-s2 and a7r-s3 come from `uor-chat --stack`, with generator-style prompts and natural ones. They are recorded in the REPORT, negatives first, with tok/s, and as one row in `docs/research/notebook/m4-serving.md`.
5. `loop report --check`, `loop gates --full` and `loop pr-check --local` pass, and a fresh-context review approves the exact head.

The item stays open: its 0.90 bits-per-byte bar, the export gap and the restore check are not this unit's.

## Stop and ask if

Stop and decide (never stop and wait) if:

- `serve_step` and `fbs-toks`'s `forced_picks` disagree on a real artifact. That would be a serving fault, and it goes in the REPORT as the finding.
- A dense-model chat test changes its reply. Then `serve_step` is not `stack_argmax` for dense models. Keep the old path for dense models, and record why.
