# REPORT: cpu-training-pilot, deepseek-217

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Prep:
[retrieval-prefix sheet](../../../../harness/prep/cpu-training-pilot-prompt-prefix-2026-11-06.md).
Base: `423dcf4d7`.

## Status

Complete as **REJECT**, with a strong content/termination diagnosis.

The TRAIN-selected four-token retrieved prefix raised body matches from
**46/61** model-only to **513/499** and beat shuffled prefixes **112/116**.
It nevertheless produced zero exact replies and zero BOS completions: every
one of the 450 rows consumed the 64-token cap on both artifacts.

The geometric model can follow a retrieved opening, but cannot terminate after
it. Exact whole-reply retrieval remains the serving default.

## What did not work

- Retrieved-prefix exact replies were **0/0**, tied with model-only and shuffled
  prefixes. The registered exact comparisons failed.
- No prefix arm emitted a complete BOS response. All four tested lengths
  generated the full `450 × 64 = 28,800` token budget on both artifacts.
- Even the perfect reference prefix failed to terminate: lengths 1/2/4 raised
  body positions to **845/1162/1929** on seed 1 and **514/1043/1878** on seed
  2, but still produced zero BOS completions.
- Full retrieved replies produced **47** exact replies and **802** body
  positions, but **174** were shorter than the fresh reference and therefore
  classified as premature BOS by this reply-level grader.
- Independent review remained unavailable under the current delegation policy.
  FIT and GO are explicitly a local exact-head self-review.

## What changed

- Added `generate_reply_with_prefix`, which feeds a forced reply prefix through
  the exact CPU FBS writer state before greedy continuation.
- Added `cpu-prompt-prefix-continuation`, which calibrates prefix length 1/2/4
  on TRAIN, retrieves the raw Jaccard nearest stored reply, and evaluates
  model-only, full retrieval, retrieved, shuffled and reference-prefix arms.

The implementation adds **537 code lines** across three files. It does not
change the serving default, sidecars, `UORLUT01`, CUDA, float training or
Claude-owned source.

## Measurements

Fresh panel:
`local/deepseek/prompt-prefix/2026-11-06/panels/dd-2781`, seed 2781, `dev,dev`,
protocol 2, 128 conversations, **450** reference-reachable replies per
artifact. TRAIN selected prefix length **4**.

- tokens:
  `6b0bdf618ba9dc1cff165002dec2934ca9da47e88c7256bc802a87f54982221e`;
- mask:
  `703b8b9c8da2b0c30527eae23dcc78ef7906c1042bce6e2a774e049d4c577be1`;
- manifest:
  `c3d795724d789bf763805151a07bd1cda5992fb0edf7c0ebff71ed353132a74b`.

| Artifact | Arm | Exact | Opening | Body | BOS complete |
|---|---|---:|---:|---:|---:|
| seed 1 | model only | 0 | 0 | 46 | 0 |
| seed 1 | retrieved prefix 4 | 0 | 122 | 513 | 0 |
| seed 1 | shuffled prefix 4 | 0 | 24 | 112 | 0 |
| seed 1 | reference prefix 4 | 0 | 450 | 1929 | 0 |
| seed 1 | full retrieval | 47 | 122 | 802 | 450 |
| seed 2 | model only | 0 | 18 | 61 | 0 |
| seed 2 | retrieved prefix 4 | 0 | 122 | 499 | 0 |
| seed 2 | shuffled prefix 4 | 0 | 24 | 116 | 0 |
| seed 2 | reference prefix 4 | 0 | 450 | 1878 | 0 |
| seed 2 | full retrieval | 47 | 122 | 802 | 450 |

Three fresh-process runs per artifact repeated after timing removal. The
retrieved-prefix body comparison passed against model-only, but both exact
comparisons failed.

Resources:

- generator build: **13.34 s**, 1,191,568 KiB peak RSS;
- panel generation: **0.58 s**, 7,632 KiB peak RSS;
- selector processes: **4.35-4.55 s**;
- maximum selector RSS: **189,568 KiB**;
- retained evidence: **136,461 bytes**;
- sealed `SHA256SUMS`:
  `a470112a4bac615da945933e1804bb92b6f3f6cd481300a5866442e582958e25`.

Compact evidence: [result.json](evidence/result.json).

## Interpretation

The prefix mechanism works as a content anchor. A forced correct prefix
increases body alignment by up to **1,878-1,929** positions, and the retrieved
prefix beats a shuffled prefix. The failure is now isolated: continuation does
not produce BOS even when the prefix is exactly correct.

The next termination work should directly train or derive a stop signal from
the forced-prefix continuation state. More reply retrieval ranking, token-set
weighting or longer exact-copy depth would not address the observed zero-BOS
failure.

## Checks run at the head

- implementation review: local exact-head self-review at `b2adf76e7`;
- full CPU library: **399 passed, 4 expected ignores**;
- claim wording and D23/D25/D32 ratchets passed;
- source, binary, tokenizer, artifacts, TRAIN, sidecar and panel pins matched;
- selected prefix was frozen from TRAIN as length 4;
- six fresh selector processes completed and repeated after timing removal;
- all arms were run on both artifacts;
- output checksum verified.

## Delivery

Pending review, protected queue merge and fresh-main verification.

## Skills used

No system-provided Codex skill applied.

## Still open

- a stop signal conditioned on forced-prefix continuation state;
- complete-reply/fresh-phrasing generation beyond stored replies;
- Claude's independent human-text and end-to-end serving track.

## Plain summary

Retrieved four-token prefixes substantially improve body alignment, including
against shuffled controls, but the model never terminates. The prefix is
useful; termination remains the blocker.
