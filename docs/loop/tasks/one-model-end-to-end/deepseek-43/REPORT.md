# REPORT: one-model-end-to-end, deepseek-43

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `59494646c`.

**In one line:** the complete `served-reply-panel` run path is now tested with
a synthetic 256-row panel: a valid sidecar must affect the serialized report,
and wrong-artifact or outside-vocabulary sidecars must stop the run.

## What did not work, first

- **The second exact-head review returned CHANGES.** The helper tests could
  pass while `run` ignored the sidecar or omitted the report fields.
- **The first run fixture doubled the expected row count.** It created one
  case per stratum slot instead of two cases per pair, so the report observed
  128/128/128/64/64 rows. The fixture now creates 32/32/32/16/16 pairs, giving
  the required 64/64/64/32/32 rows.
- **Two run-level mutations were exercised.** Replacing the attachment block
  with `None` made the report SHA field `Null` and failed the test. Passing
  `None` to `record_content_readout` likewise produced `Null` and failed.
- **No real asking artifact was scored.** This is a host integration test and
  parity rerun, not a language or capability result.

## What worked

- **The run path is directly testable.** `run` is now a thin environment
  wrapper around `run_with_args`.
- **The full product boundary is pinned.** A synthetic integer artifact,
  byte-BPE tokenizer, five-stratum panel, manifest and sidecar pass through
  argument parsing, validation, attachment, decoding and report serialization.
  The test reads the output file and asserts the exact sidecar SHA and
  `settings.content_readout = "attached"`.
- **Binding failures travel through `run`.** The same test invokes the run path
  with a sidecar naming a different artifact and with a target outside model
  vocabulary. Both runs fail before creating an output report.
- **The production path is unchanged.** Integer generation still uses
  `serve_step`; ranking remains CPU-only; reply-start priming is unchanged.

## What changed

`docs/labs/one-model-end-to-end/instrument/src/bin/served-reply-panel.rs`:

- split `run` into the environment wrapper and `run_with_args`;
- added a byte-BPE test tokenizer and temporary-directory fixture;
- added the complete run-level sidecar/report test and the two run-level
  failure cases.

This ORDER and REPORT and one notebook-row update are the remaining changes.
No bar, scope, runtime semantics or integer-library code changed.

## Measurements

- `served-reply-panel` tests with `cpu-parity`: **27 passed, 0 failed**.
- Run-level mutations:
  - omit attachment in `run`: report SHA was `Null`, test failed;
  - omit report recording in `run`: report SHA was `Null`, test failed.
- Wrong-artifact sidecar through `run`: refused as `sidecar binds artifact`;
  no output file created.
- Outside-vocabulary target through `run`: refused as `outside vocabulary`;
  no output file created.
- The sealed 256-row CPU/integer parity remains equal after timing and engine
  fields are removed, stripped SHA-256 `0081f5f3…`.
- Observed decode: 1,326 tokens/s CPU port and 829 tokens/s integer engine.
- Release binary:
  `f13fbaabd493b3dbf38e713408d1fcffd62b218b635cf6dc5e14f6534e7f4dd2`.
- Run-path reports:
  `local/deepseek/served-product-parity/final-parity-run-path/` (`cpu.json`
  `2d98d829…`, `integer.json` `ce2524ec…`).

## Checks run at the head

- `served-reply-panel` tests with `cpu-parity`: 27 passed, 0 failed.
- `rustfmt --check --edition 2021` on the changed file: pass.
- `git diff --check`: pass.
- Scoped Clippy, release build, sealed parity, report/policy checks and full
  gates run after the repair commit.

## Skills used

None beyond the loop itself: the repair ORDER, run-level fixture and mutations.

## Still open

- A real asking artifact remains unserved; claude-339 is Claude's run.
- Integer teacher-forced ranking remains unavailable by design.
- `uor-r4-cpu-train`'s independent Hamming port still omits ask/carry.

## Plain summary

The previous tests proved the sidecar helper worked, but did not prove the
program called it. This repair runs the actual command logic on a small
synthetic panel and reads the JSON it writes. The report now has the exact
sidecar identity whenever one is attached, and the run stops before writing a
report when the sidecar belongs to another model or names an impossible token.
