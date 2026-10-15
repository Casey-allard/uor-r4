# #306 — Remediation: replace the sign-restricted, magnitude-only block-norm projection feeding Hopf sector addressing

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-07-31T05:42:58Z
- **Updated:** 2026-08-05T04:57:15Z
- **Closed:** 2026-08-01T21:33:04Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/306
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Filed per the #303 Definition of Done: the measurement confirmed under-addressing, so remediation is tracked separately (none landed under #303).

## Evidence (from #303 / docs/hopf_sector_occupancy_d3.md)

Measured over the D3 held-out fixtures (596 articles, 1500 routing samples, production evolve-then-route sequence, pinned corpus CID blake3:194db0ee…0132cf):

- **7 of 512 sectors occupied (1.4%)** — against a static reachable bound of 64/512 (sign restriction: block L2 norms are non-negative, confining the 4-vector to the positive orthant of S³).
- Empirical ranges vs. binning that assumes [0, 1]: u_delta ∈ [0.478, 0.503], u_alpha ∈ [0.617, 0.626], chi_u ∈ [0.480, 0.542].
- The projection is magnitude-only: per-block norms are near-constant (relative dispersion 1.3–2.5%) and correlate only −0.36…−0.72 with a fixed signed projection — directional structure within each 128-dim block never reaches the router.

## Scope (to be refined when picked up)

Replace or augment `get_state_4d_projection` (`crates/uor-r4-router/src/lib.rs:1206`) so the 4-vector fed to `assign_sector_hopf_transport_scalar` is (a) not sign-restricted by construction and (b) carries within-block directional information. Candidate directions from #303: signed projections onto fixed unit vectors per block, or first principal direction per block. Constraint: any replacement must be re-measured with the #303 harness before and after, and must not presume the Design P (#276) coordinate scheme — if #276 lands first, this issue applies to its input projection instead.

## Links

- Measurement: #303, docs/hopf_sector_occupancy_d3.md (PR #305)
- Interaction with the addressing redesign: #276 (docs/phase_clock_address_design.md)


## Comments (3)

### comment 5147749864 — 2026-07-31T21:47:34Z — auser

Implemented in PR #312: https://github.com/UOR-Foundation/uor-r4/pull/312

The production Hopf projection now preserves signed within-block direction via deterministic Blake3-derived probes. Under the identical #303 D3 protocol, observed occupancy increased from 7/512 (1.4%) to 43/512 (8.4%); u_delta and u_alpha now span the full signed ranges observed in the run. This is an addressing remediation measurement, not a claim of serving-quality improvement or full 512-sector reachability.

### comment 5153542545 — 2026-08-01T21:33:03Z — auser

Closing: remediation landed and re-measured under the identical #303 D3 protocol — signed within-block direction via deterministic Blake3-derived probes, occupancy 7/512 (1.4%) → 43/512 (8.4%), u_delta/u_alpha spanning full signed ranges (evidence in the 2026-07-31 comment above). Merged in PR #321 (merge commit 60e5917b2c).

### comment 5187725085 — 2026-08-05T04:57:15Z — Casey-allard

The deferred retrieval/collision measurement ran (#422, PR #430): the signed projection's occupancy gain does NOT translate to retrieval value — it measures NEGATIVE vs the pre-remediation projection on sector-filtered MRR (0.0045 vs 0.0743). See #422 for the record and the redesign scope.


---
_Comment/review ids in this file: 5147749864, 5153542545, 5187725085_

