# REPORT: cpu-training-pilot, deepseek-252

Lab: deepseek. Order: [ORDER.md](ORDER.md). Parent: [deepseek-251](../deepseek-251/REPORT.md).

## What did not work, first

The first fresh review found three delivery issues:

- The REPORT did not contain the exact future checkpoint command required by
  the ORDER.
- Tests covered the model hash but not the Hamming hash, and did not cover
  malformed `hold_rows`, missing/nonfinite `d2.weight` or multiple hold tensors.
- The retained synthetic report was produced by the test harness while the
  REPORT listed only the release executable hash.

## What changed

- Added the exact one-checkpoint command to REPORT.
- Added focused refusals for Hamming hash mismatch, wrong `hold_rows` dtype and
  shape, missing/nonfinite `d2.weight`, and multiple hold tensors.
- Regenerated the synthetic report and labeled it explicitly as test-harness
  output. REPORT now records both the embedded test executable hash and the
  separate release executable hash.
- The second fresh review confirmed those repairs and found three receipt
  issues outside the implementation. [deepseek-253](../deepseek-253/REPORT.md)
  refreshes the target bytes, binds the release path and retains the store
  `NOT_RUN` receipt.

## Measurements

The corrected retained report SHA-256 is
`a5f9dacaf19cbe63eafcdf9b64df20638263a38d68d6b3073cacd043ec22211f`.
Its embedded test executable hash is
`1d039786750aa9ae9a8f788d5937e227d3d79cae50bce57a6f74ce8154cbb779`.
The release executable remains
`224dd54bf8aea4a6e993787a3d1bea4413590a322516560dc91a0120b2447a8b`.

## Checks run at the head

- Targeted library tests: 6 passed.
- Regression binary tests: 3 passed.
- Additional malformed-hold and both-hash controls: passed.
- `loop report --check` and `loop pr-check --local`: passed.
- `loop gates --full`: passed; all 11 gates were green.

## Skills used

No system-provided Codex skill applied.

## Still open

A second fresh exact-head review is still owed.

## Plain summary

This repair makes the future checkpoint command explicit, closes every named
malformed-hold test gap and states exactly which executable produced the
synthetic evidence.
