# #507 — feat(#496): real VSA semantic encoder — functional, but facet-bottlenecked

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-08T19:53:04Z
- **Updated:** 2026-10-09T13:08:51Z
- **Closed:** 2026-08-08T20:11:49Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/507
- **Merged:** 2026-08-08T20:11:49Z
- **Merge commit:** ed7fcca3745c3734bb620b2afff215378bad9c49
- **Base / Head:** main / UOR-Foundation:feat/496-real-vsa-encoder
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What

The owner decision on #496 was **build the real encoder**. Done — plus the measurement that tells you what it bought.

- **Semantic grounding.** `VsaGeometry::ground` now builds the hypervector from the **zeta word-sum content signal** (the same multiplication-free construction `SpectralGeometry` uses — no transformer, no GPU), replacing `expand_atom`'s content **hash**. Related content now gets similar vectors.
- **Commensurable scoring.** `retrieve_vsa_multi_facet_resonance` re-grounds each candidate and cosines VSA-vs-VSA (1024↔1024), fixing the 1024-vs-512 mismatch that forced every score to `0.0` (#493).

## Result — the encoder works, but Spectral still wins

`geometry_ablation` on the D3 fixture (deranged-null gate, null dead in both arms):

| arm | top-1 | MRR | recall@20 |
|---|---:|---:|---:|
| spectral | 0.90 | **0.93** | 1.00 |
| **vsa (real encoder)** | 0.47 | **0.50** | 0.53 |

(300 stories / 30 probes; full-caps spectral is 0.83 MRR.) VSA went from **dead 0.0000 on every arm** to functional, and clears the null — the encoder is real. But **Spectral wins by +0.43 MRR** (past the 0.05 gate). The cause is visible in recall: VSA's **facet candidate-set drops ~47% of targets** (recall 0.53 vs Spectral's 1.00). The grounding is fixed; the **facet index is now the bottleneck**, not the encoder. Separately, query-time re-grounding is O(candidates) — slow at full scale, which is exactly why #496 calls for storing the VSA vector at index time (the efficiency follow-up).

## Honest disposition

Even with a real semantic encoder, VSA underperforms Spectral, which already serves the semantic-retrieval path well. This PR **does not adopt VSA as a default** — Spectral stays the path. It makes the VSA path *real and measured* instead of a placeholder, and it updates `vsa_scoring_honesty.rs` (which pinned the dead ranking on purpose) to pin the fixed state: non-zero relevance, and the "fox" query now returns the fox sentence **first** (the content-hash grounding ranked it last).

**#496 stays open.** The owner call is now, on evidence rather than a placeholder: build a real facet-index (substantial — that's where VSA's distinct value would come from) vs deprecate the switch.

fmt / clippy (`-D warnings`) / 36 non-ignored router tests / `wasm32` build all green.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

