# REPORT: one-model-end-to-end, claude-357

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [ORDER.md](ORDER.md). PR #3947.

A laptop-only build, approved by the owner on 12 Nov ("Yes, build it"): what a retrain with the `longest-after-opening` copy rule needs. The rule is now in the float trainer, it travels to the exported artifact as a header field, the integer reader serves it as the artifact's own rule, and the run has its [sheet](../../../../harness/prep/one-model-end-to-end-copy-longest-2026-11-12.md).

**Nothing was trained and no pod was launched.** Whether a model trained with the rule states remembered values more often is not known. The run is unit claude-358 in the plan and waits for the owner's word.

| Gate | Reading | Status |
|---|---|---|
| G1 | Each new laptop test fails when the lines it pins are set aside (twelve set-asides, below) and passes at the head. | **met** |
| G2 | The four stage 3 artifacts, read at the head on claude-356's fresh prompts under their own rule and under `--copy-rule longest-after-opening`: eight reply files byte-identical to claude-356's. | **met** |
| G3 | The float crate's `cuda` build with the new code and its two pod-only tests compiles under the stub compiler; `uor-r4-training` checks; the integer suite, the float host suite and the instrument tests pass; no clippy warning on a changed line; the three ratchets are clean. | **met** |
| G4 | The sheet passes `uor-harness prep-check` and carries a fresh non-author fitness read that ends FIT: the first read was NOT FIT, the second, of the revised sheet, FIT. | **met** |
| G5 | This report. | **met** |

## What did not work, first

1. **Nothing here says the rule helps a trained model.** This unit builds; it measures no model.
2. **The rule's tensor code has never run.** The laptop's backend refuses tensor work. The GPU choice (`choose_longest_from`) and the forward that reads by it are type-checked only; their two tests (`the_longest_choice_takes_the_longer_match_except_at_an_opening`, `the_longest_rule_reads_what_followed_the_longer_match_except_at_an_opening`) compile and wait for the pod, where the sheet makes them the first step. What is tested on the laptop is the host side: the opening rows, the sort key's arithmetic against the integer reader's choice, save and load.
3. **ORDER decision 5 was wrong.** It said turn memory's leave rule acts only at reply openings. With `--turn-memory-reply 1`, which stage 3's recipe carries, it acts at every later reply row too (stage 3 logged `reply keys left to the copy read 160416 of 327257`). Left as it was, a later reply row could lose its key from the reply table while the copy read named another token. The leave rule now asks the copy read by the run's rule (ORDER addendum). **So the flag changes two things in training, and the run cannot tell them apart.**
4. **The run is not powered to confirm a small gain.** By the sheet's own figures the three-seed mean reaches its bar about 13 times in 100 at a true gain of +10 recall replies of 578, 1 time in 2 at +29 and 74 times in 100 at +40. I expect +10 to +40, so Level is the most likely reading. It can show a large gain or harm.
5. **The sheet's first fitness read was NOT FIT, on two blocking points, both right.** No check could fail if the flag was parsed and never changed training, and nothing showed that the trainer at the new pin, flag off, trains as stage 3's did. The sheet now carries a check for each (I-u and I-n, measurements 7 and 8) and the read word for word.
6. **The exporter does not compare the checkpoint's rule with the config's,** which the plan named. The snapshot does not carry the rule. The trainer refuses a resumed run whose marker differs, and the sheet's I-u would show a mismatch.
7. **Two trainer paths have no laptop test:** the marker check at the trainer's start (`check_run_markers` in `run_long`) and the scoring pass's check of its panel. Both sit behind the trainer's device. The sheet's smokes run them first and require the line each prints.
8. **Two small costs a reviewer found were left:** the marker check loads TRAIN a second time at start, and the scoring pass loads the checkpoint's reads a second time. Each is seconds on a run of hours.
9. **The first run of the head checks was on a tree with the integer files not yet committed.** Every check was rerun at each committed head; the table below is the last run.
10. **Found from the tokens, a reply's opening is not always where a caller marks it.** Stage 3's artifacts with the header field, against the same artifacts with the caller-set rule of claude-356: two of the four reply files are byte-identical and two differ in one reply of 984 each (n-s2 row 705, p-s1 row 841). In both, the model itself wrote `Assistant:` inside its reply, and the reader now treats the next row as an opening, as training does.
11. **The first run of that description failed on my kit,** which left out the reader's required `--mark-reply`. It was rerun with the flag.
12. **`cargo clippy --all-targets` on the integer crate exits non-zero** on an error in `crates/uor-r4-integer/tests/openai_loopback.rs:1658` ("read amount is not handled"), a file this branch does not touch. No clippy line falls on a line this branch changed.
13. **The marker check does not cover the pinned fresh-v2 split** that the scoring pass can also read. The sheet's run does not score it.
14. **The reading panel's prompts at the 192 window are not usable as they are:** the scorer marks 569 of 578 of that file's own references right and exits non-zero. Why was not chased: the sheet reads at the 256 window only, where it marks 578 of 578.

## What changed

- **`crates/uor-r4-float-train/src/flock_read_hamming.rs`.** `HammingReads::with_copy_longest(marker)`, refused without `copy_latest` or with a two-token copy key. `longer_rows` gives 0 for a reply's opening row and 1 for every other. `choose_longest_from` is the latest rule's GPU sort with one more term in its key: `bits·2·time + time·(1 − longer) + (time − 1 − s)`, where `longer` says the token before the match also agrees by the head's code. The marker is saved in the checkpoint beside `copy_latest` only when on.
- **`crates/uor-r4-float-train/src/in_space_long.rs`, `bin/train-in-space.rs`.** `--hamming-copy-longest A,B`. It needs `--hamming-read` and `--hamming-copy-latest 1`. At start the trainer refuses a TRAIN or DEV split in which the marker and the reply mask disagree at any position, and prints the count; the scoring pass makes the same check on each `--panel`. A resumed run is refused when the checkpoint's marker differs. The leave rule follows the run's rule (`copy_names_ruled`). `config.json` holds the field only when set. A scoring pass on a checkpoint that carries the marker prints it with each panel's count of reply openings, which the sheet's I-m reads.
- **`crates/uor-r4-integer/src/stack/format.rs`, `hamming.rs`, `mod.rs`, `session.rs`.** A header field `copy_longest`, left out when off, so every existing artifact keeps its bytes; refused unless the latest rule is on, the copy keys are one token and both ids are in the vocabulary. An artifact that sets it serves `longest-after-opening` as its own rule and finds the opening row from the last two tokens. An artifact without it serves as before.
- **`crates/uor-r4-integer/src/bin/fbs-export.rs`.** Reads `hamming_copy_longest` from the run config and sets the header field; a missing or null value is off.
- **The sheet,** `docs/harness/prep/one-model-end-to-end-copy-longest-2026-11-12.md`, and the ORDER's addendum.
- **`i-g-set-aside.patch`,** beside this report: one line that takes the longer-match term out of the float read's choice. The sheet's I-g applies it to a scratch copy on the pod, where the two tensor tests must then fail.

An opening row is found from the tokens (the reply marker, ids 714 and 28: the end of `Assistant`, then `:`), not passed in beside them. The plan had the trainer pass the reply mask to every forward and both readers mark every opening; with the marker, the trainer, the float read and the integer reader make one test, and no reader changed.

## Measurements

No model was measured. These are the build's own checks.

**1. Nothing served changes for an existing artifact (G2).** Reader built at the head; claude-356's fresh panel (seed 15), 256 window, 48 tokens, 984 prompts; `cmp` against claude-356's reply files.

| Artifact | Own rule | `--copy-rule longest-after-opening` |
|---|---|---|
| n-s2 | identical | identical |
| p-s2 | identical | identical |
| p-s1 | identical | identical |
| p-s3 | identical | identical |

**2. The export path, end to end, from stage 3's four sealed snapshots.** Exporter built at the head.

| Export | Result on each of the four |
|---|---|
| the run's own `config.json` | stage 3's artifact, byte for byte |
| the config with `"hamming_copy_longest":[714,28]` added | byte-identical to stage 3's artifact with the header field inserted by hand (`"copy_latest":true,"copy_longest":[714,28]`, the header padded again to 64, the data untouched) |
| the same with `hamming_copy_latest` set false | refused, no file written: "the longest rule needs the latest exact rule, one-token copy keys and a marker of two vocabulary ids" |

The four artifacts with the field are the run's comparison models; their SHA-256 are in the sheet.

**3. The header field served as the artifact's own rule** (a description, no bar). Those four artifacts read with no `--copy-rule`, against claude-356's caller-set files: p-s2 and p-s3 byte-identical; n-s2 and p-s1 differ in one reply each (point 10 above).

**4. The marker against the reply mask, counted on the laptop.** TRAIN without a line: 215,017 reply openings, each behind the pair 714, 28, and the pair occurs nowhere else. The ORDER's counts stand for the DEV split (1,226) and the two panels.

**5. The negative gates (G1).** One set-aside at a time, the new tests of that crate run, the file restored. Every build compiled; the tests not named passed.

| Set aside | Tests that then fail |
|---|---|
| `longer_rows`: no row is an opening | `the_longer_rows_are_every_row_but_a_reply_opening` |
| the sort key: the longer match sorts last | `the_longest_key_orders_as_the_integer_reader_chooses` |
| the save of the marker | `the_longest_rule_is_saved_only_when_on` |
| the trainer's refusal without `--hamming-copy-latest 1` | `the_longest_flag_is_off_by_default_and_checked` |
| `copy_names_ruled`: always the latest rule | `the_leave_rule_reads_the_longer_match_except_at_an_opening` |
| the marker check: a stray marker passes | `the_marker_check_refuses_a_missing_and_a_stray_marker` |
| the reader's `opens`: the caller's mark, not the marker | `the_marker_names_the_opening_row_in_place_of_the_caller`, `the_reply_marker_finds_the_opening_row_without_a_mark` |
| the header's check | `the_longest_rule_is_recorded_only_when_set` |
| the artifact's own rule: latest exact | `the_longest_rule_is_recorded_only_when_set`, `the_reply_marker_finds_the_opening_row_without_a_mark` |
| the header field written when off | `the_longest_rule_is_recorded_only_when_set` |
| the refusal of a resumed run whose marker differs | `the_longest_flag_is_off_by_default_and_checked` |
| a caller's rule on an artifact that names a marker | `the_reply_marker_finds_the_opening_row_without_a_mark` |

**6. The run's reading panel,** written here and read by no model: generator seed 16, 981 prompts at the 256 window, 578 recall questions (the scorer marks all 578 of its references right), rows 143 and 965 left out because they equal a seed 14 prompt. Its hashes are in the sheet.

**7. The calibration of the sheet's check I-u,** which asks whether a retrained model's read really uses the rule. Stage 3's four models were trained and scored under the latest rule. Each model's float rows from its stage 3 scoring pass (seed 14 panel, 192 window, 40,719 scored rows) against its artifact as exported and against the same artifact with the header field; `row-picks` built at the head.

| Stage 3 model | Rows differing, as exported | Rows differing, with the field | At opening rows, both |
|---|---|---|---|
| p-s2 | 670 | 1,351 | 34 |
| p-s1 | 520 | 1,227 | 34 |
| p-s3 | 597 | 1,206 | 32 |
| n-s2 | 504 | 1,139 | 31 |

A model read by one rule and served by the other differs on 609 to 707 more rows, and on no more opening rows, where the two rules are one. The first column equals claude-354's counts. For a retrained model the sheet requires the other sign.

**8. The trainer between the two pins.** From stage 3's pin (`93e089ff256654e75a19641b7e2fbd2bc8e24524`) to this head, the trainer's crate is touched by `1966da68b` (claude-352) and by this unit; `uor-r4-training`, `uor-r4-core` and the lock file by nothing. `1966da68b` changes the `--panel-prompts` mode only (`split_prompts`, `panel_prompts`, a new `prompt_cut_applies`); in a training run its two changed lines in `main` pass the arguments they passed before. Stage 3's p-s2 and n-s2 logs, two jobs of one seed, hold the same eight `[memory]` lines and the same `step 0` line, which is what the sheet's flag-off smoke (I-n) is compared with.

## Checks run at the head

Run on the laptop (macOS, CPU only) at `8a1f935c743d21550bc29787cf1cb977c23f462c`, with one of the last commit's four docs files uncommitted in the tree. Its `crates/` are those of `183bfabd0507e9837827ef1e5eb01810f20ab694`, the last commit that changes code; the commits after it change the sheet, the set-aside patch and this report only. The logs stay on the laptop under `local/claude/claude-357/`: `final2-357.log` (this table), `neg-357.log` and `r2-357.log` (measurement 5), `calib-357.log` (7), and `evid-357.log` (the counts of measurements 3, 4, 6 and 7, `prep-check`, and the patch check).

| Check | Result |
|---|---|
| `cargo test --locked --release -p uor-r4-integer --features serving-wasm --lib --no-fail-fast` | 552 passed, 0 failed, 1 ignored |
| `cargo test --locked -p uor-r4-float-train --lib --bin train-in-space --no-fail-fast` | 385 and 12 passed, 0 failed |
| `raw-id-replies` tests (instrument, release) | 13 passed |
| `cargo clippy --locked -p uor-r4-float-train --features packet-parity --all-targets`, stub compiler | exit 0: compiles, the two pod-only tests included |
| `cargo check --locked -p uor-r4-integer`, `-p uor-r4-training` | both compile |
| `cargo clippy` on the float crate (stub compiler) and the integer crate, `--all-targets` | no line on a line this branch changed; the integer run exits non-zero on point 12's file |
| `uor-harness ratchet d23`, `d25`, `d32` | all exit 0, no site added |
| `uor-harness prep-check` | reads the sheet as FIT |
| the set-aside patch of the sheet's I-g, applied to the head | applies; the float crate's `cuda` build with it compiles under the stub compiler; taken back out |

## Skills used

None beyond the project loop.

**Reviews.** Each by a fresh `delegate:reviewer` subagent of this Claude Code desktop session, given the spec and the diff and not the plan.

| Review | Of | Verdict | What was taken |
|---|---|---|---|
| The float read | `flock_read_hamming.rs` at `367e05e04` | APPROVE | the tie-key refusal names the doubled room; a test comment reworded; the set-aside of the longer-match term on the pod, now in the sheet's I-g |
| The trainer | `in_space_long.rs`, `train-in-space.rs` at `367e05e04` | APPROVE | a laptop test of the resumed run's refusal. Left: points 7 and 8 above |
| The integer reader and the export | the stack files and `fbs-export.rs` at `a7b0c4089` | APPROVE | a caller's rule over an artifact's marker is asserted; a null marker reads as off |
| Round 2 | `183bfabd0` | APPROVE, no finding | nothing to take |
| The sheet, first fitness read | at `619f9bc97` | NOT FIT, two blocking points | all six points |
| The sheet, second fitness read | at `8a1f935c7` | FIT, four non-blocking notes | all four |
| The sheet's change after that read, and the set-aside patch | at `d7bc13025` | APPROVE | nothing to take; two low items in the PR's review record |
| This report's figures against the logs | at `d7bc13025` | CHANGES: the figures it could check all match; five had no log named, two cross-references pointed at the wrong point, one check was labelled `cargo check` where `cargo clippy` ran | all taken: the counts rerun into `evid-357.log`, the references and the label corrected |

Both fitness reads are in the sheet word for word.

## Still open

- **The run** (claude-358): one pod, stage 3's four jobs with the flag, the sheet's readings. On the owner's word only.
- **An arm that separates the copy read's rule from the leave rule's.** Not in the sheet; it would be a fifth job.
- **The second token after a bare-space opening** is not spared, as in claude-356.
- **The first token of a value, stopping after a copied string, questions about things never told:** untouched.
- No milestone moves. claude-352's registered reading stays NOT_RUN.

## Plain summary

The models copy a remembered value by finding where the current word last appeared in the conversation and reading what followed. claude-356 showed that matching two words instead of one, everywhere except at a reply's first word, makes the existing models state the value a little more often (23 to 33 more answers of 560). Those models were trained matching one word.

This unit makes it possible to train a model the two-word way. The trainer has the rule behind one flag, the saved model carries it to the laptop, and the laptop program follows it without being told. On the laptop everything that can be checked without a GPU was checked: the old models still give exactly the same replies, an old model exported with the new field is byte for byte what a hand edit gives, and each new test fails when the code it guards is taken out. The part that only a GPU can run has not run.

The plan for the run is written down with its pass marks fixed in advance. A reader who did not write it found two holes on the first read: nothing would have caught the new setting being switched on and silently doing nothing, and nothing showed the old way of training still behaves as it did. Both now have a check, one of them tried out on the four existing models, and the second read passed the plan. One thing it says plainly: if training this way helps only a little, the run will probably not be able to show it. It can show a large gain or a loss.
