# ORDER: r-prime-zeta-vs-nulls, antigravity-27

Lab: antigravity. Item: [r-prime-zeta-vs-nulls](../../../../roadmap/items/r-prime-zeta-vs-nulls.md). Base: `ea257ef40`.

## Goal

The item's done-when reads:

> Zero-update reach and capacity numbers for prime-zeta, random-injective and quota-only-injective codes on the same panel, recorded as a notebook row. The bar is not yet registered; registering it, and defining "reach" and "capacity", is part of the pre-registration sheet.

This task delivers steps 1–4 of [`docs/roadmap/items/r-prime-zeta-vs-nulls.md`](../../../../roadmap/items/r-prime-zeta-vs-nulls.md):
1. Design note on [`#2881`](https://github.com/UOR-Foundation/uor-r4/issues/2881) (and [`#2029`](https://github.com/UOR-Foundation/uor-r4/issues/2029)) defining `CodebookMode::RandomInjective { seed: u32 }` and `CodebookMode::QuotaOnlyInjective { seed: u32 }`, proving why `coverage_census` alias ceiling (`14,287 / 14,287`, tie mass `0`) is algebraically invariant across all `4,096`-injective tables, and defining the zero-update reach and capacity measures.
2. Pure-integer host construction of `RandomInjective` and `QuotaOnlyInjective` in `crates/uor-r4-discrete/src/prime_zeta.rs` and CLI export in `crates/uor-r4-discrete/src/bin/prime-zeta-codebook.rs` with zero new floating-point types or literals (`D32` ratchet unchanged), zero softmax (`D23`), and zero matmul (`D25`), plus finite host unit tests across seeds `0, 1, 2`.
3. Draft pre-registration sheet `docs/harness/prep/r-prime-zeta-vs-nulls-DRAFT-2026-10-22.md` (`TEST FITNESS: PENDING`, bar marked `PROPOSED, NOT REGISTERED`) and staged zero-update GPU battery runbook and host-exported null table pins in `docs/labs/r-prime-zeta-vs-nulls/README.md`.

## Cancel probe (run first)

Perturbing the uniqueness or writer-cell quota check in `crates/uor-r4-discrete/src/prime_zeta.rs` causes `~/.cargo/bin/cargo test -p uor-r4-discrete prime_zeta` to fail (`FAILED`). Restoring it returns `PASS`.

## Files in scope

- `crates/uor-r4-discrete/**`
- `docs/harness/prep/r-prime-zeta-vs-nulls*`
- `docs/labs/r-prime-zeta-vs-nulls/**`

## Done when

- Design note posted on [`#2881`](https://github.com/UOR-Foundation/uor-r4/issues/2881) and [`#2029`](https://github.com/UOR-Foundation/uor-r4/issues/2029).
- `CodebookMode::RandomInjective { seed: u32 }` and `CodebookMode::QuotaOnlyInjective { seed: u32 }` implemented in `crates/uor-r4-discrete/src/prime_zeta.rs` and exported via `crates/uor-r4-discrete/src/bin/prime-zeta-codebook.rs`.
- `~/.cargo/bin/cargo test -p uor-r4-discrete prime_zeta` passes all finite controls with zero new `D23`/`D25`/`D32` ratchet sites and `training::driver_source_sha256()` unchanged.
- `docs/harness/prep/r-prime-zeta-vs-nulls-DRAFT-2026-10-22.md` and `docs/labs/r-prime-zeta-vs-nulls/README.md` pass `uor-harness loop gates --full`.

## Stop and ask if

- Any zero-update GPU pod execution (`eval-corpus`) is needed (requires non-author `TEST FITNESS: FIT`, owner/PM registration of the comparison bar, and explicit chat approval before pod rental).
- Any file outside `scope` must be modified.

