# REPORT: m1-address-as-read, antigravity-8

Lab: antigravity. Order: [ORDER.md](ORDER.md).

References #2833.

## What did not work, first

- Writing `crates/uor-r4-object-store/tests/address_read.rs` before adding `src/address_read.rs` and running `CARGO_TARGET_DIR=target ~/.cargo/bin/cargo test -p uor-r4-object-store --test address_read` failed as expected with `error[E0432]: unresolved import uor_r4_object_store::address_read`.
- In the initial `sorted_merge_gcd_sieve_admits_shared_primes_and_ranks_deterministically` test, `slot_two_ord` was inserted after `slot_two_rev` (`sequence = 3` vs `sequence = 2`), so when `ordered_pair_overlap` was deliberately zeroed in `AddressDirectory::read_sieve_from` (`let rank = (shared, 0usize, slot.current.sequence)`), the test still passed because `sequence` broke the tie in the same direction. Inserting `slot_two_ord` before `slot_two_rev` (`sequence = 2` vs `sequence = 3`) fixed the confound: zeroing `ordered_pair_overlap` then failed as expected (`assertion left == right failed: left: PrimeAddress { primes: [2, 3, 7], ordered_pairs: [(3, 2)] }, right: PrimeAddress { primes: [2, 3, 5], ordered_pairs: [(2, 3)] }`), and passed once `pairs` was restored.

## What changed

- Posted the design note on `#2833` (`https://github.com/UOR-Foundation/uor-r4/issues/2833#issuecomment-6336053627`) specifying the geometric `PrimeAddress` construction, resolution to `uor-r4-object-store` targets (`ByteSpan`, `WholeObject`, `TokenOccurrence`), typed read outcomes (`Hit`, `Previous`, `Absent`, `Evicted`), why it uses zero softmax and zero matmul, and the finite CPU controls.
- Added `crates/uor-r4-object-store/src/address_read.rs` (exported as `pub mod address_read;` in `crates/uor-r4-object-store/src/lib.rs`, with `PinnedTokenizer::token_bytes` made `pub(crate)` in `crates/uor-r4-object-store/src/bindings.rs`) implementing `PrimeAddress`, `ObjectTarget`, `AddressDirectory`, `AddressReadOutcome`, and `VersionSelect`.
- Added 6 finite CPU control tests in `crates/uor-r4-object-store/tests/address_read.rs` covering exact hit (`ByteSpan`, `WholeObject`, `TokenOccurrence`), exact miss (`Absent` with `0` backend record fetches), wrong address / tampered manifest / mismatched bindings refusal, versioned overwrite (`Current` vs `Previous`) and capacity ring eviction (`Evicted`), division-free sorted-merge GCD sieve admission and ranking, and byte-identical round trips across `Store` export/import at carry boundaries.
- Added the one-page pre-registration sheet `docs/harness/prep/m1-address-as-read-oracle-DRAFT-2026-10-22.md` for the serving-time address oracle test (awaiting non-author `TEST FITNESS: FIT` before GPU evaluation), added `docs/labs/m1-address-as-read/README.md` with the in-scope `m3-durable-memory` notebook row, and linked `address_read.rs` and the pre-registration sheet in `docs/mechanisms/exact-memory.md`.

## Measurements

- `CARGO_TARGET_DIR=target ~/.cargo/bin/cargo test -p uor-r4-object-store --test address_read` on macOS aarch64: 6 passed, 0 failed in 0.01s.
- `CARGO_TARGET_DIR=target ~/.cargo/bin/cargo test -p uor-r4-object-store` (full crate suite across unit, `address_read`, `bindings`, `controls`, `objpeer_cli`, `peer`, `protocol_vectors`, and `showcase`) on macOS aarch64: 67 passed (61 existing + 6 new in `address_read`), 0 failed in 27.4s.

## Checks run at the head

- `~/.cargo/bin/cargo fmt -p uor-r4-object-store -- --check`: exit 0.
- `CARGO_TARGET_DIR=target ~/.cargo/bin/cargo clippy -p uor-r4-object-store --all-targets -- -D warnings`: exit 0.
- `CARGO_TARGET_DIR=target ~/.cargo/bin/cargo test -p uor-r4-object-store --test address_read`: 6 passed, 0 failed.
- `CARGO_TARGET_DIR=target ~/.cargo/bin/cargo test -p uor-r4-object-store`: 67 passed, 0 failed.
- `uor-harness loop gates --full`: `fmt-claim-wording` GREEN, `fmt-harness` GREEN, `claim-wording` GREEN, `loop-gates-watch` GREEN, `loop-drive` GREEN, `git-hooks` GREEN.
- `uor-harness loop report --check`: PASS.
- `uor-harness loop pr-check --local`: PASS.

## Skills used

- `uor-research-architect` (`.agents/skills/uor-research-architect/SKILL.md`).

## Still open

- `docs/roadmap/items/m1-address-as-read.md` stays `status: ready`: Step 5 (GPU execution of the serving-time oracle test in `docs/harness/prep/m1-address-as-read-oracle-DRAFT-2026-10-22.md`) requires a non-author `TEST FITNESS: FIT` review (CHARTER §4.5) and owner approval in chat before any GPU pod run.

## Plain summary

- Built `uor_r4_object_store::address_read` (`crates/uor-r4-object-store/src/address_read.rs`), connecting canonical prime/semiprime addresses (`PrimeAddress`) and division-free sorted-merge GCD sieve queries (`gcd_shared_primes`, `ordered_pair_overlap`) to verified byte spans, whole objects, and token occurrences in `uor-r4-object-store` (`AddressDirectory`, `ObjectTarget`, `AddressReadOutcome`).
- Added 6 finite CPU control tests in `crates/uor-r4-object-store/tests/address_read.rs` verifying exact hits, zero-fetch `Absent` misses, wrong-address and tampered-target refusal, versioned `Current`/`Previous` overwrites, bounded capacity `Evicted` tombstones, sorted-merge GCD ranking, and byte-identical round trips across `Store` export/import.
- Drafted the one-page pre-registration sheet `docs/harness/prep/m1-address-as-read-oracle-DRAFT-2026-10-22.md` for the serving-time address oracle test (awaiting non-author FIT review before GPU evaluation) and recorded the lab summary and notebook row in `docs/labs/m1-address-as-read/README.md`.
