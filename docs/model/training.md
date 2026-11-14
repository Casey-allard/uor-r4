# How the model learns

The discrete model learns by trying every legal geometric move of a table entry and keeping a move
only when the integer loss strictly improves. The method is exact and runs on the GPU. **It has now
produced a saved model that passes M1 on two seeds.** Earlier failures remain useful negative
evidence, not the current best.

<!-- native-space-contract: Native default: one persistent native geometric state uses one declared integer or fixed-point representation for token addresses, vector coordinates, carry, routes, memory keys and reads, learned state, gradients and optimizer updates; training and serving use the same typed operators; route choices, carry state, read choices, memory addresses and token ids are bit-identical on the same prompt and starting state; the artifact losslessly serializes that state instead of an F32 master model or an export gap; floating point remains reference or diagnostic code only. -->

**D85 status:** the persistent native state/operator implementation is pending; all float-to-integer measurements on this page are historical/export evidence.

**D85 changes the default route.** The model to train next carries one persistent native
integer/fixed-point geometric state through weights, activations, SpiralCore carry, routes,
memory, gradients and optimizer updates. Training and serving must call the same typed
operators and replay the same route, carry, read and token decisions bit for bit. The FBS path
below is retained as historical/reference work; an F32 master plus export gap is not the
default model boundary.

![The discrete learning loop: score every legal move, choose the lowest integer loss, commit only a strict gain, then verify the stored result. A side panel shows BVG-K scoring several coordinates per launch.](../figures/model/discrete-learning-loop.svg)

*Discrete learning: exact counterfactual moves, strict acceptance and verification on the GPU.*

Labels follow [`docs/formal_vocabulary.md`](../formal_vocabulary.md): **proof**, **measurement**,
**hypothesis**. The [discrete-learning mechanism page](../mechanisms/discrete-learning.md) carries
the longer history.

## The discrete learning step

A *vector block* is one of four 8-number blocks in a token's vector. A *route frame* is one of 64
signed-axis rotations. A *gate* takes values −1, 0 or +1. A vector block has 16 legal states,
including "stay as it is" ([architecture](./architecture.md)).

For each candidate move $m$, the machine replays the read and writer and computes the integer
rank-margin loss:

$$L(m)=\sum_{x\in\mathcal R}\ell\big(x;\ \theta\leftarrow m\big),\qquad m\in\{\mathrm{id},m_1,\dots,m_{15}\}.$$

It chooses the lowest loss, with ties going to identity and then deterministic move order. It
applies the winner only if $L(m^*)<L(\mathrm{id})$, and re-checks the stored table. Every accepted
move is therefore legal and strictly improves its scored set; that alone says nothing about other
rows or fresh transfer.

One 16,404-batch pass scores 1,049,856 targets out of 2,425,925, or 43.3% of the training split
(**proof by arithmetic**). This is why a batch improvement is not by itself a language result.

## Row-parallel and BVG-K execution

The ordinary counterfactual evaluates 16 lanes in one block. The row-parallel form evaluates 64
rows × 16 lanes as 1,024 threads and reproduces the ordinary loss on 1,085 finite cases
([rows parity](../labs/counterfactual-rows-parity-2026-10-18/README.md)). On one RTX 5090, its
non-gating diagnostic was about 56 times faster.

**BVG-K (batched verified coordinates)** scores several coordinates per launch and commits the
block only if its joint loss beats the best single move. BVG-K is not merely a construction test:
Stage T training (`#2836`) and the later 19-table joint-pointer Hamilton trunk are part of the M1
lineage. The exact export then reproduces the saved discrete model on all 14,287 DEV rows.

## The result that closed M1

The saved 19-table `uor-r4.lut-discrete-trunk/1` model scores:

- DEV **7,002 / 7,001** against unigram 1,226 and T2 6,300;
- Fresh V2 **7,215 / 7,228** against unigram 1,279 and T2 6,582;
- p-f **7,850 / 7,872** against unigram 1,409 and T2 7,375, beyond paired band 81;
- Apple M1 decode **10,876 / 10,566 tok/s** under the bar-f protocol.

The saved model reads new prompt phrasings into known reply kinds. It does not show new reply kinds
or useful conversation; 62.5% of p-f's reference replies are verbatim TRAIN replies
([M1 close](../labs/m1-close-saved-model/README.md), [Fresh V2](../labs/m1-fresh-panel-transfer/README.md), [bar-f report](../loop/tasks/m1-heldout-phrasing-transfer/claude-211/REPORT.md)).

## Historical offline float training under D37

Before D85, the separate FBS trunk used float forward passes, gradients and an optimizer offline, in its own
scanned module. Softmax, squashing and dense matmul remain banned. Every float candidate must report
its export gap on the same evaluator, and the shipped artifact must be integer.

Since claude-179, the FBS backward uses a fixed-order reduction, so two runs give the same model
byte for byte. The `lut-stack/5` export is inside the gap bar at DEV scope. Determinism is not a
quality claim: on new phrasing, the model still starts replies poorly
([claude-217](../loop/tasks/one-model-end-to-end/claude-217/REPORT.md)).

The native-space item [`native-space-state-contract`](../roadmap/items/native-space-state-contract.md)
now starts from the state and operator inventory. No new default-path training run may treat
this historical float route as its model architecture.

## Pre-registration and runs

Before GPU training or evaluation, and before any run a milestone claim rests on, a one-page sheet
states the bar, noise, controls, seeds, cost and what each outcome decides. A non-author reviews it
for `TEST FITNESS: FIT`; a fresh-context subagent counts. Builds, finite kernel tests and host
checks need no sheet.

Since D67, a GPU or training run goes ahead on the lab's own whole-project pre-run review inside
the caps. There is no blanket owner release for every run; D41's donor pass is the named exception
that still needs the owner at dispatch. Pods go through `scripts/pod/uor-pod`, with at most 4 pods
and $8/h across all labs, and a healthy run is not cancelled for a self-set budget.

## Historical campaigns and what they taught

| Date | Run | Result and exact scope | Verdict |
|---|---|---|---|
| 14 Oct | Pilot, 1,640 batches | Top-8 173→1,169 and top-1 17→82, while total loss rose | inconclusive |
| 15 Oct | Stage A sweep, K=8 | 1,920 admitted targets against a 4,894 gate | negative configuration |
| 18 Oct | Relation/control to Q1 | Copy-successor admission fell to 225/205 of 3,847 | diagnostic stop |
| 19 Oct | K16, two objectives/two seeds | A scored 65 and 463; R collapsed to 19 or 20 cells | negative configuration |
| 19 Oct | A8 dose | Relation admission 1,351→551; control 857 | diagnostic stop |
| 25–26 Oct | Stage T four-table exports | Exact export at four checkpoints, before the M1 closing pair | positive at Stage T scope |
| 30 Oct | 19-table M1 export and close | Bars (a)–(e) at two seeds: DEV 7,002 / 7,001 and Fresh V2 7,215 / 7,228, with exact 14,287-row DEV export agreement ([M1 close](../labs/m1-close-saved-model/README.md), [Fresh V2](../labs/m1-fresh-panel-transfer/README.md)) | M1 bars (a)–(e) complete |
| 30 Oct | p-f bar (f) | 7,850 / 7,872 against T2 7,375, band 81 | M1 complete |

A negative closes a configuration, never a mechanism. These records are preserved because they show
which instruments and configurations failed before the later model passed.

## What is still open

- **Reply opening on new phrasing.** Claude-217 measures teacher-forced reply openings of 17.4%, 13.7% and 18.6% on p-s, against
  about 80% on later tokens. Exact-key memory misses reworded prompts.
- **Independent mechanism effects.** M1 is a whole-model result, not an ablation proving each read,
  gate, route or memory mechanism independently.
- **Complete generation and M2/M3.** Next-token ranking is not complete correct replies or durable memory.
- **Cost and scale.** Whole-path training and serving costs, model scaling and M5 energy are not
  established by one passing artifact or by arithmetic projections.

## Go deeper

- [Architecture](./architecture.md) and [serving and export](./serving-and-export.md).
- Mechanisms: [discrete learning](../mechanisms/discrete-learning.md), [rank margin](../mechanisms/rank-margin.md), [admission](../mechanisms/admission.md), [legal construction](../mechanisms/legal-construction.md).
- Code: [`device.rs`](../../crates/uor-r4-discrete/src/device.rs), [`training.rs`](../../crates/uor-r4-discrete/src/training.rs), [`rows_source.rs`](../../crates/uor-r4-discrete/src/rows_source.rs), [`bvgk_source.rs`](../../crates/uor-r4-discrete/src/bvgk_source.rs).
- Records: [M1 close](../labs/m1-close-saved-model/README.md), [bar-f](../loop/tasks/m1-heldout-phrasing-transfer/claude-211/REPORT.md), and [claude-217](../loop/tasks/one-model-end-to-end/claude-217/REPORT.md).
