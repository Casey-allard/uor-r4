# #2099 — Native VSA retraining result: KEEP (trained VSA helps); root-only codes collapse token identity

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T22:08:32Z
- **Updated:** 2026-10-09T22:24:07Z
- **Closed:** 2026-10-09T22:09:32Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2099
- **Merged:** 2026-10-09T22:09:32Z
- **Merge commit:** 60a6ef15dbd9521c49df822f94647a90b885beec
- **Base / Head:** main / UOR-Foundation:claude-vsa-result
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Result of the pre-registered native VSA test (M1 #2029; plan and code #2077):

- **Fixed-code VSA, trained into the learner,** improves held-out BPB by 0.014–0.023 over VSA-off on both slices and both seeds. Its within-model contribution is +0.013 to +0.016, with 95% intervals excluding zero. **KEEP.**
- **Icosian-root codes** (120 distinct codes for 4,096 tokens) give at most −0.005. They collapse token identity.
- **The engram table** carries about 0.2 BPB.

DeepSeek's null was a frozen-artifact effect.

**Limits:** the intervals are within-model ablations, because per-position losses weren't kept for cross-arm pairing. Seed spread is about 0.015.

**Next:** mode 2 (root code plus a per-token residual), run locally after a CPU-parallelism fix.

**Cadence:** dated record `docs/labs/vsa-native-test-2026-10-09/README.md`, a newest-first `current-state.md` entry, the experiments tables in README, ROADMAP and STATUS, the README results line, and the `geometry.md` conversion table. Claim wording passes.

References #2029

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6090069744 — 2026-10-09T22:08:33Z — Casey-allard

Exact-head self-review at `1b38ed9defe84e02b72820aaff03d628cba4e8e7`: numbers transcribed from the 16 ablate-prose logs (archived to iCloud as vsa-native-test-20261009); claim wording passes.


---
_Comment/review ids in this file: 6090069744_

