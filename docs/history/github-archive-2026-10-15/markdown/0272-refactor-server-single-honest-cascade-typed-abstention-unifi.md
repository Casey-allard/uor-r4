# #272 — refactor(server): single honest cascade — typed abstention, unified semantics, /engine honored (#248)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-28T18:37:25Z
- **Updated:** 2026-10-09T13:08:55Z
- **Closed:** 2026-07-28T18:48:00Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/272
- **Merged:** 2026-07-28T18:47:59Z
- **Merge commit:** e6240b2d69adc7f91bf6a58cf068e5e1ee605fba
- **Base / Head:** main / UOR-Foundation:issue-248-honest-cascade
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Closes #248.

## What

The serving cascade existed as two divergent inline implementations. Now: **one** implementation (`run_serving_cascade`), built on the previously-dead `FallbackRouter` abstraction (extended with `Abstained` status, per-tier `TierOutcome` trail, `run_cascade`), called by both `/api/chat` and `/v1/chat/completions`.

- **Typed abstention everywhere**: R4G1's D4 abstain is `Abstained` (recorded in the trail; cascade continues — PR #223 semantics, now shared by both endpoints; the policy is one line, `SERVING_ABSTAIN_POLICY`, if review prefers the stricter semantic). The geometric tier's `"Manifold resonance too sparse"` magic string is now a typed abstention — never served as if generated. Validator rejections are `Pathological` with reasons.
- **Declined-by-all is honest**: shared terminal response (`outcome: "declined_by_all"`, full per-tier trail, HTTP 200 when declared-abstained / 503 when all hard-failed) instead of serving the sparse string.
- **`/engine` honored, safely**: explicit non-default selections (attention / geometric / transformerless / r4-attention) pin the cascade to that tier with honest failure if it declines. **Deliberate exception (amendment commit): "r4g1" never pins** — the CLI persists it as the *default*, so literal pinning would silently disable all fallback on default installs; r4g1-first is already the cascade order. Test pins this rule.
- Legacy response fields (`generation_mode`, `r4g1_abstained`, `llm_connected`, …) derived from the trail — shapes preserved for existing consumers; `cascade_trail` added to both endpoints.

## Tests

7 new cascade tests in `fallback.rs` (first-success, abstain-continues, policy, pathological detail, declined-by-all shape, pinned tier, success-without-text) + 3 pre-existing untouched; server-side derive-legacy-fields and pin-rule tests; full 94-scenario BDD suite passes.

## Gates

All four green locally (macOS arm64, pinned 1.97.1, offline), run alongside the in-flight #244 certify without interference. Runtime kernel untouched.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_017tu2Y4AGZvcgM4vmoSz5x6


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

