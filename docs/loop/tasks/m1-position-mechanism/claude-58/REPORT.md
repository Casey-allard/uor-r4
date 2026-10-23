# REPORT: m1-position-mechanism, claude-58 (freeze the order-sensitive DEV slice and pin its hash)

Lab: claude. Order: [ORDER.md](ORDER.md). Item #2843 (`References`; the item stays `ready`). Base: `7f0f0710c`.

## What did not work, first

- **The sheet still clears nothing.** Freezing the slice removes one data gap. A run still needs:
  - a non-author FIT, which now has to be Kimi's, because Claude has authored sheet text;
  - `m1-trunk-assembly-and-training` (held);
  - the position mechanism's GPU wiring;
  - PM dispatch and Casey's release.
- **"Pairwise distinct" admits two readings, and N doubles between them.** The literal reading, where every successor differs from every other, selects **1,620** rows. The looser one, at least two distinct successors with repeats allowed, would select 3,277. I froze the literal reading, as the sheet words it, and recorded the other as a diagnostic only. If the PM wants the looser slice, it is a new file with a new hash.
- **Clippy `-D warnings` on `-p uor-r4-discrete --all-targets` fails, but in other crates.** `uor-r4-integer` and `uor-r4-source-interface` hit lints such as `manual is_multiple_of` and `needless_range_loop`, which local clippy applies to workspace dependencies. Both crates are untouched here: `git diff origin/main` on them is empty. Clippy reports 0 hits in this unit's two files.
- **`cargo fmt -p uor-r4-discrete` also reformats an unrelated test file,** `tests/flock_coupling_controls.rs`, which is already unformatted on main. I reverted that, and committed only this unit's own one-line reflow.
- **The setting-aside step tripped once.** My first negative-gate restore used `git checkout` on a file that wasn't committed yet, so it couldn't restore it. I put the line back by hand and re-ran the tests (6 passed) before committing.

## What changed

| File | Change |
|---|---|
| `crates/uor-r4-discrete/src/order_slice.rs` | New. `is_order_sensitive` (§5's rule), `has_two_distinct_successors` (the diagnostic), `select` over every eligible row through `PreparedCorpus::batch` at C192, `render`, and 6 tests. |
| `crates/uor-r4-discrete/src/bin/order-slice-freeze.rs`, `Cargo.toml`, `src/lib.rs` | New binary. It loads the DEV split only under the registered digests and inventory, and writes the frozen file exclusively. |
| `docs/labs/m1-position-mechanism/order-slice-v1.tsv` | The frozen subset: a header plus 1,620 rows, 17,789 bytes. |
| `docs/labs/m1-position-mechanism/README.md` | §4: the rule, the count, the hash, the input, the reproduce commands and the tests. |
| `docs/harness/prep/m1-position-mechanism-DRAFT-2026-10-22.md` | §5's slice line now carries the file, the SHA-256, N and the reproduce command. The Status line and its sub-bullet record the freeze and that the FIT must now come from Kimi. Nothing else changed. |
| this folder | The ORDER (committed first, alone) and this REPORT. |

## Measurements

- **Input:** `data/ft-dev.tar` from the HF store.
  - The archive is 143,360 bytes, with sha256 `45e93e32…0390b5c`, equal to the pin.
  - Inside it, `tokens.u16` is 85,776 bytes (sha256 `0919e90f…`) and `response_mask.u8` is 42,856 bytes (`a838bb1a…`). Both equal the registered digests, and `PreparedCorpus::load` re-checks them.
- **Freeze:** scanned 14,287 rows and selected **1,620**, with 3,277 under the looser reading.
  - The file is 17,789 bytes with SHA-256 **`f76dc5c7ff6c359dd4c9158abc280ff29105cb611f6c4c2bdba6fdfed0035b3a`**.
  - A second run to a scratch path was byte-identical (`cmp`).
- **Tests:**
  - `cargo test -p uor-r4-discrete --lib order_slice`: 6 passed.
  - The whole lib suite: 217 passed, 0 failed, 1 ignored. That is 211 before this unit plus the 6 new tests.
- **Negative gate:** with the distinctness check dropped (`occurrences >= 2` alone), 3 of the 6 tests fail: the repeated successor, the occurrence just before the query, and three occurrences. With the check restored, all 6 pass.

## Checks run at the head

The crate's lib suite, clippy on the new files, rustfmt on the new files, `loop report --check --order` and `loop pr-check --local`.

## Skills used

- `solution-integrity`: the slice comes from the pinned bytes through the crate's own window builder, not from a re-derived window. The reading that changes N is stated, not hidden.
- `scope-guard`: no bar, rule or other pin changed. Unrelated reformatting and other crates' lints were left alone.

## Still open

- Kimi's non-author FIT of the position sheet.
- If the PM prefers the looser reading of "pairwise distinct", a new frozen file.
- The position mechanism's GPU wiring and the assembled trunk (#2844) before any run.

## Plain summary

- The position test's special DEV slice is now a fixed, checkable file: 1,620 rows where a repeated word was followed by a different word each earlier time. Anyone can rebuild it byte for byte from the stored data.
- That removes one blocker from the position sheet. It does not start any training.

References #2843
