# REPORT: cpu-training-pilot, deepseek-260

Lab: deepseek. Order: [ORDER.md](ORDER.md). Parent:
[deepseek-259](../deepseek-259/REPORT.md).

## What did not work, first

- The seventh fresh exact-head review found that deepseek-259 edited the
  deepseek-255 and deepseek-256 reports outside its declared file scope.
- The same review found that exact parity with Claude-334's `recall.jsonl`
  existed only as prose. The committed audit report contained observed values,
  but no expected values, source hashes or comparison result.
- The retained-report provenance sentence incorrectly described the real
  release-CLI report as test-harness output.

## What changed

- Re-scoped the deepseek-255 and deepseek-256 record corrections under this
  order and recorded the seventh-review finding in all three reports.
- Added [real-eight-gain-parity.json](../deepseek-254/evidence/real-eight-gain-parity.json).
  It retains each `recall.jsonl` path and SHA-256, the expected and observed
  question/header/value/rest gains, both per-depth type-weight means, and an
  exact equality result for every arm.
- Made the original 06:07Z 404 explicitly an observation from that time and
  identified only the synthetic retained reports as test-harness output.

## Measurements

- Eight of eight arm comparisons are equal.
- Each arm compares 12 scalar values: nine group statistics, one rest mean and
  two per-depth type-weight means, for 96 exact scalar comparisons.
- Parity receipt SHA-256:
  `be065856b20a7cc56a0a71b1dacd02d6a188927f00bb303521fdc3b4a1cce6d1`.
- Exact deepseek-254 through deepseek-260 task files plus the resource ledger:
  `393077` bytes.
- Final `pr-check` size: tests 0 files `+0/-0`; code 2 files `+986/-109`;
  docs 16 files `+1027/-0`; generated 0 files `+0/-0`; other 7 files
  `+10475/-0`.
- No new model execution or GPU work ran. Temporary checkpoint data remained
  available only for this read-only comparison and is removed after delivery.

## Checks run at the head

- Parity structure and all equality flags: passed.
- Report policy, local policy and all 11 gates: passed at the final receipt
  head.
- An eighth fresh exact-head review is requested after the final receipt.

## Skills used

No system-provided Codex skill applied.

## Still open

Claude-334 owns the B2 interpretation. The retained parity record proves only
that the CPU audit reproduces its logged diagnostic fields; it adds no carry,
B2, capability, export or serving claim.

## Plain summary

The real-checkpoint comparison is now a committed expected-versus-observed
record, and the earlier report edits have an explicit repair order without
changing the scientific result.
