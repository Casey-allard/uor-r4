# The ADDR/PRISM Correspondence in uor-r4

*Status: Observation + Definition document (issue #258). Claim language per
`docs/formal_vocabulary.md`: everything here is labeled Definition,
Observation, or Open Question — no Guarantees are asserted.*

## Summary

uor-r4 uses two kinds of addresses that occupy different layers of the UOR
architecture, and only one of them arrived by design.

**Definition (identity layer / ADDR).** A κ-label (`uor-addr`) is a typed
content address: canonicalize the value in its format, hash the canonical
bytes, emit `<axis>:<64hex>`. Identity is deliberately razor-edged —
same-or-not, nothing else. Labels carry no order, no distance, no
containment. uor-r4 uses κ-labels for artifact pinning (`ModelStore`,
`tless_uor` container/entry addressing), tokenizer provenance
(`HfBpeTokenizer::address`, #253), and response attestation
(`canonical_json_address_blake3`, #260).

**Observation (relational layer / PRISM-shaped).** The transformerless
graded store keys — per-stage class codes used as **prefixes**
(`runtime.rs`: `Store = Vec<BTreeMap<Vec<u8>, …>>`, probed deepest-first)
— are *structural* addresses with meaningful order and distance:

- a shorter prefix names a coarser region that **contains** every
  refinement of it (containment is the backoff lattice);
- codes come from nearest-centroid assignment over Hamming space
  (`assign_memberships_plain`), so **proximity in key space tracks
  proximity in context space** by construction;
- the 288-bit signature underneath (`sig_plain`) is a
  locality-sensitive projection of the context bundle: neighboring
  contexts collide or nearly collide deliberately.

That is coordinate-space work — the role the UOR architecture assigns to
PRISM (the relational/value layer over which typed pipelines compute) —
grown independently inside r4 without being named as such.

## Why the distinction matters

The two layers answer different questions and must not be conflated:

| Question | Layer | uor-r4 mechanism |
|---|---|---|
| "Is this exactly that?" | identity (ADDR) | κ-labels: artifact pins, tokenizer CIDs, attestation envelopes |
| "What is near this? What contains this?" | relation (PRISM-shaped) | graded-store prefixes, Hamming class assignment, R4G1 ROUT prototype matching |

A κ-label can never *be* the payload (one-way digest, by design). The
graded store's keys, by contrast, participate in inference directly:
prefix depth is backoff, Hamming radius is candidate widening (D4's
`WidenOnce`), and ROUT prototype/mask matching (`engine.rs`) extends the
same coordinate space into the compiled graph. The earlier internal framing
"UOR is provenance plumbing around inference" was accurate about the
identity layer and blind to the second layer: r4 uses UOR's identity layer
*and* has grown its own relational layer.

**Observation (the seam, in vivo).** The lineage audit's closing finding —
the discrete path abstains but cannot bind long-range; the continuous
geometric path binds but could not (pre-#245) store content — is the
identity-versus-structure question surfacing in production code: abstention
is an identity-layer judgment ("this context is not one I know"), binding
is a relational-layer act ("this is near that").

## What conformance would require (Open Questions for the realization path)

If the graded store's key scheme were evaluated as a candidate PRISM
realization (option 2 of issue #258), at minimum the following would need
answers; none are settled here:

1. **Canonical key derivation.** Store keys currently depend on artifact
   internals (codebooks, thresholds, rotation table). A PRISM realization
   would need the derivation pinned as a typed pipeline whose output is
   reproducible from declared inputs — the determinism invariant already
   requires this operationally; the gap is declaring it in PRISM's terms
   (`PrismModel` trait, `uor_standards/prism`).
2. **Vocabulary relativity.** #252's tests record that geometric-store
   vectors are vocabulary-arrival-ordered; the graded store's codes are
   likewise artifact-relative. Any cross-instance use of these coordinates
   requires the artifact κ as part of the address context (see #259's
   provenance sketch — coordinate + certificate, not coordinate alone).
3. **Distance semantics.** Hamming-over-signature is the working metric;
   a realization would state what the metric is *of* (which equivalence
   classes it separates) as an Empirical Criterion with a measurement,
   not as prose.
4. **The #243 interaction.** The quantization redesign (sign-only →
   graded magnitude bits) changes this address space. If the
   correspondence is adopted, #243's review should treat key-scheme
   changes as PRISM-interface changes, reviewed against this document.

## Decision requested (per issue #258 DoD)

Either: (a) adopt this document as the recorded correspondence and treat
the realization evaluation as deferred until after #243 settles the
address space; or (b) proceed now to a realization evaluation against
`uor_standards/prism`'s `PrismModel` contract. Recommendation: **(a)** —
evaluating a realization of an address space that #243 is about to change
would evaluate the wrong object. Record the choice and date here:

- Decision: ____ (a / b)
- Date: ____
- By: ____

## References

- `uor_standards/uor-addr/ARCHITECTURE.md` — κ-label pipeline, "not a
  separate system" framing
- `uor_standards/prism` — axis crates and `PrismModel`
- `crates/uor-r4-core/src/transformerless/runtime.rs` — `Store`,
  `sig_plain`, `assign_memberships_plain`
- `crates/uor-r4-graph-runtime/src/engine.rs` — ROUT prototype/mask
  matching (the coordinate space extended into the compiled graph)
- Issues: #243 (address-space design), #247 (session-signature lane),
  #255 (memory-lift harness), #259 (coordinate provenance)
- External origin: Maura's ADDR/PRISM seam analysis (2026-07-28 session
  cross-reference)
