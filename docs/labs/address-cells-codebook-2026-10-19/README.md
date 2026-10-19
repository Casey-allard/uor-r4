# Address-structured injective codebook: constructor and exporter (source piece (a)) — 2026-10-19

- **Lab:** Claude.
- **Sheet:** `docs/harness/prep/m1-attention-address-structured-cells-DRAFT-2026-10-19.md` (DRAFT, #2738); this is its source piece (a).
- **Owner decision:** Claude chat, 2026-10-19.
- **Status:** source plus a host data-preparation export. **No model compute.** No admission was estimated on the host (D24).

## What landed

- **`src/address_cells.rs`**:
  - `BigramCounts::count` counts TRAIN `(s → t)` at the driver's eligible positions, with `s = tokens[p − 1]` (the window rule `tokens[start..p)`, `corpus.rs:270-278`).
  - `build` runs the sheet's greedy:
    - descending frequency, ties to the lower token id;
    - each token takes the cell minimising `Σ count · L1`, with L1 on the writer's root coordinates for blocks 0 and B−1;
    - a quota of V / 256 per cell, cost ties to the lowest cell id;
    - zero placed-predecessor mass goes to the PrimeZeta cell, else the nearest non-full cell by (L1, id);
    - inner blocks take the rank within the cell, so every word is an orbit root;
    - an unconditional recount requires distinct == V and every cell exactly at quota.
  - `ShuffledV1 { seed }` is the S8 control: a seeded bijective relabel of predecessors, with frequency order, quota and mass unchanged. The seed is recorded in the mode label.
- **`src/bin/address-cells-codebook.rs`** (`CONFIG OUTPUT [structured|shuffled:SEED]`):
  - loads **TRAIN only**, through the config's own FilePins; DEV is never read;
  - writes the `uor-r4.codebook-table/1` JSON exclusively, and only after a successful build;
  - prints the table, file and counts SHAs.
- **`src/corpus.rs`**: two read-only accessors, `eligible_positions()` and `tokens()`. `corpus.rs` is in `driver_source_sha256`'s include list, so **the driver hash changes**. No behaviour changes.

## Real-data export (host data preparation)

Input: the TRAIN pins `3dee858f…` / `b5e7352d…` from the injective treatment config. The export ran on the laptop in about 0.5 s per table.

| Table | Mode | `table_sha256` | file SHA-256 | Distinct | Occupied |
|---|---|---|---|---:|---:|
| A8 | `address_structured_v1` | `4122cbab44bd88658743984fc1ab79059bf032c3e2fe526ffcf4ea5f95d9cd74` | `6600bdb1306dbaea4d77f5bf021e717a6bef791e44c1ec7d7c9bb9f0e8cd3cd1` | 4,096 | 256 |
| S8 | `address_shuffled_v1_seed_20261019` | `d2fb8680a16e8017f6f0e6518df6810f6206023b0d14388dc076d4cfde2ebc5b` | `a185bdca07077fe7157e947ff1fa80919e6f99165a4dd5a04061a97873c92718` | 4,096 | 256 |

- **Counts SHA:** `755a5a81ddbf0f9d81b4d0bd88e54435b96f5485f87ad92a22d235651fe6ca44`, shared by both tables, since the shuffle acts in placement.
- **Determinism:** a structured rebuild is byte-identical, and an existing output is refused.
- **On the pod:** the pre-flight rebuilds both tables and must reproduce these file SHAs.

## Checks

- **Tests:** `uor-r4-discrete` lib tests pass 140/140. The `address_cells` tests (8) cover injectivity, quota and roots, determinism, the zero-mass rule and the predecessor pull, the shuffled control, predecessor direction, two-block refusal, a bad vocabulary, and the table round-trip through `CodebookTable::load`.
- **Negative gates:**
  - removing the quota check fails 4 tests;
  - counting `tokens[p + 1]` fails the direction test;
  - restoring the old `blocks > 2` guard fails the two-block test.
- **Lint and scans:** fmt and clippy are clean on the new files, and `check_no_softmax.py` passes.

## Review

A fresh-context reviewer subagent returned PASS-WITH-FIXES and found no D24 issue. It confirmed:
- the counts use the immediate predecessor;
- the L1 metric equals the writer's `address_distance`;
- the greedy, quota and zero-mass rules hold;
- sums are computed in u128;
- the shuffle is bijective.

Both of its defects are fixed: the injectivity recount was not run at B = 2, and the exporter created its output before building.

## Piece (b) and next

Piece (b) is expected to need no code. The loader checks no `mode` string, the attention scope keys on the table file's SHA prefix, and `validate_injective_table` requires only distinct == V. The host build refuses all model commands (D24), so this is **confirmed only at the pod pre-flight**, through `attention-build-identity`.

**Next:** piece (c), a writer K = 16 evaluation path for the P16 / A16 bracket. Then FIT, PM dispatch, and the zero-update pre-flight (about $0.40).
