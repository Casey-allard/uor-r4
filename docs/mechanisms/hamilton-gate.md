# Hamilton rotation gate

**Status: READY.** The exact forward and backward passes, the GPU kernels and the trainer switch are built and tested. The gate has never been trained: no language run exists, and nothing has been compared with the gate it replaces.

Labels used below: **proof** means derived from definitions or code, **measurement** means a recorded run or test, **hypothesis** means an untested idea. See [`docs/formal_vocabulary.md`](../formal_vocabulary.md).

## In plain words

Inside a feed-forward layer, a gate decides how much of a signal passes through. The usual gate squashes a number into the range 0 to 1 with a sigmoid curve, and the project's rules now ban that curve. This gate instead turns a gate signal into a rotation in four dimensions and rotates the matching value by it, four numbers at a time. A rotation only turns a vector. It uses no exponential and no squashing. A consequence worth knowing: a rotation cannot make a vector shorter, so this gate steers the signal rather than switching it off. Whether that is enough is not yet known.

## What it replaces in a transformer

The gate in the feed-forward layer (the multilayer perceptron) that follows attention. The common form, SwiGLU, multiplies the value by a sigmoid-shaped gate.

![Left: a value vector drawn as an arrow. As the gate input sweeps, the arrow under a sigmoid gate shrinks toward nothing, while under the Hamilton gate it keeps its length and swings around a sphere. Middle: two feed-forward wiring diagrams side by side. SwiGLU runs gate, sigmoid, then times the up branch. Hamilton runs gate, divide by length, then a quaternion product with the up branch; the banned sigmoid is crossed out and a dashed box shows a future table read for the product. Right: the backward factor one over r plotted against gate length, with the 1e-12 guard marked.](../figures/mechanisms/hamilton-gate.svg)

*What the swap does and does not do. The rotation keeps the signal's length; the length of the gate block enters only through the backward pass (right, drawn from the formula).*

## How it works

Work on blocks of four numbers. Let $g$ be a gate block and $u$ the matching "up" (value) block of the hidden layer. Compute
$$r=\sqrt{|g|^2+10^{-12}},\qquad q=g/r,\qquad \text{out}=q\otimes u,$$
where $\otimes$ is the Hamilton product (the quaternion product) and $q\otimes u=L(q)\,u$ with the $4\times4$ matrix $L$ given on the [spin-weights page](./spin-weights.md#how-it-works).

**Two facts (proof).**
- Quaternion length is multiplicative, $|q\otimes u|=|q|\,|u|$, and $|q|=|g|/\sqrt{|g|^2+10^{-12}}$. So $|{\rm out}|=|u|$ up to a factor that falls below 1 only when $|g|$ is near $10^{-6}$ or smaller (at $|g|=10^{-6}$ the factor is $0.71$). A zero gate block gives $q=0$ and zero output. The output's length is set by $u$ alone; the gate sets only its direction.
- The backward pass is exact, including the normalisation (**Definition**):
$$\partial u=\bar q\otimes d,\qquad \partial q=d\otimes\bar u,\qquad \partial g=\frac{\partial q-q\,(q\cdot\partial q)}{r},$$
with $d$ the incoming adjoint and $\bar q$ the quaternion conjugate. No `exp`, sigmoid, tanh or softmax appears in either direction.

**The hazard (proof, from the formula).** Because $\partial g$ divides by $r$, a gate block near zero gives a gradient of about $1/r\approx10^{6}$. Weight decay can push a block toward zero. No real run has checked this.

**Wiring.** The hidden width must be divisible by 4. One GPU thread handles one block. The setting `gate=hamilton` selects it; the default stays the sigmoid SwiGLU gate so old artifacts load byte for byte. Export for integer serving and quantisation-aware training refuse any non-default gate. The same operation is also the output gate of the [geometric recurrence](./recurrence-gates.md).

**What the owner's card left open (hypothesis).** An icosian snap of $q$, so the product becomes a read in the 120×120 table; an exact hard forward with a straight-through gradient bridge; and a serving form.

**Rule check.**
- *D11 (integer serving):* not servable today. The exporter refuses other gates, and the normalisation divides by a square root and multiplies two runtime values. It becomes table-servable only after the icosian snap makes $q$ one of 120 rotations.
- *D23 (no softmax or squashing):* passes by construction. D25 closed the sigmoid exception that SwiGLU relied on, which this replaces.
- *D25/D30 (no dense matrix multiplication):* a per-block Hamilton product is not a dense contraction. The owner permits a geometric feed-forward block of this gate plus Hamilton-block projections, and the projection half has since been replaced by [SpiralCore/FBS](./spiralcore-fbs.md).
- *D32/D37:* a float operation, frozen by D32. D37 lets it return only through a new pre-registered offline sheet ([decisions](../integration/DECISIONS.md#d37--floating-point-is-legal-in-offline-training-only-d32s-learning-path-float-ban-is-revised)).

## What has been tried

| Date | What was run | Result | Verdict | Record |
|---|---|---|---|---|
| 23 Sep | Scoped reading of the owner's Hamilton hint; exact signed-permutation actions on integer 4-vectors | The actions preserve length and are invertible ($K^2=-I$). Rotating every query and key together cannot change an inner-product ranking, so a useful mechanism needs relative or content-dependent operations. Not evidence for attention. No model run | inconclusive | [record](../../archive/integration/hamilton-vector-interpretation-2026-09-23.md) |
| 12 Oct | The gate (arm G2): CPU reference, GPU kernels, trainer flag, saved-config field. A rectified-square arm (G1) was removed before delivery | 944 library tests pass, 0 fail, 7 ignored. Host-executed kernel against an independent reference within $9.3\times10^{-7}$, hand-computed blocks exact. A missing half-precision dispatch was found and fixed. No run, no device receipt, no model numbers | inconclusive | [record](../../archive/labs/gate-ops-2026-10-12/README.md) |
| 13 Oct | Vast.ai RTX 5090 parity run, after a missing import broke every pod's parity step | 43 passed, 0 failed, 3 ignored (38 before the gate work). The summary line does not name the tests | inconclusive | [record](../../archive/labs/cuda-parity-gate-import-2026-10-13/README.md) |
| 13 Oct | 18 finite controls (11 on CUDA, 7 metadata or argument checks) of the declared first-rung configuration (recurrence-only trunk with this gate, SpiralCore/FBS projection, product head, margin objective) at batch 16 by context 384 | 18 of 18 pass on source `05c84bece`; peak GPU 9,274 MiB. No optimizer update, no quality number | positive, controls only | [record](../../archive/labs/rung0-complete-path-2026-10-13/README.md) |

**Reading these honestly.** Every row is a correctness check. None says the gate helps a language model, and the parity count does not name which tests ran. The rectified-square arm G1 was withdrawn by the owner as non-geometric, not tested and found wanting, so no fallback arm exists. The missing import was a test-source blocker, not a mechanism result. Together the parity run and the 18 controls (11 of them on CUDA) are the device evidence that makes the status READY rather than a source-only draft.

## What has never been tested

- **Any language fit with the gate,** or any comparison with the SwiGLU gate or the frozen pre-ban artifact.
- **The icosian snap of $q$** and the hard-forward with its gradient bridge.
- **A table-served form** for integer serving, and its export gap.
- **The $1/r$ gradient near a zero gate block** in a real run.
- **Whether a norm-preserving gate can suppress a signal at all.** The formula says the output length equals $|u|$ for any gate except a near-zero block. This is a reading of the formula; the records do not discuss it (**hypothesis** that it matters).
- **A standalone named device parity result** for the gate.

## What would reopen it

The gate is READY, so these are the steps to make it ACTIVE.

1. A new D37 offline-float sheet in a separately scanned module, with a fresh test-fitness review and the owner release that applies. The training holds still apply.
2. A matched comparison of `gate=hamilton` against the SwiGLU gate and the frozen pre-ban artifact, at equal width, steps and data, with at least 2 seeds or 200 fresh rows and fresh transfer for any KEEP. The sigmoid arm is banned for new runs, so the comparison must use the stored outputs of the frozen artifact, not a fresh SwiGLU run.
3. The three open items from the owner's card: the icosian snap, the hard-forward geometry with its gradient bridge, and a table-served form, with the export gap measured on the same evaluator.
4. A real-run check of the near-zero gate hazard.
5. A named, device-qualified parity receipt for the gate tests.

Open questions: does a rotation gate express enough, since a sigmoid gate can shut a channel and a unit rotation cannot; how $q$ can be snapped to one of 120 rotations without losing the cost advantage seen for weights (+0.0196 bits per byte for exact activations in the [activation-snap study](../../archive/labs/activation-snap-2026-10-11/README.md)); and whether it adds anything over the discrete model's own ternary gating, or only duplicates it at higher cost (architect audit, 18 October). These are **hypotheses**.

## Also known as

G2; OP1; `Gate::Hamilton`; $q(g)\otimes u$; geometric feed-forward gate; `gate=hamilton`.

## Code

- [`geometric_stack.rs`](../../crates/uor-r4-training/src/geometric_stack.rs): the `Gate` setting, the `HamiltonGate` operation, `hamilton_gate`, and the divisible-by-4 check.
- [`cuda_stack.cu`](../../crates/uor-r4-training/src/cuda_stack.cu): `hamilton_gate_fwd` and `hamilton_gate_bwd`, one thread per 4-block.
- [`geometric_stack/cuda_ops.rs`](../../crates/uor-r4-training/src/geometric_stack/cuda_ops.rs): the launcher; it errors rather than falling back to the host.
- [`tests/cuda_stack_ops_parity.rs`](../../crates/uor-r4-training/tests/cuda_stack_ops_parity.rs): `test_hamilton_gate_parity` and `test_hamilton_gate_model_parity`.
- [`examples/geometric-stack.rs`](../../crates/uor-r4-training/examples/geometric-stack.rs): the trainer flag `gate=swiglu|hamilton`.
- The frozen first-rung sheet: [`m1-rank-margin-rung0-2026-10-13.md`](../../archive/harness/prep/m1-rank-margin-rung0-2026-10-13.md).

Audit source: `docs/labs/repo-renormalization-2026-10-19/audit/mechanisms/hamilton-gate.json`.

## Related mechanisms

[recurrence-gates](./recurrence-gates.md): uses this operation as its output gate. [spin-weights](./spin-weights.md): the same Hamilton product applied to weights; the fused projection was this gate's sibling. [spiralcore-fbs](./spiralcore-fbs.md): replaced that projection; the gate and the projection form the geometric feed-forward block. [rank-margin](./rank-margin.md): the first-rung sheet that was to train this gate with the margin objective. [ternary-xor](./ternary-xor.md): the discrete model's ternary gates are the active gating mechanism.
