# #992 — #973 Gate 0: make prior-sentence R4 state load-bearing

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-28T15:44:58Z
- **Updated:** 2026-10-09T14:17:21Z
- **Closed:** 2026-08-28T15:46:29Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/992
- **Merged:** 2026-08-28T15:46:29Z
- **Merge commit:** 6c1a38a9574e9b6f376b76e159f065d6ea5d167b
- **Base / Head:** main / UOR-Foundation:issue-973-higher-scope-attention-v1
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What changed

- adds the construction-bound `PriorSentenceCountRadiusR4V1` operator over #953's existing maximum-count tie;
- makes exact candidate occupancy before the final fitted period the fourth R4 coordinate, with all fixed-point values and squares compiled into a canonical bound artifact;
- runs real, scope-disabled, and deterministic candidate-permuted controls over identical support and declared work;
- adds explicit abstention, fail-closed bounds, binding/tamper rejection, and a bounded continuation that returns to unchanged #953 selection after the first decision; and
- records and reconciles the frozen #973 Gate 0 result.

## Frozen result

Two synthetic D3-partitioned prompts had the same final ` asked` / ` for` frame, exact `{ coffee, tea }` max-count tie, identical #953 local coordinates, support, and declared work.

| Arm | Earlier `tea` | Earlier `coffee` | Exact targets |
| --- | --- | --- | ---: |
| real | ` tea.` | ` coffee.` | 2/2 |
| scope-disabled | ` coffee.` | ` coffee.` | 1/2 |
| candidate-permuted | ` coffee.` | ` tea.` | 0/2 |

Support/work mismatches and teacher/provider/source-weight/future-unit reads were zero. Operator, target-free census, and decoded-smoke identities replayed exactly. Terminal:

`RETAIN_GATE0_PRIOR_SENTENCE_ATTENTION_CONTINUE_PARAGRAPH_CONVERSATION`

This establishes one bounded exact-candidate prior-prefix lexical copy-attention mechanism through decoded output. It does not establish semantic transfer, natural-distribution generalization, general paragraph/conversation/global attention, coherence, correctness, reasoning, chat, performance, formal closure, or release readiness. #973 remains open and #954 remains blocked.

## Activated checks

- `cargo test -p uor-r4-core --offline --test prior_sentence_count_radius_attention_973 -- --nocapture` — 3/3 PASS
- `cargo check -p uor-r4-core --offline` — PASS
- targeted changed-core clippy after suppressing only pre-existing origin-main lint families — PASS
- `cargo fmt --all -- --check` — PASS
- `python3 scripts/check_claim_wording.py` — PASS
- `git diff --check` — PASS

Unmodified strict core/dependency clippy was exercised but current `origin/main` already triggers the pinned toolchain's warnings in untouched files, so it supplies no change-specific verdict. Broad QA and corpus-scale work remain `NOT_RUN`; PR/merge checks are the repository's documented transport acknowledgements.

References #973


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

