# ORDER: repair CPU M2 evidence semantics, deepseek-2

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md). Base: `440aeae14`.

## Goal

Repair the merged CPU M2 evidence lane after exact-head review. Correct document BOS and UORT vocabulary binding, complete the value-masked template census, and exercise the strongest independent CPU parity available for each supported artifact family. This remains a CPU evaluation instrument, not a training run or capability claim.

## Scope

- `docs/labs/one-model-end-to-end/**`
- `docs/loop/tasks/one-model-end-to-end/deepseek-1/**`
- `docs/loop/tasks/one-model-end-to-end/deepseek-2/**`

No edits to Claude-owned float-train, integer or serving files. No model-weight update, GPU run, sealed-panel read or milestone qualification.

## Done when

- Prompt and reply runs exclude document BOS and the first-eight-token key uses the same post-BOS convention as the census.
- The UORT header vocabulary must equal the manifest and artifact vocabulary.
- Prompt and reply value-masked template overlap is reported at K=256 and K=1024, and K=1024 prompt-template status is a row stratum.
- Supported four-table artifacts receive full independent CPU query, admitted-cell, top-8 and token parity; the 19-table M1 boundary remains explicit and writer-only.
- Missing prompt/reply runs, empty evidence, malformed inputs, unknown arguments and report replacement fail closed.
- Public smoke evidence is regenerated and deterministic, with the 19-table M1 limitation and four-table parity result recorded separately.
- README and REPORT state exact commands, test results and limitations.
