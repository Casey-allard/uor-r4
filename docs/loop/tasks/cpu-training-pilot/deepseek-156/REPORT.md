# REPORT: cpu-training-pilot, deepseek-156

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `d5246431b`.

## Status

Complete. PR #3699 delivered the deepseek-155 termination-oracle ceiling and
merged at `d5246431b7c6311e9a3680bb0b9b33acb75dd280`.

## What did not work

Nothing failed in this bookkeeping unit. The deepseek-155 negative boundaries
remain recorded: the oracle is not deployable and 442/453 train and 436/449
dev longest-first bodies were still inexact.

## What changed

Released the completed `deepseek/copy-key-termination-oracle` claim and
recorded the exact-head review, merge and fresh-main verification in
deepseek-155. No model source or serving result changed.

## Measurements

PR #3699's exact head `823d1cb7a` passed independent review and all required
checks. Fresh main passed 286 CPU library tests with 4 expected ignores, 74
binary tests, the oracle evidence seal and the recorded 11/13 result fields.

## Checks run at the head

- only the completed claim is removed;
- claims JSON parses;
- deepseek-155's delivery state matches PR #3699;
- exact-head review and protected merge were verified before closing;
- published evidence and the fresh-main CPU suite pass.

## Skills used

No system-provided Codex skill applies.

## Still open

Nothing in this coordination-only unit. The next technical action remains a
trainable copy-plus-termination mechanism.

## Plain summary

This closes the bookkeeping for the oracle-stop ceiling. It contains no new
model result.
