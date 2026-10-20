# Where UOR-R4 is today

**In one paragraph.** UOR-R4 is pre-alpha. **None of its 8 milestones is complete, and no model built under its rules holds a conversation** ([STATUS](../../STATUS.md), [NOW](../integration/NOW.md)). Many of the parts work on their own: a whole-number model path on the graphics card, three ways of reading earlier text, one shared word table, a whole-number engine that runs in a browser, and an exact storage system. But the best model trained so far guesses the next piece of text correctly far less often than two very simple tables of word counts. Nobody yet knows why. This page lists what works, what does not, and the measured numbers, each with its source.

![A horizontal bar chart on one scale from 0 to 14,287, the number of test positions. Bars from bottom to top: trained model seed 1 at 65, trained model seed 2 at 463, unigram table at 1,226, next-word table at 6,300. A dashed vertical line at 6,300 is labelled "new M1 bar: beat both tables, on each of two seeds". The two trained-model bars are coloured differently from the two table bars.](../figures/start/status-vs-baselines.svg)

*Correct first guesses out of 14,287 test positions. The trained model is far below both count tables, and the new M1 bar sits at the higher table.*

Terms are in the [glossary](./glossary.md). Claims are labelled **proof**, **measurement** or **hypothesis** ([`docs/formal_vocabulary.md`](../formal_vocabulary.md)). A passing check shows that code does what it says. It is not a trained capability, and `NOT_RUN` never means passed.

## How the model is scored

The test set is 14,287 positions in 400 held-out documents, called the DEV panel ([count tables record](../../archive/labs/host-diagnostics-2026-10-17/README.md)). At each position the model makes one guess for the next token. A **top-1** score counts how many of those guesses are right.

Two plain tables, built from the same training text the model sees, set the floor:

- The **unigram table** always guesses the single most common token. It scores **1,226** (**measurement**).
- The **next-word table** (called T2) looks only at the previous token and guesses the token that most often followed it in training. It scores **6,300** (**measurement**).

Both come from the [count tables record](../../archive/labs/host-diagnostics-2026-10-17/README.md). A model that cannot beat them has not yet learned anything a lookup table cannot do. The project compares against these tables, not against transformers ([charter](../../CHARTER.md), section 3.10).

## What works

These parts are built and pass their checks. None of them is a language result. (A **read** is the step that looks back at earlier text; **reach** is how many test positions have the right answer among the candidates the model opens at all.)

| Part | What the evidence shows | Label | Source |
|---|---|---|---|
| Whole-number model path on the GPU | The discrete model computes with integers only and passes 25 of 25 fixed GPU construction checks, plus 7 of 7 parity checks | measurement | [K16 qualification](../labs/k16-rows-qualification-2026-10-18/README.md) |
| Reads of earlier text | Route and Flock pass the 25 GPU checks; Relation passes 9 of 9 | measurement | [driver record](../../archive/labs/discrete-driver-cost-2026-10-13/README.md), [K16 qualification](../labs/k16-rows-qualification-2026-10-18/README.md) |
| One shared word table (the writer) | The same table encodes the input, carries memory and ranks the output. Four training runs each completed all 16,404 batches | measurement | [K16 outcome](../labs/k16-outcome-call-2026-10-19/README.md) |
| Distinct starting codes | PrimeZeta gives all 4,096 tokens distinct addresses | measurement | [construction](../../archive/labs/prime-zeta-construction-2026-10-14/README.md) |
| Address-structured cells | At zero training, 1,351 of 3,847 copy targets are reachable, against 127 before | measurement | [pre-flight](../labs/address-cells-preflight-2026-10-19/README.md) |
| Whole-number serving engine (D11) | 165 tests pass. In the browser build, 7 of 7 serving functions contain no multiply, divide or decimal instruction. A saved session reloads identically, on a synthetic stand-in model | measurement | [WASM runtime record](../labs/wasm-serving-runtime-2026-10-18/README.md) |
| Exact storage (object store) | 20 integration tests pass in the delivery record (60 after later peer and fleet work), and a client conformance check pulls 262,144 bytes back byte-identical | measurement | [object store](../../archive/labs/d19-object-store-2026-10-15/README.md), [conformance](../labs/uorobj01-conformance-2026-10-18/README.md) |

## What does not work yet

**No model holds a conversation** ([STATUS](../../STATUS.md)). The detail:

**The best trained model is below both count tables.** The last full experiment (called K16) trained two variants of the model, each on two random seeds, for 16,404 batches ([K16 outcome](../labs/k16-outcome-call-2026-10-19/README.md); all **measurement**):

| Variant | Seed | Correct first guesses | Reach: positions where the right answer was among the candidates opened | Cells in use (of 256) |
|---|---|---:|---:|---:|
| A (nearest-address margin) | 1 | 65 | 2,237 | 209 |
| A | 2 | 463 | 3,167 | 192 |
| R (occupied-cell rank) | 1 | 101 | 12,089 | 20 |
| R | 2 | 230 | 12,466 | 19 |

Variant A is the one the record selects. Its reach fell below the required 4,894 positions on both seeds. Variant R kept a high reach only by collapsing into 19 or 20 cells, so that each guess was drawn from about 3,400 of the 4,096 tokens, nearly the whole vocabulary; that reach is not real discrimination. The registered verdict was an occupancy collapse (record code `OCCUPANCY_COLLAPSE_TRIGGERED`), and no run was kept. All four miss the run's own top-1 bar of 1,941 and are far below the next-word table's 6,300. The large gap between seeds (65 against 463) also shows how noisy one run is.

**The cause is not established.** The first guess was that too few distinct starting vectors were to blame. That is ruled out as the main cause: in variant A, training raised the distinct vectors from 16 to about 275, yet top-1 stayed low ([K16 outcome](../labs/k16-outcome-call-2026-10-19/README.md), census section). The candidates still open are:

- **The address key may be too coarse.** The key is the distance that orders the drawers. It uses only the first and last block and takes only 5 values (0, 4, 8, 12 or 16) to order up to 256 cells (**proof**, from the kernel source), so which drawers open is largely decided by their arbitrary numbering ([the model page](../model/architecture.md), [chief architect's review](../labs/course-correction-2026-10-18/audits/d37-reactivation-architect.md) §3.2). Whether this causes the low score is a **hypothesis**.
- **The read, the ranking capacity, or the training schedule.** One full run sees fewer than half of the training targets: 1,049,856 of 2,425,925 (**measurement**, [audit dossier](../labs/repo-renormalization-2026-10-19/audit/mechanisms/discrete-learning.json)). These remain **hypotheses**.

**The newest run stopped early.** The address-cell run (A8) began with the better reach above, but training eroded it. At its first checkpoint the Relation read held 551 reachable positions, below half its starting 1,351, which triggered the registered stop (the control held 857; **measurement**, [A8 dose](../labs/a8-dose-2026-10-19/README.md)). A version that protects the reach is built; its GPU qualification is `NOT_RUN` ([controller record](../labs/admission-anchor-controller-2026-10-19/README.md)).

**No model trained under today's rules has been exported or timed.** No exporter turns a trained discrete model into a D11 file (the older float stacks were exported and served in October), and tokens per second for this model have never been measured ([lut-serving](../mechanisms/lut-serving.md)).

**Parts are missing.** The current model has no position mechanism, no normalization and no feed-forward layer ([transformer vs geometry](./transformer-vs-geometry.md)).

**Training is on hold.** Each training run needs a reviewed one-page plan and a release typed by the owner personally ([charter](../../CHARTER.md), section 5).

## What about the older models?

Before October, the project trained a decimal-number model called the geometric stack, from 7 million to 214 million parameters. Four 214-million-parameter checkpoints are published on Hugging Face as historical evidence. Their chat versions (which predate today's rules) could rewrite and summarise, while code and arithmetic were wrong ([audit dossier](../labs/repo-renormalization-2026-10-19/audit/products.json), from the model card). That stack used softmax, the SiLU activation (a smooth squashing curve) and matrix multiplication; geometry was about 0.05% of its arithmetic. Served through the integer engine, its best version (19.9 million parameters) reached 0.933 bits per byte and the charted versions 1.20 to 1.24, all missing the old M1 bar of 0.90 ([D11 serving record](../../archive/labs/d11-serving-2026-10-10/README.md); [audit dossier](../labs/repo-renormalization-2026-10-19/audit/mechanisms/geometric-stack.json)). It breaks today's rules and is PARKED ([geometric stack](../mechanisms/geometric-stack.md)). It shows the project can train a model that writes text. It does not show that the geometric replacements can.

## How the project got here

Between 12 and 19 October the owner (project lead Casey Allard) made 19 decisions, several of which rebuilt the model's foundations: no softmax anywhere, no matrix multiplication, GPU-only research compute, whole numbers on the active path. Much of that week went into rebuilding and checking rather than learning. In the week of 12 to 18 October, 34 of 309 records were results; the rest were tooling, plans and reviews, part of it ordered by those decisions. Five training runs on the new discrete path followed, and none produced a kept result ([audit synthesis](../labs/repo-renormalization-2026-10-19/audit/synthesis.md), section 3). The [history](../research/history.md) has the long story.

## Where it stands

| Measure | Value | Source |
|---|---|---|
| Milestones complete | 0 of 8 | [STATUS](../../STATUS.md) |
| Model that holds a conversation | None | [STATUS](../../STATUS.md) |
| Best top-1 on DEV, trained model | 65 and 463 of 14,287 | [K16 outcome](../labs/k16-outcome-call-2026-10-19/README.md) |
| Unigram table / next-word table | 1,226 / 6,300 of 14,287 | [count tables](../../archive/labs/host-diagnostics-2026-10-17/README.md) |
| Model trained under today's rules, exported to D11 | None | [lut-serving](../mechanisms/lut-serving.md) |
| Tokens per second, current model | Not measured | [lut-serving](../mechanisms/lut-serving.md) |
| Training | Held; each run is released by the owner | [charter](../../CHARTER.md) section 5 |

## Go deeper

- [Where we're going](./where-were-going.md): the milestones and the shortest path from here.
- [The model, end to end](../model/architecture.md) and [training](../model/training.md).
- [Open questions](../research/open-questions.md): what is unknown and what would settle it.
- Records: [K16 outcome](../labs/k16-outcome-call-2026-10-19/README.md), [A8 dose](../labs/a8-dose-2026-10-19/README.md), [count tables](../../archive/labs/host-diagnostics-2026-10-17/README.md).
- The live plan: [NOW](../integration/NOW.md).
