# #1740 — Tiered eval v2: tier C results for balp-s1, 100m-C and 29m-B (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-05T14:05:57Z
- **Updated:** 2026-10-09T13:08:48Z
- **Closed:** 2026-10-05T14:08:05Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1740
- **Merged:** 2026-10-05T14:08:05Z
- **Merge commit:** 0f443ab3711dd2a916c4b3440885005eff829dc1
- **Base / Head:** main / UOR-Foundation:codex/tiered-eval-v2-results
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #820. Stacked on `codex/tiered-eval-v2` (instrument at `b04baa47`, review approve, no blocking findings).

Adds `docs/research/tiered-eval-v2-results.md`: per-category tier C results on `conversational-v3` (the tiered-eval-v2 panel with the `exact` memory and `abstain_exact` checks) and on `conversational-v2` (#1733 record panel), graded with qwen2.5:7b, greedy, max_new_tokens=64, with per-category best-constant McNemar and the paired balp-s1 vs 100m-C comparison.

Key v3 results:
- balp-s1 35/160, 29m-B 34/160, 100m-C 27/160 acceptable. balp-s1 and 29m-B beat their best constant on self_contained_instruction and smalltalk_feelings. 100m-C beats it only on self_contained_instruction. Constant 1 wins on clarify and unknowable for every model.
- Exact memory check without the grader: balp-s1 7/30 vs 0/30 for every constant (p=0.0156). 100m-C 1/30, 29m-B 2/30. The grader accepts only 2 of balp-s1's 7.
- balp-s1 vs 100m-C, paired: acceptable 35 vs 27 (p=0.134). Check pass 35 vs 13 (p=6e-05), driven by abstain_exact (13 vs 0).

Artifacts: `~/uor-r4-local/ladder/grades/tiered-v2-2026-10-05/` (3.4 MiB). Documentation only; no code change. Not merged by the author.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5996133598 — 2026-10-05T14:07:09Z — Casey-allard

Retargeted to main so the reviewed stack lands as one squash: #1724 (tiered eval v1), #1734 (instrument v2/v3: exact memory scorer, abstention checks, per-category constants, binding-swap controls, text-only ill-posed rule; adversarial review **approved**) and this PR (results doc).

**Exact-head check at 7c26a969e001bb9a4c1a7da746b2b6b8c49e59ab:**
- `cargo test --bin chat-grade` passes 14/14 at the #1734 head. This PR adds documentation only.
- v1 panels are kept as the record.
- Panel hashes are frozen in `data/panels/MANIFEST.sha256`.

Queuing; #1724 and #1734 close as included. References #820


---
_Comment/review ids in this file: 5996133598_

