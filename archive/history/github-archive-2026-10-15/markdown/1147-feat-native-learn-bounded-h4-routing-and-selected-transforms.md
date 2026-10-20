# #1147 — feat(native): learn bounded H4 routing and selected transforms

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-06T03:21:33Z
- **Updated:** 2026-10-09T13:17:41Z
- **Closed:** 2026-09-06T03:47:42Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1147
- **Merged:** 2026-09-06T03:47:42Z
- **Merge commit:** 201d9f9159920444aed438e3a695240b4bb9d417
- **Base / Head:** main / UOR-Foundation:codex/native-learned-routing
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The native predictor used fixed lexical placement and lacked a learned selected-payload block. This adds an optional two-channel H4 residual: learned query/key/value codes select up to two recent payloads, execute query-conditioned table transports and contribute sparse token scores. Rust coordinate fitting runs the same hard selector and transport as serving. Existing artifacts retain their identities; the new block binds its exact parent and preserves nested provenance validation.

The result remains development only. On 735 authored OPEN prose/Rust next-token positions, parent scores 45 correct, learned angular 182, matched exact-code selection 223 and fixed placement 177. These are substantially byte-level synthetic results. All eight target continuations fail. Both fitted selectors preserve only 38/62 older exact responses; the unchanged parent reproduces all 62 complete generation records. Both preserve all 28 relation writes, with 16/28 angular and 28/28 exact-code answers. Retain accepted parent `067adbf0`; no angular superiority or capability promotion is claimed.

The observed successor is complete response dispatch and stopping trained with routed prediction against final output, including ordinary language and preserved memory behavior. The older entry head can force `Unknown`; bypassing it removes that prefix but does not yield coherent text. Independent residual scores also disrupt EOS. The roadmap and current-state pointer record that next step, keeping #1139 open and #1140 subsequent.

Validation: changed release CLI and both examples built offline; four focused routing/provenance/reload/session tests passed; the changed prediction path has zero measured heap allocations and passes the integer-kernel source scan. Architecture, formatting and claim-wording checks pass. The actual CLI matches the probe and exercises the entry-disabled comparison. New generated Rust did not meet its target and is not claimed as compiled/executed semantic evidence. An initial parent-binding reload failure is preserved and charged; the corrected comparison retains identical learned parameters.

Local resource use: 46.768 seconds model work this cycle, cumulative 1,410.652/1,800 seconds, leaving 389.348. The complete build/evaluation projection preceded execution. No extra storage allocation, deletion, external model or paid compute was needed. Full outputs, work counters, costs and artifact identities are linked from `docs/native_geometric_learned_routing_1139.md` and its compact evidence.

Refs #1139. This implements the first block; the full issue handoff remains unmet.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

