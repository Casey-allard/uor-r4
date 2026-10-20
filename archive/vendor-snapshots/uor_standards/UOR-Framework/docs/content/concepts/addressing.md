# Addressing

## Definition

**Content addressing** in UOR maps ring elements to Braille-encoded strings
via a bijective encoding. Each {@class https://uor.foundation/u/Element}
represents a content-addressable identifier where each byte
encodes a chunk of the element value.

## The Addressing Bijection

Two identities formalize the round-trip property:

- **AD_1**: addresses(glyph(d)) = d — addressing a glyph recovers the datum.
- **AD_2**: glyph(ι(addresses(a))) = ι_addr(a) — embeddings commute with
  addressing.

These are the foundation of content-addressable computation: every datum has
a unique address, and every address resolves to a unique datum.

## Boolean Homomorphism

A key structural property is that **Boolean operations lift to address space**
while ring-arithmetic operations do not:

- **AA_2**: braille(a ⊕ b) = braille(a) ⊕ braille(b) — XOR lifts
- **AA_3**: glyph(bnot(x)) = complement(glyph(x)) — complement lifts
- **AA_4**: glyph(add(x, y)) ≠ f(glyph(x), glyph(y)) — addition does NOT lift
- **AA_5**: Liftable operations are exactly {xor, and, or, bnot}

This means carry-free operations can be performed directly on addresses, while
carry-dependent operations (add, sub, mul, neg, succ, pred) require decoding.

## Address Metric

Identity AM_2 shows that the address metric d_addr equals the Hamming metric
on ring elements: d_addr(glyph(x), glyph(y)) = d_H(x, y). However, AM_3
notes that d_addr does NOT preserve the ring metric d_R in general — the
incompatibility metric d_Δ measures this gap (AM_4).

## Embedding Coherence

The property {@prop https://uor.foundation/morphism/addressCoherence}
certifies that an embedding's addressing diagram commutes: the composition
glyph ∘ ι ∘ addresses is well-defined and injective.

## Glyph Encoding Example

Consider the value 42 in R_8 (= Z/256Z). Its binary representation is
`00101010`. The 6-bit chunking scheme splits this into two chunks:

| Chunk | Bits | Decimal | Encoding |
|-------|------|---------|-------|
| 1 | `001010` | 10 | U+2819 (⠙) |
| 2 | `10xxxx` | 32 (padded) | U+2840 (⡀) |

Each byte encodes a chunk of the element value, producing
the {@class https://uor.foundation/u/Element} `⠙⡀`.
The addressing bijection (AD_1) guarantees that decoding this address recovers
the original value 42.

## Connection to Canonical Forms

The {@class https://uor.foundation/resolver/CanonicalFormResolver} produces
the unique canonical representation of a ring element. Once the canonical form
is computed, the addressing bijection maps it to a content-addressable
{@class https://uor.foundation/u/Element}. This two-step process — resolve
then address — ensures that semantically equivalent values receive identical
addresses. See [Canonical Form](canonical-form.html) for the resolution step.
