# REPORT: cpu-training-pilot, deepseek-152

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `292a57213`.

## Status

Complete. PR #3693 delivered the copy-key depth-age router rejection and merged
at `292a57213`.

## What did not work

Nothing failed in this bookkeeping unit. The router's negative result remains
recorded in deepseek-151.

## What changed

Released the completed `deepseek/copy-key-router` claim and recorded the final
review, merge and fresh-main verification in deepseek-151. No model source or
serving result changed.

## Measurements

PR #3693's exact head `0cba12e96` passed independent review and all required
checks. Fresh main reproduced **275** CPU library tests with 4 expected ignores,
the router binary test, all published evidence manifests and the recorded
rejection fields.

## Checks run at the head

- only the completed claim is removed;
- claims JSON parses;
- deepseek-151's delivery state matches PR #3693;
- exact-head review and protected merge were verified before closing.

## Skills used

No system-provided Codex skill applies.

## Still open

Nothing in this coordination-only unit.

## Plain summary

This closes the bookkeeping for the rejected copy-key router. It contains no
new model result.
