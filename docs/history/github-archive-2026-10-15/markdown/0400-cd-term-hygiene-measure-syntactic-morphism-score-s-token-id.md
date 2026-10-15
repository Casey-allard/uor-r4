# #400 — CD-term hygiene: measure syntactic_morphism_score's token-id-hash geometry, then re-ground or remove (#393 §7)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-04T00:16:21Z
- **Updated:** 2026-08-04T16:29:27Z
- **Closed:** 2026-08-04T16:29:27Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/400
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Motivation (program: #393 §7)

`syntactic_morphism_score` (`crates/uor-r4-graph-runtime/src/engine.rs`) computes a Cayley–Dickson centralizer score over `CayleyDicksonVector::from_u32(token)` — a multiplicative hash of the **raw token id**. It carries no corpus, teacher, or geometric content (the input is pseudo-random by construction) and predates the measurement discipline. The same function hard-codes token-id lists for periods/commas/prepositions/determiners — the only explicit syntax mechanism in the serving path, tied to the legacy tokenizer's vocabulary.

## Plan (measure first, then decide)

1. A/B the term as-is on the shipped surfaces: certify rows and #394 infill rows with the CD contribution zeroed vs enabled (single batched run; cheap-check-first per discipline). Report deltas.
2. If ~zero (expected): remove the CD term; keep or replace the hard-coded lists only per their own measured contribution, with the #394 criterion as the replacement path (superseded by the E_f/E_b mechanism issue).
3. If nonzero: investigate what it is actually correlating with before any re-grounding (a centralizer score over token-id hashes carrying signal would indicate an unintended coupling worth understanding).

Re-grounding option (only if pursued after measurement): operate on H(x)/region geometry — e.g. icosian block codes (#395) — where a centralizer score has meaning.

## Non-goals

No silent removal without the A/B record; no new scoring channels.


## Comments (4)

### comment 5173556219 — 2026-08-04T01:31:18Z — Casey-allard

## Scope finding (measured by code trace, ahead of the A/B)

`syntactic_morphism_score` is **not reachable from the certify C-row serving surface**: `uor-r4-api`'s `R4Engine` (predict_decision → score_sig paths) contains zero references to it. The term lives only in `R4G1Runtime`'s generation paths (`predict_candidates*` / `predict_distribution*`) — i.e. the chat/beam surface (`generate_r4g1_response`, server/chat).

Consequences for this issue:

1. Any A/B on certify rows would read exactly zero **by construction** — the C row never pays the term. The honest A/B runs on the R4G1 generation surface.
2. Harness is written (`r4g1_cd_ab.rs`): teacher-forced next-token through `R4G1Runtime::predict_candidates` over the held-out fixture partition, `score.r4g1` built from the checked-in fixtures via `convert-r4g1`. Arms = stock build vs a local, uncommitted patch zeroing the term (no shipped toggle unless the measurement justifies one).
3. Whatever the number, the hard-coded token-id lists in the same function (periods/commas/prepositions/determiners) are load-bearing for the chat surface only — consistent with the #393 two-stacks picture: the serving stack that certify grades and the surface users talk to are not running the same scoring code.

Numbers to follow tonight (runs queued behind the #395 v1 compute).


### comment 5174951076 — 2026-08-04T05:21:50Z — Casey-allard

## A/B status: BLOCKED on a prediction-path diagnosis — recorded before parking

Harness ran end-to-end (fixture corpus → `convert-r4g1` bundle → `R4G1Runtime::predict_candidates`, 99.6-100% served), but every position returns the SAME candidate: token 256 at a near-saturated score (ScoreQ ≈ 2147422xxx), top1/agree exactly 0.00%. Two facts from the diagnostic:

1. Passing the real ROUT context signature (`bundle_window_plain` + `sig_plain`, exactly as `generate_r4g1_response` does) *changes the winning score* but not the winner — the signature is consumed, yet one emission entry dominates everything.
2. The bundle was built tonight via `convert-r4g1 --artifacts <fixture TLA> --store <rebuilt TLS1>` (179,067 keys migrated, root prior 4,764 entries, calibrated radii **0**).

Working hypothesis, unverified: the converted bundle carries a sentinel/prior entry (token 256, uncalibrated radii) that saturates the candidate path — i.e. the failure is in this bundle's construction or in consuming a conversion-produced bundle through the beam path, not in the CD term. The A/B itself (stock vs zeroed `syntactic_morphism_score`) is meaningless until predictions are non-degenerate.

Parked pending: how the chat surface's OWNED_R4G1 bundles are actually produced (if not via this convert path, that difference is the suspect), and whether calibrated radii are required for candidate scoring. The scope finding stands regardless: certify's C row never exercises this code; the serving cost observation stands too (~0.4-0.5s/position through this path on 2 shared cores — orders of magnitude above the store paths).


### comment 5179620810 — 2026-08-04T13:18:10Z — Casey-allard

## Diagnosis update: the degeneracy is architectural, not a bundle defect — `context_backoff` short-circuits the entire graph path

Reproduced the constant-candidate behavior across BOTH bundle families (convert-r4g1 fixture bundle AND a score-pipeline natural bundle, fully matched corpus/artifact stacks). Trace through `predict_distribution_with_signature_lanes`:

1. It calls `context_backoff(base_graph, context_tokens)` FIRST and **returns immediately** on any `Some` — before any node activation, before the suffix-chain walk, before `node_scores` is touched.
2. Both tested bundles are rows-off (`--context-order 0` penalty arms; convert migrates no NGRAM), and backoff then yields a constant root-prior argmax (token 2379 / 256, score varying only with the window bonus) on every position.
3. `predict_candidates` therefore returns exactly one candidate; the node-emission iteration finds nothing (`node_scores` all MIN); `syntactic_morphism_score` — which only applies to node-derived candidates — **never executes**.

Consequences for this issue and #393 §7:

- The CD term's real firing condition on the chat surface is "context_backoff failed to resolve" — much narrower than assumed. Any A/B must use a rows-ON bundle (default `--context-order 2`) AND measure the term only on the positions where the graph path actually runs (backoff misses).
- Separately notable for the architecture docs: in this path the context signature plays no role in node activation (activation = token-emission suffix matching); and on rows-on bundles the graph chain is reached only in the backoff-miss tail. The "graph runtime" of the chat surface is, in the common case, an n-gram backoff.

Next steps: read `context_backoff`'s None conditions; build a rows-on fixture score bundle; rerun the A/B on the backoff-miss slice. Queued behind today's decider runs.


### comment 5181674028 — 2026-08-04T16:12:04Z — Casey-allard

## A/B superseded by a stronger result: execution count is ZERO — removal PR #410

Rows-on (production-shaped) fixture bundle, 1,998 served positions (99.6% coverage), healthy pipeline (36.49% top1 — matches the score pipeline's 36.5%): the multi-candidate slice — the only place syntactic_morphism_score can execute — ran **0 times**. No A/B arms needed: the term is not approximately worthless, it is unreachable in practice. Removal verified behavior-identical (36.49%/40.68% before and after). PR #410 closes this issue with the full record: scope finding (certify never sees it) → root cause (context_backoff short-circuits the graph path) → measurement (0/1,998) → verified-identical removal. The hard-coded punctuation/POS token lists in the same dead path go with it; the pub cayley_dickson module remains (unreferenced by the engine) for a separate disposition call if anyone wants one.


---
_Comment/review ids in this file: 5173556219, 5174951076, 5179620810, 5181674028_

