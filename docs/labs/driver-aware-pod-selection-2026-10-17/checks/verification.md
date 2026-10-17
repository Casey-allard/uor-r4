# Exact-driver selector implementation checks — 17 October 2026

Author checks only; independent source review/protected delivery remain pending.
Base commit6d3175dabaca055785ac701736296188c83cff94. Owned branch
codex/cf1-driver-selection. No nested AGENTS.md applies below the provider crate
or scripts/pod. No CUDA/model source, remote actual-driver gate or experiment
allowlist changed. No real API/credentials/provider/pod/model operation occurred.
All provider/SSH/GitHub operations in the runner suite use local fake adapters.

## Exact source and dependency provenance

`final-source-identities.txt` binds the seven inspected crate/runner files and
the base workspace lock. The only five authored files are provider src/lib.rs,
provider README.md, scripts/pod/uor-pod, its dryrun test, and config.example.
Parent owns all canonical record/claim/ledger/delivery changes independently.
No commit/push/post was made by this author.

The sparse workspace was not used as a build target. Its unchanged provider
Cargo.toml/src/lib.rs/src/main.rs were copied byte-identically into this local
receipt's provider/ directory, with only serde_json as the declared dependency.
The base workspace Cargo.lock was initially copied; Cargo --locked refused
because the isolated package graph required lock adjustment (retained in
provider-test.locked-refusal.log). Cargo then resolved entirely offline. Source
Cargo.toml and both copied Rust files match the final worktree bytes.

Root lock blobaf0ed1393bfd85331baa1054a190576ce0356ea1, SHA256
6656b5eebb40ce5bf744031f62a500ddd697ade8ccc29000120bc83db904b06c.
Isolated lockSHA256034f31f9ffb14c320009586010e8a0ad703a8e3a96315298408f485c81623a30.
The resolved direct/transitive versions match the root lock: serde_json1.0.150,
serde_core1.0.228, zmij1.0.23, memchr2.8.3, itoa1.0.18. Both locks are retained.
This isolated validation is not a blanket workspace build claim.

## Actual successful checks

Commands used Rust/Cargo1.97.1, CARGO_PROFILE_DEV_DEBUG=0,
CARGO_PROFILE_TEST_DEBUG=0, CARGO_INCREMENTAL=0 and an owned external target.

- cargo test --offline --manifest-path provider/Cargo.toml:8 library tests PASS,
  zero failures; binary/doc-test targets contain zero tests. Compile3.51s;
  fixture execution0.00s as reported by Cargo. Exact driver strings, unsupported
  predicates, safe identity projection, existing requests, cap-field refusal,
  pagination and credential-safe request/response fixtures are exercised.
- cargo clippy --offline --manifest-path provider/Cargo.toml --all-targets --
  -D warnings:PASS,3.24s reported. No model dependency or feature compiled.
- cargo build --offline --manifest-path provider/Cargo.toml:PASS. Actual CLI
  --version exits0; driver_version>=580.95.05 refuses with exit1 at argument
  parsing, before credentials/transport. provider-cli.log retains exact output.
- rustfmt --edition2021 --check on both actual crate Rust files:PASS.
- bash -n on actual runner and runner test:PASS. git diff --check:PASS.
- scripts/pod/tests/uor-pod-dryrun.sh:final365PASS/0FAIL. It uses only fake
  provider accounts/SSH/gh/bootstrap operations and a local stand-in Git origin.
  New checks cover cheaper wrong/missing/numeric driver exclusion, exact query,
  precreate identity/quote/time receipt, zero-create misses, selected price cap,
  malformed selector, Runpod rejection, selection linked before fake create and
  into up/cache, zero-create selected wait without Runpod fallback, ambiguous
  duplicate-ID refusal and explicit precreate receipt-append refusal. Existing
  unselected placement, caps, reconciliation, tokens and archive tests pass.

## Initial runner result and bounded test compatibility repairs

The initial full suite reported351PASS/10FAIL; runner-initial.log is retained.
No driver-filter or receipt predicate failed. One newly added --no-bootstrap
check expected `Ready: ssh`; that path correctly reports `SSH ready after`, so
its assertion was corrected. The existing HF-token test requested --no-bootstrap
but expected bootstrap token setup; it now enables only the mocked bootstrap
and resolves MAIN_SHA against the local origin. Existing cache tests expected
retired `parity PASS` wording while production bootstrap now says `build complete`;
only message expectations were updated, not cache checks or production bootstrap.
The missing-provider test now hides the Rust adapter through UOR_POD_VAST_BIN
instead of hiding the retired Python CLI from PATH. The harness itself always
pins its fake adapter so no inherited live Rust provider can be called.
The first repaired rerun361/0 proves the corrected fixture path; no production
bootstrap or model execution was introduced by those test-only changes.

Parent source review then identified two selected mutation-boundary risks:
duplicate matching objects could pass the old per-object jq predicate, and
receipt-write failure could be ignored because up_create runs under an explicit
OR-list (so errexit is insufficient). The guard now slurps and requires exactly
one matching object; selection construction, receipt construction and ledger
append each explicitly refuse on failure. mutation-boundary-jq.log preserves
old duplicate predicate exit0 versus new exit1. No empty-input bypass is claimed:
the installed jq already refuses empty input. Added fake duplicate-ID and
blocked-ledger cases assert zero create calls; the complete final365/0 rerun
passes them along with all prior cases. Runner-before-receipt-guards.log retains
the earlier361/0 run. Rust source is unchanged by this shell/test-only follow-up,
so its8-test/Clippy/CLI source identities remain applicable without rebuilding.

## Disk and artifact cleanup

Disk preflight had31678444KiB available versus31457280KiB floor. The largest
owned target observation was31020KiB. After logs/lock/binary SHA retention,
`cargo clean --manifest-path provider/Cargo.toml --target-dir target` removed
163files/29.8MiB; available31657452KiB remained above floor. No owner or peer
build target was used. Reproducible target files were not retained.
The tested normal CLI SHA256 was
cb524800bf583d1bffc956516f430e74128dc999224b5970b294588521cdc13d;
only its hash/transcript remain, not an installable/deployed binary. A selected
helper must be built/pinned after review/protected delivery before later use.

All source files are frozen for parent integration and independent review.
This receipt is host tooling evidence, not compute permission, current offer
availability, GPU/model parity or a timing result. Training/M2/Stage B HELD;
Q4 NOT_EXECUTABLE; zero new GPU spend.
