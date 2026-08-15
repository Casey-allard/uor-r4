# Octeract route-attention investigation (#661)

This record is append-only evidence for the bounded investigation in #661. It
does not activate an operator or change a serving path. Phase A is owned by
#720; the trace ceiling and disposition remain a later, separately gated
phase.

## 2026-08-14 - Phase A source and conformance record (#720)

### Source boundary

The two research inputs were supplied out of band to the repository
maintainer. They are not repository artifacts. The audit found no license or
redistribution grant in either PDF's text or metadata, so the repository uses
`license: NOASSERTION` and `redistribution: not-authorized`. Neither PDF, its
figures, nor extended source prose may be committed without a separate grant.
Only independently stated definitions, executable tests, and analysis appear
here.

| Input | Local identity | Exact bytes | SHA-256 | Provenance / license |
|---|---|---:|---|---|
| *The Octeract Cypher: Coarse-Graining and Categorical Adjunctions in 8-Bit Phase Space Collapses* | `Octeract_Cypher_Paper.pdf`; 8 pages; PDF title metadata present | 54,969 | `44bab09a20253437aeef43057ae316fcded5b00fd9f6b180f83843f06d2bbb3c` | supplied out of band; `NOASSERTION`; redistribution not authorized |
| Metadata title *Validating Octeract Cypher Mathlib*; displayed title *Formal Validation and Mechanized Verification of the Octeract Cypher: A Base-2 Kaprekar Adjunction Framework* | `Validating Octeract Cypher Mathlib.pdf`; 12 pages | 262,762 | `5322c519fa872ca836e2ad23d523ecf655defedd3dd17589ba290dec62a93a5e` | supplied out of band; `NOASSERTION`; redistribution not authorized |

Audit date: 2026-08-14. The same identity fields are pinned as data in
`uor_r4_graph_certify::octeract`.

### Finite definitions used by the investigation

**Definition** Direct byte map. Split a byte into eight binary digits, sort
the digits once in ascending and descending order, interpret both arrangements
as bytes, and subtract the ascending value from the descending value. The
certifier's direct oracle implements those steps without calling `count_ones`
or the closed form.

**Definition** Closed form. For a validated byte Hamming weight
`k in 0..=8`, the independent algebraic implementation is

```text
K8(k) = 257 - (2^(8-k) + 2^k).
```

The complete weight map is:

| `k` | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `K8(k)` | 0 | 127 | 189 | 217 | 225 | 217 | 189 | 127 | 0 |

**Definition** Class identity. For a relation block with `B` active bits and
masked Hamming distance `d`, the canonical complement-folded class is
`min(d, B-d)`. On a full byte this has five values. The integers
`0/127/189/217/225` are a bijective labeling of those five values; they have no
declared ordering or metric meaning for attention.

**Definition** Oriented class. The folded class plus the predicate
`d > floor(B/2)` reconstructs `d` exactly (the even-width equator has only its
canonical low-side spelling). Consequently, oriented folding is an alternate
representation of ordinary block weight, not a new relation by itself.

### Page/claim/owner map

Page numbers refer to the supplied files above, not to any republished copy.

| Source location | Independently stated proposal | Classification | R4 owner / consequence |
|---|---|---|---|
| Cypher paper pp. 2-3, sections 2.1-2.3; validation pp. 2-3, sections 2-3 | Binary state, Hamming weight, sorted forms, and the closed form | `directly-applicable` | Certifier conformance oracle in `uor-r4-graph-certify::octeract` |
| Cypher paper p. 4, section 2.4 | Factorization through Hamming weight | `directly-applicable` after correcting the claim that the weight-to-output map is injective | The finite map is many-to-one under `k <-> 8-k`; no attention semantics follow from factorization alone |
| Cypher paper p. 5, section 3.3; validation pp. 3-5, section 4 | Five outputs and one-step idempotency | `directly-applicable` as finite byte facts | Exhaustive #720 tests; endpoints are handled explicitly |
| Cypher paper pp. 5-6, section 4.1; validation pp. 6-7, section 6.1 | Complement/equator symmetry | `directly-applicable` as a bounded finite identity | Exhaustive #720 tests; no attention-quality consequence is inferred |
| Validation pp. 5-6, section 5 | 6.1824-bit loss under a uniform distribution on all bytes | `already-covered` as an arithmetic consequence, not an R4 measurement | #661-B must measure actual route-trace occupancy and information; the uniform-byte number is not a baseline |
| Cypher pp. 6-7, section 4; validation pp. 7-8, section 6 | Complement/equator order duality described as a Galois connection | `adaptable-hypothesis` | The bounded complement map is an order duality; it is not, by that fact alone, the deployed `K8` relation or a complete attention operator |
| Validation pp. 8-10, section 7 | Suggested Lean/mathlib declarations | `incompatible` with a proof claim in their supplied form | The declarations contain `sorry` and are a roadmap. #653 is the existing proof-integration owner; #720 introduces no second registry or toolchain |
| Both conclusions/applications (Cypher pp. 7-8; validation p. 11) | Routing, stabilization, self-correction, and pipeline benefits | `adaptable-hypothesis` only | #661's locked collision/ceiling/null experiment owns any empirical disposition; no product or quality claim is imported |

### Corrections and bounded claim status

**Guarantee** Status: **Structural**. On the complete byte domain, the direct
sort/subtract implementation equals the independent closed form, identifies
complementary weights, has exactly the five outputs in the table above, and is
idempotent after one application. The oriented class round-trips every one of
the 45 canonical `(active width, distance)` states. Evidence:
[`octeract_attention_661.rs`](../crates/uor-r4-graph-certify/tests/octeract_attention_661.rs).

1. The weight-to-output map is not injective: complementary weights have the
   same output.
2. The non-trivial output-weight identity applies only for `0 < k < 8`.
   Both endpoint weights map to zero. Idempotency itself still holds at the
   endpoints.
3. This record does not adopt the source's Galois label as a property of `K8`
   or of the complete deployed attention operator. At `k = 0`, for example,
   complement weight is 8 while `K8(0)` has weight 0.
4. The validation PDF's Lean snippets contain admitted goals. This record makes
   no machine-proof claim and relies only on exhaustive finite execution for
   the byte-domain facts.
5. The source's entropy row assumes uniformly sampled bytes. #720 does not
   adopt it as evidence about teacher support, real route-code occupancy,
   language quality, or useful pruning on R4 traces.

### Math-to-operator boundary

**Definition** Candidate placement. The source material supplies only a
possible compatibility relation or prefilter. It does not supply the rest of
the versioned #602 operator:

| #602 field | #720 finding |
|---|---|
| projections | absent; current `route-fit/1` Q/K code construction remains the later baseline |
| positional action | absent |
| compatibility relation | relation-first hypothesis `fold(popcount((q XOR key) AND mask))`; encode-first and prefilter/refine are distinct candidates and may not be conflated |
| selector | absent; current bounded `(distance, candidate-index)` selection is not implied by the papers |
| aggregation | absent |
| output projection | absent |
| state | absent |
| tie behavior | absent |

Other existing Hamming consumers remain separate owners: the packed
route-attention relation, graph-runtime route probing/VP-tree distance, and the
legacy transformerless assignment/index path. The f64 router produces the
288-bit session signature but does not perform the Hamming comparison. A later
classification-only fallback, if the attention screen stops negative, belongs
at those comparison seats and may not silently alter them.

### Executable Phase A evidence

**Guarantee** Status: **Structural**. For every masked byte pair exercised by
the declared exhaustive/representative domains, the weight difference is a
lower bound on masked Hamming distance. Full-byte oriented folds reconstruct
ordinary block weight, while unoriented folds deliberately identify
complementary distances. Evidence:
[`octeract_attention_661.rs`](../crates/uor-r4-graph-certify/tests/octeract_attention_661.rs).

`crates/uor-r4-graph-certify/tests/octeract_attention_661.rs` checks:

- all 256 byte inputs against independently coded direct and closed-form maps;
- the exact five outputs, both endpoints, complement symmetry, and one-step
  idempotency;
- all 65,536 full-mask `(query,key)` byte pairs;
- the 47,616 non-equator pairs whose exact distances collide after key
  complementation (the 17,920 distance-4 pairs are the equator);
- all 45 canonical oriented states, five-anchor relabel invariance, and the
  paper's named `10101010` / `11110000` example;
- the safe lower bound
  `abs(wt(q AND m)-wt(key AND m)) <= wt((q XOR key) AND m)` across eight
  deterministic masks and every byte pair; and
- a deterministic 36-byte composition showing ordinary block weights and
  oriented folds reconstruct the same 288-bit Hamming sum while the
  unoriented fold is lossy.

Malformed weights, widths, distances, shells, and non-canonical equator
encodings do not construct the corresponding bounded value types; operations
over admitted values are total. No serving, graph-format, runtime,
operator-registry, fit-method, artifact, or kappa byte changes in this phase.

### Next gate (not run in Phase A)

**Empirical Criterion** Status: **Unproven** until #661-B runs. An
Octeract-derived attention arm advances only under the parent issue's locked
trace-ceiling, null, frame, and promotion rules on a pinned fixture.

The next child must freeze and run #661's predeclared cheap screen over the
existing #603-shaped trace surface. `weight9` and oriented folding are controls:
when their 36 block weights are summed, they reproduce current V1 Hamming.
`fold5` is the lossy candidate. A folded/lower-bound prefilter can report only a
theoretical pruning ceiling against the current flat-scan `RAT1`; it cannot
claim runtime byte savings without a new indexed representation.

The screen must include packed V1, fold5, oriented/weight controls,
prefilter/refine and the safe lower bound, occupancy-matched and shuffled-block
nulls, and the #647 frame control. Missing real trace/corpus prerequisites are
recorded as `UNAVAILABLE`, never as zero or a negative quality verdict. At most
two candidates may advance; no packed or long run begins before that decision.

## 2026-08-14 - Phase B route-trace screen record (#722)

### Frozen screen contract

**Definition** Phase B is the certifier-side
`uor-r4-octeract-trace-screen-contract/1` contract and
`uor-r4-octeract-trace-screen/1` report. It fixes layer 0/head 0, 288 route bits
as 36 natural bytes, the current full mask, causal candidates in ascending
index order, stable ascending `(score,candidate-index)` ties, selection width
`M = min(8, trace support cap)`, and only steps with more than `M` candidates.
The full contract is embedded in every report before any support labels are
read.

The fixed rows are:

| Row | Role | Relation / work rule | May advance |
|---|---|---|---|
| V1 baseline | frame and operator control | sum 36 full-byte XOR popcounts; existing packed/reference route-attention selection | no |
| `weight9` | representation control | independently materialize each exact byte distance in `0..=8`, then sum | no |
| `octeract-fold5` | direct candidate | sum the 36 complement-folded shell classes | yes |
| `octeract-oriented` | representation control | retain shell plus high-side orientation, reconstruct every exact byte distance, then sum | no |
| `octeract-prefilter` | prefilter candidate | shortlist the first `floor(3N/4)` fold-ranked candidates, then refine with V1 | yes |
| safe lower bound | pruning control | skip a later exact distance only when its weight-difference lower bound cannot beat the current stable-tie worst selection | no |

The fixed nulls are the occupancy-matched fold null with seed `0x661B0001`,
the single nonidentity 36-block permutation with seed `0x661B0002`, and the
cyclic deranged-support null. The direct-fold gate requires a mean Jaccard
improvement of at least `0.03` over V1 and strict separation from every
applicable null. The prefilter gate requires at least `0.95` V1-selection
recall, at most `0.75` exact-refinement fraction, at least one work-eligible
step, and strict null separation.

**Guarantee** Status: **Structural**. Only `octeract-fold5` and
`octeract-prefilter` can produce an advance disposition. Synthetic or manually
constructed evidence is registry-limited to `instrument-conformance` and can
produce neither empirical `PASS` nor empirical `FAIL`. The initial closed
evidence registry has no `pinned-real` entry. Adding one requires new contract
and report format versions rather than changing either `/1` record in place.
Evidence:
[`octeract_trace_screen_661.rs`](../crates/uor-r4-graph-certify/tests/octeract_trace_screen_661.rs).

### Provenance and instrument controls

The report repeats both Phase A source identities:

| Bound source | SHA-256 |
|---|---|
| Octeract Cypher paper | `44bab09a20253437aeef43057ae316fcded5b00fd9f6b180f83843f06d2bbb3c` |
| validation roadmap | `5322c519fa872ca836e2ad23d523ecf655defedd3dd17589ba290dec62a93a5e` |

A future empirical evidence record must additionally bind one authoritative
#603 observation identity bundle and its input CID, #597 source-manifest
identity, source geometry and tokenizer-adapter records, registered source
attention operator, merged records and trace kappas, `full/1` profile,
`route-fit/1` method, fit manifest, fitted parameters, target
`r4-route-attention/1`, source snapshot, adapter, and compiler identities. The
teacher source snapshot kappa and #597 source-manifest kappa are separate
fields; neither is substituted for the other.

**Guarantee** Status: **Structural**. Input validation recomputes registered
records and full record equality, checks observation/trace/fit alignment before
scoring, and returns typed `UNAVAILABLE` for malformed geometry, missing lanes,
unknown records, incomplete identities, or an unregistered evidence class.
The empirical path also recomputes `route-fit/1` and requires its complete
output to equal the supplied fitted artifact. Publicly constructible metadata
cannot promote a synthetic bundle by changing an adapter string or requested
trace kind. Evidence:
[`octeract_trace_screen_661.rs`](../crates/uor-r4-graph-certify/tests/octeract_trace_screen_661.rs).

The same executable evidence covers V1 packed/reference/scalar identity,
independent `weight9` materialization, oriented reconstruction, fold and
complement adversaries, prefilter fidelity and no-work cases, safe lower-bound
ties, all 120 shell-label bijections, deterministic and nonvacuous nulls,
malformed-geometry no-panic behavior, and exhaustive small-domain comparison
of the bounded collision oracle with brute force. The report keeps common base
occupancy once and gives every arm/null its own consumed-domain counts. That
distinction is material: the conformance fixture's matched rows consume 20
support entries while the cyclic deranged-support row consumes 19; a separate
`M=8` fixture has base counts `1/4/42/32` for
stories/steps/candidates/support entries but prefilter work-domain counts
`1/2/23/16`. Typed optional fields likewise keep row-specific selection,
collision, oracle, shortlist, prefilter, lower-bound, and transformed-null
occupancy metrics at their actual owners.

### Availability and canonical disposition

No registered pinned-real `full/1` evidence record or corresponding exact
observation/records/trace/fit bundle was delivered by #603-#606 or present in
the configured local fixture inventory. The only local corpus manifest was the
Simple Wiki corpus input; it is not a `full/1` route-trace evidence bundle and
was not substituted. Phase B therefore exercised the explicit missing-input
branch rather than generating a checkpoint, searching for an approximate
fixture, or relabeling the synthetic #605 conformance data.

**Empirical Criterion** Status: **Unproven**. No real support labels were
available, so no attention-quality, collision-ceiling, null-separation, or
prefilter-work result is claimed.

The canonical missing-real report is:

| Field | Frozen value |
|---|---|
| trace kind | `pinned-real` |
| reason | `required explicitly supplied pinned full/1 trace input is absent` |
| report bytes | 6,587 |
| payload kappa | `blake3:8b8c3bdc41f04ac2d6b9a15ef843f5064fae92e8b6bfe57cafbc6803eca7c5a2` |
| final envelope kappa | `blake3:eab7b1bb12d9508d9815da0c4fbac248eab8b93b8258e717829542e41ac75e5e` |
| disposition | `UNAVAILABLE` |

| Arm / null | Verdict | Reason |
|---|---|---|
| V1 baseline | `UNAVAILABLE` | required explicitly supplied pinned `full/1` trace input is absent |
| `weight9` | `NOT_RUN` | unavailable prerequisite/instrument |
| `octeract-fold5` | `NOT_RUN` | unavailable prerequisite/instrument |
| `octeract-oriented` | `NOT_RUN` | unavailable prerequisite/instrument |
| `octeract-prefilter` | `NOT_RUN` | unavailable prerequisite/instrument |
| safe lower bound | `NOT_RUN` | unavailable prerequisite/instrument |
| occupancy-matched fold null | `NOT_RUN` | unavailable prerequisite/instrument |
| shuffled-block null | `NOT_RUN` | unavailable prerequisite/instrument |
| deranged-support null | `NOT_RUN` | unavailable prerequisite/instrument |

Repeated construction produces identical canonical bytes, payload kappa, and
final envelope kappa. Typed absent identities remain absent; the two Phase A
source hashes and the complete preregistered contract remain present.

### Decision and compatibility

The locked unavailable branch applies: create neither #661-C nor the
negative-only classification/cache fallback. The Octeract attention mechanism
remains dormant and unavailable unless a later, separately reviewed issue
delivers and registers one exact real evidence bundle under new contract and
report versions.

The #310 weighted-Hamming experiment is cited, not rerun: its A-W rows recovered
about 5% of the A-f32 gap, below the preregistered 25% shipping bar, and shipped
nothing runtime-side. That negative sign-space result is context, not Octeract
evidence.

No fit method, attention-operator registry entry, packed instance, graph
format, runtime kernel, artifact, serving path, default, historical Phase A
record, or existing kappa fixture changes in Phase B. The added `libm` use is
certifier-only and makes entropy/MI report arithmetic independent of a native
`f64::log2` implementation; it does not enter the deployed integer runtime.
