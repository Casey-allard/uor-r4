# R⁴ — Geometric Intelligence on Local Hardware

[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/built%20with-Rust-orange.svg)](rust-toolchain.toml)

R⁴ is an open research project building a **transformerless local AI agent**.
Its goal is to replace transformer attention, mixture-of-experts routing, and
dense learned matrix operations in the serving path with deterministic
geometric routing and lookup.

That is a very real engineering goal, not a claim that the goal has already
been reached. The long-term target is frontier-like capability on ordinary
local hardware. The project is testing whether language context, inference,
and reasoning can emerge from routes through a canonical geometric memory. The
target serving engine uses no Ollama, hosted model, or source-model weights.

> **Honest status:** the geometric storage, identity, route, and bounded-lookup
> foundation exists. Source-free geometric attention, dependable text
> generation, correct answers, and reasoning do not exist yet. The current
> dashboard is an interactive window into the research substrate, not a
> frontier model or a ChatGPT replacement.

## Try the project

With Git and a current Rust toolchain installed:

```bash
git clone https://github.com/UOR-Foundation/uor-r4.git
cd uor-r4
cargo run --bin r4 -- demo
```

Open <http://127.0.0.1:8000>.

The dashboard lets you interact with the existing geometric router and inspect
its state. It is the quickest way to see the project in motion without
downloading or compiling a language model. A first Rust build may take longer
than five minutes on some machines; later launches reuse it.

To inspect one route from the command line instead:

```bash
cargo run --bin r4 -- route "geometry is the route"
```

Both commands exercise the no-model research substrate. `demo` does not start
the historical artifact-discovery server, and `route` does not claim to answer
the prompt; it exposes how the current geometry represents it.

The browser-only WASM surface is published at
[uor-foundation.github.io/uor-r4](https://uor-foundation.github.io/uor-r4/).
With `just` and `wasm-pack` installed, `just wasm-dashboard` builds and serves
the same local surface without model weights.

## What R⁴ is trying to build

The central hypothesis is simple:

> **The geometry is the route, and the data is the location.**

Text is reversibly assigned to canonical geometric addresses. As a sequence
unfolds, its route carries local and accumulated context. A bounded geometric
query evaluates possible next locations, chooses an admitted least-cost route,
and decodes that location back to text.

```text
text
  → reversible lexical address
  → prime / semiprime route
  → spin, phase, torsion, and radial state
  → current + sentence + conversation + global context
  → bounded next-route selection
  → text
```

The working design brings together:

- primes and semiprimes as addressable atoms and route experts;
- spherical harmonics as the working description of related spin states;
- fixed zeta-zero channels with changing phase and torsion;
- S³/R⁴ transport, Hopf projection, and golden-ratio radial shells;
- a paired-H4/E8 bridge for coupled geometric state; and
- recursive context at route, sentence, paragraph, conversation, and global
  scopes.

Kappa provides canonical identity and serialization. It is not itself the
tokenizer, semantic distance, attention mechanism, or language model. A pinned
lexical codec supplies reversible text boundaries; the intelligence must come
from the geometry.

## What exists now

The current foundation can represent and rebuild prime-route state, preserve
transported trajectory and overlapping context summaries, and perform bounded
deterministic candidate lookup.

It has **not** yet demonstrated:

- prompt-to-answer source-free chat;
- recursive geometric attention that generalizes beyond recall;
- grammatical generation from geometry alone;
- correctness and calibrated abstention;
- multi-step reasoning; or
- frontier-class capability or an energy advantage.

Earlier compiler, graph, proof, conformance, and teacher-derived systems remain
in the repository as research evidence and reusable components. They are not
the current product path and are not prerequisites for trying the dashboard.

## Current roadmap

The programme is deliberately sequential so that infrastructure and testing do
not become substitutes for working intelligence:

1. **Make language reversible in geometry** — lexical codec, canonical route
   hierarchy, serialization, and reconstruction.
2. **Build geometric attention** — current, previous, sentence, paragraph,
   conversation, and global route context must causally affect selection.
3. **Generate coherent text** — source-free grammar and next-route decoding.
4. **Establish correctness** — relevance, contradiction handling, and honest
   abstention.
5. **Establish reasoning** — bounded multi-step route composition.
6. **Connect and ship the accepted engine** — chat integration, measured
   optimization, and only then release QA.

The CLI and WASM dashboard remain usable research surfaces throughout this
sequence so each new mechanism can become visible before the final engine is
complete.

The active dependency chain is tracked in
[#820](https://github.com/UOR-Foundation/uor-r4/issues/820). The immediate
implementation stage is
[#961](https://github.com/UOR-Foundation/uor-r4/issues/961).

## Find your way around

- `src/` — the `r4` executable, local server, chat shell, and WASM surface.
- `crates/uor-r4-core` — current geometric route/manifest foundation plus
  preserved runtime research.
- `crates/uor-r4-router` — geometric router, memory, and dashboard backend.
- `crates/uor-r4-graph-*` — preserved graph-format/compiler/runtime research.
- `docs/` — current programme, mathematical decisions, evidence, and archive.

Start with the [documentation guide](docs/README.md). The
[Geometric Intelligence Programme](docs/geometric_intelligence_programme.md)
is the architecture and sequencing authority. Historical records remain
available through the documentation guide without dominating the front door.

## Contributing

This is an obscure and ambitious research problem, and useful contributions
are welcome. The most valuable work advances the first unblocked roadmap stage
and produces an observable user-facing capability. Expensive experiments and
broad QA stay dormant unless a current decision truly requires them.

Read [CONTRIBUTING.md](CONTRIBUTING.md) before opening a change.

## License

MIT — see [LICENSE](LICENSE). © 2026 UOR Foundation.
