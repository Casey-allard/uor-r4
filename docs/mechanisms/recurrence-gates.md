# Geometric recurrence gates

**Status: READY.** The clamped opening, the table of fade rates, the secant credit rule and the Hamilton output gate are built, and 18 finite controls (11 on CUDA, 7 metadata or argument checks) pass. The gates have never been trained, and the fit that was meant to test them never ran.

Labels used below: **proof** means derived from definitions or code, **measurement** means a recorded run or test, **hypothesis** means an untested idea. See [`docs/formal_vocabulary.md`](../formal_vocabulary.md).

## In plain words

A recurrent layer keeps a running memory and decides at each step how fast old content fades and how much new input gets in. The old recipe used smooth curves for those decisions (sigmoid, exponential, tanh), and the project's rules now ban them. The replacement opens the input with a clamped straight line, picks the fade rate from a short table of powers of two, and carries the memory forward by a rotation. Each step is a table read and a rotation, so nothing smooth is needed. It is a drop-in for the gates of one particular recurrent design, not a standard part of a transformer.

## What it replaces in a transformer

A plain transformer has no recurrence, so this replaces nothing there directly. It belongs to the recurrent alternative to attention used in the project's older float stack. In that design it replaces four squashing functions inside the recurrence: a sigmoid opening, an exponential and softplus decay rate, and a tanh-shaped output gate.

![Left: a staircase of the 16 fade rates 2 to the minus s plus 1 against the opening score from 0 to 1, laid over the smooth curve it replaces; the selected step and its two neighbours, which the credit rule compares, are marked. Middle: quaternion lanes drawn as arrows that rotate and shrink each step by the chosen rate, with the fade rate and the keep factor shown as two sides of a right triangle inscribed in a unit circle. Right: retention of one remembered token over 32 steps for several table rates, each a clean power of two, beside the historical continuous curve.](../figures/mechanisms/recurrence-gates.svg)

*A table of 16 fixed clocks replaces a smooth dial. Each step multiplies the old state by the chosen rate and a rotation, and adds new input scaled by the matching keep factor.*

## How it works

The trunk's sequence mixer is a recurrence over lanes of four numbers (quaternions). From the source ([`geometric_stack.rs`](../../crates/uor-r4-training/src/geometric_stack.rs)), one step per lane is
$$h_t \;=\; \lambda_t\,u_t\otimes h_{t-1}\;+\;\kappa_t\,b_t,$$
where $u_t$ is a unit quaternion built from the gate projections (the rotation), $b_t$ is the convolved input drive, and $\otimes$ is the quaternion product. Four squashing sites of the historical form are replaced in the mode `rec_gates=geometric`:

1. **Opening (Definition).** $s_t=\mathrm{clamp}(g_t+d,\,0,\,1)$, where $g_t$ is the gate projection and $d$ is a per-lane decay parameter read raw, with no softplus. The clamp has an exact subgradient.
2. **Rate (Definition).** The index is $i=\min(\lfloor 16\,s_t\rfloor,\,15)$. The rate and the keep factor are read from two 16-entry tables:
$$\lambda=2^{-(i+1)},\qquad \kappa=\sqrt{1-\lambda^2}.$$
The rates are exact powers of two, from $1/2$ down to $2^{-16}$, and $\kappa$ is precomputed beside each one, so serving needs only table reads. The pair $(\lambda,\kappa)$ lies on the unit circle up to $6\times10^{-8}$ (**measurement**; the last four $\kappa$ entries round to exactly 1 in 32-bit floats).
3. **Carry (unchanged).** The quaternion scan above.
4. **Output gate (Definition).** The [Hamilton rotation gate](./hamilton-gate.md) on the recurrence output, replacing a tanh-shaped gate.

**Credit through a hard choice (Definition).** Picking a table entry has zero derivative almost everywhere, so nothing would train. The backward pass uses a declared **secant surrogate**: it pairs the incoming adjoints for $(\lambda,\kappa)$ with the full counterfactual change in the table between the selected entry and each neighbour, and resolves the contrast along the fixed edge of length $1/16$, half to each side and the whole weight to the single neighbour at either end. No probability or exponential is involved. It is a rule, not the derivative of hard selection, and whether it yields descent is the open test.

**A reading of the source (proof from the table, not a recorded finding).** Every table entry satisfies $\lambda\le\tfrac12$, so a step keeps at most half of the old state times a rotation. The historical rate $\lambda=\exp(8\,\sigma(g)\log\sigma(d))$ can approach 1 as the opening goes to 0. Whether a table capped at one half per step keeps enough long memory is untested.

The mode refuses non-GPU devices, export and quantisation-aware training. The historical mode stays the default so old artifacts reproduce.

**Rule check.**
- *D11 (integer serving):* designed for it, since the rates are table reads, the opening is a clamp and the scan is rotations. Not yet exportable, because the integer engine binds the historical decay parameter. The output gate still needs the icosian snap.
- *D23 (no softmax or squashing):* a grep on 13 October found 0 squash-family tokens in the geometric arms (the historical branches keep 8 by design).
- *D25/D30 (no dense matrix multiplication):* a clamp, a table read, a scan and a per-block rotation. The no-matmul and no-softmax counters were unchanged by the delivering PR.
- *D32/D37:* a float operator. D32 froze it and D37 allows it back only by a new offline sheet ([decisions](../integration/DECISIONS.md#d37--floating-point-is-legal-in-offline-training-only-d32s-learning-path-float-ban-is-revised)). The active discrete model carries no recurrent state, so this belongs to a separate float trunk.

## What has been tried

Rows before 13 October test related recurrences, not this table.

| Date | What was run | Result | Verdict | Record |
|---|---|---|---|---|
| 7 Aug | Long-range context ceiling and the decay constant of the shipped fold (issue #424) | Distant context is worth about +1.02 points of top-1 over a 21.62% base. The shipped fold kept 16% of that ceiling ("mechanism right, constant wrong"). The gated A/B was not run | diagnostic stop | [record](../../archive/context_horizon_424.md) |
| 28 Aug | Multirate gated-delta retention core, four banks, geometric arm against a matched plain delta, 28-event synthetic fixture | Geometric 16/28 next-token and 55/112 association margins against plain 23/28 and 98/112. Corpus qualification NOT_RUN | negative | [record](../../archive/geometric_gated_delta_retention_973.md) |
| 19 Sep | Dense ternary recurrence $h_t=\mathrm{relu}(W_xx_t+W_hh_{t-1})$ | Integer state saturates between 16 and 32 steps; recall over 8 steps scores 0.00 at widths 64, 128 and 256 | negative | [record](../../archive/integration/geometric-core-architecture-2026-09-19.md) |
| 25 Sep | One learned recurrent memory, quaternion transport against a matched Householder-pair control (carry state by pairs of reflections), 1.68M parameters, one paired seed | Both pass the limited engineering gate. At the final step, 7,324, the comparison-tail loss is 2.1104 (quaternion) against 2.0852 (ordinary recurrent learner) in nats, so no win over the control. The count-and-cache baseline scored 2.3918 and an offline transformer 1.5740 on the same population. (A later continuation record reports 2.0905 against 2.0644 at step 8,348.) Generations drift and repeat | inconclusive | [record](../../archive/integration/joint-recurrent-result-2026-09-25.md) |
| 25 Sep | Quantisation-aware, then projected continuation of both arms | Packed minus continuous loss +0.0598 (quaternion) and +0.0835 (Householder) against an allowance of +0.05; recipe rejected | negative | [quantized](../../archive/integration/quantized-recurrent-result-2026-09-25.md), [projected](../../archive/integration/projected-recurrent-result-2026-09-25.md) |
| 26 Sep | Continuation of both arms from 8,348 to 15,672 updates | Likelihood improved but no arm meets the unchanged prose rule; the quaternion integer continuation short-cycles | negative | [record](../../archive/integration/language-continuation-result-2026-09-26.md) |
| 27 Sep | Capacity-matched geometric stack (recurrence plus attention) against a transformer control, 7.15M against 7.16M parameters, one seed per arm | 1.9981 against 2.0111 nats at 7,324 updates; 789 against 788 tokens/s. The stack led by 0.158 nats at 1,000 updates, and the lead closed by about 2,750 | positive | [record](../../archive/integration/geometric-stack-cycle4-2026-09-27.md) |
| 27 Sep | Quaternion and reflection-pair recurrent lanes: an A5 word problem, then inside the stack, then a fresh replication | The skill belongs to the finite group, not to quaternions: 17/18 non-commutative runs exact to length 4,096. Inside the stack the per-seed gates fail, so no lane type stays | negative | [record](../../archive/integration/b1-finite-group-lanes-2026-09-27.md) |
| 28 Sep | Quaternion against identity transport at matched width, 1,000 updates, two seeds | Quaternion wins by 0.0711 and 0.0774 nats (bar 0.02). +332,928 parameters; 10% and 8% slower | positive | [record](../../archive/integration/transport-attribution-d1-2026-09-28.md) |
| 13 Oct | The geometric gates themselves: clamp, rate and keep tables, secant credit, Hamilton output gate | Implemented, source only. Rate table error $5\times10^{-13}$, keep $4.6\times10^{-10}$ relative, secant $5.5\times10^{-8}$ relative. Squash tokens 0 against 8 in the historical branches | inconclusive | [record](../../archive/labs/recurrence-geometric-gates-2026-10-13/README.md) |
| 13 Oct | 18 finite controls (11 on CUDA, 7 metadata or argument checks) at the first-rung shape (batch 16 by context 384, six recurrences, width 256, vocabulary 4,096) | 18 of 18 pass on source `05c84bece`. Opening and decay credit is nonzero in all six layers. Five defects repaired, including ordinary recurrence still entering the old sigmoid-exp-GELU core. No optimizer update, no quality number | positive, controls only | [record](../../archive/labs/rung0-complete-path-2026-10-13/README.md) |
| 22 Oct | Discrete `Cl(0,6)` prefix-holonomy position mechanism (`PositionHolonomyV1`, `#2843`): 64-frame causal prefix scan $F_{p,b}=\mathrm{COMPOSE}[u_{p,b}\cdot 64 + F_{p-1,b}]$, relative holonomy $F_{t,b}\circ F_{s,b}^{-1}$, parameter-matched endpoint control and shuffled-prefix null | 5 of 5 finite CPU controls pass in [`position_mechanism.rs`](../../crates/uor-r4-discrete/src/position_mechanism.rs); pre-registration drafted ([sheet](../harness/prep/m1-position-mechanism-DRAFT-2026-10-22.md), `TEST FITNESS: PENDING`); GPU order test not yet run | positive, controls only | [record](../labs/m1-position-mechanism/README.md) |

**Reading these honestly.** The failures are failures of other configurations: an unbounded dense state, a 28-event fixture, a quantisation recipe on the continuous learner, one paired seed with a control that is not worse. The quaternion-against-identity result (+0.07 nats) shows transport matters; the quaternion-against-reflection tie shows rotation is not yet shown better. None of the 13 October rows trains the table, and the stack results used the older squashing gates.

## What has never been tested

- **The first-rung six-recurrence fit,** which was held and then superseded by the discrete direction. No geometric-against-historical gates comparison exists on any data.
- **Whether 16 table entries are fine enough,** compared with a learned continuous decay.
- **Whether the secant surrogate gives descent.**
- **A standalone device test for the rate-select kernel.** None appears in the parity suite, and there is no half-precision margin kernel.
- **A fused geometric recurrence kernel.** The mode runs as a composition of operations.
- **Integer export** of the geometric mode.
- **The quaternion arm against its Householder control** with more than one paired seed, or on the geometric-gates trunk.

## What would reopen it

The gates are READY, so these are the steps to make them ACTIVE.

1. A new D37 offline-float sheet (separately scanned module, fresh test-fitness review, owner release) that runs the six-recurrence fit with `rec_gates=geometric`. The old plan compared against a historical-gates arm that uses sigmoid and exponential, which is banned for new runs, so that comparison must use stored outputs of the earlier runs or a lawful read.
2. A device qualification that names these kernels. The CUDA path is exercised only inside the 18 controls.
3. Evidence that the secant surrogate gives descent: a falling loss, or a comparison with a random-refit control.
4. Fresh transfer, at least 2 seeds and the export gap on the same evaluator. A table-served form needs the integer engine to bind the new rate scores instead of the historical decay parameter.

Open questions: do hard-selected rates train stably under the secant credit or does the table plateau; does rotation beat reflection as the carry at scale and under a finite-precision export; is a recurrence the right sequence mixer at all, now that the active model reads context through the routed and flock reads; and what the export gap looks like. These are **hypotheses**.

## Also known as

`rec_gates=geometric`; `RecGates::Geometric`; `rate_select`; dyadic rate and keep tables; clamped-linear opening; quaternion scan; "dependency 3" of the first-rung sheet (PR #2293).

## Code

- [`geometric_stack.rs`](../../crates/uor-r4-training/src/geometric_stack.rs): the `RecGates` setting, `composed_recurrence`, `quaternion_scan`, `RATE_TABLE`, `KEEP_TABLE`, `RATE_ENTRIES` and the `rate_select` operation.
- [`cuda_stack.cu`](../../crates/uor-r4-training/src/cuda_stack.cu): `rate_select_fwd`/`bwd` and `quaternion_scan_fwd`/`bwd`.
- [`geometric_stack/cuda_ops.rs`](../../crates/uor-r4-training/src/geometric_stack/cuda_ops.rs): the launchers; half-precision twins sit in `cuda_ops_bf16.rs`.
- [`examples/geometric-stack.rs`](../../crates/uor-r4-training/examples/geometric-stack.rs): trainer flag `rec_gates=historical|geometric`.
- [`learner/hamilton_transport.rs`](../../crates/uor-r4-core/src/native_geometric/learner/hamilton_transport.rs): exact signed-permutation actions used for integer transport.
- The first-rung sheet that names the four squashing sites: [`m1-rank-margin-rung0-2026-10-13.md`](../../archive/harness/prep/m1-rank-margin-rung0-2026-10-13.md).

Audit source: `docs/labs/repo-renormalization-2026-10-19/audit/mechanisms/recurrence-gates.json`.

## Related mechanisms

[hamilton-gate](./hamilton-gate.md): supplies the output gate. [spin-weights](./spin-weights.md): shares the Hamilton product; the scan is the carry. [spiralcore-fbs](./spiralcore-fbs.md): shares the secant-surrogate credit pattern. [rank-margin](./rank-margin.md): the margin objective this trunk was to train with. [ternary-xor](./ternary-xor.md): the dense ternary recurrence negative. [discrete-learning](./discrete-learning.md): the active model replaces recurrence with routed exact reads. [geometric-stack](./geometric-stack.md): the 214M float trunk these gates were written for.
