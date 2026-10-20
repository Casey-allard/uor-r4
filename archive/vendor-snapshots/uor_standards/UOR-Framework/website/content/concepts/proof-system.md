# Proofs, Derivations & Traces

The {@ns proof}, {@ns derivation}, and {@ns trace} namespaces implement the
certification pathway of the [PRISM](../pipeline/) pipeline. Every algebraic identity
must be *proved*, every proof must be *derived* from axioms, and every derivation
must be *traced* for reproducibility.

## Proof Species

UOR defines four fundamental proof types, each corresponding to a different
verification method:

- {@class https://uor.foundation/proof/ComputationCertificate} -- verified by
  exhaustive computation. The identity is checked for every element at a given
  {@concept witt-levels} scale.
- {@class https://uor.foundation/proof/AxiomaticDerivation} -- derived from
  ring axioms. The identity follows logically from the algebraic laws of
  Z/(2^n)Z without needing to enumerate elements.
- {@class https://uor.foundation/proof/InductiveProof} -- proved by induction
  over Witt levels. The base case holds at some minimal level, and the
  inductive step lifts validity from level k to level k+1.
- {@class https://uor.foundation/proof/EmpiricalVerification} -- verified by
  quantum measurement. Used for identities that involve superposition
  or measurement outcomes.

## Proof Properties

Every proof individual carries key properties:

- {@prop https://uor.foundation/proof/atWittLevel} -- the Witt level at
  which the proof is valid.
- {@prop https://uor.foundation/proof/universalScope} -- whether the proof
  holds at all Witt levels (true for Universal scope).
- {@prop https://uor.foundation/proof/derivationWitness} -- a link to the
  derivation that produced this proof.

Inductive proofs additionally carry:
- `baseCase` -- proof at the minimal Witt level.
- `inductiveStep` -- proof that validity lifts from level k to k+1.
- `validForKAtLeast` -- the certified minimum Witt level.

## Derivation Chains

The {@class https://uor.foundation/derivation/Derivation} class records how one
identity follows from another. Derivation chains trace the logical dependency
graph of the {@count:identities} named identities, showing which identities are
axioms and which are derived theorems.

## Computation Traces

The {@class https://uor.foundation/trace/ComputationTrace} class records every
step of a computation, creating an auditable log. The
{@class https://uor.foundation/trace/GeodesicTrace} records the shortest-path
trace -- the most efficient computation route through the derivation space.

Traces enable independent replay: given a trace, any verifier can re-execute the
computation and confirm the result without trusting the original prover.

## Certificates

Proofs flow into the {@ns cert} namespace, which issues formal certificates:

- {@class https://uor.foundation/cert/CompletenessAuditTrail} -- certifies that
  a completeness resolution is exhaustive (no cases were missed).
- {@class https://uor.foundation/cert/GeodesicCertificate} -- certifies that a
  computation followed the geodesic (shortest) path.
- {@class https://uor.foundation/cert/GroundingCertificate} -- certifies that
  the evaluation context is fully grounded (all bindings resolved).

## Connection to the PRISM Pipeline

The proof system spans the Resolve-to-Certify boundary. Proofs and derivations
are Resolve-stage objects that produce evidence; certificates are Certify-stage
objects that attest to that evidence. This is the pathway through which the
{@count:identities} algebraic identities are formally verified and certified.

See {@concept witt-levels} for how proofs are scoped to specific ring scales,
and {@concept resolution} for how the resolution machinery produces the evidence
that proofs reference.
