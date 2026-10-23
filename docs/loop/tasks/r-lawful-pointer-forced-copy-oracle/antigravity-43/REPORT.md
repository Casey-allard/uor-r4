# REPORT: r-lawful-pointer-forced-copy-oracle, antigravity-43

## What did not work, first

- The historical pointer-copy implementation (`crates/uor-r4-training/src/geometric_stack.rs`, `copy_identity.rs`, and `stack_prime_route.rs`) and its 2026-10-12 copy-span oracle (`StackModel::set_copy_span_oracle`, `chat-grade reply copy_oracle=span`) are gated out of the active build under `#[cfg(any())]` and D24/D29/D30 because the historical pointer used two normalized-exponential terms and a dense floating-point inner-product score (`docs/mechanisms/pointer-key.md:40-45`). Re-executing the historical dense float pointer checkpoints (`09e9dd3d…`, `71409eac…`, `70fef0ba…`) is barred by CHARTER §3.2–§3.4 (D23, D30).
- Static stored outputs from the historical 2026-10-10 runs record only the unforced condition `N` (`23/40`) and cannot answer the counterfactual `O1` (span supplied, learned gate) or `O2` (span supplied, forced gate) interventions. Consequently, the forced-copy oracle requires a lawful integer pointer with an interventional serving-time span-override hook (`CopySpanOracle`).
- Executing the GPU forced-copy oracle run itself (`N` / `O1` / `O2` on a bound discrete checkpoint) is gated on (a) a non-author `TEST FITNESS: FIT` review of the pre-registration sheet (CHARTER §4.5), (b) owner/PM registration of the `N` / `O1` / `O2` diagnostic bar on `#2880` (CHARTER §5.2), (c) a released and trained discrete memory-answering checkpoint (`uor-r4.discrete-spiralcore-fbs/2`), and (d) a typed owner release (CHARTER §5.1). Accordingly, `docs/roadmap/items/r-lawful-pointer-forced-copy-oracle.md` remains `status: ready`.

## What changed

- [`crates/uor-r4-training/src/lawful_pointer.rs`](../../../../../crates/uor-r4-training/src/lawful_pointer.rs) and [`crates/uor-r4-training/src/lib.rs`](../../../../../crates/uor-r4-training/src/lib.rs):
  - Implemented the D32 discrete integer pointer-copy head (`LawfulPointerHead`) and serving-time forced-copy span oracle (`CopySpanOracle::{Off, O1SpanLearnedGate, O2SpanForcedGate}`) with zero floating-point types, zero normalized-exponential terms, and zero dot-product or matrix-multiplication operations:
    - `SlotAddressKey`: exact square-free semiprime key $K_{\text{slot}} = p_{\text{entity}} \cdot p_{\text{attribute}} \in \mathbb{Z}_{>1}$ over distinct primes (`is_prime_u64`), admitting a candidate value span iff $K_{\text{slot}} \mid \pi_c$ ($\gcd(K_{\text{slot}}, \pi_c) = K_{\text{slot}}$).
    - `RouteAdmissionMode::{SlotAddressSemiprime, SharedAtomGcd, OrderedNlet}`: exact slot-address semiprime divisibility, prime-atom divisibility $\gcd(q, \pi_c) > 1$ weighted by Q8 integer bit-rank rarity (`ilog2_gcd_weight_q8`), and ordered $n$-let equality over `CanonicalFoldTable`.
    - `CanonicalFoldTable`: D11-legal token-id fold table (folding ASCII case and at most one leading space `b' '` to the least token ID in the equivalence class for match admission only, leaving emitted token IDs untouched).
    - `locate_last_value_span`: locates the last occurrence of an expected value across token boundaries in the decoded byte stream of `window_tokens`.
    - Six finite host unit tests verifying exact slot-address semiprime admission across multi-token value spans, bit-for-bit oracle-off reproduction of the base margin scorer, `O1` vs `O2` separation of learned vs forced gate margins and bit-for-bit restoration on `clear_copy_span_oracle`, refusal of empty/reversed/out-of-window oracle spans, last-occurrence multi-token span location, and canonical fold-table preservation of emitted tokens.
- [`docs/labs/r-lawful-pointer-forced-copy-oracle/README.md`](../../../../labs/r-lawful-pointer-forced-copy-oracle/README.md):
  - Published the design note giving the geometric construction of the lawful pointer, settling the source disagreement in favor of the audit synthesis (the forced-copy oracle is interventional on the copy selection step), explaining why D32 discrete integer sheet is chosen over D37 offline float, and documenting the 6 finite host controls.
- [`docs/harness/prep/r-lawful-pointer-forced-copy-oracle-DRAFT-2026-10-23.md`](../../../../harness/prep/r-lawful-pointer-forced-copy-oracle-DRAFT-2026-10-23.md):
  - Authored the one-page pre-registration sheet with all 8 required `prep.sheet_shape` blocks (`Bar`, `Base artifact`, `Data split`, `Steps`, `Stop`, `Cost`, `Minimum test power`, `TEST FITNESS: PENDING`), binding the sealed 256-row held-out `m2-fresh256` panel (`plan.json` `sha256:0ed1ab98…`, `inputs.json` `sha256:1de2fc28…`, `labels.json` `sha256:3b5c5477…`, `manifest.json` `sha256:7c33f3f7…`), the `N` / `O1` / `O2` diagnostic branches against the measured 16-block cluster SE (`8.64` rows / `4.32` pairs), and the stop conditions.

## Measurements

- `CARGO_TARGET_DIR=/Users/casey.allard/uor-r4/local/antigravity/target cargo test -p uor-r4-training --lib lawful_pointer`: `6 passed; 0 failed`.
- `CARGO_TARGET_DIR=/Users/casey.allard/uor-r4/local/antigravity/target cargo check -p uor-r4-training --bins`: exit `0`.
- `cargo fmt --check -p uor-r4-training`: exit `0`.

## Checks run at the head

- `uor-harness loop gates --full`: all 11 gates `GREEN` (`fmt-claim-wording` through `loop-review-changes-blocks`).
- `uor-harness loop report --check`: `PASS`.
- `uor-harness loop pr-check --local`: `PASS`.
- `uor-harness check --only harness.documented,prep.sheet_shape`: `0 failed (0 error, 0 warn)`.

## Skills used

- `uor-research-architect`, `scope-guard`, `solution-integrity`, `context-guard`.

## Still open

- Non-author `TEST FITNESS: FIT` review of [`docs/harness/prep/r-lawful-pointer-forced-copy-oracle-DRAFT-2026-10-23.md`](../../../../harness/prep/r-lawful-pointer-forced-copy-oracle-DRAFT-2026-10-23.md) (CHARTER §4.5), owner/PM registration of the `N` / `O1` / `O2` diagnostic bar on `#2880` (CHARTER §5.2), a bound discrete memory-answering checkpoint SHA-256, and a typed owner release (CHARTER §5.1) before running the GPU forced-copy oracle evaluation.

## Plain summary

Built the D32 lawful integer pointer (`SlotAddressKey`, `RouteAdmissionMode`, `CanonicalFoldTable`, `CopySpanOracle`, `locate_last_value_span`, and `LawfulPointerHead`) with 6 passing finite host controls in `crates/uor-r4-training/src/lawful_pointer.rs`, published the design note (`docs/labs/r-lawful-pointer-forced-copy-oracle/README.md`), and authored the draft forced-copy oracle pre-registration sheet (`docs/harness/prep/r-lawful-pointer-forced-copy-oracle-DRAFT-2026-10-23.md`) on the sealed 256-row `m2-fresh256` panel.

References #2880
