# REPORT: cpu-training-pilot, deepseek-178

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `4ea097661`.

## Status

Complete protected release for merged PR #3725. The scientific result remains
**INSUFFICIENT_TRAINING_DATA**.

## What did not work

Nothing in this bookkeeping unit. Artifact seed 2 stopped at 1,847 supervised
groups against the registered 2,048 floor before fit, reply evaluation or a
fresh-process repeat.

## What changed

- Recorded reviewed head
  `ad2fdfa0ee51ed7f5a5fdf1d54b73cfd69895cc0`, PR #3725 and protected merge
  `4ea0976615b4b59e4394c30476931f15291b0951`.
- Recorded CI and fresh-main verification in the deepseek-177 report.
- Removed the completed `deepseek/geometric-candidate-union` claim and the
  temporary release claim.
- Changed no model, serving, CUDA, float-inference, termination or Claude-owned
  source.

## Measurements

- reviewed delivery head:
  `ad2fdfa0ee51ed7f5a5fdf1d54b73cfd69895cc0`;
- protected merge:
  `4ea0976615b4b59e4394c30476931f15291b0951`;
- fresh-main CPU library: **326 passed**, 4 expected ignores;
- fresh-main driver binary test: **1 passed**;
- fresh-main strict Clippy: passed;
- fresh-main source pins, published evidence and no-corpus checks: passed;
- required CI checks at the approved head: passed;
- protected queue: `WOULD-QUEUE`; merge: `QUEUED`.

## Checks run at the head

- Exact-head independent review: **APPROVE**.
- `loop pr-check`; its only pre-CI refusal was the expected missing approval
  and required-check records, which were then satisfied.
- All required PR checks at the approved head.
- Fresh-main CPU library, binary test, strict Clippy and evidence seals.
- Claims JSON parsing and absence of both completed claims.
- Report, scope, evidence and `git diff --check`.
- `loop gates --full`, docs catalog and `prep-check`.

## Size

This release changes four documentation paths against base `4ea097661`.

## Skills used

No system-provided Codex skill applies. Project delivery procedures were used.

## Still open

Useful local chat remains open. The next candidate-ranker experiment needs a
construction or objective that clears the fixed floor on both artifacts under
a new seed and registration. Seed 2768 must not be reused.

## Plain summary

The geometric candidate-union experiment and its insufficiency evidence are
merged and verified on main. This record releases the completed claims.
