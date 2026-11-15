# ORDER: m4-export-uorlut01-cli, codex-5

Lab: codex. Item: [m4-export-uorlut01-cli](../../../../roadmap/items/m4-export-uorlut01-cli.md).
Parent: [codex-4](../codex-4/ORDER.md). Reviewed head: `f88230fce72cc23724ddace01af3f5b0132fbe11`.

## Why this unit exists

The serving engine writes and reads `uor-r4.lut-stack/12` (the exact 48-byte Q31 state), but the
retained `uor-r4-lut` metadata facade refuses the schema. Any reference tool that inspects a
served artifact therefore cannot see the state record. This unit teaches the facade the `/12`
record and gives it a verified state accessor, without adding an arithmetic path.

## Claim record

`loop claim m4-export-uorlut01-cli --lab-topic q31-lut-facade` was refused with exit 2 because
PR #3983 claims overlapping `crates/uor-r4-integer/src/discrete_d11.rs` scope. This unit touches
only the `uor-r4-lut` crate and its records, and continues under the owner's open D85 direction.

## Native state schema

Unchanged: table `q31.state` holds the exact 48 bytes as twelve little-endian `u32` words; the
header record binds its SHA-256, epoch, step shift and version. The facade is a metadata reader;
it validates the record structure and the digest, and leaves the closed Q31 domain to the serving
engine's `Q31HamiltonModel::from_bytes`.

## Shared operators and implementations

No operator is added. The facade does not serve; the serving engine (`uor-r4-integer`) remains
the only reader that attaches the state. The facade's accessor returns the exact bytes after its
own digest check.

## Transition replay

A round-trip fixture builds a free-transport `/12` stack with the fixed trained state, parses it
with the facade, and requires the exact 48 bytes back. A flipped table byte must fail the digest;
a `/12` header without the record must be refused by schema. A plain stack is byte-identical to
before.

## Files in scope

- `crates/uor-r4-lut/src/format.rs`
- `crates/uor-r4-lut/README.md`
- `docs/loop/tasks/m4-export-uorlut01-cli/codex-5/**`

Out of scope: any serving or arithmetic change, `crates/uor-r4-integer/**`, any CUDA, any
training run.

## Size

About 150 to 250 lines of code, tests and records.

## Done when

1. The facade reads a `/12` header, validates the record and table, and returns the exact bytes
   after a digest check.
2. A tampered state table and a `/12` header without the record are refused.
3. A plain stack artifact is byte-identical to before.
4. `cargo test -p uor-r4-lut`, formatting, touched-file Clippy, `git diff --check`,
   `uor-harness check --only harness.native_space_contract`, the REPORT check and the exact-head
   review pass.

## Cancel

Stop if supporting `/12` requires an arithmetic path, a serving dependency, or a change to the
serving engine. Do not validate the closed Q31 domain here; that stays with the serving operator.

STOP and report when done.
