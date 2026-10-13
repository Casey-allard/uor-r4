# UOR-R4 Geometric Language Model

*An open research project: can geometry replace a transformer's run-time machinery in a language
model that runs on a laptop?*

[![License: MIT](https://img.shields.io/github/license/UOR-Foundation/uor-r4)](LICENSE)
[![Rust 1.97.1](https://img.shields.io/badge/rust-1.97.1-orange?logo=rust)](rust-toolchain.toml)
[![Status: pre-alpha](https://img.shields.io/badge/status-pre--alpha-yellow)](STATUS.md)
[![Milestones: 0 of 8 complete](https://img.shields.io/badge/milestones-0%2F8%20complete-blue)](#where-we-are)
[![Last commit](https://img.shields.io/github/last-commit/UOR-Foundation/uor-r4/main)](https://github.com/UOR-Foundation/uor-r4/commits/main)
[![Hugging Face](https://img.shields.io/badge/%F0%9F%A4%97-Hugging%20Face-ffcc4d)](https://huggingface.co/caseyallard/uor-r4-geometric-214m)

**Status, 13 October 2026: pre-alpha.** No model in this repository holds a useful conversation yet.
Under D32 the model and its learning/qualification instruments are discrete: no
floating-point forward, gradients, optimizer or matrix multiplication. The new
SpiralCore/FBS shared-vector path passes **all eleven admitted CUDA construction
controls** after Claude FIT at source `a2a5b89cdddd1b32bf32f0e3c18d6b21e994e523`.
Earlier attempts remain observations outside required review order. Delivery: #2379.
Selection is query-addressed against fixed branch/frontier roots; leaf content supplies
the carried payload, and the query's own prefix leaf is selectable. M1's quality target
remains SUSPENDED pending a discriminating reply grader under R14.
**Training and the separate M2 holds remain held**; no update or useful-chat result.
[Discrete record](docs/labs/discrete-geometric-learning-2026-10-13/README.md).
Earlier 31/18-control float receipts retain their exact historical scope;
[PR #2370](https://github.com/UOR-Foundation/uor-r4/pull/2370) preserves that evidence.

## The question

Transformers spend most of their run-time work on dense matrix products and softmax attention. This
project asks whether a language model can carry its state, read its past and choose its next token
through **geometry** instead, and run on an M1-class laptop with no floating point and no multiplier
instruction at serving time ([D11](docs/integration/DECISIONS.md#d11--native-multiplier-free-transformerless-serving-for-every-lab-d10-superseded)).

"Geometry" here means concrete, exact structures:

- **State and transport:** unit quaternions (R4/S³) that rotate the state each step.
- **Exact structure:** the 120 icosians (the 600-cell), exact golden integers `Z[φ]`, E8 = H4 ⊕ φH4.
- **Addressing:** prime (UOR) addresses, SpiralCore signed transport, fixed zeta-zero phases.

The answer is recorded either way. Architectural priority is not a measured advantage, and the
project keeps the two apart.

## Architecture

The active standalone discrete path uses actual64-state SpiralCore transport,
ordered FBS context and one integer codebook for input, carried vectors and output.
Its sixteen-root orbit is not full E8 coverage or a language result; Claude's
discrete read/integration remain separate. The diagram and older line descriptions
below preserve the prior architecture snapshot, not an executable float trunk.

<img src="docs/figures/architecture.svg" alt="Architecture: line A, the geometric stack language base, shows the earlier operator status; the native head now exists, the revised compiled boundary passes finite controls at its declared source scope, and training is held; line B, the native learner, completes 437 of 512 development replies but 0 of 128 on fresh transfer; both join as one model served under D11 at M4" width="100%">

- **Line A, the language base (M1).** A geometric stack language model: recurrence layers that carry a
  quaternion state by rotation, geometric feed-forward blocks and a geometric head.
  Code: [`crates/uor-r4-training`](crates/uor-r4-training).
- **Line B, grounded reply from exact memory (M2, M3).** A learned compiler writes facts into an exact,
  versioned, addressed store; a learned emitter writes the reply. That historical compiler/emitter
  source is preserved; the selected core build now exposes fixed geometry and exact-storage support.
  Code: [`crates/uor-r4-core/src/native_geometric`](crates/uor-r4-core/src/native_geometric).
- **Serving (M4 target).** One model under D11 remains unqualified. The selected
  [`crates/uor-r4-integer`](crates/uor-r4-integer) build exposes fixed geometry and metadata;
  historical model-serving executors are excluded.

The geometry is explained, with pictures, in **[The geometry, illustrated](docs/geometry.md)**.

## Where we are

**0 of 8 milestones complete.** Each closes only when its acceptance is met on a saved model.
Live tracker: [#2028](https://github.com/UOR-Foundation/uor-r4/issues/2028).

| Milestone | Issue | Status | Headline now → target |
|---|---|---|---|
| M1 Language base | [#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029) | in progress; training held | reply headline SUSPENDED/no target under R14 pending a discriminating v2 grader; prior memory20/40 remains historical |
| M2 Grounded reply from exact memory | [#2030](https://github.com/UOR-Foundation/uor-r4/issues/2030) | held | 437/512 development replies (above 256), but 0/128 on fresh transfer |
| M3 Durable conversation memory | [#2031](https://github.com/UOR-Foundation/uor-r4/issues/2031) | paused | evaluator and session built; waits on an M1 or M2 model |
| M4 One served model (D11 and a CLI) | [#2032](https://github.com/UOR-Foundation/uor-r4/issues/2032) | held | softmax at runtime: yes → must be no (D23) |
| M5 Laptop cost | [#2033](https://github.com/UOR-Foundation/uor-r4/issues/2033) | not started | energy per token on an M1: not measured |
| M6 Reasoning and coding | [#2034](https://github.com/UOR-Foundation/uor-r4/issues/2034) | not started | exact arithmetic is the visible gap |
| M7 Distribution (API, WASM, Studio) | [#2035](https://github.com/UOR-Foundation/uor-r4/issues/2035) | not started | local API only |
| M8 Alpha release | [#2036](https://github.com/UOR-Foundation/uor-r4/issues/2036) | not started | owner decision |

<img src="docs/figures/milestones.svg" alt="Milestones M1 to M8: M1 in progress, M2 and M4 held, M3 paused, M5 to M8 planned" width="100%">

Per-lab measured position: [STATUS.md](STATUS.md). Plan of record: [ROADMAP.md](ROADMAP.md) and
[project-track.md](docs/integration/project-track.md).

## The current path: one lawful model, then training

The owner's rules of 12 October ([D23–D25](#rules-the-project-works-under)) prohibit softmax, sigmoid,
tanh, softplus and dense matrix products anywhere a model executes, in training as well as serving.
No earlier training configuration satisfies them, so the project is rebuilding the language base's
operators geometrically (the **operator phase**, led by Kimi) before training again.

<img src="docs/figures/geometry/compliant-path.svg" alt="The compliant next-token path: embedding, recurrence with geometric gates, geometric feed-forward with the SpiralCore/FBS projection and the G2 Hamilton gate, a geometric head, raw scores, a rank-margin hinge in training and greedy argmax in serving" width="100%">

| Step | Replaces | Status | Record |
|---|---|---|---|
| G2 Hamilton gate: the gate rotates the value, `q(g) ⊗ u` | the SwiGLU sigmoid | source, tested | [#2260](https://github.com/UOR-Foundation/uor-r4/pull/2260) |
| SpiralCore/FBS projection: select one carrier, transport by signed BFS | dense projections | source, tested | [#2279](https://github.com/UOR-Foundation/uor-r4/pull/2279), [brief](docs/mechanisms/spiralcore-fbs-projection.md) |
| Geometric recurrence gates: clamped opening, exact rate table, Hamilton output gate | sigmoid, `exp`, softplus, GELU | source, tested | [#2293](https://github.com/UOR-Foundation/uor-r4/pull/2293) |
| Rank-margin objective: a hinge over declared rivals on raw scores | softmax cross-entropy | source, tested | [#2255](https://github.com/UOR-Foundation/uor-r4/pull/2255), [brief](docs/mechanisms/rank-margin-objective.md) |
| Geometric head (OP3a): addressed raw-distance scoring through the shared token table | the dense tied head | implemented; earlier source receipts retained; repaired compiled boundary passes 31 finite controls at `1f1547bfd` | [head/writer](docs/labs/geometric-head-shared-writer-2026-10-13/README.md), [D29 boundary](docs/labs/d29-vendor-boundary-2026-10-13/README.md) |
| Device qualification: stored or analytic expected values, no CPU reference | the CPU-referenced parity gate | repaired D29 source's **31 mandatory controls PASS** at `1f1547bfd` on RTX4090; failed setup attempts retained, protected merge pending | [finite FIT sheet](docs/harness/prep/m1-d29-vendor-finite-2026-10-13.md) |
| Rung 0: the first lawful training run (2 arms × 2 seeds, ≤ $4) | — | Amendment6 independent FIT; **held**, finite source qualification recorded; protected delivery and owner release required | [sheet](docs/harness/prep/m1-rank-margin-rung0-2026-10-13.md) |
| Rung 0b: starling / rank-table reads with a rank-consistent gradient | softmax attention | after rung 0 | [brief](docs/mechanisms/starling-read.md) |

The diagrams retain the earlier operator snapshot; the table above is current. The selected native API now excludes historical core/integer model executors, teachers, LUT engines and exporters from compilation. Source/checkpoint lineage binds the masked TRAIN count reference separately from the DEV diagnostic oracle. "Source, tested" labels retain their named source/control scope. The repaired dependency graph separately passes the 31 finite controls at `1f1547bfd`; that result establishes neither trained language quality nor D11 serving. None of these operators has been trained
yet, and none is a capability claim. Pictures of the three new operators:
[the geometry page](docs/geometry.md#the-operators-of-the-compliant-trunk).

## Rules the project works under

| Rule | What it says |
|---|---|
| [D11](docs/integration/DECISIONS.md#d11--native-multiplier-free-transformerless-serving-for-every-lab-d10-superseded) | Serving uses adds, subtracts, shifts, compares and table reads: no floating point, no multiplier instruction, no transformer backbone. |
| [D22](docs/integration/DECISIONS.md#d22--a-negative-closes-a-configuration-never-a-mechanism-five-closures-reopened) | A negative result closes a configuration, never a mechanism. Decisive runs use ≥ 2 seeds or ≥ 200 fresh rows and a pre-registered bar reviewed `TEST FITNESS: FIT` before compute. |
| [D23](docs/integration/DECISIONS.md#d23---no-softmax-anywhere-in-the-active-project) | No softmax anywhere: training, serving, sampling, objectives, teachers, comparators, renamed or table forms. |
| [D24](docs/integration/DECISIONS.md#d24---no-cpu-model-compute-and-no-gpu-on-non-decisive-work) | No CPU model compute; no GPU on work that cannot change a decision. |
| [D25](docs/integration/DECISIONS.md#d25---geometric-operator-phase-and-current-lab-direction) | No softplus, sigmoid or tanh, and no dense matrix multiplication, in training or serving. The constraint set is frozen until the first lawful run reports. |
| [D26](docs/integration/DECISIONS.md#d26---vastai-is-the-default-provider-for-new-pods) | GPU pods are rented on Vast.ai through `scripts/pod/uor-pod`, within ≤ 4 pods and ≤ $8/h; Runpod is an emergency backup only. |
| [D27](docs/integration/DECISIONS.md#d27---rust-first-and-the-uor-foundation-ecosystem-as-the-shared-floor) | Rust is the default for all new code, tooling included; the uor-foundation crates are the shared floor. |

Rust interlocks `uor-harness ratchet d23` and `uor-harness ratchet d25` ratchet the source tree.
The D29 selected dependency/PTX checks additionally inspect the compiled boundary; lexical ratchets
alone do not certify that a scalar loop is geometric. All decisions: [DECISIONS.md](docs/integration/DECISIONS.md).

## Mechanisms under test

One page per novel mechanism: the idea, where it lives, what has been tried and what a fair test needs.
Index: [docs/mechanisms/](docs/mechanisms/README.md).

| Mechanism | Milestone | Where it stands |
|---|---|---|
| [Discrete SpiralCore/FBS learning and shared writer](docs/mechanisms/discrete-geometric-learning.md) | M1, M2, M4 | source a2a5b89c; eleven admitted CUDA construction controls PASS after Claude FIT; delivery #2379, training held |
| [SpiralCore/FBS projection](docs/mechanisms/spiralcore-fbs-projection.md) | M1, M2 | prior floating form preserved inactive under D32 |
| [Rank-margin objective](docs/mechanisms/rank-margin-objective.md) | M1 | prior floating objective preserved; discrete successor is separately registered |
| [Starling read](docs/mechanisms/starling-read.md) (~7 neighbours on the 600-cell) | M4 | served softmax-free; its gradient is rung 0b |
| [Flock / rank-table reads](docs/mechanisms/flock-rank-reads.md) | M4 | served; earlier measurements from a softmax-trained stack |
| [E8 orbit memory](docs/mechanisms/e8-orbit-memory.md) | M4 | source; rung 1 KEEP and iterate (historical) |
| [Exact addressed memory](docs/mechanisms/exact-addressed-memory.md) | M1, M3 | built; product-key memory trained in historically |
| [Invariant-addressed read](docs/mechanisms/invariant-addressed-read.md) | M1 | source; its router campaign was cancelled under D23/D24 |
| [Read binding](docs/mechanisms/read-binding.md) | M1 | KEEP in the historical float form |
| [Pointer with an identity key](docs/mechanisms/pointer-identity-key.md) | M1 | historical evidence |
| [Spin-native weights](docs/mechanisms/spin-native-weights.md) | M4 | quaternion weights and 2-bit norms KEEP (historical) |
| [Unified geometric carrier credit](docs/mechanisms/unified-geometric-carrier-credit.md) | M2 | source; training held |
| [Protected legal construction](docs/mechanisms/protected-legal-construction.md) | M2 | reopened by D22; not active |
| [Ternary / XOR arithmetic](docs/mechanisms/ternary-xor-arithmetic.md) and its [evidence sheet](docs/mechanisms/ternary-xor-evidence-factsheet.md) | M4, M5 | assessed, not adopted |

## Results so far

Every number holds only at its exact artifact, data, operator and budget. Rows marked **historical**
come from the pre-D23 float form (softmax reads and head, dense products): they are evidence of what
was measured, not a path that runs today.

| Result | Value | Scope |
|---|---|---|
| M1 memory and replies (29M chat line, frozen panels) | v5 memory **20/40**; open reply panel **37/232** acceptable | historical; [mixture dose](docs/labs/mixture-dose-2026-10-10/README.md) |
| Read binding, powered (3 seeds against matched controls) | wrong-value failures **47 → 30** of 120; **KEEP** | historical; [record](docs/labs/read-binding-powered-2026-10-11/README.md) |
| Native grounded learner | **437/512** development replies (from 177); **0/128** on fresh transfer: panel fitting | training held; [record](docs/labs/m2-pooled-rank-2026-10-10/README.md) |
| 19.9M chat stack | 0.877550 BPB float / 0.886838 served multiplier-free (11.9 MB) | historical; 512 windows |
| D11 integer engine | bit-exact with its float reference on 3,072 targets | engine property, not model quality |
| 214M Plan A base | FineWeb dev NLL 2.90 → 2.073 | historical; rewrite and summarize usable, code and arithmetic wrong |

<details>
<summary><b>All results, negatives and retractions</b></summary>

| Result | Value | Scope |
|---|---|---|
| Sealed 8M stack | 1.1199 BPB (Kneser-Ney 5-gram: 1.2803) | historical sealed report |
| Product-key memory trained into the M1 stack (`memory_layers=`) | layer 4, 64 sub-keys 19/40 (2 seeds); layer 8 (a read layer) 20/40 with wrong-value failures 17 → 10 | historical; [record](docs/labs/addressed-memory-2026-10-10/README.md) |
| Addressed read (product-key at the answer layer) | wrong-value 47 → 34 of 120, headline flat; −90 % per-token compute on that layer, +36.9 M parameters | historical; [record](docs/labs/addressed-read-2026-10-11/README.md) |
| Criterion 2, 29M M1 artifacts served under D11 | 1.20138 BPB against a ≤ 0.90 target; engine bit-exact | historical |
| v4 memory panel (`exact` check) | 31/40 at 214M, 26/40 at 96M | historical; one seed each |
| Softmax-free flock reads (2 seeds each) | Δ BPB against softmax: learned rank tables +0.0172 to +0.0185; fixed rank +0.040; Hamming rank +0.259 | historical; reads trained beside a softmax stack |
| Trained VSA codes in the native learner | held-out BPB improves 0.014–0.023; icosian-root-only codes collapse token identity | [record](docs/labs/vsa-native-test-2026-10-09/README.md) |
| MQAR toy (1.37M) | 0.99919 in-class vs 0.2534 control | synthetic; advantage confined to a learning-rate band |

**Negatives and retractions, kept as evidence.**

- D0 stack against a matched transformer control: 1.998 vs 2.011 NLL, parity. No tested geometric
  mechanism has yet beaten a matched ordinary control.
- Larger is not always better: a 214M SmolTalk-mixture stack scored 1.0966 BPB against 0.9607 for a
  7.16M chat-only stack on the same panel.
- Fine-tune levers of Steps 5–12 (knowledge corpus, dialogue recall, gate supervision, phase binding,
  constant learning rate, abstention) were null or rejected.
- The native learner's 437/512 is panel fitting: 0/128 on fresh literal-bank transfer.
- Track A1 stopped (D18); the transformer-conversion track is parked after its parity failure (#1518);
  complete-roadmap claims of 8 September were retracted by audit.
- The starling ladder's recall results (#2222–#2249) learned through softmax teachers and are inactive
  evidence under D23 ([archived branches](docs/history/branch-archive/INDEX.md)).

Sources: [current state](docs/integration/current-state.md), [evidence index](docs/integration/EVIDENCE.md),
[mechanism admissibility](docs/integration/mechanism-admissibility-2026-10.md).

</details>

## Bookkeeping: where things are recorded

| What | Where |
|---|---|
| What is true now, newest first | [docs/integration/current-state.md](docs/integration/current-state.md) |
| Owner decisions D0–D27 | [docs/integration/DECISIONS.md](docs/integration/DECISIONS.md) |
| Live tracker and milestone issues | [#2028](https://github.com/UOR-Foundation/uor-r4/issues/2028), #2029–#2036 |
| One dated record per completed unit | `docs/labs/<topic>-<date>/README.md` |
| Pre-registrations and their `TEST FITNESS` verdicts | [docs/harness/prep/](docs/harness/prep/) and the milestone issue |
| Who holds which branch, what each lab is doing, what ran | the research harness: [docs/harness/](docs/harness/README.md) |
| Unmerged, superseded or negative source | [docs/history/branch-archive/](docs/history/branch-archive/INDEX.md) (patches with sha256) |
| Compute spend and pods | `scripts/pod/uor-pod log`, the compute board [#2037](https://github.com/UOR-Foundation/uor-r4/issues/2037) |
| Results and artifacts | iCloud (MD5-verified `cloud-store`) and the private HF store; public release on Hugging Face |
| How the README and figures are kept current | [docs/bookkeeping.md](docs/bookkeeping.md) |

## How the project is run

The owner, Casey Allard, sets direction and holds the decisions on spending, holds and mechanism stops.
The work is done by AI research labs that share one repository and one harness:

- **Kimi:** project manager. One plan, line assignments, collision resolution and the operator phase.
- **Claude:** expert lab for the starling and rank reads and the GPU pods. It is also the proposed
  keeper of this README, the figures and the geometry page ([bookkeeping](docs/bookkeeping.md)).
- **Codex (Sol):** expert lab for enforcement, M2, the geometric head and the writer.
- **DeepSeek:** the harness and maintenance, plus the invariant-addressed router and memory.
- **Antigravity:** a fifth lab, currently offline.

Every unit follows the same cadence: claim a branch in the harness, pre-register any compute and get a
`TEST FITNESS: FIT` review, deliver through a protected PR with checks run at its exact head, write the
dated record and current-state entry, merge, then delete the branch on both sides.
`origin/main` is the only source of truth. Full procedure: [CONTRIBUTING.md](CONTRIBUTING.md); rules for
agents: [AGENTS.md](AGENTS.md); the PM's operating rules: [pm-rules.md](docs/harness/pm-rules.md); the shared
harness: [docs/harness/README.md](docs/harness/README.md).

## Quick start

Rust 1.97.1 is pinned by `rust-toolchain.toml`. Training data and trained artifacts are not in the
repository (see [Models and data](#models-and-data)). Model compute runs on GPU pods only (D24).

```sh
git clone https://github.com/UOR-Foundation/uor-r4 && cd uor-r4
CARGO_TARGET_DIR=local/readme-target cargo check -p uor-r4-training --lib --example geometric-stack
CARGO_TARGET_DIR=local/harness-target cargo build -p uor-r4-harness
UOR_LAB=codex ./local/harness-target/debug/uor-harness ratchet d23
UOR_LAB=codex ./local/harness-target/debug/uor-harness ratchet d25
```

These commands compile/check source only; they execute no model. The native `geometric-stack`
example supports CUDA `train` and masked `evaluate`; historical sampling/export modes and integer
model executors are excluded. CUDA builds and finite controls use the admitted pod runner with
the pinned finite sheet; neither the commands nor FIT release a training hold. Use your own lab
identifier and target directory.

New lab: set `UOR_LAB`, then run the harness onboarding ([docs/harness/ONBOARD-PROMPT.md](docs/harness/ONBOARD-PROMPT.md)).
GPU work goes through `scripts/pod/uor-pod` ([docs/labs/compute.md](docs/labs/compute.md)); never rent by hand.

<a id="models-and-data"></a>
<details>
<summary><b>Models and data</b></summary>

Released publicly on Hugging Face (9 October 2026). These checkpoints are **historical**: trained before
D23–D25 in the float form. The labs' working store stays private.

**[caseyallard/uor-r4-geometric-214m](https://huggingface.co/caseyallard/uor-r4-geometric-214m)**: four 214M
geometric-stack checkpoints, with configs, sanitized run reports and the byte-BPE tokenizer.

| Checkpoint | What it is | Result |
|---|---|---|
| `base-tinystories/` | Step 8 base: TinyStories, TinyDialogues and chat-v0-p2 | TinyStories val NLL 1.032 |
| `base-planA/` | the base above, continued for 1.84B SmolLM-Corpus tokens | FineWeb dev NLL 2.90 → 2.073 |
| `chat-planA-b8/` | Plan A base plus an assistant fine-tune (173M-token mix, 8k steps) | rewrite and summarize usable; code and arithmetic wrong |
| `chat-smoltalk-b16/` | Step 8 base plus the same fine-tune mix (16k steps) | rewrite and summarize usable |

**[datasets/caseyallard/uor-r4-data](https://huggingface.co/datasets/caseyallard/uor-r4-data)**: the
project-generated data: the tokenizer, the memory-dialogue sets (`mw-balp`, `mw-orcp`) with the fine-tune
manifest, the dev split, the conversational panels, and TinyStories tokenized for the native learner.

| Source | Used for | License |
|---|---|---|
| [SmolLM-Corpus](https://huggingface.co/datasets/HuggingFaceTB/smollm-corpus) (FineWeb-Edu, Cosmopedia-v2) | Plan A pretraining | ODC-BY |
| [TinyStories](https://huggingface.co/datasets/roneneldan/TinyStories) | 8M–214M ladder pretraining | CDLA-Sharing-1.0 |
| [TinyDialogues](https://huggingface.co/datasets/styfeng/TinyDialogues) | 214M Step 8 pretraining | MIT |
| [SmolTalk](https://huggingface.co/datasets/HuggingFaceTB/smoltalk) and [UltraChat 200k](https://huggingface.co/datasets/HuggingFaceH4/ultrachat_200k) (as `chat-v0-p2`) | chat pretraining share and the assistant fine-tune | Apache-2.0 / MIT |
| Project-generated memory dialogues (`mw-balp`, `mw-orcp`, dialogue-recall) | memory fine-tunes, recall panels | MIT (this project) |

Third-party text is not republished. SmolLM2 weights (Apache-2.0) were used only as an offline comparator,
never on a serving path, and no new comparator runs are permitted (D23).

</details>

<details>
<summary><b>Repository map</b></summary>

| Path | Purpose |
|---|---|
| [`crates/uor-r4-core`](crates/uor-r4-core) | selected fixed geometry, tokenizer and exact-storage support via `native_support.rs`; historical learner/model sources preserved inactive |
| [`crates/uor-r4-training`](crates/uor-r4-training) | native-only geometric CUDA stack, shared vector writer/store and train/evaluate CLI; historical teacher/export/experiment modules excluded |
| [`crates/uor-r4-integer`](crates/uor-r4-integer) | selected fixed H4 geometry and report metadata; historical D11 model/CLI source retained inactive, compliant serving remains an obligation |
| [`crates/uor-r4-harness`](crates/uor-r4-harness) | the Rust research harness (`uor-harness onboard`) |
| [`crates/uor-r4-api`](crates/uor-r4-api) | typed compile and engine APIs; the `r4-native-chat` CLI |
| [`crates/uor-r4-tokenizer`](crates/uor-r4-tokenizer) | byte-level BPE engine and tokenizer identity |
| [`crates/uor-r4-lut`](crates/uor-r4-lut), [`crates/uor-r4-simd`](crates/uor-r4-simd) | LUT selected format/grid metadata facade; historical comparator/SIMD model source outside the native training graph |
| [`crates/uor-r4-router`](crates/uor-r4-router) | reusable routing plus historical paths |
| `crates/uor-r4-graph-*` | the frozen R4G1 graph format, compiler, runtime, certifier and CLI |
| [`crates/uor-r4-proof-model`](crates/uor-r4-proof-model), [`crates/uor-r4-naf`](crates/uor-r4-naf), [`crates/uor-r4-workbench`](crates/uor-r4-workbench) | proof specification, the UOR-NAF slice, the bounded research reference |
| [`crates/repo-model`](crates/repo-model), [`crates/repo-conformance`](crates/repo-conformance), [`xtask`](xtask) | claim registries, honesty gates, `cargo xtask` repository tasks (including `figures`) |
| [`scripts/`](scripts) | the harness CLI, pod tooling (`scripts/pod/`), storage hygiene, interlocks |
| [`docs/`](docs) | decisions, current state, lab records, mechanism briefs, the geometry page, history |

The [project map](docs/PROJECT_MAP.md) covers every crate and directory.

**Statistics, 13 October 2026** (mostly multi-agent lab work, not a count of human contributors): about
1,990 commits since 9 June 2026; 1,761 merged PRs (968 in the last 30 days); about 1.68M lines of Rust;
8,131 `#[test]` functions; decisions D0 to D27; 14 open issues.

</details>

## Acknowledgements

- **Alex Flom** (UOR Foundation, [@afflom](https://github.com/afflom)): the upstream UOR framework, addressing and Lean sources this project builds on, and review of its early design.
- **Ari** (UOR, [@auser](https://github.com/auser)): migrated the transformerless engine into this workspace and authored the R4G1/graph-compiler design and early certify measurement work.
- **Maura** (UOR, [@maurathat](https://github.com/maurathat)): UOR contributions including uor-addr, and a README correction here (PR #235).
- **Mark (N3mesis)**: geometry research in [NEMESIS-Theory](https://github.com/markrnd87-cmd/NEMESIS-Theory), the octonion/Fano and structure-carrying criteria that informed the substrate reviews.
- **Matthew**: author of SpiralCore (the v63 Cl(0,6) octonion operator convention and the v68/v69 material), whose signed transport is now the trunk's projection.
- **Gerrit** (DarkUnicorn, [LinkedIn](https://www.linkedin.com/in/gerrit-b%C3%B6hme-36b3a9413)): author of GoldSnnail and goldworm-coder, reviewed as external sources for state-layout patterns and evaluation ideas.
- **Wil Dahn** ([W33-Theory](https://github.com/wilcompute/W33-Theory)): theory discussions on the high-level geometry and its allowed pathing; author of the W(3,3) corpus reviewed by the labs.
- **John Haines**: theory discussions on the high-level geometry and its allowed pathing.
- **Ron Brazil**: theory discussions and engineering design.
- **The AI research labs** (Kimi, Claude, Codex, DeepSeek), which do most implementation under the owner's direction; owner and principal investigator Casey Allard.

## Citation and license

Cite the software with [CITATION.cff](CITATION.cff) (GitHub's "Cite this repository" button reads it). No
paper is claimed. [MIT](LICENSE); third-party research and dependencies keep their own licenses.
