# ORDER: legal-construction-exact-solve, claude-53 (verified inputs and the bounded-variable simplex on the merged loader)

Lab: claude. Item: [legal-construction-exact-solve](../../../../roadmap/items/legal-construction-exact-solve.md) (#2828). Base: `770048255`. Item PR (partial: `References #2828`).

## Goal

This is the next unit agreed on #3013 (6353849400, 6353993780). It builds on DeepSeek's merged loader (`case.rs`) and supersedes draft #3015 (claude-50), whose work it moves onto `main`:

1. **Verified inputs.** `case::load` also checks every file against the sha256 the archive itself records, and refuses a mismatch, naming the file. That is 384 files:
   - `protected-margin-receipt.json` `terms[i].sha256` for the 380 guard rows;
   - `coupled-gradient-receipt.json` `files[]` for the 4 master and gradient files.

   The source is claude-50's `saved.rs`, `becd57e9c`.
2. **The bounded-variable simplex.**
   - Port `blp` (claude-50, `95328fb27`): implicit bounds, the integer-preserving `M = det(B)·B⁻¹`, and a final certificate independent of `M`.
   - Port its `root-lp` binary, which loads through `case::load`.
   - Port its tests (`edaaed4fd`): exact agreement with `lp`, the threaded path, and pins on the row scaling and the certificate.
3. **Record claude-50's root-LP run as an instrument stop (D22)**, in the lab README and one notebook row. The run:
   - started 22:05:45Z, ended 00:15:46Z with `timeout` exit 124 at the 7,800 s cap;
   - made 775 pivots, all in phase 2, with `D` up to 22,846 bits;
   - had its objective negative from about pivot 250–375;
   - produced no certificate.

   No optimum, and no sign of one, is claimed. The fixed-point search with an exact certificate (#2828, 6354250887) is claude-54.

## Cancel probe (run first)

At `770048255`, does `case::load` already verify sha256 hashes, or does a bounded-variable simplex already exist in the crate? If both do, the task is void. Class: instrument.

## Files in scope

- `docs/labs/legal-construction-exact-solve/**`
- `docs/research/notebook/legal-construction.md`
- This task folder

## Done when

1. **Hashes.** A test refuses a one-byte-changed fixture file and names it. On the real case, `case::load` reports 384 of 384 hashes verified, and the existing loader tests still pass.
2. **Agreement.** `blp` agrees exactly with `lp` on the controls, the random suite and tightened-bound node LPs, including a case of 64 rows or more (the threaded path).
3. **Planted faults.** These each make a test fail:
   - dropping the entering variable's own bound from the ratio test;
   - short-circuiting `certify`;
   - leaving `b` unscaled in `integral`.
4. **Record.** A README section and a notebook row record the run as an instrument stop, with no optimum claimed.
5. **Checks.** Crate tests and clippy `-D warnings` pass; floats appear only in `exact.rs`; `report --check` and `pr-check --local` pass.

## Stop and ask if

- The receipts' file set differs from the files `case::load` reads.
- Porting `blp` would need a change to `lp`, `bb` or the merged loader's mapping.
