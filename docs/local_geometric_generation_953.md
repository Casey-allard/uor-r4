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
