# #246 — Re-port the 16-window zeta design-matrix + QR spectral projection from prime-router (replaces window/eigenvalue stubs)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-07-28T03:52:06Z
- **Updated:** 2026-08-01T21:52:50Z
- **Closed:** 2026-08-01T21:52:50Z
- **Labels:** enhancement
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/246
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

### Summary
The content-bearing spectral projection from the original prime-router was dropped in the Rust port and replaced by stubs:
- No `design_matrix_sparse_r4`, no QR orthonormal bases, no precomputed windows anywhere in Rust.
- `SpectralGeometry::encode` (`geometry.rs:94-116`) picks the scale window by raw L2 norm of 32-dim brain-state slices ×(1+identity bias).
- Eigenvalues are a hardcoded constant vector (`uor-r4-router/src/lib.rs:1388`).
- GloVe semantic blend (first 50 dims in Python) has no Rust code; `src/server.rs:3298` hardcodes `"glove_loaded": false`.

Original implementation to port: prime-router `server.py:606-633` (sparse Dirichlet-series/zeta design matrix + QR per window). `ZETA_ZEROS` (all 512 imaginary parts) is already in `uor-r4-core/src/zeta_zeros.rs`.

### Scope
1. Port the 16-window design-matrix + QR projection into `uor-r4-core` (compiler-side; floats permitted — this never touches the deployed kernel).
2. Replace the L2-norm window stub and the eigenvalue constant with projected values.
3. Decide GloVe's fate in the same pass: re-add a semantic blend, or record zeta-only as the intended design and remove the tombstone field.

### DoD
- Window selection responds to query text (test: distinct queries under identical session state can select different windows — impossible today).
- Eigenvalue output derives from the projection, not a constant.
- Retrieval-quality delta vs the content-reconnection baseline measured on the same fixtures.

*Depends on the content-reconnection issue (#245-area) for measurable value. From the prime-router → uor-r4 lineage audit.*


## Comments (2)

### comment 5147893644 — 2026-07-31T22:10:44Z — auser

Implemented in PR #313: https://github.com/UOR-Foundation/uor-r4/pull/313

Highlights:
- deterministic 16-window sparse zeta design matrices over the pinned 512 zeros
- reduced complex QR bases cached with OnceLock
- query-responsive window selection with identical session state
- sparse ranges and QR coefficient state carried through routing/indexing/retrieval
- eigenvalues derived from six-subwindow covariance rather than hard-coded
- no GloVe dependency reintroduced because the source is unavailable/tombstoned

Focused fixture measurement:
- #245 content-reconnection baseline: top-1 0.167, MRR 0.444
- sparse-QR routed path: top-1 1.000, MRR 1.000
- delta: +0.833 top-1, +0.556 MRR

All local repository gates pass, including the full offline workspace suite and the new quality harness.

### comment 5153614304 — 2026-08-01T21:52:50Z — auser

Closing as complete via PR #313 (merge commit 0c27ab4, on main). DoD verified on origin/main: (1) window selection responds to query text — deterministic content signal blended into session state, regression test included; (2) eigenvalue output derives from the projection — hardcoded constant replaced by six-subwindow covariance eigenvalues; (3) retrieval-quality delta measured on the fixture: top-1 0.167 / MRR 0.444 → 1.000 / 1.000 vs the #245 content-reconnection baseline (docs/zeta_projection_246.md). GloVe fate decided: zeta-only is the intended design, tombstone rationale documented. Verified 2026-08-01: cargo test -p uor-r4-router green (32 passed), including tests/zeta_projection_quality.rs.


---
_Comment/review ids in this file: 5147893644, 5153614304_

