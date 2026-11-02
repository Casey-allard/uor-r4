# REPORT: cpu-training-pilot, deepseek-7

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `e1de1ff2c`. Order: [ORDER.md](ORDER.md).

## What did not work, first

- **This is a gate, not a replacement exporter.** The wrapper audits an already
  exported artifact and promotes an exact copy only after the report passes.
  Candidate export scripts must call it before publishing.
- **The public artifact still has no Hamming or memory records.** The successful
  public gate proves provenance, deterministic reporting, complete trunk/writer
  parity and atomic promotion, not trained full-path coverage.
- **The gate intentionally uses a CPU build dependency.** It does not make the
  integer serving crate depend on `uor-r4-cpu-train`.

## What changed

- `cpu-served-forward-audit` now accepts a canonical `UORT` file with
  `--token-offset` and `--token-count`. It validates the header, span,
  vocabulary and source hash, then records a path-independent token-source
  manifest in JSON.
- The command can require `--expected-artifact-sha256`.
- `scripts/audit_served_forward_gate.py` independently validates deterministic
  mode, schema, artifact identity and bytes, token provenance, expected
  position/coordinate/token/kept-cell counts, all mismatch counters and equal
  decision hashes. It parses the UORT header independently, recomputes the
  selected-token-sequence hash from the exact token IDs, and checks source
  total/vocabulary. It can atomically copy an artifact to a new destination
  only after validation. Promotion is bound to the exact artifact hash audited;
  all input/output aliases are rejected, and hard-link publication fails if
  either destination is created concurrently.
- Python tests cover report tampering, count mismatches, provenance failures,
  selected-token drift, audited-hash drift, path aliasing, existing-destination
  refusal and exact-byte promotion.

## Measurements

Public seed 1 ran through the complete gate using the first 64 tokens of the
public DEV `UORT` file:

- artifact SHA-256: `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd`;
- token source SHA-256: `add1f3954fc67e12822bdac395d7167bf6d6a0f074856efe2beec2c4ac81e442`;
- span: offset 0, count 64 of 1,925 tokens, vocabulary 4,096;
- parity: zero mismatches over 16,384 Q12 coordinates, 64 tokens and 512 kept
  cells;
- CPU and serving decision SHA-256:
  `8419b6ac5e74196205cb8a37389f7a4505804ade3503fe3dd60bc558839ca8c5`;
- promoted copy: byte-identical to the source artifact.

The report SHA-256 is
`fa8cbe7f8d3554bc91b36720af02c60f5c41b8edb21a53263481d052829ac2f5`.
The audit binary SHA-256 is
`37303166c6d87dbf42fb1c1d4fa203e7f1fda672388b1e1966cb21d00fbd4fda`.

## Checks run at the head

- Rust served-forward tests: 9 passed on a fresh release target, including UORT
  parsing, slice bounds, artifact identity and existing audit controls.
- Python gate tests: 9 passed.
- Complete real-artifact gate and byte-identical promotion: passed.
- Evidence SHA-256 inventory: passed.
- Formatting and Clippy checks are recorded by the exact-head review.

## Skills used

No repository skill was invoked. The unit reused the public `UORT` format and
the merged independent audit.

## Still open

- Wire candidate export scripts to this gate.
- Add a compact trained or artifact-level full-path fixture containing Hamming,
  memory and point-key records.
- Close training-through-serving gradients and then run the new bounded
  comparison.

## Plain summary

Candidate FBS artifacts can now be bound to a specific token-file slice and
audited before release. Any mismatch in provenance, artifact identity, decision
counts or Q12/token/kept-cell parity stops the gate, and the destination copy
is created only after a complete pass.
