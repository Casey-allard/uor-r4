# Lab Note: `m1-faster-writer` — Bucketed CSR CUDA Writer Qualification (`#2840`)

- **Roadmap item:** [`docs/roadmap/items/m1-faster-writer.md`](../../roadmap/items/m1-faster-writer.md) (`#2840`, Milestone M1 `#2029`)
- **Pre-registration sheet:** [`docs/harness/prep/m1-faster-writer-DRAFT-2026-10-22.md`](../../harness/prep/m1-faster-writer-DRAFT-2026-10-22.md)
- **Finite host controls:** [`crates/uor-r4-discrete/tests/writer_bucket_controls.rs`](../../../crates/uor-r4-discrete/tests/writer_bucket_controls.rs) (`e10d10926`, PR `#2983`)
- **Current status:** `TEST FITNESS: NOT_FIT` (non-author audit recorded by `antigravity` in `antigravity-35`; item stays `status: ready`)

## Units Delivered So Far

1. **Unit 1 (`claude-34`, PR `#2983`, `e10d10926`):**
   - Posted the geometric design note on `#2840` (`6346787358`): a CSR bucket index (`offsets[0..=256]`, `members[0..V)`, base `cells[0..V)`, `absent` count) over the 256 root-pair address cells (`cell = root_id(block 0) * 16 + root_id(block B-1)`), one-pass 256-cell distance evaluation + counting-sort admission, and id-ordered bucket membership walk.
   - Committed 6 finite host controls in `crates/uor-r4-discrete/tests/writer_bucket_controls.rs` proving exact match against the full-scan reference across a 300-seed sweep, cell-id and token-id ties, `ABSENT` tokens, `kind == 1` virtual moves, and per-row operation counters (`token_cell_calls == 0` on `NO_MOVE`, `1` on `kind == 1`; `cell_distance_evals == 256`).
2. **Unit 2 (`claude-36`, PR `#2989`, `47d781a49`):**
   - Drafted `docs/harness/prep/m1-faster-writer-DRAFT-2026-10-22.md` specifying the finite CUDA qualification, full-output `B1` parity check, `B2` operation-count bar (`PROPOSED, NOT REGISTERED`), dynamic CSR index update verification across 64 scripted edits and 64 real `discrete_counterfactual` → `discrete_apply` steps, and 4 planted negative controls.
3. **Non-Author `TEST FITNESS` Audit (`antigravity-35`):**
   - Independently verified all `discrete.cu` line numbers (`:20-34`, `:270-496`), call-count formulas (`V + 1 + A*V = 69,633` at `V = 4,096, A = K = 16`), SHA-256 digests (`6600bdb1…` for `A8`, `83c1380a…` for `PrimeZeta`), and the 4 planted faults.
   - Recorded `TEST FITNESS: NOT_FIT - M1 #2029 m1-faster-writer` with reviewer recommendations on the two open choices in §5 and the 3 prerequisites before `FIT`:
     1. Owner/PM registration of the §3 `B1`/`B2` bar with an ISO-UTC timestamp (CHARTER §5.1, §5.2).
     2. Pinning §5's two open choices into one single configuration (recommended: seeded `V = 16,384` table at seeds 1, 2, 3, plus the 14,287-row DEV real-read panel with full DEV token/mask SHA-256 pins `0919e90f953e…` / `a838bb1a67e9…` on both `A8` `6600bdb1…` and `PrimeZeta` `83c1380a…`).
     3. Listing the exact admitted PTX `.entry` inventory and `writer-bucket-qualify` CLI argv in §6, and stating in §3 whether `ad = address_distance(q, g, s, target_cell)` at `discrete.cu:326` reuses the admission distance array or is counted outside the 256-call admission bar.
