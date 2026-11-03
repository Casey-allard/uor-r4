# ORDER: one-model-end-to-end, deepseek-14

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `640367b302af5a0aecd09d78fc66c9c1c23437c6`.

## Goal

Add the missing count-table arm to the complete-reply evidence instrument.
The public M1 model currently has greedy complete-reply readings but no
matched simple count-table reading on the same prompts, stop rules, tokenizer
and generated control. Without that arm, the report cannot say whether the
model's reply collapse is distinguishable from the control corpus's own
successor frequencies.

This is CPU development evidence. It does not replace a GPU qualification
bar, establish semantic correctness or claim useful conversation.

## Files in scope

- `docs/labs/one-model-end-to-end/**`
- `docs/loop/tasks/one-model-end-to-end/deepseek-14/**`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/one-model-end-to-end.md`

## Done when

1. The evidence instrument builds a deterministic transition count table only
   from adjacent tokens inside the supplied reference-corpus documents. It
   never joins tokens across a document boundary; ties choose the smaller
   token id; an unseen previous token uses the corpus-wide most common
   successor with the same tie rule.
2. On a requested complete-reply run, the instrument generates the same
   bounded greedy replies through the model and the count table, using the
   same prompts, context limit, token limit, EOS id and complete-reply
   measurements.
3. The report records the count-table source role, transition and prefix
   counts, table generation and a direct comparison with the model generation.
4. The public seed-1 M1 artifact and seed-2201 generated control are rerun at
   the PR head. The REPORT records both arms' completion, exact-reference,
   distinct-text, repetition, replacement-character and novelty readings.
5. A second run is byte-identical. The three absence controls remain explicit:
   memory-off and no-read require a trained full-path artifact; canonical
   TRAIN is not available on this host, so the count table uses the declared
   generated control.
6. Focused tests, strict package Clippy, rustfmt, report/roadmap checks,
   `git diff --check` and the local PR gate pass.

## Stop if

- The public artifact, tokenizer or generated-control hashes differ from the
  existing deepseek-13 pins.
- The count table crosses document boundaries, resolves ties
  nondeterministically, or produces a different reply trajectory on the
  repeated run.
- The existing teacher-forced, writer and CPU-trunk parity checks regress.
