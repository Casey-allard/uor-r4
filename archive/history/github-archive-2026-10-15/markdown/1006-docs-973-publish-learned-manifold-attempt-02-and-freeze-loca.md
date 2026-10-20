# #1006 — docs(#973): publish learned-manifold Attempt 02 and freeze localization

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-30T04:11:09Z
- **Updated:** 2026-10-09T14:17:20Z
- **Closed:** 2026-08-30T04:12:30Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1006
- **Merged:** 2026-08-30T04:12:30Z
- **Merge commit:** dda9c37db8e8ff9778dbe464c814a10c9d55c6e0
- **Base / Head:** main / UOR-Foundation:issue-973-helm-d-attempt-02-result
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Outcome

Publishes the exact `HelmDLearnedManifoldR4ConstructionV2` Attempt 02 result and fitted pre-validation checkpoint from protected `main` revision `c6b86b8f8dc5ea9e4e2c5567ae7d95ffecd8de73`.

Terminal: `FAIL_HELM_D_MANIFOLD_CONSTRUCTION_REVISE_PROJECTION_SCORE_CENTROID_OR_TRAINING`

Result self-CID: `blake3:9144913380c6ebdeebb5848138bc8e6642c1e7020d8e7a097aa3cd73cb829020`

Checkpoint self-CID: `blake3:dd04bfc1cf15e5dd2c6c8be5afa363ecb452386f54632cd120bce56018444789`

This is a valid non-D3 construction-validation negative, not `UNAVAILABLE`. Donor/gauge parity, deterministic replay, exact causal work, and every destructive-control separation passed. Learned Lorentz failed donor retention and matched Euclidean parity; control separation establishes intervention sensitivity only.

## Direction

- Keeps `HELM-D-R4` as the qualified bounded ordinary causal-softmax reference in coherent R4/Spin frames.
- Keeps D3, resonance, recurrence, E8 expansion, lowering, scale, and #954 blocked.
- Makes `docs/r4_intelligence_completion_plan.md` the post-v0.1 sequencing authority and synchronizes all living/status documentation.
- Freezes the next bounded action as `HelmDScoreCentroidLocalizationR4V1`: an 8/8 Lorentz/Euclidean score by normalized-Lorentz/tangent-readout construction audit with a two-document fail-fast gate.
- Records that the prior 740-second canary omitted the 14 full-decoder validation/replay passes; observed total was 9,330 seconds.

## Verification

- `cargo test --workspace --offline --no-fail-fast`
- `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings`
- `cargo fmt --check`
- both graph-format `no_std` checks
- WASM library check
- `python3 scripts/check_claim_wording.py`
- independent result/checkpoint/provenance/causality audit

References #973.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

