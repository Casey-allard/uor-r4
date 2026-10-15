# #1733 — Re-score chat models on the tiered eval: tier C does not beat the 'I'm not sure' constant (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-05T10:39:28Z
- **Updated:** 2026-10-09T13:08:48Z
- **Closed:** 2026-10-05T14:14:23Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1733
- **Merged:** not merged
- **Base / Head:** main / UOR-Foundation:codex/tiered-eval-rescore
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #820. Stacked on #1724 (base `codex/tiered-eval` at `f3737c28`), so the diff is one doc. **Do not merge** until #1724 lands and an exact-head review approves.

## What

This PR re-scores seven existing chat models on the #1724 tiers. It adds `docs/research/tiered-eval-2026-10-05.md`, which holds the full tables, method, cost and the list of sealed report roots. No code or model changed.

- **Tier C:** a new `chat-grade grade` run for each model on `conversational-v2`. Settings: qwen2.5:7b, `max_new_tokens=64`, greedy, with the default constant and derangement controls.
- **Tier K-clean, K-ill-posed and everyday:** `chat-grade tiers` applied to the existing `*-powered-7b` reports, with no regeneration.
- **Pairs:** paired exact McNemar per tier, against chat-29m-B-lr5e-4 and against chat-100m-C.

## Result

| model | C acc. /160 | p vs const-1 (42/160) | K-clean /110 | everyday /32 |
|---|---|---|---|---|
| chat-29m-B-lr5e-4 | 34 | 0.35 | 10 | 9 |
| chat-100m-B | 35 | 0.40 | 12 | 9 |
| chat-100m-C | 27 | 0.049 (worse) | 12 | 8 |
| chat-100m-L | 27 | 0.032 (worse) | 12 | 5 |
| chat-100m-Cx | 25 | 0.014 (worse) | 8 | 7 |
| chat-100m-L12 | 25 | 0.019 (worse) | 11 | 4 |
| chat-100m-C12 | 33 | 0.24 | 10 | 8 |

- **Tier C:** no model beats the fixed reply "I'm not sure. Can you tell me more about what you mean?", and five models are significantly below it. The constant scores 18/24 on clarify and 15/24 on unknowable. The models score 0–1/24 on unknowable, because they invent answers.
- **Memory:** 0–3/30 for every model.
- **Pairs:** no pair differs on any tier at p < 0.05, against either reference.
- **Controls:** every model beats the derangement control and the other two constants.

## Instrument issue for #1724

As currently weighted, the tier C total can be beaten by a constant. Either report tier C per category against the best constant for that category, or make the clarify and unknowable checks require the model to abstain only when it should and to answer when it can. This is written up in the doc and is not changed here.

## Checks

- `chat-grade check` on v2: pass, worst case 323/384.
- Panel sha256 values match `MANIFEST.sha256`.
- The constant tallies were the same (42, 7, 2) in all seven runs.
- All 42 report roots are sealed.
- `scripts/check_claim_wording.py`: pass.

Cost: 2 h 51 min of sequential grading (2 threads plus ollama, one job at a time, lock registered and then removed) and a 7 min 49 s build. New storage is 2.4 MiB.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5996269438 — 2026-10-05T14:14:22Z — Casey-allard

Replaced by the doc-only PR from branch codex/tiered-eval-rescore-doc. Its document lands there, and its other commits are already on main via #1740.


---
_Comment/review ids in this file: 5996269438_

