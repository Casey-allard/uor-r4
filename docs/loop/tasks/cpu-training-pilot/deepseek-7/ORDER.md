# ORDER: gate exported FBS artifacts on exact served-forward parity, deepseek-7

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `e1de1ff2c`.

## Goal

Turn `cpu-served-forward-audit` into a reusable candidate-artifact gate. The gate
must bind a real `UORT` token slice, run the deterministic audit, reject any
parity or provenance mismatch, and promote an artifact to its final path only
after the complete report passes.

The production integer crate must not depend on the development CPU crate. A
separate host wrapper invokes the audited binary and validates its JSON.

## Cancel probe

Check whether any current export command already binds a UORT slice, runs the
complete CPU/serving audit and atomically promotes the artifact only after all
Q12/token/kept-cell decisions pass. If one does, stop. Otherwise extend the
existing audit rather than adding another parity implementation.

## Scope

- `crates/uor-r4-cpu-train/**`
- `scripts/audit_served_forward_gate.py`
- `scripts/tests/test_audit_served_forward_gate.py`
- `docs/loop/tasks/cpu-training-pilot/deepseek-7/**`
- `docs/research/cpu-training-program.md`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`

No float/integer kernel change is expected.

## Done when

- `cpu-served-forward-audit` accepts a canonical `UORT` file plus token offset
  and count, validates its header and requested span, and records a
  path-independent source SHA-256, offset and count in the report.
- The binary can require an expected artifact SHA-256 and exits nonzero on a
  mismatch.
- A host gate validates schema, deterministic mode, provenance, artifact
  identity, all parity counts and decision hashes before optionally atomically
  promoting an artifact. It never overwrites an existing destination.
- Tests cover malformed UORT, out-of-range slices, expected-hash mismatch,
  report tampering and promotion refusal before success.
- One public artifact slice runs through the complete gate; the limitation that
  public artifacts lack Hamming/memory remains explicit.
- Focused tests, formatting, CPU Clippy and loop checks pass.
