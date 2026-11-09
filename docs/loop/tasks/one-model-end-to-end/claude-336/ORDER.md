# ORDER: one-model-end-to-end, claude-336

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `1e5e84a1d`.

## Goal

**Serve the ask heads and the question carry in the integer engine.** [claude-335](../claude-335/REPORT.md) passed B2 at both seeds with the question carry: 466–512 of 512 per far bucket, at most 54 under `−C`. The sheet's branch 1 names the next step: serve the carry at the decode step and in the export. Today the D11 export refuses both, by name (`fbs_export.rs:398` and `:416`, and `round_hamming_as_exported` `:535` and `:552`). So no ask-head model, carried or not, reaches the integer session.

What the ask heads compute, read from the float forward (`flock_read_hamming.rs:1612–1660`, `:1809–1917`):
- **The question** of a type head at row `t` is the key rotation's sign code of `table[x_t] + a[x_t]`, plus, at the readout read with the carry, `Σ_{s=1..h} c[x_{t−s}] · (table + a)[x_{t−s}]` over rows of the same window.
  - Without ask heads, the question comes from the residual through the query rotation.
  - `table` is the shared token table, `a` the learned question table `(vocab, width)`, and `c` the per-token carry gain `(vocab,)`.
- **The keys** (`key_rotation(table[x_{s−1}])`), the value payload (`value_rotation(residual)`), the margins and the copy heads are as today.
- So a step needs only the last `h` token ids of its window and the two tables.

1. **The export:** write `a` and `c` into the Hamming read's image, behind a new schema version, and stop refusing them. Rounding (`round_hamming_as_exported`) rounds them as the export stores them. A read without them exports byte-identically to today.
2. **The integer read:** `TypeHeads` takes the ask table and, at the readout read, the carry. The question is the sign code of the exact integer sum. The float step's `from > 0` refusal of the carry stays: the float cached step is out of scope, and float decodes a carried read uncached.
3. **Tests, each with a mutation that fails it:**
   - the integer ask heads match an f64 reference of the float question on a synthetic read, with and without the carry;
   - a carried term reaches exactly the next `h` rows of its window and no further;
   - a read without `a` and `c` is byte-identical in export and decode, and the existing `lut-stack/6` round trips still pass;
   - a malformed `a` or `c` (wrong shape or length) is refused.
4. **Parity on a trained model:** export claude-335's sealed B2-aqC checkpoint at seed 1. Then compare the float Hamming trace with the integer session's on B2 windows, with `fbs-snapshot --hamming-trace` and `fbs-compare`. This is a host check on the laptop, not a training or evaluation run: a few windows, no bar.

**Claim check:**
- **(a) Step:** the run-claude order's step 2 (the model's parts served in integers) and step 4 (one model end to end), for the recall gap (11).
- **(b) What it changes:** the first read that passes B2 can be exported and served on a device. That is required before a language run with the carry is worth training, because a trained carry that no integer session can serve is not the product.

**Not in scope, named here:** the float cached decode step for the carry (float decodes it uncached today); a language run with the carry (its own sheet); the hold (closed by claude-334).

## Cancel probe (run first)

1. No open PR serves the ask heads or the carry, or touches the integer Hamming read.
2. claude-335's code, ORDER and REPORT are on `main` at the base.
3. **Compute:** the laptop CPU only. Builds, finite tests and the parity host check need no FIT (CHARTER §4 item 5).
4. **Disk:** at least 5 GiB free; one target dir in this worktree, removed on merge.

**Result (instrument, at claim):** pass.
- Open PRs: #3889 (DeepSeek, `crates/uor-r4-cpu-train` only) and #3890 (Kimi, workbench) touch none of these files.
- claude-335 merged as `1e5e84a1d497222099a97e22da024b07759750a8` and was verified on fresh `main`.
- 18 GiB free.
- **`loop claim` refused** on "scope Cargo.lock overlaps Cargo.lock of item cpu-training-pilot (PR #3889)". The item's scope lists `Cargo.lock`, and so does DeepSeek's. This unit adds no dependency and does not touch `Cargo.lock`, so no file overlaps. The ORDER is written by hand under Casey's harness exception ("i approve you to make harness exceptions as needed to do what I ask"), as claude-331 to claude-335 were.

## Files in scope

- `crates/uor-r4-float-train/src/fbs_export.rs`
- `crates/uor-r4-float-train/src/flock_read_hamming.rs`
- `crates/uor-r4-integer/src/stack/hamming.rs`
- `crates/uor-r4-integer/src/stack/format.rs`
- `crates/uor-r4-integer/src/stack/tests.rs`
- `crates/uor-r4-integer/src/stack/synthetic.rs`
- `crates/uor-r4-integer/src/fbs_snapshot.rs`
- **Added at claim (lab decision, logged here):** `crates/uor-r4-float-train/src/bin/fbs-snapshot.rs`, `crates/uor-r4-integer/src/stack/mod.rs` and `crates/uor-r4-integer/src/stack/session.rs`, only if the read's image cannot reach the session without them. The item's file gains them in this PR. `integer-hamming-read`, which held them, is done; `turn-memory-integer` and `point-key-integer` list `session.rs` and are not claimed.
- `docs/loop/tasks/one-model-end-to-end/**`, `docs/research/notebook/float-training.md`, `docs/roadmap/items/one-model-end-to-end.md`
- **Added in the unit (lab decision, logged here and in the [REPORT](REPORT.md)):**
  - `crates/uor-r4-integer/src/bin/fbs-export.rs`, the command that turns the snapshot into an artifact. It reads the ask block under its own JSON key.
  - Of the conditional files above, `bin/fbs-snapshot.rs` and `stack/mod.rs` were used and `stack/session.rs` was not.
  - The item's scope gains the three files used.

## Done when

1. The export writes `a` and `c`, and the integer engine serves them. Both are merged and verified on `main`. Each test passes, and its mutation fails.
2. Every existing `uor-r4-integer` and `uor-r4-float-train` library test passes at the head.
3. The parity host check on claude-335's B2-aqC is recorded: the windows compared and the first differing decision, if any. A mismatch is reported as a finding of the instrument or of the rounding, with its trace row, not hidden.
4. A fresh exact-head review passes the PR, which merges and is verified on `main`.

## Stop and ask if

Nothing waits on Casey (§5.1). Bound: one unit for the code and the parity check. If parity needs a decode-side change outside these files, the REPORT names it, and the next unit takes it.
