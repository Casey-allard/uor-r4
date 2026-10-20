# #91 — D3 natural-corpus observation run (issue #72)

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-07-23T05:03:28Z
- **Updated:** 2026-10-09T14:17:18Z
- **Closed:** 2026-07-23T07:48:33Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/91
- **Merged:** 2026-07-23T07:48:33Z
- **Merge commit:** 8bfe2b4b82a879ec2d64c2359e2c6d65e010c681
- **Base / Head:** main / UOR-Foundation:issue-72-d3-natural-corpus
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What

Completes the D3 natural-corpus observation run (issue #72): the sealed Simple English Wikipedia sample (3000 articles, CC BY-SA 4.0) is now observed, manifested, and measured.

- **`Tokenizer::encode_lossy`**: the legacy llama2.c vocab has ASCII-only byte fallback and panicked on 61/3000 articles; unencodable characters are deterministically replaced with spaces (736 total) and counted in the observation report.
- **`score --stories PATH`**: Gate C honors the D3 partition (`blake3(article id)[0] % 5 == 0`, 2404/596 articles construction/held-out) from the observation stories index instead of the ordinal train cut.
- **`input_cid`** in the observation manifest: PROV reference to the corpus CID, verified byte-identical (`blake3:194db0ee…`) against `committed.bin` and the corpus manifest.
- **Corpus manifest committed** under `.uor-models/corpora/` (gitignore exception; bulk data stays local): CID, SPDX `CC-BY-SA-4.0`, article count, split rule.
- **`merged.bin`** written on completion (Gate C `--corpus-recs`; `state.bin` is `--corpus-meta`).

## Measured (posted in full to #72)

Observation: 667,942 records, 16/16 shards κ-pinned, merged κ `blake3:212bfdd4…`. Gate C on the 133,417-position held-out natural partition: **Rule 1+2 top-1 47.9% / 18.25 bits/token vs TLA3 baseline 47.9% / 35.55** — all held-out predictions resolved via ExactContext (0 Graph, 0 Novel), so the natural-partition judge currently measures exact-context memory; caveat noted in the issue for #75.

## Gates

- [x] local: fmt, clippy, nextest (289/289), bdd (13/13), doc tests
- [x] observation run complete (3000/3000 articles, resumable-sharded, content-addressed)
- [x] natural-partition Gate C 4-way table → https://github.com/UOR-Foundation/uor-r4/issues/72#issuecomment-5055665554
- [x] corpus manifest committed with CID + SPDX + split rule

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

