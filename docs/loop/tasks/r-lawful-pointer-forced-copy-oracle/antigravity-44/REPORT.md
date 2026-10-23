# REPORT: r-lawful-pointer-forced-copy-oracle, antigravity-44

## What did not work, first

- In `antigravity-43` (`ac3b556a4`), Claude's exact-head review (`issuecomment-6371944551`) identified two blocking issues and two should-fix items:
  1. **Blocking 1 (`O1` confounded source addressing with base-margin comparison):** `CopySpanOracle::Off` (`N`) copied only when `learned_gate_margin >= gate_threshold_margin && total_copy_margin > base_top_margin`, whereas `CopySpanOracle::O1SpanLearnedGate` (`O1`) checked only `learned_gate_margin >= gate_threshold_margin` and dropped `total_copy_margin > base_top_margin`. On rows where the gate passed but the base token outscored the copy margin, `O1` copied where `N` did not even on the same span.
  2. **Blocking 2 (Unpinned selection clauses in unit tests):** Every gate-open test in `antigravity-43` had `total_copy_margin` above `base_top_margin`, so deleting `&& total_copy_margin > base_top_margin` left the 6 tests green; `oracle_off_reproduces_base_scorer_bit_for_bit_when_gate_closed_or_unadmitted` tested only the gate-closed case and not the unadmitted-candidate case; and no test exercised `RouteAdmissionMode::SharedAtomGcd` or the latest-span-start (`span.0`) tie-break.
  3. **Should-fix 3 (`locate_last_value_span` lacked alphanumeric boundary checks):** Raw byte matching allowed a short value such as `"4"` to match inside `"47"` or `"14"`.
  4. **Should-fix 4 (Pre-registration sheet refinements for the FIT reviewer):** Branches A, B, and C did not form a total partition when row and pair deltas disagreed (`C_O1 - C_N >= 32` with `P_O1 - P_N < 16`), there was no wrong-span negative control (`W`), the paired standard error $\text{SE}_{\Delta}$ and seed list (`1, 2, 3`) were not pinned, and Stop 1(d) said "typed owner release" instead of chat approval (`#2895`).

## What changed

- [`crates/uor-r4-training/src/lawful_pointer.rs`](../../../../../crates/uor-r4-training/src/lawful_pointer.rs):
  - **Unconfounded `O1` vs `N` (`oracle_route_score` and unified margin comparison):** Added `LawfulPointerHead::oracle_route_score` so that an oracle-supplied span `[start, end)` receives its canonical route weight under `self.admission_mode` plus its candidate `learned_rank_bonus` (or `0` when absent). All three conditions (`Off`, `O1SpanLearnedGate`, `O2SpanForcedGate`) now pass through the single unified comparator `effective_gate_margin >= self.gate_threshold_margin && total_copy_margin > base_top_margin`. When `O1` is supplied the same span that `N` admitted, `O1` and `N` produce bit-for-bit identical `PointerDecision` outputs on every row.
  - **Alphanumeric boundary check in `locate_last_value_span` (`has_alphanumeric_boundary`):** Rejects matches where an alphanumeric start or end byte of `expected_value` is adjacent to another ASCII alphanumeric byte in `flat_bytes`, preventing `"4"` or `"7"` from matching inside `"47"` while still matching multi-token values (`" 4"` + `"7"` for `"47"`).
  - **Expanded finite host test suite (`6 -> 9` unit tests):**
    - Updated `oracle_off_reproduces_base_scorer_bit_for_bit_when_gate_closed_or_unadmitted` to test both the gate-closed branch (`learned_gate_margin = 40 < 100`, admitted candidate) and the unadmitted-candidate branch (`learned_gate_margin = 500 >= 100`, non-dividing slot semiprime `77` vs `55`).
    - Added `base_margin_comparison_blocks_copy_when_base_outscores_copy_in_both_n_and_o1` to pin `&& total_copy_margin > base_top_margin` (verifying that at `learned_gate_margin = 120 >= 100` and `total_copy_margin = 1474`, `base_top_margin = 1500` blocks the copy in both `N` and `O1`, whereas `base_top_margin = 1400` opens the copy in both `N` and `O1`).
    - Added `shared_atom_gcd_ranks_rarer_primes_higher_and_breaks_ties_by_latest_span_start` to pin `RouteAdmissionMode::SharedAtomGcd` prime rarity ranking (`prime 13 -> 768` over `prime 3 -> 256`) and the latest-start (`candidate.span.0 > best_span.0`) tie-break.
    - Added `locate_last_value_span_enforces_alphanumeric_boundaries` to verify `"4"` and `"7"` return `None` on `"Ana locker is 47."` and find standalone `"4"` at `[3, 4)` when followed by `"47"`.
- [`docs/labs/r-lawful-pointer-forced-copy-oracle/README.md`](../../../../labs/r-lawful-pointer-forced-copy-oracle/README.md) and [`docs/harness/prep/r-lawful-pointer-forced-copy-oracle-DRAFT-2026-10-23.md`](../../../../harness/prep/r-lawful-pointer-forced-copy-oracle-DRAFT-2026-10-23.md):
  - Updated the design note and pre-registration sheet to document the unified `N`/`O1`/`O2` margin law, the 9 finite unit tests, the wrong-span distractor negative control (`W`), the total four-branch partition (`Branches A, B, C, D`), the 16-block paired cluster jackknife standard error $\text{SE}_{\Delta}$, the pinned seed list (`seeds = [1, 2, 3]`), the checkpoint `learned_gate_margin`/`route_score` prerequisite, and chat approval under Stop 1(d) (`#2895`).

## Measurements

- `CARGO_TARGET_DIR=/Users/casey.allard/uor-r4/local/antigravity/target cargo test -p uor-r4-training --lib lawful_pointer`: `9 passed; 0 failed`.
- `CARGO_TARGET_DIR=/Users/casey.allard/uor-r4/local/antigravity/target cargo check -p uor-r4-training --bins`: exit `0`.
- `cargo fmt --check -p uor-r4-training`: exit `0`.

## Checks run at the head

- `uor-harness loop report --check`: `PASS`.
- `uor-harness loop pr-check --local`: `PASS`.
- `uor-harness check --only harness,prep`: `0 error`.

## Skills used

- `uor-research-architect`, `scope-guard`, `solution-integrity`, `context-guard`.

## Still open

- Non-author `TEST FITNESS: FIT` review of [`docs/harness/prep/r-lawful-pointer-forced-copy-oracle-DRAFT-2026-10-23.md`](../../../../harness/prep/r-lawful-pointer-forced-copy-oracle-DRAFT-2026-10-23.md) (CHARTER §4.5), owner/PM registration of the `N` / `O1` / `O2` / `W` diagnostic bar on `#2880` (CHARTER §5.2), a bound discrete memory-answering checkpoint SHA-256 (seeds `1, 2, 3`), and Casey's approval in chat (`CHARTER §5.1`, `#2895`) before running the GPU forced-copy oracle evaluation.

## Plain summary

Repaired all four review findings on PR `#3086`: unified the `total_copy_margin > base_top_margin` comparison across `N`, `O1`, and `O2` so `O1 - N` isolates source addressing alone; enforced alphanumeric word boundaries in `locate_last_value_span`; expanded finite host unit tests from 6 to 9 to pin the base-margin comparison, unadmitted candidates, `SharedAtomGcd` rarity + latest-start tie-breaking, and alphanumeric boundaries; and updated the pre-registration sheet with a total 4-branch partition, wrong-span control `W`, paired cluster SE, pinned seeds `1, 2, 3`, and chat approval.

References #2880
