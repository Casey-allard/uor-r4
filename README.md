# UOR-R4 Geometric Language Model

UOR-R4 is a research language model written in Rust. It asks whether geometry
(quaternion transport, lattice codes, hyperbolic and harmonic similarity) can
replace the parts of a transformer that are expensive at run time. The aim is a
model with frontier-level quality that is stored on, and runs from, a 16 GB
laptop.

**Status: pre-alpha research.** No model in this repository holds a useful
conversation yet. General prose, reasoning, coding, frontier equivalence and
energy savings are not established. The [status page](STATUS.md) gives the
measured position of every active line of work.

## Approach

The work runs on two tracks that share one set of geometric mechanisms. The
[continuation plan](docs/labs/plan-2026-09-29.md) sets their dependencies and
evidence gates.

- **Track A: a native geometric chat model.** A small (≤ 30M parameter) model,
  trained from scratch and served without floating point or a hardware
  multiplier. Its first milestone is a sealed conversation panel: multi-turn
  replies, recall of updated facts, and new instruction wordings.
- **Track B: geometric conversion of pretrained transformers.** Distill open
  models (SmolLM2 135M → 360M → 1.7B) into a geometric runtime, and measure the
  quality gap to the teacher and the bytes and operations per token.
  - It is approved in D13 and continues under D14's shared workflow.
  - Under D11 the source transformers stay offline teachers and comparators.
  - A converted model is served only if it meets D11.

The four geometric mechanisms under test:

| Mechanism | Idea | Status |
| --- | --- | --- |
| Flock attention | Each query selects a sink, a local window and its *k* nearest keys. A pointer/copy path must separately preserve the selected payload | Being built (A1, B0) |
| Spherical-harmonic attention | Harmonic features of normalized queries and keys give a fixed-size recurrent state | Planned (B2) |
| Quaternion / 2I transport | Recurrent state carried by unit quaternions, snapped to the 120-element binary icosahedral group | Trained into the main-line model; D11 kernel in review |
| E8 / 2I lattice codes | Weights stored as lattice codewords and read by table lookup | B3 has a reported negative awaiting review; the geometric toolbox is preserved |

A geometric mechanism stays when it is within 0.02 nats of its ordinary matrix
equivalent. The rule was set by the owner on 29 September and is judged on
paired arms with at least two seeds.

## Architecture

- **The geometric stack** ([`geometric_stack.rs`](crates/uor-r4-training/src/geometric_stack.rs))
  interleaves quaternion-transport recurrence layers (`r`) and multi-head reads
  with a Lorentz or dot score (`a`). A SwiGLU MLP follows each layer. An
  ordinary transformer control runs on the same kernels.
- **Memory:** an exact, addressed store with entity/relation token keys (AERM). It
  exists today as a probe, is not yet in the served model, and becomes an index
  into an exact log of the conversation.
- **Serving (D11):** the integer engine ([`uor-r4-integer`](crates/uor-r4-integer/README.md))
  runs 4-bit weights with integer add, shift, compare and table reads only: no
  floating point, no multiply or divide instruction, and an instruction-level
  audit of the binary.
  - Every weight is still read for every token.
  - A measured product-table emulator used 4.3× the energy of its float comparator;
    no general energy advantage is established.
- **Training:** offline in Rust (Candle), with floating point allowed.
  Quantization-aware training reads exactly the values the export writes.

## Repository map

| Path | Contents |
| --- | --- |
| [`crates/uor-r4-training`](crates/uor-r4-training) | The geometric stack, memory, training worlds, export, and the experiment drivers in `examples/` |
| [`crates/uor-r4-integer`](crates/uor-r4-integer) | The multiplier-free serving engine and the `uor-r4-stack` and `uor-chat` binaries |
| [`crates/uor-r4-lut`](crates/uor-r4-lut) | The serving artifact format |
| [`crates/uor-r4-model-source`](crates/uor-r4-model-source) | The exact CPU reference for teacher models (Llama/SmolLM2, GPT-2) and the attention-replacement seam |
| [`crates/uor-r4-core`](crates/uor-r4-core) | Geometric primitives (R4/S3/H4, exact `Z[φ]`, prime addressing), the earlier native learner and sealed report output |
| [`crates/uor-r4-tokenizer`](crates/uor-r4-tokenizer) | The shared byte-level BPE tokenizer |
| `crates/uor-r4-graph-*`, `uor-r4-router` | The frozen TLA/R4G1 graph runtime and certified routing, under their own scoped contracts |
| [`docs/`](docs) | Plans, decisions, results and the full research history |

The [project map](docs/PROJECT_MAP.md) covers every crate and historical engine.

## Quick start

Rust is pinned by `rust-toolchain.toml` (1.97.1), and rustup selects it
automatically. Training data and model artifacts are local material and are
not in the repository.

```sh
cargo build --release -p uor-r4-training --example geometric-stack
cargo build --release -p uor-r4-integer --bin uor-r4-stack
cargo test -p uor-r4-training --lib geometric_stack
```

- `geometric-stack` trains, evaluates and exports stacks. Run it without
  arguments to list its modes.
- `uor-r4-stack generate ARTIFACT PROMPT_IDS NEW_TOKENS` serves an artifact
  written by `geometric-stack export` under D11. `PROMPT_IDS` is a
  comma-separated list of token IDs.
- The interactive `uor-chat --bundle <dir>` serves the earlier integer model.
  Its stack profile is not built yet.

## How the work is run

Any number of labs may join and leave, using GitHub for durable work and evidence.
Renewable claims and an independent council replace a permanent provider lead.
The owner retains the mission, evidence, unique-data and spending boundaries.

- [Lab entry and extended goals](docs/labs/README.md): the shared workflow,
  continuation plan, resource cadence, recovery and client instructions.
- [STATUS.md](STATUS.md): one row per lab, with the current item and the latest result.
- [ROADMAP.md](ROADMAP.md): active dependencies and preserved historical plans.
- [DECISIONS.md](docs/integration/DECISIONS.md): owner and delegated council decisions.
- [Current state](docs/integration/current-state.md): measured results and artifacts.
- The [programme tracker #820](https://github.com/UOR-Foundation/uor-r4/issues/820)
  and epics [#1508](https://github.com/UOR-Foundation/uor-r4/issues/1508) (Track A),
  [#1509](https://github.com/UOR-Foundation/uor-r4/issues/1509) (Track B) and
  [#1510](https://github.com/UOR-Foundation/uor-r4/issues/1510) (infrastructure).
- [AGENTS.md](AGENTS.md): the operating rules for every contributor and agent.

**Evidence rules.**
- Numbers come from committed code run into sealed report directories. Label
  self-reported, independently read and independently rerun evidence separately.
- Experiments are pre-registered with a gate and a kill criterion. A kill ends
  an experiment, not a mechanism family (D12).
- Negative results keep their exact scope.
- Claims follow the [formal vocabulary](docs/formal_vocabulary.md).

## History

The research began with exact geometric memory and a transformerless prose
learner, then moved to learned recurrent models with integer serving. The
[README as of 29 September 2026](https://github.com/UOR-Foundation/uor-r4/blob/0fb03372/README.md)
keeps the dated results log. The [evidence index](docs/integration/EVIDENCE.md)
and the [research ledger](docs/RESEARCH.md) cover the full history.

## License

[MIT](LICENSE). Third-party research and dependencies keep their own licenses.
