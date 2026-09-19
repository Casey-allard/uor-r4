# UOR-R4 Geometric Language Model

UOR-R4 is an experimental autoregressive geometric state model with exact addressed memory and learned typed operators, developed in Rust. The goal is useful local conversation, durable memory, reasoning and Rust coding on consumer M1-class hardware, ultimately with competitive quality and lower energy and wasted computation.

**Status: pre-alpha research.** The repository contains working geometric primitives, bounded learned memory/composition experiments and natural-text training infrastructure. Sustained general prose, useful chat, general reasoning/coding, frontier capability and complete-task energy savings are not established. There is no single qualified artifact combining all implemented capabilities.

Start with the [September 19 takeover review](docs/integration/takeover-review-2026-09-19.md), [current state](docs/integration/current-state.md), [canonical plan](docs/integration/project-track.md) and [next DeepSeek handoff](docs/integration/deepseek-next-step-2026-09-19.md). The review reconciles the earlier Codex work, intervening development and the latest mechanism measurements. Live [programme issue #820](https://github.com/UOR-Foundation/uor-r4/issues/820) owns delivery status; dated experiment records retain their original evidence scope.

## Goal and serving contract

The research priority is learned geometric addressing, state and selected computation: prime identities and ordered n-lets; R4/S3/H4 transport; exact `Z[phi]` and typed paired-H4/icosian structure; fixed zeta phases; exact occurrence/version memory; and shared learned operators. Each mechanism needs an implemented role and a measurement appropriate to that role. Canonical identity is not semantic distance, and exact geometry does not by itself establish predictive advantage.

The owner confirmed [DECISIONS.md D0-b](docs/integration/DECISIONS.md#d0-b--what-no-matmul-at-serving-means-adopted) during the September 19 takeover:

- Rust preparation and offline training may use floating point, gradients and matrix multiplication.
- Bounded integer/ternary linear maps are permitted at serving when executed through adds, subtracts, shifts and table reads, with no multiplier instruction in the kernel; learned weights are at most four bits, ternary preferred.
- Geometric routing and selected work remain preferred. No floating-point serving or hidden runtime teacher/provider is part of the target.
- Energy savings require physical measurement on a named machine, together with useful output quality.

This is a **multiplier-free serving target**, not a mathematical prohibition on every linear map. Whole-path compliance is not yet established for every experimental or shipped path. Allocation tests, symbol-level instruction checks and arithmetic parity tests support only their tested boundaries. The separate frozen TLA/R4G1 runtime retains its own stricter scoped contract. See [AGENTS.md](AGENTS.md) and the [execution policy](docs/integration/agent-execution-policy.md).

## Three model paths and their evidence

| Path | What exists | What the evidence establishes |
| --- | --- | --- |
| Retained native memory and shared language experiments | Exact retained values, versioned relations, Copy/Add, learned source choice, dependent reads, Read/Emit/Stop and bounded phrase/role composition | Causal generated behavior on authored tasks with retained controls. Later contextual-role repairs also use authored syntax rules. These are not general prose or an integrated chat model. |
| TinyStories prose learner and `.rgm` serving | Learned root assignments, JEPA/lexical objectives, lattice/count features, VSA routing, binary artifacts, CLI/API generation | A reported 555M-token training run and measurable text prediction. Training-time, exported full-vocabulary and routed serving scorers differ. Generations remain repetitive; historical memory tests do not qualify this new prose artifact. |
| Current geometric addressed-memory core | Fixed token-to-element assignment, ordered address pairs, accumulated value memory, ternary readout and optional group kernels | Synthetic recall/composition diagnostics and mechanism controls. Real-text geometric-core training and language qualification remain pending; the latest real-text count comparisons are baselines, not a model result or proved ceiling. |

The [evidence index](docs/integration/EVIDENCE.md) binds September 19 measurements to artifacts and receipts. Corrections and superseding rows must be read together.

### Selected results, with their limits

- **TinyStories training:** the recorded run processes 546,644,574 sequence tokens from a 555,385,505-token corpus in 3,982.42 seconds. Reported **1.2372 bits/byte** belongs to the continuous training-time probability model. The later exported full-vocabulary measurement is **1.8055 BPB**, or **1.7989 BPB** after removing the coarse lattice tier. These numbers do not measure the routed generation distribution on one common evaluation boundary. The repository n-gram comparator uses a smaller training sample and requires independent validation before it can support a matched Kneser–Ney claim. [Training record](docs/integration/current-state.md), [scorer and ablation evidence](docs/integration/EVIDENCE.md).
- **Codebook repair:** deriving VSA codes from learned roots improves routed candidate recall from approximately **8.4–8.8% to 10.7–11.2%** on two development slices. A full-vocabulary scoring ablation missed this routing effect. The result motivates measuring candidate admission separately from ranking; recall is still low. [Routing receipt](docs/evidence/native_geometric_shortlist_routing_2026-09-19.txt).
- **Exact finite composition:** the 120-element 2I table has checked identity, inverses and associativity over all 1,728,000 triples. Replacing selected quaternion accumulations preserved the measured outputs. This is an algebraic/implementation result, not a language-quality result. [Group-table receipt](docs/evidence/native_geometric_group_table_2026-09-19.txt).
- **Real-text residue comparison:** count predictors using two token-ID residues modulo 120 report **5.0019 bits/token** on repository source and **5.9546** on repository prose, versus full two-token-context comparators at **4.1573** and **4.8955**. These are fitted finite-data baselines. They neither prove an information-theoretic ceiling nor measure the current core's accumulated memory state. Their BPB values are not comparable to the TinyStories artifact's scores. [Receipt](docs/evidence/native_geometric_realtext_ceiling_2026-09-19.txt), [interpretation and next experiment](docs/integration/takeover-review-2026-09-19.md).
- **Efficiency:** one later release measurement reports native generation at **617 tokens/second** against approximately **36.1** for CPU Qwen2.5-1.5B Q4, with substantially unequal quality and different timing boundaries. It is not a quality-matched win. Whole-task energy/token remains **UNAVAILABLE**. Earlier small-kernel throughput figures must not be presented as interactive serving performance. [Incumbent receipt](docs/evidence/native_geometric_p1_incumbent_baseline_2026-09-19.txt).

## Current research step

Validate the real-text experiment's causal memory semantics and evaluation before scaling it: define what is written before each prediction, prevent target leakage, use document-separated data and the same exported serving computation, and compare against appropriate count and context interventions. Then measure complete trainer step cost and admit a bounded pilot from that timing. The [DeepSeek handoff](docs/integration/deepseek-next-step-2026-09-19.md) gives the concrete execution and reporting requirements.

The longer programme still requires coherent language, durable isolated memory, grounded correctness, compositional reasoning, executable Rust tasks and complete M1 cost on the **same accepted artifact**. The [canonical plan](docs/integration/project-track.md) owns that acceptance. The existing [Studio project](https://github.com/Casey-allard/uor-r4-wasm-chat) is a later integration target; a working UI is not evidence that an accepted native model runs in it.

## Source and tools

| Component | Source |
| --- | --- |
| Current geometric memory and low-bit substrate | [geometric_attention.rs](crates/uor-r4-core/src/native_geometric/learner/geometric_attention.rs), [lowbit.rs](crates/uor-r4-core/src/native_geometric/learner/lowbit.rs), [group_table.rs](crates/uor-r4-core/src/native_geometric/learner/group_table.rs) |
| Retained memory and shared language composition | [memory_runtime.rs](crates/uor-r4-core/src/native_geometric/memory_runtime.rs), [relation.rs](crates/uor-r4-core/src/native_geometric/relation.rs), [value_runtime.rs](crates/uor-r4-core/src/native_geometric/value_runtime.rs), [dependent_language](crates/uor-r4-core/src/native_geometric/dependent_language/) |
| Prose training and artifact | [jepa_trainer.rs](crates/uor-r4-core/src/native_geometric/learner/jepa_trainer.rs), [binary_model.rs](crates/uor-r4-core/src/native_geometric/learner/binary_model.rs), [train-native-prose.rs](crates/uor-r4-core/src/bin/train-native-prose.rs) |
| Routing, geometric observations and count features | [vsa](crates/uor-r4-core/src/native_geometric/vsa/), [lattice_table.rs](crates/uor-r4-core/src/native_geometric/lattice_table.rs), [hopf_metric.rs](crates/uor-r4-core/src/native_geometric/hopf_metric.rs), [engram.rs](crates/uor-r4-core/src/native_geometric/engram.rs) |
| Corpus preparation | [pretokenize-corpus.rs](crates/uor-r4-core/src/bin/pretokenize-corpus.rs), [mmap_corpus.rs](crates/uor-r4-core/src/native_geometric/mmap_corpus.rs) |
| Evaluation and diagnostics | [geometric-realtext-ceiling.rs](crates/uor-r4-core/src/bin/geometric-realtext-ceiling.rs) (historical name; fitted baselines), [ablate-prose.rs](crates/uor-r4-core/src/bin/ablate-prose.rs), [shortlist-recall.rs](crates/uor-r4-core/src/bin/shortlist-recall.rs) |
| Prose CLI/API | [r4-native-chat.rs](crates/uor-r4-api/src/bin/r4-native-chat.rs), [native_capability_api.rs](crates/uor-r4-api/src/native_capability_api.rs) |

The CLI accepts an explicit artifact and its matching tokenizer; trained artifacts are local ignored material and are not supplied by cloning this repository. For example, after building the relevant binary:

```sh
target/release/r4-native-chat --model /path/to/model.rgm --tokenizer /path/to/tokenizer.json --temperature 0 --top-k 1 "Once upon a time"
```

With no prompt it opens the interactive loop (`/reset`, `/stats`, `/exit`). Consult each linked binary's argument parser/help for current options. The corpus and training tools belong to package `uor-r4-core`; the chat binary belongs to `uor-r4-api`. Do not start a training or historical qualification campaign before recording its resource projection and verifying its inputs.

## Research, verification and continuity

- [Project map](docs/PROJECT_MAP.md): source families, historical engines, artifact locations and interface boundaries.
- [Architecture audit](docs/integration/architecture-2026-09/README.md) and [import audit](docs/integration/architecture-2026-09/imports.md): reusable UOR, NEMESIS, W33, GoldSnnail, SpiralCore and other mechanisms, with source/pin/claim limits.
- [Geometric-attention synthesis](docs/integration/geometric-attention-research-2026-09/README.md) and [mathematical foundations](docs/integration/geometric-attention-research-2026-09/mathematical-foundations.md): exact constructions, proposed bridges and missing learning evidence.
- [Research archive](research/README.md), [research ledger](docs/RESEARCH.md), [September 16 review](docs/integration/review-2026-09-16/00-project-brain-index.md) and [experiment cards](docs/integration/cards/): preserved historical evidence. Dated recommendations do not override current owner direction.
- [Formal vocabulary](docs/formal_vocabulary.md): distinguish definitions, assumptions, proof, empirical results and objectives.

Use focused compilation/tests and actual generated behavior for changed model paths; use allocation, serialization and operation checks where the changed boundary warrants them. Queue compatibility acknowledgements are not tests. The [historical qualification driver](scripts/verify_qualification.sh) has local evidence dependencies and tests the retained dependent-language family; it is not qualification of every model in the repository.

Preserve research, original checkouts, artifact parents, negative candidates and receipts. Record cumulative local resource charges and necessary authorized extensions before use; no paid/external compute is implied. Deliver named changes through protected pull requests and update the owning issue, current state and claims after each run. See [AGENTS.md](AGENTS.md) for the complete workflow.

## License

UOR-R4 is licensed under the [MIT License](LICENSE). Third-party research and dependencies retain their original licenses and attribution; a research link does not itself grant permission to reuse its contents.
