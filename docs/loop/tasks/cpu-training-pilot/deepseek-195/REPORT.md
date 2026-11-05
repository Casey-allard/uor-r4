# REPORT: cpu-training-pilot, deepseek-195

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `719a3ce42`.

## Status

Complete. Removes the final release-branch claim after deepseek-193's result
and delivery records merged. No experiment or result changes.

## What did not work

Nothing failed in this bookkeeping unit. PR #3749 correctly removed the source
claim but its release branch itself still carried a team claim after merge.

## What changed

- Removed the completed `deepseek/copy-suffix-release` claim.
- No source, model, serving, CUDA, float-training, exporter or Claude-owned
  file changed.

## Measurements

- suffix source claims after change: 0;
- suffix release claims after change: 0;
- changed paths: two docs and one claims bookkeeping file;
- source, model and evidence files changed: none.

## Checks run at the head

- `deepseek/copy-suffix-memory` was already absent from `claims.json`;
- release claim removal succeeded;
- report and local PR checks pass before delivery.

## Skills used

No system-provided Codex skill applied. Repository claim and protected-delivery
procedures were used.

## Still open

- a continuation mechanism that preserves local exact copy rather than
  displacing it;
- complete-reply/fresh-phrasing capability on the one-model path;
- Claude's independent scale and serving track.

## Plain summary

The suffix-memory work and its release are merged. This final bookkeeping PR
removes the release branch's completed claim so no suffix-memory claim remains.
