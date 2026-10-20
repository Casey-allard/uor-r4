# Learned source-span emission — #1139 / #1140

## Decision — 2026-09-07

**Retain the active parent `c6a98c04`; reject promotion of experimental
`7e928bd2`.** The [evidence](evidence/native_geometric_source_span_1139.json)
binds the artifact, exact reports, controls, decision and resource receipts.

| Complete output and EOS | Parent or disabled extension | Candidate |
| --- | ---: | ---: |
| New construction | 2/5 | 5/5 |
| Exposed open development | 1/6 | 6/6 |
| Separately executed fresh diagnostic | 3/10 | 9/10 |
| Retained construction | 663/663 | 647/663 |
| Earlier exposed role-read cases | 24/24 | 20/24 |
| Explicit same-space semantic boundary | NOT_RUN | 0/1 |

The twenty preservation losses all overextend a location into `holds OWNER`.
Other preservation reports retain identical text/stop: numeric, literal,
identifier, dependent, relation, earlier source-context (32/32), literal-binding
(16/16), admission (12/12), source/NoRead (20/20) and five session turns including
isolation. These successes do not cancel the twenty regressions.

The fitted model uses three separator codes and six training decisions from five
raw documents. Its existing H4 learner improves 5/6 initial decisions to 6/6,
with two accepted parameter updates, no timed stop and no skipped documents.
The complete fresh diagnostic is 9/10: six multiword values plus two single-word
values and NoRead pass. The two-space value returns `Vale` rather than
`Vale  Crest`. The explicit phrase-boundary case returns
`Blue Harbour near Briar` rather than `Blue Harbour`.

Candidate rejection froze before executing the fresh diagnostic. Those cases
were separately authored and excluded from fitting; their familiar question
family and previously used owner names do not establish unrestricted transfer.
There was one fit and no post-fresh tuning.

## Observed bottleneck and implementation — 2026-09-07

The parent `c6a98c04`, delivered through PR #1166 at `8faf20f8`, selects the
correct first word in the six open examples but ends multiword answers early:
`New.`, `Rio.` and `red.` instead of `New York.`, `Rio de Janeiro.` and
`red flowers.`. The baseline is one complete answer out of six, including EOS.
This localizes these cases to source extent and emission, after source selection.

The new optional `source_span` artifact component reuses the existing two-lane
signed-H4 code/landmark learner for Continue versus Finish. Its current feature
is the exact separator byte. It learns offline from raw prompt/response pairs;
serving uses integer/table operations and no targets. Every parent parameter,
dictionary, first-source selector and lineage component remains unchanged.
Removing `source_span` reconstructs the complete parent. This is an optional
learned operator, not a replacement for the existing source/NoRead selector.

An exact delimiter travels with each retained word. The copy operator uses the
source byte ordinals to establish a one-byte gap before admitting an adjacent
word. A learned extent is computed from the frozen query capture before entry;
only a matching observed entry commits it. One causal byte cursor then traverses
the selected word, separators and following words, preserving the original
source commitment. No output observation chooses a new source. Query support
stays at sixteen words. Extensions are capped at 28 bytes within the existing
32-step response ceiling; wider gaps and unseen separators stop. Persistent and
dependent sources, numeric execution and NoRead retain their original paths.

This follows the existing exact-source and committed-operator architecture:
prime identity and geometric selection choose an occurrence; exact transport
executes the selected value. It reuses ordered source state and H4 operators
without adding a parser, transformer, matrix product, expert population or paging
subsystem. The broader [source audit](integration/architecture-2026-09/README.md)
and prior negative geometric/readout experiments remain intact. This experiment
does not separately establish an angular-versus-equality advantage or a new
predictive role for Hopf, fibers, zeta zeros or the paired-H4 representation.

## Boundary that remains

A separator-only head cannot distinguish `Blue Harbour` from
`Blue Harbour near Briar` when each transition uses a space. The explicit
same-space post-value clause checks that limitation. Two spaces are also outside
the implemented one-byte-gap transport. General phrase boundaries, variable
whitespace, unrestricted prose, syntax and reasoning remain unqualified.

Checkpoint restoration recomputes the learned extent from the committed source
and validates the emitted prefix, including separators. Available token evidence
checks delimiter position as well as spelling, including repeated words in one
lexical token. Missing legacy metadata, evicted bytes and older source/response
intervals without a reconstructible absolute source offset remain declared,
unauthenticated state. This is not authentication of arbitrary ancient history.

## Reproduction

Use the pinned Rust toolchain and the public example dispatch:

```text
native_geometric_value_probe source-span fit PARENT ROOT NEW_DIR
native_geometric_value_probe source-span fresh MODEL EXISTING_DIR
```

`ROOT` is the retained `native-typed-value-2026-09-05` artifact directory. The
fit command writes its five raw construction pairs, evaluates the exposed open
set, fits once and replays prior preservation. The separate fresh command first
executes fixed separately authored span cases after design selection. Compact
reports keep input, target, actual output bytes, stop, verdict and aggregate work;
per-token value/completion traces are omitted from direct text/stop generation
nodes to bound storage. Inherited typed/relation diagnostic fields remain. The actual-model
integration check uses `R4_SOURCE_SPAN_MODEL` and `R4_SOURCE_SPAN_PARENT`.

## Verification and resources

Six focused codec/extent/snapshot tests, the integer-kernel source guard and the
native architecture-policy check pass. The original snapshot test filter matched
zero tests; the corrected module filter executed all three. The actual candidate
loads and restores its exact parent parameters, rejects malformed H4 code and
forged extent, preserves source commitment through interruption, replays every
output-byte checkpoint, and executes ingestion/prediction/observation with zero
measured steady-state allocations. Fitting and session allocation are excluded.

On this Apple M1 with 16 GiB RAM, 28 uncached warm continuation predict/observe
steps measured median 292 ns and maximum 145,667 ns (0.146 ms). These are mostly
exact copying steps. Initial source-entry prediction, loading, encoding,
ingestion and checkpoints are outside that timing. The first timing receipt
populated the prediction cache during replay comparison; it is preserved but
excluded from uncached latency evidence. Neither measurement establishes
end-to-end language latency, frontier capability or energy savings.

The pre-execution projection capped model work at 90 seconds, engineering at
900 seconds, growth at 96 MiB, one model process, one build job and model-child
RSS at 4 GiB. Actual model work, including baseline, fitting, evaluation and both
artifact checks, is **83.477 seconds**. Cumulative use is **4156.246/4170 seconds**,
leaving **13.754 seconds**. Peak sampled model-child RSS was 954,548,224 bytes;
periodic sampling is not an exact transient-peak guarantee. The candidate is
11,627,817 bytes. Sampled growth at verification is 66,879,488 bytes, within
96 MiB. The initial compiler error and corrected timing check are charged; no
external compute, budget extension, artifact removal or research deletion occurred.
Engineering commands and final storage/delivery receipts remain in the retained
artifact directory. CI compatibility acknowledgements are not model tests.

## Next concrete direction

Keep the successful transport and causal commitment implementation as an
experimental component. Learn Continue/Finish from candidate-owned ordered source
context and query/value binding, using fresh construction/open counterfactuals
that require both actions at the same separator. Keep the first-source selector
and numeric/dependent execution fixed. Reuse the existing exact prime identities
and H4 feature/landmark learner; do not invent a semantic metric from hash bits
or substitute a hardcoded grammar rule for the missing learned boundary.

The remaining cumulative time does not cover another full fit and validation
under this cycle. No second fit was launched. Establish a complete successor
resource projection before execution; do not silently enlarge the allocation or
tune against the just-opened fresh diagnostic. The active parent, experimental
artifact, all twenty regressions and earlier historical results must survive.
