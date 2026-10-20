# #476 — perf(core,certify): #469 lever B — prepared dot tables for the corpus code passes (1.60x)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-07T07:58:25Z
- **Updated:** 2026-08-07T08:29:41Z
- **Closed:** 2026-08-07T08:24:07Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/476
- **Merged:** 2026-08-07T08:24:07Z
- **Merge commit:** b90abe96d39c008b45292745c90d138ad5c16911
- **Base / Head:** main / UOR-Foundation:perf/469-lever-b
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Closes #469. Lever A (κ-keyed code sidecar) shipped in #470; this is lever B.

## Why the kernel could not just be called

`simd::dot_argmax` already existed and `Runtime` already used it. The free-function assign path — what every corpus-scale code pass actually calls — did not, and **could not**: `DotTables::from_packed` is a *load-time* expansion of the packed u16 tables, and a free function has nowhere to cache it.

Measured on the pinned TLA7 artifact:

| | us |
|---|---:|
| `DotTables::from_packed` (one build) | 2,337 |
| one whole scalar assignment | ~487 |

So wiring the kernel per bundle would have been roughly a **4x regression**. The kernel is only a win once the decode is amortized — which is why the API is a batch one.

## What landed

`runtime::AssignTables` makes the amortization explicit: build once per artifact, share across rayon workers, call `assign_code_for_bundle_with` / `code_plain_with` per bundle.

| path | us/call |
|---|---:|
| `assign_for_bundle` (today) | 496.3 |
| `assign_code_for_bundle` (scalar argmax) | 486.5 |
| **prepared + kernel (lever B)** | **310.6** |

**1.60x**, and the one-time build pays for itself after 7.5 calls against the 500,000 in a corpus pass. Pinned TLA7 artifact fixture, 500k-record corpus, release profile, x86_64/AVX2.

Wired: the Gate C left-code pass (on a sidecar miss — the cold run worth speeding up) and `derive_right_codes` (uncached, so *every* scored run; the consumer #471 names as dominant).

## Bit-identity is proven, not argued

The path is κ-pinned, and an assignment that is faster and *almost* identical silently invalidates every pinned measurement. Equality rests on three facts, each machine-checked: same addends (exact `i64` sums of the same decoded terms; integer addition is associative even on wrapping overflow, so accumulation order cannot matter), same tie rule (ascending class scan, strict improvement, lowest index wins), same stage arity.

`tests/assign_prepared.rs` checks prepared == scalar over:
- both dot artifact shapes (TLA6 plain, TLA7 residual)
- the sign-metric fallback (no dot tables → scalar, no third outcome)
- short `dot_cb` stage arity
- **512 real corpus positions on the committed TLA7 artifact fixture**

That fixture arm **fails rather than skips** when its inputs are missing. The κ test compiles a fresh artifact and skips wherever the pinned checkpoint is absent — which is most machines — and a κ-pinned change cannot rest on a gate that silently does not run. That is the #354 vacuous-green failure mode, and this change would have been exposed to it. `kappa_reproduction.rs` additionally witnesses the prepared path on a fresh compile where the checkpoint exists.

## A negative, recorded because it redirected the work

`assign_for_bundle` builds the full membership beam and discards everything but the primary code. That looked like the obvious waste, and removing it measured **0.95x — nothing**. The `K x D` scan (4 x 256 x 288 entries) dominates so completely that the beam sits inside the noise. Cheap to test; expensive to have assumed.

The design doc's existing non-goal still holds: TLA7 residual *subtraction* remains unvectorized. The residual path gains here because its per-stage scan uses the adapter, not because the subtraction changed.

## Gates

`cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `check_claim_wording.py`, `cargo check --target wasm32-unknown-unknown -p uor-r4-wasm-router --lib`, full `uor-r4-core` and `uor-r4-graph-certify` suites — all green.

Kernel dispatch is chosen at runtime (AVX2 / NEON / scalar), so a single run witnesses one dispatch. The x86_64/AVX2 path is witnessed above; the aarch64/NEON witness is posted in a comment below.

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5214188845 — 2026-08-07T07:59:12Z — Casey-allard

**aarch64 / NEON witness** — run on the development Mac (`uname -m` = arm64), debug profile, this branch at bd4d6df:

```
running 6 tests
arch = aarch64, prepared dot tables decoded; kernel dispatch is chosen at runtime
test prepared_reports_vectorization ... ok
test sign_metric_artifacts_fall_back_to_scalar ... ok
test short_dot_table_list_leaves_the_same_trailing_stages ... ok
test beam_primary_equals_prepared_code ... ok
test prepared_matches_scalar_on_dot_and_residual_artifacts ... ok
pinned artifact (TLA7 residual): 512 corpus positions, 486 distinct codes, all forms agree
test prepared_matches_scalar_on_the_pinned_artifact_fixture ... ok

test result: ok. 6 passed; 0 failed
```

Both dispatches are now witnessed on the same commit: AVX2 in the PR body, NEON here. That matters because `simd::dot_argmax` selects its kernel from runtime CPU features, so a green CI run alone would only have covered whichever path the runner took — and the two kernels are separate implementations (`dot_argmax_avx2_single_term` vs `dot_argmax_neon_single_term`), not one implementation behind a portable abstraction. A tie-break or accumulation difference could have existed in one and not the other.

486 distinct codes across 512 sampled positions on the pinned TLA7 artifact, so the comparison is exercising the scan rather than agreeing on a constant.


---
_Comment/review ids in this file: 5214188845_

