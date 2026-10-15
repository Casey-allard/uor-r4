# #1697 — Learn exact native crossings and expose causal bank attention

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-04T21:03:28Z
- **Updated:** 2026-10-09T13:17:27Z
- **Closed:** 2026-10-04T21:05:29Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1697
- **Merged:** 2026-10-04T21:05:29Z
- **Merge commit:** f862a04d903558b9c9fde9114e9f6cfded52ad30
- **Base / Head:** main / UOR-Foundation:codex/geometric-native-route-crossing-20261004
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The geometric bridge previously changed packed coefficients without changing native decisions. This adds exact native root-crossing proposals and records32 actual crossings: each improves its local witness, but every full64 native CE worsens, so the accepted parent is retained. The top16 witnesses per arm all concern one Period position; this is a bounded proposal result, not a geometry-family rejection.

The integer realizer also gains a chronological occurrence-bank API with exact causal candidate-position mapping and one global Copy/Period/Stop reduction. A Rust source-only preparer and native-only probe preserve the single-source own-prefix parent, while distractor/order/version rows expose missing bank learning. The roadmap now prioritizes learning actual bank retrieval and terminal decisions with native hard-forward parity, without gold-record selection or per-record normalization.

Validation:6 crossing,4 feedback,4 bank-library and4 preparer release tests pass;3 release examples build. Independent saved crossing audit784,922 checks/0errors, exact returned SHA/file sets, actual native six-row bank generation, cargo fmt --check and claim wording pass. Source82d0d088 was compiled and run on LinuxCPU48/noGPU. Initial Serialize compile failure and all negative candidates retained. No general chat, fullprefix1000+, allocation-free or M1energy qualification. Evidence and complete cumulative costs are linked in current state.

References #1552. References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

