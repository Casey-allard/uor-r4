# #642 — feat(#604): R4RouteAttentionV1 — reference semantics + packed lowering (dormant)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-12T16:29:35Z
- **Updated:** 2026-10-09T14:17:07Z
- **Closed:** 2026-08-12T16:49:57Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/642
- **Merged:** 2026-08-12T16:49:57Z
- **Merge commit:** 518b8a78abd5cd353bfb04eaf5af4405777c68d4
- **Base / Head:** main / UOR-Foundation:i604-route-attention
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

feat(#604): R4RouteAttentionV1 - reference semantics + packed lowering (dormant)

The first R4-native target operator. Reference (certify-side, scalar,
integer-only): 288-bit route codes (the signature width, justified from
compiler.rs D=288 and ROUT prototype/mask windows), masked XOR +
table-popcount relation, bounded top-M (M<=8, N<=64) with lowest-index
ties, ScoreQ saturating aggregation in pinned selection order, explicit
caps refused via the sanctioned graph-substrate error. Packed R4G1
lowering lives IN a P-4-scanned file (added to the scan list, passes
scan_for_forbidden_arith): borrowed byte views, caller-owned RouteState
(epoch pattern), shift/add addressing, allocation-free steady state
(census-asserted, packed step x8 = 0 allocations). Witness
uor-r4-route-attention-witness/1 with independent replayer verifying
digests, per-slot distances, strict selection order, completeness-by-
optimality, aggregate refold, and closed-form census - 12 tamper
classes rejected by name. Reference-vs-packed bit-identical on the
pinned fixture. Registered as r4-route-attention/1 in the #602 registry
(no QKV reuse; deployed-integer operation class). Nothing enters R4G1
artifacts; packed-routing-dormant NOT enabled; serving default
untouched; historical fixtures byte-identical (deterministic rebuild
--include-ignored green). Ledger r4-route-attention-dormant carries the
predeclared activation record (first verdict = correctness; quality
only via the fitting issue). Docs in MODEL_LIFECYCLE + contract
cross-ref. Seven gates green (128 suites). No new deps.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

