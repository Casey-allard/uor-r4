# #953 bounded source-free geometric generation loop

- **Issue:** [#953](https://github.com/UOR-Foundation/uor-r4/issues/953)
- **Date:** 2026-08-27
- **Verdict:** `REVISE_I1_GENERATOR_IN_PLACE`
- **Scope:** provider-free decoded-loop plumbing plus one rejected lexical-relabel smoke

## Implemented path

`LocalGeometricGenerator` executes the first reusable decoded loop over the
accepted #969 selector:

```text
prompt bytes -> reconstructed canonical lexical codec -> registered routes
             -> unchanged schema-2 natural admission
             -> full causal-path select-or-abstain
             -> exact route-to-payload inversion
             -> deterministic lexical-boundary rendering
             -> causal route append
             -> punctuation, abstention, short-cycle safety, or cap stop
```

The byte-based constructor strictly decodes and transitively validates the
canonical route artifact, reconstructs its declared input, recompiles the
lexical codec, requires both codec and vocabulary kappas to reproduce, decodes
the embedded schema-2 manifest, compiles the unchanged bounded attention rows,
and binds the exact H4 table. The generator exposes only `FullPath` and
`StateDisabled`; #969's `LastOnly` diagnostic is not promoted into I1.
This reconstruction accepts only artifacts whose embedded construction/global
input reproduces the complete parent codec registry; a valid artifact with
unobserved registered units fails closed until an artifact-native codec
reconstruction seam exists.

The new seams are limited to artifact-bound prompt encoding, exact inversion,
word/punctuation boundary rendering, bounded append/termination, and a
deterministic report projection. Trailing prompt whitespace is rejected fail
closed before selection because accepting it without normalization would make
word and closing-punctuation boundaries ambiguous. The CLI command
`r4 bounded-geometric-generate` loads one canonical artifact, preserves every
accepted prompt byte, selects one of the two declared controls, and emits
labeled continuation plus typed stop reason or the canonical JSON report. It
is a research surface, not the product chat path.

## Frozen contract and identities

The public issue contract and terminal literals were corrected before the new
generator ran. The lexical-surface corpus, prompts, two controls, two-unit cap,
three-repeat short-cycle definition, and construction artifact were then
frozen before the generator smoke was first executed.

| Object | Kappa |
|---|---|
| Canonical lexical codec | `blake3:71aa5e35465be4da1847bbfdbb7a836a4a21194f289fd638b79b4bfe576c8c09` |
| Natural construction artifact | `blake3:411f091f9455dd401711861db6db534482780f4b07645454c6bc1579072cc0ad` |
| Embedded attention manifest | `blake3:55465770d59b8e27cc232e09511c59654b4c93acd074ee3f26652e4a03eb76d2` |
| H4 root table | `blake3:8d33d62a239fb8001fea2bd14a9a5ec7321d0f07d81c74a5715eaeb3df53aa76` |
| H4 multiplication table | `blake3:90ee73a27ee2e8ba5bccd1507d7fb37ed1f044b1640772c86752bc0bb2111759` |
| Smoke fixture | `blake3:b6e1f5a0665fb4e8a329c1b769697ced0b0f7b16ff20ff5a9288b279b1880409` |
| Canonical four-arm revision record | `blake3:f8738ae16585b5817108ad6c8bc1ec7aee93f9d5a6cacffaa3aa084bb643cf72` |

Before delivery, independent review found two trace-label defects: the first
schema called artifact reconstruction a zero higher-scope read, and a selected
record called the total route count the number appended. The aggregate record
was re-pinned after separating constructor disclosures from selection-time
zero counters and naming the after-append route count exactly. Candidates,
costs, decoded outputs, and stops did not change. The terminal was then bound to
the revision verdict after the separate natural-language contract audit below.

The construction sentences are `train carefully`, `walk slowly`, and the
single-unit sentences `active`, `agile`, `alert`, `athletes`, and `run`; the
bounded global snapshot contains `brave`. These ten lexical surfaces are in
strict byte order and preserve #969's identity-derived route placement
position-for-position.

The matched prompts are:

```text
left:  active agile athletes run
right: agile active athletes run
```

Both are four-unit lexical strings with the same multiset, suffix, candidate
union, and comparison budget. Construction contains no four-unit sentence.
Last-one, last-two, and ordered-sentence rows miss at every generated step, so
neither requested full history nor its continuation is stored.

## Observed relabel smoke

The four complete continuations are:

| Prompt | Full causal path | State disabled | Stop |
|---|---|---|---|
| `active agile athletes run` | `slowly carefully` | `slowly slowly` | cap after 2 units |
| `agile active athletes run` | `carefully slowly` | `slowly slowly` | cap after 2 units |

The full path changes the first route for the reordered prompt while the
state-disabled arm is prompt-inert. The report retains the leading ASCII
boundary byte required to append each continuation to its word-final prompt.

At both steps and in all arms, the naturally admitted union is exactly
`{carefully, slowly}`. Each candidate has one adjacent-spin count and no direct
I1/I2/IS count. Work is fixed by the existing selector:

| Step | Observed routes | Rows | Entries | Candidates | Keys/candidate | H4 comparisons |
|---:|---:|---:|---:|---:|---:|---:|
| 1 | 4 | 7 | 2 | 2 | 4 | 8 |
| 2 | 5 | 7 | 2 | 2 | 5 | 10 |

The full-path exact costs are:

- left step 1: `slowly` 36 degrees/age 4; `carefully` 60 degrees/age 2;
- right step 1: `carefully` 36 degrees/age 4; `slowly` 60 degrees/age 2;
- left step 2: `carefully` 36 degrees/age 6; `slowly` orthogonal/age 3; and
- right step 2: `slowly` 36 degrees/age 5; `carefully` 36 degrees/age 6.

State-disabled repeats the same candidate-only costs on both prompts and both
steps: `slowly` 72 degrees/age 1 and `carefully` 120 degrees/age 1. It therefore
emits `slowly slowly` for both prompts under the same support and work shape.

All 16/16 admitted candidate occurrences and all 8/8 selected occurrences in
the four-arm record independently invert to their registered payload bytes.
Every selected route is appended through `observe_path`; every report ends with
six observed units, below the eight-unit bound. Termination is the frozen
two-unit cap. No report contains a period-1 through period-4 cycle under the
existing three-equal-trailing-period definition.

Each arm was generated twice and its complete report value, canonical JSON
bytes, and report kappa were identical. A separate second execution of the
focused two-test file reproduced the pinned aggregate record kappa above.

## Why the positive gate did not pass

Independent terminal review established that this fixture is an exact
rank-preserving lexical relabel of #969:

- #969's byte-sorted `aa bb cc dd gg ll qq rr uu vv` registry maps in order to
  `active agile alert athletes brave carefully run slowly train walk`;
- `uu ll | vv rr` maps exactly to `train carefully | walk slowly`;
- `aa bb dd qq | bb aa dd qq` maps exactly to the two prompts above; and
- `rr ll | ll rr` and the complete 36/60 then 36/orthogonal cost signatures
  reproduce as `slowly carefully | carefully slowly`.

Every contextual last-one, last-two, and ordered-sentence row misses. The two
adverbs are admitted only by the same adjacent-spin topology as #969, so no
linguistic evidence connects either candidate to either prompt. Both prompts
permit or reject `slowly` and `carefully` equally, and the uncoordinated
two-adverb strings do not establish bounded grammatical sentence formation.
The loop execution is real implementation evidence; the relabel cannot be used
to manufacture the frozen positive terminal.

## Source boundary

The only generation inputs are the validated canonical artifact, its embedded
schema-2 manifest, the fixed exact H4 table, and prompt bytes. The report
explicitly records that the constructor reconstructed the artifact input and
compiled its schema-2 rebuild witnesses. It separately records zero
selection-time source-weight reads, teacher forwards, provider calls,
source-attention calls, learned-router calls, dense-matrix operations,
future-event reads, and paragraph/conversation/global selection reads.
Canonical artifact validation also requires `source_weights_opened = false`
and `teacher_forwards = 0`.

The original Prime R4 report and archived router implementation were treated as
hypothesis material only. The old trigram idea did not add a new table because
schema 2 already supplies deterministic predecessor/successor admission. No
Ollama output, cosine/repetition selector, random fallback, dense vector,
post-selection Hopf trace, or future-return field was reused.

## Decision and nonclaims

The terminal is `REVISE_I1_GENERATOR_IN_PLACE`. The reusable provider-free
decode/render/append component and research CLI are genuine partial progress,
but #953 remains open because its smoke did not present incompatible natural
choices independently of #969's rank-derived geometry. #973 remains blocked.

The next #953 action is one independently motivated grammatical contrast whose
natural surfaces, candidate union, support/work budget, artifact identity, and
expected incompatible choices are frozen before any H4 selection is observed.
Subject-number agreement with `{run, runs}` is one candidate contract; it is
not executed or tuned in this record.

It does **not** establish lexical semantics, factual correctness, broad
coherence, knowledge, reasoning, higher-scope attention, performance advantage,
product chat quality, optimization, formal closure, or release readiness.

## Focused verification

Executed locally:

- focused #953 fixture freeze and relabel-smoke revision witness;
- deterministic in-test double generation for all four arms;
- directly implicated #969 decoded path regression;
- focused core and `r4` CLI compilation;
- CLI parser/help coverage;
- formatting, diff-whitespace, and changed-document claim wording; and
- the root WASM library check because the exposed core module is re-exported by
  the root library.

Workspace-wide tests, broad corpus/model/teacher runs, generation canaries,
benchmarks, BDD, doc tests, clippy, no-std ladders, deterministic rebuilds,
kappa reproduction, Gate C, audit, fuzz, formal proof, conformance, product QA,
and release qualification were `NOT_RUN`. Protected queue checks are transport
acknowledgements at this research stage and are not local or release-QA PASS
evidence.

## Natural agreement revision — pre-selection freeze

This append-only revision freezes one independently motivated
agreement-attractor contrast before any H4 path cost or selection is opened.
At this checkpoint the selection-blind support preflight and all four selector
arms are `NOT_RUN`; the identities below were produced only by canonical codec,
artifact, and schema-2 manifest compilation.

One exact `ConversationInput` is shared by codec compilation and artifact
ingestion. Its identity scope is `issue-953/natural-agreement-v1`, its sole turn
is `turn-0001`, its global snapshot is the registration-only unit `near`, and
its ordered construction sentences are:

```text
athletes generally still run
one athlete generally still runs
```

The exact matched prompts, frozen expected continuations, and cap are:

```text
left:      one athlete near athletes generally
expected:  still runs

right:     athletes near one athlete generally
expected:  still run

continuation cap: 2
controls: full_path | state_disabled
failure terminal: REVISE_I1_GENERATOR_IN_PLACE
```

The two prompts contain the same five-unit multiset. Both must first admit and
emit only `still`; after that append, the decisive query has the identical
suffix `generally still` and candidate union `{run, runs}`. No lexical surface,
registration set, observation order, scope ID, global snapshot, prompt,
expected output, cap, rank, prime, spin sector, H4 coordinate, label, or support
budget may change after selection is observed.

The exact fixture contains eight distinct registered surfaces: seven
construction lexemes plus global-only `near`. Each completed arm contains seven
observed route occurrences: five prompt routes plus two emitted routes. This
preserves the complete requested construction while making the registry/trace
count distinction explicit.

| Frozen object | Kappa |
|---|---|
| Natural agreement fixture | `blake3:0e018c9bcd43a29ed6f043665b2646c9579dd31d881d331f198fb89543184259` |
| Canonical lexical codec | `blake3:6db64540ef344562903e01adac102f7bcc96c65908d162b1deca9b83550b35ed` |
| Canonical vocabulary | `blake3:3b74f7ace425c039b4eab751b400f2603d92baf4ccfc9f4b8ac9409446291b58` |
| Natural construction artifact | `blake3:b222510ccc01ed3257c8b38b743ca771f5e60c87ebf12c565f92fadbbd00332d` |
| Embedded/compiled attention manifest | `blake3:1c3baf432b9fdcf2f3d90014797a5cae5850c0acba2fda63e0d6b659d49562de` |

The frozen selection-blind preflight contract is:

| Step | Rows | Entries | Candidate union | Per-candidate source counts `(I1,I2,IS,D,AS)` | Keys/candidate | Declared H4 comparisons |
|---:|---:|---:|---|---|---:|---:|
| prompt to `still` | 7 | 3 | `{still}` | `still=(2,1,0,2,0)` | 5 | 5 |
| after frozen append `still` | 7 | 6 | `{run,runs}` | each `(1,1,0,1,0)` | 6 | 12 |

Both prompts must have the same candidate counts, source counts, and work shape;
no truncation or adjacent-spin contribution is allowed. The preflight uses a
support-only API but projects only row source/hit/count/admission data and never
inspects or logs row keys. The trace carries no measured candidate energy, H4
state, path cost, or selected candidate. The declared H4 comparisons are count
arithmetic only (`admitted candidates * observed routes`). Any mismatch
stops this revision at `REVISE_I1_GENERATOR_IN_PLACE` before path selection.

If and only if that preflight passes unchanged, the generator may run once in
the four frozen arms, followed only by a complete replay for byte identity. A
positive terminal requires both full-path expected continuations, prompt-inert
disabled decisive behavior, matched support/work, exact inversion and append,
cap termination, no short cycle, source/provider closure, and deterministic
report bytes. Even a positive result would establish only this bounded
source-free geometric agreement witness, not broad grammar, coherent language,
semantics, correctness, attention in general, or reasoning.

## Natural agreement revision — support hard stop

The frozen support-only preflight was executed after the checkpoint above. Its
support/work steps reproduced identically across the two matched prompts, but
they failed the declared shape before any H4 path selection:

- support preflight record:
  `blake3:70375921e267b5ceff2198f879356cfb42dd6907accc0c2b720fc8b89b59b271`;
- left/right support and work: identical;
- truncation: none (`5` candidates below the ceiling of `8`);
- H4 path costs, selector outputs, and all four generator arms: `NOT_RUN`; and
- terminal: `REVISE_I1_GENERATOR_IN_PLACE`.

The observed count-only support was:

| Step | Rows | Entries | Candidate union | Per-candidate source counts `(I1,I2,IS,D,AS)` | Adjacent-spin rows hit | Keys/candidate | Declared H4 comparisons |
|---:|---:|---:|---|---|---:|---:|---:|
| prompt | 7 | 8 | `{athlete,generally,run,runs,still}` | `athlete=(0,0,0,0,1)`; `generally=(0,0,0,0,2)`; `run=(0,0,0,0,1)`; `runs=(0,0,0,0,1)`; `still=(2,1,0,2,2)` | 1 | 5 | 25 |
| after frozen `still` append | 7 | 11 | `{athlete,generally,run,runs,still}` | `athlete=(0,0,0,0,1)`; `generally=(0,0,0,0,2)`; `run=(1,1,0,1,1)`; `runs=(1,1,0,1,1)`; `still=(0,0,0,0,2)` | 1 | 6 | 30 |

This is an admission/indexing failure, not a geometric-ranking result. One
adjacent-spin row admitted four extra lexical surfaces at the first step and
retained three extra surfaces at the decisive step. Although the direct and
divisor rows gave the frozen candidates their expected counts, the current
union merges those rows with broad adjacent-spin fallback support before the
candidate ceiling. Consequently the generator never reached its shared-prefix
bridge or decisive agreement choice, and no statement can be made about the
H4 path metric on this fixture.

The single recommended next hypothesis is the smallest contract-preserving
direct-row-first admission seam: make I1/I2/ordered-sentence rows plus divisor
the primary tier, preserving the frozen direct and divisor counts, and consult
adjacent-spin rows only when that primary tier is empty. That hypothesis must
preserve this exact frozen fixture, give the tier policy a visible trace/policy
identity, and rerun its support gate before any later H4 selection. It is
recorded for the next #953 revision only; implementing or tuning it after this
result was not authorized in this session. The original trigram/Markov
generator remains dormant and is not the proposed mechanism.

A future tiered trace must distinguish a row slot/key, whether it was
consulted, whether a physical row existed, and how many entries were admitted.
It cannot relabel the physically present adjacent-spin row as a miss or silently
reinterpret the old seven-row/entry fields. Any revised work contract and query
policy identity must be frozen before that repair is exercised.

### Natural agreement revision verification

Focused local checks exercised only the changed seam and directly implicated
regressions:

- the natural-agreement integration target passed its identity and negative
  support-record tests; its four-arm witness was ignored with
  `NOT_RUN_SUPPORT_PREFLIGHT_HARD_STOP`;
- the existing #953 relabel regression, the #969 two-unit causal-path
  regression, and all eight #958 attention regressions passed;
- focused core compilation, the root WASM library check, formatting,
  diff-whitespace, and claim wording passed; and
- a focused strict-clippy attempt reached existing main-line lint debt outside
  the changed paths. Repeating the same target while allowing only those five
  pre-existing lint categories passed; no unrelated lint cleanup was included.

Workspace-wide tests, corpus/model/teacher work, generation canaries, BDD,
Gate C, kappa reproduction, audit, fuzz, formal proof, conformance, product QA,
and release qualification were `NOT_RUN`. Repository-required checks remain
transport acknowledgements while product QA is dormant; they are not test or
release evidence.

## Programme direction addendum — 2026-08-27

This addendum records the accepted next design boundary; it adds no measurement
and does not change the hard-stop evidence above.

#953 will separate candidate admission from harmonic influence. Its primary
tier is I1/I2/ordered-sentence plus divisor. When that tier is non-empty, the
physically present adjacent-spin row remains visible as consulted/present in the
trace but contributes zero admitted entries. When the primary tier is empty,
the same bounded row may activate under an explicit fallback policy. The frozen
support preflight reruns before any H4 selection or generator arm.

The adjacent-spin rows are retained rather than discarded, but only as traced
retrieval fallback and diagnostic data. They map a coarse current sector to
historically observed next candidates; they are not operator coefficients or
exact-class relations. #973 must build any neighbor operator independently over
exact classes and routes already admitted by #953.

That later operator prototype may reuse the existing full
signed-S3/Hopf/fiber/torsion `shared_class_kappa`, not a Hopf octant. A
direction-sensitive relation requires either a new exact `SpinTorsionState`
relative relation or an explicitly bound spin-to-H4 map; the existing relative
H4 witness is prime-derived route state. Similar non-identical states require a
separately frozen finite relative-angular kernel. Neither operator path may
inject candidates, rewrite immutable addresses, or broadcast across the
corpus.

This direction is intended to address the observed support contamination and
to create a causal location for a higher-scope exact-spin operator prototype.
Calling that operator harmonic additionally requires a bound basis, mode order,
coefficients, quantization, and transition law. It does not
establish grammar, coherence, semantic spin placement, correctness, or
reasoning. #953 remains open at `REVISE_I1_GENERATOR_IN_PLACE`, and #973 remains
blocked.

## Tiered-admission repair and frozen selector outcome — 2026-08-27

This append-only revision implements the previously frozen admission seam as
the versioned query policy
`uor-r4.attention-query-policy/primary-then-adjacent-spin-fallback-v1`
(`PrimaryThenAdjacentSpinFallbackV1`). Its policy kappa is
`blake3:18c514b74b7d3e0e8796d9834c74d84745f0eddc88be0ef87236474f97a83820`.
Within each active tier, selection remains
`SourceBreadthThenTotalCountThenCanonicalAddress`; the admission policy and the
within-tier ordering policy are separate identities.

Slots 0–3 are the primary I1/last-one, I2/last-two, ordered-sentence, and
divisor rows. Slots 4–6 are the adjacent-spin center, previous, and next rows.
Every slot is consulted and reported in deterministic order. A non-empty
primary tier leaves adjacent rows physically visible and reports their
available entries, but marks fallback inactive and examines and admits zero of
those entries. Only an empty primary tier activates adjacent-spin fallback.
The schema-2 generation report and each row trace bind the query-policy
identity and kappa, slot, source, key, consulted flag, physical-row presence,
fallback state, and available/examined/admitted entry counts. The existing
seven-row and candidate ceilings remain unchanged.

The repair preserved all five frozen natural-agreement identities:

| Frozen object | Kappa |
|---|---|
| Natural agreement fixture | `blake3:0e018c9bcd43a29ed6f043665b2646c9579dd31d881d331f198fb89543184259` |
| Canonical lexical codec | `blake3:6db64540ef344562903e01adac102f7bcc96c65908d162b1deca9b83550b35ed` |
| Canonical vocabulary | `blake3:3b74f7ace425c039b4eab751b400f2603d92baf4ccfc9f4b8ac9409446291b58` |
| Natural construction artifact | `blake3:b222510ccc01ed3257c8b38b743ca771f5e60c87ebf12c565f92fadbbd00332d` |
| Embedded/compiled attention manifest | `blake3:1c3baf432b9fdcf2f3d90014797a5cae5850c0acba2fda63e0d6b659d49562de` |

The prior flat-union hard-stop record
`blake3:70375921e267b5ceff2198f879356cfb42dd6907accc0c2b720fc8b89b59b271`
remains historical evidence. The repaired selection-blind support record is
`blake3:aab38fc513521cdd495bad74cc4a87754ec43ecdef5cb6e098b101412d3d7fe9`.

| Step | Rows | Available | Examined/admitted | Candidate union | Per-candidate source counts `(I1,I2,IS,D,AS)` | Keys/candidate | Declared H4 comparisons |
|---:|---:|---:|---:|---|---|---:|---:|
| prompt to `still` | 7 | 8 | 3 / 3 | `{still}` | `still=(2,1,0,2,0)` | 5 | 5 |
| after frozen append `still` | 7 | 11 | 6 / 6 | `{run,runs}` | each `(1,1,0,1,0)` | 6 | 12 |

At both steps one adjacent-spin row was physically present with five available
entries. All three adjacent slots were consulted, fallback was false, and their
examined/admitted counts were zero. The two prompt orders had identical
support and work after excluding the necessarily prompt-specific canonical row
keys. Thus the repaired preflight passed exactly before H4 selection was
opened.

The directly implicated #969 empty-primary regression also passed. Its three
adjacent rows activated under fallback, examined and admitted exactly two
entries, preserved its causal decoded choices and counts, and retained record
kappa
`blake3:60360a9e22a56ea4af363e43f7103bb8104d015d58feb582d921fc17afaf207f`.

The natural-agreement selector was then executed exactly once in the four
frozen arms, followed only by the declared complete in-test replay:

| Prompt/control | Decoded continuation |
|---|---|
| left / full path | `still run` |
| right / full path | `still run` |
| left / state disabled | `still runs` |
| right / state disabled | `still runs` |

Every arm emitted the shared first unit `still`. The right full-path arm matched
its frozen continuation, but the left full-path arm did not; both full-path
decisive choices were the same rather than incompatible. The disabled arms
were prompt-inert. Support/work equality, exact address-to-payload inversion,
append, two-unit cap termination, no period-1 through period-4 short cycle,
source/provider closure, and byte-identical replay all passed. The frozen
four-arm record kappa is
`blake3:dfe03d4c56f7e5e9cf48d524f2f0b10482c4b3b85fae152dd29c64543caa0b79`.
The separate schema-2 relabel regression is bound by
`blake3:b0248e715d4eab726588f47bef5dc4bb330580096b9d2e3bd3de9162d267081c`;
its earlier schema record
`blake3:f8738ae16585b5817108ad6c8bc1ec7aee93f9d5a6cacffaa3aa084bb643cf72`
remains historical.

The terminal remains `REVISE_I1_GENERATOR_IN_PLACE`. Admission is no longer the
defect, and the prompt-inert disabled control rules out candidate starvation as
the explanation for the observed full-path collapse. The localized defect is
that the current candidate-relative representation/scorer does not distinguish
the two required same-object, order-sensitive agreement outcomes. No second
tuned natural experiment was run.

The next in-place #953 hypothesis is a bounded construction-only,
corpus-induced, same-object and order-sensitive candidate-relative
compatibility or placement overlay. It must freeze its representation,
placement, partitions, quantization, provenance, policy identity, and kappa
before evaluation; preserve the repaired candidate union and work; and compare
real, disabled, same-artifact placement-permuted, and order-shuffled controls on
held-out anti-recall histories. Historical same-object tests support this only
as a hypothesis: exact self-match is a sanity anchor, while corpus-derived
placement has shown selective proxy alignment without a decoded causal choice.
Generic fixed Hopf routing, broad sector occupancy, and Poincare low-mode
alignment do not establish the missing decision and are not the immediate
repair. The original Markov/trigram generator, larger model/corpus runs, and
higher-scope operator work remain dormant.

The immediate operand mismatch is concrete. The current selector composes the
live ordered fold with a candidate leaf assigned from the candidate prime modulo
the fixed 120-root table; that identity-derived leaf is not a construction-
induced representation of histories in which the same candidate occurred.
The proposed local overlay instead encodes construction-only predecessor
histories with the same versioned ordered encoder used at query time and binds
each already-admitted candidate to one exact prototype or bounded prototype
set. Equal exact costs abstain. A compiler/query same-frame pair must reproduce
the same exact H4 state or report `UNAVAILABLE_FRAME_MISMATCH` before any
quality verdict.

The historical #486 same-object audit explains the guard but does not supply a
decoder score. Routing-query self-match was chance, whereas placing both sides
in the same content frame yielded identity self-similarity/top-1 of 1.0 and a
shipped-probe top-1 of 0.8. On the bounded retrieval probe, content-query
placement improved top-1/MRR/recall@20 from
`0.6240/0.7179/0.9720` to `0.7840/0.8542/0.9900`; the exact identity result is
tautological and retrieval is not generation. #490 retained routing state and
changed only the comparison operand. #502's later `W=0` simplification added
only 0.032 top-1 and 0.022 MRR at unchanged recall, below its historical 0.05
bar. The frame-consistency audit also records a false zero caused by comparing
rotated frames; same-frame operands restored cosine 1.0 and 1.327 separation.
Those values are anti-vacuity/representation evidence, not promotion
thresholds.

Archived geometric evidence reinforces the boundary. Generic fixed Hopf
routing was indistinguishable from placement-permuted or randomly permuted
controls, including validation perplexities 164.54 versus 164.41 in a
transformer experiment. Corpus-derived PPMI placement and selected four-
dimensional subspaces later produced selective routing and low-frequency proxy
alignment, including roughly 40–48% task-margin low-frequency improvement
across four seeds, but no decoded causal choice; one preregistered shell gate
was degenerate rather than passed. These records justify testing a frozen local
candidate-context placement, not inserting Hopf/Poincare machinery or claiming
semantic attention.

#953 therefore remains open and assigned. The positive-only handoff to #973 is
not activated; #973 and #954 remain blocked. This result establishes a
versioned bounded admission policy and one negative decoded-loop witness only.
It does not establish grammar, coherent generation, semantic placement,
correctness, higher-scope attention, reasoning, performance advantage, formal
closure, product readiness, or release readiness.

### Focused verification for this revision

The decision-bearing checks executed locally were:

- the frozen natural-agreement identity, historical hard-stop, and repaired
  support-preflight tests: 3 passed, with the explicit four-arm evidence test
  ignored during routine replay;
- the explicit natural four-arm witness: one permitted run passed and performed
  its complete byte-identical replay internally;
- the existing #953 relabel regression: passed with its schema-2 record pinned;
- the #969 empty-primary adjacent-fallback and causal-path regression: 1 passed,
  retaining its prior record;
- the #958 policy identity, truthful row trace, bounds, and attention
  regressions: 8 passed;
- focused `uor-r4-core` all-target compilation: passed;
- the root WASM library check required by the exported core change: passed with
  existing warnings outside this revision;
- formatting, changed-document claim wording, and diff-whitespace checks:
  passed.

Workspace-wide tests, strict workspace clippy, no-std ladders, corpus/model/
teacher work, generation canaries, BDD, Gate C, kappa reproduction, cargo audit,
fuzz, formal proof, conformance, product QA, performance measurement, and
release qualification were `NOT_RUN`. The protected merge queue remains the
binding integration transport; its checks are not local product or research
qualification evidence.

## `LocalSameObjectContextPlacementV1` preflight hard stop — 2026-08-27

This append-only revision executed the next frozen #953 mechanism contract. It
did not modify `PrimaryThenAdjacentSpinFallbackV1`, the five frozen natural
fixture identities, the repaired support record, the historical four-arm
record, the #969 selector, or the schema-2 decoded generator. It added only a
construction-derived overlay, a label-free preflight input, and a raw relation
census that was both label-free and selection-blind.

The frozen identities are:

| Object | Identity / kappa |
|---|---|
| Placement policy | `uor-r4.local-same-object-context-placement/1` |
| Placement-policy kappa | `blake3:e09af2db10f41efaf02b24e075a97fe42dc966834c43566689579763ba95b49c` |
| Ordered encoder | `uor-r4.recent-suffix-ordered-h4-trajectory/1` |
| Exact relation vector | `uor-r4.exact-recent-suffix-h4-shell-vector/1` |
| Overlay artifact | `blake3:2be03c8acc1e97d1ba830805653ec1b16745065127ee1e00cb431b933a173ee1` |
| Label-free preflight input | `blake3:ce5430048c3789e85e18ed17a80f10f79d317a66769ad8be0fd28224668eb72e` |
| Raw label-free, selection-blind relation census | `blake3:50e67c087e1ec5e04aa47cf09d42e9b0857c15e4cafa07b1363d67faf96c6aeb` |
| Label-attached preflight outcome | `blake3:d5e2f3614c8f1d3c6c629e2261ec42dc970fa5484982b2a86cb4f4b06b06a372` |

The overlay binds schema, the placement and repaired-admission policies,
manifest kappa and all four provenance CIDs, H4 root and multiplication-table
kappas, width and occupancy-aware identity padding, suffix/comparison order,
the four-prototype cap, exact candidate membership, causal witness provenance,
and prototype coordinates. Its compiler consumed three artifact-bound rebuild
witnesses containing seven within-witness predecessor-to-observed-next
transitions. Those yielded five candidate classes and seven retained
prototypes; the decisive `run` and `runs` candidates each had one prototype.
All seven compiler trajectories reproduced through the online query encoder,
all seven exact self-matches were coincident, and the complete typed
construction inventory had zero cross-candidate class collisions and zero
padding-identity aliases.

The fixed carrier is the exact left-to-right ordered-H4 fold of recent suffix
lengths 1, 2, 3, and 4, compared in that lexicographic order. The `run`
predecessor has length three and occupancy `[true,true,true,false]`; its fourth
slot is typed H4-identity padding with an explicit occupancy bit. The `runs`
predecessor has length four and occupancy `[true,true,true,true]`. Live query
histories use the same encoder and table. A manifest/table mismatch returns
`UNAVAILABLE_FRAME_MISMATCH`.

For each prompt, the preflight first ran the live repaired support query over
the prompt history. Its causal singleton support supplied the observed `still`
route, which was appended before the decisive `{run,runs}` query. `still` was
not injected as a selector candidate: the raw census records two causal
singleton-support observations and zero selector candidate-append inputs.

The repaired admission contract remained exact on both prompt orders:

| Step | Rows | Available | Examined/admitted | Union | Source counts `(I1,I2,IS,D,AS)` | Historical path work |
|---:|---:|---:|---:|---|---|---:|
| prompt to `still` | 7 | 8 | 3 / 3 | `{still}` | `(2,1,0,2,0)` | 5 keys / 5 comparisons |
| after observed `still` | 7 | 11 | 6 / 6 | `{run,runs}` | each `(1,1,0,1,0)` | 6 keys / 12 comparisons |

Every decisive placement arm performed two prototype evaluations and eight
slot comparisons. Within each prompt, real, placement-permuted, and
order-shuffled controls reused the same overlay artifact, causal history,
support, admission, and work; only the predeclared prototype placement or
construction order changed.

The frozen real-placement inventory was:

| Prompt | Candidate | Exact shell vector, suffix lengths 1→4 | Exact-slot mask |
|---|---|---|---|
| left | `run` | `Coincident, Coincident, Coincident, Degrees120` | `true,true,true,false` |
| left | `runs` | `Coincident, Coincident, Orthogonal, Degrees120` | `true,true,false,false` |
| right | `run` | `Coincident, Coincident, Orthogonal, Degrees108` | `true,true,false,false` |
| right | `runs` | `Coincident, Coincident, Coincident, Coincident` | `true,true,true,true` |

Both candidates tie on suffix lengths one (`still`) and two (`generally
still`). The shortest-to-longest lexicographic contract therefore decides at
length three: the left retained suffix exactly recalls `athletes generally
still` and points to construction candidate `run`, while the right retained
suffix exactly recalls `athlete generally still` and points to construction
candidate `runs`. The controlling earlier subject is outside the decisive
width. No complete six-route held-out history equals a construction
predecessor, but that full-history anti-recall check is insufficient: the
operative retained representation contains these exact shorter construction
subhistories.

Attaching the already-frozen expected continuations only after the relation
census produced:

| Placement/control | Left decisive relation | Right decisive relation | Intended matches |
|---|---|---|---:|
| real | `run` | `runs` | 0 / 2 |
| canonical cyclic placement permutation | `runs` | `run` | 2 / 2 |
| reversed construction order | `runs` | `runs` | 1 / 2 |

This is clean separability with the wrong causal orientation, not an H4 class
collision. The same-artifact placement-permuted control exactly reproduces the
desired pair and therefore outperforms the real mechanism. The strict
frozen-contract real-placement `strict_selection_ceiling` is `0/2`;
`PASS_LOCAL_CONTEXT_PLACEMENT_PREFLIGHT` is `UNAVAILABLE`. Per the frozen
contract, no selector or report-schema change was added; decoded generation and
decoded replay were `NOT_RUN`, and no scalar,
cost order, width, fixture, prototype cap, or second representation was tried.
The terminal remains `REVISE_I1_GENERATOR_IN_PLACE`.

The overlay, label-free preflight input, and raw selection-blind census use only
canonical construction witnesses, immutable addresses, exact H4 tables, and
observed query history. Expected-continuation, held-out-label,
actual-future-route, source-tensor, teacher, provider, and candidate-append
inputs are all zero. Canonical overlay bytes and the complete raw census
reproduced byte-for-byte before expected continuations were attached to produce
the separate label-attached outcome.

Focused verification for this revision:

- the exact raw relation-census freeze passed at
  `blake3:50e67c087e1ec5e04aa47cf09d42e9b0857c15e4cafa07b1363d67faf96c6aeb`;
- the exact label-attached hard-stop outcome passed at
  `blake3:d5e2f3614c8f1d3c6c629e2261ec42dc970fa5484982b2a86cb4f4b06b06a372`;
- the explicit historical-generator quarantine passed without constructing or
  running the generator;
- the focused #969 causal-path decoded regression retained record
  `blake3:60360a9e22a56ea4af363e43f7103bb8104d015d58feb582d921fc17afaf207f`;
- focused core/test compilation and changed-mechanism clippy passed after
  excluding unchanged `origin/main` lint categories;
- the exported WASM library check passed with existing warnings outside this
  revision; and
- formatting, claim wording, and diff-whitespace checks passed.

Decoded #953 generation/replay, workspace-wide tests/clippy, no-std ladders,
corpus/model/teacher work, generation canaries, BDD, Gate C, kappa
reproduction, audit, fuzz, formal proof, conformance, product QA, performance,
and release qualification were `NOT_RUN`.

This result does not establish grammar, coherent generation, semantic
placement, correctness, higher-scope attention, reasoning, performance
advantage, formal closure, product readiness, or release readiness. #953 awaits
a newly frozen maintainer plan inside its existing scope; #973 and #954 remain
blocked, and no downstream issue is activated.

## A1Q-L3 blocker handoff (2026-08-28)

The final sentence above records the ownership state when this append-only #953
evidence was written. Live sequencing now keeps #953 open, parked, unassigned,
and untouched behind #986. Only a full-positive #986
`CorpusSignedTransportV1`, or a separately qualified table-value successor
created by #986's table branch, may enter this decoded loop after a fresh
label-free preflight. The known #953 population remains quarantined; this
handoff authorizes no new representation, generation run, or replay inside it.
