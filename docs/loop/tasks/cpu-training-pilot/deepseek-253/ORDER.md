# ORDER: cpu-training-pilot, deepseek-253 (carry-hold final receipt)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Base: `835f9b04a`. Parent ORDER: [deepseek-252](../deepseek-252/ORDER.md).

## Goal

Repair the three receipt findings from the second fresh review:

1. Refresh the build-target byte measurement at the final head.
2. Bind the exact checkpoint command to the recorded target directory and
   release executable path.
3. Retain the store inventory result for
   `results/claude/one-model-end-to-end-carry-hold-2026-11-09/runs/`; the
   current listing returns HTTP 404 and exit 1, consistent with `NOT_RUN`.

No executable, parser, test, manifest, evidence result, Claude file, serving
path or GPU state changes.

## Files in scope

- `docs/labs/cpu-training-pilot-carry-hold-audit-2026-11-09/resource-ledger.json`
- `docs/loop/tasks/cpu-training-pilot/deepseek-251/**`
- `docs/loop/tasks/cpu-training-pilot/deepseek-252/REPORT.md`
- `docs/loop/tasks/cpu-training-pilot/deepseek-253/**`

## Done when

1. Build-target bytes and timestamp match the final tree.
2. README and REPORT print the actual recorded release-binary path.
3. Store `NOT_RUN` has a retained 404 receipt.
4. Policy, gates and a final fresh review pass.

## Stop and ask if

Do not touch Claude's active run, GPU pods, serving, export or another lab.
