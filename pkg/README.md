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

> **Honest status:** the geometric storage/identity foundation, one bounded
> causal R4/S3 path selector, and reusable provider-free decode/render/append
> plumbing exist. The first #953 smoke was an exact lexical relabel of #969, so
> it did not qualify a natural grammar loop. `PrimaryThenAdjacentSpinFallbackV1`
> repaired the frozen agreement admission to exact `{still}` then `{run,runs}`
> support under equal work, but the one permitted four-arm run chose `still run`
> for both full-path prompts and `still runs` for both state-disabled prompts.
> The frozen `LocalSameObjectContextPlacementV1` preflight then reproduced 7/7
> construction prototypes with zero class collisions and zero
> padding-identity aliases, but real placement selected 0/2 intended candidates
> while the same-artifact placement-permuted and order-shuffled controls selected
> 2/2 and 1/2. Generation and replay were `NOT_RUN`; the terminal remains
> `REVISE_I1_GENERATOR_IN_PLACE`. #953 awaits a newly frozen maintainer plan;
> #973 and #954 remain blocked.
> Higher-scope attention, correct answers, and reasoning do not exist yet. The
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

To run the one fixed canonical-ingestion witness:

```bash
cargo run --bin r4 -- lexical-ingestion-witness
```

To reproduce the bounded A1R associative ordered-summary decision:

```bash
cargo run --bin r4 -- associative-ordered-summary-a1r-probe
```

To reproduce the corrected A1P paired-H4-derived exact R4-heatmap
identifiability decision:

```bash
cargo run --bin r4 -- candidate-relative-identifiability-a1p-probe
```

To run the #953 decoded loop against a canonical route artifact:

```bash
cargo run --bin r4 -- bounded-geometric-generate \
  --artifact /path/to/canonical-route.json \
  --prompt "active agile athletes run" \
  --continuation-cap 2 --json
```

This research command loads no provider or source weights. It currently accepts
only a canonical artifact whose embedded construction/global input can fully
reconstruct the parent codec registry; subset-observation artifacts fail closed.
Plain output labels both the appendable continuation and typed stop reason;
`--json` emits the full deterministic witness. Trailing prompt whitespace is
also rejected fail closed so the lexical-boundary contract cannot silently
rewrite the prompt. The command is bounded to that reconstructed vocabulary and
the local #969 path; it is not `ask`, `chat`, or a correctness-qualified answer
surface.

The A1R command uses only the frozen construction/evaluation fixture and exact
finite tables. Its frozen report kappa is
`blake3:f0db7a5d5c81d51ebf3b4bf8a2715c4960ec16b14161e8bf7598d7b98c48c881`.
The associative state passed the declared scope, independent-global, fold,
incremental, and support invariants. The full arm produced distinct `ll`/`rr`
relative states on all 6 queries, but shortest Cayley distance mapped both to
energy 2 and tied every query. The terminal verdict is `RETAIN_STATE_ONLY`: it
does not generate text or establish full attention.

The A1P command preserves those six queries as regression-only evidence,
prepares construction and sealed-validation geometry/support without labels,
and derives S4 parity from each exact history and the frozen role order before
joining the separate label ledgers. Its paired contract computes
`X=C(H,c)`, `Y=C(P_c,c)`, and `D=X*Y^-1` in the signed `(1,i)` R4 chart. The
exact endpoint rule is `sin=±1, cos=0 -> 1` with chirality retained and
`sin=0, cos=±1 -> 0` with cosine polarity retained; `q0=q1=0` is typed-null
abstention, not a threshold shortcut. `q2` and `q3` remain in the full `D`
witness but are not scorer-key fields.

The target-free structural census covers 120×120 = 14,400 ordered pairs, 120
relative rows, 45 exact heatmap classes, and 480 typed-null pairs. Across 36
fixture decisions, 14 classes were exercised; construction coverage was 12/12
and pure, construction classes covered 10/12 validation decisions, the
no-class-splitting oracle ceiling was 10/12, strict construction transfer was
0/6, and eight heatmap classes were incompatible. The hard gate therefore
stops before scalar search; every downstream selection, control, and placement
row is `NOT_RUN_IDENTIFIABILITY_HARD_STOP`, not PASS. Its terminal literal is
`RETAIN_H4_STATE_ONLY_ADVANCE_MULTICHANNEL_A1Q`. Contract, universe, and report
kappas are
`blake3:2daacf538c022fab9580d1e124af6c18d0b06da04604fbc962a01bda57f08a98`,
`blake3:dca725c0ec6060166bcd0023df956e1ff029661b5fa7800ccb9f20808712b796`,
and `blake3:5f9239150dea8c0c27c4dfa6ad2e4d0068bc3d18afc127b315c0ec358ceddb3f`.
This negative is bounded to the paired-H4-derived heatmap readout. Fixed-zeta
phases, ordered n-lets, exact `phi` radial transport, and the typed
`sqrt(2) <-> 2i <-> [0,2]` adapters remain structural under
`STRUCTURAL_BINDING_ONLY_NO_ZETA_NLET_TO_PHI_EXPONENT_RULE`; they are not
scorer inputs. It does not establish attention or generation, and #969 becomes
the next stage only after protected #970 merge. #969 has since delivered one
bounded causal path selector. #953 has driven it through real decoded-loop
plumbing and tiered admission on the frozen preflight, but the natural agreement
run made the same full-path choice for both prompts and did not qualify a
natural grammar result.

The ingestion witness maps two turns of text through the pinned lexical codec,
prime/spin route state, canonical hierarchy manifest, strict reload, and exact
lexical reconstruction. It also exercises the declared fail-closed unknown-unit
path. It loads no model and establishes reversible state plumbing only—not
attention, inference, correctness, or reasoning.

The additive serving envelope is
`uor-r4.canonical-lexical-route-manifest/1`; it transitively embeds the frozen
`uor-r4.prime-route-spin-manifest/2` bytes. Its codec identity is
`uor-r4.unicode-lexical-runs/1`: UTF-8 identity normalization, caller-declared
sentence/paragraph/turn boundaries, canonical surface-byte vocabulary order,
and rejection of unknown units before mutation. The parent keeps the complete
codec route-address registry in stable lexical-unit order; the unchanged child
manifest contains only addresses witnessed by its causal sentences. The fixed
input ceiling is 8 turns, 32 paragraphs, 31 sentences, 128 units per sentence,
512 total units, and a 64-unit content-addressed global snapshot.

Downstream code consumes `CanonicalRouteArtifact::decode_canonical`,
`attention_consumer_trace`, `attention_consumer_trace_for_cursor`,
`attention_consumer_trace_with_ordered_h4`,
`incremental_update_trace`, `incremental_cursor`,
`lookup_shared_class_trace`, `scope_ceilings`, and `reconstruct_input`. The
attention handoff is ordered current, previous, last-two, sentence, paragraph,
conversation, then bounded global; the cursor resolver returns those same seven
slots and marks not-yet-established boundaries absent. S0 serializes state and
numeric geometry only: every candidate row ceiling is zero and marked
`NOT_IMPLEMENTED_S0_STATE_ONLY`. #952 established candidate/value reachability
but found its reusable summaries order-erasing. #967 landed the exact ordered
state repair but retained it as state only after the candidate tie. #970's
corrected paired-H4-derived exact R4-heatmap gate stopped at bounded readout
identifiability without searching another scalar. #969 then qualified one local
causal path selector, and #953 implemented the first bounded decoded
library/CLI plumbing. Its relabelled smoke terminated
`REVISE_I1_GENERATOR_IN_PLACE`. `PrimaryThenAdjacentSpinFallbackV1` then
recovered exact `{still}` then `{run,runs}` primary support while consulting
and truthfully tracing adjacent-spin rows, which remained non-admitting until
the primary tier was empty. The one permitted four-arm run produced `still run`
for both full-path prompts and `still runs`
for both state-disabled prompts, with deterministic replay. The terminal
remains `REVISE_I1_GENERATOR_IN_PLACE`. The first frozen local same-object,
order-sensitive candidate-placement preflight then failed before generation or
replay: real placement selected 0/2 intended candidates while its same-artifact cyclic
placement control selected 2/2. #953 now awaits a newly frozen maintainer plan,
not immediate Poincare/Hopf/harmonic machinery; #973 and #954 remain blocked.
See the [append-only #953 record](docs/local_geometric_generation_953.md).
Stored H4/Hopf/zeta/icosian and related route fields remain
structural state, diagnostics, or controls unless the owning stage qualifies a
specific term.

These commands exercise the no-model research substrate. `demo` does not start
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
- a qualified natural grammatical generation loop;
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
[#953](https://github.com/UOR-Foundation/uor-r4/issues/953). #973 and #954
remain blocked downstream of #953; #954 also depends on #973.

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
