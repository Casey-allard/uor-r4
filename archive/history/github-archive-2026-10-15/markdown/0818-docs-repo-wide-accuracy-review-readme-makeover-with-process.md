# #818 — docs: repo-wide accuracy review + README makeover with process diagrams

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-19T03:15:32Z
- **Updated:** 2026-10-09T13:08:50Z
- **Closed:** 2026-08-19T03:35:41Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/818
- **Merged:** 2026-08-19T03:35:41Z
- **Merge commit:** 2ee0790a9d00f9ead1d8a4033973a0f7560ec7e9
- **Base / Head:** main / UOR-Foundation:docs-review-post-655
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

Maintainer-requested repo-wide documentation review after the #655/#741/#811 closes and the v0.1 release, plus the README makeover: canonicalized accurate information, written so someone with no idea what this repo is understands it before the reference material starts, with **four Mermaid process diagrams** that render natively on GitHub (text-maintained, no image assets to drift).

### README makeover

- **Plain-language opening** — the pay-the-math-once framing, what "compiling a teacher" means, the witness/abstention honesty model, who it's for, explainers linked up front.
- **Diagrams**: "The idea in one picture" (offline compile vs online integer serving) · architecture (kept) · "How a request is served" (profile gate → D4 policy → decode, every outcome typed) · "Releases and verified install" (tag → CI → published release → hard-verified install). All four render-validated.
- Quick start gains the **2-minute path**: `install-release --tag v0.1` + `ask`.
- "What actually works" refreshed to the measured state: honest-serving and shipped-release bullets; the quality limitation rewritten around the 0/15 → 15/15 decode canary, #784 distinctness, and the D4 semantic-OOD finding. Stale ask-gating claim corrected.
- CLI/HTTP reference, docs map, requirements, troubleshooting updated.

### Accuracy fixes

- **ROADMAP**: post-close "Next up" (#804 the only open issue; S1 verdict → recompile/quality read toward a server-admissible bundle); Landed gains the #655 epic + v0.1; Capabilities row updated.
- **MODEL_LIFECYCLE**: install-release as the skip-the-pipeline entry point; "#655 still open" note replaced with the closed end-state.
- **RESEARCH**: dated continuation of the load-bearing "which track" section (validity/distinctness decomposition, decode default, D4 finding, #784 gated on S1); route-attention paragraph updated to S1-in-progress.
- **THREAT_MODEL**: release-asset supply-chain row for the new distribution surface.
- **Record correction**: the F0 addendum + rename inventory claimed no THREAT_MODEL.md exists — wrong (it lives at `docs/transformerless/THREAT_MODEL.md`; the grep only covered `docs/` root). Corrected in both docs and on the closed #655 thread (issuecomment-5337018357).
- `src/release_bundle_loader.rs`: stale "no shipped bundle produces a sidecar yet" module doc updated (comment-only).

## Gates

Claim-wording gate clean · fmt clean · lib compiles (one comment-only code change) · all README anchors resolve · all four Mermaid diagrams render-validated to SVG.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_016X39CfJSLCU4KhjNP2XXTW


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

