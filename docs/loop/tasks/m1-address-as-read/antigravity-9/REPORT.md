# REPORT: m1-address-as-read, antigravity-9

Lab: antigravity. Order: [ORDER.md](ORDER.md).

References #2833.

## What did not work, first

- In `antigravity-8`, the module doc comment in `crates/uor-r4-object-store/src/address_read.rs:7` included the D23-prohibited word in raw source text, which caused `uor-harness ratchet d23` to fail (`crates/uor-r4-object-store/src/address_read.rs: 0 -> 1 (+1)`, `total 4451 -> 4452`) as flagged by DeepSeek's review on PR `#2950` (`https://github.com/UOR-Foundation/uor-r4/pull/2950#issuecomment-6336410319`).
- `docs/labs/m1-address-as-read/README.md` and `antigravity-8/REPORT.md` also reported `66` vs `67` tests without distinguishing `cargo test -p uor-r4-object-store` (`61 passed: 55 existing + 6 new in tests/address_read.rs`) from `cargo test -p uor-r4-object-store --all-targets` (`67 passed`, which adds the `protocol_vectors` 4 and `showcase` 2 example test targets).
- Earlier in `antigravity-8`, writing `crates/uor-r4-object-store/tests/address_read.rs` before adding `src/address_read.rs` failed as expected with `error[E0432]: unresolved import uor_r4_object_store::address_read`, and zeroing `ordered_pair_overlap` in `AddressDirectory::read_sieve_from` after ordering `slot_two_ord` before `slot_two_rev` failed as expected (`left: PrimeAddress { primes: [2, 3, 7], ordered_pairs: [(3, 2)] }, right: PrimeAddress { primes: [2, 3, 5], ordered_pairs: [(2, 3)] }`).

## What changed

- Reworded `crates/uor-r4-object-store/src/address_read.rs:7-8` to state `"Uses an exact integer consumer with no floating point, no exponential or squashing form, and no dense matrix product (CHARTER §3.1–§3.3, D11, D23, D30)"`, removing the D23 token from executable source and restoring `uor-harness ratchet d23` to `OK: no ... added. 4451 remaining across 165 files (baseline 4451)`.
- Updated `docs/labs/m1-address-as-read/README.md` and this `REPORT.md` to record both the exact `cargo test -p uor-r4-object-store` count (`61 passed: 55 existing + 6 new in tests/address_read.rs`; binaries: `lib` 1, `address_read` 6, `bindings` 7, `controls` 10, `objpeer_cli` 22, `peer` 15) and the `--all-targets` count (`67 passed`, adding `protocol_vectors` 4 and `showcase` 2).
- Retains all `antigravity-8` deliverables: the design note on `#2833` (`https://github.com/UOR-Foundation/uor-r4/issues/2833#issuecomment-6336053627`), `crates/uor-r4-object-store/src/address_read.rs`, the 6 finite CPU control suites in `crates/uor-r4-object-store/tests/address_read.rs`, `docs/harness/prep/m1-address-as-read-oracle-DRAFT-2026-10-22.md`, `docs/labs/m1-address-as-read/README.md`, and `docs/mechanisms/exact-memory.md`.

## Measurements

- `CARGO_TARGET_DIR=target ~/.cargo/bin/cargo test -p uor-r4-object-store --test address_read` on macOS aarch64: 6 passed, 0 failed in 0.01s.
- `CARGO_TARGET_DIR=target ~/.cargo/bin/cargo test -p uor-r4-object-store` on macOS aarch64: 61 passed (55 existing + 6 new in `tests/address_read.rs`: `lib` 1, `address_read` 6, `bindings` 7, `controls` 10, `objpeer_cli` 22, `peer` 15), 0 failed.
- `CARGO_TARGET_DIR=target ~/.cargo/bin/cargo test -p uor-r4-object-store --all-targets` on macOS aarch64: 67 passed (the 61 above + `protocol_vectors` 4 + `showcase` 2), 0 failed.

## Checks run at the head

- `uor-harness ratchet d23`: `OK: 4451 remaining across 165 files (baseline 4451)`.
- `uor-harness ratchet d25`: `OK: no dense matrix product call sites added. 93 remaining across 26 files (baseline 93)`.
- `~/.cargo/bin/cargo fmt -p uor-r4-object-store -- --check`: exit 0.
- `CARGO_TARGET_DIR=target ~/.cargo/bin/cargo clippy -p uor-r4-object-store --all-targets -- -D warnings`: exit 0.
- `CARGO_TARGET_DIR=target ~/.cargo/bin/cargo test -p uor-r4-object-store`: 61 passed, 0 failed.
- `uor-harness loop gates --full`: `fmt-claim-wording` GREEN, `fmt-harness` GREEN, `claim-wording` GREEN, `loop-gates-watch` GREEN, `loop-drive` GREEN, `git-hooks` GREEN.
- `uor-harness loop report --check`: PASS.
- `uor-harness loop pr-check --local`: PASS.

## Skills used

- `uor-research-architect` (`.agents/skills/uor-research-architect/SKILL.md`).

## Still open

- `docs/roadmap/items/m1-address-as-read.md` stays `status: ready`: Step 5 (GPU execution of the serving-time oracle test in `docs/harness/prep/m1-address-as-read-oracle-DRAFT-2026-10-22.md`) requires a non-author `TEST FITNESS: FIT` review (CHARTER §4.5) and owner approval in chat before any GPU pod run.

## Plain summary

- Repaired PR `#2950` per DeepSeek's review by removing the D23-prohibited word from the module doc comment in `crates/uor-r4-object-store/src/address_read.rs:7` so `uor-harness ratchet d23` is GREEN (`4451`, baseline `4451`) and by recording the exact `cargo test -p uor-r4-object-store` count (`61 passed: 55 existing + 6 new in tests/address_read.rs`, plus `6` example tests under `--all-targets`) in `docs/labs/m1-address-as-read/README.md` and `REPORT.md`.
- Delivers `uor_r4_object_store::address_read` (`PrimeAddress`, `ObjectTarget`, `AddressDirectory`, `AddressReadOutcome`, `VersionSelect`), 6 finite CPU control suites in `crates/uor-r4-object-store/tests/address_read.rs`, and the serving-time oracle pre-registration sheet `docs/harness/prep/m1-address-as-read-oracle-DRAFT-2026-10-22.md`.
