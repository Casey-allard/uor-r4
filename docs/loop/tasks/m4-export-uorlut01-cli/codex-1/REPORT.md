# REPORT: m4-export-uorlut01-cli, codex-1

Lab: codex. Order: [ORDER.md](ORDER.md).

## What did not work, first

- `loop claim m4-export-uorlut01-cli --lab-topic q31-artifact-state` was refused with exit 2
  because PR #3985 claims overlapping `crates/uor-r4-integer/src/discrete_d11.rs` scope. The work
  continued under the owner's open D85 direction, and that file was not touched.
- The first compile failed with `E0382` because `artifact.sha256` was moved into the model before
  `artifact.q31_state_bytes()` borrowed the container. The state is now read before the model is
  constructed, and the attach follows it.
- `rustfmt --check` on the touched integer files reports pre-existing style differences across the
  crate (long lines and older blocks). The project's formatting gates cover only the claim-wording
  and harness crates; this unit did not reformat unrelated integer code, and the new code follows
  the file's existing style.
- The `uor-r4-lut` metadata facade was not taught `/12`. It refuses an unknown stack schema rather
  than silently dropping the Q31 record, which is the safe behaviour and keeps this unit's surface
  in the serving parser and exporter.

## What changed

- `UORLUT01` gains the optional `uor-r4.lut-stack/12` schema. A stack that carries it has the
  table `q31.state` (twelve little-endian `u32` words, the exact 48 bytes) and the header record
  `q31 = { state, sha256, epoch, step_shift, version }`.
- `Container::parse` validates the record name, the table kind and length, the SHA-256 over the
  reconstructed bytes, the closed Q31 domain through the shared `Q31HamiltonModel::from_bytes`,
  and the epoch, step shift and version in the record. A malformed, tampered or wrong-schema
  record is refused. A `/12` artifact also refuses any other optional mechanism record
  (address, point, memory, hamming or memory chain), so two schemas cannot hide under one name.
- `IntegerStackModel::parse` reads the section and attaches it through
  `attach_native_q31_state`; serving needs no runtime side-load. A stack without the section keeps
  the existing artifact-derived path.
- `export_fbs_trunk_uorlut01_with_q31` writes the state. The exporter inserts the `q31` key and
  table only when a state is passed, so a plain export is byte-identical to before. The state is
  validated through `Q31HamiltonModel::from_bytes` before it is written.
- `inspect_uorlut01` exposes the validated `q31` record, omitted from the written metadata when
  absent.
- Two focused tests cover the artifact round-trip plus bit-identical serving, and a tampered state
  word refused at parse.
- The serving/export page and the M4 notebook carry the new contract and one result row.

Size: 434 inserted and 12 removed lines across 8 files (4 Rust files + 4 records), inside the
ORDER's 250–450-line estimate before this REPORT.

## Measurements

- Artifact record: schema `uor-r4.lut-stack/12`, SHA-256 over the exact 48 bytes, epoch `64`,
  step shift `8`, version `1`.
- Serving parity: the artifact-attached model and the same artifact with the state attached in
  memory produce identical token ids and identical raw Q31 traces on `[3, 5, 8, 13]`.
- Refusal: flipping one state word makes `IntegerStackModel::parse` fail on the digest.
- Plain path: an export without the state has no `q31` record and its bytes are unchanged.
- `8 / 8` focused Q31 tests pass. The integer library is `651` passed, `1` ignored.
- No language, capability, speed, memory, M4-bar or milestone result is measured.

## Checks run at the head

- `cargo check --locked -p uor-r4-integer --features serving-wasm` — passed.
- `cargo test --locked -p uor-r4-integer --features serving-wasm q31_serving` — `8` passed.
- `cargo test --locked -p uor-r4-integer --features serving-wasm --lib` — `651` passed, `1` ignored.
- `cargo clippy --locked --no-deps -p uor-r4-integer --features serving-wasm --lib` — exit 0; the
  only warning in a touched file is pre-existing (`format.rs:327`).
- `git diff --check` — passed.
- `uor-harness check --only harness.native_space_contract` — 1 rule, 0 failed.
- `uor-harness loop report --check`, `loop pr-check --local` and the exact-head review — at the head.

## Skills used

- `native-space-cycle`: one state, one operator, serialization without translation.
- `solution-integrity`: kept the container change out of the arithmetic path and validated the
  domain through the existing operator.
- `scope-guard`: avoided the file claimed by PR #3985 and every out-of-scope path.
- `test-freeze`: added tests; changed or deleted none.
- `working-discipline`: recorded the failed claim, the compile move and the formatting baseline.

## Still open

- The synthetic artifact proves the container and serve path. A trained real model exported with
  the state, restored in a fresh `uor-chat` process and served at the M4 bar is still open.
- The `uor-r4-lut` facade keeps refusing `/12` by schema; teaching the retained metadata tooling
  the record is separate work.
- No model training or kernel change is part of this unit.

## Plain summary

The served file can now carry the trained 48-byte model state itself. Before, the integer session
could read that state only when a caller handed it over separately. Now the file holds the exact
bytes in one table, binds their digest, and attaches them on load, so the same state training
wrote is the state serving runs. The test compares raw words and tokens, and a changed byte is
refused. This proves a finite artifact path, not language ability or the M4 bar.
