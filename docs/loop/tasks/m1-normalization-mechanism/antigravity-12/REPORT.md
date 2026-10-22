# REPORT: m1-normalization-mechanism, antigravity-12

Lab: antigravity. Item: [m1-normalization-mechanism](../../../../roadmap/items/m1-normalization-mechanism.md). Order: [ORDER.md](ORDER.md).

References #2842.

## What did not work, first

- **Un-normalized multi-stage residual accumulation (`UnnormalizedControlV1`):** Across $S = 16$ sequential residual stages ($x^{(s)} = x^{(s-1)} + \rho_{r_s}$), block $L_1$ norms drift linearly to $\|x^{(S)}\|_1 = 68$ and $\|x^{(S)}\|_\infty = 34$, pushing endpoint cell-admission distances (`endpoint_address_distance`) above the canonical 5-value range $\{0, 4, 8, 12, 16\}$ (`multistage_residual_accumulation_vs_unnormalized_drift`).
- **Un-scaled `snap_1` on multi-term sums (`UnitSnapControlV1`):** Comparing a 4-term sum $3\rho_{r_0} + \rho_{r_{\text{orth}}}$ (3 votes for root `0`, 1 vote for an orthogonal root $r_{\text{orth}}$ with disjoint support) against un-scaled reference roots $1 \cdot \rho_r$ ties on $L_1$ distance ($12 = 12$) and selects the 1-vote minority root $r_{\text{orth}}$ by Hamming distance ($2 < 4$), whereas multiplicity-matched `snap_4` ($n = 4$) strictly selects the 3-vote majority root `0` ($L_1 = 8 < 24$, `scale_equivariance_on_orbit_multiples`).
- **GPU trunk training (`N_snap` vs. `C_unscaled` and `C_raw`):** Not run in this CPU-scoped unit; the Stage 3 trunk (`m1-trunk-assembly-and-training`) is not yet assembled and GPU runs require a non-author `TEST FITNESS: FIT` review of `docs/harness/prep/m1-normalization-mechanism-DRAFT-2026-10-22.md`, owner confirmation of the `#2028` bar decision card, and a typed D40 owner release.

## What changed

- **Design note & bar decision card:** Posted the `#2842` design note ([`#2842` comment 6338588907](https://github.com/UOR-Foundation/uor-r4/issues/2842#issuecomment-6338588907)) and the CHARTER §5.2 bar registration decision card ([`#2028` comment 6338596024](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6338596024)).
- **Discrete normalization module (`crates/uor-r4-discrete/src/normalization_mechanism.rs`, +446/-0 lines; wired in `crates/uor-r4-discrete/src/lib.rs`, +1/-0 lines):**
  - Implemented `NormalizationMode` (`ScaleMatchedSnapV1`, `UnitSnapControlV1`, `UnnormalizedControlV1`), `BlockNormInvariants`, `SnapProjection`, `NormFrameTable`, `root_scaled_cost`, `normalize_block_with_frame`, `normalize_block_mode`, `normalize_carrier_vector`, and `endpoint_address_distance` over the build-time generated `geometry.rs` tables (`ROOTS`, `AXES`, `GENERATORS`, `IDENTITY`), using pure integer arithmetic (`0` floats, `0` smooth gates/exponentials, `0` dense matmuls).
  - Implemented 6 finite CPU unit tests verifying norm stability, two-pass idempotence, scale equivariance and majority-vote preservation over un-scaled `snap_1`, deterministic tie-breaking on zero and all 120 root pairs, 64-frame $\mathrm{Cl}(0,6)$ equivariance and 4-bit-per-block D11 export, and multi-stage residual norm stability.
- **Pre-registration draft, lab report, and architecture/geometry docs (+142/-4 lines):**
  - Added [`docs/harness/prep/m1-normalization-mechanism-DRAFT-2026-10-22.md`](../../../../harness/prep/m1-normalization-mechanism-DRAFT-2026-10-22.md) (`TEST FITNESS: PENDING`) and [`docs/labs/m1-normalization-mechanism/README.md`](../../../../labs/m1-normalization-mechanism/README.md).
  - Updated [`docs/model/architecture.md`](../../../../model/architecture.md), [`docs/geometry/e8-h4-and-the-600-cell.md`](../../../../geometry/e8-h4-and-the-600-cell.md), [`docs/geometry/quaternions-and-rotation.md`](../../../../geometry/quaternions-and-rotation.md), and [`docs/mechanisms/hamilton-gate.md`](../../../../mechanisms/hamilton-gate.md).

## Measurements

- `~/.cargo/bin/cargo test -p uor-r4-discrete --lib normalization_mechanism` -> `6 passed; 0 failed; 0 ignored` in `0.07s`:
  - `norm_stability_and_bounded_range`: $\|\hat{x}\|_1 = 4$, $\|\hat{x}\|_2^2 = 8$, $\|\hat{x}\|_\infty = 2$, support $= 2$ across all tested inputs and $n \in 1..=16$.
  - `idempotence_two_pass_stability`: $\operatorname{Norm}_1(\operatorname{Norm}_n(x)) = \operatorname{Norm}_n(x)$ with second-pass $d_1 = 0, d_H = 0$ across 64 seeded vectors $\times 4$ multiplicities plus all 16 canonical roots.
  - `scale_equivariance_on_orbit_multiples`: $\operatorname{Norm}_n(n\rho_r) = \rho_r$ ($d_1 = 0, d_H = 0$) vs. un-scaled $d_1(n\rho_r, \rho_r) = 4(n - 1)$; on $3\rho_0 + \rho_{\text{orth}}$, `snap_4` selects majority root `0` ($d_1 = 8$) while `snap_1` flips to minority root $r_{\text{orth}}$ ($d_1 = 12, d_H = 2 < 4$).
  - `deterministic_tie_breaking_on_zero_and_midpoints`: $x = \mathbf{0} \mapsto \text{root } 0$ ($d_1 = 4n, d_H = 2$); all $\binom{16}{2} = 120$ root-pair sums commute and tie-break deterministically.
  - `cl06_frame_equivariance_across_all_64_actions`: exact equivariance across all $64 \times 16 \times 3 = 3{,}072$ `(frame, root, multiplicity)` triples; `NormFrameTable` exports 4 bits/block (`16` bits at $B = 4$).
  - `multistage_residual_accumulation_vs_unnormalized_drift`: across $S = 16$ stages, `UnnormalizedControlV1` drifts to $\|x\|_1 = 68$, $\|x\|_\infty = 34$, and `endpoint_address_distance > 16`, while `ScaleMatchedSnapV1` holds $\|\hat{x}\|_1 = 4$, $\|\hat{x}\|_\infty = 2$, and `endpoint_address_distance` $\in \{0, 4, 8, 12, 16\}$.

## Checks run at the head

- `~/.cargo/bin/cargo test -p uor-r4-discrete --lib normalization_mechanism` -> `6 passed; 0 failed; 0 ignored`.
- `~/.cargo/bin/cargo test -p uor-r4-discrete --lib apply_calls_are_confined_to_bounded_fixture_and_admitted_transition` -> `1 passed; 0 failed`.
- `~/.cargo/bin/cargo clippy --no-deps -p uor-r4-discrete --lib --message-format=short` -> `0` warnings in `normalization_mechanism.rs`.
- `uor-harness ratchet d23`, `uor-harness ratchet d25`, `uor-harness ratchet d32` -> `0` new sites.
- `~/.cargo/bin/cargo run -q --locked -p xtask -- links` -> `930 broken` (unchanged from baseline `930`).
- `uor-harness loop gates --full` and `uor-harness loop report --receipts` -> 6 gates `GREEN`, receipts recorded in [`receipts.jsonl`](receipts.jsonl).

## Skills used

- `uor-research-architect`

## Still open

- Non-author `TEST FITNESS: FIT` review of `docs/harness/prep/m1-normalization-mechanism-DRAFT-2026-10-22.md`, owner decision on the `#2028` bar registration card (`https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6338596024`), trunk assembly (`m1-trunk-assembly-and-training`), and a typed D40 owner release before running `N_snap`, `C_unscaled`, and `C_raw` on GPU (tracked on `#2842`).

## Plain summary

When a multi-stage model adds several 8-number integer blocks together across layers, the numbers grow with each addition: after 16 additions, a block whose numbers started at $-2, 0, +2$ can reach $\pm 34$, which breaks the fixed 5-value distance scale used by the output writer to pick candidate words. Worse, if the model snaps a sum of 4 blocks back to a single block without accounting for the fact that 4 blocks were added (`snap_1`), a 3-to-1 majority vote ($3\rho_0 + \rho_{\text{orth}}$) ties on total distance and mistakenly picks the 1-vote minority block because its two smaller numbers happen to match $+2$ exactly. This task builds `OrbitRootSnapNormV1` in `crates/uor-r4-discrete/src/normalization_mechanism.rs`, which compares an $n$-block sum against $n$-times-scaled lattice roots ($n\rho_r$, built by repeated integer addition with no multiplication or floating-point numbers) and snaps back to one of the 16 canonical roots ($\pm 2$ in two positions, $0$ in six positions, keeping $L_1 = 4$ and 4 bits per block), proves its stability across 6 finite CPU controls, and pre-registers its trunk comparison sheet.
