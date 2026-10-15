# #2113 — VSA mode 2 is worse than fixed codes: not KEEP (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-10T00:20:41Z
- **Updated:** 2026-10-10T00:29:51Z
- **Closed:** 2026-10-10T00:29:31Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2113
- **Merged:** 2026-10-10T00:29:30Z
- **Merge commit:** 4c35d83b19dfaa6808c1352bfb92a3c43be3c20d
- **Base / Head:** main / UOR-Foundation:claude-vsa-mode2
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Result 2 of the native VSA test, pre-registered on #2029 and run on the owner's laptop CPU (main `c7fce45b1`, ~50 min, no pod).

- **Mode 2 vs fixed codes:** mode 2 (root codes + per-token readout residual) is worse in all 4 seed × slice cells, by +0.013 to +0.028 BPB.
- **Its own VSA term:** ablating it lowers BPB by 0.002–0.005, with every CI below 0.
- **Decision:** not KEEP. Fixed codes stay the default.
- **Reproducibility:** the laptop rerun of the fixed arm matches the earlier pod run to every printed digit.
- **Hypothesis (untested):** the codebook is rebuilt every 1,000 steps.

Changes: the record (Result 2) in `docs/labs/vsa-native-test-2026-10-09/README.md`, a current-state entry, and the README/STATUS/ROADMAP row.

References #2029

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 6091536321 — 2026-10-10T00:20:51Z — Casey-allard

Exact-head self-review at `6d1f419606d80f916da86070221593a6d47e3e43`: docs only. Every number in the table was copied from `ablate-prose` logs in ~/uor-r4-local/vsa-mode2-20261009/*/eval-*.log (baseline line plus vsa/engram rows), and the refresh setting from the train log (`vsa_codes readout (refresh 1000)`). Claim-wording gate passes.

### comment 6091613854 — 2026-10-10T00:28:20Z — Casey-allard

Merged origin/main at `9bc8e2fc53f532b567739df611e731423613b98f` (current-state.md: both new top entries kept). Docs only; claim-wording gate passes.


---
_Comment/review ids in this file: 6091536321, 6091613854_

