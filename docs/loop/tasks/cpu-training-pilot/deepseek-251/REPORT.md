# REPORT: cpu-training-pilot, deepseek-251

Lab: deepseek. Order: [ORDER.md](ORDER.md).

## What did not work, first

- `uor-harness loop claim cpu-training-pilot --lab-topic carry-hold-audit`
  refused on the known umbrella-scope false overlap with an unrelated open
  one-model-end-to-end order.
- Claude-334 has not sealed either real B2 checkpoint. The store currently
  lists only its `tests/t1` through `tests/t3` diagnostics, so real
  `B2-aqH`/`B2-aH` checkpoint measurements are `NOT_RUN`.
- The first build-time test fixture declared a malformed gain shape but
  supplied 63 values to a 64-element view. The fixture, not production parsing,
  was corrected to carry the requested wrong shape explicitly.
- The first final `loop gates --full` run hit the harness's known transient
  `Text file busy` race in its fake-`gh` test; the clean rerun passed.
- The first fresh review found that the REPORT lacked the exact next command,
  several malformed-hold controls were untested and the retained synthetic
  report's test-harness provenance was unclear. [deepseek-252](../deepseek-252/ORDER.md)
  repairs all three.
- The second fresh review confirmed the code/tests and found three final
  receipt issues: stale build-target bytes, an unbound release path in the
  exact command and no retained store inventory receipt. [deepseek-253](../deepseek-253/ORDER.md)
  fixes them.
- This audit verifies checkpoint structure and learned table movement. It does
  not execute the held write and does not prove that a selected value reaches
  `H4`.

## What changed

- `TrainedAskOffsets` now parses and validates an optional held write:
  `hold_rows` must accompany exactly one `d2.hold`; the depth-6 hold is refused;
  the gain must be finite `[vocab]`; and `d2.weight` must be finite
  `[2,width]`.
- `AskHoldSummary` reports hold rows, gain nonzeros, mean, minimum, maximum,
  mean absolute value, type-write nonzeros and type-write mean absolute value.
- Added `cpu-carry-hold-audit`. It binds model and Hamming hashes, requires the
  Claude-334 `[2,6]` depths and expected row count, re-audits the 16 ask
  relations, and writes a new deterministic report.
- Added tests for gain/write statistics, missing or mismatched row count,
  stream-only placement, wrong gain/weight shapes, nonfinite gain, expected-row
  refusal, wrong hashes, output replacement and deterministic repeats.
- No float-training, Claude-owned, serving, export or GPU file changed.
- Size: `pr-check` reports 14 changed paths: code 3 files `+764/-0`,
  documentation 8 files `+398/-0`, and other text evidence 3 files `+649/-0`
  (`+1,811/-0` total). The ORDER gave no line estimate.

## Measurements

The synthetic held checkpoint used one ask head, width 8, depths 2 and 6,
`hold_rows=8`, two nonzero gain entries and one nonzero type-write coordinate.
The audit measured:

- gain mean `0.00390625`, min `-0.25`, max `0.5`, mean absolute value
  `0.01171875`;
- type-write mean absolute value `0.1875`;
- 32 ask cells, 28 target-nearest cells and two regressed cells.

The retained [store inventory receipt](evidence/store-runs-inventory.txt)
records an HTTP 404 for the `runs/` root and exit 1, confirming that no B2 run
tree was published at the measurement time.

Two fresh report outputs were byte-identical. The retained synthetic report
SHA-256 is
`a5f9dacaf19cbe63eafcdf9b64df20638263a38d68d6b3073cacd043ec22211f`.
It was generated through the binary test harness, whose embedded executable
SHA-256 is
`1d039786750aa9ae9a8f788d5937e227d3d79cae50bce57a6f74ce8154cbb779`;
it is not release-CLI output. The separately built release executable SHA-256
is
`224dd54bf8aea4a6e993787a3d1bea4413590a322516560dc91a0120b2447a8b`.

Full CPU library tests passed 480 with 5 ignored in 47.06 s; process wall was
47.16 s at 2,176% CPU and 755,748 KiB peak RSS.

Resources are in the
[D84 ledger](../../../../labs/cpu-training-pilot-carry-hold-audit-2026-11-09/resource-ledger.json).
The previous admitted total `109,367.677487699 s` plus this unit's 1,800-second
projection gives `111,167.677487699 s`.

## Checks run at the head

- `cargo test --offline --locked -p uor-r4-cpu-train --lib
  ask_offset_audit::tests`: 6 passed.
- `cargo test --offline --locked -p uor-r4-cpu-train --bin
  cpu-carry-hold-audit`: 3 passed.
- `cargo test --offline --locked -p uor-r4-cpu-train --lib`: 480 passed,
  5 ignored.
- `cargo check --offline --locked -p uor-r4-cpu-train --all-targets`: passed.
- Scoped Clippy: passed with only pre-existing warnings.
- Release build: passed.
- `loop report --check`, `loop pr-check --local` and `loop gates --full`: passed;
  all 11 gates were green.
- `uor-harness check`: 365 rules, 0 errors and 331 pre-existing warnings outside
  this unit.

## Skills used

No system-provided Codex skill applied. The repository CPU-research and loop
delivery instructions were followed directly.

## Still open

- Run the binary on each sealed B2-aqH and B2-aH checkpoint after Claude-334
  publishes them.
- Exact command for each checkpoint:

  ```sh
  target/deepseek-carry-hold-audit/release/cpu-carry-hold-audit \
    --checkpoint "$CKPT" \
    --expected-model-sha256 "$MODEL_SHA256" \
    --expected-hamming-sha256 "$HAMMING_SHA256" \
    --expected-hold-rows 8 \
    --out "$NEW_REPORT"
  ```

- Interpret zero gain as hold-not-trained. Interpret moved gain with failed
  carry as a write/trunk-carry problem. Neither is a B2 verdict by itself.
- Implement no serving or export path from this result.

## Plain summary

This unit makes the CPU audit understand the held write Claude added. It checks
that the checkpoint really carries a stream-only eight-row gain, measures
whether that gain and type-write row moved, and preserves the ask-selection
readout. It is ready to inspect the real B2 checkpoints without spending GPU
time.
