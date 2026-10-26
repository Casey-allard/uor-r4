# SpiralCore transport with ternary tree binders (float projection form)

**Status: READY.** The float projection is built, checked on a real GPU with exact test cases, ran inside 18 finite controls, and passed the pre-registered 256-step offline float secant surrogate descent check (`#2886`, `G-Fbs-Descent = PASS (mechanism descent)` across two seeds on an RTX 5090, bit-identical across two clean builds). Full-rung offline float training ran on 26 Oct (`#2891`, [record](../labs/spiralcore-fbs-offline-float-sheet/README.md)). Descent and tokens per second were measured, and that configuration missed the sheet's fit and export-gap bars, so the float form stays READY. Its integer form is ACTIVE: it is the core of the current model, described under [`discrete-learning`](./discrete-learning.md).

Labels used below: **proof** means derived from definitions or code, **measurement** means a recorded run or test, **hypothesis** means an untested idea. See [`docs/formal_vocabulary.md`](../formal_vocabulary.md).

## In plain words

A normal layer mixes every input into every output using learned weights. This one does something much simpler. Each output block picks one input block from a small tree of stored blocks, three branches per node, and moves it along a path of exact steps in an eight-dimensional lattice called E8. Each step only swaps coordinates and flips signs. No learned coefficient is multiplied in; the path itself does the transforming. The trade is that each output reads one input block instead of a blend of all of them, and nobody has yet measured whether that carries enough information.

## What it replaces in a transformer

The dense projections: the large weight grids that turn each vector into queries, values and layer outputs. In the float stack this replaced a dense quaternion version of the same job ([`spin-weights`](./spin-weights.md)).

![Left: a three-branch tree of atoms with one root-to-leaf path highlighted, beside the 16 roots of the E8 anchor orbit drawn as nodes of a 15-edge graph. As the query point moves, the nearest root changes, and the chosen atom's 8 coordinates travel along the graph edges, each edge permuting axes and flipping signs; then the path is run backward to show the exact inverse. Middle: a concrete 8-vector pushed through two edges, coordinate by coordinate, then the two steps composed into one signed action and checked against the product. Right: a dense grid where every input block connects to every output block, beside a grid with exactly one line per output block, each labelled with the atom it chose.](../figures/mechanisms/spiralcore-fbs.svg)

*One output block, one chosen atom, a short path of signed coordinate swaps (left and middle). The dense projection connects every pair; this one connects one (right).*

## How it works

**The tree (Definition).** Input channels are 8-dimensional integer atoms kept by a static Rust compiler in ordered base-3 binders. The compiler stores children, parent links, atom identity, support length and the incomplete frontier (at most 6,560 atoms, 16 frontier roots and 8 stages). FBS is the project's name for this tree. Exact atom identity stays separate from the geometric state.

**The steps (Definition, from SpiralCore).** SpiralCore is an external exact-geometry source (a hosted file, reviewed on 9 October). Octonion left actions give 15 generators $B$, each a signed coordinate permutation of $\mathbb R^8$ with $B^2=-I$. They act on the 16 roots of one E8 anchor orbit through 64 signed operator states. A breadth-first search over these labelled edges gives an ordered history between any two roots, composed by left-composition.

**Exact inverse (proof).** From $B^2=-I$ it follows $B^4=I$ and so $B^3=B^{-1}$. The inverse of a history is therefore the reversed sequence with each step cubed.

**Forward pass (Definition).** The model stores query points $z\in\mathbb R^{[\text{output blocks},\ \text{stages},\ 8]}$. At each stage a query picks the nearest root of the fixed orbit, and the history between roots composes a signed action $A_j$. Output block $j$ selects one atom $\sigma(j)$ and transports it:
$$y_j \;=\; A_j\,x_{\sigma(j)} .$$
The forward is a gather and a sign. The input adjoint is the exact inverse action, $A_j^{-1}\delta_j$, scattered to the selected atom. No coefficient exists for any input-output pair (proof, by construction).

**Credit for the query (Definition).** Selecting a root is a hard choice. The query credit is a declared **secant surrogate** on root edges: force each lawful sibling, reroute its descendants, pair the output credit with the displacement of the carried block, and resolve along the edge. It is not the derivative of hard selection.

**The capacity question (proof of the count, hypothesis of the consequence).** Each output block reads exactly one input atom, so an output block has one connection where a dense projection has all of them. Whether one transported atom per block is enough, or whether a small routed set per block is needed, has not been compared.

The path runs on the GPU only; the CPU fallback is refused.

**Rule check.**
- *D11 (integer serving):* the float form (32-bit and half-precision) is not servable. Sign flips, permutations and table reads need no multiplier, and the integer descendant in [`crates/uor-r4-discrete`](../../crates/uor-r4-discrete/src/discrete.cu) is integer-only on the GPU.
- *D23 (no softmax):* no probability weights, no normalised exponentials and no learned input-output coefficient grids; selection is hard by distance. The no-softmax counter was 1649 of 1649 on the delivering PR.
- *D25/D30 (no dense matrix multiplication):* retires the dense Hamilton contraction. The no-matmul counter fell from 95 to 93. Static table construction reads no model tensors and is not model compute.
- *D32/D37:* the float form was frozen by D32 and can return only by re-registration under D37. The integer transport is the substrate D32 names as the backbone, and it is the ACTIVE model path ([decisions](../integration/DECISIONS.md#d37--floating-point-is-legal-in-offline-training-only-d32s-learning-path-float-ban-is-revised)).

## What has been tried

Rows marked "integer form" test the ACTIVE descendant, not the READY float form.

| Date | What was run | Result | Verdict | Record |
|---|---|---|---|---|
| 9 Oct | Source review of hosted SpiralCore v69 (240 E8 roots, 15 bivectors, search routes, exact inverses) | Keep the exact finite-action machinery as an input. Not promoted as a model integration. No runtime executed. File 1,025,707 bytes | inconclusive | [review](../../archive/labs/spiralcore-live-review-2026-10-09/README.md) |
| 12 Oct | Persistent base-3 tree per 600-cell vertex retaining keys by learned salience (the "starling" tree), float stack with a softmax teacher from before the softmax ban, 3 seeds, context 1,536 | The tree met its own target: hit rate 0.99, 0.68, 0.90 (bar 0.85 on 2 of 3 seeds). Recall missed its bar: 0.031, 0.615, 0.340, mean 0.329 against 0.54 (softmax 0.999, ring-only 0.241). The readout, not the tree, now limits | positive for the tree | [record](../../archive/labs/starling-tree-2026-10-12/README.md) |
| 12 Oct | Signed-matrix composition rewritten as direct signed-axis permutation, no generic 8×8 multiply | Static algebra checks pass. Not a shared writer, a learning rule or a GPU kernel. No throughput claim | positive | [record](../../archive/labs/native-vector-transport-2026-10-12/README.md) |
| 12 Oct | Finite-kernel check of the float projection on a rented RTX 5090 (CUDA 12.8), no model, no training | Four GPU tests, three static-plan tests, two projection tests and the retired-dense refusal test all pass: all 27 terminal children of a three-level binder, the selected-input adjoint, a two-atom secant, shared credit, and 32-bit and half-precision controls. Exact fixtures show zero absolute error; secant controls within $10^{-5}$. Cost about $0.17 | positive | [record](../../archive/labs/spiralcore-fbs-projection-2026-10-12/README.md) |
| 13 Oct | Product head that reads the projection through one shared embedding table | Qualified at finite GPU boundaries; at most 16 cells × 32 slots = 512 comparisons (128 by default at vocabulary 4,096). No training | positive | [record](../../archive/labs/geometric-head-shared-writer-2026-10-13/README.md) |
| 13 Oct | 18 finite controls (11 on CUDA, 7 metadata or argument checks) at the first-rung shape with this projection and the product head | 18 of 18 pass on source `05c84bece`; peak GPU 9,274 MiB. No optimizer update, no quality number | positive, controls only | [record](../../archive/labs/rung0-complete-path-2026-10-13/README.md) |
| 13 Oct | Integer form: context, tree routing and rotation into a shared integer writer, learning by proposals only | All eleven construction controls pass after the test-fitness review (first attempt: 1 pass, 1 GPU launch-resource stop, 9 NOT_RUN). Virtual proposal witness 640 to 544; applying updates never ran | positive, controls only | [record](../../archive/labs/discrete-driver-cost-2026-10-13/README.md), [census](../labs/course-correction-2026-10-18/audits/mechanism-census.md) |
| 14 Oct | Integer form: native driver and GPU evaluator, vocabulary 4,096, batch 4, context 192, 64 rows | 25 controls and the operation-cost measurement pass. Forward 37.84 ms (routed read) and 40.58 ms (flock read). One counterfactual proposal 3.9 s. About 17.8 hours per fixed-batch coordinate sweep, an UNVERIFIED estimate | positive, controls only | [record](../../archive/labs/discrete-driver-cost-2026-10-13/README.md), [census](../labs/course-correction-2026-10-18/audits/mechanism-census.md) |
| 14 Oct | Integer form: first 1,640-batch training pilot, seed 1, one RTX 5090, 236 accepted updates | Top-8 correct rows 173 to 1,169; top-1 17 to 82; but the loss numerator rose 13,816,032 to 13,884,032. Later traced to an initializer giving only 16 distinct vectors (155 of 4,096 identities after training) | inconclusive | [record](../../archive/labs/a2-pilot-2026-10-14/README.md) |
| 25 Oct | Pre-registered 256-step offline float secant surrogate descent check (`r-secant-surrogate-descent`, `#2886`, [sheet](../harness/prep/r-secant-surrogate-descent-2026-10-25.md)) on `crates/uor-r4-float-train` (`pattern="rrrrrr"`, `width=256`, `batch=4`, `context=192`, `Objective::MarginHinge { rivals: 8 }`, `layers.*.*.spiralcore_fbs` isolated, seeds `1` and `2`, 1 $\times$ RTX 5090 `sm_120`, ~8,790 training tokens/s, bit-identical across Builds A & B) | `FbsSecant_Active` reduces `MarginHinge` loss by $\Delta L = +0.067059$ (seed 1) and $+0.104003$ (seed 2) against `FbsZeroCredit_Null` ($\Delta L = 0.000000$) and `FbsRandomRefit_Null` ($\Delta L = +0.023534$ seed 1, $-0.019526$ seed 2), beating the best null control on both seeds by $+0.043525$ (seed 1) and $+0.104003$ (seed 2) against $\Delta_{\text{noise}} = 0.043060$ ($1.011\times$ and $2.415\times$ noise band), while crossing discrete stage roots on `5.59%` (`200/3577`) and `5.37%` (`192/3573`) of queries and flipping output Clifford routes $A_o$ on `9.79%` (`155/1584`) and `9.60%` (`152/1584`) of outputs (`I-1..3` PASS). Scope: 256-step offline float surrogate descent gate only; not a full-rung fit or D11 integer export | positive | [record](../labs/r-secant-surrogate-descent/README.md) |
| 26 Oct | The pre-registered full-rung offline float sheet (`spiralcore-fbs-offline-float-sheet`, `#2891`, [sheet](../harness/prep/spiralcore-fbs-offline-float-sheet-2026-10-25.md)): 4 arms (one atom, two-atom mix, identity transport, frozen queries) × 2 seeds × 1,024 steps on the rung-0 Hamilton stack, one RTX 5090, two byte-identical builds | Training ran at 20,643–23,929 tok/s, float DEV evaluation at step 1,024 at 127,123–134,225 tok/s and greedy decode at 109–140 tok/s (a full-recompute lower bound). Float DEV loss fell from 2.65–2.85 to 1.80–1.93 on every arm. Learned routing lost to frozen random routing on paired float top-1 in 8 of 8 comparisons, by 1,851–5,098 rows. Identity transport had the lowest loss in all 4 seed-build cells. The one-atom arm's integer export kept the loss gap inside the band but failed the top-1 bar. One atom was retained over two. Trained-query top-1 did not reproduce between builds: it moved by up to 2,212 rows, against at most 280 for the frozen null. `I-1..4` PASS, with route mobility 5.46%–7.95% | negative | [record](../labs/spiralcore-fbs-offline-float-sheet/README.md) |

**Reading these honestly.** Before 25 October the float form had only correctness evidence: exact fixtures and finite controls, with zero optimizer updates. The 25 October row establishes that the root-edge secant surrogate (`spiralcore_fbs_bwd`) crosses discrete $E_8$ root boundaries (`5.37%–5.59%` of stage queries, `9.60%–9.79%` output route flips) and yields genuine descent beating both zero-credit and norm-matched Rademacher random-refit controls beyond the two-seed noise band across two seeds (`G-Fbs-Descent = PASS`). The weak recall in the 12 October tree row was a readout failure, since the tree kept the source (hit rate 0.99 on seed 1) while the table readout failed to use it (recall 0.03). The 14 October pilot rise in loss traced to the initializer, not to the transport. The 26 October full-rung row is a negative for one configuration only (1,024 steps, `StackAdamW` at the Hamilton sheet's rates). Its trained-query arms did not reproduce between two byte-identical builds. A hypothesis that fits this: atomically accumulated adjoints flip hard root choices, so a stable comparison of trained routes needs a deterministic backward first.

## What has never been tested

- **A reproducible comparison of trained routes.** The 26 October trained-query arms moved by up to 2,212 top-1 rows between two byte-identical builds.
- **Capacity of one atom per block** against an all-input mixing projection. A two-atom routed mix was compared on 26 October, and one atom was retained. No all-input comparison was run.
- **Input-conditioned behaviour.** Selection by shared learned queries does not by itself make the read depend on the input.
- **Coverage of all 240 E8 roots.** Only one 16-root anchor orbit is used, with no independent partner space and not the complete external tree object.
- **Tokens per second or energy** against the retired dense arm, and a full integer table export of the float stack. The 26 October export mode rounds only the writer table and the final hidden state, and snaps the Hamilton gates.

## What would reopen it

The float form is READY, so these are the steps to make it ACTIVE.

1. ~~A new D37 sheet in a separately scanned float module (`#2891`, now unblocked by `#2886`), with a fresh test-fitness review, pre-run review, an export gap, and the shared writer and geometric head as dependencies.~~ Pre-registered and cleared (`TEST FITNESS: FIT`, D68 `APPROVE`) in [`docs/harness/prep/spiralcore-fbs-offline-float-sheet-2026-10-25.md`](../harness/prep/spiralcore-fbs-offline-float-sheet-2026-10-25.md) (`antigravity-81`, `#2891`). Run on 26 October (`claude-119`, [record](../labs/spiralcore-fbs-offline-float-sheet/README.md)): `G-Fbs-Sheet-Fit` and `G-Fbs-ExportGap` failed for that configuration.
2. ~~Show that the secant credit gives descent (a falling loss, or a comparison with a random-refit control) and that routing is stable.~~ Answered at 256-step micro-probe scale in `#2886` (`G-Fbs-Descent = PASS`, [record](../labs/r-secant-surrogate-descent/README.md)).
3. Justify the total number of 8-dimensional blocks in advance, and record that each output block reads one atom, so a capacity comparison against a mixing projection is needed.
4. Fresh transfer and D22 noise requirements for any KEEP, and tokens per second on the same GPU harness against the retired dense arm.

Open questions: does one atom per output block lose too much against the all-pair sum, and can a small routed set per block recover it; can the secant credit be compared with the discrete counterfactual-and-choose rule on the same carrier; how many 8D blocks are enough (the 6,560-atom bound is operational, not a quality bar, and no 384-dimensional dependency is adopted); and would a float trunk add anything beyond the integer path that already runs. These are **hypotheses**.

## Also known as

`projection=spiralcore_fbs`; SpiralCore BFS plus Atomic Ternary FBS; `quaternion_weights=spiralcore_fbs`; "OP2 native projection" (PR #2279); SpiralCore tree; the HoloBox/FBS base-3 tree (the retention tree).

## Code

- [`spiralcore_fbs.rs`](../../crates/uor-r4-training/src/spiralcore_fbs.rs): the static compiler (binders, histories, actions, inverses, artifact hashes) and the custom operation, schema `spiralcore-fbs-projection/1`.
- [`spiralcore_fbs.cu`](../../crates/uor-r4-training/src/spiralcore_fbs.cu): GPU forward, input adjoint and secant kernels.
- [`spiralcore_operator.rs`](../../crates/uor-r4-core/src/spiralcore_operator.rs): exact signed 8×8 matrices, the Clifford generators and the finite composition table.
- [`native_vector_writer.rs`](../../crates/uor-r4-training/src/native_vector_writer.rs): the shared vector writer fed by the projection.
- [`discrete.cu`](../../crates/uor-r4-discrete/src/discrete.cu) and [`build.rs`](../../crates/uor-r4-discrete/build.rs): the ACTIVE integer descendant and the exact 64-state geometry tables it imports.
- [`m6_exact_arithmetic.rs`](../../crates/uor-r4-integer/src/m6_exact_arithmetic.rs): M6 exact multi-digit arithmetic on the 16-root $E_8$ anchor orbit, 64-state $\mathrm{Cl}(0,6)$ `BFS`/`COMPOSE` actions, ternary carry/borrow trits, and `IntegerAddressedMemory` ([lab record](../labs/m6-exact-arithmetic-mechanism/README.md)).
- The dense source this replaced is preserved as an indexed patch: `archive/history/branch-archive/INDEX.md`.

Audit source: `docs/labs/repo-renormalization-2026-10-19/audit/mechanisms/spiralcore-fbs.json`.

## Related mechanisms

[discrete-learning](./discrete-learning.md): the integer form of the same transport is the ACTIVE model path. [flock-read](./flock-read.md): flock reads transport payloads through the same actions, with the starling tree as the retention layer. [spin-weights](./spin-weights.md): this projection replaced the dense quaternion contraction. [hamilton-gate](./hamilton-gate.md): the feed-forward partner of this projection. [recurrence-gates](./recurrence-gates.md): shares the secant credit pattern. [route-read](./route-read.md): the routed read selects ports by comparing the transported query with fixed branch roots. [e8-orbit-memory](./e8-orbit-memory.md): the same E8 root orbit used as exact addresses. [ternary-xor](./ternary-xor.md): the ternary binder tree and trit gates.
