# R⁴ documentation

This is the map for understanding the repository without having to reconstruct
its history first.

R⁴ is currently pursuing one programme: a local, source-free language agent
whose attention, inference, and reasoning are performed by geometric routing
and lookup rather than a transformer, MoE, sparse learned router, or dense
learned matrix engine in the serving path.

The goal is real. Its success remains unproven. The current implementation is a
storage/recall and route-query foundation, not a working geometric language
model.

## Start here

Choose the shortest path that matches what you need:

- **Understand the project:** read the
  [Geometric Intelligence Programme](geometric_intelligence_programme.md).
- **Understand the current geometric mechanism:** read
  [ADR-0004](adr/0004-geometric-intelligence-route-hierarchy.md), then use the
  [glossary](transformerless/GLOSSARY.md) for unfamiliar terms.
- **Contribute to the active build:** start from live issue
  [#820](https://github.com/UOR-Foundation/uor-r4/issues/820) and take only the
  first unblocked stage.
- **Audit a result or claim:** use the [research ledger](RESEARCH.md), then open
  the exact issue-numbered evidence record it names.
- **Run the existing interface:** return to the root
  [README](../README.md#try-the-project).

If an older roadmap disagrees with the current programme or live GitHub
dependency graph, the current programme and live dependency graph win.

## Current authority

These are the small set of living documents that define the present work:

1. [Geometric Intelligence Programme](geometric_intelligence_programme.md) —
   goal, architecture, claim boundaries, and work order.
2. [ADR-0003: fixed-zeta prime routes](adr/0003-fixed-zeta-prime-route-attention.md)
   — the retained storage/recall substrate.
3. [ADR-0004: recursive route hierarchy](adr/0004-geometric-intelligence-route-hierarchy.md)
   — attention scopes, geometric transport, and reconstruction requirements.
4. [Corpus-Induced Harmonic Signed Transport Plan](corpus_induced_signed_transport_attention_plan_986.md)
   — the current post-#983 semantic-placement, self-plus-six link-state,
   transport, comparator, and decision contract.
5. [Geometric Intelligence Evaluation](geometric_intelligence_evaluation.md) —
   the minimal decision-bearing evidence policy.
6. [Glossary](transformerless/GLOSSARY.md) and
   [formal vocabulary](formal_vocabulary.md) — shared language and disciplined
   claim types.

Living implementation documentation may explain a current component, but it
does not independently promote a capability. Storage is not attention;
attention is not inference; readable text is not correctness; correctness is
not reasoning.

## Programme at a glance

```text
reversible lexical geometry
  → held-out corpus-induced semantic placement and harmonic link state
  → candidate-relative signed geometric transport
  → coherent source-free generation
  → correctness and abstention
  → multi-step reasoning
  → chat / CLI / WASM product integration
  → measured optimization
  → release QA
```

Only the first unblocked stage is active. Formalization, optimization, and
large test programmes are supporting tools, not substitutes for reaching the
next observable behavior.

## Research record and archive

The repository contains years of useful positive, negative, and incomplete
work. It is preserved because rigor and failed hypotheses matter, but it is
not required reading for a newcomer.

Use [RESEARCH.md](RESEARCH.md) as the archive index. It leads to:

- issue-numbered measurement records;
- the earlier TLA and R4G1 table/graph compiler and integer runtime;
- proof, conformance, certification, and performance work;
- the original prime-router and geometric-context evidence;
- superseded decoder and intelligence roadmaps; and
- teacher-derived comparison and reproduction procedures.

Those documents report what a particular artifact established at a particular
time. They do not silently become evidence for the current route-native engine.
Historical commands may still run, but they are research reproductions unless
the current programme explicitly adopts them.

## How to read claims

- **Implemented** means the code or artifact exists.
- **Observed** means a named bounded run produced the recorded result.
- **Qualified** means the declared falsifier and threshold were met.
- **Not run** and **unavailable** are never treated as success.
- **Goal** or **hypothesis** is not a present capability.

For exact definitions, use the [formal vocabulary](formal_vocabulary.md).

## Documentation maintenance

Keep the root README approachable. Put architecture here under the current
programme, and put exact measurements in the research ledger and their named
records. Preserve historical evidence; add a clear superseded or historical
banner when old present-tense language could confuse readers.

Prefer linking to one current authority over copying the same mechanism into
many pages. If a measurement changes a claim, update the living summary and
append the new evidence to the ledger.
