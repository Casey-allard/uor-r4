# Read binding (supervised value binding)

**Status: PARKED.** In the one form tested, it met its own target and moved the memory score by a pooled 8 of 120 rows over three seeds (one seed was down 1 row, and only one seed was individually significant). But it was applied to a softmax attention read, which the project now bans, and it has never been tried on any read that is allowed. It stays open.

Labels used below: **proof** means derived from definitions or code, **measurement** means a recorded run, **hypothesis** means an untested idea. See [`docs/formal_vocabulary.md`](../formal_vocabulary.md).

## In plain words

Ask a model "Ana's locker is 47. Ben's locker is 12. What is Ana's locker?" A common failure is to find a stated number but pick the wrong one. During training, a teacher marks which earlier words hold the value the question asks for, and which hold the other stated values. The model is rewarded when its attention lands on the right ones. Nothing changes when the model is used: the labels only shape what the attention learns. It aims at one failure, "found a value, but the wrong one".

## What it replaces in a transformer

It adds to the loss, the training score. A transformer is usually trained only to predict the next word, and nothing tells its attention which earlier word to look at. This adds a second target on the attention read itself. It replaces no layer and adds no operation at serving time.

![Attention bars over the passage about Ana's and Ben's lockers. Before training the attention is split between 47 and 12, with bound mass 0.73 and competing mass 0.14. After training it piles onto 47, at 0.90. Beside it, a paired comparison of wrong-value counts over 120 memory rows, and a bar chart of the memory score when each read layer is silenced in turn.](../figures/mechanisms/read-binding.svg)

*Left, attention before and after supervision. Right, the wrong-value count falls, and silencing the supervised layer removes the answers.*

## How it works

For a question whose next word belongs to a stated value, the training data carries two sets of earlier positions. $B$ is the set that holds the asked value (bound). $C$ is the set that holds the other stated values (competing). The labels come from a sidecar file written with the dialogue corpus. Each attention head $h$ puts weight $a^{(h)}_j$ on earlier position $j$. Define the bound and competing mass:

$$m^{(h)}_{\text{bound}}=\sum_{j\in B}a^{(h)}_j,\qquad m^{(h)}_{\text{comp}}=\sum_{j\in C}a^{(h)}_j .$$

The extra loss, on the best-binding head, is

$$\mathcal{L}_{\text{bind}}=-\log\!\Big(\max_h m^{(h)}_{\text{bound}}+10^{-6}\Big),\qquad \mathcal{L}=\mathcal{L}_{\text{language}}+W\,\mathcal{L}_{\text{bind}} .$$

The floor $10^{-6}$ keeps the logarithm finite. The weight $W$ was 0.1 in the run kept and 0.5 in the first test. The loss is zero only when all attention sits on the bound positions (ignoring the floor). That follows from the formula (proof). One variant also penalised competing mass with $0.5\,\max(0,\,m_{\text{comp}}-0.03)$.

The labels never touch a hidden state or a score for the next word. The served network is unchanged, and only the read learns to separate values. The tested runs applied it to a late read layer (layer 8, in a stack whose reads are layers 2, 5 and 8) from step 0 of a fine-tune.

An exact-by-construction variant (`read_lineage = phase_bind`) gives each value a fixed phase tag, so binding is exact rather than learned. A random rotation tag is its control.

**Rule check.**

| Rule | Verdict |
|---|---|
| No softmax (D23) | Broken in the tested form. The loss has no softmax, but the mass $m$ comes from a softmax attention read ([attention-state review](../labs/attention-state-2026-10-18/README.md): "it is a softmax read") |
| No matrix multiplication (D30) | Not clean. The 29M-parameter stack that was trained is a dense matrix-multiplication stack |
| Integer-only serving (D11) | Nothing to serve. It is a training signal and adds no served operation |
| Discrete path (D32) and offline float (D37) | Not on the discrete path. Float training is legal offline under [D37](../integration/DECISIONS.md#d37--floating-point-is-legal-in-offline-training-only-d32s-learning-path-float-ban-is-revised), so the signal is admissible if attached to a lawful read and exported to an integer artifact |

## What has been tried

"Wrong-value" counts rows where the model answered with a stated value that was not the asked one. The memory panel has 40 rows per seed; "of 120" pools three seeds.

| Date | What was run | Result | Verdict | Record |
|---|---|---|---|---|
| Sep (undated) | Optimizer-clock correction on a copied attention cell (precursor) | Top-1 rose from 12.01% to 68.29% with the training recipe alone, against a strict bar above 99%. A later copy scored 99.17%, so the failures came from the recipe | inconclusive | [record](../../archive/r4_zoology_optimizer_clock_1055.md) |
| 3 Oct | Eight surface forms per relation, no read labels | Unseen statement forms 37.8% to 94.6% (McNemar p = 3.1e-11). Unseen question forms 58.1% to 63.5% (p = 0.54). The leftover failure is specific to the question's form | positive | [record](../../archive/integration/surface-form-breadth-2026-10-03.md) |
| 9 Oct | Binding supervision 0.1 on a 214M-parameter float pointer stack (arm P) against arm Q | 29 of 40 against 31 of 40, target 34. The 9 misses split into wrong-value and no-value with no single fix | negative | [record](../../archive/labs/v4-memory-panel-reproduction-2026-10-09/README.md) |
| 10 Oct | More binding examples in the training data, no labels | Memory 17 of 40 against 20 of 40 at the same dose. Third flat cycle in a row | negative | [record](../../archive/labs/binding-dense-2026-10-10/README.md) |
| 10 Oct | $W$ = 0.1 and 0.5, late fine-tune, one seed each | Bound mass 0.732 to 0.896 at $W$=0.1. Memory 19 and 17 of 40 against an anchor of 20. Wrong-value 17 to 12. Hit its own target, left the panel flat | inconclusive | [record](../../archive/labs/read-binding-2026-10-10/README.md) |
| 11 Oct | Four serving-time tests on one model | Normal 23 of 40. Copy head off 22. Facts removed 0. Copy forced to half 7. Answers come from the attention read | positive | [record](../../archive/labs/answer-path-2026-10-11/README.md) |
| 11 Oct | **Keeper.** $W$ = 0.1 from step 0, 3 seeds, each against a matched $W$=0 control | Bound mass 0.732 to 0.896, 0.899, 0.905. Competing 0.140 to 0.090, 0.086, 0.081. **Wrong-value 47 to 30 of 120. Memory 57 to 65 of 120.** Best seed 24 against 20 of 40 (McNemar p = 0.0034; 12 repaired, 1 broken). Bits per byte 1.17142, 1.17067, 1.17060 against 1.17425 for the anchor | positive | [record](../../archive/labs/read-binding-powered-2026-10-11/README.md) |
| 11 Oct | Silence one read layer at serving time on the bound model | Unablated 24 of 40. Layer 8 silenced: 0 of 40 (37 rows give no value). Layer 5: 7. Layer 2: 2 | positive | [record](../../archive/labs/head-ablation-2026-10-11/README.md) |
| 11 Oct | Margin hinge on competing mass added | Competing mass halved (0.0462 against 0.0897) but wrong-value stayed at 30 and memory at 64 of 120 against 65 | negative | [record](../../archive/labs/read-binding-margin-2026-10-11/README.md) |
| 11 Oct | Phase binding (fixed phases) on top of $W$=0.1, two seeds | Same bound mass, worse answers: memory 17 against 19 and 20 against 24 of 40. Training 13.4 times slower (0.058 to 0.777 s per step) | negative | [record](../../archive/labs/phase-binding-2026-10-11/README.md) |
| 11 Oct | The keeper recipe at 8,000 steps (four times longer), 3 seeds | Wrong-value 49 to 23 of 120. Memory 48 to 64 of 120. Bound mass 0.969 to 0.971. Two seeds individually significant (p = 0.031, 0.035) | positive | [record](../../archive/labs/read-binding-scaled-2026-10-11/README.md) |
| 11 Oct | 16,000 steps, 2 seeds | Wrong-value 25 to 15 of 80. Memory 39 to 45 of 80. Bound mass 0.985. The advantage did not grow, so the training-length lever is closed | positive | [record](../../archive/labs/read-binding-16k-2026-10-11/README.md) |
| 12 Oct | Forced-copy oracle on the bound model | The instrument was built (919 library tests pass). The run was not done | blocked | [record](../../archive/labs/copy-span-oracle-2026-10-12/README.md) |
| 18 Oct | Independent read-only review of every attention mechanism | Calls this the one powered result in which a learned read moved a headline, in a form now inactive. Nothing in the integer substrate has it | inconclusive | [record](../labs/attention-state-2026-10-18/README.md) |

**Reading these honestly.** This is the clearest positive among the learned-read ideas, and also a narrow one. It was measured on a 40-row panel per seed, on fine-tunes of a single 29M-parameter model. The bound and competing masses are measured on training data. The headline gain is about 8 rows of 120. The first one-seed test (10 October) moved the panel by one row, inside the 2 to 3 row noise, and the project's rule on closing configurations ([D22](../integration/DECISIONS.md#d22--a-negative-closes-a-configuration-never-a-mechanism-five-closures-reopened)) reopened it. The failures close specific configurations: a late bolt-on, a hinge, fixed-phase tags, more steps and a data-only lever. They do not close the idea. Notably the masses kept sharpening (0.985) while the panel gain stopped growing. So bound mass may not be the whole lever (hypothesis).

## What has never been tested

- Any discrete or geometric read, such as a routed read, a flocking read or a relation read. Any read that is not a softmax attention.
- Supervising every head, or routing the output through the binding head. Both were named after the 16,000-step result.
- Pairing the binding with the pointer so that bound positions feed the copy distribution directly.
- Held-out bound and competing mass. The masses reported are train-side.
- At least 120 memory rows per seed. Every panel has 40.
- A model pretrained with the mechanism. Every run fine-tuned one 29M base.
- A fresh-draw transfer number for the kept result, which [D22](../integration/DECISIONS.md#d22--a-negative-closes-a-configuration-never-a-mechanism-five-closures-reopened) asks for.

## What would reopen it

1. **Re-register on an allowed read.** Use an offline float sheet (D37) on a read that is not a softmax attention (a rank-table, flock or relation read). It must be a separately scanned module with an integer export and a reported export gap, plus a fresh `TEST FITNESS: FIT` review and an owner release.
2. **Or port the signal to the discrete read.** Define the discrete analogue of bound against competing mass on a routed, flocking or relation read, and supervise it from the start of training.
3. **Meet the fair-test conditions.** Supervise from the start, not as a late bolt-on. Use at least 3 seeds, at least 120 memory rows, a matched $W=0$ control and held-out masses. Add a head-ablation check that the bound head drives the answer. The bar is a gain of at least 5 rows of 40 over the control.
4. **Change the supervision, not its length or sharpness.** Per-head supervision, or routing the output through the binding head, are the named candidates.

## Also known as

Read-binding supervision (`read_binding_supervision`); bound-and-competing-mass loss; phase binding (`read_lineage=phase_bind`), the exact-by-construction variant; teacher-only binding labels.

## Code

- [`geometric_stack.rs`](../../crates/uor-r4-training/src/geometric_stack.rs): `ReadBinding`, `ReadBindingTarget`, `ReadSupervisedLoss`, the $10^{-6}$ floor and read lineage.
- [`examples/geometric-stack.rs`](../../crates/uor-r4-training/examples/geometric-stack.rs): the training command that carries the `read_binding_supervision=W` and `read_lineage` flags.
- [`bin/dialogue-recall-corpus.rs`](../../crates/uor-r4-training/src/bin/dialogue-recall-corpus.rs): writes the binding-labels sidecar.
- [`bin/chat-grade.rs`](../../crates/uor-r4-training/src/bin/chat-grade.rs): the frozen-panel grader, with the read-silence switch used by the head ablation.

## Related mechanisms

[pointer-key](./pointer-key.md): the copy head, found not to be the answer path; binding acts on the read it bypasses. [exact-memory](./exact-memory.md): the exact-address alternative to learning which value a read binds to. [flock-read](./flock-read.md) and [relation-read](./relation-read.md): the softmax-free reads a re-registration would supervise. [route-read](./route-read.md): reads by invariant address, with no mass to supervise. [admission](./admission.md): the measured ceiling on what any read can reach, which supervision cannot raise.
