# Phase 6: Path-4 theory-deferred register

## What this phase ships

1. **[docs/theory_deferred.md](../theory_deferred.md)** — one row per
   Path-4 class, paired with the research question blocking its
   closure. 20 classes across three clusters:
   - Cohomology machinery (7 classes in `cohomology/`) —
     `CochainComplex`, `CohomologyGroup`, `GluingObstruction`,
     `RestrictionMap`, `Section`, `Sheaf`, `Stalk`.
   - Monoidal/operad (2 classes) — `monoidal:MonoidalProduct`,
     `operad:OperadComposition`.
   - Runtime-integration (5 `parallel/` + 6 `stream/`) — deferred
     pending a no_std concurrency/reactive-semantics amendment.

2. **[conformance/src/validators/rust/theory_deferred_register.rs]
   (../../conformance/src/validators/rust/theory_deferred_register.rs)**
    — new conformance check
   `rust/theory_deferred_register` asserting bijection between
   Path-4 classifications and register rows. Breaking the bijection
   in either direction fails CI.

## Register row format

The register is a Markdown table; each row is
`| <class IRI> | <namespace> | <research question> |`. The validator:

1. Walks `docs/theory_deferred.md`, parses every non-empty data row
   (skipping the `|---|` separator), extracts IRI + namespace + rq.
2. Walks `classify_all(Ontology::full())` and filters for
   `PathKind::Path4TheoryDeferred`.
3. Fails if:
   - A Path-4 class has no register row (missing row).
   - A register row names a class that's not Path-4 (dangling row).
   - A register row has an empty research-question column.

The check is additive; registers grow as new classes become
theory-deferred, and shrink as theory lands and classes re-classify
out of Path-4.

## Orphan count after Phase 6

Unchanged: **188 Null stubs** close the Path-1 + Path-2 orphans that
passed the emitable-set fixed point. The 20 Path-4 classes remain
orphan by design — the register records *why* each is orphan and
*what theory* must land before it can be closed.

Remaining orphans after all six phases:
- 20 Path-4 (theory-deferred, registered)
- 237 Path-1 cascade-dropped (enum accessors, cross-reference chains
  to Path-3/4 unemitted classes)
- 3 Path-2 cascade-dropped (same reason)

Total: 260 open trait orphans. Down from the starting ~430.

## Verification

- `cargo run --bin uor-conformance` — 536 passed (was 535; +1 from the
  new `rust/theory_deferred_register` check), 0 failed.
- `cargo test` — all 96 test suites green, including the new
  `conformance/tests/theory_deferred_register_parity.rs`.
- `git diff --exit-code` on `foundation/src/` after
  `cargo run --bin uor-crate` — clean.
