# #1440 — Lab 2: localize the source-answer failures to the read ranking

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-27T22:05:51Z
- **Updated:** 2026-10-09T13:09:02Z
- **Closed:** 2026-09-28T05:33:45Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1440
- **Merged:** 2026-09-28T05:33:45Z
- **Merge commit:** fa9faf16290ec85a77449a5a9ea9c3a71883692e
- **Base / Head:** main / UOR-Foundation:lab/opencode/read-localization
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Lab 2 (OpenCode) — draft for cross-lab review

**Do not merge without director approval.** This PR follows the September 27 three-lab working agreement (`lab/<lab>/<topic>` branches, draft PRs, director-only merges) and supersedes this lab's earlier self-merged delivery style.

## Summary

A read-only localization instrument for the mainline joint learner, executed once on the frozen step-15,672 quaternion parent: decision-0 read-mass attribution over the 32-row source panel under five declared prompt conditions, to localize the measured "wrong first noun" failures. No training, no weight change, no serving-path change; the runtime constraints (no transformer / matmul / float in serving) are untouched. Frozen contract: `docs/integration/read-localization-plan-2026-09-27.md`.

## What shipped

- `crates/uor-r4-training/src/read_localize.rs` (new): token-id helpers (exact entity-span resolution, splicing, normalization/causality checks), a canonical deterministic greedy loop capturing the decision-0 `JointStep` read masses, 5 focused tests.
- `crates/uor-r4-training/examples/joint-read-localize.rs` (new): loads the sealed parent checkpoint + campaign, runs conditions A/B/C/D/D′, writes a self-verifying report. The only shared-file edit is one module registration in `lib.rs`.
- `ROADMAP.md`: Lab 2 track entry (goal, hypothesis, status, next decision point) plus the parked-path list for this track.

## Result: predeclared **MIXED**, with a post-hoc READ_RANKING refinement

Baseline parity is exact on all 32 rows (prompt ids, generated ids, stop, text, both verdict flags, and decision-0 read fields) against the retained parent `story-probes.json`; masses normalize everywhere; the full run is 1.29 s.

| Class | Rows | A | B | C | D (entity mention) | D′ (matched control) |
|---|---|---|---|---|---|---|
| Distractor copy (`clouds.`) | 5 | 0/5 | 0/5 | 0/5 | **5/5** | 0/5 |
| Correct noun + extra phrase | 5 | 0/5 | 0/5 | 0/5 | 3/5 | 4/5 |
| Morphology (`pears.`) | 1 | 0/1 | 0/1 | 0/1 | 0/1 | 0/1 |

**Predeclared MIXED** (not relaxed): the entity share < 5% / rank > 5 clause failed (0/5 — entity present at rank 2–3, 16–32% share), and the share-or-rank disjunction is non-discriminative (D 39–64% share vs D′ 1.4–4.2%, but sparse D′ rows attained rank 3); STATE_EMISSION failed because D changed the verdict in 5/5.

**Post-hoc (labeled):** in the five condition-A distractor rows the emitted token equals the read top-1 (` clouds`) while the entity sits at rank 2–3; the entity mention completes 5/5 and the matched non-entity control completes 0/5 while emitting the inserted noun (` carrot`) in 5/5. At this scope the distractor failures are consistent with a **read-ranking/recency effect**, not entity absence. Corrections applied from the independent audit: the top-1 identity is asserted only for condition-A distractor rows (the copy path aggregates per-token mass; e.g. D `04|edited` emits ` bear` while the top single occurrence is ` clouds`), and D changes recency, duplication and local context together, so the flip is also consistent with local clause continuation. The extra-phrase class is not entity-specific (D 3/5 vs D′ 4/5); the morphology row emits the correct noun at rank 1 and fails only the completion.

## Limits and next decision point

Exposed 32-row development panel, one parent, one deterministic policy; no fresh holdout. The predeclared rule returned MIXED; READ_RANKING is a post-hoc signature at this exact scope and does not show that a long-range ranking repair is learnable. **Recommended next step (needs director sign-off): an oracle read re-rank intervention** — clamp decision-0 read mass onto the entity occurrence and check whether the distractor rows flip — to causally confirm the ranking bottleneck before any mechanism change.

## Artifacts and checks

Executed `joint-read-localize` release example sha256 `1c1c589d…` (preserved in the model store `binaries/`); module/example/retained-file shas verified; superseded attempts preserved (`…-opencode-1`). Evidence `docs/evidence/read-localization-2026-09-27.json`; result `docs/integration/read-localization-result-2026-09-27.md`; `current-state.md` updated. Independent evidence audit performed; its corrections are applied (aggregation caveat, share-or-rank wording, mutation divergence, causality declaration, provenance). Cost charged once on the shared ledger with a receipt through draft-PR creation; merge awaits the director.

References #973, #820 (partial; does not close either).


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5864080211 — 2026-09-28T05:32:11Z — Casey-allard

**Director: merging under the owner's instruction to clear the PR backlog.** Merged main into this branch (`b52d08f9`). `ROADMAP.md` keeps main's version, because the director roadmap supersedes this branch's early lab-brief section. The read-localization result stays in this PR's plan, result and evidence files and in the roadmap's T2 row. No other file changed. References #820.


---
_Comment/review ids in this file: 5864080211_

