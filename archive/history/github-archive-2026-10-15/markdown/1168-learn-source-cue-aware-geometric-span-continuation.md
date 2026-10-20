# #1168 — Learn source-cue-aware geometric span continuation

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-07T12:57:30Z
- **Updated:** 2026-10-09T13:17:59Z
- **Closed:** 2026-09-07T12:58:38Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1168
- **Merged:** 2026-09-07T12:58:38Z
- **Merge commit:** f0556bb7cf5e6da355157443739c00769e297195
- **Base / Head:** main / UOR-Foundation:codex/source-span-context
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The separator-only span operator copied `Rome holds ada` instead of `Rome`. This change gives the existing signed-H4 Continue/Finish learner an exact next-word/original-source-cue prime pair, so it can stop before relational `holds` while retaining `holds` inside an object value. A small construction-only registry reuses the existing Rust word scanner and prime constructor; all parent parameters and initial source selection stay fixed.

Retain artifact `419ba3a7` at bounded source-span scope: construction 5/14 → 14/14, open 2/6 → 6/6, fresh 3/9 → 9/9. Removing the cue pair gives 11/14, 4/6 and 6/9. All 663 retained construction answers, earlier 24/24 cases and other output/write/session preservation pass. Unseen connectors and double-space gaps remain 0/2; this does not qualify general phrase understanding. The separator-only negative and active parent remain preserved.

Validation: public CLI and release Rust probe/library tests built; seven focused state/kernel tests and architecture policy passed; actual artifact parent restoration, malformed registry/code rejection, causal commitment, interrupted copying, every-byte checkpoints and zero allocations passed. README, current state, roadmap, research and lifecycle now point to the result and the next multiword-memory step. The rebuilt CLI loads the artifact and returns the exact three-word value with EOS. Model work is 73.059 seconds; all failed/stopped builds are charged, including a storage-guard stop recovered through named recreatable compiler-cache cleanup inside unchanged caps. Remote required statuses are compatibility acknowledgements, not model-test evidence.

Refs #1139, #1140. Evidence: `docs/evidence/native_geometric_span_context_1139.json`; result: `docs/native_geometric_span_context_1139.md`.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

