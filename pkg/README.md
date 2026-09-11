# UOR-R4 Geometric Language Model

**An experimental autoregressive geometric state model with exact addressed memory and learned typed operators. Built in Rust for local execution.**

We are developing a language model for useful prose, conversation, memory, reasoning and Rust coding on consumer laptops. The ultimate objective is frontier-level capability on an M1-class machine while reducing energy and wasted computation. The project is **pre-alpha**; that objective has not been achieved.

The architecture uses prime-addressed ordered context, fixed zeta-zero phases, R4/S3/H4 state and transport, exact `Z[phi]`, signed orientation and typed UOR identity. Offline Rust training may use matrix multiplication. Final serving must execute **no mathematical matrix products and no transformer backbone**, using learned geometric transitions, shared typed operators and bounded integer/table execution. Deterministic geometric address/page selection is allowed. Expert gates remain a conditional future option, not the current design. The name describes the model; existing Cargo package and CLI identifiers and the native model artifact schema are preserved. The repaired API contract is explicitly versioned as `uor-r4.native-capability-api/2`; [migration and usage](docs/integration/recovery-2026-09-08.md#api-version-migration) describe optional measurements and required artifact loading.

**September 8 recovery:** the published alpha qualification and numerical performance comparisons are withdrawn. [The audit and recovery record](docs/integration/recovery-2026-09-08.md) explains the false qualification checks, disconnected serving paths and preserved baseline. The repository includes Studio code, but browser integration does not establish model capability.

## What exists today

| Capability | Current evidence |
|---|---|
| Inference | Native artifact loading, sessions, observe/predict, selected operators and bounded generation exist |
| Attention | Bounded learned contextual source selection, copying and causal context controls exist |
| Memory and computation | Exact retained versions, selected arithmetic, dependent reads and some eviction/restart behavior pass at bounded scope |
| General prose | **Not established**; short authored answers are not sustained language generation |
| General reasoning and coding | **Not established**; reusable primitives and bounded cases exist |
| Laptop efficiency | Scoped integer/table, allocation and kernel measurements; full-task energy and broad submillisecond execution are unmeasured |
| AI Studio | Explicit artifact loading and actual browser generation/checkpoint/cancellation checks exist; the model is not qualified as a general assistant |

[Current state](docs/integration/current-state.md) binds the exact retained artifact and newest result. [The capability assessment](docs/integration/model-direction-2026-09.md) explains the evidence and remaining gaps. Historical softmax/R4 transformer references can generate text, but that result does not transfer to the native geometric model.

## Where the project goes next

The [shared nonnumeric emission result](docs/native_geometric_word_emission_973.md) connects exact words/spans to the existing signed-H4 lexical selector. The [contextual writer correction](docs/native_geometric_writer_lexical_973.md) distinguishes the tested instruction/fact collision, and [owner/value field composition](docs/native_geometric_field_composition_973.md) lets that same selector place an exact stored owner before its value under explicit owner-first requests. General prose and program synthesis remain unqualified. The [owner/revision repair](docs/native_geometric_owner_revision_973.md) now includes [writer-role transfer across value primes](docs/native_geometric_writer_role_transfer_973.md), repairing the tested newest-fact binding while preserving complete revision chains. The earlier [current-version source candidate](docs/native_geometric_current_version_read_973.md) remains rejected. The [reader/writer integration](docs/native_geometric_current_source_integration_973.md) improves the tested current-version selection while preserving that writer state. The [query-owner refinement](docs/native_geometric_query_owner_selection_973.md) repairs the measured competing-chain selection tie using the same geometric code vocabulary. The [historical-read result](docs/native_geometric_historical_selection_973.md) is now integrated with [reverse phrase-start transfer](docs/native_geometric_reverse_start_transfer_973.md): the retained combined model recovers complete stored values and passes the bounded fresh/preservation gates. [Historical owner/value composition](docs/native_geometric_historical_field_973.md) now learns past-tense emission from the exact selected old version while preserving current-record safeguards and retained behavior. [Contextual historical selection](docs/native_geometric_historical_query_context_973.md) now repairs the tested longer historical owner-first forms and competing-owner selection while preserving current-intent deferral. The [contextual current-record handoff](docs/native_geometric_current_query_handoff_973.md) now repairs the tested longer current requests while preserving genuine dependency and factual-word roles through exact source-occurrence context. The [learned version intent](docs/native_geometric_historical_version_973.md) now selects the exact initial version through validated ancestor chains with owner-role masking, and its [truncated-history abstention](docs/native_geometric_truncated_history_abstention_973.md) answers `Unknown.` when later facts have evicted the requested root, preserving previous/current answers. The [reassertion-versus-truncation repair](docs/native_geometric_reassertion_chain_repair_973.md) answers an independent review: a resident same-value reassertion is a validated link, a chain is truncated only when its predecessor is proven evicted, and report directories are created exclusively. The [head contract](docs/native_geometric_head_reassertion_973.md) then makes a repeated current fact keep the retained history readable: the first hop below a same-value reassertion head is valid, previous is the head's immediate previous record and initial is the root. Next under #973 learn intermediate-version intent with exact depth labels. [Current state](docs/integration/current-state.md) owns artifact identities, measured results and resource receipts.

[The canonical plan](docs/integration/project-track.md) owns order and acceptance. [ROADMAP.md](ROADMAP.md) provides the issue navigation; [#820](https://github.com/UOR-Foundation/uor-r4/issues/820) is the live programme tracker. Priorities guide development, while actual evidence and dependencies determine the next useful step.

## Find the code, research and evidence

Start with [the project map](docs/PROJECT_MAP.md). It distinguishes these surfaces:

- [Native model](crates/uor-r4-core/src/native_geometric): Rust learning, exact memory/state, artifact/session and inference implementation.
- [Native CLI/workflow](docs/native_geometric_workflow.md) and [API crate](crates/uor-r4-api/README.md): existing interfaces and their actual scope.
- [Architecture and source audit](docs/integration/architecture-2026-09/README.md): all discovered engine families, mathematical mechanisms and contributor sources, with source links and limitations.
- [Research directory](research/README.md), [research results](docs/RESEARCH.md) and [documentation index](docs/README.md): imported work, dated positive/negative results and technical references.
- [Local knowledge tooling](tools/uor-knowledge/README.md): provenance-bound discovery, distinct from model memory. Ignored `.uor-models/` artifacts are not included in a fresh Git clone.
- [Separate Studio repository](https://github.com/Casey-allard/uor-r4-wasm-chat): reusable product interface; inspect its live implementation before integration.

## Getting started

```sh
git clone https://github.com/UOR-Foundation/uor-r4.git
cd uor-r4
~/.cargo/bin/cargo run --bin r4 -- geometric --help
```

Use the pinned [Rust toolchain](rust-toolchain.toml). The help command discovers the native interface; it is not a model-quality check. Actual generation requires a compatible local learned artifact. Follow [the native workflow](docs/native_geometric_workflow.md) and current-state pointer for existing artifacts and command scopes. Project complete build/model resource requirements before a training run; a new clone does not supply the local checkpoints or reset cumulative budgets.

The historical `r4 demo` and `r4 route` surfaces illustrate earlier router/dashboard mechanisms. Their availability is distinct from qualification of the current native model. Preserved CLI, HTTP and reference examples remain in the [pre-reconciliation README](https://github.com/UOR-Foundation/uor-r4/blob/bc03f2d7ffde99608da370808eca542360e54508/README.md), [lifecycle](docs/MODEL_LIFECYCLE.md) and [configuration reference](docs/CONFIGURATION.md).

## Contributing and continuity

Read [CONTRIBUTING.md](CONTRIBUTING.md), [AGENTS.md](AGENTS.md) and [the continuation instructions](docs/integration/CONTINUE.md). Reuse existing history, make a concrete native improvement, exercise the affected path and deliver through protected pull requests. Preserve negative results and distinguish proof, measured behavior and hypothesis.

Protected PR/merge-queue status names are compatibility acknowledgements and execute no tests. Focused local checks provide actual validation; broader suites remain available for explicitly relevant release work. Documentation reorganization adds no new model capability result.

Project code is under [MIT](LICENSE); imported sources retain their own provenance and applicable licenses. Historical product names and upstream READMEs are preserved rather than relabeled as our implementation.
