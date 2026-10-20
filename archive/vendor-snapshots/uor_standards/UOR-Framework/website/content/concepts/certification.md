# Certification and Verification

The final stage of the PRISM pipeline is **Certify**: every resolution result
must be attested with a machine-verifiable certificate before it leaves the
pipeline. The {@ns cert} namespace encodes this attestation layer.

## Why Certificates?

A resolution that computes a correct answer but cannot *prove* it is incomplete.
The UOR framework treats proof and computation as inseparable — the
{@class https://uor.foundation/cert/Certificate} class is the root of a
hierarchy of typed attestations, each tied to a specific algebraic property.

## Certificate Hierarchy

Certificates are not generic. Each subclass targets a distinct invariant:

- {@class https://uor.foundation/cert/TransformCertificate} — attests that a
  morphism preserves the claimed structure.
- {@class https://uor.foundation/cert/IsometryCertificate} — attests metric
  preservation under a transform.
- {@class https://uor.foundation/cert/CompletenessCertificate} — attests that a
  type satisfies the completeness criterion (identity IT\_7d).
- {@class https://uor.foundation/cert/GroundingCertificate} — attests that a
  context has reached full grounding (all sites pinned).
- {@class https://uor.foundation/cert/GeodesicCertificate} — attests that a
  geodesic trace satisfies both GD\_1 conditions.
- {@class https://uor.foundation/cert/MeasurementCertificate} — attests that a
  measurement event respected the von Neumann–Landauer bridge.

## Audit Trails

For complex verifications the framework records step-by-step provenance.
The {@class https://uor.foundation/cert/CompletenessAuditTrail} collects an
ordered sequence of completeness witnesses, and the
{@class https://uor.foundation/cert/ChainAuditTrail} records per-step evidence
for lift-chain certificates that span multiple Witt levels.

## Connecting to Proofs

Certificates are the *output* of the {@concept proof-system}. A
{@class https://uor.foundation/proof/ComputationCertificate} verifies an
identity at a specific Witt level, while an
{@class https://uor.foundation/proof/AxiomaticDerivation} establishes universal
scope. The cert namespace consumes these proofs and packages them as
self-contained attestations that downstream systems can verify independently.
