# ORDER: public integer-chat baseline, deepseek-6

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md).
Base: `b44402e0c`.

## Goal

Measure what the current public integer FBS artifact actually emits through the
product `uor-chat` path on a bounded public prompt panel. Record exact
transcripts, reply ids, stop reasons and decode speed for two published model
seeds, including two multi-turn memory probes.

This is an ungraded integration baseline. It does not qualify language
capability and does not replace Claude's capacity or sealed-panel runs.

## Cancel probe

Check whether a merged report already contains `uor-chat` transcripts and
throughput for both public dual-c131 seeds on the current public conversational
panel. If so, reuse it and stop.

## Scope

- `docs/harness/prep/one-model-end-to-end-public-chat-baseline-2026-11-02.md`
- `docs/harness/prep/one-model-end-to-end-public-chat-post-smoke-2026-11-02.md`
- `docs/labs/one-model-end-to-end/**`
- `docs/loop/tasks/one-model-end-to-end/deepseek-6/**`
- `docs/roadmap/items/one-model-end-to-end.md`

No model, serving-format, tokenizer or training code changes.

## Required design

- Pin both public FBS artifacts and both tokenizers by SHA-256.
- Pin the public panel, executable and executable source by SHA-256.
- Select eight prompts across all six categories of the public 160-prompt
  conversational panel before seeing outputs.
- Use the trained dialogue protocol 2, greedy decoding, 16 maximum response
  tokens and 16 worker threads.
- Run one-turn prompts in one-shot `--say` mode. Run each of the two memory
  entries in its own REPL process: one with two user turns and one with three.
- Capture `/tokens` before and after every REPL turn, and define the extraction
  of each exact reply-id suffix from the post-turn history before inference.
- Repeat one seed to confirm deterministic transcripts.
- Record exact response text and ids, stop reason, latency, ids/s and history
  depth.

## Done when

- The post-smoke FIT-reviewed prep sheet and exact commands are committed
  before protocol-2 outputs are read.
- Both seeds complete every selected prompt without a crash or protocol error.
- The report shows the exact transcripts and timing and classifies whether this
  public artifact is ready to demonstrate useful conversational behaviour.
- The report preserves the ungraded boundary: no answer-correctness,
  complete-reply, fresh-phrasing or language-capability claim is made.
