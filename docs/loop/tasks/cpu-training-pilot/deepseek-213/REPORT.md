# REPORT: cpu-training-pilot, deepseek-213

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Prep:
[whole-reply ranker sheet](../../../../harness/prep/cpu-training-pilot-prompt-reply-ranker-2026-11-06.md).
Base: `dc2a05d72`.

## Status

Complete as **REJECT** for the fixed eight-feature integer neighbor ranker.

The ranker improved artifact seed 2 from **55 to 59** exact replies and from
**966 to 1,022** body positions, but artifact seed 1 fell from **55 to 21**
exact and **966 to 325** body. The candidate oracle contained **163** correct
replies and the matched shuffle was active, so this is a mechanism-result
negative rather than insufficient power.

The exact 4,096-bit prompt selector remains the serving default. This closes
the fixed top-16 feature/objective configuration, not all learning over
retrieved replies.

## What did not work

- On artifact seed 1, learned retrieval lost **34** exact replies, **64**
  correct openings and **641** body positions against raw Jaccard. The paired
  exact comparison failed (`19` learned-only versus `53` raw-only) and body
  quality fell.
- On artifact seed 2, learned retrieval gained **4** exact replies, **18**
  openings and **56** body positions. Its exact comparison passed (`15` versus
  `11`, threshold `7`), but the gain did not replicate.
- The shuffled-target ranker produced **14** exact replies on both artifacts,
  below the learned arm, so the label control was active.
- Query geometry produced **21/17** exact replies and **490/455** body
  positions, also below raw Jaccard. It did not rescue the ranker instability.
- The writer-distance feature made the fitted weights artifact-dependent. The
  learned weights were
  `[0,-7063,-46980,-28516,-31305,-1063,5048,-521]` on seed 1 and
  `[0,31931,-3375,-6915,3655,3745,4910,-1170]` on seed 2. That dependence is
  acceptable only if both artifacts improve; they did not.
- The first preflight rejected a direct byte comparison between current TRAIN
  samples and the older sidecar. The old sidecar retains prompt-set identities
  but not original prompt order. The repair compares unique prompt sets and
  replies while using the ordered TRAIN prompts for writer features. No panel
  had been generated at that point.
- Independent review remained unavailable under the current delegation policy.
  FIT and GO are explicitly a local exact-head self-review.

## What changed

- Added `prompt_reply_rank.rs`, a deterministic integer ranker over the raw
  Jaccard top-16 with eight bounded features, an eight-epoch pairwise
  perceptron and a matched positive-count label shuffle.
- Added `cpu-prompt-reply-ranker`, which validates the sidecar by prompt-set
  and reply identity, fits only on TRAIN, supports a TRAIN-only power preflight,
  evaluates six arms and emits paired exact/body comparisons.
- Added tests for Q16 bounds, distance arithmetic, pairwise separation,
  deterministic shuffle selection and membership fallback.

The implementation adds **1,276** code lines across three files. It changes no
serving default, artifact format, sidecar bytes, `UORLUT01`, serving kernel,
CUDA, float training or Claude-owned source.

## Measurements

Fresh panel:
`local/deepseek/prompt-reply-ranker/2026-11-06/panels/dd-2779`, seed 2779,
`dev,dev`, protocol 2, 128 conversations, **464** reference-reachable replies
per artifact.

- tokens:
  `ce95b88a26c32f38703d279efd5a6873c7a9b2399f7d8c152c2b0ef8414f5e83`;
- mask:
  `7d047492b98e6648711c08e6cdf5bbabac30d64e22706a17909098a665c6cda1`;
- manifest:
  `d6f5c7aea46df2f7e157beae215611533585d4f00f9a0b98a72abb1a3fc99824`.

TRAIN preflight:

- **193** supervised groups;
- **119** baseline-wrong groups;
- **3,088** candidate rows;
- **506** positive rows;
- **756** changed labels under the matched shuffle;
- **163** fresh rows had an exact reference in the top 16.

| Artifact | Arm | Exact | Opening | Body | Emitted | Decision SHA |
|---|---|---:|---:|---:|---:|---|
| seed 1 | raw Jaccard | 55 | 127 | 966 | 464 | `02790c00…099038` |
| seed 1 | learned | 21 | 63 | 325 | 464 | `a3fd6352…a50ce4` |
| seed 1 | shuffled | 14 | 71 | 401 | 464 | `5decb643…d8221` |
| seed 2 | raw Jaccard | 55 | 127 | 966 | 464 | `02790c00…099038` |
| seed 2 | learned | 59 | 145 | 1022 | 464 | `9c306820…63e836` |
| seed 2 | shuffled | 14 | 59 | 270 | 464 | `c144c9d9…456333` |

The reference-membership oracle reached **210** exact replies and **2,244**
body positions. Every arm emitted all reachable rows with zero premature BOS.
Three alternating fresh-process runs per artifact repeated after timing removal.

Resources:

- generator build: **9.88 s**, 1,264,608 KiB peak RSS;
- panel generation: **0.63 s**, 8,064 KiB peak RSS;
- selector processes: **5.77-5.86 s**;
- maximum selector RSS: **208,424 KiB**;
- retained evidence: **182,625 bytes**;
- sealed `SHA256SUMS`:
  `c05089c74bedd90d95b56b05cb00be24c5d20d7eb3044e48f653ec0bf0d6f7a9`.

Compact evidence: [result.json](evidence/result.json).

## Interpretation

The raw top-16 contains enough correct replies to improve: 163 fresh rows
contain their exact reference, versus 55 selected by raw Jaccard. The learned
ranker found part of that gap on one artifact but discarded most of it on the
other.

The artifact-dependent writer-distance feature is a likely instability source:
easy prompt-set features and query geometry are combined in one additive
score, and small query differences can reverse close candidates. The next
ranker should separate artifact-independent reply semantics from geometric
features, or require a per-artifact calibration that is itself artifact-bound,
rather than sharing one fixed additive recipe across qualitatively different
query surfaces.

## Checks run at the head

- implementation review: local exact-head self-review at `169af13d5`;
- focused ranker tests: **4 passed**;
- full CPU library: **398 passed, 4 expected ignores**;
- claim wording and D23/D25/D32 ratchets passed;
- source, binary, tokenizer, artifacts, TRAIN, sidecar and panel pins matched;
- all six arms emitted every reachable row with zero premature BOS;
- learned, shuffled and raw repeat decisions were identical after timing
  removal;
- the candidate oracle and matched shuffle were active;
- output checksum and retry-root binding verified.

## Delivery

Pending review, protected queue merge and fresh-main verification.

## Skills used

No system-provided Codex skill applied.

## Still open

- artifact-stable ranking over the correct top-16 candidates;
- fresh response generation beyond selecting stored replies;
- artifact/export-bundle propagation of the prompt-reply sidecar;
- Claude's independent human-text and end-to-end serving track.

## Plain summary

The ranker found useful neighbors and improved one artifact seed, but it lost
most of the raw selector's quality on the other. The fixed integer ranking
recipe is rejected, and exact prompt-set Jaccard remains the default while the
next attempt separates artifact-independent reply semantics from artifact-bound
writer geometry.
