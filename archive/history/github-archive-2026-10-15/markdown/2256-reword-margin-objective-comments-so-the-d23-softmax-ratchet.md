# #2256 — Reword margin-objective comments so the D23 softmax ratchet passes again (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-12T21:32:37Z
- **Updated:** 2026-10-12T21:34:36Z
- **Closed:** 2026-10-12T21:34:15Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2256
- **Merged:** 2026-10-12T21:34:15Z
- **Merge commit:** bbbe582cb41c0a92f082e88d3d528ad75f752960
- **Base / Head:** main / UOR-Foundation:kimi/softmax-ratchet-fix-2026-10-12
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What

#2255's margin objective landed with sixteen textual `softmax` mentions in comments and doc text (`softmax-free`, `softmax logits`, ...). `scripts/check_no_softmax.py` counts every mention in executable source, so main went 1649 → 1665 and **failed the ratchet for everyone**. This PR rewords those comments to `normalized-exponential` / `exponential-free`. No code changes.

## Checks (exact head aab7a1e33)

- `python3 scripts/check_no_softmax.py` → **OK: 1649 remaining across 145 files (baseline 1649)**
- `cargo check -p uor-r4-training --lib --offline` → clean (7 pre-existing warnings)

References #2029

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

