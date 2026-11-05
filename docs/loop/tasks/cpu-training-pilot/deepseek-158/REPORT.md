# REPORT: cpu-training-pilot, deepseek-158

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `663129a83`.

## Status

Complete. PR #3701 delivered the deepseek-157 generated-stop rejection and
merged at `663129a8336438755800c09f11d9bc3983ef2079`.

## What did not work

Nothing failed in this bookkeeping unit. The deepseek-157 negative remains
recorded: the fixed generated-trajectory gate learned some endpoints but
produced zero exact replies and exceeded the premature-stop limit.

## What changed

Released the completed `deepseek/copy-generated-stop` claim and recorded the
exact-head review, merge and fresh-main verification in deepseek-157. No model
source or serving result changed.

## Measurements

PR #3701's exact head `72ab72717` passed independent review and all required
checks. Fresh main passed 286 CPU library tests with 4 expected ignores, 77
binary tests, the attempt-6 evidence seal and the recorded zero-exact fields.

## Checks run at the head

- only the completed claim is removed;
- claims JSON parses;
- deepseek-157's delivery state matches PR #3701;
- exact-head review and protected merge were verified before closing;
- published evidence and the fresh-main CPU suite pass.

## Skills used

No system-provided Codex skill applies.

## Still open

Nothing in this coordination-only unit. The next technical action is a
copy-boundary or exact-body-only termination objective.

## Plain summary

This closes the bookkeeping for the rejected generated stop gate. It contains
no new model result.
