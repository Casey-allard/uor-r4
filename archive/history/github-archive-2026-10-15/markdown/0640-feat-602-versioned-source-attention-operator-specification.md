# #640 — feat(#602): versioned source attention operator specification

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-12T12:30:58Z
- **Updated:** 2026-10-09T14:17:07Z
- **Closed:** 2026-08-12T12:50:00Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/640
- **Merged:** 2026-08-12T12:50:00Z
- **Merge commit:** 960cd0730611aacb8256e4ba73b5938931e7f82e
- **Base / Head:** main / UOR-Foundation:i602-operator-spec
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

feat(#602): versioned source attention operator specification

Truthful operator inventory made typed and versioned: AttentionOperatorSpec
(new attention.rs beside geometry.rs) with canonical bytes + declared-
identity digest. Two registered operators: standard-source-attention/1
(scaled dot product, full head width) and experimental-r4-source-
attention/1 - the r4_attention branch recorded as it actually computes:
chunked-4-wide dot product, softmax IDENTICAL to standard (max-subtract
exp sum-normalize - not a bypass), trailing H mod 4 q/k dims truncated
from scores (perturbation-tested), scale still sqrt(full H). Stale
Spin(4)/softmax-free wording corrected in README (false 250.000000aster
claim removed), CLI help, doc comments; dated correction appended to
the deferral record (history retained); non-normative cross-ref in
INFERENCE_OPERATION_CONTRACT.md (contract untouched). Registry refuses
unknown (id,version) via sanctioned surface (R5). Boolean switch maps
to operator ids at the oracle (TeacherOracle default None, legacy
interpretation documented). Identity carried on the #597/#600/#601
seams: ObservationManifest auto-populated, CoverReport via
--attention-operator flag, api stage_b + server bind naturally;
certify-side gap recorded as honesty item (no oracle seam there,
consistent with prior fields). Standard + experimental per-head
attention factored into free deterministic functions, arithmetic
untouched (bit-exact forward_batch parity green). 13 new tests incl.
non-divisible head widths. No dormant activation; #515 gates untouched.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

