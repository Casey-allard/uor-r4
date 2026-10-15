# #999 — research: qualify direct geometric attention V3 and freeze V4 direction

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-29T04:47:54Z
- **Updated:** 2026-10-09T14:17:19Z
- **Closed:** 2026-08-29T04:50:15Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/999
- **Merged:** 2026-08-29T04:50:15Z
- **Merge commit:** 5ccc739927b1a2d6e662670b6de585bf39c4088a
- **Base / Head:** main / UOR-Foundation:issue-973-geometric-gated-delta-retention
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #973

## Outcome

This delivers the first bounded gated-delta cell and a direct dense causal Q/K/V/O attention oracle over R4 state, then binds the negative and positive evidence without promoting either beyond its scope.

- `GeometricGatedDeltaRetentionR4V1` is structurally exercised but does not beat the matched plain delta control: full geometric `16/28` next-token and `55/112` association versus plain `23/28` and `98/112`.
- Historical direct-attention V2 is `NON_PROMOTABLE_BUDGET_MISMATCH`.
- Fresh equal-manifold-budget V3 returns geometric `3/12`, plain fixed-tangent `12/12`, current-only `6/12`, alternative-connection diagnostic `10/12`, key-isometry `7/12`, order-shuffled `5/12`, and value-permuted `8/12`.
- The binding conclusion is narrow: dense causal learning, softmax, and value aggregation work; the tested mixed-gauge H4 projection/connection/optimizer combination does not. The H4 group action itself remains algebraically valid.
- The next and only active rung is `ConnectionGaugeCovarianceV4`, with explicit local coordinates and separately trained H4, alternative-tangent, and fixed-tangent arms.
- Paired-H4/E8 hierarchy input, corpus qualification, resonance replacement, recurrent factorization, exact lowering, autonomous generation, and #954 are still `NOT_RUN` or blocked.

The repository programme documentation, roadmap, glossary, lifecycle, research summary, ADR, and historical forward-action records now agree on that sequence. The resonance audit retains the multi-resonance sieve as the intended post-oracle softmax replacement, not as current evidence.

## Frozen identities

- V3 artifact: `blake3:136f0bac7361ca77a30946d8f120843b2d42eacccedbb1f64ab604af3cdf50f3`
- V3 experiment: `blake3:8ff5c7f584f82c8fc7cf39ebbd28274140f4de3faf05eff370bc22e7aa429785`
- GGDR artifact: `blake3:6bf22c9c5283b971d8a9e5e7f4bce067424064a49394f5c7b02a2174e6f38973`
- GGDR report: `blake3:f99b9815044f139ec0380b5a82502aaf4e159e25761626c0c25eee39173816e1`

## Verification

- direct-attention unit tests: `6/6` PASS
- direct-attention integration tests: `7/7` PASS
- gated-delta unit tests: `8/8` PASS
- gated-delta integration tests: `3/3` PASS, including all frozen tuple and CID assertions
- touched core all-target check: PASS
- root WASM library check: PASS with existing warnings
- graph-format no-std and alloc ladders: PASS
- BDD: `124/124` scenarios and `414/414` steps PASS
- formatting, claim-wording, and diff-whitespace gates: PASS
- workspace no-fail-fast run was partitioned after it exposed only three baseline-drift failures; the repaired prime-route source audit, model-source closure, and browser-selection BDD targets each pass on rerun
- strict workspace clippy remains unavailable as a change-specific verdict because current main already has unrelated denied warnings; no new-file lint blocker was found

## Baseline-drift repairs exposed by verification

This PR also records three narrow repository-wide repairs that were required to make the existing gates describe current behavior: moving an offline compiler helper outside the runtime-source audit markers, completing the exact production-source closure list, and distinguishing the server R4G1 default from the browser geometric-demo default. These are called out explicitly because they are not scientific evidence for #973.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

