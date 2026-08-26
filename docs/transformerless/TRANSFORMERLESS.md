# Transformerless: Cross-Compiling Language Models into Table-Native Artifacts

> **Migrated (2026-07-18):** this document moved from the `transformerless`
> repository into uor-r4, where the implementation now lives inside
> `uor_r4_core::transformerless` (`crates/uor-r4-core/src/transformerless`). The "parent platform" referenced
> in §5–§6 is this repository's UOR substrate (uor-addr, uor-foundation).

> **Current scope (2026-08-25, #948).** This document specifies the retained
> TLA table-cross-compiler and its historical multiplication-free runtime. It
> does not specify the active
> [geometric causal decoder](../geometric_causal_decoder_plan.md). The
> two-surface teacher boundary below remains valid for TLA compilation only;
> the active decoder deliberately studies residual/Q/K/V/attention behavior as
> a reference while replacing causal self-attention.

This document is the extrapolation companion to the implementation in this
crate and the proof/certificate in PROOF.md. It assumes the graded
κ-coordinate research program (the report and plan ship with the parent
research artifacts); here the frame is the compilation itself.

## 1. The equivalence frame

UOR's central move is equivalence-based representation: an object is not its
encoding but its equivalence class, and any member of the class that
preserves the governing invariants is as good as any other. Applied to
language models, the object is a *behavior* — a conditional next-token map —
and a trained transformer is one encoding of it: knowledge superposed across
continuous weights, retrieved by change of basis, which is why its inference
is dense multiplication. Cross-compilation is deriving, from that single
fixed source, a different member of the behavioral class whose encoding is
tables: knowledge as enumerable entries, retrieved by addressing.

Two consequences follow from taking equivalence seriously. First, the
compilation never claims identity — different function classes cannot agree
everywhere — so the artifact ships with a *measured residual*: the
certificate states agreement and bits/token against the source on a held-out
stream, with the source's own floor and ceiling bracketing it. Equivalence
class membership is a measured, bounded claim, never an asserted one.
Second, the TLA compiler's interface to the source must be architectural
invariants, not architecture. Two surfaces suffice for that compiler: an **embedding table**
(the source's representation of tokens) and a **next-token oracle** (the
source's behavior). Llama, qwen, phi, and every transformer LM of practical
interest expose both. Everything between the surfaces — attention variants,
gating, normalization choices — is the source's private encoding and never
appears in the compilation. This crate instantiates the llama-family
adapter behind an explicit `TeacherOracle` trait, and every consumer —
compiler, certifier, comparator, scenario suite — is written against the
trait, so the two-surface boundary is type-enforced rather than
disciplinary; a qwen or phi adapter implements the trait and nothing
downstream changes.

## 2. What the compiler extracts, and where each transformer role goes

A transformer performs four roles at inference. The compiler relocates each:

**Representation** → the token codebook: a frozen residual quantizer over
the source's embedding table. Each token's coordinate is a short digit
string; its integer vector is a sum of κ-pinned stage entries. The one
learned artifact in the target, derived from the source, multiplication
spent once at compile time.

**Knowledge** → the datastore: graded context classes mapping to next-token
evidence, built by running the *teacher* (the source, at compile time only)
over a corpus and recording its behavior. Knowledge that lived superposed in
FFN weights becomes enumerable, attributable, deletable entries. The
measured scaling law (parent report §3.10) is the payoff: capability grows
with entries at constant per-token compute, and the resolution knee migrates
outward with store size as log_K predicts.

**Selection** → grade-depth resolution: deepest populated class wins,
coarser classes back off underneath. What attention computed numerically
(relevance by dot product) the coordinate carries structurally (relevance by
shared prefix), the geometry purchased once at freeze time.

**Composition** → the context encoder: a dyadic-recency integer bundle of
token vectors, then a bit signature. The parent report's §3.11 measured the
decisive fact: table-native context encoders *converge* on the transformer's
own context vector as the store grows, every one of them gaining more per
store doubling than the hidden state does. Composition is the role that was
expected to resist, and at available scale it is the role that is closing.

## 3. A vector at each bit

The context signature makes the coordinate geometry literal. Each dimension
of the bundle has a compiled threshold; bit b of the signature records which
side of threshold b the context falls on. One bit is a halfspace — a
direction in context space and a decision about it. A prefix of bits is an
intersection of halfspaces: an *area*. Reading the coordinate left to right
is progressive localization: every additional bit points into a smaller
region containing the context, and two contexts are similar to degree d
exactly when the first d bits agree — they occupy the same area at that
resolution.

The certificate measures this geometry in two forms. Design A quantizes the
areas by codebook classes (the signature is assigned to the nearest class
signature by Hamming distance — xor, popcount table, add); design B uses no
classes at all: the store is keyed directly by signature prefixes, the raw
"vector at each bit" with the bits themselves as digits. B is the purist
construction — zero codebook machinery on the context side — and its
measured standing against A is part of the certificate.

Not putting the target in a box is the same discipline in the other
direction. BitNet-style binarized transformers are one member of the
matmul-free space — they remove the multiplier from the weights but keep the
monolith, so there are no entries to point at and no witness to ship. The
compilation here targets the other end: no monolith at all, knowledge as
store, arithmetic as addressing. Between the two ends lies a family of
targets (binarized encoders over stores, PQ-table hybrids, pure bit-prefix
coordinates), and the equivalence frame prices each by its measured
residual rather than privileging any encoding a priori.

## 4. The multiplication-free claim, delimited

Three phases, three rules. The **compiler** runs once, offline, may multiply
freely, and every output is a frozen blake3-κ-pinned artifact — the E8CB
contract: the substrate does not bound the encoder's error, and in exchange
everything downstream is exact. The **runtime** performs every arithmetic
operation through a kernel whose complete method set is add / shift / xor /
compare / table-read; multiplication is absent from the interface, so the
claim is by construction, and the census printed with every certificate is
the measurement. The **certifier** is instrumentation — floating point,
division, anything — and never executes at inference.

Per token, the measured runtime path is: decode-on-demand of the compressed
token rows (i8 stage books with power-of-two fixed-point scales — table
reads, shifts, adds; the expanded table never ships, see PROOF.md P5),
table reads and shift-adds for the bundle (dyadic weights are shifts, by
design), one compare per dimension for the signature, xor + popcount-table + add for class assignment, lexicographic
byte compares for the store probe (a B-tree, so even the container performs
no hashing arithmetic), and count compares for the argmax. The popcount
table is the stratum observable of the byte plane — the machinery that
failed at carrying semantics is exactly right as the metric arithmetic of
the semantic plane.

## 5. What certifiability buys

Because every prediction resolves to specific store entries with counts,
produced by schedule-independent integer arithmetic, a prediction can ship
with a witness: the context coordinate, the resolved keys, the evidence.
Any peer can recheck it. Provenance and deletion are structural — to know
why the artifact predicted a token is to read entries; to remove a
contribution is to remove its κ. A store distributed over content-addressed
infrastructure is a distributed model with per-entry attribution. None of
this is available from any weight encoding, binarized or otherwise, because
a weight is not an entry.

## 6. Open, stated as such

The measured residual at this scale is real: the certificate's numbers are
against a 15M-parameter source on its own distribution with a ~10^5-entry
store, and the parent report's scaling measurements say the store, not the
encoder, is the binding constraint at these sizes. Open beyond this crate's
measurements: convergence beyond ~10^5 entries and on distributions where
long-range structure dominates; qwen/phi adapters (mechanical, unbuilt);
codebook induction without gradient descent (a separate research program);
and store persistence as κ-keyed content in the parent platform's stores,
for which the wire form exists in the parent crate.
