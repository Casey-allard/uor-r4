# #894 — eval(#841): free-running trajectory gap — frozen contract + run-1 (the gap is total)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-21T08:06:18Z
- **Updated:** 2026-10-09T13:08:56Z
- **Closed:** 2026-08-21T08:28:21Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/894
- **Merged:** 2026-08-21T08:28:21Z
- **Merge commit:** 11d7ca8e2cce6588f555b483b2f424339be0af8e
- **Base / Head:** main / UOR-Foundation:issue-841-free-running-gap
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Problem and outcome

Teacher-forced and recorded-replay signals do not measure how errors compound under the system's own prefixes; the project lacked a matched, step-attributed measurement of where teacher-forced quality ceases to predict free-running behavior. This PR freezes that evaluation and executes run-1 against the normative deployed `R4Engine` — the first offline gap instrument the S3 tracker (#824) explicitly sanctions once the S0 evaluation is frozen (S3 STAGE closure remains gated on the #822/#823 stage verdicts, untouched here).

**Run-1 establishes that the gap is total:** median first-divergence step **0** at H=32 greedy (590‰ of student rollouts depart the recorded text at the first generated token; 0‰ survive even H=8 on-text, against a 304‰ matched teacher-forced per-step ceiling); **99/100 free-running rollouts are token-identical to suffix-only (2-token) rollouts** — generation, like scoring (#874/#834), is memoryless beyond the suffix; **710‰ of rollouts collapse into ≤4-period cycles** (vs 110‰ under teacher prefixes); and **zero typed abstentions on any side** — the #811 always-servable finding at trajectory scale. No coherence is claimed; no corrective mechanism is prescribed.

## Scope

- `docs/free_running_eval_841.md` — the frozen contract: versioned prompt families (v1 = 100 held-out story prompts, CID-bound; future families extend domains without renumbering), horizon ladder {8,16,32,64} (run-1 executes {8,32}), greedy + seeded-sampled modes reported separately (sampled UNAVAILABLE in run-1, no pinned decoder-seed identity), matched sides (teacher-prefix / student-prefix / shuffled-prompt / repeated-prefix / suffix-only), trace schema v1 with per-step exact-context/ngram/graph/decline attribution and CID binding, deterministic primary metrics (first-divergence, TF agreement, serve/abstain/widen, cycle detector, distinct-1, prompt-overlap proxy, termination), versioned-judge discipline (judge metrics without a pinned judge identity are UNAVAILABLE), and the frozen corrective stopping rule for #840 (median first-divergence +≥2 steps AND at0 −≥100‰ per round, no TF regression >10‰, ≤3 rounds else GENERATION-NOT-ESTABLISHED).
- `crates/uor-r4-api/tests/free_running_eval_841.rs` — the instrument + run: trace schema round-trip/tamper/truncation teeth; first-divergence locator on planted cases; planted early-drift model (distinct-but-wrong from step 0 — distinctness alone cannot pass) and repetition-only model (flagged by the cycle metric) both fail as required; overlap/median-CI determinism; judge-absence → UNAVAILABLE; the ignored engine-driven run (greedy rollouts with per-trajectory reset, double-run determinism asserted, shuffled-prompt non-degeneracy asserted at run scale).
- `docs/free_running_841_result.json` — CID-bound record (corpus_meta_cid `blake3:aa9d1767…`, family cid `blake3:6cad1dfe…`, traces cid, result_cid `blake3:59ec2f60…`): per-mode aggregates at both horizons, TF agreement, suffix-locality statistic, controls, UNAVAILABLE markers.
- `docs/RESEARCH.md` — S3 item-A summary paragraph appended.

## Non-goals

Aggregate teacher-forced top-1 is not used as a generation metric (the whole point); lexical diversity/grammar alone are not treated as coherence (the planted early-drift negative enforces it); prompts/horizons will not change after corrective results (frozen); no serving-path changes; no ids.toml row; no CONFORMANCE regeneration (evaluation infrastructure; RF-14/21/22/29 evidence language).

## Acceptance-criteria status (issue #841)

- Teacher- and student-prefix measurements share exact prompts, positions, identities, metrics — DONE (matched sides from identical windows; family + corpus CIDs pinned).
- First divergence and scorer/state provenance replayable — DONE (trace schema v1, per-step path attribution, CID-bound canonical bytes, greedy double-run asserted).
- Controls distinguish decoder effects, support gaps, state starvation, simple repetition — DONE (teacher-prefix doubles as the decoder-only control; suffix-only isolates the memoryless floor; repeated-prefix is the repetition reference; shuffled-prompt isolates order; the run separates cycle collapse from divergence).
- Power and thresholds frozen before corrective fitting — DONE (n=100, order-statistic median CI, the §6 stopping rule; frozen BEFORE #840 exists as work).
- Planted early-drift and repetition-only models fail the intended metrics — DONE (non-ignored CI tests).

## Verification

Run contract posted to #841 BEFORE the run; outcome posted against it. Fixture teeth 5/5 green (CI-run); the run is deterministic (asserted); local gates green before push: fmt, claim-wording, clippy -D warnings (workspace/all-targets/all-features), no_std ×2, wasm lib, xtask validate, repo-conformance, cargo-deny bans, doc tests, workspace suite (0 failures across every binary; the local-model bdd soak is CI-skipped by design).

## Compatibility and migration

Evaluation-only; the schema is versioned; existing reports remain readable and cannot satisfy the free-running claim without the new identities (as specified).

## Conformance effects

None generated; evidence-language extension only.

## Claim-boundary changes

`docs/RESEARCH.md` now records the quantified gap and its diagnostic categories (immediate off-text divergence, memoryless generation, unguarded cycle collapse, zero abstention). This narrows claims; nothing is promoted.

## Negative or unavailable results

Sampled mode and judge metrics: UNAVAILABLE in run-1, recorded explicitly (no pinned identities). The state-shuffle control is UNAVAILABLE by construction today (no deployed stateful mechanism to shuffle) and the contract says so.

## Intentionally excluded

Pre-existing untracked user dirs (`obs-text/`, `tf1-pkg/`) are not staged.

Closes #841.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_01J2R2qs9kfcYpsjUua1SYmi


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

