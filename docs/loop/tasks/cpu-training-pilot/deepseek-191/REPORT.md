# REPORT: cpu-training-pilot, deepseek-191

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Prep:
[reply-boundary sheet](../../../harness/prep/cpu-training-pilot-copy-boundary-2026-11-05.md).
Base: `86d654170`.

## Status

Complete as **INSUFFICIENT_POWER** under the pre-registered bar. The fresh
oracle completed 7 replies, below the required 10, so exact-reply capability is
not assessed. The true boundary did change stopping behavior: it fired 14 times
against 3 for the matched shuffle, but produced zero exact replies and 5
premature stops on each artifact seed.

This is exact-memory recall research. It is not fresh-wording generalization,
serving evidence or a useful-chat claim.

## What did not work

- The first FIT reviews found and rejected a stale head assertion, missing
  manifest hash evidence, prose-only resource stops, a mismatched shuffled
  control description, missing behavior tests and digests that omitted result
  state. All six were repaired before compute.
- The TRAIN-only preflight showed the mechanism and its control were active:
  true boundary produced all **11** oracle-exact replies but also **41**
  premature stops, while shuffled boundary produced **1** exact and **9**
  premature stops.
- The fresh panel had **456** reachable replies, but only **7** oracle-exact
  bodies on each artifact seed. The registered assessment needs at least 10,
  so it stops as **INSUFFICIENT_POWER**.
- On both artifact seeds, true and shuffled boundaries stopped **14** and
  **3** replies, with **5** and **2** premature stops and **0** exact replies.
  The stopping rule therefore acted, but no complete reference body was matched.
- Exact-copy body quality remained the limiting factor: every arm produced
  **346** correct body positions. Stopping cannot repair the other body
  mismatches.
- This does not establish that exact reply-boundary memory is inferior to a
  matched classifier; the assigned fresh body set was too weak to decide that.

## What changed

- Added `ReplyBoundaryMemory` and deterministic TRAIN-only 4..32-token suffix
  derivation.
- Added terminal-only stop decisions, with the longest known generated suffix
  deciding and any continuation observation blocking BOS.
- Added a depth-preserving shuffled control that moves terminal counts while
  leaving each suffix's nonterminal count fixed.
- Added `cpu-copy-boundary-reply` with no-boundary, true-boundary,
  shuffled-boundary and reference-length-oracle arms.
- The no-boundary arm must equal the existing `Longest32To1` evaluator row for
  row.
- Added finite tests for exact-limit reachability, terminal BOS firing,
  no-boundary/oracle behavior, shuffled count preservation, stop-change
  accounting, paired thresholds and result-bound digests.

No serving, exporter, CUDA, float-training or Claude-owned source changed.
This boundary table is CPU-offline research and is not in the `UORLUT01`
serving contract.

## Measurements

Fresh panel:
`local/deepseek/copy-boundary/2026-11-05/panels/dd-2723`, seed 2723,
`dev,dev`, protocol 2, 128 conversations.

- tokens:
  `df88a1826bcb98929e46f56f94d2ae59098a7c9d29dde36c2e4aafa2657d3fc9`;
- response mask:
  `e06cf8594d6a8ac3d96dfd052e5f015c1c2e9d20a5bc6982ac9b952eb4a4b0f7`;
- manifest:
  `ebc75d980dc6503afe9e614e12687aca236989d5e819ac977e5b469ef0d98cb9`.

Boundary table census:

- 12,356 suffixes and 3,287 terminal occurrences;
- 2,011 terminal-only suffixes and one ambiguous suffix;
- 4,060 key labels changed by the shuffled control.

| Seed | Arm | Exact | BOS | Premature | Body | Boundary stops | tok/s |
|---:|---|---:|---:|---:|---:|---:|---:|
| 1 | no boundary | 0 | 0 | 0 | 346 | 0 | 69,274 |
| 1 | shuffled | 0 | 3 | 2 | 346 | 3 | 66,529 |
| 1 | true | 0 | 14 | 5 | 346 | 14 | 67,960 |
| 1 | oracle | 7 | 456 | 0 | 346 | 456 | 21,136 |
| 2 | no boundary | 0 | 0 | 0 | 346 | 0 | 68,416 |
| 2 | shuffled | 0 | 3 | 2 | 346 | 3 | 67,934 |
| 2 | true | 0 | 14 | 5 | 346 | 14 | 66,859 |
| 2 | oracle | 7 | 456 | 0 | 346 | 456 | 21,083 |

The true and shuffled generated stopping decisions differed on 11
row/step/depth keys per seed. The no-boundary arm matched the existing
`Longest32To1` rows exactly on both seeds.

The TRAIN-only preflight covered 453 reachable replies:

| Arm | Exact | BOS | Premature | Body | Boundary stops |
|---|---:|---:|---:|---:|---:|
| no boundary | 0 | 0 | 0 | 409 | 0 |
| shuffled | 1 | 17 | 9 | 409 | 17 |
| true | 11 | 78 | 41 | 409 | 78 |
| oracle | 11 | 453 | 0 | 409 | 453 |

The preflight therefore established that terminal-only suffixes can emit BOS
and that the shuffled control is behaviorally distinct, without opening the
fresh panel.

Resources:

- whole registered sequence: about **14.5 s**;
- maximum process RSS: **330,724 KiB**;
- corrected fresh-process normalized SHA-256:
  seed 1 `7991e7684bb448cc49a66aef03f3a985c32fd28c20f39ad9d65df2fc44aa3e5a`,
  seed 2 `9a811992d0bba60517ab1a685a4755fa4f4edfa7f7578ea4bf11159b9300472a`;
- local sealed `SHA256SUMS`:
  `7f447e78ad969edd23e4d1f807e0f2741e9eb5a04059b0d59e571ed75b7af4b9`.

Compact evidence:
[train preflight](evidence/train-preflight-summary.json),
[seed 1](evidence/seed1-summary.json),
[seed 2](evidence/seed2-summary.json) and
[repeat identity](evidence/repeat-identity.json), bound by
[PUBLISHED-SHA256SUMS](evidence/PUBLISHED-SHA256SUMS).

## Checks run at the head

- Fresh round-3 reviewer: **TEST FITNESS: FIT** and **D67 PRE-RUN: GO** at
  `1b747597a`;
- focused reply-boundary tests: **8 passed**;
- full CPU release suite: **359 passed**, 4 expected ignores, all binary tests
  passed;
- strict scoped Clippy, formatting, diff, shell syntax and claim wording
  passed;
- `prep-check`, `loop report --check` and `loop pr-check --local` passed;
- source, evaluator and generator pins matched;
- the complete sequence used one 30-minute deadline and a per-process 2-GiB
  RSS stop;
- fresh-panel hashes, input manifests, no-boundary parity and reachable-power
  checks passed;
- both artifact repeats matched after timing removal;
- the local output is sealed and its checksum manifest read back.

## Skills used

No system-provided Codex skill applied. Repository pre-registration, FIT/GO,
D84 CPU research and loop-delivery procedures were used.

## Still open

- A fresh panel with at least 10 oracle-complete longest-copy replies is
  required to assess this boundary mechanism.
- The present result says stopping was exercised but did not complete an exact
  reply because body reconstruction remains weak.
- Any later positive stop mechanism needs artifact-bound serving integration
  and exact integer-session parity.
- Exact-head delivery review, protected merge, fresh-main verification and
  claim release remain.

## Plain summary

Exact reply-boundary memory can fire and is distinguishable from its shuffled
control, but the fresh copy bodies offered only seven possible exact replies.
The run is therefore underpowered and cannot decide whether the boundary
mechanism improves completion. The dominant limitation is still body quality,
not just the absence of a stop.
