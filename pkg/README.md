# UOR-R4 Geometric Language Model

**An experimental autoregressive geometric state model with exact addressed memory and learned typed operators. Built in Rust for local execution.**

We are developing a language model for useful prose, conversation, memory, reasoning and Rust coding on consumer laptops. The ultimate objective is frontier-level capability on an M1-class machine while reducing energy and wasted computation. The project is **pre-alpha**; that objective has not been achieved.

The architecture uses prime-addressed ordered context, fixed zeta-zero phases, R4/S3/H4 state and transport, exact `Z[phi]`, signed orientation and typed UOR identity. Offline Rust training may use matrix multiplication. Final serving must execute **no mathematical matrix products and no transformer backbone**, using learned geometric transitions, shared typed operators and bounded integer/table execution. Deterministic geometric address/page selection is allowed. Expert gates remain a conditional future option, not the current design. The name describes the model; existing Cargo packages, CLI names and artifact schemas remain compatible.

## What exists today

| Capability | Current evidence |
|---|---|
| Inference | Native artifact loading, sessions, observe/predict, selected operators and bounded generation exist |
| Attention | Bounded learned contextual source selection, copying and causal context controls exist |
| Memory and computation | Exact retained versions, selected arithmetic, dependent reads and some eviction/restart behavior pass at bounded scope |
| General prose | **Not established**; short authored answers are not sustained language generation |
| General reasoning and coding | **Not established**; reusable primitives and bounded cases exist |
| Laptop efficiency | Scoped integer/table, allocation and kernel measurements; full-task energy and broad submillisecond execution are unmeasured |
| AI Studio | Separate reusable UI/WASM infrastructure; the current retained native artifact has not been qualified in Pages Studio |

[Current state](docs/integration/current-state.md) binds the exact retained artifact and newest result. [The capability assessment](docs/integration/model-direction-2026-09.md) explains the evidence and remaining gaps. Historical softmax/R4 transformer references can generate text, but that result does not transfer to the native geometric model.

## Where the project goes next

The [contextual writer result](docs/native_geometric_writer_refinement_1139.md) selects `79710468`: all 232 writer labels, 32 new construction answers, 12 open answers and 76 prior phrases pass, followed by 12 fresh evicted phrases and four short boundary cases after selection. Learned separator/adjacency context repairs the missing multiword writes while preserving all 663 retained answers. A matched check also exposes 41 preexisting reader failures among 200 historical writer anchors; those outputs remain unchanged. The next direct implementation is a shared state transition in which a second operator consumes the actual first committed result, followed by compositional emission. The model must progress beyond copied-answer patterns.

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
