# REPORT: cpu-training-pilot, deepseek-154

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `ab49404cf`.

## Status

Complete. PR #3697 delivered the deepseek-153 result and merged at
`ab49404cf74e5fd57f1d05037af5392215db0eed`.

## What did not work

Nothing failed in this bookkeeping unit. The deepseek-153 negative boundaries
remain recorded: no arm produced an exact or BOS-complete reply, and exact copy
alone cannot terminate.

## What changed

Released the completed `deepseek/copy-key-longest-reply` claim and recorded the
exact-head review, merge and fresh-main verification in deepseek-153. No model
source or serving result changed.

## Measurements

PR #3697's exact head `b042a43c8` passed independent review and all required
checks. Fresh main reproduced **286** CPU library tests with 4 expected
ignores, all binary tests, both FIT records, the published evidence seal and
the recorded result fields.

## Checks run at the head

- only the completed claim is removed;
- claims JSON parses;
- deepseek-153's delivery state matches PR #3697;
- exact-head review and protected merge were verified before closing;
- hash-bound evidence and the fresh-main contract tests pass.

## Skills used

No system-provided Codex skill applies.

## Still open

Nothing in this coordination-only unit. The next technical action remains
deepseek-153's copy-plus-termination follow-up.

## Plain summary

This closes the bookkeeping for the completed complete-reply copy screen. It
contains no new model result.
