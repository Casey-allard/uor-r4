# REPORT: one-model-end-to-end, deepseek-42

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `59494646c`.

**In one line:** the fresh review's two test gaps are repaired: report
identity is now asserted directly, and sidecars bound to another artifact or
whose target is outside the model vocabulary are refused.

## What did not work, first

- **The first exact-head review returned CHANGES.** The report assignments
  were correct in code, but no test would have failed if
  `inputs.content_readout_sha256` or `settings.content_readout` were removed.
  The same review found only a matching sidecar was tested.
- **Two report mutations were run to confirm the repair.** Removing the sidecar
  hash made the identity test return `Null`; removing the attachment marker
  also made it return `Null`. Both tests failed as they must and the fields
  were restored.
- **No model was trained or scored.** This repair changes test coverage and
  report assembly only. Claude's claude-339 language run remains separate.

## What worked

- **Report identity is pinned.** `record_content_readout` owns the two report
  assignments. A test asserts both exact fields when a sidecar SHA is present
  and asserts neither field when it is absent.
- **Artifact binding is pinned.** A sidecar naming a different artifact SHA is
  refused.
- **Vocabulary binding is pinned.** A sidecar naming a target at
  `model.shape().vocab` is refused outside the vocabulary.
- **The product path is unchanged.** Generation still calls `serve_step`,
  teacher-forced ranking remains on the CPU port, and reply-start priming
  remains before the prompt's final token.

## What changed

`docs/labs/one-model-end-to-end/instrument/src/bin/served-reply-panel.rs`:

- extracts the two report fields into `record_content_readout`;
- adds a report identity/absence test;
- extends the content-readout test with wrong-artifact and
  outside-vocabulary refusals.

This ORDER and REPORT and one notebook-row update are the remaining changes.
No bar, item status, scope, runtime behavior or integer-library code changed.

## Measurements

- `served-reply-panel` tests with `cpu-parity`: **26 passed, 0 failed**.
- Two new report mutations fail the identity test:
  - omit `inputs.content_readout_sha256`;
  - omit `settings.content_readout`.
- Wrong-artifact sidecar: refused as `sidecar binds artifact`.
- Outside-vocabulary target: refused as `outside vocabulary`.
- The prior 256-row CPU/integer parity remains byte-identical after timing and
  engine fields are removed, stripped SHA-256 `0081f5f3…`.
- Observed decode: 1,354 tokens/s CPU port and 824 tokens/s integer engine.
- Release binary:
  `691730071a05bfdac992c59669199859b6024b7080bcbf091d9c9211ea49d469`.
- Repair reports:
  `local/deepseek/served-product-parity/final-parity-repair/` (`cpu.json`
  `bcc4ab52…`, `integer.json` `415075d6…`).

## Checks run at the head

- `served-reply-panel` tests with `cpu-parity`: 26 passed, 0 failed.
- `rustfmt --check --edition 2021` on the changed file: pass.
- `git diff --check`: pass.
- Report, policy and full gates run after the repair commit.

## Skills used

None beyond the loop itself: the repair ORDER, focused tests and mutations.

## Still open

- A real asking artifact with a content readout has not been scored.
- Integer-session teacher-forced ranking is still unavailable by design.
- `uor-r4-cpu-train`'s independent Hamming port still does not read ask/carry.
- The first successful asking artifact still needs a `uor-chat` product smoke.

## Plain summary

The scorer already wrote the content-readout identity correctly, but the tests
did not prove it. Now deleting either identity field makes a test fail. A
sidecar tied to another model or carrying an impossible token is also tested
and rejected. The scoring path itself did not change.
