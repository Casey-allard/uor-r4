# Hopf Projection Remediation (Issue #306)

- **Date:** 2026-07-31
- **Baseline:** [Issue #303 D3 measurement](hopf_sector_occupancy_d3.md)
- **Harness:** `crates/uor-r4-router/tests/hopf_sector_occupancy.rs`
- **Claim language:** the results below are **Empirical Criteria** on the pinned
  D3 fixtures, not a proof of general addressing capacity.

## Change

`get_state_4d_projection` previously reduced each 128-dimensional block to an
L2 norm. This made every Hopf input component non-negative and discarded the
within-block direction. The replacement projects each block onto a fixed,
domain-separated Blake3-derived signed unit direction, then L2-normalizes the
four signed components.

The direction is deterministic, content-independent, and unchanged across
runs. It is part of the exploratory floating-point router path; it does not
alter the deployed integer inference kernel.

## Same-protocol measurement

Command:

```bash
cargo test -p uor-r4-router --release --offline \
  --test hopf_sector_occupancy -- --ignored --nocapture
```

The run used the same 596 held-out articles, 1,500 routing samples, article
identities, chunk size, gamma, sector budget, and production evolve-then-route
sequence as #303.

| measurement | #303 magnitude-only baseline | #306 signed projection |
|---|---:|---:|
| distinct sectors | 7 / 512 (1.4%) | 43 / 512 (8.4%) |
| `chi_u` range | [0.4799, 0.5419] | [0.0001, 0.4741] |
| `u_delta` range | [0.4783, 0.5029] | [0.0001, 1.0000] |
| `u_alpha` range | [0.6169, 0.6260] | [0.1406, 0.8031] |

The result removes the sign-restricted reachable region and increases observed
occupancy by 6.1× on this corpus/protocol. It does not establish that all 512
sectors are useful or reachable; retrieval quality and collision behavior need
separate measurement before this projection can be treated as an improvement
to serving quality.

## Regression coverage

The router unit suite verifies that a block aligned with the signed probe
produces a positive projected component and that negating the same block
produces a negative component. Formatting, the targeted router tests, and the
ignored D3 measurement pass.
