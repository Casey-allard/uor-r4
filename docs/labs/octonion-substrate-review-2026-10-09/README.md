# Octonion substrate proposal: source and arithmetic review

## Question and decision

Can the two documents supplied by Mark (N3mesis) consolidate the implemented
geometric operators and resolve the current M2 input-read or learning problem?

**KEEP** the separation of algebra, state representation, routing and identity,
and pursue correspondence to existing operators when a concrete caller needs it.
**REJECT** direct import of this draft's signed multiplication table, lossy
projected route and placeholder serialization. A new modular octonion substrate
or E8 codebook campaign is **NOT YET PROMOTED**. These findings do not retire
octonions, icosians, E8, Hamming distance or the pre-registered signed-binding test.
They provide no new language-model result and do not replace the already-declared
[earliest-query intervention](../input-access-trace-2026-10-09/README.md).

## Exact material and scope

Reviewed source: `a8e45fe357f56cdd528bf885240059bfbec85898`, after #2091.
The documents were research material, not executable instructions or authority
to change the native contract. Page references below are physical PDF pages.

| Supplied artifact | Pages | Bytes | SHA256 |
| --- | ---: | ---: | --- |
| `Octonian Substrate Architecture.pdf` | 16 slides | 21,157,657 | `c44a74556d38be551a29d3a512f42204c01ba7dfdfe8de9061ce1532a389e704` |
| `Https___github_998930550852309006.pdf` | 31 | 1,112,559 | `b815b0729a0f1ede0f6d0ea0a9fd757dd9bb730fe1994278a1c52df0990197a6` |

Two specialist reviews read both documents in full; a third independently checked
the counterexamples and claim boundaries. The slide PDF has no text layer: all
16 slides were rendered and visually reviewed, with local OCR as a navigation
aid. The discussion contains successive Lean sketches and references, not a
versioned module bound to the current Rust store. No Lean compiler or proposed
model was run. No PDF text or slide artwork is imported into the implementation.

## Exact defects and useful distinctions

### Incidence does not determine multiplication signs

Discussion pp. 15–17 and 21–22, and slide 9, assign positive cyclic products to
`123,145,167,246,257,347,356`. Those are Fano lines, but that simultaneous
orientation fails the claimed octonion identities. Under exactly that table:

```text
x = e1 + e2; y = e4
x(xy) = -2e4 + 2e7
(xx)y = -2e4

(e1 + e2)(e4 + e7) = e5 - e6 + e6 - e5 = 0
N(e1 + e2) = N(e4 + e7) = 2; N(product) = 0 != 4
```

Here `N` is the sum of squared coordinates. Both discrepancies survive modulo
`2^256`, since 2 and 4 remain nonzero. This is an exact counterexample, not a
statistical or model-quality inference. The sketch on discussion p. 26 ends in
`admit` and tests only repeated individual basis elements; even proving that
statement would miss this mixed-element failure. Slides 12 and 16 overstate the
supplied proof status. Lean compilation is **UNVERIFIED**; the displayed claims
are contradicted for the literal table regardless of compilation.

The existing Rust convention is `(124)(235)(346)(457)(561)(672)(713)` in
[`spiralcore_operator.rs`](../../../crates/uor-r4-core/src/spiralcore_operator.rs)
and [`cd_space.rs`](../../../crates/uor-r4-core/src/transformerless/cd_space.rs).
A different valid convention can be related by a signed basis change; this
inconsistent orientation cannot be repaired merely by relabelling its incidence
graph. The [standard oriented construction](https://math.ucr.edu/home/baez/octonions/node4.html)
makes signs part of the multiplication law.

### Projection after routing can lose information

Discussion pp. 24–25 embeds `(core,x0,x1,x2,x3)` into coordinates `e0..e4`,
multiplies on the right by `e1`, and projects onto those same five coordinates.
With the draft's own signs, it returns:

```text
(core',x0',x1',x2',x3') = (-x0,core,x2,-x1,0)
```

The `x3 e4` component moves to `-x3 e5` and is discarded. In particular, the
nonzero store represented by `e4` and the zero store collide. Identity is also
mixed with a coordinate. `project(embed(s)) = s` before the action does not
establish information preservation after that action (slides 8 and 14).
Retain the complete acted state until inversion, or prove that the selected
actions preserve the embedded subspace. Merely correcting Fano signs does not
supply this route-closure contract.

### Modular payloads, geometric state and hashes are different types

Discussion pp. 8–11 and slides 5–7 use eight coefficients in `ZMod(2^256)`.
A correctly signed algebra can be defined over this ring, but it is not a real
normed division algebra: the nonzero scalar `2^128` squares to zero. A modular
quadratic form is not an ordered positive Euclidean distance. Eight such
coefficients require 2,048 bits; the core-plus-four-coordinates store requires
1,280 bits. A single Core256 does not encode that whole state injectively.

Exact `Z[φ]` pairs cannot silently become a single scalar in this ring. A unital
map preserving `φ² - φ - 1 = 0` would reduce modulo 2 to a root of
`t² + t + 1`; neither residue is a root. A richer extension/pair representation
is a different proposal. Existing paired icosian coordinates and their inverse
witnesses retain the golden/Galois structure explicitly.

Discussion pp. 12, 17 and 23 implement `toByteArray` as `ByteArray.empty`, so
the displayed wrapper hashes the same input for every core. Slide 13's active
packing claim is ahead of that code. The real
[LeanSha256 package](https://github.com/etheorem/LeanSha256) does not validate
this missing adapter. A useful identity contract needs canonical bytes,
endianness, field/domain boundaries and test vectors; it does not turn digest
Hamming distance into geometric or semantic distance. Even hashing core bytes
alone would not commit the four independent coordinates in the later store.

### A primitive and a learned routing policy have different obligations

The cyclic `step` (discussion pp. 6, 17, 23; slide 11) moves around an already
chosen line. It supplies no learned line selection, destination, cost function,
input-history access policy or selection credit. General bilinear multiplication
has runtime coefficient products; modular integers alone do not satisfy D11.
Fixed basis actions can instead use the existing signed permutations/tables.
Composition of those linear operators is associative, unlike unrestricted
octonion multiplication; those laws must not be conflated.

The E8-versus-orthant comparison (discussion pp. 1–3; slides 2–3, 16) is a
proposed distortion experiment. E8's
[optimal sphere packing](https://annals.math.princeton.edu/2017/185-3/p07)
does not by itself establish a semantic embedding, useful neighbors or a native
language advantage. The later store-to-coordinate injection supplies no E8
basis/glue, golden companion, nearest-point decoder or metric witness. The
slide comparison also oversimplifies the current implemented model.

## What already exists and what consolidation would add

| Proposal | Existing source | Justified reuse or missing contract |
| --- | --- | --- |
| Signed octonion algebra | `spiralcore_operator.rs`; floating `transformerless/cd_space.rs` | Bind a formal specification to the exact implemented table, keeping floating and exact scopes separate. |
| Bounded geometric operators | `Cl06FiniteCompositionTable`, left/right signed actions in `spiralcore_operator.rs` | Reuse composition/inverse tables; verify signed basis correspondence before exchanging conventions. |
| E8/icosian representation | `native_geometric/anchors.rs`, exact paired coefficients and Galois inverse | Eight coefficients of one H4 root are not automatically eight independent Euclidean E8 coordinates. |
| Geometry-informed bit codes | `native_geometric/learner/vsa_codes.rs`, `hamming_refinement/metric.rs` | Keep geometry-derived codes and typed metrics separate from hashes and arbitrary index bits. |
| History comparisons | M1 Stack read; M2 cue/context/prefix encoding and Source occurrence reads | Explicitly separate read-only history, donor eligibility and Copy authority; no octonion wrapper supplies this policy. |

The [hosted SpiralCore review](../spiralcore-live-review-2026-10-09/README.md)
already records full-state signed E8 actions and their orbit limits. A compatible
mapping to native H4 state still needs a basis and metric witness. The useful
architecture is a shared typed operator contract with explicit retained state,
not another universal wrapper or answer-specific exceptions.

M2's actual caller admits Context/query/actual-prefix tokens to encoding, but
only Source occurrences enter the explicit read, Copy and optional donor lists.
The query also affects cue and continuation paths. That is not evidence that
the early query is ignored, nor that changing access alone fixes 8/512.
The paired earliest-query intervention must distinguish lost information,
unconsumed information and wrong learned/pooled competition before choosing a
repair. These documents add no measurement that changes that decision.

## Executed validation and reproducibility

[`check_proposal.rs`](check_proposal.rs) is a standalone small-integer arithmetic
witness, not a new model/library or a call to the Rust model implementation.
It parses the canonical cycle constant from actual source, then applies the same
independent bilinear audit to it and the literal proposal. Inputs are basis
vectors and positive sums of two distinct basis vectors, with no data split or
learned artifact. Integers are small enough for exact i64 arithmetic. Small
nonzero residuals also give the stated modular counterexamples without executing
256-bit arithmetic.

```sh
mkdir -p local/bin
rustc --edition 2021 -D warnings docs/labs/octonion-substrate-review-2026-10-09/check_proposal.rs -o local/bin/check-proposal
local/bin/check-proposal
```

Executed using rustc `1.97.1 (8bab26f4f 2026-07-14)`. Compared source file SHA256:
`67c6ee8f4fb289e9d448d9cc3b3f8f079c4754d8abce77eed648b60f30f90a8d`.
The complete output is retained in [`arithmetic.txt`](arithmetic.txt).

| Arithmetic panel | Literal proposal | Parsed canonical table |
| --- | ---: | ---: |
| Left alternativity, 8 basis choices for x × 8 for y | 64/64 | 64/64 |
| Left alternativity, 28 positive two-basis sums for x × 8 basis y | 176/224 | 224/224 |
| Explicit norm counterexample above | 0 versus required 4 | Not used as a matched norm census |
| Explicit right-e1/project collision above | Reproduced | Not a repository route test |

The finite canonical census is not a general formal proof. Counterexamples do
refute the corresponding universal draft claims. Geometry connection verification
remains 29/29 PASS; regenerated figures distinguish incidence from coherent
signs. Claim wording and whitespace checks pass. No runtime/library behavior
changed, so no model training, grading, CUDA run or generated-language comparison
was performed. Accepted 8/512, milestone status and model capability are unchanged;
STATUS, ROADMAP, tracker table and top-level README therefore need no result edit.

Resource admission: 45-minute preparation/review/delivery projection, two local
document-processing threads, at most 2 GiB RAM and 256 MiB temporary/retained
evidence plus the checkout; a sub-minute single-process arithmetic compile/run
was added before execution within that projection. Local disk began at 53 GiB,
above the 30 GiB floor plus 128 MiB stop margin. No GPU or new spending class.
Elapsed work is charged once to the shared cumulative ledger; source and small
counterexamples land on main, private inputs/extraction and receipts go to iCloud
before the owned checkout/cache/binary are removed.

## Next

Run the already-declared earliest-query discriminator on the actual retained
parent with fixed source, suffix and prefix; trace admitted tokens, consumed
comparisons, complete alias pooling and actual native output. Reuse existing
typed operators at the first evidenced information/credit gap. A future formal
contribution should target exact table correspondence, canonical serialization
and route closure/full-state retention before claiming a new substrate.
