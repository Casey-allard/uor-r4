# REPORT: cpu-training-pilot, deepseek-150

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `ab0a759cd`.

## Status

Complete. PR #3690 delivered the copy-key depth screen and merged at
`ab0a759cd`.

## What did not work

Nothing failed in this bookkeeping unit. The two earlier copy-key input-path
faults are part of deepseek-149 and remain sealed there.

## What changed

Released the completed `deepseek/copy-key-depth` claim and recorded the final
review, merge and fresh-main verification in deepseek-149. No model source or
serving result changed.

## Measurements

PR #3690's exact head `4daadc58b` passed independent review and all required
checks. Fresh main reproduced **267** library tests with 4 expected ignores,
the copy-key binary test, all published evidence manifests and the reported
successful result fields.

## Checks run at the head

- only the completed claim is removed;
- claims JSON parses;
- deepseek-149's delivery state matches PR #3690;
- exact-head review and protected merge will be verified before closing.

## Skills used

No system-provided Codex skill applies.

## Still open

Exact-head review and merge of this coordination-only unit.

## Plain summary

This closes the bookkeeping for the successful copy-key depth screen. It
contains no new model result.
