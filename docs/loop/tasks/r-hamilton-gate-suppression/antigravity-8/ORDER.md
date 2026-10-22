# ORDER: r-hamilton-gate-suppression, antigravity-8

Lab: antigravity. Item: [r-hamilton-gate-suppression](../../../../roadmap/items/r-hamilton-gate-suppression.md). Base: `60a0c9cda`.

## Goal

The item's done-when reads:

> A proof or finite test that settles whether such a gate can suppress a signal, labelled **proof** or **measurement** (see `docs/formal_vocabulary.md`). The definition of "suppress" is registered with it, and the result is one row in a research notebook.

## Cancel probe (run first)

Checked `crates/uor-r4-training/src/geometric_stack_native.rs` (`hamilton_gate` at line 317, `mlp` at line 990, `HamiltonGate` at line 1430, `mod tests` at line 1559), `crates/uor-r4-training/src/cuda_stack.cu` (`hamilton_gate_fwd` at line 172 and `hamilton_gate_bwd` at line 192), and `docs/mechanisms/hamilton-gate.md`:

1. No test in `crates/uor-r4-training/src/geometric_stack_native.rs::tests` checks the per-block amplitude ratio $\rho(g, u) = |\text{out}|/|u|$ of the Hamilton gate formula across gate norm decades, its exact threshold for half-amplitude attenuation ($\tau = 0.5$), or the coupled backward Jacobian scale $1/r$.
2. `docs/mechanisms/hamilton-gate.md:60` still lists *"Whether a norm-preserving gate can suppress a signal at all"* under "What has never been tested" as an unsettled reading of the formula, and its `## Code` section still cites only the inactive `geometric_stack.rs` and the `#![cfg(any())]`-disabled parity test file.

Cancel conditions are false; proceeding.

## Files in scope

- `crates/uor-r4-training/**`
- `docs/formal_vocabulary.md`
- `docs/geometry/quaternions-and-rotation.md`
- `docs/mechanisms/hamilton-gate.md`
- `docs/harness/prep/r-hamilton-gate-suppression*`
- `docs/labs/r-hamilton-gate-suppression/**`

## Done when

1. A design note is posted on `#2879` before any test is written, registering the definition of "suppress" (intrinsic per-4-block amplitude attenuation $\rho(g, u) = |\text{out}|/|u| \le \tau$ with bounded backward Jacobian scale $\|\partial q / \partial g\|_2 \le 10$, contrasted with single-direction downstream steering), stating what counts as YES vs NO, deriving the exact effect of the `1e-12` guard and `1/r` backward scaling (**proof**), and declaring test form `(a)` (host f32 scalar formula + Hamilton product test in `geometric_stack_native.rs::tests`).
2. A finite host test in `crates/uor-r4-training/src/geometric_stack_native.rs::tests` (no matmul, no softmax) verifies the closed-form ratio $|g|/\sqrt{|g|^2 + 10^{-12}}$, the scalar Hamilton product `q ⊗ u`, f32 rounding across decades $0$ and $10^{-9}..10^{3}$ over multiple gate/value directions, the exact threshold $|g| = \tau \cdot 10^{-6}/\sqrt{1-\tau^2}$, the coupled backward Jacobian identity $|q|^2 + (10^{-6}/r)^2 = 1$, and single-direction orthogonal steering without block norm reduction.
3. `docs/mechanisms/hamilton-gate.md`, `docs/geometry/quaternions-and-rotation.md`, and `docs/formal_vocabulary.md` are updated with the **proof** (**Guarantee**, Status: **Structural**), and `docs/labs/r-hamilton-gate-suppression/README.md` records the proof and notebook row (while asking on `#2879` which `docs/research/notebook/` page to place the feed-forward gate row in and noting the out-of-scope follow-up on `docs/research/open-questions.md:97`).

## Stop and ask if

- Any file outside the declared `scope:` must be edited.
- Any GPU pod execution is needed (not needed for form `(a)`).
- Any charter rule, milestone bar, hold, or spend cap is implicated.
