# Pre-registration: artifact-independent lexical reply ranker

Run ID: `cpu-training-pilot-prompt-reply-lexical-2026-11-06`

## Status

COMPLETE - **REJECT**. The lexical-only ranker was artifact-identical but fell
from raw **34 to 23** exact replies and **766 to 422** body positions on both
artifacts. The combined ranker remained artifact-stable only in its failure:
19/301 on seed 1 and 48/880 on seed 2. All 455 reachable rows were emitted with
zero premature BOS. Record:
[deepseek-215](../../loop/tasks/cpu-training-pilot/deepseek-215/REPORT.md).

This is D84 CPU research. It isolates the artifact-independent features in the
whole-reply ranker rejected by deepseek-213. It does not change serving.

## Question

Does the lexical-only form of the fixed eight-feature ranker produce a stable
exact and body gain over raw Jaccard across two artifacts?

## Fixed construction

The candidate set, eight integer features, eight epochs, unit learning rate,
weight bound and matched positive-count label shuffle are unchanged from
deepseek-213. The ranker fits two weight vectors per artifact:

1. `combined`: all features;
2. `lexical`: feature index 5, `writer_distance_advantage`, is zero in scoring
   and receives no update.

The lexical fit must produce identical weights and decisions on both artifacts.
Combined and query-geometry arms remain reported controls but do not set the
bar.

## Arms

1. `learned_lexical`;
2. `raw_unigram_jaccard`;
3. `shuffled_lexical`;
4. `learned_combined`;
5. `shuffled_combined`;
6. `query_geometry`;
7. `fixed_ordinal`;
8. `reference_membership_oracle`.

## Data split

TRAIN `tt-2711` supplies all fitting. A fresh `cell=dev,dev`, seed 2780,
protocol-2 panel is generated only after FIT and GO. No evaluation reply enters
feature construction or training. Both public dual-c131 artifacts run.

## Instrument gate

- source, artifact, tokenizer, TRAIN and panel pins match;
- the seed-2780 panel and output root are absent at registration;
- TRAIN retains at least 96 supervised and 24 baseline-wrong groups;
- both target shuffles change labels and produce distinct weights;
- lexical weights equal across artifacts and every lexical decision repeats;
- all arms emit every reachable row with zero premature BOS;
- sidecars remain byte-identical.

## Bar

For both artifacts:

- lexical exact replies exceed raw Jaccard by the paired two-sigma rule;
- lexical body positions do not fall below raw;
- lexical exact replies exceed `shuffled_lexical` by the paired two-sigma rule;
- lexical reading and decision SHA match across artifacts after timing removal;
- all 464-or-more reachable rows emit with zero premature BOS;
- three process runs repeat after timing removal.

## Outcomes

- **KEEP** if lexical exact/body and shuffle comparisons pass on both artifacts
  and lexical decisions are artifact-identical.
- **REJECT** if the lexical fit is valid and any primary exact/body comparison
  fails.
- **CONTROL_FAULT** on divergent lexical decisions across artifacts, vacuous
  shuffle, input drift, nondeterminism or emission failure.

## Stop rule

Do not retune features, top-16, epochs, learning rate or tie rules after seeing
seed 2780. Stop on any instrument or primary-bar failure.

## Cost

Local CPU only. Cap: 30 minutes, 2 GiB peak RSS and 32 MiB retained evidence.

## Base and inputs

- base: `86cd73af4`;
- TRAIN tokens:
  `140cf50d638d6066aec0a31345183712b6fc5d95502384223b91af9504b48f91`;
- TRAIN mask:
  `d0aaa2b1007e62098ab943187f25e13bddbaed3508dce881a176c2827052c0ad`;
- TRAIN manifest:
  `d570caedbb628167c59b96b14746863b99998c2f67289d1deca16dbebd402ba5`;
- tokenizer:
  `a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377`;
- artifact seed 1:
  `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd`;
- artifact seed 2:
  `da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1`;
- sidecar seed 1:
  `2a298b44ea34d1a8e2d40191951d3da9b3300e2ac7de65e6c2ec13950a0b6424`;
- sidecar seed 2:
  `6504dcb047ce7157dd5e968c29441505cf48d993f01ec63222125634c61963b7`.
- implementation: `1f5b4fbcb`;
- ranker source:
  `8588af2f0e71718e96ed73170d22e42f49a226289071a5e86f13504d93c97454`;
- binary source:
  `d38faa656fef2dcad681394fd705d1eb170464d9800b3daaaa9b0a38d477d0dc`;
- CPU crate export:
  `b900a16aec7e08f3ec9b34c3490ecabfcfe224b77f807cb7e7c4340168ff78b8`;
- release binary:
  `723903ea3bacaf52aa758ee273f146f99d09f7551df09859864f43f44d8fcc45`.

## Output

`local/deepseek/prompt-reply-lexical/2026-11-06`

Fresh panel: `.../panels/dd-2780`. Both roots are absent at registration.

## Steps

After FIT and GO, build `cpu-prompt-reply-ranker`, generate seed 2780 from the
isolated generator copy, and run the binary three times per artifact in
alternating order:

```sh
"$BIN" "$SIDECAR" "$TRAIN" "$PANEL" "$ARTIFACT" "$TOKENIZER" \
  --protocol 2 --expected-artifact "$ARTIFACT_SHA256" \
  --expected-tokenizer a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377
```

Seal panel bytes, process reports, timings, source and binary pins, and tests.

## Minimum test power

- two artifacts;
- at least 96 TRAIN groups and 24 baseline-wrong groups;
- at least 200 fresh reachable rows;
- at least 20 candidate-oracle exact rows;
- 16 candidates and three processes per artifact;
- paired lexical-vs-raw exact and body comparisons;
- paired lexical-vs-shuffled exact comparison;
- artifact-identical lexical decisions.

## TEST FITNESS

TEST FITNESS: FIT - M2 #2030 lexical reply-ranker ablation
sheet: docs/harness/prep/cpu-training-pilot-prompt-reply-lexical-2026-11-06.md
reviewer: deepseek exact-head self-review (independent delegation unavailable in this run)
note: Implementation 1f5b4fbcb is clean. Five focused ranker tests and the full
  399-test CPU library suite pass with four expected ignores. Combined and
  lexical fits share groups and shuffles; lexical scoring excludes feature 5.
  Input and source pins match; seed 2780 and its output root remain absent.
date: 2026-11-06

## Pre-run GO

GO - exact-head self-review under the current disabled-delegation policy.
Implementation `1f5b4fbcb`, source pins and release binary
`723903ea3bacaf52aa758ee273f146f99d09f7551df09859864f43f44d8fcc45`
match. This was not an independent review. Seed 2780 may now be generated under
the fixed construction and stop rule.
