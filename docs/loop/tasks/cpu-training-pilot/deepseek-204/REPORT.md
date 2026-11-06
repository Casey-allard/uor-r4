# REPORT: cpu-training-pilot, deepseek-204

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Prep:
[prompt-set retrieval sheet](../../../harness/prep/cpu-training-pilot-prompt-set-retrieval-2026-11-05.md).
Base: `77f189f58`.

## Status

Complete as **KEEP** under the registered bar. On fresh seed 2777, exact
unigram-set Jaccard retrieval returned **39/471** complete exact replies on
both public artifacts with **zero premature BOS** and **948** correct body
positions. All required paired exact, body, complete-emission and control
conditions passed.

This is deterministic exact-memory retrieval, not general language
understanding, useful chat or GPU qualification. The treatment is
artifact-independent, while the Q12 writer-query arm remains a matched
geometric control.

## What did not work

- Bigram Jaccard, shuffled complete replies, fixed ordinal and Q12
  writer-query geometry were all weaker than unigram-set retrieval.
- The Q12 geometry tie reduced exact replies to **34/31** and body positions to
  **913/836**, so writer geometry did not improve the token-set ranking.
- The design was first motivated by sealed trace analysis; no single trace-only
  endpoint feature was robust enough to justify another stop-table experiment.
- Independent review was unavailable under the current delegation policy. The
  FIT and GO records are explicitly a local exact-head self-review, not an
  independent review.

## What changed

Added `prompt_retrieval.rs`, which:

1. stores each normalized TRAIN prompt's token set, adjacent-token-pair set,
   final writer query and complete normalized reply;
2. ranks prompts by exact unigram-set Jaccard using integer
   cross-multiplication;
3. resolves equal scores by lower TRAIN ordinal;
4. emits one complete reply and BOS rather than attempting tokenwise stopping.

Added `cpu-prompt-retrieval`, with treatment, bigram Jaccard, shuffled complete
replies, Q12 query-geometry nearest, unigram geometry tie, fixed ordinal and
reference-membership oracle arms.

No serving, CUDA, float-training, exporter or Claude-owned source changed.

## Measurements

Fresh panel:
`local/deepseek/prompt-retrieval/2026-11-05/panels/dd-2777`, seed 2777,
`dev,dev`, protocol 2, 128 conversations, 471 reference-reachable replies.

- tokens SHA-256:
  `13146ad6c1869524d4121b0c016f9548e8c28289390b07e106ec8563a7b1b2b6`;
- response-mask SHA-256:
  `ba97d44e56d690b4d0f39ded6469afc5b6a1ee5f25cc1f0822e9762812995bec`;
- manifest SHA-256:
  `e19cae91843c8d6db15a5862aac911bcd7bf538b2fa62b927d6b9be1ead147e0`.

The TRAIN memory contained 453 entries, 348 unique unigram prompt sets, 355
unique bigram sets and 275 unique replies. None of the 471 panel prompts had an
exact TRAIN prompt match. The deterministic shuffle changed 450 complete-reply
associations.

| Seed | Arm | Exact | Opening | Body | Premature |
|---:|---|---:|---:|---:|---:|
| 1 | unigram set, ordinal tie | **39** | **131** | **948** | **0** |
| 1 | unigram set, geometry tie | 34 | 130 | 913 | 0 |
| 1 | bigram set, geometry tie | 29 | 84 | 617 | 0 |
| 1 | shuffled replies | 1 | 27 | 145 | 0 |
| 1 | Q12 query nearest | 12 | 83 | 383 | 0 |
| 1 | fixed ordinal | 0 | 4 | 52 | 0 |
| 1 | membership oracle | 191 | 191 | 1,685 | 0 |
| 2 | unigram set, ordinal tie | **39** | **131** | **948** | **0** |
| 2 | unigram set, geometry tie | 31 | 130 | 836 | 0 |
| 2 | bigram set, geometry tie | 27 | 88 | 597 | 0 |
| 2 | shuffled replies | 6 | 32 | 171 | 0 |
| 2 | Q12 query nearest | 12 | 70 | 423 | 0 |
| 2 | fixed ordinal | 0 | 4 | 52 | 0 |
| 2 | membership oracle | 191 | 191 | 1,685 | 0 |

Treatment beat bigram, shuffled and fixed-query arms on exact replies by the
paired two-sigma rule. It also beat Q12 query geometry and fixed ordinal on
body positions. Exact treatment reply and decision digests were identical on
both artifacts:

- reply SHA-256:
  `0a013e6f1a3f0cbe59179a49658fa1e04f6dbd2c153cddaa521eb40244538175`;
- decision SHA-256:
  `c446917530af3285a9cfbda05f3fd87faa46bf91970b0bcf299dff3610b96983`.

Resources:

- release build: **23.93 s**, 1,302,476 KiB peak RSS;
- generator build: **10.78 s**, 1,263,344 KiB peak RSS;
- panel generation: **0.53 s**, 7,932 KiB peak RSS;
- seed 1 / seed 2: **4.87 / 4.88 s**, 225,088 / 224,260 KiB peak RSS;
- seed 1 / seed 2 repeat: **4.90 / 4.89 s**, 223,252 / 224,240 KiB peak RSS;
- normalized repeats matched exactly;
- retained evidence: **756,544 bytes**;
- sealed `SHA256SUMS`:
  `9ab3efc1cba30c3f5b64042f9a29c33253e860fbbc7514b034e403bd45c75b22`.

Compact evidence: [result.json](evidence/result.json).

## Interpretation

This is the first CPU completion experiment in this sequence to clear its
registered exact and stopping bars. The gain came from changing the object
being selected: one complete discrete response with an observed endpoint,
rather than a token trajectory plus a learned stop rule.

The result also bounds the claim. All 39 exact replies were present somewhere
in TRAIN, and the membership oracle was 191, so this measures prompt-set
retrieval over learned responses. No panel prompt was exact, but the panel
still shares the milestone-world vocabulary and response set.

Q12 writer-query geometry did not improve the treatment. Unigram set overlap
remains the stronger prompt signal on this panel; the geometry arm and
geometry-tie arm are controls, not causes of the pass.

## Checks run at the head

- TEST FITNESS: FIT at implementation `135a40b6c`, with the limitation that
  independent delegation was unavailable and the exact-head review was a local
  self-review;
- pre-run GO self-review: pass;
- focused prompt-retrieval tests: **5 passed**;
- full CPU release suite: **394 library tests passed, 4 expected ignores**,
  plus **112 binary tests passed**;
- strict scoped Clippy with `--no-deps -D warnings`: passed;
- targeted rustfmt and `git diff --check`: passed;
- source, tokenizer, artifact, TRAIN panel and generator pins: matched;
- fresh panel and output root absence before release: passed;
- all 471 rows reference-reachable and emitted by every non-oracle arm;
- all 471 panel prompts lacked an exact TRAIN prompt match;
- 450 shuffled reply associations changed;
- both artifacts and both fresh-process repeats: matched;
- sealed output manifest read-back: passed.
- final D23 wording correction left the release binary byte-identical:
  `6f9eda0b9ad014407780635ce2ae24dff409d58936d82420ce0f7228e4ae5368`.

## Delivery

PR #3761 merged through the protected queue as
`5fc76143d1b96933f2e91c1094bc939896c61489`. Fresh `main` contains the reviewed
source and evidence hashes, passes **394 library tests with 4 expected
ignores** plus **112 binary tests**, and reproduces the sealed seed-1 report
after removing timing and absolute path fields.

## Skills used

No system-provided Codex skill applied. Repository D22/D67, D84 CPU research,
exact-evidence and protected-delivery procedures were used.

## Still open

- integration of prompt-set retrieval as a bounded CPU sidecar;
- replacement of literal prompt-set overlap with an artifact-bound geometric
  or VSA index if it can preserve the gain;
- generalization beyond replies already present in TRAIN;
- Claude's independent scale probe, export and serving track.

## Plain summary

Retrieving one complete learned reply by token-set overlap achieved 39 exact
complete replies on a fresh 471-row panel, with no premature stops. It clearly
outperformed bigram overlap, shuffled replies, pure Q12 query geometry and a
fixed response. This establishes a viable CPU completion mechanism while
keeping the exact-memory and artifact scope explicit.
