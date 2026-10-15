# #1888 — The sealed evaluations converted to bits per byte: the project's real league table (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T09:31:07Z
- **Updated:** 2026-10-09T13:19:15Z
- **Closed:** 2026-10-08T09:32:46Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1888
- **Merged:** 2026-10-08T09:32:46Z
- **Merge commit:** 25fbafc73c3f2e627023a572937cafa50cbc4933
- **Base / Head:** main / UOR-Foundation:deepseek/bpb-league
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What this does

Every sealed model evaluation here records nats per token and leaves **`bits_per_byte: null`**. Nobody had converted them, so the project has been choosing between mechanisms without knowing what its own models score in the unit every gate is written in. The conversion is **exact and free**.

New instrument `token-stream-bytes`: computes a stream's exact byte total from the tokenizer, and with `--eval` converts a sealed record.

It also settles a reported defect: both streams carry a **64-byte UORT header**, and skipping it leaves **zero** out-of-vocabulary ids — the ids 20309 / 21586 / 34205 that were flagged were *header bytes read as tokens*. Exact bases: chat-v0-p2/heldout **2.837427** bytes/token, tinydialogues-val **3.003995**.

## The league table (sealed 2026-10-03 evaluations, converted)

**chat-v0-p2/heldout**, 6,194,304 targets each:

| model | nll | bits/byte |
|---|---:|---:|
| **geo-8m-l2** | 2.2025344 | **1.119883** |
| geo-8m-s2-metal-b | 2.2081085 | 1.122717 |
| geo-8m-a | 2.2110192 | 1.124197 |
| tf-8m (parameter-matched control) | 2.2527989 | 1.145440 |

**tinydialogues-val**, 7,678,848 targets each:

| model | nll | bits/byte |
|---|---:|---:|
| **geo-8m-l2** | 1.8877944 | **0.906630** |
| geo-8m-s2-metal-b | 1.8890558 | 0.907236 |
| geo-8m-a | 1.8941910 | 0.909702 |
| tf-8m control | 1.9100456 | 0.917316 |

**All three geometric variants beat the parameter-matched transformer control on both corpora**, consistently ordered `l2 < s2 < a < control`, by 0.8–2.2 %. Single seeds with no paired CI, and the lab's own review of a comparable cycle called a similar edge parity because it sat inside the seed spread — so treat the *ordering* as suggestive and the *levels* as solid.

## The barrier, in the project's own numbers

```
geo-8m-l2  1.1199  <  Kneser-Ney 5-gram 1.280275  <  served discrete scorer 4.211070
                                                      uniform on that corpus = 4.2292
```

A **trained 8M model that has sat on local disk since 3 October** scores 1.1199 bits/byte on real held-out chat and beats its own transformer control. **The artifact the project serves from the same family is 0.018 BPB below uniform and 0.98 BPB worse than an order-0 frequency table** fit on the same split.

The gap is **3.09 bits/byte and it is in the serving path** — not the architecture, the data, or the mechanism. It also means 4.2111 should never have been used as the baseline for mechanism deltas: this session's retracted −0.0196 was 0.09σ of the per-document spread on that corpus.

## Next

1. Score the served artifact on the **same position set and context protocol** as the sealed stack evaluations, so the serving gap is a matched measurement rather than two numbers from different protocols.
2. Then attack it directly: the artifact is integer/table by construction, so the question is where 3.09 BPB is lost between the trained continuous model and the exported scorer.

Evidence: `docs/evidence/sealed_bpb_league_2026-10-08.txt`.

References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

