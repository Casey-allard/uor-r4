# ORDER: pre-register the bounded public CPU rank-margin pilot, deepseek-1

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `02767ac73`.

## Goal

Pre-register a bounded, two-artifact CPU rank-margin pilot on the public trained FBS dual-c131 pair and the public response-masked corpus. The sheet must let a reviewer decide pass, fail and stop before any nonzero-step run.

## Scope

- `docs/harness/prep/cpu-training-pilot*`
- `docs/loop/tasks/cpu-training-pilot/deepseek-1/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`

No trainer, serving, corpus or GPU edit. No corpus-scale training runs in this unit.

## Required design

- Base artifacts: public `dual-c131-seed-1-raw_both.uorlut` (`a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd`) and seed 2 (`da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1`).
- Corpus: public `caseyallard/uor-r4-data` dev token bytes (`0919e90f…`) and mask (`a838bb1a…`), 42,856 tokens and 14,287 response targets, artifact context 192 and manifest horizon 256.
- Split by complete BOS documents: odd `evaluation_stride=2` documents reserved for document-disjoint internal evaluation, no overlap with the subsequent training documents. Cap evaluation at 1,024 deterministically. The public manifest says `split=train`, and base-model exposure to these documents is unverified.
- Arms: zero-update baseline and exactly 256 rank-margin updates per artifact seed, using the same training order and schedule for both seeds.
- Checkpoint every 64 steps and at final, with a reloaded checkpoint and final artifact identity.

## Outcomes

- Report exact CPU/serving query, token and kept-cell gaps; target admission; top-1/top-8; margin sum; target rank; accepted/refused updates; target visits/s; context-tokens/s; mean step time; wall time; peak RSS; checkpoint/artifact hashes.
- A seed passes only if its final artifact preserves baseline admission and exact export, does not regress the non-primary ranking metric, and improves either top-1 or top-8 by at least three binomial standard deviations over its own baseline.
- The bounded development positive requires that bar on both artifact seeds. A one-seed-only result or a no-change result is retained as a configuration negative, not smoothed into a pass.
- Stop before the 256-step arm if a 16-step probe projects more than one CPU hour for either seed. Never shorten a healthy arm after it starts.

## Claim boundary

This sheet tests the effect of the subsequent 256 CPU updates on public document-disjoint internal-development fit only. It is not fresh-phrasing evidence and does not establish that the published base models were unexposed to the public documents. Sealed `m2-fresh256` complete-reply inputs/labels remain unavailable without credentials, so complete-reply and genuine fresh-phrasing outcomes are explicitly `NOT_RUN`. Public-corpus restart is also `NOT_RUN` because the driver has no resume CLI; this sheet does not close the full #3506 contract.

## Done when

- A one-page sheet names the question, arms, base artifact, data split, steps, pass/fail rules, minimum test power, stop rule, cost and claim boundary.
- A fresh non-author reviewer records `TEST FITNESS: FIT` before any nonzero-step execution.
- The pilot roadmap records that the pre-registration exists but execution remains pending.
- No corpus training or language result is claimed in this unit.
