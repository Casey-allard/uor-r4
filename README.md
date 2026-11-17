# UOR-R4 · A geometric language model

**Can a laptop predict language by rotating points, following a few addresses and reading exact memory?** UOR-R4 is an open research project testing that question. We build the model in Rust, measure each proposed mechanism against a simple baseline, and publish the failures alongside the results.

[Research book](https://uor-foundation.github.io/uor-r4/) · [Studio](https://uor-foundation.github.io/uor-r4/studio/) · [M1 model files](https://huggingface.co/caseyallard/uor-r4-m1-model) · [Roadmap and open work](https://github.com/UOR-Foundation/uor-r4/issues/2028)

![A brass geometric frame on graph paper beside a laptop, casting a shadow across a field of points.](docs/figures/start/geometric-workbench.jpg)

**Pre-alpha · 1 of 8 milestones complete.** We have published a trained integer model that ranks the next token. Producing complete replies grounded in earlier turns is an open problem. The target is one model that can carry a conversation, remember across sessions and eventually reason and write code on ordinary hardware.

| If you want to… | Start here |
|---|---|
| Understand a prediction from input to output | [Illustrated introduction](docs/start/what-is-uor-r4.md) and [model pipeline](docs/model/pipeline-map.md) |
| Inspect a result or reproduce a measurement | [Two-seed M1 record](docs/labs/m1-close-saved-model/README.md), [released weights and tokenizer](https://huggingface.co/caseyallard/uor-r4-m1-model), [research notebooks](docs/research/notebook/README.md) |
| Work on the next experiment | [Current research direction](#current-research-direction), [open roadmap items](docs/roadmap/README.md) and [contribution guide](CONTRIBUTING.md) |
| Follow the formalization | [Casey-allard/lexlean-r4](https://github.com/Casey-allard/lexlean-r4) and its [documentation](https://casey-allard.github.io/lexlean-r4/) |

Here we can test competing designs, retain a negative and change direction within the [research charter](CHARTER.md). LexLean-R4 takes the same goal toward a product through a separate formal implementation in LexLean and PrismPM. Each repository records the evidence for its own implementation.

## Try the M1 predictor

From the repository root, run the released seed-1 model on the token IDs for `The capital of France is`:

```sh
curl -fL -o seed1.d11 https://huggingface.co/caseyallard/uor-r4-m1-model/resolve/1fec0d4c42d4d5929a0cf08b1a0f17291d1dbc55/t16-frozen-zero-seed1-trunk.d11
printf '%s  %s\n' 04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d seed1.d11 | shasum -a 256 -c - && \
  cargo run --locked -p uor-r4-integer --features serving-wasm --bin uor-chat -- \
    --discrete seed1.d11 --prompt-ids 413,2743,3695,372,3704,1651,435 --max-tokens 1 --temperature 0
```

The JSON result contains `"generated_ids": [223]`. This is one next-token prediction.

The CLI returns one greedy token, ID **223** (a space). To see the eight ranked candidates for the same prefix, open **Models** in the public Studio, choose **Load M1 next-token predictor (seed 1)**, enter `The capital of France is` in **Prediction prefix**, then choose **Predict next tokens**. On November 17, the browser ranked 223 first with its network disabled after loading. [Public Studio](https://uor-foundation.github.io/uor-r4/studio/) · [Full result and model hashes](docs/products/web-client.md#run-the-seed-1-example) · [Published catalog](https://github.com/UOR-Foundation/uor-r4/blob/2878cdc029c1d232e7468e4751a610cc91fc8ab0/studio/models.json)

## Current research direction

On 16 November the lab adopted an experiment in **writing observations directly into addressed integer cells**. Counts establish what follows each token sequence. Phase P2 asks whether geometry improves which observations the model retrieves. With equal table bytes, it must use at least 1% fewer DEV-test bits than the best of token context, re-seeded transport and SplitMix, with the paired 95% interval excluding zero. Training and serving must replay the same persistent integer state. [Research decision](docs/research/reviews/pi-review-2026-11-16/DECISION.md) · [Native state and replay contract](docs/loop/tasks/written-state-count-model/claude-409/NATIVE-SPACE.md)

The separate whole-model quality gate requires at most **0.80 times the bits per byte of R\*** on sealed D87 DEV-test. R\* is an unpruned modified Kneser–Ney count model. At order 2 its scorer measured **2.004736 bits per byte** on D87 DEV-tune and **2.194312** on novel targets. Lower is better. The sealed reference and the geometric model comparison still await merged results. [Calibration measurement](docs/loop/tasks/written-state-count-model/claude-421/REPORT.md) · [Experiment sequence](PLAN.md)

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

### Earlier experiments on human text and conversation (6–14 November)

**Does the model learn human-written text?** M1's panels are templated. On real human text, a
held-out panel of 1,460,697 rows from public conversation and encyclopedia text, counted as correct
next-token guesses:

- **First runs (6 Nov):** training loss stays flat over 8,192 steps at two model widths, and every
  run trails the unigram table
  ([report](docs/loop/tasks/scale-probe/claude-295/REPORT.md)).
- **More data and more steps (6 Nov):** the best run gets 272,867 and 234,892 rows right at its two
  seeds against T2's 256,660, **inside the noise band**
  ([report](docs/loop/tasks/one-model-end-to-end/claude-297/REPORT.md)).
- **A batch of 64 with twice the training text (6 Nov):** 331,698 and 342,250 rows right against
  T2's 256,660, **ahead of T2 at both seeds** and on every sub-panel. The gain is the larger batch
  with twice the tokens, not the batch alone, and whether the seeds' spread is training noise is
  undecided at two seeds ([report](docs/loop/tasks/one-model-end-to-end/claude-301/REPORT.md)).
- **Width (8 Nov):** width 256 gets about 125,000 more rows right than width 128; width 512 did not
  fit the GPU's memory ([notebook row](docs/research/notebook/float-training.md), claude-317).

**Can the model answer in a conversation?** Next-token counts are not replies. Since 10 Nov the
work has been recall in controlled conversations. The current models state a value told earlier in
the conversation on 314, 320 and 325 of 578 recall questions at three seeds
([report](docs/loop/tasks/one-model-end-to-end/claude-358/REPORT.md)). The chat program has an
optional table counted from the training text, the pair experts: when its experts agree, it
replaces the model's reply with a stored training reply, so a reply served that way comes from the
table and not from the model ([report](docs/loop/tasks/one-model-end-to-end/claude-367/REPORT.md)).
What the model trains and serves today is on the
[mechanism board](docs/mechanisms/README.md#since-31-oct-what-the-current-model-trains-and-serves-14-nov-2026).

Every result, negative ones included, is one row in the
[research notebook](docs/research/notebook/float-training.md).

## The three products

None ships today. A trained integer artifact exists; an integrated, qualified product does not
([products](docs/products/README.md)).

- **[The model](docs/products/language-model.md):** one whole-number model file for local serving,
  with integer multiply and divide allowed by D80/D81. The saved discrete M1 pair is real; so is a
  separate `lut-stack/5` FBS export with exact rotations and its DEV export gap inside the bar.
  New FBS exports are `lut-stack/10`, which binds the stack contract and the tokenizer. Neither is a released conversational product.
- **[Web client](docs/products/web-client.md):** the Studio and public model files are available separately. The M1 inference task is top-8 next-token prediction. The synthetic
  parity harness passes; M7's accepted-model parity and a qualified product bundle remain open.
- **[Desktop client](docs/products/desktop-client.md):** the Tauri shell exists and verifies
  durable session behavior on a synthetic artifact. It has no qualified trained product bundle.

## Where we're going

M2 asks for complete correct replies grounded in exact memory; M3 then qualifies durable memory.
After that come one served model (M4), laptop cost (M5), reasoning and coding (M6), browser/API
distribution (M7) and an alpha release (M8). The current written-state experiment tests whether geometric reads add predictive value beyond counts. Grounded replies still have to pass controls for memory and speaker roles ([#3407](https://github.com/UOR-Foundation/uor-r4/issues/3407)). This is a plan, not a promise: [where we're going](docs/start/where-were-going.md) and [PLAN.md](PLAN.md).

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
projection or the FBS RMS normalization. The current FBS models (claude-358) read with the
Hamming read, the XOR/popcount form of [Ternary/XOR](docs/mechanisms/ternary-xor.md), not the
Route, Flock or Relation reads
([board](docs/mechanisms/README.md#the-current-recipe-claude-358s-p-recipe)). M1 does not establish every replacement's
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
- **[History](docs/research/history.md):** how the project got here, June to November 2026
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
- **sharmis-tp** ([@sharmis-tp](https://github.com/sharmis-tp)): the bounded `rustc --test` verifier for the frozen M6 CodeTasks panel (PR #3754, adopted and completed by the DeepSeek lab).
- **The AI research labs** (Kimi, Claude, Antigravity, DeepSeek and Codex; Codex left the registry on 27 Oct 2026 and rejoined on 12 Nov, PR #3942), which do most implementation under the owner's direction; owner and principal investigator Casey Allard.

## Citation and license

Cite the software with [CITATION.cff](CITATION.cff) (GitHub's "Cite this repository" button reads it). No
paper is claimed. [MIT](LICENSE); third-party research and dependencies keep their own licenses.
