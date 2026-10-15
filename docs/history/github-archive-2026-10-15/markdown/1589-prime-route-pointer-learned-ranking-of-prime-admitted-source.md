# #1589 — Prime-route pointer: learned ranking of prime-admitted sources and ordered n-let admission

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-01T23:44:54Z
- **Updated:** 2026-10-09T13:08:47Z
- **Closed:** 2026-10-02T00:35:17Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1589
- **Merged:** 2026-10-02T00:35:17Z
- **Merge commit:** f9c926925caa232a4c32e9729d8c5ca574ee32b9
- **Base / Head:** main / UOR-Foundation:codex/prime-route-ranked
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #1512.

This follows #1587. Its exact token route was measured NEGATIVE on emit-1's recipe (#1512): MQAR 2/109 against the soft pointer's 16, and reachable 0.25 against 0.54. It was both too narrow (the identical preceding token was required) and too permissive (any shared common piece admits). This PR adds the two refinements that diagnosis points to. Both are opt-in route modes; nothing changes for existing models.

**1. Ranked route: admission by prime, ranking learned** (`pointer_route=prime-ranked:W`, `PrimeRoute::ranked`).
- Prime arithmetic decides which sources are admitted.
- An admitted source scores its learned pointer score plus `ROUTE_SHARPNESS` × its route score, softmaxed over the admitted sources.
- With nothing admitted, the attention is the learned pointer's softmax over every source. So a reply's first token, which no key matches, can still copy by content.
- The query and key train as under a selection. The backward is unchanged, because it uses only the final kept-source weights.
- At zero learned scores it equals the exact route bit for bit.
- This applies the AGENTS.md rule to keep admission separate from ranking.

**2. Ordered n-let admission with backoff** (`ngram:W` / `ngram-ranked:W`, `RouteAdmission::Ngram`). This is ADR-0003's transition indexes with divisor fallback (I2 before I1), as in `uor-r4-core/src/prime_route_attention.rs`.
- A source is admitted when the n tokens before it equal, in order, the query's last n. This is equality of the ordered registered-prime sequences, since the registry is injective.
- n is the longest value ≤ W with any match; shorter n are tried only when no longer one matches.
- The score is the n-let's prime weight (Σ ln p) plus recency.
- On an MQAR reply "…lukgostjal is", the fact "…lukgostjal is 293" matches the trigram, while the question "…lukgostjal was?" matches only shorter n-lets.

**Configuration.** `PrimeRoute` gains `ranked` and `admission`, each serialized only when not the default, so #1587 routes load unchanged.

**Checks run locally** (head `43500647`; its tree equals the tested `a23aa8f1`, because the merge only re-applies #1587's squash).
- 21 route tests:
  - ranked equals exact at zero scores (bit for bit);
  - learned scores re-rank equally matched sources and never admit an unadmitted one;
  - the fallback equals the soft pointer bit for bit;
  - n-let longest match, backoff, order sensitivity, recency, and no admission without recurrence;
  - the grammar and the saved forms;
  - the exact n-let route trains only its gate;
  - for the ranked shared-atom and ranked n-let routes, under Dot and Lorentz: the loss, the 24 scored rows and `next_scores` match the f64 reference, and every pointer variable's gradient (query, key, gate, scale) matches finite differences.
- Existing pointer tests pass, plus the CLI tests: `geometric-stack` 9 and `m-world` 5.
- GitHub evidence: branch `codex/ci/prime-route-ranked` (workflow v3) at this head.

**Measurements, running on the internal slot (#1512).**
- R2: emit-1's recipe with `prime-ranked:1`.
- R3: `ngram:4` and `ngram-ranked:4`, post hoc on emit-1 and as 1,500-step fine-tunes.
- The results will be added here. No language result is claimed by this PR.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 5943341966 — 2026-10-02T00:34:23Z — Casey-allard

**Exact-head review: `023af14cbffb29a292870fb7e27b651fcb5c08fc`** (self-review, under the owner rule of 30 September on #820).

**Scope checked.**
- **Ranked route:**
  - admitted sources score their learned score plus 4 × the route score, softmaxed over the admitted set;
  - with nothing admitted it returns `pointer_weights(learned, None)`, the soft pointer's own weights;
  - the backward forms score gradients only from the final weights, so the softmax Jacobian over the kept set is the selection path already tested for flock and top-K;
  - at zero scores it is bit-equal to the exact route (tested).
- **n-let admission:**
  - longest n ≤ W with any match; the source's n-let ends at `j − 1 ≤ t − n`, so it never overlaps the query;
  - matching is by ordered id equality, equivalent to ordered prime-sequence equality since the registry is injective;
  - the weight is Σ ln p, the same for every source at one n, so only recency orders equal matches (tested);
  - the slice indices are in bounds for every t ≥ 0 and 1 ≤ n ≤ min(W, t+1).
- **Configuration:** `ranked` and `admission` are serialized only when not the default, so #1587 routes and older configurations load unchanged (tested). The CLI grammar is `prime|prime-ranked|ngram|ngram-ranked:W`.
- **Merge:** the merge of main brought in #1588's `geometric_stack.rs` changes with no textual conflict. The 38 route and pointer tests pass locally at this head.

**Evidence at this head.** Run https://github.com/UOR-Foundation/uor-r4/actions/runs/36944707250 (evidence commit `3644bf7d`, parent = this head):
- fmt and the check of all targets: success;
- `uor-r4-training` lib: 494 passed, 0 failed, 6 ignored;
- examples `geometric-stack` and `m-world`: pass.

**Measured so far** (#1512, one draw each, M-world v2 development):
- `prime-ranked:1` trained ties the soft pointer (instruction 18 vs 11, responsive 475 vs 460, MQAR 10 vs 16, relation 20 vs 25).
- `ngram-ranked:4` applied post hoc to emit-1 raises MQAR from 16 to 22, with the gains at 4 and 8 facts.
- The trained n-let arms (R3) finish within the hour.

The failing delivery-evidence check is the advisory gate.

Verdict: approve for merge at this head.


### comment 5943478968 — 2026-10-02T00:48:22Z — Casey-allard

**Merged** as `f9c92692` through the queue at head `023af14c`. Main's tree at `f9c92692` equals the PR head's tree. The trained n-let results are in: `ngram-ranked:4` ties the soft pointer (MQAR 15 vs 16, responsive 472 vs 460, relation 15 vs 25); exact `ngram:4` is worse (MQAR 3). Item-paired McNemar: no ranked arm differs significantly from soft. Details on #1512.


---
_Comment/review ids in this file: 5943341966, 5943478968_

