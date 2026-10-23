# REPORT: m1-faster-writer, antigravity-35

Lab: antigravity. Item: [`m1-faster-writer`](../../../../roadmap/items/m1-faster-writer.md) (#2840). Milestone: M1 (#2029).

## What did not work, first

- **`docs/harness/prep/m1-faster-writer-DRAFT-2026-10-22.md` cannot receive `TEST FITNESS: FIT` yet because its §3 bar is `PROPOSED, NOT REGISTERED` and §5 leaves two open design choices (`TEST FITNESS: NOT_FIT`, configuration / pre-registration gap, not a mechanism negative):**
  1. **Unregistered bar (§3):** §3 is explicitly marked `## 3. Bar: PROPOSED, NOT REGISTERED` ("Casey registers the bar"). Under CHARTER §5.1 and §5.2, a reviewer cannot register a bar; the owner/PM must register the §3 `B1`/`B2` bar with an ISO-UTC timestamp and §3 must quote that registration.
  2. **Unresolved open choices in §5:** Under D22, a `FIT` verdict clears one single, unambiguous configuration. §5 currently leaves two open choices for the reviewer and owner (plus the optional `PrimeZeta` table):
     - Choice 1: seeded `V = 16,384` table (seeds 1, 2, 3) vs. waiting for a 16,384-piece table built under its own item (`#2872`).
     - Choice 2: including the 14,287-row DEV real-read panel (`q` from `read_vector` on pinned `A8`) vs. qualifying on seeded `random_query` rows only (plus whether to include `PrimeZeta` `83c1380a…`).
  3. **Concrete PTX `.entry` inventory and CLI wiring in §6 (plus `:326` target `ad` note in §3):** §6 names `src/bucket_source.rs` and `src/bin/writer-bucket-qualify.rs` (to be built in Unit 3) without listing the exact admitted PTX `.entry` symbol names or CLI invocation, and §3 should state whether `ad = address_distance(q, g, s, target_cell)` at `discrete.cu:326` reuses the admission pass's `distance[target_cell]` array or is counted outside the 256-call admission bar alongside `:330-333`.

## What changed

- Audited [`docs/harness/prep/m1-faster-writer-DRAFT-2026-10-22.md`](../../../../harness/prep/m1-faster-writer-DRAFT-2026-10-22.md) via a fresh-context non-author reviewer subagent against `crates/uor-r4-discrete/src/discrete.cu`, `crates/uor-r4-discrete/tests/writer_bucket_controls.rs`, `crates/uor-r4-discrete/src/attention_learning.rs`, `crates/uor-r4-discrete/src/measurement.rs`, and `crates/uor-r4-discrete/build.rs`.
- Recorded `TEST FITNESS: NOT_FIT - M1 #2029 m1-faster-writer` in `docs/harness/prep/m1-faster-writer-DRAFT-2026-10-22.md` along with the reviewer recommendations on §5's two open choices (pinning seeded `V = 16,384` at seeds 1–3 plus the 14,287-row DEV real-read panel with full DEV token/mask SHA-256 digests `0919e90f953e…` / `a838bb1a67e9…` on both `A8` `6600bdb1…` and `PrimeZeta` `83c1380a…`).
- Created [`docs/labs/m1-faster-writer/README.md`](../../../../labs/m1-faster-writer/README.md) summarizing Units 1–2 and the non-author `TEST FITNESS` review.

## Measurements

- **Verified source citations & counts (`crates/uor-r4-discrete/src/discrete.cu`):**
  - `writer` (`:307-345`): occupancy scan `:309` (`V` calls to `token_cell`), target cell `:311` (`+1` call to `token_cell`), admission & membership loop `:313-325` (up to `K * 256` calls to `address_distance` and `A * V` calls to `token_cell`), total `token_cell` calls in `writer` = `V + 1 + A*V = 69,633` at `V = 4,096, A = K = 16`.
  - `discrete_export_token_cells` (`:367-372`): early return on `ABSENT` at `:370` without writing `cells[token]`.
  - `discrete_evaluate` (`:469-496`): calls `writer` at `:474` and runs a second `O(V)` `token_cell` + `token_distance` top-8 scan at `:479-489` with status-11 check at `:490`.
- **Verified table & parameter pins:**
  - `A8_TABLE_FILE_SHA256` (`attention_learning.rs:33`) = `6600bdb1306dbaea4d77f5bf021e717a6bef791e44c1ec7d7c9bb9f0e8cd3cd1`.
  - `PRIME_ZETA_TABLE_FILE_SHA256` (`attention_learning.rs:31`) = `83c1380a8f507e51d0342faa1b089dea6a20561a03552208c093fa640afc124b`.
  - `vector_parameters` (`measurement.rs:213`) = `16384` (`4,096 * 4`).
- **Finite host controls (`crates/uor-r4-discrete/tests/writer_bucket_controls.rs`):** 6/6 tests pass (`token_cell_calls == 0` on `NO_MOVE`, `1` on `kind == 1`; `cell_distance_evals == 256`).

## Checks run at the head

- `~/.cargo/bin/cargo test -p uor-r4-discrete --test writer_bucket_controls` (6 passed).
- `uor-harness loop report --check --file docs/loop/tasks/m1-faster-writer/antigravity-35/REPORT.md`
- `uor-harness loop pr-check --local`
- `uor-harness loop gates --full`

## Skills used

- `uor-research-architect`, `solution-integrity`, `scope-guard`.

## Still open

- Owner/PM registration of the §3 `B1`/`B2` bar with an ISO-UTC timestamp on `#2840`, plus pinning §§3, 5, and 6 in `docs/harness/prep/m1-faster-writer-DRAFT-2026-10-22.md` before re-requesting `TEST FITNESS: FIT` and building Unit 3. Item `m1-faster-writer` stays `status: ready`. References #2840, #2029.

## Plain summary

I audited the pre-registration sheet for the faster bucketed CUDA writer (`m1-faster-writer`, `#2840`), verified that all of its kernel line citations, comparison-count formulas, table hashes, and 4 planted fault controls match the repository code, and recorded a `TEST FITNESS: NOT_FIT` review with reviewer recommendations on the two open panel choices so that once the owner registers the bar and those choices are pinned, the sheet can be cleared for the GPU qualification run.
