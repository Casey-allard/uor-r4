# Where UOR-R4 is going

**In one paragraph.** The goal is a fast language model that runs on your own laptop or in your browser, holds a conversation with a durable memory, and later reasons and writes code, all on the geometric path with no transformer. The project marks progress with eight milestones, M1 to M8. None is complete yet. On 19 October 2026 the owner set a plain first bar: **the model must beat two simple word-count tables, on each of two seeds, through its whole-number export.** This page explains the milestones in plain words, the shortest credible path from today to a first simple conversation, and the three products at the end. The path is a plan, not a promise. Every training step still needs the owner's release, and the central question, whether a softmax-free model can learn to rank words well, is unanswered.

![A left-to-right road with six numbered stops, Stage 0 to Stage 5: "run what is already FIT'd", "make capacity measurable", "pick the learning law", "add the missing parts", "export and serve", "first conversation". A "you are here" pin stands at Stage 0. Gate symbols between stops are labelled "owner release". Under the road, milestone flags line up with the stages: M1 under Stages 2 to 4, M4 under Stage 4, M2 under Stage 5, and M3 marked not yet staged. Beyond Stage 5 the road continues lighter to M5 (laptop cost), M6 (reasoning and coding), M7 (browser and API) and M8 (alpha), ending at three product icons: a model file, a browser tab and a laptop app.](../figures/start/roadmap.svg)

*The road from today to the products. Stage 0 is where the project stands. Each gate needs an owner release.*

Terms are in the [glossary](./glossary.md). Labels are **proof**, **measurement** or **hypothesis** ([`docs/formal_vocabulary.md`](../formal_vocabulary.md)).

## The eight milestones in plain words

| | Milestone | In plain words | Status | Issue |
|---|---|---|---|---|
| M1 | Language base | A trained model that predicts the next piece of text better than simple count tables | Open; nothing passes yet | [#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029) |
| M2 | Grounded reply from exact memory | Answer a question from facts stored earlier, on saved and fresh questions | Held | [#2030](https://github.com/UOR-Foundation/uor-r4/issues/2030) |
| M3 | Durable conversation memory | Remember across sessions and stay grounded in what was stored | Paused | [#2031](https://github.com/UOR-Foundation/uor-r4/issues/2031) |
| M4 | One served model | One model, exported as a whole-number file under D11 (the whole-number serving contract), usable from the command line, with held-out quality at or below 0.90 bits per byte | Held | [#2032](https://github.com/UOR-Foundation/uor-r4/issues/2032) |
| M5 | Laptop cost | Measured energy per token, memory use and tokens per second on an M1-class laptop | Not started | [#2033](https://github.com/UOR-Foundation/uor-r4/issues/2033) |
| M6 | Reasoning and coding | Working reasoning and coding on the same model | Not started | [#2034](https://github.com/UOR-Foundation/uor-r4/issues/2034) |
| M7 | API, browser and Studio | The model reachable through an API, in the browser, and in the Studio interface, giving bit-identical tokens across library, command line and browser | Engine delivered; no model to load | [#2035](https://github.com/UOR-Foundation/uor-r4/issues/2035) |
| M8 | Alpha release | One versioned, checksummed model file, declared alpha by the owner | Not started | [#2036](https://github.com/UOR-Foundation/uor-r4/issues/2036) |

Statuses are from the [ROADMAP](../../ROADMAP.md) table, which states each bar in full. A milestone closes only when its bar is met on a saved model.

## The new M1 bar

M1 closes when **one saved discrete model** passes five conditions ([charter](../../CHARTER.md), section 4.8; [D39](../integration/DECISIONS.md#d39--m1-closing-bar-k16-reachability-gate-post-qualification-sequence-and-lab-self-direction-owner-2026-10-18), item 1):

- **(a) Starting reach.** Before training, the right answer is among the candidates the model opens (its "reach") at no fewer than 4,894 of the 14,287 test positions (the gate called G-Pre; the "G-" names are gate codes from D39).
- **(b) Reach kept.** That reach survives training, on both seeds (G-Adm).
- **(c) Real spread.** More than 16 of the 256 word cells (the "drawers" of words) are in use, plus the further occupancy condition in D39 (G-Occ).
- **(d) Beat the count tables (new, 19 October).** Whole-number top-1, measured through the D11 export, beats both the unigram table (1,226 of 14,287) and the next-word table (6,300 of 14,287) on the same test panel, on each of two seeds, with tokens per second reported. This replaces the earlier bar of 0.90 bits per byte, which the current model cannot be scored on because its loss gives no probabilities. Bits per byte returns at a later milestone.
- **(e) Fresh transfer.** The gain holds on a fresh, sealed test panel registered in advance.

For scale: the best trained model today scores 65 and 463 (**measurement**, [where we are](./where-we-are.md)). The gap to 6,300 is large.

## The shortest path to a first conversation

The audit of 19 October laid out the shortest credible path as six stages ([audit synthesis](../labs/repo-renormalization-2026-10-19/audit/synthesis.md), section 4). It is a proposal. The stage order follows the owner's fixed sequence in [D39](../integration/DECISIONS.md#d39--m1-closing-bar-k16-reachability-gate-post-qualification-sequence-and-lab-self-direction-owner-2026-10-18) item 4; changing it is the owner's decision.

**Stage 0. Run what is already FIT'd.** A capacity test (called W2, about $0.91) is FIT-reviewed and queued, awaiting an owner release ([NOW](../integration/NOW.md), row 6). "Capacity" here means how many different words the addresses can tell apart. W2 decides whether the shortfall comes from the *representation* (the addresses cannot tell enough words apart) or the *learning law* (the training method cannot find good addresses). Next comes the rerun of the address-cell model with its reach protected ([controller record](../labs/admission-anchor-controller-2026-10-19/README.md)). *Decides:* fix the addresses, or move on to Stage 2.

**Stage 1. Make the capacity measurable.** Replace the coarse address key (today it can take only 5 different values, so it barely separates the drawers) with one that uses the whole vector, and measure it before any training. A wider code is optional and needs new GPU code. *Bar:* enough distinct vectors that the number of positions still winnable clears the run bar of 1,941, and a starting reach above the address-cell model's 1,351 of 3,847.

**Stage 2. Pick the learning law.** First a faster discrete method that scores 16 coordinates at once (called BVG-K; its first stage is qualified), at about $12 for two seeds. Only if Stage 0 blames the learning law *and* BVG-K fails: one decimal-training run in a separate module, at about $5 to $10 plus build work. Decimal numbers are allowed in offline training only; the shipped file stays whole-number ([charter](../../CHARTER.md), section 3.7). *Bar:* beat the unigram table, then the admission-matched next-word table (the version W2 produces, not the 6,300 full-panel table), on two seeds.

**Stage 3. Add the missing parts.** Build a trunk with a feed-forward layer (the Hamilton gate), a recurrence with quaternion rotation, and the Flock and Relation reads. None of these has been trained together, and the only supporting evidence is weak, so this stage is a **hypothesis** test, not reuse of proven parts. Scale and data are also open: no trained stack has used the 1.39-billion-token chat corpus the project assembled (UOR-Foundation/uor-r4#1660). *Bar:* beat Stage 2's best by more than the seed-to-seed noise, with a fresh-panel number.

**Stage 4. Export and serve.** Write the missing exporter that turns a trained model into a D11 file, then measure the export gap and tokens per second on a laptop. *Bar:* the integer audits stay green, the gap stays within the noise band, and tokens per second is reported for both reading the prompt and writing the reply. This is where M1's bar (d) and M4 are tested.

**Stage 5. First simple conversation.** Train on fixed example answers produced once by a donor model (not yet generated), as the owner's one-time exception D41 allows: the student sees only the answers, never the donor's probabilities. Ground replies in the exact memory and the object store. *Bar:* M2's bar, at least 256 of 512 saved questions and at least 40% of fresh ones, graded by fixed deterministic checks.

**Binding conditions on every stage.** Training is held until the owner releases each reviewed plan, and the owner types each release personally ([charter](../../CHARTER.md), section 5). M1 can only be closed by a whole-number-trained ("discrete") model, so if Stages 3 and 4 use a trunk (the main body of the network) trained with decimals, the owner must first redefine the milestone. Under [D41](../integration/DECISIONS.md#d41--one-time-offline-donor-corpus-exception-owner-2026-10-19), the Stage 5 student stays on the discrete path unless the owner decides otherwise ([audit synthesis](../labs/repo-renormalization-2026-10-19/audit/synthesis.md), section 4).

**What the audit proposes to stop:** rerunning the old starting table (built from a fixed random seed) that gives at most 16 distinct vectors; tuning admission objectives on the 5-value key; and reading much into any run that loses to the two count tables.

## After the first conversation

Stages 0 to 5 aim at M1, M4 and M2; M3 is not yet staged. The rest of the road is:

- **M5:** measure the real cost on an M1-class laptop: energy per token, memory and tokens per second. An integer path is not an efficiency result until it is measured ([charter](../../CHARTER.md), section 3.9, "Efficiency is measured").
- **M6:** add reasoning and coding to the same model. Exact arithmetic is the visible gap.
- **M7:** make the browser worker, the command line and the Rust library give identical tokens.
- **M8:** the owner declares an alpha release.

## The products at the end

The owner defined three products on 19 October ([charter](../../CHARTER.md), section 2; [products](../products/README.md)):

| Product | What it will be | What exists today |
|---|---|---|
| [The model](../products/language-model.md) | One whole-number model file, built in `crates/uor-r4-discrete` and served by `crates/uor-r4-integer` | The serving engine, tested on a synthetic stand-in. No trained model to put in it |
| [Web client](../products/web-client.md) | The Studio in your browser, running the model on your own device in WebAssembly | The Studio page and the browser engine, not yet connected. Website deploys are paused |
| [Desktop client](../products/desktop-client.md) | A Tauri app sharing the web interface, running the model natively, with durable memory and offline use on M1-class laptops | A written specification only |

Nothing ships until a trained model passes M1 to M8. Running the finished model on your own computer or in your browser is the product, and the rule that research compute must use a GPU does not apply to it ([charter](../../CHARTER.md), section 3.6).

## The biggest unknown

Every read that learned well in the older experiments had help from softmax during training, and the best softmax-free read was rejected ([softmax-free read](../../archive/labs/softmax-free-read-2026-10-10/README.md)). Whether a lawful read can learn to rank words without it is a **hypothesis**, and no amount of assembly answers it before Stage 2 runs ([open questions](../research/open-questions.md)).

## Where it stands

| Item | State | Source |
|---|---|---|
| Milestones complete | 0 of 8 | [STATUS](../../STATUS.md) |
| M1 bar | Beat 1,226 and 6,300 through the D11 export, two seeds, tokens per second reported | [charter](../../CHARTER.md) 4.8 |
| Current stage | Stage 0: W2 FIT'd and queued, awaiting an owner release; address-cell rerun built, its qualification `NOT_RUN` | [NOW](../integration/NOW.md) |
| Stages needing new build work | 1 (wider code, optional), 3, 4 | [audit synthesis](../labs/repo-renormalization-2026-10-19/audit/synthesis.md), section 4 |
| Products shipped | None | [products](../products/README.md) |

## Go deeper

- [Where we are](./where-we-are.md): the measured starting point.
- [Transformer vs geometry](./transformer-vs-geometry.md): which parts exist and which are missing.
- [NOW](../integration/NOW.md): the live one-page plan, and [ROADMAP](../../ROADMAP.md) for milestone detail.
- [Products](../products/README.md): the model, the web client and the desktop client.
- [Charter](../../CHARTER.md): the rules every stage must follow.
