# ORDER: CPU stratified M2 evidence lane, deepseek-1

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md). Base: `b1e377b18`.

## Goal

Build a CPU-only evidence instrument that evaluates a supplied token panel with the exact integer engine and reports the strata needed to distinguish TRAIN-reply recall from new behaviour. This is an evaluation instrument, not a training run or milestone claim.

## Cancel probe

Inspect the existing `docs/labs/one-model-end-to-end/instrument` and CPU corpus/panel code first. If an existing command already emits the required panel/TRAIN overlap, keyed/unkeyed, completion, novelty, admission and exact CPU/serving strata, record the command and stop without creating a duplicate.

## Scope

- `docs/labs/one-model-end-to-end/instrument/**`
- `docs/labs/one-model-end-to-end/**`
- `docs/loop/tasks/one-model-end-to-end/deepseek-1/**`

No edits to Claude-owned float-train, integer or Q5 files. No sealed panel is read without its registered input and credential. No model-weight update, GPU run or capability verdict.

## Done when

- A deterministic Rust instrument accepts explicit corpus, mask, manifest, tokenizer and
  integer-artifact paths.
- It reports artifact identity, target admission, top-1/top-8 and row counts.
- It independently recomputes the writer from the serving Q12 query and reports admitted-cell, top-8 and top-1 parity.
  - Full two-sided CPU-trunk parity is required only when the CPU path supports the artifact family.
  - The current 19-table M1 artifact has only writer parity; full trunk parity is reported `unavailable` with the missing mechanism.
- Where the inputs permit, it reports exact prompt/TRAIN overlap, value-masked prompt-template
  overlap, keyed/unkeyed status, complete/incomplete output, exact reply/TRAIN overlap and reply
  n-gram novelty in separate strata.
- Every unavailable metric is named `unavailable`, with the missing input or boundary, rather than inferred.
- Count-table, memory-off or no-read controls are reported where already available; missing controls remain explicit.
- Focused tests cover tie/boundary cases, absent fields, malformed corpus and artifact inputs, empty inputs, argument refusal, create-new output, deterministic output and a planted wrong metric.
- README and task REPORT state exact commands, measured output, limitations and the next M2 decision it informs.

## Stop

Stop on an unavailable sealed input, CPU/serving mismatch, malformed artifact or a metric that cannot distinguish TRAIN overlap from generation. Record an instrument result, not a model result.
