# REPORT: cpu-training-pilot, deepseek-215

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Prep:
[lexical reply ranker sheet](../../../../harness/prep/cpu-training-pilot-prompt-reply-lexical-2026-11-06.md).
Base: `86cd73af4`.

## Status

Complete as **REJECT** for the lexical-only ablation.

The lexical ranker removed artifact divergence exactly as designed: both
artifacts produced the same weights and decision SHA. It nevertheless lost
against raw Jaccard on both artifacts, falling from **34 to 23** exact replies
and **766 to 422** body positions.

This closes the tested additive lexical ranking function, not all learning over
reply candidates. Exact prompt-set Jaccard remains the serving default.

## What did not work

- Lexical exact replies were **23** versus raw **34** on each artifact; the
  paired comparison had **22** lexical-only and **33** raw-only rows, below the
  required threshold of 12.
- Lexical body positions were **422** versus raw **766**.
- The lexical arm also lost to its shuffled control, **23** versus **25**
  exact, so target supervision did not add a useful signal.
- The combined ranker remained artifact-dependent: exact/body was **19/301**
  on seed 1 and **48/880** on seed 2. It beat raw only on seed 2.
- Query geometry reached **19/17** exact and **413/382** body positions, below
  raw on both artifacts.
- Independent review remained unavailable under the current delegation policy.
  FIT and GO are explicitly a local exact-head self-review.

## What changed

- Added an explicit `Combined` / `Lexical` feature-set split to the existing
  integer whole-reply ranker.
- Trained true and shuffled lexical rankers alongside the combined ranker,
  with writer feature index 5 excluded from lexical scoring and updates.
- Added lexical and shuffled-lexical evaluation arms and paired comparisons.
- Added a direct test proving that lexical scoring ignores writer distance.

The code change is **129 additions and 17 deletions** across three files. It
does not alter the raw selector, sidecars, `UORLUT01`, serving kernels, CUDA,
float training or Claude-owned source.

## Measurements

Fresh panel:
`local/deepseek/prompt-reply-lexical/2026-11-06/panels/dd-2780`, seed 2780,
`dev,dev`, protocol 2, 128 conversations, **455** reference-reachable replies
per artifact.

- tokens:
  `a1b847809eb8ce50a2fc6efe55f1dd6922bf976a16f822bca27550e051695861`;
- mask:
  `6316cc17ed5356828db6cc2f37d93da917c9a1c46931f1604cd330cdbcc056c9`;
- manifest:
  `9cc4d0ead8e35fc2e179ad40a0a96a4da6c15fec11b17c1dcbad5bdbd853d9c3`.

TRAIN preflight:

- **193** supervised groups;
- **119** baseline-wrong groups;
- **3,088** candidate rows;
- **506** positive rows;
- **756** changed shuffled labels;
- **137** fresh rows had an exact reference in the top 16.

The lexical weights were identical on both artifacts:
`[0,7545,-7716,-31163,-18757,0,5323,-986]`.

| Arm | Exact | Opening | Body | Decision SHA |
|---|---:|---:|---:|---|
| raw Jaccard | 34 | 117 | 766 | `4bb3a5a0...bded0` |
| shuffled lexical | 25 | 70 | 438 | `e1d2097c...aefe` |
| learned lexical | 23 | 79 | 422 | `ca8424ec...19ad` |
| combined seed 1 | 19 | 65 | 301 | `6b44f11f...7965e` |
| combined seed 2 | 48 | 128 | 880 | `25fd92c6...a17d` |
| reference membership oracle | 194 | 253 | 2046 | `89d0c409...6043` |

Every arm emitted all 455 reachable rows with zero premature BOS. Three
alternating fresh-process runs per artifact repeated after timing removal.

Resources:

- generator build: **9.81 s**, 1,269,268 KiB peak RSS;
- panel generation: **0.59 s**, 7,980 KiB peak RSS;
- selector processes: **6.38-6.62 s**;
- maximum selector RSS: **208,464 KiB**;
- retained evidence: **196,104 bytes**;
- sealed `SHA256SUMS`:
  `f3dea77341376bc263d880d320bf287f2d44ba3a5a6f7b8b64290856641d113e`.

Compact evidence: [result.json](evidence/result.json).

## Interpretation

Artifact independence was necessary but not sufficient. Lexical Jaccard,
coverage, reply overlap, frequency and length features consistently chose
complete replies that were worse than the raw nearest prompt. The learned
linear direction is therefore not an answer-quality model; it mostly selects
reply-frequency or length patterns that do not track correctness.

The next reply-selection attempt should learn a different target or
representation, such as reply-token sequence compatibility or a geometric
continuation score. Re-weighting the same seven prompt/reply summary features
would repeat this closed configuration.

## Checks run at the head

- implementation review: local exact-head self-review at `1f5b4fbcb`;
- focused ranker tests: **5 passed**;
- full CPU library: **399 passed, 4 expected ignores**;
- claim wording and D23/D25/D32 ratchets passed;
- source, binary, tokenizer, artifacts, TRAIN, sidecar and panel pins matched;
- all eight arms emitted every reachable row with zero premature BOS;
- lexical weights and decisions matched across artifacts;
- three runs per artifact repeated after timing removal;
- candidate and membership oracles were active;
- output checksum verified.

## Delivery

Pending review, protected queue merge and fresh-main verification.

## Skills used

No system-provided Codex skill applied.

## Still open

- a reply score that models sequence compatibility rather than summary features;
- fresh response generation beyond selecting stored replies;
- Claude's independent human-text and end-to-end serving track.

## Plain summary

Removing writer geometry made the ranker deterministic across artifacts, but it
then lost to raw prompt retrieval on both. The lexical ablation is rejected;
only the combined ranker's seed-2 gain remains an unreplicated observation.
