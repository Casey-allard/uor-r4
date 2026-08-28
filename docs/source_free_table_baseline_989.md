# #989 source-free table-native lexical baseline

- **Date:** 2026-08-28
- **Issue:** #989
- **Status:** established empirical lexical baseline
- **Terminal:** `ESTABLISH_TABLE_NATIVE_LEXICAL_BASELINE`
- **Claim scope:** construction-trained statistical lexical prediction and
  bounded exact decoding only

## Decision

The frozen B0 decision is positive. A deterministic source-free integer table
engine compiled from the construction partition, predicted held-out known
lexical targets at 22.261404% top-1 versus 5.413561% for the construction-only
unigram baseline, decoded a 16-unit continuation, and reproduced byte for byte
in two complete executions. The uplift is +16.847843 percentage points, above
the predeclared +5-point floor.

This establishes the table engine as the frozen non-geometric lexical reference
for exactly one later #953 geometric intervention under the same corpus,
support, decode, and work budget. It does not establish semantics, attention,
geometry, correctness, reasoning, chat, product readiness, performance
superiority, formal closure, or release readiness.

## Frozen input and artifact identities

| Item | Identity or value |
| --- | --- |
| Corpus CID | `blake3:194db0eebf2d49823ece01ee935447a0cc9edeaf018454ceea480ce7590132cf` |
| Manifest CID | `blake3:bb5f446ce92df60f7824ed5a1f04ede385386e7a47b9c198ae83a5d0f907bab3` |
| Unique documents | 3,000 |
| D3 construction documents | 2,404 |
| D3 held-out documents | 596 |
| Lexical routes | 116,061 |
| Packed table artifact bytes | 35,655,288 |
| Packed table artifact CID | `blake3:ccdc399731cb866a329be478467a434cda4e445813421e5d17c21ccc87288297` |
| Report payload CID | `blake3:6427efadc889f795d80bceeb508ac5f88f6d4934ac1cb4904706f1a92acf5fc3` |
| Artifact SHA-256 | `129d7061c652cc57616863f7a3a456cd00f1ec841780b672964438fa82767d7d` |

The corpus contained 3,000 unique document IDs. The D3 partition produced
2,404 construction documents and 596 held-out documents. Held-out documents
did not alter the lexical vocabulary, transition counts, candidates, or
selection.

## Held-out result

| Metric | Result |
| --- | ---: |
| Encoded held-out positions | 617,710 |
| Known-target positions | 446,342 |
| Table top-1 correct | 99,362 / 446,342 = 22.261404% |
| Unigram top-1 correct | 24,163 / 446,342 = 5.413561% |
| Absolute uplift | +16.847843 percentage points |
| Trigram selections | 319,336 |
| Bigram selections | 108,738 |
| Unigram selections | 18,268 |

The trigram, bigram, and unigram selection counts sum exactly to the 446,342
known-target positions. This is a statistical lexical result. It is not a
semantic score and does not turn context-table usage into attention.

## Bounded decoded continuation

The fixed prompt was `The United States` with a continuation cap of 16 lexical
units. The engine emitted all 16 units and stopped at the cap:

```text
. It is the most important thing to do so.
 1985 – The first people
```

The continuation contains a newline between `so.` and ` 1985`. The decoded
bytes were valid UTF-8 and contained no period-1 or period-2 cycle.
Readable text is evidence that the full corpus-to-table-to-payload path is
working; it is not evidence of factual correctness, coherence beyond this
bounded sample, semantics, or reasoning.

## Deterministic replay

Two complete corpus-to-report executions emitted identical report bytes. Their
packed artifacts were identical under byte comparison and shared the artifact
CID and SHA-256 above. The report payload CID was also identical. This closes
the external replay requirement for the frozen B0 run.

## Cheap natural fixture

The focused test completed 3/3 checks. Its held-out transcript recorded 2/10
correct table choices versus 0/10 for unigram and decoded:

```text
 in Bombay, India.
```

That fixture shows a construction-transferred lexical context effect and exact
decoding at small scale. It does not establish semantic understanding.

## Activated checks

- Focused `source_free_table_baseline_989` test: 3/3.
- Focused compile for `uor-r4-core` and the `r4` CLI: clean.
- Formatting for the named touched Rust paths: clean.
- Frozen real-corpus command: completed twice with identical reports and
  artifacts.

No broader verification was activated. Workspace tests, BDD, teacher paths,
graph and Gate C work, kappa reproduction, audit, fuzz, WASM, conformance,
formal verification, performance qualification, product chat, and release
checks are `NOT_RUN`, not PASS.

## Forward boundary

The table artifact and its corpus/support/decode/work contract are now frozen.
The only permitted next local comparison is exactly one #953 geometric
intervention against this unchanged reference. H4, SpiralCore, harmonic,
algebraic, placement, transport, higher-scope, and scale expansions remain
dormant unless that one issue explicitly activates the needed mechanism. #973
remains blocked behind #953.
