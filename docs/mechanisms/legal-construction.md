# Protected legal construction

**Status: PARKED.** No legal move was ever scored on the real test panel. Every attempt either broke its own safety check after rounding or stopped on solver arithmetic. The line was archived by owner decision D21 and reopened by D22, and its goal now lives on in [discrete learning](./discrete-learning.md) ([direct construction record](../../archive/labs/direct-legal-construction-2026-10-09/README.md)).

Labels used below: **proof** means derived from definitions or code, **measurement** means a recorded run, **hypothesis** means an untested idea. See [`docs/formal_vocabulary.md`](../formal_vocabulary.md).

## In plain words

The usual way to train a model for a small device is to train with smooth decimals and round at the end. Rounding can quietly undo what was learned. This idea skips that. Each parameter change is chosen straight from the short list of values the final integer model can hold, so every accepted step is already in its servable form. A checker also guarantees that a set of answers the model already gets right stays right. Choosing all the changes at once is a large puzzle for a solver. The attempts that rounded a smooth step broke the safety check, and the attempts that solved the puzzle directly hit arithmetic trouble before producing a single move.

## What it replaces in a transformer

The optimiser plus the rounding step that follows it: train in decimals, then quantise, then check by hand what broke.

![A number line for one coefficient. The unrounded training value is a dot. Fifteen evenly spaced ticks mark the legal quarter-step values from minus 7 over 4 to plus 7 over 4. A smooth step moves the dot to a new place, and rounding lands it on the nearest tick. A small box of protected answers beside the line turns red when the landing tick is on the wrong side of a guard. A second panel shows a grid of 380 guard rows against four step sizes, with many red cells after rounding and none for the unrounded step.](../figures/mechanisms/legal-construction.svg)

*Why rounding breaks protection: a step that passes every guard in smooth form can fail some of them once it lands on the legal grid.*

## How it works

**Setting (definition).** The older learner for milestone M2 (answering from memory; see the [milestones](../start/glossary.md)) holds 1,920 coefficients as decimal masters, in two groups of 960 (called Prefix and Generate). They are served as quarter-step codes $q/4$ with $q\in\{-7,\dots,7\}$, which gives 15 legal destinations per coefficient.

**Protected transaction (definition).** A change $\delta$ commits only if three things hold: the native scoring strictly lowers the pooled loss; 17 reference cases and 380 original winners still win; and the linearised guard test holds, $J\delta\ge-10^{-10}\lVert\delta\rVert$. Here each row of $J$ is the slope of one protected answer's margin.

**Two ways to build $\delta$ (definition).**

1. *Residual feedback.* Take a smooth, protected direction and round it onto the grid over 32 feedback rounds.
2. *Direct construction.* Solve a mixed-integer program. For each coefficient with unrounded value $m$ and current code $q_0$, binaries $l$ and $r$ pick "keep", "go lower" or "go higher":

$$l+r\le1,\quad q\ge q_0+r-(q_0+7)\,l,\quad q\le q_0-l+(7-q_0)\,r,\quad \delta=\tfrac{q-q_0}{4}+(l+r)\Big(\tfrac{q_0}{4}-m\Big).$$

Minimise the original gradient $g\cdot\delta$ subject to the 380 unit-normalised guard rows $\hat J\delta\ge0$. That is 1,960 variables, 440 constraints and a 4,096-node branch-and-bound cap, run offline in a pure-Rust solver (`microlp`).

**Numerical repair (definition).** Replace the solver's fixed pivot threshold by a scale-aware one, $\tau_j=32\,n\,u\,\max_i|a_{ij}|$ (with $n$ the matrix size and $u$ the unit round-off), and check the residual of every fresh factorisation.

**Consequence (proof, from the definition).** Any committed move is a legal code and, by the commit rule, strictly lowers the loss. Whether the guards also survive is not guaranteed when a smooth move is rounded afterwards. The rounding runs below measure that.

**Rule check.** Aligned with D11: every accepted step is already in the servable code set. The solver is offline float arithmetic and is not a serving part. The pooled-ranking loss behind the M2 learner used softmax and softplus forms that D23 bans, so a rerun needs a compliant objective. Not yet assessed against D30 (the guard test forms a 380-by-1,960 Jacobian product and the solver does dense LU), so a rerun would have to be classified or run on the GPU. The float solver was frozen by D32 and can return under D37 as an offline float sheet with a fresh review, an owner release, a separately scanned module and a reported export gap.

## What has been tried

"Conditional" counts correct positions inside one fixed 15-position training episode. "8 of 512" is the accepted model's complete correct replies on the 512-row development panel, which none of these runs changed. All records are from 9 October 2026 unless stated.

| What was run | Result | Verdict | Record |
|---|---|---|---|
| Differentiable margin so protected answers contribute gradient | Arithmetic tests pass; no model benefit claimed; accepted model stays 8 of 512 | inconclusive | [protected margin](../../archive/labs/protected-margin-credit-2026-10-09/README.md) |
| Protected joint direction, rounded to the grid at four radii | The smooth direction passes protection; every rounded version violates it: 104, 170, 98 and 93 of 380 constraints at radii 1, 2, 4 and 7 | negative (that rounding rule) | [joint construction](../../archive/labs/protected-joint-construction-2026-10-09/README.md) |
| Attribution of those four failures | Rounding accounts for roughly 90 to 96% of the adverse pressure (analysis of saved arithmetic) | inconclusive | [attribution](../../archive/labs/quantized-protection-attribution-2026-10-09/README.md) |
| Discrete feedback, 32 rounds, 15 codes per coefficient | 28 distinct destinations, none eligible, 52 to 218 violated rows per round; no proposal or commit | negative (that constructor) | [discrete feedback](../../archive/labs/protected-discrete-feedback-2026-10-09/README.md), [attribution](../../archive/labs/discrete-feedback-attribution-2026-10-09/README.md) |
| Coordinated move along the full gradient at four radii | All four lower the combined loss; none passes the unchanged protections. Conditional 6 of 15 against 5 of 15 | negative | [prefix vector](../../archive/labs/prefix-vector-run-2026-10-09/README.md) |
| Full donor run and exporter fix, 1,920 coefficients | 956 changes accepted of 14,292 alternatives; conditional 9 of 15, as the older negative; not promoted | negative | [full donor](../../archive/labs/full-donor-run-2026-10-09/README.md), [export recovery](../../archive/labs/coupled-export-recovery-2026-10-09/README.md) |
| Mathematical bound on legal pair changes at the first error | The target's best case sits below the rival's worst case at all nine reference points, so pair-only changes cannot fix it | negative (pair-only, one frame) | [pair-copy bound](../../archive/labs/pair-copy-bound-2026-10-09/README.md) |
| Direct mixed-integer solve, both encodings | Both stop with a singular-matrix error before any proposal | blocked (tooling) | [direct construction](../../archive/labs/direct-legal-construction-2026-10-09/README.md) |
| Replay of the failing solver bases, then the scale-aware repair | A nonzero pivot just under the threshold, not an empty column; after the repair both captured bases solve accurately, but a later singular matrix returns no assignment (see note) | inconclusive | [basis diagnosis](../../archive/labs/legal-basis-diagnosis-2026-10-09/README.md), [residual repair](../../archive/labs/basis-residual-repair-2026-10-09/README.md) |
| 2026-10-10: full current-parent constructor on the 512-row objective | Interrupted after 102.9 minutes for a session transfer; no assignment; recorded as unfinished, not negative | blocked | [D22 constructor](../../archive/labs/d22-constructor-2026-10-10/README.md) |
| 2026-10-19: discrete analogue (block apply on fixtures) | All qualifier controls pass; a block commits only if its joint loss strictly beats the best single move; 3,419 fixture updates | positive (fixtures only) | [C2 qualifier](../labs/injective-preflight-and-c2-qualify-2026-10-19/README.md) |

**Note on the basis row.** The failing pivot was about $9\times10^{-11}$ against a threshold of $10^{-10}$. After the repair the record counts 150 factorisations, 148 completed original-basis verifications and two later pivot rejections.

**What the negatives say.** The direct construction never produced a move, so the mechanism was never judged: a numerical blocker is not a negative (D22 section 1, [full text](../../archive/integration/DECISIONS-2026-10-19.md#d22--a-negative-closes-a-configuration-never-a-mechanism-five-closures-reopened)). The rounding runs are negatives for those exact rounding rules and radii. The 9-of-15 runs moved only Prefix and Generate coefficients, with the other tables frozen. The pair bound covers pair-only changes at one frame.

## What has never been tested

- Any legal displacement scored on the 512-row panel or the 128 fresh rows.
- The constructor in exact rational or $\mathbb{Z}[\varphi]$-integer arithmetic.
- A float-trained-then-quantised control at matched compute, over at least two seeds, on the same learner.
- A greedy legal coordinate search as a cheap baseline before the full solver.
- Splitting coefficients into groups to shrink the problem, or relaxing the 380 guards to penalties.
- Pair credit combined with a single-coefficient move at the same frame.
- Legal-by-construction moves with an admission-preserving acceptance rule on the discrete model.

## What would reopen it

1. Solve the saved hard instance (called `input245`: 1,960 variables, 440 constraints, 15 legal destinations) in exact rational or $\mathbb{Z}[\varphi]$-integer arithmetic, or with a numerically robust solver, so the singular-pivot blocker is gone.
2. Score at least one legal displacement on the 512-row panel and the 128 fresh rows, over at least two seeds, against a float-trained-then-quantised control at matched compute (the control comes from the mechanism brief; the power rule is D22 section 3 and the reopen order is 5(a), [full text](../../archive/integration/DECISIONS-2026-10-19.md#d22--a-negative-closes-a-configuration-never-a-mechanism-five-closures-reopened)).
3. Run a greedy legal coordinate search first, as the cheap baseline.
4. Or re-register the idea as an offline float sheet under D37: fresh test-fitness review, owner release of the M2 training hold, a separately scanned module and a reported export gap.
5. Or carry the legal-by-construction idea on the discrete path, with the block-apply rule plus an admission-preserving acceptance rule, under its own review.

## Also known as

Protected or discrete legal construction; direct joint legal-set construction; the protected Prefix/Generate transaction; quantised-form parameter moves; the exact legal-constructor solver (mixed-integer program); the D22 constructor.

## Code

- [`protected_legal_set.rs`](../../crates/uor-r4-training/examples/geometric_frozen_map_fit/protected_legal_set.rs): the direct joint constructor. [`protected_joint_vector.rs`](../../crates/uor-r4-training/examples/geometric_frozen_map_fit/protected_joint_vector.rs), [`protected_credit_import.rs`](../../crates/uor-r4-training/examples/geometric_frozen_map_fit/protected_credit_import.rs), [`gradient_vector_prefix.rs`](../../crates/uor-r4-training/examples/geometric_frozen_map_fit/gradient_vector_prefix.rs): the rounded-direction forms.
- Unfinished full constructor and the pivot repair: inactive patches indexed in [`archive/history/branch-archive/INDEX.md`](../../archive/history/branch-archive/INDEX.md) (`d22-constructor-unfinished-20261010.patch`, `basis-residual-repair-20261009.patch`).
- [`bvgk_source.rs`](../../crates/uor-r4-discrete/src/bvgk_source.rs): the discrete analogue, a block apply committed only if the joint loss beats the best single move.

## Related mechanisms

[discrete-learning](./discrete-learning.md) reaches the same goal differently: every move is legal by construction. [carrier-credit](./carrier-credit.md) ran on the same learner and panels but changed where credit goes, not which moves are legal. [rank-margin](./rank-margin.md) supplies the descent target in its integer form. [spin-weights](./spin-weights.md) is the source of the quarter-step code set. [ternary-xor](./ternary-xor.md) is another multiplier-free weight alphabet with the same export question.
