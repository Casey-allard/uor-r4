# ORDER: one-model-end-to-end, deepseek-43 (served-panel run-path repair)

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `59494646c`.

## Goal

Repair the second exact-head review of PR #3895 by pinning the complete
`served-reply-panel` run path: sidecar argument, artifact/vocabulary
validation, model attachment, report identity and serialized output.

## Finding to repair

At `9edd4127e`, the tests call `attach_content_readout` and
`record_content_readout` directly. They do not exercise `run`. Removing the
sidecar attachment or report-recording call from `run` would leave every test
green, so the product path is not fully pinned.

## Design

- Split `run` into a thin environment wrapper and
  `run_with_args(args)`, so tests can invoke the real run path without changing
  process arguments.
- Add a minimal valid synthetic panel fixture:
  - a 256-wide nearest-head integer artifact;
  - a 259-token byte-BPE tokenizer with BOS, EOS, UNK and the byte alphabet;
  - 256 cases in the five required strata and 128 pairs;
  - hash-pinned inputs, labels, plan and manifest;
  - an always-firing but artifact-bound content readout.
- Call `run_with_args` with `--engine integer` and `--content-readout`.
- Assert the serialized report carries the exact sidecar SHA and
  `settings.content_readout = "attached"`.
- Call the same run path with wrong-artifact and outside-vocabulary sidecars
  and assert refusal.

## Files in scope

- `docs/labs/one-model-end-to-end/instrument/src/bin/served-reply-panel.rs`
- `docs/loop/tasks/one-model-end-to-end/deepseek-43/**`
- `docs/research/notebook/float-training.md`

## Done when

1. A run-level test passes the sidecar through the real command/run/report
   path and asserts the serialized identity fields.
2. The same run path refuses wrong-artifact and outside-vocabulary sidecars.
3. Removing either the run-level attachment or report-recording call makes the
   test fail.
4. Focused tests, sealed CPU/integer parity, formatting, report/policy checks
   and gates pass.
5. A fresh exact-head review approves the repair; the same PR merges and is
   verified on `main`.

## Stop and ask

Do not change the integer library, Claude's run, any bar or scope, or the
already-approved generation behavior. Do not touch a pod or sealed result.
