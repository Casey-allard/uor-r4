# REPORT: fail-closed served-forward export gate, deepseek-86

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Order: [ORDER.md](ORDER.md). Base: `08213d45b`.

## What did not work, first

- Before this unit, `cpu-corpus-driver` promoted `final.uorlut` after parsing
  and byte-identical reload only. It did not execute the complete
  served-forward audit.
- The public zero-step smoke proves the gate and receipt path, but it promotes
  an unchanged raw artifact. It is not trained-artifact improvement.
- The public artifact contains no Hamming, turn-memory or point-key records.
  Full-path trained-artifact coverage remains open.
- The existing independent Python gate validates an audit receipt, but it was
  not wired into this driver or the Studio release path.

## What changed

`cpu-corpus-driver` now binds the first 64 corpus tokens, or the complete
context if shorter, to a canonical UORT token-source manifest. Final artifact
promotion now:

1. runs `audit_served_forward` with the expected artifact hash;
2. validates the audited artifact identity, token provenance, position/Q12/
   token/kept-cell counts, zero mismatch counters, equal decision hashes and
   absent instrument fault;
3. serializes the deterministic audit as `final.audit.json`;
4. writes a temporary artifact and receipt, then publishes the receipt before
   the artifact; and
5. reloads both outputs and removes them together if either differs.

The run report records the receipt path/hash, audited positions and decision
workload. Existing final artifact or audit receipts are refused rather than
overwritten.

## Measurements

The public seed-1 zero-step smoke used the unchanged artifact:

- artifact SHA-256:
  `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd`;
- UORT source SHA-256:
  `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b`;
- 64 audited positions, 16,384 Q12 coordinates, 64 token decisions and 512
  kept cells;
- zero query, token and kept-cell mismatches;
- CPU and serving decision SHA-256:
  `66b43e690f5a07bdef06e6f301f4b8be6b88b66f78c366b1a39266a2021b6e82`;
- audit receipt SHA-256:
  `0175e1c6c6bd7bc3100ff8403a8b048edaf9de4edc89873f90f26acc2070973e`;
- 64-row panel gate: accepted, unchanged from source.

The smoke took 29.695 seconds wall time and used 54,068 KiB peak RSS. The
driver binary SHA-256 was
`f678c238b7f4039596815603237a8e6f04de0d7dbee2bf9104e14e7892cba426`.
The full evidence is in
[public-seed-1-zero-step-smoke.json](evidence/public-seed-1-zero-step-smoke.json)
and [public-seed-1-final.audit.json](evidence/public-seed-1-final.audit.json).

## Checks run at the head

- CPU all-target suite: 128 library tests passed, 4 ignored; 6 driver tests
  and all auxiliary binaries passed.
- Strict package Clippy with `--all-targets --no-deps -- -D warnings`: passed.
- Package formatting and `git diff --check`: passed.
- Tampered parity counters and an artifact/token vocabulary mismatch are
  refused without publishing either output.
- Real public artifact gate: passed with exact CPU/serving parity.
- Evidence `SHA256SUMS`: passed.

## Skills used

No available Codex skill applied. The existing audit, CPU forward, UORT reader
and corpus driver were inspected directly.

## Still open

- Studio and other release scripts still need a universal audit-receipt check
  before copying UORLUT artifacts.
- A genuinely trained Hamming, turn-memory and point-key artifact is still
  unavailable on this host.
- The materially new discrete counterfactual CPU comparison remains the next
  training unit; this gate is its prerequisite.

## Plain summary

The training driver can now publish an integer model only after independently
executing the same decisions through the CPU model and the serving session.
Every Q12 coordinate, selected token and kept cell must match, and the complete
audit is saved beside the model. The gate was exercised on a public artifact.
It does not yet prove a trained full-path model because the public artifact
contains only the trunk.
