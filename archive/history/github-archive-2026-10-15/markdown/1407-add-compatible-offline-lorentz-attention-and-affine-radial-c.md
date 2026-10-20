# #1407 — Add compatible offline Lorentz attention and affine radial control

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-27T01:30:54Z
- **Updated:** 2026-10-09T13:17:49Z
- **Closed:** 2026-09-27T01:45:39Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1407
- **Merged:** 2026-09-27T01:45:39Z
- **Merge commit:** e3191a835fd70fb1570f7c8e5fdc59fe4610a200
- **Base / Head:** main / UOR-Foundation:codex/radial-read-control-20260926
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The native Lorentz reader combines radial query/key compatibility with a nonlinear distance-to-score law. Add an offline `LorentzAffine` control on the same radial features so a subsequent paired language study can test whether that nonlinear weighting helps.

The control shares the native reader maps, two learned scalars, clamp, age terms, NoRead, values and copy/output paths. Its fixed tangent matches Lorentz distance and slope at the initialization reference; it does not claim equal initial entropy or gradients everywhere. It has a distinct serialized operator identity. Existing Dot and Lorentz numerical contracts and serving behavior are preserved, including the Lorentz implementation merged from #1401 during development. Only the new affine variant is refused at unsupported quantization/export/integer boundaries. Existing `joint-fit` campaign JSON can select the continuous control.

Validation on committed source `48fd43c16cf2596e6379a5869500b19d43571731` with that exact build-source identity:
- Four `radial_read_` checks and two retained `calibrated_shadow_` checks pass on the same CPU executable. They exercise arithmetic/gradients, both reader paths, causality, exact retained identities, checkpoint round trips and unsupported affine serving.
- Independent mathematical and adversarial source reviews found no remaining blocker against main `ab90b9e5`.
- Touched Rust formatting, diff and claim-wording checks pass.
- Final compile/check commands took 79.98 and 11.10 seconds, with peak command RSS 2.24 GB. All prior attempts and cumulative elapsed accounting are retained in `docs/evidence/offline-radial-read-validation-2026-09-27.json`. An earlier source-provenance failure was resolved by the source-bound build, without changing the model or weakening a check.

The research notes reconcile Claude's subsequent bug fixes and shared native packet. No paired training study, capability evaluation, energy result or new serving-policy decision is included. The next comparison must preserve data/tokenizer/configuration identities and evaluate actual generated behavior.

References #973. References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

