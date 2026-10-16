# Source and host metadata validation

This is **NOT_RUN for GPU/model controls**. No CUDA context, model/reference
evaluator, pod or model compute was used for these checks. Rust1.97.1 compiled
the standalone instrument without the CUDA feature.

| Check | Observed result |
|---|---|
| Standalone host cargo check | PASS; existing dependency dead-code warnings remain |
| Six focused controls metadata tests | 6 passed, 0 failed |
| Full standalone host suite after integration | 36 passed, 0 failed, 1 existing ignored |
| rustfmt and diff whitespace | PASS |
| Claim wording | PASS |
| Template/plan | status NOT_RUN; model_compute_executed false; fixed40/128/16 proposal/endpoint/admission inventory |
| Metadata identity command | No context/model work; compiled kernel identity UNAVAILABLE_WITHOUT_CUDA_BUILD |
| Independent embedded-source hash recomputation | Matches source-template and metadata identity |

The focused tests cover refusing-template execution, seed/inventory/unknown
field rejection, status8 loss/trace completeness, typed admission versus CUDA
failure, standalone source identity stability, and no host execution fallback.
They inspect metadata/receipts; none demonstrates CUDA arithmetic parity.
The ignored existing checkpoint test needs archived exact bytes and performs
metadata hashing only; it was not newly ignored for this change.

Final framed standalone source SHA256:
`871639c3c9f38acaf617a84fc218f9bdf263e34f3d1fef58a06c3ffefcc6892b`.
Fixed fixture inventory SHA256:
`53d61d6c6e0942a274baa907df8a4a8e08a207290b6812fce35234807331c465`.
The [pin template](pin-template.json) deliberately leaves the future CUDA
binary/source/PTX and authority fields unfilled. Its hash is source preparation,
not a completed execution pin. The source hash inventory and framing are
defined in `counterfactual_controls::standalone_source_hash`.

An author-design helper implemented the standalone module. A separate
adversarial reviewer inspected the fixture/control design, caught the inert
context3 Route0 fixture, and requested virtual-ineligible early-break coverage
and endpoint input hashes. The parent integrated decoded-lane serialization
with raw words retained and repeated the full host suite. Final committed-head
source review and TEST FITNESS are separate pending receipts, not inferred
from these preliminary checks.

The production device/generator/CUDA files and existing diagnostic module are
unchanged in this source task. CUDA-only code still needs real exact-head
check and strict-clippy/all-targets construction transcripts. No finite GPU
result, runtime resource result or execution release follows from host checks.
