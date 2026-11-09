# ORDER: cpu-training-pilot, deepseek-252 (carry-hold review repair)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Base: `24fad747e`. Parent ORDER: [deepseek-251](../deepseek-251/ORDER.md).

## Goal

Repair the three findings from the fresh exact-head review:

1. Put an exact, copyable command for auditing future Claude-334 checkpoints in
   the REPORT.
2. Test both checkpoint hash refusals plus malformed `hold_rows` dtype/shape,
   missing/nonfinite `d2.weight` and multiple hold tensors.
3. Label the retained synthetic evidence as test-harness output with the test
   executable hash, rather than implying it came from the release CLI.

## Files in scope

- `crates/uor-r4-cpu-train/src/ask_offset_audit.rs`
- `crates/uor-r4-cpu-train/src/bin/cpu-carry-hold-audit.rs`
- `docs/loop/tasks/cpu-training-pilot/deepseek-251/REPORT.md`
- `docs/loop/tasks/cpu-training-pilot/deepseek-252/**`
- `docs/labs/cpu-training-pilot-carry-hold-audit-2026-11-09/resource-ledger.json`

## Done when

1. The exact next command is in REPORT.
2. Every named malformed case has a focused failing test.
3. Evidence provenance distinguishes the test-harness report from the release
   executable.
4. Targeted tests, policy, gates and a second fresh review pass.

## Stop and ask if

Do not modify Claude-owned files, its active run, serving, export or GPU state.
