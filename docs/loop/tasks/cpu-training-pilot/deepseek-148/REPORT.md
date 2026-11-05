# REPORT: cpu-training-pilot, deepseek-148

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `67736f850`.

## Status

Complete. This unit only released the merged `deepseek/trajectory-density`
coordination claim.

## What did not work

Nothing in this coordination cleanup failed. An earlier packaging attempt that
contained only the claims edit was rejected by the order/report delivery gate,
so this unit adds the required ORDER and REPORT rather than bypassing it.

## What changed

`uor-harness team release --branch deepseek/trajectory-density --lab deepseek`
removed the completed branch claim from `docs/harness/claims.json`. No model
compute, result, bar, serving path or Claude-owned source changed.

## Measurements

PR #3687 is merged at `67736f850`. Fresh `main` verified the delivered evidence
manifest `f56a5dc8be880189fb71776cac9f9979c6a7a0a6daa4a1c249230bcfa6c90467`,
the documented result fields and **261** library plus **10** driver tests. This
unit itself is a three-path coordination edit: one ORDER, one REPORT and the
claims removal.

## Checks run at the head

- claims JSON parses after release;
- only the named claim is removed;
- scope and commit-order checks pass;
- exact-head review and protected merge are required before completion;
- fresh-main claim absence is verified after merge.

## Skills used

No system-provided Codex skill applies.

## Still open

The protected merge and fresh-main absence check remain until this cleanup PR
lands.

## Plain summary

This is bookkeeping only: it releases the claim after the already-verified
trajectory-density result merged. No capability or model result follows.
