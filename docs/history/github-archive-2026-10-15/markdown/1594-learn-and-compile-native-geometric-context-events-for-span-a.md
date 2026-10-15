# #1594 — Learn and compile native geometric context events for span attention

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-02T02:24:05Z
- **Updated:** 2026-10-09T13:17:24Z
- **Closed:** 2026-10-02T03:07:58Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1594
- **Merged:** 2026-10-02T03:07:58Z
- **Merge commit:** 774c3e71a4d305722eaf997a32057fe8347b9f7b
- **Base / Head:** main / UOR-Foundation:codex/geometric-event-native-20261001
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The ordered geometric span reader still depended on a floating contextual controller for HOLD/OPEN/APPEND/COMMIT. This change learns a two-lane signed-H4 context controller whose compiled execution selects group actions from token/own/neighbor potentials, composes states synchronously, and sends typed integer events directly into the existing span register. It includes a Rust training bridge, source-bound Q24 compiler, strict independent reload, and a fixed evaluator in the same reader. No runtime q/k is added.

Both 320-update seeds completed: all28,976 actual events are correct, all34,688 padded float/native group-action/context/event traces agree, and all512 parent answer decisions and both heads' source-majority flags are retained. Reset causes role-specific capture/source/answer errors. One previous HOLD error on a key becomes APPEND; its span/logits change while its answer stays unchanged. Seed1's existing99/109 answer results and head1's zero correct-source majorities remain. Natural-language capture, geometric superiority, donor reasoning transfer and complete integer serving are unqualified; the surrounding current-code/value/output graph still contains floats. Donor operator recompilation stays in the roadmap.

Validation:12 focused executed checks covering integer arithmetic/synchrony/admission, delayed and ordinary-answer gradient credit, source/compiler/reload/tamper, retained reader and evaluator; formatting/diff/claim-wording checks. Independent source, sealed41-file result and release-assembly review. The428-instruction step/reset bodies have no multiply/divide/float arithmetic or successful-path calls; defensive panic callees are outside that scope. The fixed process took256.69s with110,903,296B maxRSS; this is study cost, not optimized serving performance.

The study and machine receipt preserve source/executable/model/table identities, histories, controls, corrected capture row and remaining work. References #1512.


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5944449434 — 2026-10-02T02:25:09Z — Casey-allard

Independent nonauthor architecture review: APPROVE protected delivery at exact
head9d436269f881dc36c79fc8165bfa2f8e7dc92f89, tree
b77788a3487b239a71edeeef042e03cf2cfdcab3. Reviewer role:
adoption_review_architecture expert agent; this is not a claim of a separate
human GitHub approval. The integer/assembly engineer independently reviewed
the12 executed checks and428-instruction step/reset audit. The principal
mathematician verified the constructive grammar witness and surrogate checks.

No invalidating finding remains. Executed source7b0e7deb differs from the
implementation only by diagnostic JSON serialization; finalhead adds four
intended docs. Raw41-file sealed inventory/hashes, model/source/native bindings,
histories and512 row decisions are verified. The corrected parent HOLD→APPEND
row, distinct compilation drift, seed-specific reset/FSM effects, head1zero
source-majority result, reused development panels and remaining float boundaries
are accurately reported. Donor token/context/output compilation is retained.
No general-language, donor-reasoning, geometric-superiority, complete-serving or
energy claim is approved. #1512 remains open.



---
_Comment/review ids in this file: 5944449434_

