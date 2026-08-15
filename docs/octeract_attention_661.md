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
