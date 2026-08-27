# A1P paired-H4 R4-heatmap identifiability — issue #970

## Status and decision

**Terminal bounded result:**
`RETAIN_H4_STATE_ONLY_ADVANCE_MULTICHANNEL_A1Q`.

The corrected public contract evaluates both exact H4 operands

```text
X(H,c) = C(H,c)
Y(P_c,c) = C(P_c,c)
D(H,c) = X(H,c) * Y(P_c,c)^-1
```

and runs the binary translation against the exact signed R4 heatmap derived
from `D`. It does not score one H4 operand, an opaque table offset, a digest, or
a fitted sign/threshold. The paired heatmap remains useful structural state,
but its exact classes alias incompatible candidate outcomes on the frozen
construction/validation envelope. The predeclared hard gate therefore stopped
before scalar search, selector compilation, validation selection, control-arm
execution, or placement intervention.

This is a bounded readout-identifiability negative. It is not evidence that
H4×H4, the exact R4 heatmap, fixed-zeta phases, ordered n-lets, or golden radial
transport have no value. It establishes no attention, inference, generation,
correctness, reasoning, or product readiness.

## Bound identities

| Artifact | Identity | Status |
|---|---|---|
| #952 partition | `blake3:d008b82eda9b16b102cf4c7ffa4a47a40ad514b30f0763ed3f46c0ebae3e277b` | reproduced |
| Construction artifact | `blake3:2b70588d654c8e8bb2d8ab063f41853d45a21487d742ff7567f93a42cfb9011b` | reproduced |
| Embedded schema-2 manifest | `blake3:1c77c4103732964af6776f1dfcabc8b2a9191eea875a8ba205c36ebbf5618a99` | reproduced |
| Inherited #967 report | `blake3:f0db7a5d5c81d51ebf3b4bf8a2715c4960ec16b14161e8bf7598d7b98c48c881` | reproduced |
| `A1P-CONSTRUCTION-1` | `blake3:fb5f27fc1107f527d616f32affa8eba1746a2f60cfdb95ddbb21a0e493299652` | reproduced |
| `A1P-VALIDATION-1` | `blake3:ecbe8b404e7542d801ff4b4e66c91a41f90158d84efa484dc4edb53aff38b602` | reproduced |
| Paired-H4/R4 heatmap contract | `blake3:2daacf538c022fab9580d1e124af6c18d0b06da04604fbc962a01bda57f08a98` | reproduced |
| Complete 120×120 pair census | `blake3:dca725c0ec6060166bcd0023df956e1ff029661b5fa7800ccb9f20808712b796` | reproduced twice |
| Corrected #970 report | `blake3:5f9239150dea8c0c27c4dfa6ad2e4d0068bc3d18afc127b315c0ec358ceddb3f` | reproduced twice, byte-identical |

The former local single-`D` report κ
`blake3:0359b08da5318b737379d4b5f1e2cbf1782b01960682b6defe308c46bd868eef`
is a superseded scope diagnostic. It is not terminal evidence and is not used
as the result identity.

## Exact heatmap contract

For

```text
D = (q0 + q1 i + q2 j + q3 k) / 2,
qk = ak + bk*phi in Z[phi],
```

the frozen oriented `(1,i)` chart is

```text
sin        = q0 / 2
cos        = q1 / 2
activation = sin^2 = q0^2 / 4
chirality  = sign(sin)
polarity   = sign(cos).
```

All values remain exact in `Z[phi]`; no float or rounding enters the class
key. `q2` and `q3` remain in the full `D` witness but are not scorer-key fields.
When both `q0` and `q1` are zero, the chart is a typed-null abstention.

The geometric endpoint bit is:

| Heatmap endpoint | Activation bit | Retained orientation |
|---|---:|---|
| `sin=+1, cos=0` | 1 | positive chirality |
| `sin=-1, cos=0` | 1 | negative chirality |
| `sin=0, cos=+1` | 0 | positive cosine polarity |
| `sin=0, cos=-1` | 0 | negative cosine polarity |

The activation bit is not a candidate label. `SELECT`/`REJECT` is compiled
only when a complete exact construction heatmap class is pure. Non-landmark
roots retain their exact class; unseen, impure, and typed-null classes abstain.

The same immutable contract binds the fixed zeta grid κ
`blake3:512243ed9e2c1deef0691515caf02ca25e3d5c7990184cd804f6d65c1cc8d94c`,
ordered multiplicity-preserving prime n-lets, and the exact golden shell maps

```text
(a,b)*phi    = (b,a+b)
(a,b)*phi^-1 = (b-a,a).
```

It also retains the typed cross-chart marker
`Euclidean sqrt(2) <-> complex 2i <-> Riemannian [0,2]`. These are typed
representatives; units, orientation, normalization, and error must be supplied
by an adapter rather than treating the raw values as ordinary scalar equality.
No zeta/n-let-to-`phi` shell-exponent rule was supplied or established, so the
binding is truthfully
`STRUCTURAL_BINDING_ONLY_NO_ZETA_NLET_TO_PHI_EXPONENT_RULE`. Raw zeta identity,
n-let identity, and radial magnitude are excluded from the scorer.

## Complete paired structural universe

The probe enumerated the entire frozen ordered pair domain before inspecting
any fixture labels:

| Measure | Result |
|---|---:|
| Exact H4 operands | 120 |
| Ordered `(X,Y)` pairs | 14,400 / 14,400 |
| Exact relative `D=X*Y^-1` image | 120 / 120 |
| Pair multiplicity per relative image row | 120 |
| Exact signed `(1,i)` heatmap classes | 45 |
| Pairs projecting to typed-null heatmap rows | 480 |
| Integer-only, no rounding | yes |
| Target/label input | none |

The report stores the complete ordered-pair census κ plus the 120 unique
relative-image heatmap rows. This is equivalent to serializing 14,400 repeated
rows because every exact relative row has multiplicity 120; the focused test
independently re-enumerates that multiplicity from the frozen multiplication
and inverse tables.

## Independent fixtures and anti-recall boundary

The three populations remain separate:

| Population | Histories | Candidate decisions | History suffix | Role |
|---|---:|---:|---|---|
| Inherited A1R regression | 6 | 12 | `cc qq` | falsification/root-cause only |
| `A1P-CONSTRUCTION-1` | 6 | 12 | `dd qq` | derives the sign-rule orientation |
| `A1P-VALIDATION-1` | 6 | 12 | `bb qq` | held-out anti-recall transfer |

All 18 exact histories are disjoint. The S4 parity for every construction and
validation row is derived by inversion count from the exact history and frozen
role order `[aa,bb,cc,dd]`; the serialized parity literal must reproduce it.
Malformed or duplicate-role histories fail closed.

Construction and validation geometry/support are prepared from a type that has
no observed-next field. Both full populations are complete before the separate
label ledgers are joined for class purity and ceiling calculation. Permuting
validation labels therefore cannot change any preparation input.

Every population preserves:

- natural admitted candidates `{ll,rr}` on 6/6 queries;
- 7 row reads/query, 42 total;
- 2 candidate entries examined/query, 12 total;
- candidate-entry ceiling 56/query and candidate ceiling 8;
- maximum two admitted candidates;
- 12/12 exact payload inversions;
- direct/exact and divisor misses on 6/6 queries;
- adjacent-spin-only support;
- no target injection, future event, admission truncation, or support drift; and
- support-denominator κ
  `blake3:7b17bdc7f3686fd734c1770a4d6c5cc1a0590accbe8ceaa4e143ec1cd3369777`.

## Paired-H4 R4-heatmap identifiability

| Measure | Result |
|---|---:|
| Exact classes across 36 decisions | 14 |
| Inherited classes | 9 |
| Construction classes | 10 |
| Validation classes | 7 |
| Construction coverage | 12 / 12 |
| Impure construction classes | 0 |
| Validation decisions covered by construction | 10 / 12 |
| No-class-splitting validation oracle ceiling | 10 / 12 |
| Construction-transfer strict-selection ceiling | 0 / 6 |
| Incompatible exact heatmap classes | 8 |

The original decisive same-candidate alias survives in exact heatmap class
`R4-HEATMAP-004`:

- `A1R-R05 / ll / SELECT`, relative root offset 70;
- `A1R-R06 / ll / REJECT`, relative root offset 70; and
- `A1P-C04 / ll / SELECT`, relative root offset 70.

Its exact heatmap is `sin=0`, `cos=1/2`, activation `0`, zero chirality,
positive cosine polarity. The same complete heatmap class would have to select
and reject `ll`.

The binary landmarks themselves also demonstrate why the geometric activation
bit cannot be mistaken for a label:

- `R4-HEATMAP-005` has `sin=0`, `cos=+1`, activation bit `0`, yet contains
  both `SELECT` and `REJECT` outcomes;
- `R4-HEATMAP-014` has `sin=+1`, `cos=0`, activation bit `1`, yet likewise
  contains incompatible candidate outcomes.

Independently, no validation query receives both a construction-derived pure
positive class for its intended candidate and a pure negative class for the
alternative. The strict transfer ceiling is therefore 0/6 even though the
exact heatmap improves class coverage from the superseded single-`D` diagnostic
to 10/12 validation decisions.

## Existing-additive comparator

The unchanged exact class compiler was also applied to
`(A*(H),A*(c),A*(P_c))`. `A*` is the existing non-digest additive projection
with spelling, position/occurrence identifiers, lexical-unit ID, prime and
factor identity, address/boundary/chain identity, κ/digests, and provenance
absent by construction.

| Measure | Result |
|---|---:|
| Exact classes across 36 decisions | 2 |
| Construction coverage | 12 / 12 |
| Impure construction classes | 2 / 2 |
| Validation decisions covered by construction | 12 / 12 |
| No-class-splitting validation oracle ceiling | 6 / 12 |
| Construction-transfer strict-selection ceiling | 0 / 6 |
| Incompatible retained classes | 2 / 2 |

The additive comparator cannot compile a pure construction scorer. It is not
relabeled as a tied or weaker exercised selection arm.

## Downstream controls and claim boundary

The identifiability failure is the predeclared hard stop. The exact frozen rows
are `full-paired-h4-r4-heatmap`, `current-only`,
`additive-summary-compiled-scorer`, `factor-count-only`,
`deterministic-geometry-permutation`, `candidate-relabeling`,
`prime-assignment-permutation`, `hierarchy-disabled`, `exact-recall-only`, and
the optional `placement-intervention`.

Every row is `NOT_RUN_IDENTIFIABILITY_HARD_STOP` with zero selection, tie,
abstention, exact-hit, validation-query, candidate-decision, and row-read work.
`NOT_RUN` is not PASS. No scalar was searched, no readout artifact was compiled,
and no placement intervention was authorized.

Source weights opened, teacher forwards, transformer calls, MoE calls, learned
router calls, dense-intelligence-matrix calls, Ollama calls, hosted-provider
calls, external/corpus population scans, #969 fixtures consumed, and generated
lexical units were all zero.

## Reproduction and run record

The binding focused command is:

```text
cargo test -p uor-r4-core --offline \
  --test recursive_geometric_attention_970 -- --nocapture
```

It executes the complete corrected probe twice, compares the reports and
canonical bytes, and pins both result identities. Binding result after
precomputing the 120 exact relative-state heatmap rows: **1 passed, 0 failed**,
115.43 seconds (test runtime; compilation excluded).

An earlier representation-only attempt serialized all 14,400 repeated pair
rows directly. It was interrupted before any result after about eight minutes
(`ABORTED_PRE_RESULT_REPRESENTATION_COST`, exit 130). The final form preserves
the exhaustive enumeration through its census κ and 120 multiplicity-bound
relative rows; no metric, class, or decision was changed to reduce cost.
