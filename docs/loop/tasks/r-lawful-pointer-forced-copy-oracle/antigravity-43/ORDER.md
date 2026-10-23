# ORDER: r-lawful-pointer-forced-copy-oracle, antigravity-43

Lab: antigravity. Item: [r-lawful-pointer-forced-copy-oracle](../../../../roadmap/items/r-lawful-pointer-forced-copy-oracle.md). Base: `d0527ed8f`.

## Goal

Deliver the host-checkable design, lawful integer pointer implementation, finite host controls, and draft forced-copy oracle pre-registration sheet for [`r-lawful-pointer-forced-copy-oracle`](../../../../roadmap/items/r-lawful-pointer-forced-copy-oracle.md) (`#2880`):
1. Publish the design note (`docs/labs/r-lawful-pointer-forced-copy-oracle/README.md` and `#2880` comment) specifying the geometric construction of the lawful pointer on the D32 discrete integer path (exact slot-address semiprime `entity_prime * attribute_prime` / prime-atom divisibility $\gcd(q, \pi_j) > 1$ or ordered $n$-let admission, canonical token-id fold table affecting match decisions only, integer rank/margin selection with zero normalized-exponential terms and zero dot-product/matmul operations, and why the forced-copy oracle requires an interventional serving-time span-override hook rather than stored dense float outputs).
2. Implement the lawful integer pointer and forced-copy span-oracle instrument in `crates/uor-r4-training/src/lawful_pointer.rs` (`SlotAddressKey`, `RouteAdmissionMode`, `CanonicalFoldTable`, `CopySpanOracle`, `locate_last_value_span`, `LawfulPointerHead`) with finite host unit tests verifying slot-address admission on fixed entity/attribute primes, bit-for-bit oracle-off reproduction of the base scorer, `O1` vs `O2` multi-token value span emission, refusal of empty/reversed/out-of-window spans, last-occurrence span locator, and canonical fold-table preservation of emitted token IDs.
3. Author the draft forced-copy oracle pre-registration sheet `docs/harness/prep/r-lawful-pointer-forced-copy-oracle-DRAFT-2026-10-23.md` with all 8 required `prep.sheet_shape` blocks (`Bar`, `Base artifact`, `Data split`, `Steps`, `Stop`, `Cost`, `Minimum test power`, `TEST FITNESS: PENDING`).

## Cancel probe (run first)

- Probe: `uor-harness loop gates --quick` (cancelled with SIGINT)
- Cancel signal: `SIGINT` (`exit 130`), clean exit in `< 1s`.

## Files in scope

- `crates/uor-r4-training/src/lib.rs`
- `crates/uor-r4-training/src/lawful_pointer.rs`
- `docs/harness/prep/r-lawful-pointer-forced-copy-oracle-DRAFT-2026-10-23.md`
- `docs/labs/r-lawful-pointer-forced-copy-oracle/README.md`
- `docs/loop/tasks/r-lawful-pointer-forced-copy-oracle/antigravity-43/ORDER.md`
- `docs/loop/tasks/r-lawful-pointer-forced-copy-oracle/antigravity-43/REPORT.md`

## Done when

- `cargo test -p uor-r4-training --lib lawful_pointer`, `cargo check -p uor-r4-training --bins`, and `cargo fmt --check -p uor-r4-training` pass with zero warnings.
- `docs/labs/r-lawful-pointer-forced-copy-oracle/README.md` and `docs/harness/prep/r-lawful-pointer-forced-copy-oracle-DRAFT-2026-10-23.md` are committed, and the design note is posted on `#2880`.
- `uor-harness loop gates --full`, `uor-harness loop report --check`, and `uor-harness loop pr-check --local` pass at the branch head.

## Stop and ask if

- Any GPU evaluation run is needed before a non-author `TEST FITNESS: FIT` review, owner/PM bar registration, and a bound discrete memory-answering checkpoint exist (CHARTER §3.5, §4.5, §5.1, §5.2).
- Any file outside `r-lawful-pointer-forced-copy-oracle`'s `scope:` (`crates/uor-r4-training/**`, `docs/harness/prep/r-lawful-pointer-forced-copy-oracle*`, `docs/labs/r-lawful-pointer-forced-copy-oracle/**`) would need to change.
