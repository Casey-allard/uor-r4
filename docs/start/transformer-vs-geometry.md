# Transformer parts and their geometric replacements

**In one paragraph.** A transformer is built from a handful of standard parts: a tokenizer, an embedding table, a position signal, attention, feed-forward layers, normalization, an output layer, a loss and an optimizer. UOR-R4 tries to replace each part with a geometric or exact mechanism that avoids softmax and matrix multiplication. This page puts each transformer part beside the project's replacement and says how far that replacement has got. The short version: most parts have a candidate, and several candidates are built and tested. **Two parts have no candidate at all: position and normalization.** No two replacements have yet been trained together into a model that beats simple word-count tables ([audit synthesis](../labs/repo-renormalization-2026-10-19/audit/synthesis.md), section 1).

![Two columns joined by arrows. The left column lists the parts of a transformer from input to output: tokenizer, embedding, position, attention, copy heads, context memory, feed-forward, gates, normalization, output head, loss, optimizer and runtime. The right column shows the UOR-R4 mechanism for each, coloured by status: green for ACTIVE, amber for READY, grey for PARKED. Two right-hand boxes, beside "position" and "normalization", are empty, drawn with dashed red outlines and labelled "no mechanism".](../figures/start/transformer-vs-geometry.svg)

*Each transformer part beside its replacement. Colour gives the status. The two dashed boxes are the gaps.*

Terms are in the [glossary](./glossary.md). Labels follow [`docs/formal_vocabulary.md`](../formal_vocabulary.md): **proof**, **measurement** or **hypothesis**.

## How to read the status

Every mechanism has exactly one of three statuses, set on the [mechanism board](../mechanisms/README.md):

- **ACTIVE**: on the current model path, `crates/uor-r4-discrete`.
- **READY**: built and tested, but never trained into a model.
- **PARKED**: the forms tried so far failed or broke a rule. The mechanism stays open, and its page lists what would reopen it.

There is no "retired" or "dead" status. A failed run closes one configuration, never a mechanism ([charter](../../CHARTER.md), section 4.1). Today the board has **10 ACTIVE, 4 READY and 13 PARKED** mechanisms.

A passing test shows that code does what it claims. It is not a trained capability, and a test marked `NOT_RUN` is not a pass.

## Part by part

| Transformer part | What it does | UOR-R4 replacement (status) | Where it stands |
|---|---|---|---|
| Tokenizer | Cuts text into numbered pieces | [tokenizer](../mechanisms/tokenizer.md) (ACTIVE) | 4,096 pieces, pinned in every corpus and artifact. No other vocabulary size has been compared |
| Embedding table | Gives each token a starting vector | [prime-zeta codebook](../mechanisms/prime-zeta-codebook.md) (ACTIVE), [address cells](../mechanisms/address-cells.md) (ACTIVE) | PrimeZeta gives 4,096 of 4,096 tokens distinct vectors (**measurement**, [record](../../archive/labs/prime-zeta-construction-2026-10-14/README.md)). Address cells raised zero-training reach from 127 to 1,351 of 3,847 rows; training then eroded it to 551, below the control's 857 (**measurement**, [pre-flight](../labs/address-cells-preflight-2026-10-19/README.md), [dose](../labs/a8-dose-2026-10-19/README.md)) |
| **Position signal** | Tells the model the order of tokens | **None** | **Gap 1.** See below |
| Attention: what to read | Picks which earlier tokens matter | [route read](../mechanisms/route-read.md), [flock read](../mechanisms/flock-read.md), [relation read](../mechanisms/relation-read.md), [admission](../mechanisms/admission.md) (all ACTIVE); [Hopf observation](../mechanisms/hopf-observation.md) (PARKED) | Flock passes its 6 of the 25 shared GPU construction checks (11 Route, 6 Flock, 8 driver) (**measurement**, [record](../../archive/labs/discrete-driver-cost-2026-10-13/README.md)). No read's own value has been measured: Route received 20 learning proposals in 16,404 batches ([attention state](../labs/attention-state-2026-10-18/README.md)) |
| Attention: mixing what was read | Combines the chosen tokens | [flock read](../mechanisms/flock-read.md) (ACTIVE) | Rotates a few nearby earlier tokens into one frame and moves toward their consensus |
| Copy heads | Copies a word seen earlier | [relation read](../mechanisms/relation-read.md) (ACTIVE), [pointer key](../mechanisms/pointer-key.md) (PARKED) | Relation passes 9 of 9 checks and trained twice; both runs stopped before a verdict |
| Context memory (key-value cache) | Stores every earlier position | [exact memory](../mechanisms/exact-memory.md), [VSA](../mechanisms/vsa.md), [read binding](../mechanisms/read-binding.md) (all PARKED) | Using an exact address *as* the read has never been built |
| Durable memory outside the model | Not a transformer part | [object store](../mechanisms/object-store.md) (READY) | 20 integration tests at delivery (60 after later work) and a conformance pass. Nothing uses it yet |
| Feed-forward layer | Processes each token on its own; most of the multiplication | [SpiralCore/FBS projection](../mechanisms/spiralcore-fbs.md), [Hamilton gate](../mechanisms/hamilton-gate.md) (READY); [E8 orbit memory](../mechanisms/e8-orbit-memory.md) (PARKED) | None has ever been trained. The current model has no feed-forward layer |
| Gates and activation functions | Squash signals with sigmoid, tanh and the like | [Hamilton gate](../mechanisms/hamilton-gate.md), [recurrence gates](../mechanisms/recurrence-gates.md) (READY) | Turn the gate into a rotation, or use tables of powers of two. 18 GPU construction checks pass; never trained |
| Recurrent mixing (in hybrids) | Carries a running state forward | [recurrence gates](../mechanisms/recurrence-gates.md) (READY), [geometric stack](../mechanisms/geometric-stack.md) (PARKED) | The old stack trained, but used softmax and matrix multiplication |
| Matrix arithmetic | Multiplies by learned grids | [D11 integer serving](../mechanisms/lut-serving.md) (ACTIVE), [SpiralCore/FBS](../mechanisms/spiralcore-fbs.md) (READY); [spin weights](../mechanisms/spin-weights.md), [ternary/XOR](../mechanisms/ternary-xor.md) (PARKED) | Integer engine's audits are green, but no model built under today's rules has been exported into it |
| **Normalization** | Rescales signals between layers | **None** | **Gap 2.** See below |
| Output head (softmax over all words) | Scores every word and picks one | [admission](../mechanisms/admission.md), [discrete learning](../mechanisms/discrete-learning.md) (ACTIVE) | Best trained result: 65 and 463 of 14,287 correct, against 1,226 and 6,300 for two count tables (**measurement**, [K16 outcome](../labs/k16-outcome-call-2026-10-19/README.md)) |
| Loss (cross-entropy) | Measures how wrong a guess was | [rank-margin hinge](../mechanisms/rank-margin.md) (ACTIVE); [read binding](../mechanisms/read-binding.md) (PARKED) | Asks only whether the right word beats its strongest rivals by a gap. It gives no probability, so no bits-per-byte score |
| Optimizer (backpropagation) | Adjusts the learned numbers | [discrete learning](../mechanisms/discrete-learning.md) (ACTIVE); [carrier credit](../mechanisms/carrier-credit.md), [legal construction](../mechanisms/legal-construction.md) (PARKED) | Tries every legal move for one entry and keeps it only if the score strictly improves. A full run accepts about 130 to 410 moves |
| Runtime | Runs the finished model | [D11 integer serving](../mechanisms/lut-serving.md) (ACTIVE); [R4G1 graph](../mechanisms/r4g1-graph.md) (PARKED) | Runs in a browser with no multiply instruction in 7 of 7 serving functions (**measurement**, [record](../labs/wasm-serving-runtime-2026-10-18/README.md)) |

Sources: the [mechanism board](../mechanisms/README.md) slot map and the audit's stitching map ([synthesis](../labs/repo-renormalization-2026-10-19/audit/synthesis.md), section 1), which cite each record. The move counts are from the [K16 outcome](../labs/k16-outcome-call-2026-10-19/README.md) (130, 129, 407 and 410 accepted moves).

One status is disputed. The board keeps D11 integer serving as ACTIVE, as its own page says, but the audit would call it READY, because no lawful model has been exported through it yet ([mechanism board](../mechanisms/README.md), contradictions).

## The two gaps

**Gap 1: position.** A transformer adds a signal that says where each token sits in the sequence. No UOR-R4 mechanism is assigned to this job, and no lawful evidence exists for one. The current model keeps the most recent tokens in order at the leaves of a small tree, so order is present in its layout, but nothing is learned about it ([the model page](../model/architecture.md)). The nearest result is about carrying a running state by quaternion rotation. It beat no rotation by 0.071 and 0.077 nats (a nat is a unit of prediction error; lower is better), but in a model that still used softmax attention, a sigmoid gate and dense layers, with 332,928 fewer parameters in the comparison arm ([transport record](../../archive/integration/transport-attribution-d1-2026-09-28.md)). So "rotation can carry order" is a **hypothesis**.

**Gap 2: normalization.** A transformer rescales its signals between layers so they stay in a workable range. UOR-R4 has no designed mechanism for this. Snapping each block back to one of its fixed lattice points might play that role, but no record says so or tests it ([mechanism board](../mechanisms/README.md)). This is also a **hypothesis**.

## What "replaces" does not mean yet

Having a replacement on paper is not the same as having a working model:

- **No replacement has matched its transformer part on quality.** Every attention-style read that learned well in the older decimal experiments had help from softmax during training, and the best softmax-free read was rejected ([softmax-free read](../../archive/labs/softmax-free-read-2026-10-10/README.md); audit synthesis (`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md`), section 1).
- **No two parts have been trained together** except on the current discrete path. That path has no feed-forward layer and no position mechanism, and it scores below a table of word counts (**measurement**, [K16 outcome](../labs/k16-outcome-call-2026-10-19/README.md)).
- **Several other things are not built:** using an exact address as the read, any consumer of the object store, an exporter that turns a trained model into a D11 file, and an output step that avoids scanning the whole vocabulary ([audit synthesis](../labs/repo-renormalization-2026-10-19/audit/synthesis.md), section 1).

The project does not compare its models against transformers. It compares them against count tables and earlier geometric models, and grades them with fixed, deterministic checks ([charter](../../CHARTER.md), sections 3.10 and 4.4).

## Where it stands

| Measure | Value | Source |
|---|---|---|
| Mechanisms | 27: 10 ACTIVE, 4 READY, 13 PARKED | [mechanism board](../mechanisms/README.md) |
| Transformer parts with no mechanism | 2: position, normalization | [mechanism board](../mechanisms/README.md) slot map |
| READY parts never trained in | Hamilton gate, recurrence gates, SpiralCore/FBS (decimal form), object store | [mechanism board](../mechanisms/README.md) |
| Parts trained together | Only the discrete path: tokenizer, codebook or cells, reads, admission, hinge, discrete learning | [the model page](../model/architecture.md) |
| Best trained result vs count tables | 65 and 463 vs 1,226 and 6,300, of 14,287 | [K16 outcome](../labs/k16-outcome-call-2026-10-19/README.md), [count tables](../../archive/labs/host-diagnostics-2026-10-17/README.md) |

## Go deeper

- [What UOR-R4 is](./what-is-uor-r4.md) and [where we are](./where-we-are.md).
- [The model, end to end](../model/architecture.md): the current pipeline stage by stage.
- [Mechanism board](../mechanisms/README.md): one page per mechanism, with reopen conditions for PARKED ones.
- [Geometry](../geometry/README.md): the mathematics behind the replacements.
- [Open questions](../research/open-questions.md) and the [history](../research/history.md) of how each part was tried.
