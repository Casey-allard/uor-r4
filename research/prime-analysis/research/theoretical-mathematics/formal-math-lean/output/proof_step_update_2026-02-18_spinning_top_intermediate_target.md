# Proof Step Update: Spinning-Top Intermediate Target T (2026-02-18)

## Added Formal Intermediate Target

New Lean module:
- `research/formal/lean/PrimeRiemannBridgeSpinningTopFrontier.lean`

Core additions:
- `SpinningTopSignedPayloadTerm`
- `ingham_payload_of_spinning_top_signed_payload`
- `rh_from_spinning_top_signed_payload`
- `SpinningTopSignedPayloadProvider`
- `rh_from_spinning_top_signed_provider`
- `rh_from_spinning_top_signed_provider_via_w2b`

Meaning:
- We now have an explicit intermediate theorem target `T` in signed oscillation form.
- In-repo proof shows `T` implies the current Ingham payload, hence implies `RHStatement`.
- This creates a concrete mathematical frontier to attack without changing core proof wiring.

## Build Verification

- `lake build PrimeRiemannBridgeSpinningTopFrontier` passed.
- `lake build` passed.
- Formal audit still reports `axiom_count = 0`.

## New Data Probe for T

New program:
- `research/spinning_top_signed_transfer_probe.py`

Key outputs:
- `research/output/spinning_top_signed_transfer_probe_2026-02-18.json`
- `research/output/spinning_top_signed_transfer_probe_2026-02-18.md`

Current run metrics (Odlyzko 100k source, first 256 zeros, x in [1e4, 1e8]):
- `effective_exponent_abs_envelope = -0.012273267693910215`
- `effective_superhalf_margin = -0.5122732676939102`
- `interpretation = no_superhalf_growth_detected`

Interpretation:
- On this finite range/model, we do not observe super-half growth behavior consistent with the signed lower-envelope target.
- This does not disprove RH; it confirms the remaining gap is genuinely at the deep analytic transfer layer.

## Frontier Status

- Axiom-free formal skeleton remains intact.
- Remaining proof item remains one true mathematical frontier.
- The project now has a precise intermediate theorem target `T` and a fast cached probe to track evidence against that target shape.
