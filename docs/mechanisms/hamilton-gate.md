# Hamilton rotation gate

**Status: READY.** The exact forward and backward passes, the GPU kernels, the 120-icosian snap + Q12 `i32` export path, and the first 256-step D37 offline-float language-fit and export-gap sheet (`#2824`) are built and measured on `1 x RTX 5090`. Learned quaternion rotation beats the fixed-random-icosian null by `+1,010..+1,402` DEV `top-1` rows and exports with a `2..23`-row (`0.014..0.161 pp`) gap under hard-forward icosian training (`Hamilton_IcosianHard`), but trails the un-rotated identity control (`IdentityGate_Control`) at the 256-step micro-rung budget.

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

**The hazard and per-block suppression (`Supp_Hamilton`, proof).** Define intrinsic per-block suppression (**Definition**, `Supp_Hamilton`) as achieving amplitude ratio $\rho(g,u):=\|\text{out}\|_2/\|u\|_2 \le \tau$ (registered at $\tau=0.5$) with bounded backward Jacobian norm $\|\partial q/\partial g\|_2 \le 10$. Because $\rho(g,u)=\|g\|_2/\sqrt{\|g\|_2^2+10^{-12}}$ and $\rho^2 + 10^{-12}/r^2 = 1$, we have $\rho(g,u)\le\tau$ if and only if $\|g\|_2 \le \frac{\tau}{\sqrt{1-\tau^2}}\times 10^{-6}$ ($\approx 5.7735\times 10^{-7}$ at $\tau=0.5$), where the orthogonal backward scale is $1/r = 10^6\sqrt{1-\rho^2} \ge \frac{\sqrt{3}}{2}\times 10^6 \approx 8.6603\times 10^5$ and the radial derivative is $\frac{d\|q\|_2}{d\|g\|_2} = 10^6(1-\rho^2)^{3/2} \ge 6.4952\times 10^5$ (**Guarantee**, Status: **Structural**, [`geometric_stack_native.rs`](../../crates/uor-r4-training/src/geometric_stack_native.rs) `hamilton_gate_suppression_closed_form_and_backward_hazard_proof`). Thus a norm-preserving Hamilton gate cannot suppress per-block amplitude outside the $10^{-12}$ division-by-zero guard core, and inside that core any amplitude reduction is algebraically locked to a $\sim 10^6$ backward gradient blow-up. Weight decay can push a block toward zero; no real training run has checked this.

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
| 22 Oct | Analytic **proof** and finite `f32` check (`r-hamilton-gate-suppression`, `#2879`) under registered definition `Supp_Hamilton` ($\rho(g,u)=\|\text{out}\|_2/\|u\|_2 \le 0.5$ with bounded backward Jacobian $\|\partial q/\partial g\|_2 \le 10$), swept across $\|g\|_2 \in \{0\}\cup[10^{-9},10^3]$ | $\rho(g,u)=\|g\|_2/\sqrt{\|g\|_2^2+10^{-12}} \le 0.5$ holds iff $\|g\|_2 \le 10^{-6}/\sqrt{3} \approx 5.7735\times 10^{-7}$, where $1/r = 10^6\sqrt{1-\rho^2} \ge 8.6603\times 10^5$ (smallest decade ratio $9.999995\times 10^{-4}$ at $\|g\|_2=10^{-9}$; $1.0\text{f32}$ exact for $\|g\|_2\ge 10^{-2}$). Single-direction orthogonal steering ($\langle w,q\otimes u\rangle=0$) preserves full block norm $\|\text{out}\|_2=\|u\|_2$ | negative for per-block amplitude suppression (**proof**; closes un-scaled unit-norm amplitude shutoff per D22, not rotation mechanism) | [record](../labs/r-hamilton-gate-suppression/README.md) |
| 25 Oct | 4-arm $\times$ 2-seed $\times$ 256-step D37 offline-float language-fit, 120-icosian + Q12 `i32` export-gap, and signal-suppression sheet (`hamilton-gate-offline-float-sheet`, `#2824`, [sheet](../harness/prep/hamilton-gate-offline-float-sheet-2026-10-25.md)) on `1 x RTX 5090` (`sm_120`, Vast pod `vast-57698805`, binary `b9cbd600e601515820caf909c48b09c83b9d7ef2a52a161678a49960eee7e15c` replicated across two clean `nvcc` builds) over `mw-balp/train` (`64` batches $\times$ `4` $\times$ `192`) and all `14,287` eligible DEV targets (`ft/dev/tokens.u16`) | **Speed:** `14,206–14,318 tok/s` integer-exported DEV eval (`12,942–13,063 tok/s` train for `Hamilton_Active`; `4,461–4,485 tok/s` train for `Hamilton_IcosianHard`; `124,482–125,674 tok/s` int eval for `IdentityGate_Control`). **Integer DEV `top-1` (Build A / Build B):** `Hamilton_Active` `1,270` (`8.89%`) & `1,298` (`9.09%`) / `1,242` (`8.69%`) & `1,057` (`7.40%`); `Hamilton_IcosianHard` `1,430` (`10.01%`) & `1,253` (`8.77%`) / `1,304` (`9.13%`) & `1,034` (`7.24%`), beating `RandomGate_Null` (`24–28`, `+1,010..+1,402` rows) and beating `TrainUnigram_CountTable` (`1,226`, `8.58%`) on both seeds in Build A and seed 1 in Build B, but trailing `IdentityGate_Control` (`3,041–3,908`, `21.29%–27.35%`) by `1,743–2,644` rows. **Export gap (`G-Hamilton-ExportGap = PASS`):** `Hamilton_Active` $\Delta_{\text{top1}} = 80–312$ rows (`17.35%–20.23%` row-wise agreement); `Hamilton_IcosianHard` **$\Delta_{\text{top1}} = 2–23$ rows (`0.014–0.161 pp`)** with **`97.59%–97.84%`** float-vs-integer row agreement. **Signal suppression:** `0 / 78,888,960` blocks enter $\rho \le 0.5$ or $\|g\|_2 \le 10^{-3}$ ($\min \|g\|_2 = 0.00918$, $\max 1/r = 108.94$, $\min \rho = 0.99999971$), and `0 / 986,112` downstream `SpiralcoreFbs` rows show $\gamma_{\text{down}} < 0.8$ (`mean = 1.000000`) | mixed (**measurement** + **proof**: `I-1..4 = PASS`, `G-Beats-Random = PASS`, `G-Hamilton-ExportGap = PASS`, `G-Fit-Unigram = FAIL` against `IdentityGate_Control` at 256 steps; confirms zero intrinsic or `SpiralcoreFbs`-downstream amplitude suppression; closes 256-step un-preconditioned micro-rung config only per CHARTER §4.1) | [record](../labs/hamilton-gate-offline-float-sheet/README.md), [`receipt.json`](../labs/hamilton-gate-offline-float-sheet/receipt.json) (`ac82937195856ef136b98c57158254337fcd76fdcae30a7603de9d78696c432f`) |

**Reading these honestly.** The 22 October row settles per-block amplitude suppression by **proof** (**Guarantee**, Status: **Structural**), and the 25 October D37 offline-float run (`#2824`, **measurement**) settles the empirical training and export-gap questions at the 256-step micro-rung scale: (1) under `weight_decay = 0.1`, **zero** of `78,888,960` gate blocks per evaluation ever approach the $10^{-12}$ guard core ($\min \|g_b\|_2 = 0.00918$, $\max 1/r = 108.94$, $\min \rho = 0.99999971$), and because `QuaternionWeights::SpiralcoreFbs` applies an orthogonal `Cl(0,6)` signed permutation to each routed 8D input atom (two 4-blocks), the downstream projection norm ratio is also identically `1.000000` (`0 / 986,112` rows with $\gamma_{\text{down}} < 0.8$, **proof** + **measurement**); (2) hard-forward 120-icosian snapping with a straight-through backward pass (`Hamilton_IcosianHard`) eliminates the continuous-to-icosian quantization shock (`97.59%–97.84%` row-wise float-vs-integer top-1 agreement, $\Delta_{\text{top1}} = 2–23$ rows vs `80–312` rows for post-hoc snapping of `Hamilton_Active`); and (3) while learned quaternion rotation beats `RandomGate_Null` by `> 1,000` rows and clears `TrainUnigram_CountTable` (`1,226`) on both seeds in Build A, it trails the un-rotated `IdentityGate_Control` (`3,041–3,908`) by `1,743–2,644` rows at 256 steps because random initial `mlp.gate`/`rec.gate` projections scramble the `up` branch at step 0 (`admitted = 5.87%`).

## What has never been tested

- **Identity-preconditioned or residual-interpolated `Hamilton_IcosianHard` initialization** (starting $q_b$ at the identity icosian $(1,0,0,0)$ so step 0 matches `IdentityGate_Control` rather than uniform $S^3$ scrambling) at longer horizon (`>= 1,024` steps).
- **A matched comparison against the frozen pre-ban artifact's stored outputs** at full rung scale.
- **Cross-block mixing downstream of `Gate::Hamilton`** (where relative quaternion phases between distinct 4-blocks can cancel in sum, unlike single-atom `SpiralcoreFbs` routing).

## What would reopen it

The gate is READY, so these are the steps to make it ACTIVE.

1. A new D37 offline-float sheet in a separately scanned module, with a fresh test-fitness review and the owner release that applies. The training holds still apply.
2. A matched comparison of `gate=hamilton` against the SwiGLU gate and the frozen pre-ban artifact, at equal width, steps and data, with at least 2 seeds or 200 fresh rows and fresh transfer for any KEEP. The sigmoid arm is banned for new runs, so the comparison must use the stored outputs of the frozen artifact, not a fresh SwiGLU run.
3. The three open items from the owner's card: the icosian snap, the hard-forward geometry with its gradient bridge, and a table-served form, with the export gap measured on the same evaluator.
4. A real-run check of the near-zero gate hazard.
5. A named, device-qualified parity receipt for the gate tests.

Open questions: does a rotation gate express enough in a language run given that it cannot attenuate a 4-block's amplitude outside the $10^{-12}$ guard core (**proof**, `#2879`); how $q$ can be snapped to one of 120 rotations without losing the cost advantage seen for weights (+0.0196 bits per byte for exact activations in the [activation-snap study](../../archive/labs/activation-snap-2026-10-11/README.md)); and whether it adds anything over the discrete model's own ternary gating, or only duplicates it at higher cost (architect audit, 18 October). These are **hypotheses**.

## Also known as

G2; OP1; `Gate::Hamilton`; $q(g)\otimes u$; geometric feed-forward gate; `gate=hamilton`.

## Code

- [`hamilton_sheet.rs`](../../crates/uor-r4-float-train/src/hamilton_sheet.rs) and [`bin/hamilton-gate-sheet.rs`](../../crates/uor-r4-float-train/src/bin/hamilton-gate-sheet.rs): the D37 offline-float 4-arm runner (`Hamilton_Active`, `Hamilton_IcosianHard`, `IdentityGate_Control`, `RandomGate_Null`), 120-icosian snap + Q12 `i32` `NativeVectorWriter` integer-export evaluator, and intrinsic + downstream suppression telemetry (`#2824`).
- [`geometric_stack_native.rs`](../../crates/uor-r4-training/src/geometric_stack_native.rs): the compiled `geometric_stack` module (`crates/uor-r4-training/src/lib.rs:110`), including `Gate`, `HamiltonGate`, `hamilton_gate` (line 317), `StackModel::mlp` (line 990), and `tests::hamilton_gate_suppression_closed_form_and_backward_hazard_proof`. (The older [`geometric_stack.rs`](../../crates/uor-r4-training/src/geometric_stack.rs) is retained inactive source from which the native module was extracted.)
- [`cuda_stack.cu`](../../crates/uor-r4-training/src/cuda_stack.cu): `hamilton_gate_fwd` (line 172) and `hamilton_gate_bwd` (line 192), one thread per 4-block.
- [`geometric_stack/cuda_ops.rs`](../../crates/uor-r4-training/src/geometric_stack/cuda_ops.rs): the launcher; it errors rather than falling back to the host.
- [`tests/cuda_stack_ops_parity.rs`](../../crates/uor-r4-training/tests/cuda_stack_ops_parity.rs): retained historical parity source (`#![cfg(any())]`, excluded from builds).
- [`examples/geometric-stack.rs`](../../crates/uor-r4-training/examples/geometric-stack.rs): the trainer flag `gate=swiglu|hamilton`.
- The frozen first-rung sheet: [`m1-rank-margin-rung0-2026-10-13.md`](../../archive/harness/prep/m1-rank-margin-rung0-2026-10-13.md).

Audit source: `docs/labs/repo-renormalization-2026-10-19/audit/mechanisms/hamilton-gate.json`.

## Related mechanisms

[recurrence-gates](./recurrence-gates.md): uses this operation as its output gate. [spin-weights](./spin-weights.md): the same Hamilton product applied to weights; the fused projection was this gate's sibling. [spiralcore-fbs](./spiralcore-fbs.md): replaced that projection; the gate and the projection form the geometric feed-forward block. [rank-margin](./rank-margin.md): the first-rung sheet that was to train this gate with the margin objective. [ternary-xor](./ternary-xor.md): the discrete model's ternary gates are the active gating mechanism. Because unit-quaternion rotation preserves $|u|$, inter-stage norm stabilization on the discrete path is handled separately by scale-matched E8 orbit-root snapping (`OrbitRootSnapNormV1`, `#2842`; [`normalization_mechanism.rs`](../../crates/uor-r4-discrete/src/normalization_mechanism.rs), [record](../labs/m1-normalization-mechanism/README.md)).
