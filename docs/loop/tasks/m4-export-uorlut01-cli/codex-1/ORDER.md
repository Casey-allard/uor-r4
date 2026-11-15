# ORDER: m4-export-uorlut01-cli, codex-1

Lab: codex. Item: [m4-export-uorlut01-cli](../../../../roadmap/items/m4-export-uorlut01-cli.md).
Reviewed head: `2cac733bd93dfe861f53c36befa99c972ffd9ba6`.

## Why this unit exists

PR #3982 put the exact 48-byte native Q31 state on the real `IntegerStackSession`
recurrence, but an artifact does not carry it: every served Q31 model needs a runtime
side-load. D85 requires the artifact to serialize that state losslessly instead of a
separate image, and `UORLUT01` is the served container. This unit closes that gap for one
finite artifact path.

## Claim record

`loop claim m4-export-uorlut01-cli --lab-topic q31-artifact-state` was refused with exit 2:

```text
scope crates/uor-r4-integer/** overlaps crates/uor-r4-integer/src/discrete_d11.rs of item
one-model-end-to-end (PR #3985 (docs/loop/tasks/one-model-end-to-end/claude-385/ORDER.md))
```

This unit continues under the owner's open D85 direction, changes no file named by PR
#3985, and does not touch `crates/uor-r4-integer/src/discrete_d11.rs`.

## Native state schema

- The state is exactly 48 bytes: four little-endian Q31 gate words, `epoch: u64`,
  `step_shift: u32`, version `1`, three zero bytes.
- The artifact stores those 48 bytes as twelve little-endian `u32` words in the table
  `q31.state`, so the section round-trips bit for bit.
- The header records `q31 = { state: "q31.state", sha256, epoch, step_shift, version }`
  under a new schema `uor-r4.lut-stack/12`; the record is metadata, and the 48 bytes are
  authoritative.
- No field is converted, rescaled or re-derived. A malformed record, a checksum mismatch,
  a wrong length or an out-of-domain gate is refused.

## Shared operators and implementations

- Operator: `uor_r4_native_vector::Q31HamiltonModel::from_bytes` /
  `q31_hamilton_rotate`, and the serving adapter `Q31ServingSession::transition`.
- The artifact is a container, not a second arithmetic path: loading calls the same
  `from_bytes` and attaches the same `Q31ServingSession` that PR #3982 serves.
- CUDA training and CPU oracle implementations are unchanged.

## Transition replay

One artifact is exported with the fixed trained state
`8c60b177…01000000` (gate `[2008113292, 336469910, 0, 0]`, epoch 64). Loading it must
attach the state by itself; the served tokens, raw carries, transports and post-drive words
must equal a model where the same 48 bytes were attached in memory. The comparison is on
raw words and token ids, not rounded values. An artifact without the section must stay
byte-identical and follow the existing path.

## Serialization

`UORLUT01` gains one optional section behind schema `uor-r4.lut-stack/12`. Plain exports
write no `q31` key and no section, so their bytes do not change. An engine that does not
serve the section refuses the artifact by its schema rather than dropping the state.

## Files in scope

- `crates/uor-r4-integer/src/stack/format.rs`
- `crates/uor-r4-integer/src/stack/session.rs`
- `crates/uor-r4-integer/src/stack/mod.rs`
- `crates/uor-r4-integer/src/stack/q31_serving.rs`
- `docs/model/serving-and-export.md`
- `docs/research/notebook/m4-serving.md`
- `docs/loop/tasks/m4-export-uorlut01-cli/codex-1/**`

Out of scope: `crates/uor-r4-integer/src/discrete_d11.rs`, any CUDA kernel change, any
training run, the `uor-r4-lut` facade (it keeps refusing unknown stack schemas rather than
silently dropping a state), and any other lab's files.

## Size

About 250 to 450 changed Rust and test lines, plus the loop and notebook records.

## Done when

1. The exporter writes the exact 48-byte state under `uor-r4.lut-stack/12` with a SHA-256
   over those bytes, and a plain export is byte-identical to today's.
2. The parser validates the record, section, checksum and Q31 domain, exposes the exact
   bytes, and refuses a malformed, mismatched or wrong-schema record.
3. `IntegerStackModel::parse` attaches the state from the artifact; no side-load is needed.
4. A model without the section follows the existing path unchanged.
5. The artifact-attached model and the in-memory-attached model produce bit-identical
   tokens and raw carries on the fixed prompt.
6. Focused Q31 and format tests, formatting, touched-file Clippy, `git diff --check`,
   `uor-harness check --only harness.native_space_contract`, and the REPORT checks pass.

## Cancel

Stop if the section cannot be added without changing the bytes of a plain artifact, if the
stored words require conversion to load, or if the parser cannot validate the closed Q31
domain through the existing operator. Do not add a second arithmetic path or a float
fallback.

STOP and report when done. Do not start a training run or the next unit.
