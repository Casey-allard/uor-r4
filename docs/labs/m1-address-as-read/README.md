# m1-address-as-read (#2833) — Object-store address-as-read consumer and oracle pre-registration

## Summary

Delivers Steps 1–4 of [`m1-address-as-read`](../../roadmap/items/m1-address-as-read.md) (`#2833`):
1. **Design note** posted on [`#2833`](https://github.com/UOR-Foundation/uor-r4/issues/2833#issuecomment-6336053627).
2. **Object-store consumer module** [`crates/uor-r4-object-store/src/address_read.rs`](../../../crates/uor-r4-object-store/src/address_read.rs) (`PrimeAddress`, `ObjectTarget`, `AddressDirectory`, `AddressReadOutcome`, `VersionSelect`), resolving canonical prime/semiprime addresses and division-free sorted-merge GCD sieve queries to verified byte spans, whole objects, and token occurrences in `uor-r4-object-store`.
3. **Finite CPU controls** in [`crates/uor-r4-object-store/tests/address_read.rs`](../../../crates/uor-r4-object-store/tests/address_read.rs) covering exact hit, exact miss (`Absent` with `0` backend fetches), wrong address / tampered manifest / mismatched bindings refusal, versioned overwrite (`Current` vs `Previous`) and bounded capacity eviction (`Evicted`), sorted-merge GCD sieve admission and ranking, and byte-identical round trips across `Store` export/import at carry boundaries.
4. **Serving-time oracle pre-registration sheet** [`docs/harness/prep/m1-address-as-read-oracle-DRAFT-2026-10-22.md`](../../harness/prep/m1-address-as-read-oracle-DRAFT-2026-10-22.md), ready for non-author `TEST FITNESS: FIT` review before any GPU evaluation run (Step 5).

## Geometry and ratchet checks

- **Geometry only, no softmax, no matmul (CHARTER §3.1–§3.3, D11, D23, D30):** `PrimeAddress` uses integer primality checks, sorted `u32` factor lists, exact semiprime products $k = p_a p_b$, and a two-pointer sorted merge for $\gcd(q, A_c) > 1$ admission and integer tuple ranking `(shared_primes, ordered_pair_matches, sequence)`. Zero floating point, zero exponentials/softmax/squashing, and zero matrix multiplication.

## Notebook row (in-scope record for `m3-durable-memory`)

Because `docs/research/notebook/m3-durable-memory.md` is outside `m1-address-as-read.md`'s frontmatter `scope:`, the notebook row is recorded here in the in-scope lab README:

| Date | Item | Change / Finding | Verification |
|---|---|---|---|
| 2026-10-22 | `m1-address-as-read` (`#2833`, Steps 1–4) | Built `uor_r4_object_store::address_read` (`PrimeAddress`, `ObjectTarget`, `AddressDirectory`, `AddressReadOutcome`, `VersionSelect`) connecting prime/semiprime addresses and sorted-merge GCD sieve queries to verified `uor-r4-object-store` payloads (`ByteSpan`, `WholeObject`, `TokenOccurrence`), with 6 finite CPU control suites and the serving-time oracle pre-registration sheet `docs/harness/prep/m1-address-as-read-oracle-DRAFT-2026-10-22.md` (awaiting FIT before GPU evaluation). | `cargo test -p uor-r4-object-store` (61 passed: 55 existing + 6 new in `tests/address_read.rs`; binaries: `lib` 1, `address_read` 6, `bindings` 7, `controls` 10, `objpeer_cli` 22, `peer` 15; 67 passed with `--all-targets` including `protocol_vectors` 4 and `showcase` 2; 0 failed). |
