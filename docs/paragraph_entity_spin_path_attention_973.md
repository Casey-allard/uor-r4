# #973 paragraph entity SpinTorsion-path attention record

- **Date:** 2026-08-28
- **Issue:** #973
- **Contract:** [frozen on the live issue](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5454973642)
  before implementation or outcome-bearing geometry and decoding
- **Mechanism:** `ParagraphEntitySpinPathR4V1`
- **Outcome:** positive on the exact two-history paragraph contract
- **Terminal:**
  `RETAIN_PARAGRAPH_ENTITY_SPIN_PATH_ATTENTION_CONTINUE_CONVERSATION`
- **Issue state after this result:** #973 remains open; #954 remains blocked

## Question and accepted input

The empirical question was whether one construction-bound path over the exact
stored lexical `SpinTorsionState` could make a typed paragraph entity binding
causally change decoded output while all of the following stayed fixed:

- both descriptor facts occurred once in both held-out histories;
- neither admitted candidate occurred in either observed prompt;
- the local #953 candidate support, coordinates, radii, fallback, and declared
  work were identical; and
- only the entity-to-descriptor binding changed.

The operator consumes only the count-one maximum-count tie already admitted by
the unchanged #953 `MultiscaleCountRadiusR4V1` API. It does not alter the
source-free table, lexical ids, active backoff row, support, decoder, or later
#953 continuation policy. The production #989/#953 artifacts remain unchanged;
this record concerns a separate two-document synthetic fixture.

## Frozen synthetic contrast

Construction used D3-construction partition IDs `20` and `21`:

```text
20:
Nora carried the striped marker.

For Nora the registry code is amber.

21:
Owen carried the dotted marker.

For Owen the registry code is cobalt.
```

The target-free census used D3-held-out partition IDs `26` and `38`:

```text
26:
Mara carried the striped marker. Iris carried the dotted marker.

For Mara the registry code is

38:
Mara carried the dotted marker. Iris carried the striped marker.

For Mara the registry code is
```

Construction and held-out IDs and text CIDs were disjoint. Construction
entities `{Nora, Owen}` and held-out entities `{Mara, Iris}` were disjoint. The
two held-out prompts had the same canonical lexical multiset and query frame;
only the two entity bindings differed. Exact admitted tokens ` amber` and
` cobalt` occurred zero times in both prompts.

The expected continuations ` amber.` and ` cobalt.` were predeclared publicly
in the frozen issue contract. The focused test attached them only after the
target-free operator and census replay had frozen. This is an execution-order
separation, not a blinded, cryptographically sealed, or independently hidden
evaluation.

The prose is synthetic. D3 enforces the declared id partition for this probe,
but supplies no corpus provenance, natural-distribution evidence, or semantic
transfer.

## Frozen construction-bound operator

Exact parsing resolves the descriptor bytes bound to queried entity `Mara`.
Construction binds the two candidate prototypes

```text
prototype(amber)  = F(striped)
prototype(cobalt) = F(dotted)

F(d) = G(carried) ⊙ G(the) ⊙ G(d) ⊙ G(marker)
```

where `G(u)` contains the exact mapped H4 state plus stored Q29 fiber and
torsion phases. H4 multiplication and inverse are exact table operations;
fiber and torsion compose with bounded integer phase wrapping. The exact
spin-to-H4 map accepts only S3 coordinates that are exact multiples of `2^29`
and resolves them by coordinate equality in the bound H4 root table. It does
not use nearest-root projection, raw table-index distance, a float scorer, or a
Cayley approximation.

At query time the descriptor equality key selects one already compiled
construction path, then refolds its four stored leaves. The query path does not
induce a new semantic representation from held-out prose. It is exact
construction-derived descriptor recurrence.

For each unchanged #953 candidate `c`, selection uses

```text
Delta(H,c) = prototype(c)^-1 ⊙ F(binding_H(Mara))
cost(H,c)  = (H4S3AngularShell(Delta.h4),
              circular_abs_q29(Delta.fiber),
              circular_abs_q29(Delta.torsion))
```

The unique lexicographic minimum wins; an equal minimum abstains to #953. The
query path uses prevalidated H4 state, product-row, inverse-index, and coordinate
caches, so its H4 operations remain exact bounded table reads rather than
revalidating or searching the root table at query time.

The stored spin state is upstream procedural provenance, not a learned semantic
embedding. Its S3 row is assigned from canonical lexical unit ID, and its phase
coordinates are deterministic functions of that unit ID and its registered
prime. The geometry is therefore bound to this construction vocabulary and
prime registry. The construction vocabulary includes the candidate readouts;
this result does not establish vocabulary-independent or intrinsic word
geometry.

Every path leaf records its lexical unit ID, registry index, prime, address
kappa, radial `Z[phi]`, payload CID, S3/Hopf/fiber/torsion state, and exact H4
coordinate. Each candidate separately records the same address/payload/radial
provenance and exact payload inversion. Candidate geometry is explicitly not a
ranking coordinate: candidate identity indexes the already-admitted support
and its construction prototype only.

The artifact binds the table and overlay CIDs, construction IDs and text CIDs,
codec, vocabulary, route manifest, grammar, routing policy and fixed-work
schedule, spin-to-H4 map, H4 tables, bounds, and complete candidate/prototype
traces.

## Matched controls

Four arms share the original paragraph bytes, #953 support, two-fact scan, two
entity comparisons, full two-row descriptor scan, four stored-leaf reads, two
candidate relations, fixed two-row minimum scan, and final-choice schedule:

- **real:** rank with the descriptor bound to `Mara`;
- **paragraph-disabled:** complete the same path and relation census, suppress
  its ranking costs, and return the #953 fallback;
- **entity-binding-permuted:** use the other fact's descriptor after the same
  parse; and
- **fact-order-reversed:** reverse the parsed fact vector while preserving its
  bindings.

The last arm is an internal parsed-vector iteration-order control under
unchanged paragraph bytes. It does not establish equivariance to feeding a
textually sentence-reversed paragraph.

## Target-free hard-gate result

The source-free table, #953 overlay, and paragraph operator were serialized and
reloaded from their canonical bytes before the census. Both target-free
predictions and the complete census were then recomputed byte identically
before expected continuations were attached.

Both prompts ended in the exact active trigram frame ` code` / ` is`. The local
support was exactly ` amber` and ` cobalt`, each at count one. Both candidates
had identical #953 coordinates

```text
(2^31, 2^31, 143165576, 3 * 2^30)
```

and identical #953 radius `19620161960467810368`. The unchanged #953 choice was
therefore the canonical fallback ` amber`, with identical support and declared
local work. Each prompt had 29 encoded lexical units.

The prior-prefix Gate 0 `PriorSentenceCountRadiusR4V1` operator separately
abstained with `NoPriorCandidateOccurrence` on both prompts. Thus exact admitted
candidate copying is excluded for this fixture.

The two construction paths had the same final H4 coordinate. Their stored phase
states were

```text
F(striped) = (H4_tied, fiber=160683320, torsion=-15268680)
F(dotted)  = (H4_tied, fiber=144614988, torsion=-13741812)
```

Every matching relation had cost `(Coincident, 0, 0)`. Every nonmatching
relation had cost `(Coincident, 16068332, 1526868)`. S3, Hopf, and mapped H4
were also identical for the two descriptor leaves. H4 orientation therefore
did not discriminate these candidates. Fiber was the first and sufficient
lexicographic discriminator. Torsion was nonzero, recorded, and evaluated, but
was not independently load-bearing because selection was already decided by
fiber.

The target-free first-unit matrix was:

| Prompt binding for `Mara` | Real | Paragraph disabled | Binding permuted | Parsed-vector reversed |
| --- | --- | --- | --- | --- |
| `striped` | ` amber` | ` amber` | ` cobalt` | ` amber` |
| `dotted` | ` cobalt` | ` amber` | ` amber` | ` cobalt` |

Support mismatches and declared-work mismatches were both zero. Structural
teacher, provider, source-weight, future-unit, and target-read counters were
all zero. These counters describe the bounded module/API closure; they are not
a system-wide dynamic information-flow audit.

## Decoded causal consequence

After the target-free replay froze, the test attached the two predeclared
continuations. The paragraph operator owned only the first selected unit;
subsequent units returned to unchanged #953 selection, appended `.`, observed
EOS, and terminated.

| Arm | ID 26 | ID 38 | Exact predeclared continuations |
| --- | --- | --- | ---: |
| Real | ` amber.` | ` cobalt.` | 2/2 |
| Paragraph disabled | ` amber.` | ` amber.` | 1/2 |
| Entity binding permuted | ` cobalt.` | ` amber.` | 0/2 |
| Parsed fact vector reversed | ` amber.` | ` cobalt.` | 2/2 |

All eight continuations emitted two lexical units and observed EOS. The real
arm changed the second history from the unchanged #953 fallback, while the
binding-permuted arm reversed both choices under the same admitted support and
declared work. This is the bounded decoded causal consequence; a distinct
serialized trace alone would not have qualified.

Support/work equality is claimed at the shared first decision only. Once arms
choose different first units, their later contexts differ and no cross-arm
support/work equality is inferred.

## Canonical artifacts and replay

| Item | Result |
| --- | --- |
| Tiny synthetic source-free table CID | `blake3:ddf8c096136a7e9880cdd9c40ba24cf5fc5a2665abfe9cda4b73ad96028dcadd` |
| Tiny #953 overlay CID | `blake3:9c2fbb41e5572d46baea359aca06a94d90377fdfe3572078c29d38b882df8045` |
| #973 paragraph operator bytes | 7,531 |
| #973 paragraph operator CID | `blake3:9221efa7ad952e4890aae335970418b38ec93beb8cb4de65c5aa1d8c67f70afd` |
| Codec kappa | `blake3:6653eacd79236f78007626cc726cc295fb8be94d91e140d58b83a8f32c4824b2` |
| Vocabulary kappa | `blake3:b75a4a1aebbc3dad8773125dc5881847de856d85f5e149a5c8e3720fbd65c6a9` |
| Route-manifest kappa | `blake3:c53a300ba9df268960e6f852c2ddbefbbdbf9530ea86094120ee7ae9a5279b38` |
| Exact spin-to-H4 map kappa | `blake3:5cc6d40c7b80e7c0519a08d95ea33ef6b079c9decfbd602c99c9937fabb7bb01` |
| Grammar kappa | `blake3:56b847165e8645dc553a5aad5c3585a3fb06ae0bfacc8d899c36e662b141ffb8` |
| Routing-policy/work kappa | `blake3:4ac4c2a04f26087fd493ed8317347afb7fb49a0143ff2cf59783d2ca20709991` |
| H4 root-table kappa | `blake3:8d33d62a239fb8001fea2bd14a9a5ec7321d0f07d81c74a5715eaeb3df53aa76` |
| H4 multiplication-table kappa | `blake3:90ee73a27ee2e8ba5bccd1507d7fb37ed1f044b1640772c86752bc0bb2111759` |
| Target-free census CID | `blake3:515720686b96dbebc2f055f9a21d3f0684f76092018381c836b51abf47a4d197` |
| Decoded-smoke CID | `blake3:0ba32e5fe26f1280ec2eef2b115023de52f2ef946882352311dcecc531d76a32` |

The focused test reloaded the table, overlay, and operator from canonical
bytes, recomputed the two target-free predictions and census, and replayed the
two decoded continuations and complete smoke report byte identically. The CIDs
identify deterministic in-memory probe artifacts reconstructed by the focused
test; no generated binary fixture is checked into the repository. This is a
same-test deterministic replay, not an independent-machine reproducibility
claim.

## Decision and claim boundary

The frozen positive terminal is
`RETAIN_PARAGRAPH_ENTITY_SPIN_PATH_ATTENTION_CONTINUE_CONVERSATION`.

This establishes only one bounded synthetic construction-bound
exact-descriptor/entity-binding phase-path selector. Within this exact
implementation, its stored phase score changes an already-admitted candidate
and two-unit decoded continuation under matched controls even though the
candidate token is absent from the observed paragraph. It is therefore a
non-candidate-copy decoded causal mechanism.

Exact descriptor recurrence remains the operative lookup key. No arm compares
this mechanism against a direct non-geometric descriptor-to-candidate map, so
the result does not establish a geometric advantage over that simpler lookup.
It also does not establish semantic or paraphrastic similarity, anti-recall
beyond exact candidate absence, natural/corpus transfer, vocabulary-independent
or intrinsic spin meaning, a general entity model, general paragraph attention,
conversation/global/recursive attention, grammar, broad coherence,
correctness, reasoning, performance advantage, allocation-free serving, chat
readiness, formal closure, or release readiness.

#973 remains open. The next decision-bearing action is one independently frozen
conversation-scope contrast. It must establish its own candidate-relative
decoded consequence under matched disabled and permuted controls; it inherits
no conversation claim from this paragraph result. Bounded-global and corpus
induction remain `NOT_RUN`. #954 stays blocked until #973 reaches its full
native terminal.

## Activated checks and observed cost

- focused target-free census, sealed decoded matrix, and exact replay: PASS in
  178.33 seconds under the debug test profile;
- exact real/disabled/permuted/reversed matrix: 2/2, 1/2, 0/2, 2/2;
- support mismatches: 0;
- declared-work mismatches: 0;
- touched-package compile and the required wasm library compile: PASS;
- claim-wording, Rust-format, and diff-whitespace checks: PASS; and
- terminal:
  `RETAIN_PARAGRAPH_ENTITY_SPIN_PATH_ATTENTION_CONTINUE_CONVERSATION`.

The frozen contract estimated a sub-minute focused probe. The observed debug
run took 178.33 seconds because it performed full artifact revalidation. The
estimate was false and is corrected here; this record makes no performance
claim.

Workspace-wide tests, BDD, teacher/model paths, no-std ladders, Gate C, kappa
reproduction, audit, fuzz, conformance, corpus-scale, product, performance,
formal, and release qualification are `NOT_RUN`.

## Forward-action update (2026-08-28)

This bounded result remains unchanged. Its then-next sequencing is complete;
the then-current #973 work was ADR-0005
`PredictiveConnectionRetentionGate0V1`. The current 2026-08-29 action is
`ConnectionGaugeCovarianceV4`, following the negative gated-delta smoke and
the direct-attention V3 result (geometric `3/12`, fixed-tangent plain `12/12`).
