# ORDER: one-model-end-to-end, deepseek-12

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `b45f37b44cba3c8360186233b5b568a13d930dfe`.

## Goal

Extend the CPU-only `m2-evidence` instrument from teacher-forced next-token
grading to deterministic complete-reply grading. The public 19-table M1 smoke
currently records generated complete replies and generated-reply novelty as
unavailable even though its artifact and paired prompt/reply panel are
available. Add an opt-in greedy decode that records completion, EOS, context
stops, exact-reference matches, repeated 4-grams and generated 4-/8-gram
novelty against the supplied reference corpus.

## Cancel probe

1. `m2-evidence` still records generated metrics as unavailable and has no
   generated-reply input or decode path.
2. The public M1 artifact, tokenizer, 58 paired prompts/replies, and generated
   control are locally available with pinned hashes.
3. The existing teacher-forced evidence is not a complete-reply result; a
   reply-id decoder would evaluate a distinct question.

## Files in scope

- `docs/labs/one-model-end-to-end/**`
- `docs/loop/tasks/one-model-end-to-end/deepseek-12/**`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/one-model-end-to-end.md`

## Done when

1. `m2-evidence` accepts an opt-in positive generated-token limit.
2. For each paired prompt/reply, greedy generation starts from the model's
   actual available document prefix, preserves context boundaries, appends the
   predicted token, stops at the tokenizer's added `<|eos|>`, and distinguishes EOS,
   token-limit and context-limit stops.
3. The report records generated reply ids and text, completion, exact-reference
   matches, repeated 4-grams, and generated 4-/8-gram novelty against the
   supplied reference corpus when available.
4. The teacher-forced, writer and full CPU trunk parity paths are unchanged.
5. Focused tests cover EOS completion, token and context limits, exact-reference
   matching, repeated 4-grams, novelty accounting and malformed limits.
6. The public 616-target panel is rerun with a bounded decode, with exact
   artifact/tokenizer/corpus identity retained.
7. The REPORT and notebook state exactly what completion and exact-reference
   matching prove and leave conversational correctness, fresh phrasing,
   training and GPU qualification open.

## Stop if

- Generation changes teacher-forced predictions, CPU/serving parity, model
  weights or the exported artifact.
- The panel has no valid prompt/reply pairing or its manifest EOS differs.
- A generated reply reaches the context boundary in a way the report cannot
  distinguish from EOS completion.
