# #283 — feat(runtime): single-key store + read-time query-beam — the #244 decision (#281)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-29T16:14:09Z
- **Updated:** 2026-10-09T13:08:55Z
- **Closed:** 2026-07-29T17:53:06Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/283
- **Merged:** 2026-07-29T17:53:05Z
- **Merge commit:** d7526830450bdaec492059ee81a807417b1d14d4
- **Base / Head:** main / UOR-Foundation:issue-281-single-key-store
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What

Implements the #244 decision per the plan on #281: evidence writes under the primary code prefix only (2.56× fewer keys), with the membership fan-out moved to read time via new beam predict variants on both the plain and kernel-counted paths. Serving's transformerless tier runs on the beam path (`assign_window_memberships` → `predict_witness_beam`/`_with_priors_beam`). `build_store_multi` retains the old fan-out for the certifier's ablation row only.

Certify matrix continuity: A (shipped: single-key + query-beam) [expected 28.2/31.9 @ ~134,733 keys], A-multi (ablation, write-time fan-out) [the recorded 28.2/31.9 @ 344,604], A-single (ablation, no beam) [27.0/30.5]; equality witness now witnesses kernel==plain on the beam path.

Op discipline: merges are adds + table reads counted through the existing OpKernel hooks; multiply remains 0; mul-free contract §4 unchanged. Era note: TLS1 store κ changes by construction; bundles regenerate at next compile; era-sensitive test expectations hold unchanged.

## Measured basis

#244 matrix, reproduced bit-for-bit ×3 and across two independent implementations (PR #282 comment). DoD per #281: shipped A row within 0.1pp of 28.2/31.9 at ~134,733 keys on the next D3 certify run.

## Gates

cargo test --workspace --offline: 71 suites ok. clippy --all-targets --all-features -D warnings: clean. fmt: clean. Runtime kernel ops unchanged in kind.

Closes #281. Refs #244, #282.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_017tu2Y4AGZvcgM4vmoSz5x6

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5120847329 — 2026-07-29T16:47:00Z — Casey-allard

**Gate C fixture price, disclosed (era-note re-pin in the last commit).** The regression gate correctly caught the deliberate #281 semantics change on the CI fixture corpus: top-1 0.317→0.300 (−1.7pp, within the 2pp threshold) and WB 9.86→10.2357 (+0.3757, over the 0.1 threshold — hence the re-pin, thresholds unchanged). On D3 — the scale the #244 decision was made at — the price is ±0.0pp top-1 and +0.087 WB at 2.56× fewer keys. The fixture's 150-story miniature dilutes sparse merged evidence harder than the real corpus; if the team weighs the fixture signal differently, the revert is a one-function swap (build_store_multi + primary-key predicts). Automerge stays armed; CI reruns on the re-pin commit.


---
_Comment/review ids in this file: 5120847329_

