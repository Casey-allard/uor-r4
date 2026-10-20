# #941 — ci(#940): replace the certification gauntlet with a fast product gate

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-25T19:41:25Z
- **Updated:** 2026-10-09T14:16:56Z
- **Closed:** 2026-08-25T19:53:02Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/941
- **Merged:** 2026-08-25T19:53:02Z
- **Merge commit:** c266f63c7c4bbbc0bbbcdf37c34805e9f5fdf392
- **Base / Head:** main / UOR-Foundation:issue-940-lean-ci
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Outcome

Replace the 18–24 minute, five-context required queue with one path-aware `fast build + product smoke` context.

- docs/workflow changes: claim wording only
- Rust/build changes: fmt + workspace compile + library/product-path unit tests
- BDD, doctests, no_std, deterministic rebuild, κ, Gate C, all-features, WASM, fuzz, Kani, conformance, audit, and D2: nightly/manual
- duplicate Gate C execution removed
- full post-merge reruns removed
- contributor, agent, and issue-template guidance now requires focused checks instead of a universal gauntlet

## Baseline

Recent successful merge-group runs took 15–24 minutes. Docs-only PR #939 took 24m23s; its core job spent 9m19s in workspace nextest, 2m19s in κ, and 6m55s in a second Gate C run after the dedicated 7m45s Gate C job had already run.

## Targets

- one required context
- docs-only under 2 minutes
- ordinary warm-cache Rust change under 8 minutes

## Verification

- `git diff --check`
- YAML parse for all three edited YAML files
- `python3 scripts/check_claim_wording.py`
- classifier fixtures: Rust/Cargo/embedded-contract => code; docs/workflow => no-code
- this PR and its speculative merge provide the live timing proof

No runtime/model behavior changes.

Closes #940

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

