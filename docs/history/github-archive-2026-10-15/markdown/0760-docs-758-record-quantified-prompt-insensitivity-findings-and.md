# #760 — docs(#758): record quantified prompt-insensitivity findings and mechanism

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-17T05:21:27Z
- **Updated:** 2026-08-17T08:42:14Z
- **Closed:** 2026-08-17T05:42:27Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/760
- **Merged:** 2026-08-17T05:42:27Z
- **Merge commit:** 25fd9d6ecc9593c14b33e0f5be4823bec2c65e7e
- **Base / Head:** main / UOR-Foundation:issue-758-prompt-insensitivity
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

Docs-only PR closing out #758, a direct follow-up to #755's noted residual prompt-insensitivity. No code changed.

Ran a controlled 15-prompt comparison (12 real questions + 3 edge cases, exact-match hashed rather than eyeballed) against two bundles:

- **`smollm2-135m-instruct`** (rebuilt post-#755 fix via `r4 transformerless compile-recorded`): 9 distinct outputs across 15 prompts, largest collapse group 5 prompts byte-identical.
- **`smollm2-360m-broad`** (#509's larger, cleaner-corpus bundle — unaffected by #755 either way, its records were already 0% non-contiguous): 11 distinct outputs across 15 prompts, largest collapse group 4 identical.

Traced the mechanism directly in the code (`crates/uor-r4-graph-runtime/src/engine.rs`, `crates/uor-r4-core/src/transformerless/runtime.rs`): `r4 ask` genuinely uses the prompt's trailing tokens as a lookup key, but on lookup miss — the common case for arbitrary out-of-corpus prompts — both the R4G1 path and the legacy plain-TLA path collapse to one context-independent default shared by every prompt that reaches that tier. Larger corpus coverage reduces how often the fallback fires but doesn't remove the underlying design issue.

Full findings posted on #758; fix tracked separately as #759 (design change with architectural reach, following this repo's established diagnose-then-fix pattern from #752 → #755).

## Verification

- `python3 scripts/check_claim_wording.py` — pass
- No Rust files touched; full cargo verification ladder not re-run (no code change), consistent with prior docs-only PRs in this repo.

Closes #758
References #745, #755, #759

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

