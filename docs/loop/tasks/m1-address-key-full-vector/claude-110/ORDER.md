# ORDER: m1-address-key-full-vector, claude-110

Lab: claude. Item: [m1-address-key-full-vector](../../../../roadmap/items/m1-address-key-full-vector.md) (#2834). Base: `616384cac`.

## Goal

The item's done-when reads:

> Stage 1 bar: "enough distinct vectors that the number of positions still winnable clears the run bar of 1,941, and a starting reach above the address-cell model's 1,351 of 3,847", measured before any training. The audit also lists within-cell tie mass. The bar is not G-Pre of 4,894 at checkpoint 0, which every K16 arm already met.

antigravity-37 and -38 (#3070, merged as `b8aa3aefd`) delivered:
- the host metric, `AddressCellsMode::FullVectorV1`, `cell_envelope_l1` and `full_vector_cell_l1` in `address_cells.rs`;
- 11 finite CPU controls;
- the draft sheet `docs/harness/prep/m1-address-key-full-vector-DRAFT-2026-10-23.md` (TEST FITNESS: PENDING).

Their open item is this unit:
1. **The kernel.** An opt-in writer address-metric dispatch in `discrete.cu`: `outer_l1_v1` (default, byte for byte), `cell_envelope_l1_v1` and `full_vector_cell_l1_v1`.
   - The per-cell distances are built once per row from one pass over the vocabulary (`env[c] = min`, `sum[c] += token_distance`).
   - The host config field serializes only when it is not the default, so every existing config and lineage hash stays the same.
   - Update capabilities are refused for a non-default metric, because the sheet registers zero updates only.
2. **Finite CUDA qualification controls:** host and device give the same per-cell distances and admitted cells on fixtures, and the outer default is unchanged.
3. **The sheet, finalized:**
   - its registration line follows D67 (the lab's own pre-run review), not the retired owner/PM registration;
   - then a non-author FIT;
   - then the D67 pre-run review.
4. **The zero-update GPU pre-flight** (5 arms, about $0.40), graded against the sheet's G-Exact, G-Occ/G-Cap and starting-reach bars. Then the REPORT, the notebook row and the item status.

## Cancel probe (run first)

Run at `616384cac`. Class: records and instrument.

| Check | Result |
|---|---|
| The host metric and its controls are merged: `cargo test -p uor-r4-discrete --lib address_cells` | **Holds:** 11 passed, 0 failed |
| `discrete.cu` has no full-vector dispatch: `address_distance` (l.328) is the two-block outer key, used only in `writer` (l.369, 380, 385) | **Holds** (grep) |
| No other open order or PR holds the kernel or the sheet | **Holds:** `loop claim` passed |
| The draft sheet is unchanged since its author unit | **Holds:** one commit, `b8aa3aefd` (#3070) |

## Files in scope

- `crates/uor-r4-discrete/**` (the kernel dispatch, the config field, the qualification controls)
- `docs/harness/prep/m1-address-key-full-vector*`
- `docs/labs/m1-address-key-full-vector/**`
- `docs/mechanisms/discrete-learning.md`
- this task folder, and the item's notebook row

## Done when

- The dispatch and its controls are built, with tests that fail without them, and pass a fresh exact-head review.
- The finalized sheet has a non-author FIT and a passing D67 pre-run review.
- The 5 arms run, with the A8-Outer exactness gate first.
- The outcome is graded against the sheet's bars on fresh-context-reviewed claims, with one notebook row.
- If the Stage 1 bar is met, the item is set `done` and the REPORT says `Closes #2834`. A negative closes the configuration only (D22) and routes as the sheet says, to `m1-wider-code-carrier` (#2847).

## Stop and ask if

Stop and decide, never stop and wait.

- A8-Outer fails to reproduce 4,244 / 1,351 / 133, or host and device distances differ: that is an instrument fault (D31). Fix it, and read no treatment number first.
- The kernel cost projects past the sheet's 30-minute ceiling.
