# UOR-R4 Geometric Language Model

**An open-source research project to replace transformer-based large language models with a learned geometric language model running on local, commercially available hardware.**

The objective is frontier-level prose, conversation, memory, reasoning and coding without a transformer backbone or mathematical matrix multiplication at inference time. We are investigating whether better mathematical representations of identity, context, relationships and computation can deliver those capabilities with substantially less energy and wasted work. Consumer hardware, including M1-class laptops, is the target, not a remote frontier model hidden behind a local interface.

Technically, UOR-R4 is an **experimental autoregressive geometric state model with exact addressed memory and learned typed operators**, developed in Rust. It uses prime-addressed ordered context, fixed zeta-zero phases, signed R4/S3/H4 geometry and exact golden-number arithmetic as primary architectural mechanisms. Learning must turn those structures into useful contextual decisions and language.

**Status: pre-alpha research.** Frontier capability, sustained general prose, general reasoning/coding and complete-task energy savings have not been established. The repository contains implemented mechanisms, bounded experimental results, preserved failures and broader research sources; these are not all one integrated, qualified model. The [canonical plan](docs/integration/project-track.md) defines the objective and acceptance, and [current state](docs/integration/current-state.md) identifies the actual artifacts and latest results.

**Read this page by topic:** [Runtime contract](#runtime-contract) · [Architecture](#architecture) · [Mathematics](#mathematics-and-its-computational-role) · [Active source map](#active-source-map) · [Available mechanisms](#available-mechanisms-and-research) · [Evidence](#current-evidence-and-open-problems) · [Repository map](#repository-and-documentation-map) · [Getting started](#getting-started) · [Contributing](#contributing-and-research-continuity)

## Runtime contract

The goal is a different computational architecture, not a transformer renamed after its coordinate system.

| Boundary | Contract |
| --- | --- |
| **Final inference** | **No transformer backbone and no mathematical matrix products.** This excludes dense projections and matrix contractions implemented indirectly through coded lookup/addition, including serving calls to `uor-matmul`. Removing hardware multiply instructions does not remove the mathematical matrix product. |
| **Permitted serving mechanisms** | Learned finite geometric transitions; exact group composition by lookup; direct signed permutations; bounded integer/bit operations; geometric address/page selection; exact selected reads, writes and typed computation; learned output decisions. Each adopted mechanism needs an explicit execution and cost contract. |
| **Offline preparation and training** | Rust data preparation, learning and artifact construction may use floating point, gradients and matrix multiplication. Declared teacher/reference data can inform offline work; no teacher, provider or other model may author the serving response. |
| **Model organization** | Shared typed operators and a recurrent geometric language core are the current design. An operator such as Read, Copy or Add is not a neural expert bank. Expert gates remain conditional future research, not an adopted architecture or permission to introduce forbidden serving arithmetic. |
| **Hardware and efficiency** | Target useful local operation on consumer laptops/desktops. Measure complete requests, including encoding, access, state updates, output, loading and persistence where relevant. A fast isolated kernel is not a whole-model speed or energy result. |

The historical TLA/R4G1 kernel has a **separate, narrower** no-multiply/no-divide/no-float, allocation-free steady-state contract. That contract remains intact; it must not be confused with proof that every native prototype already meets every deployment requirement. See [agent policy and execution lanes](AGENTS.md), the [import audit](docs/integration/architecture-2026-09/imports.md) and [runtime constraint issue #964](https://github.com/UOR-Foundation/uor-r4/issues/964).

## Architecture

### The organizing idea

Separate four things that are easy to conflate:

| Responsibility | What it means here |
| --- | --- |
| **Identity** | Preserve which exact bytes, occurrence, source, version or derived value an object represents. Canonical identity is not a meaning vector. |
| **Geometry** | Represent ordered state, relative transformations, phase and orientation; provide explicit coordinates and finite actions for access and computation. |
| **Learning** | Decide which contextual distinctions matter, which source or operation to select, how state changes and what to emit. Algebraic structure does not supply those decisions automatically. |
| **Evidence** | Establish that the actual selected source and operation caused the correct complete output, under stated data, controls and resource bounds. |

The central research question is whether **exact memory plus learned, shared geometric operations** can replace the representational and computational work ordinarily performed by a large dense neural model. Preserving an object exactly and learning how it matters in context are complementary tasks.

### Intended end-to-end system

This is the target integration, not a claim that every connection below is already qualified:

```text
Raw text / causal observations
          |
          v
Canonical lexical identities + exact ordered occurrences
          |
          v
Geometric working state + context-dependent query
          |
          v
Bounded geometric access <------> Exact versioned memory / derived values
          |
          v
Learned contextual source and typed-operation selection
          |
          v
Selected payload + finite geometric/value operation
          |
          +------> Causal state/memory update ------> Next query/read
          |
          v
Learned byte/EOS output or exact selected copy
          |
          v
One native model/session API -> CLI / local service -> eventual WASM Studio

Offline Rust learning -> artifact-bound parameters, codes and transition tables
```

Here **attention** means learning which available context to use. It does not imply transformer attention, softmax-weighted dense values or a matrix projection. The access law, candidate eligibility, relevance decision, payload retrieval, state update and output decision have distinct responsibilities. A better ranking cannot recover a correct source that was never admitted.

The [geometric-attention synthesis](docs/integration/geometric-attention-research-2026-09/README.md) connects these responsibilities to the mathematics and learning literature. The [first shared-core design](docs/integration/shared-geometric-core-2026-09.md) preserves an earlier implementation design; the living plan and current-state record supersede its dated scheduling.

### What currently executes

There are two important native tracks under `crates/uor-r4-core/src/native_geometric/`:

**Retained native model.** The existing artifact/session path combines bounded learned prediction and source selection with exact occurrence/version memory, selected arithmetic, copying, response coordination and checkpoint restoration. Its accumulated memory-repair work is parked and preserved while the shared language core is developed. This is the normal-model reference, not a claim of general language capability.

**Active experimental language core.** The newer lineage connects raw-language occurrence matching, exact phrase reads, learned query updates and recurrent output control. Its functional loop is:

```text
Supplied text records + question
    -> canonical word/occurrence encoding and signed prefix history
    -> learned source/query roles and query-evidence participation
    -> ordered source correspondence and exact payload-span selection
    -> learned Read / Emit / Stop control
    -> selected phrase updates the next query when another read is needed
    -> repeat contextual reading, or emit bytes/EOS, or return typed unresolved
```

This work reuses and extends the earlier typed reader, updater, scheduler and writer. It is not a collection of independent models that can be added together to claim a larger capability. Components are often fitted with other parameters frozen; the goal of a broadly useful shared predictive learner remains unfinished.

As documented on **September 14, 2026**, the retained normal artifact is `15baec48` and the latest evaluated experimental artifact is `60d19679`. They are **different artifacts**. The experiment has not replaced the normal model or been qualified as a general assistant. These are short artifact identifiers, not source commit IDs. For full identities, exact local paths and subsequent changes, use [current state](docs/integration/current-state.md), not a historical “next action.”

## Mathematics and its computational role

The mathematical program is broader than the currently connected learner. The following map distinguishes a computational role from a proposed semantic advantage. Detailed source ranges, conventions and limitations are in the [mathematics audit](docs/integration/architecture-2026-09/mathematics.md) and [newer mathematical synthesis](docs/integration/geometric-attention-research-2026-09/mathematical-foundations.md).

### Prime identity, ordered context and zeta phases

Prime atoms and **ordered n-lets** give records and compositions explicit structural identities. A product or factor multiset alone is insufficient for language: `pq = qp`, while reversing the corresponding occurrences can change the answer. Ordered occurrences and exact source/version references therefore remain separate from commutative arithmetic summaries.

The fixed spectral coordinates have the form

```text
phase_j(p) = gamma_j * log(p) mod 2*pi
relative_phase_j(p_from, p_to) = gamma_j * log(p_to / p_from) mod 2*pi
```

Here `gamma_j` is a supplied zeta-zero ordinate in a pinned finite table. These channels provide structured logarithmic phase observations. They are not a pointwise identification of a prime with a zero, and using the finite constants does not depend on proving the classical Riemann Hypothesis. The native compiler and individual experimental paths use different finite channel subsets; availability is not the same as every channel influencing every output.

The research obligation is to show which distinctions the phases preserve and whether a learned model uses them advantageously. A benefit from phase coordinates and a benefit specific to **zeta-derived** coordinates are separate questions. See [prime-route foundations](crates/uor-r4-core/src/prime_route_attention.rs), [ADR0003](docs/adr/0003-fixed-zeta-prime-route-attention.md) and the [attention synthesis](docs/integration/geometric-attention-research-2026-09/README.md).

### Signed geometry, exact algebra and finite actions

| Mechanism | Computational role and implementation boundary |
| --- | --- |
| **R4 and S3** | Four-dimensional coordinates and unit-quaternion state support orientation-bearing representations. The declared normalization, frame and update law matter; arbitrary four numbers do not automatically constitute the intended geometric state. |
| **H4-derived finite group state** | The implemented 120-root binary-icosahedral construction supplies exact noncommutative product/inverse tables. Ordered left/right prefix products retain sequence information that a final product alone loses. Exact occurrence identities remain alongside the geometric history. |
| **Exact `Z[phi]`** | With `phi^2 = phi + 1`, values `a + b*phi` can be represented by exact coefficient pairs. This supports canonical root construction and algebraic witnesses; selected finite actions can be compiled to tables. Compiler algebra is not automatically a serving operation. |
| **Paired H4 / icosian / E8 construction** | The project uses a specific golden/Galois-coupled representation with declared basis, glue and inverse witnesses. “E8 = H4 x H4” is project shorthand for that construction, not an assertion that arbitrary 8D information fits losslessly into 4D or that the coupled companion is independent learned state. |
| **Hopf observation, fiber and torsion** | S3-to-S2 observation provides a lower-dimensional view. Retain the fiber/orientation information needed for subsequent computation rather than assuming the projection is reversible. Associated vector-bundle or connection mechanisms require explicit carried values and transition maps. |
| **Hamming signatures** | Fixed geometric landmark signatures support bounded XOR/popcount comparisons. A scalar distance describes compatibility but does not retain every direction, role or relative transformation; exact identity and orientation remain separate. |
| **Directed relative transformations** | For states in an associative group, `r(i,j) = inverse(g_i) * g_j` preserves a directed relation and composes as `r(i,j) * r(j,k) = r(i,k)`. The synthesis proposes richer relation-preserving reads; this is not a claim that every proposed relation block is integrated. Such node-derived edges telescope around loops and do not alone establish nontrivial curvature. |

The current ordered-state implementation is in [ordered_state/runtime.rs](crates/uor-r4-core/src/native_geometric/ordered_state/runtime.rs); finite comparison is in [hamming_refinement/metric.rs](crates/uor-r4-core/src/native_geometric/hamming_refinement/metric.rs). The [repository mathematics map](docs/integration/geometric-attention-research-2026-09/repository-mathematics.md) connects these mechanisms to actual source and distinguishes them from historical hyperbolic **H^4**, which is not the same object as finite H4-derived state.

### Why contextual learning remains essential

A representation can retain the exact words yet discard a distinction at the decision interface. If two occurrences have the same observation `O(x) = O(y)` but require different roles, no deterministic classifier using **only that observation** can assign both intended roles. Adding more training to the unchanged observation does not restore the missing information.

The latest independent transfer experiment exposes precisely such a source-role alias: an unfamiliar name containing *will* and an auxiliary use of *will* can share the same finite neighbor key. This motivates testing additional ordered contextual evidence, not inserting a word-specific exception. The role-label obstruction is distinct from proving the effect of a complete runtime repair. See the [localized failure and unrun boundaries](docs/native_geometric_independent_neighbor_973.md).

## Active source map

Paths below are relative to `crates/uor-r4-core/src/native_geometric/` unless a link names another location. “Implemented” describes source and recorded bounded behavior, not normal-model promotion.

| Layer | Source entry points | Responsibility |
| --- | --- | --- |
| Retained model and training | `mod.rs`, `training.rs`, `runtime.rs` | Artifact/schema validation, corpus preparation, learned bounded causal prediction. |
| Exact memory and selected computation | `memory_runtime.rs`, `relation.rs`, `dependent_read.rs`, `typed_routing.rs`, `value_runtime.rs` | Occurrences and retained versions; source/operand selection; selected copying, addition and dependent lookup. |
| Response and persistence | `response_entry_runtime.rs`, `response_runtime.rs`, `completion_runtime.rs`, `snapshot.rs` | Coordinate generated output, observed causal commitment and model-bound session restore. |
| Shared experimental substrate | `addressed_attention/`, `hamming_refinement/`, `hamming_policy/`, `shared_core/` | Exact geometry/memory and discrete execution/learning experiments. Earlier initialized policies and failed pilots remain references, not the latest qualified learner. |
| Reusable learned primitives | `relational_attention/`, `dependent_attention/`, `adaptive_attention/`, `text_attention/`, `recurrent_text/` | Contextual reads, query updates, adaptive read/emit control and variable-length byte/EOS writing. |
| Ordered raw-language representation | `ordered_state/`, `language_relation/`, `relative_language/` | Signed prefix history, canonical words and learned relative source/query binding. |
| Dependent language loop | [dependent_language/](crates/uor-r4-core/src/native_geometric/dependent_language/), especially `runtime.rs`, `scheduling.rs`, `completion.rs`, `phrase.rs` | Carry selected content into subsequent queries and reads; share control and output; distinguish completed from unresolved continuations. |
| Spans and occurrence correspondence | Inside `dependent_language/`: `span.rs`, `span_boundary.rs`, `occurrence.rs`, `correspondence.rs` | Exact byte extents, phrase boundaries and ordered occurrence-specific source witnesses. |
| Contextual roles and participation | [occurrence_role.rs](crates/uor-r4-core/src/native_geometric/dependent_language/occurrence_role.rs), [query_participation.rs](crates/uor-r4-core/src/native_geometric/dependent_language/query_participation.rs) | Learn source/query content-versus-context roles, required query evidence and dependent replacement eligibility. These tables do not directly contain the answer payload. |
| Learning and qualification | `dependent_language/*learning.rs`, role learners and `*_report.rs`; [evidence records](docs/evidence/) | Training-only induction, actual-output credit, retained-output comparisons, interventions and independent transfer evaluation. Report-only oracles are not inference components. |
| Public interfaces | [src/native_geometric_cli.rs](src/native_geometric_cli.rs), [src/native_geometric_service.rs](src/native_geometric_service.rs), [API crate](crates/uor-r4-api/README.md) | Native CLI, local service and library access. Existing historical graph APIs are separate surfaces. |

Start at the [native source directory](crates/uor-r4-core/src/native_geometric/) and [detailed project map](docs/PROJECT_MAP.md) for direct file/result links. Experimental artifacts nest parent artifacts and bind their digests; following that lineage is necessary to determine which parameters are inherited, changed or frozen. A module's existence does not mean the retained artifact contains it.

## Available mechanisms and research

**Available** here means implemented infrastructure, preserved source, imported reference or a documented research proposal, as labeled below. It does not mean “already active in inference,” “compatible unchanged,” or “included in a fresh clone with every external artifact.”

| Family | What is available | Adoption boundary / navigation |
| --- | --- | --- |
| **Prime/angular/geometric routing** | Prime registries, ordered n-lets, overlap/index structures, bounded prime-route attention; older Hopf, Poincare and shell/sector experiments. | Reuse explicit addressing, chart and locality mechanisms. Historical floating-point, transformer and expert-dispatch engines are not the serving design. [Mathematics audit](docs/integration/architecture-2026-09/mathematics.md), [engine audit](docs/integration/architecture-2026-09/engines.md). |
| **SpiralCore and Atomic Ternary FBS** | The [Rust v63 adapter](crates/uor-r4-core/src/spiralcore_operator.rs) implements exact oriented finite actions, Clifford-related generators/bivectors and composition tables. [v68/FBS sources](research/spiralcore-v68/README.md) are separately preserved research. | A learned semantic action and explicit bridge to the native state remain necessary. A newer research revision does not silently replace the v63 adapter. |
| **Harmonics, calculus, bundles and topology** | Spherical-harmonic/heat-kernel scoring ideas, transported-frame and gradient references, phase/chart studies, finite constraint topology and proposed relation graphs. | Distinguish mathematical proposals, ontology declarations, offline numerical references and executable finite operators. No complete general-purpose native R4 triangulation/field-calculus language engine is established. [Foundations](docs/integration/geometric-attention-research-2026-09/mathematical-foundations.md), [import audit](docs/integration/architecture-2026-09/imports.md). |
| **UOR identity and typed execution** | `uor-addr`, UOR-Framework and Prism provide canonicalization, typed carriers, manifests, finite operations and structural verification interfaces. | Use actual [Cargo manifest](Cargo.toml)/[lockfile](Cargo.lock) versions, not legacy `uor_standards/` copies. Hash proximity is not semantics; ontology names are not learned capabilities. [Dependency decisions](docs/integration/architecture-2026-09/imports.md). |
| **Offline arithmetic and dense references** | `uor-matmul`, historical softmax/R4/HELM models and [training tools](tools/r4-softmax-trainer/README.md) provide numerical implementations, curricula, losses and controls. | Useful offline; their matrix products and transformer backbones remain excluded from final inference. Their text-generation results do not transfer to the native model. |
| **TLA/R4G1 graph infrastructure** | Packed formats, deterministic compilers, bounded integer/table kernels, borrowed storage, operation accounting and certification machinery. | Infrastructure donors, not a ready geometric language learner. Preserve their own frozen contracts. [Graph compiler](crates/uor-r4-graph-compiler/README.md), [graph format](crates/uor-r4-graph-format/README.md), [runtime](crates/uor-r4-graph-runtime/src/). |
| **Dormant attention, planning and operators** | Packed route attention, MSA selection, bounded planning/search, tropical composition and other archived finite operator tracks. | Several lack serving callers or have preserved negative qualifications. Grounding a language request into the right state/operator is a separate missing bridge. The [engine audit](docs/integration/architecture-2026-09/engines.md) and [model ledger](model/ledger.toml) identify actual scope. |
| **NEMESIS, W33 and other contributor work** | State/transition fidelity criteria; W33 finite incidence/operators and immutable page/DAG ideas; GoldSnnail state-layout and finite-program references; additional provenance/storage research. | These are source-specific donors, not turnkey replacement models. Some material is in external audit caches; licenses, revisions, negative results and missing representation bridges are recorded in the [contributor audit](docs/integration/architecture-2026-09/imports.md). |
| **NAF/GNAF and formal work** | Typed interchange/witness work, imported proof artifacts, conformance vocabulary and Lean research. | Structural or conditional proofs do not establish language quality or an unproved end-to-end theorem. [GNAF provenance](docs/gnaf_import_provenance.md), [formal vocabulary](docs/formal_vocabulary.md), [Riemann/Lean archive](research/riemann-lean/README.md). |
| **Alternative learning objectives** | Relation-graph traversal and predictive geometric hierarchies, including a JEPA-like auxiliary objective, are documented alternatives/extensions. | Research candidates, not claims of an integrated learner or permission to import a prohibited backbone. [Architecture alternatives and learning synthesis](docs/integration/geometric-attention-research-2026-09/README.md). |
| **Exploratory router, knowledge tooling and Studio** | [Rust router](crates/uor-r4-router/README.md), [local source index](tools/uor-knowledge/README.md) and [companion Studio](https://github.com/Casey-allard/uor-r4-wasm-chat) preserve retrieval/UI/discovery work. | Keep exploratory floating-point/Markov generation, developer knowledge retrieval and actual model memory distinct. Browser integration must execute the identified native artifact, not substitute another backend. |

The [research archive](research/README.md) also preserves prime/semiprime, modular-admissibility, spectral, spin/torsion, networking, benchmarking and formal investigations. Its canonical geometric-router snapshot is under `research/ai-research/ai-router/router-research/`; overlapping copies are historical snapshots, not independent replications. The [architecture source inventory](docs/integration/architecture-2026-09/source-inventory.json) and [README inventory](docs/integration/readme-inventory-2026-09.json) provide deeper navigation. This page maps the major families; it does not claim every archived file or theorem has been audited.

## Current evidence and open problems

### Dated experimental snapshot

At source revision [`edc59b5`](https://github.com/UOR-Foundation/uor-r4/commit/edc59b5fd95f459d3c4a528dad910f8391c08e12), September 14, 2026:

| Question | Recorded result |
| --- | --- |
| Can the experimental path compose contextual reads and phrase updates? | Bounded learned read/update/scheduling/span mechanisms are integrated; their individual scopes and controls are linked in the [project map](docs/PROJECT_MAP.md). |
| Does the latest independent lexical-neighbor panel pass? | **No: `FAIL_INDEPENDENT_NEIGHBOR_TRANSFER`, 2,016/2,304.** All 288 failures involve an interior *will* in an unfamiliar three-word name. |
| Were earlier outputs preserved in that step? | The report records exact retention of the 492-, 300-, 200- and 6,688-output populations, including 512 typed unresolved outcomes. These are retained comparison populations, not a claim of that many independent semantic tasks. |
| Are there causal and reload controls? | Full and ExactIdentity agree on 2,304/2,304, including failures; reload agrees on the same panel. Read/update-disabled controls yield zero correct answered dependent outputs. That does not by itself qualify the meaning of every unresolved outcome. |
| Does this show a geometric predictive advantage? | Not on this panel: ExactIdentity has the same outcomes. Geometry's architectural role, correctness and comparative predictive benefit remain distinct claims. |
| Is the frontier/local-hardware objective achieved? | **No.** Broad prose, discourse, coding/reasoning, scaling and complete-path energy savings remain unqualified. |

These are **reported repository results**, not experiments rerun by this documentation update. The [full independent-neighbor report](docs/native_geometric_independent_neighbor_973.md) and [machine-readable evidence](docs/evidence/native_geometric_independent_neighbor_973.json) contain acceptance, controls, resources and the unrun limits. That panel was independent at first evaluation; it is now exposed development evidence.

**Historical correction:** the September 8 recovery withdrew the earlier published alpha qualification and numerical performance comparisons after identifying false qualification checks and disconnected serving paths. The [recovery record](docs/integration/recovery-2026-09-08.md) remains part of the evidence. UI readiness and historical reference-model outputs do not reverse that withdrawal.

### What must be solved next

The immediate research boundary is **context-sensitive role separability** while preserving existing source/query correspondence, exact identities and successful outputs. The latest record calls for matched role controls and a causal test of additional contextual evidence before learning a correction. It does not authorize inserting exposed names or copying a query rule into the source table without testing its effect.

The broader sequence is to develop reusable contextual learning and emission, establish sustained language and compositional reasoning/coding, connect the shared core to the retained memory/session capabilities, and qualify the same model across interfaces and complete-task resource measurements. A successful next bounded test is progress toward that program, not its completion.

[Current state](docs/integration/current-state.md) owns the precise next action and artifact/resource receipts. The [canonical plan](docs/integration/project-track.md) owns sequence and acceptance; [#973](https://github.com/UOR-Foundation/uor-r4/issues/973) tracks integrated language learning, [#964](https://github.com/UOR-Foundation/uor-r4/issues/964) the serving constraint, and [#820](https://github.com/UOR-Foundation/uor-r4/issues/820) the overall program. This README does not create a second execution queue.

## Repository and documentation map

| Start here | What it answers |
| --- | --- |
| [Canonical plan](docs/integration/project-track.md) | What are we building, under which constraints, and in what sequence? |
| [Current state](docs/integration/current-state.md) | Which artifact and experiment are current? What passed, failed or remains unrun? |
| [Project map](docs/PROJECT_MAP.md) | Which source files, result records and retained engines implement each responsibility? |
| [Capability assessment](docs/integration/model-direction-2026-09.md) | What does the implemented model establish, and what remains missing? |
| [Geometric-attention synthesis](docs/integration/geometric-attention-research-2026-09/README.md) | How do the mathematical mechanisms, learning choices and alternatives fit together? |
| [Architecture audit](docs/integration/architecture-2026-09/README.md) | What exists across the broader code/research/import inventory? Its dated “next” statements are historical. |
| [Research results](docs/RESEARCH.md) and [evidence](docs/evidence/) | Where are measured results, exact bindings, controls and preserved negative findings? |
| [Research archive](research/README.md) and [documentation index](docs/README.md) | Where are the original snapshots, mathematical studies, references and technical documents? |
| [Native workflow](docs/native_geometric_workflow.md) and [API crate](crates/uor-r4-api/README.md) | How do the current native CLI, artifacts and session interfaces work? |
| [Contribution guide](CONTRIBUTING.md), [agent instructions](AGENTS.md), [continuation guide](docs/integration/CONTINUE.md) | How should work preserve research, respect resources and reach protected delivery? |

At the top level, `crates/` and `src/` contain executable libraries/interfaces; `docs/` contains specifications and evidence; `research/` preserves the broader program; `model/` and `tools/` retain compiler/training histories and development utilities. `proofs/`, `features/` and `uor_standards/` have their own formal, compatibility or historical roles. They are not additional qualified language models.

## Getting started

```sh
git clone https://github.com/UOR-Foundation/uor-r4.git
cd uor-r4
~/.cargo/bin/cargo run --bin r4 -- geometric --help
```

Use the pinned [Rust toolchain](rust-toolchain.toml). The command discovers the native interface; it does not download a qualified assistant or establish model quality. Actual generation needs a compatible learned artifact and the command/configuration documented in the [native workflow](docs/native_geometric_workflow.md).

**A clone is source, not the complete local experiment.** Ignored `.uor-models/` and `.uor-handoff/` material includes retained candidates, data, sealed traces and resource receipts referenced by the result records. Some tracked research media also require Git LFS payloads. Follow exact artifact identities and recorded prerequisites; do not assume another host has the original author's local paths or that recreating a missing file reproduces its evidence.

The native capability API contract is `uor-r4.native-capability-api/2`; see [migration and loading requirements](docs/integration/recovery-2026-09-08.md#api-version-migration). Package, CLI and artifact schema identifiers are preserved. Historical `r4 demo`/`r4 route` examples and the [model lifecycle](docs/MODEL_LIFECYCLE.md) refer to other mechanisms; they are not shortcuts to the current experimental model.

The intended product path is one useful native model through its CLI/library/local service, followed by that same identified artifact running in a WASM worker behind Studio. A static GitHub Pages interface, a mock response or an external model backend is not evidence that this deployment has been achieved.

## Contributing and research continuity

Contributions should connect a concrete mechanism to the native objective, retain the runtime contract and demonstrate its effect through the actual execution path. Preserve exact content/occurrence/version identity, causal state changes and the distinction between candidate admission, selection and emission. Report generated behavior, not only loss, compilation or correctly named geometry.

Keep development evaluation separate from final independent evaluation after design selection. Preserve failed candidates and successful prior paths; bind claims to source, artifact, data, operators, controls and resources. Compare geometry with meaningful exact-identity or alternative-representation controls rather than assuming that structural elegance guarantees predictive benefit.

Project complete build/preparation/training/evaluation/retry/storage costs before model work. A new task or clone does not reset the original experiment's cumulative ledger. Documentation-only changes do not require a training run. When editing capability claims, run `python3 scripts/check_claim_wording.py` and state the validation scope.

Deliver through protected pull requests. The historical PR/merge-queue status names are **compatibility acknowledgements, not tests**; actual focused checks supply validation. Update the owning plan/state/source map when implementation changes, without rewriting historical outcomes to simplify the narrative.

Project code is under [MIT](LICENSE). Imported research and dependencies retain their own attribution, provenance and applicable licenses; inclusion in this repository does not relicense them.
