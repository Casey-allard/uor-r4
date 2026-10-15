# #281 — Implement the #244 decision: single-key store + read-time query-beam in the runtime

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-07-29T14:55:28Z
- **Updated:** 2026-07-29T17:53:07Z
- **Closed:** 2026-07-29T17:53:07Z
- **Labels:** enhancement
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/281
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Decision being implemented

The #244 matrix (complete, reproduced bit-for-bit ×3) measured that moving the membership fan-out from write time to read time preserves accuracy exactly at the printed precision — A-single+query-beam 28.2% top1 / 31.9% agreement, identical to A-binary — at **134,733 vs 344,604 keys (2.56× smaller store)**. Measured price: +0.087 WB bits/token and up to TOP_M lookups+merge per level per query (compares + table reads only; mul-free contract §4 unaffected).

## Scope

- `runtime::build_store`: write evidence under the primary code prefix only (drop `add_evidence_multi`'s fan-out on the write side).
- `runtime::predict` (and the graded probe path): at each level, merge the distributions under the query's TOP_M membership prefixes — reference semantics are the certifier's `eval_query_beam`/`merged_beam_distribution` (added to certify in the #244 measurement PR).
- Store container (TLS1) unchanged; κ-pinning discipline carries over. New store κ must be recorded in the era notes (key population changes by construction).
- Serving equivalence check: cascade behavior (#248) unchanged — same abstain semantics, evidence identical up to write/read reorganization.

## Definition of done

Certify on the D3 artifacts reports the runtime path (A row) at the query-beam numbers: top1/agreement within 0.1pp of 28.2/31.9 at ~134,733 keys; equality witness and op census (multiply=0) intact; all 4 local gates + queue green.

Refs: #244 (decision + matrix), #243 (Phase A baseline uses these rows), #248.

## Comments (1)

### comment 5120254871 — 2026-07-29T15:53:24Z — Casey-allard

## Implementation plan (assigned; branch `issue-281-single-key-store`)

**Write side.** `runtime::build_store` writes evidence under the primary code prefix only (`add_evidence`), dropping the `add_evidence_multi` fan-out from the shipped path. `add_evidence_multi` is retained for the certifier's ablation row (below). Store container (TLS1) unchanged; key population changes by construction → new store κ, recorded in the era notes, compiled bundles regenerated.

**Read side.** New beam lookup shared by all four predict variants (`predict_witness_plain`, `predict_witness_plain_with_priors`, `Runtime::predict_witness`, `predict_witness_with_priors`): at each depth d (deepest first), merge the token→count distributions under the query's TOP_M membership prefixes at d (reference semantics: certifier's `eval_query_beam`/`merged_beam_distribution` from 1f5088b), then apply the variant's existing argmax/prior/repetition-penalty logic over the merged distribution. Memberships come from `assign_memberships_plain(...).1` — already computed during assignment; API change is passing `by_depth` into predict (explicit parameter, no hidden state). Kernel counting: merges are adds + table reads; counted via the existing OpKernel hooks — the census growth **is** the measured read-cost of the #244 decision and will be reported, with the mul-free contract (§4) unchanged.

**Callers.** transformerless serving tier (`src/tless_uor.rs` generation loop), graph-cli store builders (lib.rs:966/1702/1793 — flow through unchanged), scenarios/cover_sweep fixtures, and the certify equality witness (kernel-vs-plain now witnesses the beam path on both sides).

**Certify matrix continuity.** After this change the shipped-runtime row *is* single-key+query-beam, so rows are relabeled without losing comparability: "A (shipped: single-key store + query-beam)" [expected 28.2/31.9 at ~134,733 keys], "A-multi (ablation: write-time fan-out)" [the old A-binary, via retained `add_evidence_multi`], "A-single (ablation: no beam)" [27.0/30.5]. A-f32/B/C rows unchanged.

**Definition of done** (from issue): shipped A row within 0.1pp of 28.2%/31.9% at ~134,733 keys; equality witness 512/512 on the beam path; op census multiply=0; store κ delta recorded; all 4 local gates + merge queue green. The implementation-independent reproduction of the target numbers is on record (PR #282 comment).

**Sequencing note:** implemented directly against main (post-#282); no interaction with #280 (graph scaffold) or #278 (C-stage perf).


---
_Comment/review ids in this file: 5120254871_

