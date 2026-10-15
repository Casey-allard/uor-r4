# #977 — I1: add bounded provider-free generation plumbing

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-27T21:13:26Z
- **Updated:** 2026-10-09T14:17:20Z
- **Closed:** 2026-08-27T21:14:33Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/977
- **Merged:** 2026-08-27T21:14:33Z
- **Merge commit:** 9473f6b1a58bda1e756e5c361346c5c06e544e35
- **Base / Head:** main / UOR-Foundation:issue-953-bounded-geometric-generation
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

- correct the public and living #953 contract to the actual #969 local-path boundary;
- add reusable `LocalGeometricGenerator` plumbing for canonical bytes -> routes -> schema-2 admission -> #969 select/abstain -> exact inversion -> boundary rendering -> route append -> bounded stop;
- expose the loop through the visible research-only `r4 bounded-geometric-generate` command;
- record the frozen two-arm witness and its truthful terminal, `REVISE_I1_GENERATOR_IN_PLACE`.

The GitHub #953 title/body and stale inherited wording in #973/#954/#955/#962/#964 were corrected before the generator ran. Native blocker edges were not changed.

## Reused and added seams

Reused unchanged:

- canonical artifact decode/transitive validation and reconstructible codec identities;
- embedded schema-2 manifest and natural candidate admission;
- #969 causal path-state construction, select-or-abstain, and append;
- exact H4 table and exact route-to-payload inversion.

Added only:

- fail-closed artifact-bound prompt encoding;
- deterministic word/punctuation boundary rendering;
- bounded append/abstention/punctuation/short-cycle/cap control;
- a deterministic report with candidate, cost, inversion, append, work, stop, and source-boundary witnesses;
- a CLI adapter whose plain mode always renders the typed terminal, including empty abstention.

The current constructor intentionally fails closed unless the artifact's embedded construction/global input reconstructs the complete parent codec registry. Trailing prompt whitespace also fails closed rather than being normalized silently.

## Frozen revision witness

Identities:

- fixture: `blake3:b6e1f5a0665fb4e8a329c1b769697ced0b0f7b16ff20ff5a9288b279b1880409`
- codec: `blake3:71aa5e35465be4da1847bbfdbb7a836a4a21194f289fd638b79b4bfe576c8c09`
- construction artifact: `blake3:411f091f9455dd401711861db6db534482780f4b07645454c6bc1579072cc0ad`
- attention manifest: `blake3:55465770d59b8e27cc232e09511c59654b4c93acd074ee3f26652e4a03eb76d2`
- H4 root/multiplication tables: `blake3:8d33d62a239fb8001fea2bd14a9a5ec7321d0f07d81c74a5715eaeb3df53aa76` / `blake3:90ee73a27ee2e8ba5bccd1507d7fb37ed1f044b1640772c86752bc0bb2111759`
- canonical four-arm revision record: `blake3:f8738ae16585b5817108ad6c8bc1ec7aee93f9d5a6cacffaa3aa084bb643cf72`

Observed continuations:

| Prompt | Full path | State disabled |
|---|---|---|
| `active agile athletes run` | `slowly carefully` | `slowly slowly` |
| `agile active athletes run` | `carefully slowly` | `slowly slowly` |

All arms admit exactly `{carefully, slowly}` at both steps. Each step reads 7 rows and 2 entries; work is 4 then 5 keys/candidate and 8 then 10 H4 comparisons. All I1/I2/ordered-sentence rows miss. All 16/16 candidate occurrences and 8/8 selected occurrences invert exactly; every selected route appends; every arm stops at the two-unit cap with 6 total observed units; no period-1 through period-4 three-repeat cycle occurs. Two final executions reproduced the complete report bytes and revision kappa.

The report discloses artifact-input reconstruction and schema-2 witness compilation separately from zero selection-time source-weight, teacher, provider, source-attention, learned-router, dense-matrix, future-event, and higher-scope reads.

## Terminal and next action

The positive terminal is not supported. Independent audit proved that the ten lexical surfaces, construction topology, prompts, outputs, and exact costs are a rank-preserving relabel of the already observed #969 witness. With every contextual row missing, no linguistic evidence connects either admitted adverb to either prompt; neither prompt requires one over the other. This cannot qualify bounded grammar or expose #973.

Terminal: `REVISE_I1_GENERATOR_IN_PLACE`.

#953 remains open for one independently motivated grammatical contrast whose natural surfaces, incompatible choices, support/work budget, and artifact identity are frozen before H4 selection. #973 and #954 remain blocked. This PR does not implement higher scopes, correctness, reasoning, chat, optimization, formal closure, or release QA.

## Focused verification

- focused #953 freeze + revision witness, executed twice;
- directly implicated #969 decoded-path regression;
- focused core and root CLI compilation;
- CLI parser, help, and typed-terminal rendering tests;
- `cargo fmt --check`;
- `git diff --check`;
- changed-document claim-wording gate;
- WASM root library check (pre-existing target-specific warnings only).

Workspace-wide tests, broad corpus/model/teacher runs, generation canaries, benchmarks, BDD, doc tests, clippy, no-std ladders, deterministic rebuilds, kappa reproduction, Gate C, audit, fuzz, formal proof, conformance, product QA, and release qualification are `NOT_RUN`. Protected queue contexts are transport acknowledgements, not QA evidence.

References #953


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

