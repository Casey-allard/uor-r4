# Final Non-Circular Kernel (2026-02-18)

## Single Remaining Mathematical Target

`PrimeRiemannBridgeInghamImportedSlot.InghamImportedPayloadTerm`

Prove this term directly (non-circularly):
- if `Re(s) > 1/2` for a nontrivial zero under `VonKochPrimeErrorCriterion E`,
- then there exists `beta > 1/2` and a tail lower envelope
- `forall X, exists x >= X, |E(x)| >= x^beta`.

## Closed vs Open

- `K1` (open): zero-to-cos/sin explicit decomposition from a right-half zero.
- `K2` (closed relative to K1): phase pinning on an unbounded sequence.
- `K3` (closed relative to K1): remainder domination on that sequence.
- `K4` (closed): convert constant-factor envelope to final Ingham payload exponent form.

## New theorem closure (Lean)

File:
- `/Users/adminamn/Documents/New project/research/formal/lean/PrimeRiemannBridgeSpinningTopFrontier.lean`

Theorem:
- `ingham_payload_of_zero_to_cos_sin_phase`

This theorem proves: **K1 implies the full final payload term**, using explicit phase-locked sequences at `0` and `π/2` and asymptotic remainder control.

## Guardrail (No Circularity)

Do **not** use any theorem concluding `RHStatement` while proving K1.

## Done Condition

Once `K1` is proved non-circularly, the final payload is immediate via
`ingham_payload_of_zero_to_cos_sin_phase`, and the RH closure chain is already in place.
