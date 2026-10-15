# #1003 — Record intrinsic Lorentz R4 construction result (#973)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-29T20:17:41Z
- **Updated:** 2026-10-09T14:16:58Z
- **Closed:** 2026-08-29T21:56:53Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1003
- **Merged:** 2026-08-29T21:56:53Z
- **Merge commit:** 133e72b054ba174ab2f12b058c588852d159f232
- **Base / Head:** main / UOR-Foundation:issue-973-intrinsic-r4-attention
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #973

This PR freezes, executes, and records the bounded intrinsic Lorentz R4 ordinary-softmax qualification.

Established predecessor:
- `HELM-D-R4` remains `PASS` for bounded ordinary full-prefix causal softmax in coherent transported R4/Spin frames.
- That is preliminary ordinary attention evidence, not curvature-specific advantage or transformerless serving.

Attempt 01:
- producer commit `c10d44ed2ad180562bfd8c88102bbf162bece217`
- `UNAVAILABLE_INTRINSIC_LORENTZ_R4_STOP_BEFORE_HELD_OUT` at checkpoint readback
- D3 remained sealed; no attention verdict

Attempt 02:
- implementation commit `a348d7ec31f75524906a39cc04327bbcbdd47a56`
- terminal `UNAVAILABLE_INTRINSIC_LORENTZ_R4_STOP_BEFORE_HELD_OUT`
- embedded result CID `blake3:da2a63323d6211b8d581e5a4ed75d788eb919ff0f210d2e3beb8a749ee1bc64f`
- canonical result file BLAKE3 `blake3:3f8ba48d9830eca0636df37eafa8af167475005970e77f5af6f3bec20d191518`
- fit checkpoint CID `blake3:0372ae31b6464c4967c07b70f7d4bd3cea437c971c34e16ab1b8048630144dc2`
- barycenter covariance `9.121400701417315e-8` versus frozen `1e-8`
- construction-validation NLL: donor `2.6513639557648623`, curved `3.9044978436394797`, flat `3.6955705255518287`
- exact parameter/report replay, work/shape, donor trace, liveness, zero faults, and zero future reads passed
- `heldout: null`; no D3 reveal marker; no held-out decode

Claim boundary:
- this is not a held-out intrinsic-attention negative because the frozen evidence audit failed
- intrinsic curvature-specific attention remains unestablished
- multi-resonance, recurrence, exact lowering, #954, correctness, and reasoning remain blocked
- no Attempt 03 is authorized under the V1 freeze

Next bounded seam:
- one fresh construction-only, source-faithful HELM-D learned-manifold Q/K/V qualifier
- declared Lorentz-inner-product score, learned scale/bias, stable equivariant centroid
- equal-capacity Euclidean arm and fresh non-D3 validation before any reveal

Evidence publication commit: `cde26e395f7e82932ecd41770a595f8d5ed1715d`.

Verification:
- complete result committed byte-for-byte and BLAKE3 verified
- compact summary values matched the canonical result
- D3 marker absent
- claim-wording gate PASS
- JSON parse and staged diff integrity PASS
- pre-result code gates on `a348d7ec`: full workspace tests, clippy with warnings denied, fmt, no_std, wasm, focused harness, and independent SHIP audit PASS

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

