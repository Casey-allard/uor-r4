# REPORT: one-model-end-to-end, claude-267 (the slot key measured on the data: right less often than copy key 2 on DEV and p-w, so step 2 does not run; copy key 2 backed off to copy key 1 is right most often)

Lab: claude. [ORDER](./ORDER.md). PR #3623, item #3407.

**In one paragraph.** The slot key keys a copy head on the token and its clause's rarest token. Measured on the data before any training, by the ORDER's exact-match count, it is right on fewer copyable reply positions than copy key 2: 2,136 against 3,506 on DEV, and 5,834 against 10,971 on p-w. Its purity (right among positions with an exact match) is 0.600 and 0.564, no better than copy key 1's 0.612 and 0.588. So the rule fails on both panels and step 2 (the slot key in training, export and serving) does not run. The instrument was checked before the negative was called: the boundary set is the ORDER's, clauses average 6.5 tokens on DEV, and the atom tests pass. Three diagnostics, added after the slot result and outside the rule (ORDER, Amendment), explain it and choose the next unit. The ORDER's named next rule, the two rarest atoms, is worse still. A suffix-local atom, the rarest of the last four tokens, recovers copy key 2's purity (0.794, 0.772) but not its count. Copy key 2 backed off to copy key 1 is right on 4,388 (DEV) and 13,990 (p-w) positions, 13–14% more than copy key 1, the trained base, at the same coverage. This is a key-quality reading of the data, not a model result.

## Tokens per second first

No model ran in this unit, so there is no model tok/s. The count is an instrument: one pass over DEV (428 windows) takes about 2.0 s on the laptop in a debug build, and p-w (1,389 windows) a few seconds.

## What did not work

1. **The slot key fails the rule on both panels** (a negative for this atom rule, a configuration, CHARTER §4.1). Right: 2,136 vs copy key 2's 3,506 on DEV; 5,834 vs 10,971 on p-w. It also has fewer exact matches than copy key 2 (3,560 vs 4,383; 10,345 vs 14,069) and copy key 1's purity, so the atom removes matches without making the remaining ones more often right.
2. **The ORDER's named next rule, the two rarest atoms unordered (`atom2`), is worse** (diagnostic): right on 1,413 and 3,693, purity 0.550 and 0.491. The unit it was to get is not needed.
3. **No atom rule beats copy key 2 on this count.** The suffix-local atom (`win4`) comes closest: purity 0.794 and 0.772 against copy key 2's 0.800 and 0.780, but right on 3,200 and 9,751 against 3,506 and 10,971.
4. **The mechanism, as far as the counts show it.** A copy aligns on suffixes: where a reply repeats a span from the middle of a source clause, its last tokens match the source's. The clause atom is a running minimum over the clause's *prefix*, and the reply's prefix is not the source's, so the atoms disagree at exactly the positions a copy needs. Two prefix atoms disagree more often than one. The window atom, which looks only at the suffix, gets back to copy key 2's purity. This reading is from the counts, not from a per-position audit.
5. **`cargo clippy -p uor-r4-float-train --all-targets -- -D warnings` fails, but not in this unit's files.** The 14 errors are in `uor-r4-integer` (a dependency this unit does not touch): `is_multiple_of`, loop-index and type-complexity lints from the current toolchain. Clippy without `-D warnings` reports nothing in `flock_read_hamming.rs` or `train-in-space.rs`. Not fixed here (another crate, outside the ORDER's scope).

## What changed

- `crates/uor-r4-float-train/src/flock_read_hamming.rs`: `slot_boundary_flags` (boundary tokens from the pinned tokenizer: `.` `!` `?` and the byte-level newline and carriage return, added tokens, unassigned ids), `slot_rank_table` (one `u16` per token by TRAIN count, rarer lower, ties by id, boundaries 0), `slot_atoms`, and the diagnostics' `slot_atom_pairs` and `window_atoms`. Five tests.
- `crates/uor-r4-float-train/src/bin/train-in-space.rs`: `--copy-key-oracle TOKENS MASK`, a mode that reads an unpinned UORT pair, cuts it into DEV scoring's windows, prints the count and exits. It takes TRAIN and the tokenizer from `--data-root` at their pins. It refuses every other mode flag and `--recall`. Other training and panel flags given with it are ignored, not refused, because the mode returns before they are read.
- No change to the read, the export or the serve: step 2 did not run, so copy keys 1 and 2 are untouched.
- This REPORT, the ORDER's amendment, one notebook row in `docs/research/notebook/float-training.md`, and the item's line.

## Measurements

`train-in-space --data-root <claude-261's driver-data> --copy-key-oracle TOKENS MASK` at `88746eb6b`. Tokenizer at its pin, TRAIN at its pin. DEV is `ft/dev` (`tokens.u16` sha256 `0919e90f…`, `response_mask.u8` `a838bb1a…`); p-w is the store's `inputs/claude/heldout-panel-p-w1200-2026-10-31/p-w` (`038b0085…`, `797fc234…`). Boundary tokens: 16 of 4,096. "Right" means the latest earlier position whose key equals the query exactly carries the next token.

| Rule | DEV with exact | DEV right | DEV purity | p-w with exact | p-w right | p-w purity |
|---|---:|---:|---:|---:|---:|---:|
| k1 (copy key 1, the trained base) | 6,306 | 3,859 | 0.612 | 21,041 | 12,371 | 0.588 |
| k2 (copy key 2) | 4,383 | 3,506 | 0.800 | 14,069 | 10,971 | 0.780 |
| **slot (the rule's key)** | 3,560 | **2,136** | 0.600 | 10,345 | **5,834** | 0.564 |
| atom2 (diagnostic) | 2,568 | 1,413 | 0.550 | 7,525 | 3,693 | 0.491 |
| win4 (diagnostic) | 4,030 | 3,200 | 0.794 | 12,630 | 9,751 | 0.772 |
| k2>k1 (diagnostic) | 6,306 | **4,388** | 0.696 | 21,041 | **13,990** | 0.665 |

Copyable positions (the next token occurs earlier in the window): 8,243 on DEV, 28,341 on p-w, the same for every rule. `k2>k1` has exactly copy key 1's coverage, because a copy key 2 match is always a copy key 1 match.

**Instrument checks (before the negative was called, §4.2).** The 13 vocabulary boundary tokens are `!` `.` `?` Ċ č `."` `!"` `?"` `".` `.”` `!”` `?”` `?".`; with `<|bos|>`, `<|eos|>` and `<|unk|>` they make the 16. On DEV, 6,577 of 42,856 tokens are boundaries (Ċ 2,216, `.` 1,765, `<|eos|>` 1,226, `?` 577, `<|bos|>` 400, `!` 385, `".` 8), so a clause averages 6.5 tokens. The atom tests include the ORDER's example: "my friend's name is" and "my dog's name is" get different atoms. The count's window slice matches `build_document_windows`: `offset = p − 1 − win_start`.

## The rule, read

"Step 2 runs if the slot key is right on more positions than copy key 2, on DEV and on p-w." 2,136 < 3,506 and 5,834 < 10,971. **Step 2 does not run.** By Done-when 2, no code beyond step 1's count merges.

## Next

- **The next copy-key unit is the mixed pair of copy heads,** not the two rarest atoms: one copy head keyed on copy key 2 and the other on copy key 1, trained as served, the trained-read form of `k2>k1`. The recipe has two copy heads (`--width 256 --hamming-read 4`). Its build unit adds the per-head key to the read, the export and the integer serve, with parity tests. Then a sheet with FIT runs it against (b) at three seeds. This count is a proxy: the trained read keeps 16 graded candidates, not one exact match. The proxy's order is consistent with claude-266's trained result, where copy key 2 did not lead copy key 1 (better at one seed, worse at two); here copy key 1 is right more often, 3,859 against 3,506.
- claude-268 (the slot-key GPU run) is not run: step 2 did not run.

## Checks run at the head

- `cargo test -p uor-r4-float-train --lib slot --no-fail-fast`: 6 passed (the four slot tests and two older tests whose names hold "slot"). `--lib flock_read_hamming`: 21 passed, including the diagnostics' test.
- The boundary test pins every boundary byte. With `!` and the carriage return dropped from the boundary set, `slot_boundaries_are_sentence_ends_newlines_added_and_unassigned_ids` fails; restored, it passes.
- `cargo build -p uor-r4-float-train --bin train-in-space`: ok.
- `cargo clippy -p uor-r4-float-train --all-targets`: nothing in this unit's two files; with `-D warnings` it stops on `uor-r4-integer` (What did not work, item 5).
- The count on DEV and p-w, twice each (once with the slot key only, once with the diagnostics), with the same k1, k2 and slot numbers both times. The input SHA-256s were read with `shasum -a 256`.
- The instrument checks: the boundary list read from the tokenizer with `jq`, and DEV's boundary counts read with `od` and `awk`.
- `uor-harness loop report --check`, `loop pr-check --local` and `loop gates --full` on this REPORT and branch.

## Skills used

None applied.

## Still open

1. **The mixed pair of copy heads** (one on copy key 2, one on copy key 1) is the next copy-key unit: a build unit for the per-head key in the read, the export and the serve, then a sheet with FIT and three seeds against (b).
2. **The count is a proxy.** It reads exact matches and the latest one; the trained read keeps 16 graded candidates. A key that wins here can still lose in training, which only the run can show.
3. **`uor-r4-integer`'s clippy errors** under the current toolchain, outside this ORDER's scope.

## Plain summary

The model copies words from earlier in the conversation by matching a short "key" at each position. This unit tested, on the data alone and with no training, a new key that pairs each word with the rarest word of its sentence part. The test asks a simple question: when the key finds an exact earlier match, is the word after that match the right next word? The new key was right less often than the existing two-word key on both test sets (2,136 against 3,506 positions, and 5,834 against 10,971), so it is not built. The reason looks structural: a copied phrase matches at its end, while the rarest-word key depends on how the sentence began. One cheap combination did better than anything else: use the two-word key when it finds a match, and the one-word key otherwise. It was right on 13–14% more positions than the key the model uses today. That combination is the next thing to build and train.

References #3407.
