# ORDER: one-model-end-to-end, deepseek-13

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `686d9c872aa3b7cb650f2ffae73d93708829827f`.

## Goal

Test whether the public M1 complete-reply negative survives an unused
held-out-phrasing draw. Generate a new `dev,dev` panel and an independent
`train,train` generated control at seed 2201, verify that the panel introduces
prompt templates, then run deterministic greedy complete-reply grading over
the public seed-1 19-table M1 artifact.

This is CPU development evidence. It cannot replace a GPU qualification bar.
It tests prompt-template freshness, not semantic correctness, unseen words or
useful conversation.

## Cancel probe

1. `m2-evidence` already grades public M1 complete replies on the seed-4201
   dev panel; that panel has been read and is no longer a fresh draw.
2. Seed 2201 is unused by the one-model panels documented in this repository.
3. The canonical held-out-phrasing cell is `dev,dev`; `train,train` is the
   generated positive control.

## Files in scope

- `docs/labs/one-model-end-to-end/**`
- `docs/loop/tasks/one-model-end-to-end/deepseek-13/**`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/one-model-end-to-end.md`

## Done when

1. `heldout-panel` writes a 64-conversation seed-2201 `dev,dev` panel and a
   same-seed, same-size `train,train` control with the pinned public M1
   tokenizer.
2. The panel and control hashes, sizes, prompt/reply counts and tokenizer
   identity are recorded. The control is declared a generated control, not
   canonical TRAIN.
3. `m2-evidence` reports the panel's prompt-template share at K = 1,024 and
   records `new_prompt_templates_at_1024` (share at most 0.5).
4. The public seed-1 M1 artifact is run on the panel with
   `generate-new-tokens=16`, recording completion, EOS/token/context stops,
   exact-reference matches, repeated 4-grams, distinct non-empty and empty
   texts, replacement characters and EOS-inclusive 4-/8-gram novelty against
   the generated control.
5. A second public run at the same head reproduces the report byte for byte.
6. Teacher-forced scoring, writer parity and full CPU trunk parity remain
   exact under the new panel.
7. The REPORT compares the result with deepseek-12 and states whether the
   non-chat readiness negative survived. It leaves semantic correctness,
   fresh reply wording, useful conversation, training and GPU qualification
   open.
8. Focused tests, strict package Clippy, rustfmt, report, roadmap, diff and
   local PR checks pass.

## Stop if

- The pinned artifact, tokenizer or generated corpora fail identity checks.
- The K = 1,024 prompt-template share fails to establish a new-template
  panel; the run is then a repeated-template diagnostic, not fresh phrasing.
- The two deterministic decode reports differ.
- Teacher-forced predictions, writer parity or CPU trunk parity fail.
