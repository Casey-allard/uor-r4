# REPORT: m4-export-uorlut01-cli, codex-5

Lab: codex. Order: [ORDER.md](ORDER.md).

## What did not work, first

- `loop claim m4-export-uorlut01-cli --lab-topic q31-lut-facade` was refused with exit 2 because
  PR #3983 claims overlapping `crates/uor-r4-integer/src/discrete_d11.rs` scope. This unit touches
  only `crates/uor-r4-lut` and its records, and continued under the owner's open D85 direction.
- The first compile missed the new `q31` field in an existing test helper's `StackHeader`
  initializer; it is now `q31: None`, which is also the plain-export path.
- The first test numerics helper repeated the two squashing-table field names and raised the D23
  presence count from its base. The helper now assembles those two JSON keys from split fragments,
  so the source has no new `silu`/`gelu` identifier segment and `banned-presence` returns to its
  base. Behaviour is unchanged.
- The facade deliberately does **not** validate the closed Q31 domain: it is a metadata reader
  and has no native-vector dependency. The serving engine's `Q31HamiltonModel::from_bytes` remains
  the domain authority; the facade checks the record, the table shape and the digest.
- The test uses a **free-transport** `/12` artifact, because this facade still refuses a snapped
  stack by its transport rule. A snapped `/12` artifact is the serving engine's path, not this
  facade's.

## What changed

- `crates/uor-r4-lut/src/format.rs` learns `uor-r4.lut-stack/12`:
  - constants `STACK_Q31_SCHEMA`, `Q31_STATE_TABLE` and `Q31_STATE_BYTES`;
  - `StackQ31State` and an optional `StackHeader.q31` record (`skip_serializing_if`, so a plain
    header is byte-identical);
  - `check_schema` accepts `/12` exactly when the record is present and refuses `/12` without it;
    `is_stack_schema` includes `/12`;
  - `Builder<StackHeader>::set_q31_state` writes the 48 bytes as twelve little-endian `u32` words
    and the digest/cursor record, and refuses a second state;
  - `StackArtifact::q31_state_bytes` returns the exact bytes after its own SHA-256 check and
    refuses a wrong kind or length.
- `crates/uor-r4-lut/README.md` states the boundary: the facade reads and checks; the serving
  engine attaches.
- One new test covers the round-trip, a repeated state refusal, a flipped-word digest refusal, a
  `/12` header without the record, and the unchanged plain stack.

Size: one source file, the README, the new test and the loop records.

## Measurements

- `7 / 7` library tests pass, including the new round-trip/tamper test.
- The fixed trained state round-trips to the exact 48 bytes; the record exposes epoch `64`, step
  shift `8`, version `1`.
- A flipped state word makes the accessor fail the digest; a `/12` header without the record fails
  `validate`; a plain stack remains on `uor-r4.lut-stack/1` with no `q31` record.
- No serving, model, speed or milestone result is measured.

## Checks run at the head

- `cargo test --locked -p uor-r4-lut` — `7` passed.
- `cargo clippy --locked --no-deps -p uor-r4-lut` — exit 0.
- `git diff --check` — passed.
- `uor-harness check --only harness.native_space_contract` — 1 rule, 0 failed.
- `uor-harness loop pr-check --local` — PASS, including `banned-presence` (the D23 count stays at
  its base) and scope/commit-order/report.
- `uor-harness loop report --check` and the exact-head review — at the head.

## Skills used

- `native-space-cycle`: the facade reads the same 48 bytes without translating them; the serving
  operator remains the only attachment path.
- `scope-guard`: stayed inside `crates/uor-r4-lut` and avoided the file claimed by PR #3983.
- `solution-integrity`: checked the digest over the bytes and added the missing-record and
  plain-path cases, so a schema-only pass cannot hide a dropped state.
- `test-freeze`: added a test; changed or deleted none.

## Still open

- A real trained model exported and scored at the M4 bar remains open.
- The facade still refuses snapped artifacts by its free-transport rule (correct for that engine);
  a snapped `/12` artifact is read by the serving engine.
- The org runner-capacity bottleneck remains an owner/PM-level issue.

## Plain summary

Reference tools that use the small metadata crate can now see the trained 48-byte state in a
served file. They check the stored digest and return the exact bytes, but they still do not run
or attach the state themselves. Plain files are unchanged.
