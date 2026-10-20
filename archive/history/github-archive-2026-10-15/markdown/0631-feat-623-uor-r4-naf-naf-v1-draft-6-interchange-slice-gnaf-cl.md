# #631 — feat(#623): uor-r4-naf — NAF v1-draft.6 interchange slice + GNAF claim vocabulary

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-12T01:06:29Z
- **Updated:** 2026-10-09T14:17:06Z
- **Closed:** 2026-08-12T01:25:41Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/631
- **Merged:** 2026-08-12T01:25:41Z
- **Merge commit:** 2bfd360a27835247bb6788383abeed8441d17095
- **Base / Head:** main / UOR-Foundation:i623-naf-slice
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

feat(#623): uor-r4-naf — UOR-NAF v1-draft.6 interchange slice + GNAF vocabulary

New non-shipped crate implementing the pre-registered #623 adoption
slice: capabilities core, integer, tensor, address, address-binding
(integer|tensor). Arbitrary-precision NAF normalization (§3 laws),
strict uvar/CoreNAFBytes/signed grammars, semantic+artifact payloads,
§8.2 sha256 manifest chains, full artifact decode with embedded-κ
verification, and the 6-state outcome taxonomy (NAF-DOM-006) with
resource-refusal and policy-out-of-domain kept distinct. Golden vectors
(§12.2 integer-3 and tensor chains, byte-for-byte incl. all four κ) and
the §12.4 mandatory rejection corpus run as tests. Dual-label rule is
enforced in types: NafLabel is sha256-only and unconstructible from
strings — r4's blake3 store keys cannot flow in. claims module adopts
GNAF §12.4/§12.5 vocabulary (13 claim classes; dual status) as labels
only; wiring into certify output stays with #515 governance. Atlas
words, Adapter C domains, naf-plan, execution/optimality: deferred on
#623 with reasons. All NAF κ are provisional until the spec freezes.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

