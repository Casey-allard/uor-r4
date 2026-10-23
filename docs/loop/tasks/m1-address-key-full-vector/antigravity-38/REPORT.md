# REPORT: m1-address-key-full-vector, antigravity-38

Lab: antigravity. Item: [m1-address-key-full-vector](../../../../roadmap/items/m1-address-key-full-vector.md).

## What did not work

- **D23 ratchet trigger in a doc comment (`crates/uor-r4-discrete/src/address_cells.rs:156`):** As `deepseek` and `claude` flagged in their `CHANGES` reviews on PR `#3070` (`250a8ed576df9a302a25c231a408dc3fadbc8728`), the new doc comment on `cell_envelope_l1` contained the D23 banned family word in prose, causing `uor-harness ratchet d23` to report `crates/uor-r4-discrete/src/address_cells.rs (0 -> 1)` and failing `CI / harness rules and tests`.
- **Broken external-path markdown link in `antigravity-38/REPORT.md:32`:** At `5cc576c15db7e398765838677e68c6a323d25e3b`, `CI / harness rules and tests` turned green, but `docs/docs archive and link integrity` (`xtask links`) caught a relative link to `../../../../../../.gemini/config/skills/scope-guard/SKILL.md` (which sits outside the repository root on CI runners). Removed the out-of-repo relative link.
- **No GPU execution or capability claim:** No GPU run or baseline ratchet change was performed. Item `m1-address-key-full-vector` stays `status: ready`. References #2834, #2029.

## What changed

- `crates/uor-r4-discrete/src/address_cells.rs` (lines 118–250):
  - Reworded the doc comment on `cell_envelope_l1` (`/// Uses only integer subtraction, absolute value, addition, and minimum.`) so no D23 banned token appears in `.rs` source.
  - Replaced `(x - ...).unsigned_abs()` with overflow-free `x.abs_diff(...)` in `block_l1` and `token_full_l1` (per Claude's non-blocking finding #3).
  - Updated the doc comment on `latin_interior_anchor` to describe it accurately as a deterministic starting interior root pair with per-cell uniqueness enforced by `used_interior`, and added explicit D24 host-reference-oracle doc notes on `cell_outer_l1`, `token_full_l1`, `cell_envelope_l1`, `full_vector_cell_l1`, and `admit_cells_by_distances` (per Claude's non-blocking findings #2 and #4).
- `docs/loop/tasks/m1-address-key-full-vector/antigravity-38/{ORDER,REPORT}.md`: Added the repair order and report for `antigravity-38` with repo-internal links only.

## Measurements

- **`uor-harness ratchet d23`:** `OK: no softmax added. 4451 remaining across 165 files.` (`crates/uor-r4-discrete/src/address_cells.rs` stays at `0`).
- **`uor-harness ratchet d25` / `d32`:** `OK: no dense matrix product call sites added.` / `OK: no discrete-path floating-point sites added.`
- **Unit tests (`cargo test -p uor-r4-discrete --lib address_cells`):** `11 passed; 0 failed`.

## Checks run at the head

- `uor-harness ratchet d23`, `uor-harness ratchet d25`, `uor-harness ratchet d32`: PASS.
- `~/.cargo/bin/cargo test -p uor-r4-discrete --lib address_cells`: `11 passed; 0 failed`.
- `uor-harness loop report --check`: PASS.
- `uor-harness loop pr-check --local`: PASS.
- `uor-harness loop gates --full`: PASS.

## Skills used

- `uor-research-architect` ([SKILL.md](../../../../../.agents/skills/uor-research-architect/SKILL.md)): Verified the D23, D25, and D32 ratchets and overflow-free `abs_diff` integer arithmetic on the branch.
- `scope-guard`: Kept the repair to `crates/uor-r4-discrete/src/address_cells.rs` and the `antigravity-38` task folder.

## Still open

- CUDA kernel dispatch for `cell_envelope_l1_v1` / `full_vector_cell_l1_v1` in `crates/uor-r4-discrete/src/discrete.cu` and finite CUDA qualification on a GPU pod, plus non-author `TEST FITNESS: FIT` and owner/PM registration on `#2834` for `docs/harness/prep/m1-address-key-full-vector-DRAFT-2026-10-23.md`. Item `m1-address-key-full-vector` stays `status: ready`. References #2834, #2029.

## Plain summary

- Reworded the `cell_envelope_l1` doc comment at `crates/uor-r4-discrete/src/address_cells.rs:156` to pass the D23 ratchet, switched `block_l1` and `token_full_l1` to overflow-free `abs_diff`, clarified the `latin_interior_anchor` and D24 reference-oracle doc comments, and removed the out-of-repo markdown link in `antigravity-38/REPORT.md` so both `harness rules and tests` and `docs archive and link integrity` pass.
