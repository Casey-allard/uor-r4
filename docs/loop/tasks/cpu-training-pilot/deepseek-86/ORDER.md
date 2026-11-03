# ORDER: cpu-training-pilot, deepseek-86 (fail-closed served-forward export gate)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `08213d45b`.

## Goal

Make the CPU corpus driver's final UORLUT promotion use the exact
served-forward audit as a fail-closed export gate, and retain the deterministic
audit receipt beside the promoted artifact.

## Cancel probe (run first)

At the base, `write_verified_artifact` reparses `final.uorlut` and checks byte
identity but does not call `audit_served_forward`. Repository search finds no
export/release call site for `scripts/audit_served_forward_gate.py`. The unit
stands.

## Files in scope

- `crates/uor-r4-cpu-train/src/bin/cpu-corpus-driver.rs`
- `docs/loop/tasks/cpu-training-pilot/deepseek-86/**`
- `docs/research/notebook/cpu-training.md`
- `docs/research/cpu-training-program.md`
- `docs/roadmap/items/cpu-training-pilot.md`
- `docs/harness/claims.json`

## Done when

1. Final artifact promotion first runs the full served-forward audit over a
   deterministic UORT token slice bound to the corpus input.
2. The gate requires the audit artifact hash, expected position/Q12/token/
   kept-cell counts, zero mismatches, equal decision hashes and no instrument
   fault.
3. A deterministic `final.audit.json` receipt is persisted and its hash and
   parity counts are recorded in the run report.
4. A failed audit leaves neither `final.uorlut` nor `final.audit.json`.
5. Tampered audit counters and invalid artifacts are refused by focused tests.
6. CPU all-target tests, strict Clippy, formatting and evidence checks pass at
   the exact head.

## Stop and ask if

The public trained artifact cannot pass the supported served-forward audit, or
the change would require modifying another lab's active files or weakening an
existing exactness check.
