# REPORT: one-model-end-to-end, claude-336

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). ORDER: [ORDER.md](ORDER.md). Code head `917908c2b0f3833990f621e86ef9bbebcb22a5c0`, on base `1e5e84a1d497222099a97e22da024b07759750a8`.

**In one line:** the D11 export now writes the ask table `a` and the carry gain `c`. The integer session serves both. On claude-335's sealed B2-aqC checkpoint at seed 1, the integer session's Hamming choices match the float trace, rounded as exported, on every decision of 8 windows.

## What did not work, first

- **The parity windows are not B2 windows.**
  - The ORDER asked for the trace "on B2 windows". B2's windows are made inside the trainer (`train-in-space`), with no data root to read them from. `fbs-snapshot --hamming-trace` takes either the D11 readout's 8 fixed full-context sequences (`fbs_export.rs` `fbs_float_readout`: token `(37 s + 11 t + 1) mod vocab`, 192 tokens each) or `--dev DATA_ROOT`, a language corpus.
  - So the check ran on the 8 fixed sequences. It shows that the integer engine computes the trained `a` and `c` as the rounded float does, on every row, with the carry at `h = 4`, so every lag from 1 to 4 enters the sum from row 4 of each window on. It says nothing about B2's answers, and the carry control removes the whole carry at once, not one lag at a time.
  - B2's answer under the integer session is the next unit's question: a B2 decode through `fbs-toks`, or a B2 window writer for `fbs-snapshot`.
- **The parity check ran on a GPU pod, not on the laptop as the ORDER said.**
  - `fbs-snapshot` opens a CUDA device for the float forward. So the float side ran on one Vast RTX 5090 (`vast-59878078`, $0.563/h, 455 s up to down, about $0.07). The integer side ran on the laptop.
  - It is a host check (a build and three readouts of a sealed checkpoint), not training or evaluation, so it needs no FIT (CHARTER §4 item 5).
- **Rounding alone moves 21 of 3,072 depth-2 type decisions** (0.7%). This is a finding of the rounding, not of the instrument.
  - Against the float trace **unrounded**, the integer session agrees on 3,051 of 3,072 depth-2 type decisions (21 query flips, 0 key flips).
  - At depth 6, 1,978 type rows fall downstream of a depth-2 difference and are not judged. Of the other 1,094, the integer agrees on 1,091 (3 query flips).
  - The rounded float trace, which the integer must match, agrees on all of them (below). So these 21 + 3 differences are the export's Q12 rounding of the read tables, measured once here and not hidden.
- **Deviations from the ORDER, named:**
  - **No new schema string.** The ORDER said "behind a new schema version". The artifact keeps `lut-stack/6`. The new record fields (`asks`, `carry_rows`) and the two section kinds (`hamming.d{depth}.ask`, `hamming.d{depth}.carry`) are optional, so a read without them exports byte-identically. Their presence makes every older engine refuse the artifact through `deny_unknown_fields`. This follows `copy_key`'s precedent; the lab's call is that a schema bump would add nothing that refusal does not already give.
    - The review named the residual risk: a reader that takes an asking artifact for a plain one. `add_point_key_uorlut01` does bump its schema.
- **One such reader exists, and a schema bump would not have caught it.**
  - `crates/uor-r4-cpu-train/src/hamming.rs` (DeepSeek's crate, D84) is an independent CPU port of `lut-stack/6`. It takes `HammingImage` from the integer crate's reader, which now accepts and validates the ask sections, and builds its type heads from each read without reading `HammingReadImage::ask` (`hamming.rs:303`).
  - So, given an asking artifact, cpu-train would serve the old question, silently.
  - No asking artifact exists outside this unit's local folder, so nothing is served wrong today.
  - This unit does not edit cpu-train (another lab's crate). The fix belongs there: refuse a read whose image carries `ask`, until the port serves it. It is sent to DeepSeek on #2028.
  - **One file outside the ORDER's list:** `crates/uor-r4-integer/src/bin/fbs-export.rs`, the command that turns the snapshot into an artifact. The ask block reaches it under its own JSON key `hamming_ask` (`FBS_HAMMING_ASK_KEY`), because `FbsTrunkSnapshot` denies unknown fields and cpu-train builds its structs literally, so a new field could not be added to them. Of the conditional files, `bin/fbs-snapshot.rs` and `stack/mod.rs` were needed and `stack/session.rs` was not: the session already passes its whole window, so `start = 0` serves the carry with no session change. The item's scope gains the three files used.
- **The float carry sums f32 products; the integer sum is exact.** The integer computes the question at Q28 in `i64` (`(table + a) << 16 + c(Q16) · (table + a)(Q12)`). The float forward adds `c · (table + a)` in f32. Near a sign at zero they could differ. No query bit flipped on the 8 windows, so none was seen.
- **Instrument faults (L-007, not mechanism results):**
  - `cargo check --all-targets` on `uor-r4-float-train` fails at the base: `packet-parity` needs the `cuda` feature. Each target was checked instead (below).
  - **`rustfmt`:** the touched files were not format-clean at the base, and this unit adds unformatted hunks to them. Counts at the base / head, per file, with `rustfmt --edition 2021 --check`:
    - `fbs_export.rs` 12 / 27, `flock_read_hamming.rs` 133 / 134, `bin/fbs-export.rs` 1 / 6, `format.rs` 49 / 53, `hamming.rs` 39 / 49, `tests.rs` 279 / 303;
    - `bin/fbs-snapshot.rs` and `fbs_snapshot.rs` 0 / 0.
    - The repo does not gate formatting. Formatting whole files would rewrite other labs' lines, so this PR leaves them.
  - Two builders stalled in slice 2: one at launch, one at 200k context without committing. Slice 2 was finished in the main session.

## What worked

- **The export writes `a` and `c`, and the integer engine serves them.**
  - `fbs_hamming_snapshot_with_ask` (float) writes the ask table and the carry, and `round_hamming_as_exported` rounds them as stored: `a` to Q12 in `i16`, `c` to Q16 in `i32`. An overflow is refused by name. `fbs_hamming_snapshot` still refuses an ask read by name, so no caller drops it silently. The hold is still refused (claude-334).
  - `quantize_hamming_ask` (integer) turns the block into `AskImage`. `add_hamming_ask_uorlut01` writes the sections, and `hamming_image` reads them back.
    - Records refuse a flag list of the wrong length, all flags off, a carry off the readout read, a carry on a read that does not ask, `h` outside 1 to `ASK_CARRY_MAX` (4,096), and a missing or misshapen section.
  - `TypeHeads::with_ask` asks with `key_rotation(table[x_t] + a[x_t])`. At the readout read it adds `Σ_{s=1..h} c[x_{t−s}] · (table + a)[x_{t−s}]` over earlier rows of the window, at Q28 in `i64`. The question is the sign code of that exact sum.
- **Parity on the trained model: equal on every decision.** claude-335's B2-aqC at seed 1, step 16,384, exported to a 6,367,488-byte artifact. The trace covers 8 windows × 192 rows, depths 2 and 6, copy and type heads.

  | Integer artifact, against the float trace rounded as exported (`d11-tie-rounded`) | d2 copy | d2 type | d6 copy | d6 type |
  |---|---|---|---|---|
  | **as exported (ask + carry)** | 3,072 / 3,072 | **3,072 / 3,072** | 3,072 / 3,072 | **3,072 / 3,072** |
  | control: ask block removed | 3,072 / 3,072 | 290 / 3,072 (2,782 query flips) | 3,072 / 3,072 | 288 / 288 judged (2,784 downstream) |
  | control: carry removed (`c` = none) | 3,072 / 3,072 | 3,072 / 3,072 | 3,072 / 3,072 | 602 / 3,072 (2,470 query flips) |

  - On the exported artifact, every row reads 0 query flips, 0 key flips, 0 ties, 0 not served and 0 instrument faults. Top-1 agrees on 3,040 of 3,040 judged per row.
  - **The comparison can fail, and fails where it should.**
    - Removing the ask block moves the depth-2 type questions.
    - Removing only the carry leaves depth 2 exact and moves 2,470 depth-6 type questions. So the carry is served at the readout read alone, and it carries real weight there.
    - The copy heads never change: they do not ask.
- **The tests pass, and each mutation fails its test.**
  - New tests:
    - `ask_head_queries_match_the_f64_question`: the integer ask heads against an f64 reference of the float question, with and without the carry;
    - `a_carried_term_reaches_the_next_rows_of_its_window_only`;
    - `malformed_ask_heads_are_refused` and `malformed_hamming_ask_heads_are_refused`;
    - `hamming_ask_heads_round_trip` and `a_session_serves_the_ask_heads_queries`;
    - `hamming_ask_tables_quantize_as_stored`;
    - float: `ask_heads_and_the_carry_export_and_round_as_stored`.
  - Mutations, each set aside and restored, with the test failing as it must:
    - M1–M4 in the ask heads' sum and window;
    - M5 `type_queries` dropping the ask table;
    - M6 loading the sections;
    - M7 the readout-only carry check;
    - M8 the ask rounding;
    - M9 the carry dropped in the export;
    - M10 truncation in place of rounding.

## What changed

- **`crates/uor-r4-float-train`:**
  - `fbs_export.rs`: `fbs_hamming_snapshot_with_ask` exports the ask table and the carry; `fbs_hamming_snapshot` refuses an ask read by name and points to it; `round_hamming_as_exported` rounds `a` and `c` as stored.
  - `bin/fbs-snapshot.rs`: writes the ask block under `hamming_ask` when a read asks, and the same JSON as before otherwise.
  - `flock_read_hamming.rs`: the carry test now expects rounding to succeed (the specified behaviour changed).
- **`crates/uor-r4-integer`:**
  - `stack/hamming.rs`: `AskImage`; `TypeHeads::with_ask`, which builds the question; and loading of the read's ask sections.
  - `stack/format.rs`: the record fields `asks` and `carry_rows`, their refusals, and `add_hamming_ask_uorlut01`.
  - `fbs_snapshot.rs`: `quantize_hamming_ask` and the block's types.
  - `bin/fbs-export.rs`: reads the block and writes the sections.
  - `stack/mod.rs`: re-exports.
  - `stack/tests.rs`: the tests above.
- This REPORT, the ORDER's scope note, one notebook row in `docs/research/notebook/float-training.md`, and the item's scope gains the three files used.
- No bar, status, scope rule or cap changed. The float cached decode step still refuses the carry (`from > 0`): float decodes a carried read uncached.

## Measurements

- **Pod:** `vast-59878078`, 1 × RTX 5090, $0.563/h, 455 s up to down, about $0.07. Build 26 s, snapshot 1 s, each trace 16–25 s.
- **Integer trace on the laptop:** 1.25 s for 8 windows × 192 rows.
- **Artifact:** 6,367,488 bytes. The ask tables are 4,096 × 256 `i16` per read (2 MiB each), and the carry is 4,096 `i32`.
- **Pins:**
  - Pod side: `job-336.sh` sha256 `912248c0badbc06ede98a127ad3c87848beab32bdc111c05d6fa6158b21eb57a`. Laptop driver `drive-336.sh` sha256 `3a1db66543dd3fe7afc292034f970ee3c0c3bd19bf15f292cd39394c7d7cdb3c`. The checkpoint's three files were hash-checked on the pod before the job started.
  - Float outputs (sha256):
    - `snap.json` `2564b61238eb09f62890c8b3c47900f15ca7429922929eefdc5c585cd2427f2f`
    - `d11-tie-rounded.json` `80e501ab697fb60fda9007ec14b022acf16348d89b260fd84b3846fd9528e4b7`
    - `trace-tie-rounded.json` `20a740b95c4337e825dd230f2e448378bd83d0ec04a97f168ed23f3cdeec934a`
    - `trace-tie.json` `551c3bf9a50844901bbcffae097f0becac0d9773d26802db251f98f9514dbe85`
  - Integer, on the laptop at the code head:
    - artifact `0f9e34317b475cea30997db5ff602bcfe65ec96411e67a27d92c635b57a363f7`
    - trace `b598595d7eb4852f881ef8204e2a9c40564a8503c2d25727783503fd2aac5277`
    - comparison `899c4de897889bc4ad71dd95cec402aec9e329fa034378472bd066a620a0f086`
  - The files stay in the lab's local folder, `local/claude/claude-336/pulled/`, and are not committed (no binaries or model outputs in the repo).

## What this does and does not show

- **It shows:**
  - A trained model with ask heads and the question carry exports through D11 and runs in the integer session.
  - Its Hamming choices equal the float's as exported, on every row of 8 full-context windows, including the carry at `h = 4`: every lag from 1 to 4 enters the sum from row 4 of each window on.
  - Both the ask table and the carry change the integer choices when removed, so the match is not a match of an unused part.
- **It does not show:**
  - whether the integer session answers B2: the windows are the fixed sequences, not B2's;
  - anything about language text;
  - the float cached decode step with the carry.

Closes nothing: #3407 stays open.

## Checks run at the head

- `uor-r4-integer --lib --features serving-wasm`: 499 passed, 0 failed, 1 ignored. `uor-r4-float-train --lib`: 336 passed, 0 failed. Both run again after the review fixes.
- **The in-unit fresh review:** three reviewers, given the spec and the diff only. Slice A (integer serving, about 390 lines) and slice B (float export and the tests, about 480 lines) each returned APPROVE-WITH-MINORS; the claims check returned CLAIMS-OK-WITH-FIXES. Fixed in this PR:
  - `round_hamming_as_exported` refuses a carry on a read that does not ask, as the export does. The test now asserts it, and with the check set aside the test fails, at its `expect_err`.
  - `fbs-export` takes a `hamming_ask` block in which no read asks as the plain snapshot. Checked by hand: `[null, null]` exports byte-identically to the snapshot without the key (sha256 `66d11f8c…`), where before it was refused ("not all off"). The real export is unchanged (`0f9e3431…`).
  - A doc comment moved back to its test.
  - Two wordings narrowed: the carry's lags, and the plain summary's controls.
  - Left as named: the schema string (above); the integer side owns the ask table's vocab check (`AskImage::checked`); the ask snapshot's JSON is written through `serde_json::Value`, so its key order differs from the plain snapshot's (the no-ask path is unchanged); one `pub use` line's order.
- `cargo check` passes:
  - `uor-r4-cpu-train --all-targets`;
  - `uor-r4-float-train --lib` and `--bin fbs-snapshot`;
  - `uor-r4-integer --all-targets --features serving-wasm`.
- Mutations M1–M10, each set aside and restored, with its test failing.
- No banned word in the diff.
- `loop gates --full`, `loop report --check` and `loop pr-check --local` are run at the head before the queue. The fresh exact-head review is posted with `loop review`.

## Skills used

- None beyond the loop itself: the ORDER, the parity host check and the fresh exact-head review.

## Still open

- **B2's answer through the integer session:** a B2 decode in `fbs-toks`, or a B2 window writer for `fbs-snapshot`. Then a language run with the carry, under its own sheet.
- **cpu-train's Hamming port ignores the ask heads** (above). DeepSeek's crate should refuse an image whose read carries `ask` until it serves it. This was sent to DeepSeek on #2028.
- **The float cached decode step for the carry:** out of scope. Float decodes a carried read uncached.
- **Rounding's 21 of 3,072 depth-2 type decisions:** the export's Q12 rounding of the read tables moves them against the unrounded float. It is the export gap of the read, recorded, not a parity fault.
- **Formatting:** the touched files were not `rustfmt`-clean at the base, and this unit adds hunks (counts above). This is for a formatting pass, if the project takes one up.

## Plain summary

The model that learned to carry a question forward (claude-335) can now be turned into the integer form that runs on ordinary devices, and run there. Before this unit, the export refused it.

On eight test texts of 192 tokens each, the integer version picks the same memory entries as the training version at every position, once the training version's numbers are rounded the way the export stores them. With either new part removed, most of the choices that part feeds disagree, while the choices that never used it stay the same. So the match comes from the new parts being served right, not from parts they do not touch. One rented GPU did the float side in under eight minutes for about seven cents.

The next step is to check the integer version on the recall task itself, then try the carry on real text.
