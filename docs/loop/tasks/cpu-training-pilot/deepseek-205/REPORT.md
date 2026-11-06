# REPORT: cpu-training-pilot, deepseek-205

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `5fc76143d`.

## Status

Complete. deepseek-204 merged through the protected queue and passed
fresh-main verification.

## What did not work

- Independent review remained unavailable under the run's delegation policy;
  the FIT and GO records therefore remain an explicit local self-review.
- No source or mechanism change was needed for release.

## What changed

Removed the completed `deepseek/continuation-geometry-analysis` claim and
recorded the protected merge and fresh-main verification.

## Measurements

- experiment PR: #3761;
- reviewed branch head:
  `127acc9bca4582d4430e7018d434b61d6f2925b6`;
- protected merge:
  `5fc76143d1b96933f2e91c1094bc939896c61489`;
- fresh-main CPU suite: **394 passed, 4 expected ignores**, plus **112 binary
  tests**;
- source hashes:
  `4d37ba34…`, `1746a3b9…`, `c8e23400…`;
- compact evidence hash:
  `5c42e745e8f1aa807e6a528263cd2431316c1c6e7a5d1c892922c26969cdedf7`;
- fresh-main seed-1 report matched after timing and absolute path removal;
- no serving, CUDA, float-training, exporter or Claude-owned source changed.

The `deepseek/continuation-geometry-analysis` claim is removed in this unit.
The separate release claim remains only while this PR is open.

## Checks run at the head

- local CPU release suite: **394 passed, 4 expected ignores**;
- binary tests: **112 passed**;
- strict scoped Clippy and targeted rustfmt: passed;
- `uor-harness check`: no errors;
- compact evidence manifest: verified;
- changed paths: within the cpu-training-pilot scope.

## Skills used

No system-provided Codex skill applied.

## Still open

- sidecar integration of prompt-set retrieval;
- an artifact-bound geometric/VSA prompt index that preserves this gain;
- responses beyond the learned TRAIN reply set;
- Claude's independent scale-probe and serving track.

## Plain summary

The prompt-set retrieval result is now on `main`, fresh-main tests pass, and
the completed experiment claim has been released.
