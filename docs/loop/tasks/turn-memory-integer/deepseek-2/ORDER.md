# ORDER: turn-memory-integer, deepseek-2 (fail closed on served-export contract mismatches)

Lab: deepseek. Item: [turn-memory-integer](../../../../roadmap/items/turn-memory-integer.md) (#3392). Base: `66c50fa19a3acc783819c198c598f24a2a55fe67`.

## Goal

As an export prerequisite for serving the trained turn memory from the
integer artifact, make `fbs-export --run-config` bind the raw/snapped contract
recorded by the run. A checkpoint trained through the unsnapped served bridge
must not be silently exported with snapped gates and transition.

This is a host-only instrument repair. It does not change a trained model,
served arithmetic, a capability bar or Claude's open branch.

## Cancel probe (run first)

At the base, `fbs-export --run-config` validates the run's readout and
retention fields but does not read `train_as_served`. The trainer writes that
field from `--train-as-served 1`, whose current contract is `SERVED_SNAPS`.
The exporter still starts from `FbsExportSnaps::SNAPPED`; therefore a valid
run config can produce a snapped artifact for an unsnapped-trained
checkpoint. The task stands.

## Files in scope

- `crates/uor-r4-integer/src/bin/fbs-export.rs`
- `docs/loop/tasks/turn-memory-integer/deepseek-2/**`
- `docs/research/notebook/m4-serving.md`
- `docs/harness/claims.json`

## Done when

1. A `train_as_served: true` config requires both `--raw-gates` and
   `--raw-transition`, and every incomplete combination is refused.
2. An absent or false flag preserves existing exporter behaviour; a
   non-boolean value is refused.
3. The check runs before artifact construction and cannot create an output on
   refusal.
4. Focused tests and the relevant integer crate checks pass.
5. The notebook, evidence and REPORT state the guard and its limits.
6. Exact-head review, PR checks, merge and fresh-main verification pass.

## Stop if

- The run configuration cannot distinguish the served contract.
- The change would alter the bytes of an otherwise unchanged valid export.
- The implementation requires a change to Claude's open PR or its files.
- The path would weaken D11 integer serving or silently select an alternate
  contract.
