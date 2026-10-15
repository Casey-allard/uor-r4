# #508 — feat(#496): real VSA facet-index — recall parity, now competitive with Spectral

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-08T20:48:08Z
- **Updated:** 2026-10-09T13:08:51Z
- **Closed:** 2026-08-08T21:08:04Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/508
- **Merged:** 2026-08-08T21:08:04Z
- **Merge commit:** 9d2f58df5e97a2ccd8d60f31d2ade51297be6751
- **Base / Head:** main / UOR-Foundation:feat/496-facet-index
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What

The second stage of #496 (after the encoder in #507): the **real facet-index** you asked for. The encoder made VSA functional but it lost to Spectral because the facet candidate-set dropped ~47% of targets. This fixes that.

Two changes, both in the router:

- **Store the VSA vector at index time** (`vsa_vectors`) — retrieval scores against precomputed vectors instead of re-grounding every candidate (the O(candidates) cost that made the first cut too slow to run at full scale).
- **Salient-content-dimension inverted index** (`vsa_dim_index`) — each item is indexed under the top-16 magnitude dims of its content signal; retrieval **unions** the query's salient dims for a high-recall, content-addressed candidate set, replacing the coarse `type=sum%100` / `entity=[100,200]` facets.

## Result — recall bottleneck fixed to parity

`geometry_ablation` (deranged null dead in both arms):

| caps | arm | top-1 | MRR | recall@20 |
|---|---|---:|---:|---:|
| 2000st/200pr (46,342 windows) | spectral | 0.85 | 0.89 | 0.98 |
| | **vsa** | 0.73 | **0.81** | **0.98** |
| 300st/30pr | spectral | 0.90 | 0.93 | 1.00 |
| | **vsa** | 0.87 | **0.92** | 1.00 |

The facet-index **collapsed the MRR gap from 0.43 → 0.08** and lifted **VSA recall 0.53 → 0.98** (now equal to Spectral), and it's fast (stored vectors, no re-grounding). On the small corpus they're statistically indifferent; at full scale Spectral still edges VSA by +0.08 (just over the 0.05 gate), so **Spectral stays the default** — but VSA is now a genuinely competitive, facet-structured retriever, **not a placeholder to deprecate**.

## Disposition

This resolves #496: the encoder + facet-index are built, and VSA is now viable and competitive rather than dead. It stays (not deprecated). Further closing the 0.08 gap — a slice-norm-style scaling on the VSA cosine, or salient-dim-count tuning — is optional optimization, not a blocker, and can be a follow-up if VSA-as-primary is ever wanted.

`clear_corpus`/`reset_to_defaults` clear the new maps (item ids restart at 0); the legacy facet-intersection path is retained for router state serialized before this index. fmt / clippy (`-D warnings`) / 36 router tests / `wasm32` build all green.

Closes #496.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

