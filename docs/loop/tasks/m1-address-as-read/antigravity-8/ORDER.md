# ORDER: m1-address-as-read, antigravity-8

Lab: antigravity. Item: [m1-address-as-read](../../../../roadmap/items/m1-address-as-read.md). Base: `fb9e49e92`.

## Goal

The item's done-when reads:

> A read that resolves an address to stored content from the object store, with finite tests, and a registered serving-time oracle test that measures the gain from supplying the true address.

## Cancel probe (run first)

Inspected `crates/uor-r4-object-store/src/lib.rs`, `docs/mechanisms/exact-memory.md`, `archive/labs/addressed-memory-2026-10-10/README.md`, `archive/labs/d19-object-store-2026-10-15/README.md`, `docs/labs/uorobj01-conformance-2026-10-18/README.md`, and `docs/labs/repo-renormalization-2026-10-19/audit/mechanisms/exact-memory.json`:
- `uor-r4-object-store` currently provides `Store`, `Manifest`, `Record`, `Bindings`, and `peer`, but has no `address_read` consumer module resolving a geometric prime/semiprime address to stored object-store content (`Manifest` / byte range / `Bindings` occurrence), and no pre-registration sheet exists under `docs/harness/prep/m1-address-as-read*`.
- Still needed (CPU deliverables for Steps 1–4 of `#2833`, before any GPU oracle run): (1) design note on `#2833`, (2) `address_read` consumer module and finite CPU controls in `crates/uor-r4-object-store`, (3) one-page pre-registration sheet `docs/harness/prep/m1-address-as-read-oracle-DRAFT-2026-10-22.md` for the serving-time oracle test (awaiting non-author FIT review before GPU evaluation), and (4) lab record in `docs/labs/m1-address-as-read/README.md` and code link in `docs/mechanisms/exact-memory.md`.

## Files in scope

- `crates/uor-r4-object-store/**`
- `docs/mechanisms/exact-memory.md`
- `docs/labs/m1-address-as-read/**`
- `docs/harness/prep/m1-address-as-read*`

## Done when

- Design note posted on `#2833` specifying the geometric address construction, resolution to an object-store target (`Manifest` + byte span or `Bindings` occurrence), typed read outcomes (`Hit`, `Previous`, `Absent`, `Evicted`), why it uses zero softmax and zero matmul, and the finite CPU controls.
- `crates/uor-r4-object-store` exports `address_read` with finite CPU unit and integration tests covering exact hit, exact miss (`Absent`), wrong address, overwrite/eviction history, sorted-merge prime-factor `gcd` admission, and byte-identical round trip through `Store`, and `cargo test -p uor-r4-object-store` passes.
- One-page pre-registration sheet `docs/harness/prep/m1-address-as-read-oracle-DRAFT-2026-10-22.md`, lab record `docs/labs/m1-address-as-read/README.md`, and `docs/mechanisms/exact-memory.md` code entry are committed, and `uor-harness loop pr-check --local` passes (`References #2833`, holding `status: ready` until the GPU oracle run completes after FIT).

## Stop and ask if

- GPU execution of the serving-time oracle test is ready to run (requires non-author `TEST FITNESS: FIT` on the pre-registration sheet and Casey's approval in chat).
- Any file outside `crates/uor-r4-object-store/**`, `docs/mechanisms/exact-memory.md`, `docs/labs/m1-address-as-read/**`, or `docs/harness/prep/m1-address-as-read*` needs to change.
