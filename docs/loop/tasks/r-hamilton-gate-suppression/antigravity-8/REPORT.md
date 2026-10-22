# REPORT: r-hamilton-gate-suppression, antigravity-8

Lab: antigravity. Order: [ORDER.md](ORDER.md).

Closes #2879.

## What did not work, first

- **Per-block amplitude suppression by a norm-preserving Hamilton gate is falsified by proof (`Supp_Hamilton`, `Guarantee`, Status: `Structural`):**
  - Under the registered definition `Supp_Hamilton` (posted on `#2879` at `https://github.com/UOR-Foundation/uor-r4/issues/2879#issuecomment-6336354313` before testing: achieving per-4-block amplitude ratio $\rho(g, u) := \|\text{out}(g, u)\|_2 / \|u\|_2 \le \tau = 0.5$ with bounded backward Jacobian norm $\|\partial q / \partial g\|_2 \le 10$), exact quaternion norm multiplicativity gives $\rho(g, u) = \|q\|_2 = \|g\|_2 / \sqrt{\|g\|_2^2 + 10^{-12}}$ and the exact coupling identity $\rho^2 + 10^{-12} / r^2 = 1$.
  - Therefore $\rho(g, u) \le 0.5$ holds if and only if $\|g\|_2 \le 10^{-6} / \sqrt{3} \approx 5.7735027 \times 10^{-7}$, where the backward Jacobian norm on $q^\perp$ is $1/r = 10^6 \sqrt{1 - \rho^2} \ge \frac{\sqrt{3}}{2} \times 10^6 \approx 8.660254 \times 10^5$ (and the radial derivative is $10^6 (1 - \rho^2)^{3/2} \ge 6.4951905 \times 10^5$, reaching $10^6$ at $g = 0$). In `f32`, for all $\|g\|_2 \ge 10^{-2}$, $\|g\|_2^2 + 10^{-12} = \|g\|_2^2$ bit-for-bit so $\rho(g, u) = 1.0\text{f32}$ exact; across the decade sweep $\|g\|_2 \in [10^{-9}, 10^3]$, the smallest ratio reached is $9.9999947 \times 10^{-4}$ at $\|g\|_2 = 10^{-9}$ ($0.0$ at $\|g\|_2 = 0$).
  - Per D22 (CHARTER §4.1), this negative closes the **un-scaled unit-norm configuration** as an amplitude-shutoff gate, never the Hamilton rotation mechanism itself (which remains a pure 4D directional rotor and can zero a single scalar readout projection $\langle w, q \otimes u \rangle = 0$ while preserving full 4-block norm $\|q \otimes u\|_2 = \|u\|_2$).
- Writing `hamilton_gate_suppression_closed_form_and_backward_hazard_proof` in `crates/uor-r4-training/src/geometric_stack_native.rs` before adding `hamilton_gate_scalar_ratio` and running `cargo test -p uor-r4-training --lib hamilton_gate_suppression_closed_form_and_backward_hazard_proof` failed as expected with `error[E0425]: cannot find function hamilton_gate_scalar_ratio in this scope`.
- Replacing `/ r` with `* r` in `hamilton_gate_block_bwd` caused `hamilton_gate_suppression_closed_form_and_backward_hazard_proof` to fail as expected (`|d_gate| 0.000001 != 1/r 1000000`), and passed once `/ r` was restored.

## What changed

- Posted the design note on `#2879` (`https://github.com/UOR-Foundation/uor-r4/issues/2879#issuecomment-6336354313`) before writing any test, registering `Supp_Hamilton`, YES/NO criteria, the analytic proof of $\rho(g, u)$ and $1/r$, test form `(a)`, and the notebook-page question for the maintainer.
- Added `hamilton_product_4`, `hamilton_gate_scalar_ratio`, `hamilton_gate_block_fwd`, `hamilton_gate_block_bwd`, and `hamilton_gate_suppression_closed_form_and_backward_hazard_proof` in `crates/uor-r4-training/src/geometric_stack_native.rs` (`mod tests`), with zero matmul and zero softmax.
- Updated `docs/mechanisms/hamilton-gate.md` ("How it works", "What has been tried", "What has never been tested", and "Code" — pointing to the compiled `geometric_stack_native.rs` and noting that `geometric_stack.rs` and `tests/cuda_stack_ops_parity.rs` are inactive/disabled), `docs/geometry/quaternions-and-rotation.md`, and `docs/formal_vocabulary.md` (`Supp_Hamilton`, **Definition** and **Guarantee**, Status: **Structural**).
- Added `docs/labs/r-hamilton-gate-suppression/README.md` with the research notebook row and marked `docs/roadmap/items/r-hamilton-gate-suppression.md` `status: done`.

## Measurements

- `CARGO_TARGET_DIR=/Users/casey.allard/uor-r4/local/antigravity/cargo-target ~/.cargo/bin/cargo test -p uor-r4-training --lib hamilton_gate_suppression_closed_form_and_backward_hazard_proof -- --nocapture` on macOS aarch64: 1 passed, 0 failed in 0.00s (`smallest ratio reached = 9.9999947e-4 (at |g|=1e-9); at |g|=0 ratio = 0.0`).

## Checks run at the head

- `~/.cargo/bin/cargo fmt -p uor-r4-training -- --check`: exit 0.
- `CARGO_TARGET_DIR=/Users/casey.allard/uor-r4/local/antigravity/cargo-target ~/.cargo/bin/cargo test -p uor-r4-training --lib hamilton_gate_suppression_closed_form_and_backward_hazard_proof -- --nocapture`: 1 passed, 0 failed.
- `uor-harness loop gates --full`: `fmt-claim-wording` GREEN, `fmt-harness` GREEN, `claim-wording` GREEN, `loop-gates-watch` GREEN, `loop-drive` GREEN, `git-hooks` GREEN.
- `uor-harness loop report --check`: PASS.
- `uor-harness loop pr-check --local`: PASS.

## Skills used

- `uor-research-architect` (`.agents/skills/uor-research-architect/SKILL.md`).

## Still open

- None for `#2879`; `r-hamilton-gate-suppression` is complete (`status: done`). Which `docs/research/notebook/` page should host the feed-forward gate row (and mirroring the update to `docs/research/open-questions.md:97`, both outside `r-hamilton-gate-suppression.md`'s `scope:`) is asked on `#2879` (`https://github.com/UOR-Foundation/uor-r4/issues/2879#issuecomment-6336354313`).

## Plain summary

- Settled `r-hamilton-gate-suppression` (`#2879`) by analytic **proof** (`Guarantee`, Status: `Structural`) and a finite `f32` host check (`hamilton_gate_suppression_closed_form_and_backward_hazard_proof` in `crates/uor-r4-training/src/geometric_stack_native.rs`): under the registered definition `Supp_Hamilton` ($\rho(g, u) = \|\text{out}\|_2 / \|u\|_2 \le 0.5$ with bounded backward Jacobian $\|\partial q / \partial g\|_2 \le 10$), the norm-preserving Hamilton gate $q = g / \sqrt{\|g\|_2^2 + 10^{-12}}$ achieves $\rho(g, u) \le 0.5$ if and only if $\|g\|_2 \le 10^{-6}/\sqrt{3} \approx 5.7735 \times 10^{-7}$, where the backward factor $1/r = 10^6 \sqrt{1 - \rho^2} \ge 8.6603 \times 10^5$ blows up.
- Per D22, this negative closes the un-scaled unit-norm configuration as an amplitude-shutoff gate, not the Hamilton rotation mechanism itself.
- Updated `docs/mechanisms/hamilton-gate.md`, `docs/geometry/quaternions-and-rotation.md`, and `docs/formal_vocabulary.md`, and recorded the lab summary and notebook row in `docs/labs/r-hamilton-gate-suppression/README.md`.
