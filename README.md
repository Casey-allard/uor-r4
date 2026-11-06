# UOR-R4 Geometric Language Model

*An open research project: can geometry replace a transformer's run-time machinery in a language
model that runs on a laptop?*

[![License: MIT](https://img.shields.io/github/license/UOR-Foundation/uor-r4)](LICENSE)
[![Rust 1.97.1](https://img.shields.io/badge/rust-1.97.1-orange?logo=rust)](rust-toolchain.toml)
[![Status: pre-alpha](https://img.shields.io/badge/status-pre--alpha-yellow)](PLAN.md)
[![Milestones: 1 of 8 complete](https://img.shields.io/badge/milestones-1%2F8%20complete-blue)](#where-we-are)
[![Last commit](https://img.shields.io/github/last-commit/UOR-Foundation/uor-r4/main)](https://github.com/UOR-Foundation/uor-r4/commits/main)
[![Hugging Face](https://img.shields.io/badge/%F0%9F%A4%97-Hugging%20Face-ffcc4d)](https://huggingface.co/caseyallard/uor-r4-m1-model)

![Four panels read left to right: a sentence split into numbered pieces; each piece placed as a point on a geometric object; the newest piece looking at only a few nearby earlier points; a few drawers opening near what was read, with the nearest word inside them picked. A band underneath reads "Served as whole numbers: add, subtract, integer multiply/divide, compare, table look-up. No floats.".](docs/figures/start/hero-how-it-thinks.svg)

*How UOR-R4 is designed to predict the next word. The picture shows the design, not a claim of useful conversation.*

## What this is

Chat assistants run on **transformers**: programs that spend much of their effort multiplying very
large grids of decimal numbers. UOR-R4 bets that a different toolkit can do the same job more
cheaply: geometric rotations, whole-number addresses built from primes, small lookup tables and an
exact memory. The goal is a fast language model that trains quickly and runs on your own laptop or
in your browser, computing locally by default. The web Studio alone has an optional mode that sends
prompts to an OpenAI-compatible endpoint the user configures; the desktop shell has none.

The project is open research and **pre-alpha**. One of eight milestones is complete, but **no model
built under its rules holds a useful conversation yet.** [What UOR-R4 is](docs/start/what-is-uor-r4.md)
explains the bet in five minutes, and the [glossary](docs/start/glossary.md) defines every term used
below.

## Where we are

**Milestone 1 is complete (30 Oct 2026).** A saved discrete model, exported as two integer
artifacts (one per training seed), predicts the next token better than two fixed baselines built
from its own training text:

- the **unigram** table, which always guesses by overall word frequency;
- the **T2** table, which looks at the previous word and guesses the word most often seen after it.

| Test panel | Saved model, seed 1 / seed 2 | Unigram | T2 | Outcome |
|---|---:|---:|---:|---|
| Development panel, 14,287 rows | **7,002 / 7,001** | 1,226 | 6,300 | Beats both at both seeds |
| A fresh panel, 14,479 rows | **7,215 / 7,228** | 1,279 | 6,582 | Beats both beyond the paired noise band |
| Prompts in phrasings absent from training, 15,953 rows | **7,850 / 7,872** | 1,409 | 7,375 | Beats both beyond the paired noise band |

The integer artifacts match the reference model exactly on the development panel and decode at a
median of about **10,900 / 10,600 tokens per second on an Apple M1 laptop**.

**What this does not show.** These panels come from a templated conversation corpus, and 62.5% of
the held-out panel's reference replies also appear verbatim in training. M1 shows a model reading
new prompts into known kinds of reply. It does not show a model inventing new replies or carrying a
conversation. The evidence: [M1 issue](https://github.com/UOR-Foundation/uor-r4/issues/2029) and
[M1 close](docs/labs/m1-close-saved-model/README.md).

### The open research question

**Does the model learn human-written text?** M1's panels are templated. On real human text, a
held-out panel of 1,460,697 rows from public conversation and encyclopedia text, the model does not
yet beat the T2 baseline:

- **First runs (6 Nov):** training loss stays flat over 8,192 steps at two model widths, and every
  run trails the unigram table
  ([report](docs/loop/tasks/scale-probe/claude-295/REPORT.md)).
- **More data and more steps (6 Nov):** the best run (more training text, 32,768 steps) gets
  272,867 and 234,892 rows right at its two seeds against T2's 256,660. One seed is ahead and one
  behind, so the result is **inside the noise band, not a win**. Every run now beats the unigram
  table, and more training text is the one change that helped beyond the noise band at both seeds,
  at 8,192 steps ([report](docs/loop/tasks/one-model-end-to-end/claude-297/REPORT.md)).
- **Next:** the same recipe at a four-times-larger batch, to test whether the two seeds'
  disagreement is training noise ([#3407](https://github.com/UOR-Foundation/uor-r4/issues/3407)).

Every result, negative ones included, is one row in the
[research notebook](docs/research/notebook/float-training.md).

## The three products

None ships today. A trained integer artifact exists; an integrated, qualified product does not
([products](docs/products/README.md)).

- **[The model](docs/products/language-model.md):** one whole-number model file for local serving,
  with integer multiply and divide allowed by D80/D81. The saved discrete M1 pair is real; so is a
  separate `lut-stack/5` FBS export with exact rotations and its DEV export gap inside the bar. Neither is a released conversational product.
- **[Web client](docs/products/web-client.md):** the live Studio can load the publicly released M1
  model after an explicit click and serves top-8 next-token prediction—not chat. The synthetic
  parity harness passes; M7's accepted-model parity and a qualified product bundle remain open.
- **[Desktop client](docs/products/desktop-client.md):** the Tauri shell exists and verifies
  durable session behavior on a synthetic artifact. It has no qualified trained product bundle.

## Where we're going

M2 asks for complete correct replies grounded in exact memory; M3 then qualifies durable memory.
After that come one served model (M4), laptop cost (M5), reasoning and coding (M6), browser/API
distribution (M7) and an alpha release (M8). The immediate work is the open question above: a
model that beats both baselines on human-written text at every seed. Serving, speed and size work
for that model follow once it does. This is a plan, not a promise: [where we're going](docs/start/where-were-going.md) and [PLAN.md](PLAN.md).

## How it works

Each transformer part gets a geometric or exact replacement that avoids softmax and dense matrix
multiplication. Statuses come from the [mechanism board](docs/mechanisms/README.md): **ACTIVE** is
on a current model path, **READY** is built and tested but not trained in, and **PARKED** means the
tested forms failed or broke a rule while the mechanism stays open.

| Transformer part | Geometric replacement in UOR-R4 | Status |
|---|---|---|
| Embedding table (a starting vector per token) | [Prime-zeta codebook](docs/mechanisms/prime-zeta-codebook.md) and [address cells](docs/mechanisms/address-cells.md): whole-number addresses from primes and zeta-zero phases | ACTIVE |
| Attention (which earlier tokens matter) | [Route](docs/mechanisms/route-read.md), [Flock](docs/mechanisms/flock-read.md) and [Relation](docs/mechanisms/relation-read.md) reads: walk an address tree, rotate a few neighbours into one frame, follow copy links | ACTIVE; independent effects remain unisolated |
| Feed-forward layers (most of the multiplying) | [Hamilton rotation gate](docs/mechanisms/hamilton-gate.md) and [SpiralCore/FBS routes](docs/mechanisms/spiralcore-fbs.md) | Hamilton gate is in M1 and FBS; FBS routes are trained and served on FBS only; dated float projection form READY |
| Softmax output and cross-entropy loss | [Admission](docs/mechanisms/admission.md) (open a few address "drawers", called cells) and a [rank-margin hinge](docs/mechanisms/rank-margin.md) | ACTIVE |
| Backpropagation optimizer | [Discrete learning](docs/mechanisms/discrete-learning.md): try each legal rotation of one entry, keep it only if the score strictly improves | ACTIVE |
| Matrix-multiply runtime | [Integer serving (D11)](docs/mechanisms/lut-serving.md): whole-number adds, shifts, compares and table reads; integer multiply and divide are allowed | ACTIVE; the discrete M1 pair and the separate FBS `lut-stack/5` export both serve |

Position and normalization have replacements on the FBS trunk: SpiralCore transport spin and RMS
norm with an integer square root. M1 instead uses Flock `FrozenZeroV1`, `DiscreteHamiltonGateV1`,
`DiscreteQuaternionRecurrenceV1`, `PositionHolonomyV1`, `OrbitRootSnapNormV1`, an orbit-root causal pointer and post-rotation `OuterL1V1`—not FBS
projection or the FBS RMS normalization. M1 does not establish every replacement's
independent effect. The board lists
**28 mechanisms: 12 ACTIVE, 5 READY and 11 PARKED**. Part by part:
[transformer vs geometry](docs/start/transformer-vs-geometry.md).

<details>
<summary><strong>Historical K16 result — 19 Oct 2026</strong></summary>

Before the M1 closing artifacts, the selected K16-A discrete model scored **65 and 463** correct
next-token guesses out of 14,287, below the unigram table's **1,226** and T2's **6,300**. K16-R
collapsed to 19 or 20 occupied cells. These remain dated negative results, not the current best
model: [K16 outcome](docs/labs/k16-outcome-call-2026-10-19/README.md).

Four older 214-million-parameter checkpoints on Hugging Face predate today's rules (they used
softmax and matrix multiplication) and are kept as historical evidence, not as the D11 product.

</details>

## Explore

- **[Start here](docs/start/what-is-uor-r4.md)**, with a [glossary](docs/start/glossary.md) and [FAQ](docs/start/faq.md)
- **[Where we are, with the evidence](docs/start/where-we-are.md)** and the [pipeline map](docs/model/pipeline-map.md)
- **[Mechanism board](docs/mechanisms/README.md):** 28 mechanisms, one page each, with reopen conditions for the PARKED ones
- **[Geometry](docs/geometry/README.md):** primes and zeta phases, quaternions, E8/H4 and the 600-cell, SpiralCore, the Hopf map
- **[Research findings](docs/research/findings.md)** and **[open questions](docs/research/open-questions.md)**
- **[History](docs/research/history.md):** how the project got here, June to October 2026
- **[Archive catalog](archive/CATALOG.md):** every dated record, kept byte-identical and indexed
- **[Charter](CHARTER.md):** the rules (geometry only, no softmax, no matrix multiplication, integer serving) and why each exists
- **[Documentation hub](docs/README.md):** every page, by question

## Contribute

```sh
cargo check --locked -p uor-r4-discrete --lib
cargo test --locked -p uor-r4-discrete --lib
cargo run -q --locked -p uor-r4-claim-wording -- .
```

These checks run no model. Documentation fixes, reproductions of a recorded result and tooling are
welcome; model work is gated by the milestones. Start with [CONTRIBUTING.md](CONTRIBUTING.md). AI
research labs read [CHARTER.md](CHARTER.md) and [AGENTS.md](AGENTS.md).

## Acknowledgements

- **Alex Flom** (UOR Foundation, [@afflom](https://github.com/afflom)): the upstream UOR framework, addressing and Lean sources this project builds on, and review of its early design.
- **Ari** (UOR, [@auser](https://github.com/auser)): migrated the transformerless engine into this workspace and authored the R4G1/graph-compiler design and early certify measurement work.
- **Maura** (UOR, [@maurathat](https://github.com/maurathat)): UOR contributions including uor-addr, and a README correction here (PR #235).
- **Mark (N3mesis)**: geometry research in [NEMESIS-Theory](https://github.com/markrnd87-cmd/NEMESIS-Theory), the octonion/Fano and structure-carrying criteria that informed the substrate reviews.
- **Matthew**: author of SpiralCore (the v63 Cl(0,6) octonion operator convention and the v68/v69 material), whose signed transport is now the trunk's projection.
- **Gerrit** (DarkUnicorn, [LinkedIn](https://www.linkedin.com/in/gerrit-b%C3%B6hme-36b3a9413)): author of GoldSnnail and goldworm-coder, reviewed as external sources for state-layout patterns and evaluation ideas.
- **Wil Dahn** ([W33-Theory](https://github.com/wilcompute/W33-Theory)): theory discussions on the high-level geometry and its allowed pathing; author of the W(3,3) corpus reviewed by the labs.
- **John Haines**: theory discussions on the high-level geometry and its allowed pathing.
- **Ron Brazil**: theory discussions, engineering design, and research and development work.
- **Adam De Delva** ([LinkedIn](https://www.linkedin.com/in/adamdedelva/)): vision, relationships, unending support, product analysis, promotion, and commercialization efforts.
- **Yassine Amraoui** ([LinkedIn](https://www.linkedin.com/in/amraoui-yassine/)): research and development work.
- **Andrew Kadziolka**: development integration, assessment, analysis and conceptual design.
- **Yazan Barakat** ([@Cyborg-Hawk-AI](https://github.com/Cyborg-Hawk-AI)): research and development work on the WebAssembly D11 integer serving runtime (#1172, PR #2654).
- **The AI research labs** (Kimi, Claude, Antigravity, DeepSeek; Codex took part earlier and left the registry), which do most implementation under the owner's direction; owner and principal investigator Casey Allard.

## Citation and license

Cite the software with [CITATION.cff](CITATION.cff) (GitHub's "Cite this repository" button reads it). No
paper is claimed. [MIT](LICENSE); third-party research and dependencies keep their own licenses.
