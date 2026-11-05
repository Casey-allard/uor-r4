# REPORT: scale-probe, claude-294 (the prep sheet)

Lab: claude. Order: [ORDER.md](ORDER.md). Item: [scale-probe](../../../../roadmap/items/scale-probe.md) (#3472), done-when 1. Base: `9577aebc9`. PR #3744.

**Result: the scale probe's prep sheet is written, with its instrument, its mixes and its overlap census.**
- **The sheet** [`scale-probe-2026-11-05.md`](../../../../harness/prep/scale-probe-2026-11-05.md) registers 18 grid runs: widths 256, 512 and 1,024 × D1, D2 and D3 (a quarter, a half and all of claude-293's 27,968,113 human-written TRAIN tokens) × 2 seeds. Every run takes the same 25,165,824 positions. Two templated contrast runs are reported only. On three 1 × RTX 5090 pods, the cost is about 17–20 GPU-hours, $16–19.
- **The bars are set in advance:** R1 (width), R2 (data) and R3 (against its own TRAIN's T2 and unigram), each read on all rows and on the rows whose context D3 does not hold. An ordered six-row table says what each outcome unlocks.
- **The instrument** `docs/labs/scale-probe/instrument` (bin `scale-probe`, laptop counting only) has four commands, `mix`, `seen`, `grade` and `pair`, and 58 tests.
- **The mixes and the census** are stored under `results/claude/scale-probe-2026-11-05/` (`inputs/`, `census/`), each with a `SHA256SUMS` read back.
- **The decision card** moves the item's Data and panel lines to real text (the ORDER's card, and the item file).

**Claim check.** Casey's 4 November plan, P5 (the lab's list, which Casey adopted that day): the bigger train is `scale-probe`, bounded at one sheet unit and one or two run units. It is step 6 of the order decided on #2028 (6634218223). This is the sheet unit. No code that trains or serves changes. The sheet fixes the run that says whether the one model gets better with width and with data, which decides whether the trunk is scaled next.

## What did not work, first

- **Three FIT rounds, not one.** Each round found items in the sheet or the instrument; each is fixed at the head:
  - **Round 1** (FIT WITH CHANGES):
    - the outcome table was not ordered or exhaustive;
    - no band was defined for a third seed, and `pair` could not read one;
    - the width limits were not cited, and no fallback was registered for a refused 1,024;
    - I-2's cut did not end every run on an evaluation;
    - R3 had no mixed reading and an unscoped claim;
    - R8 did not name its DEV source;
    - the power section stated an expectation as if measured.
  - **Round 2,** in four narrow reviews:
    - **The instrument:** FIT WITH CHANGES.
      - `pair clean=` did not check that the seen set was built on a1's TRAIN.
      - Three refusals had no test: a wrong TRAIN SHA-256, a wrong panel mask SHA-256, and a cell whose seeds train on different TRAIN.
    - **The widths:** sound, with wording fixes. Every limit admits 1,024, inclusively. `--hamming-read` is the width over 64. The stack heads' width and the writer's blocks grow with the width.
    - **The scored rows:** FIT after two fixes.
      - The sheet said `--score-rows` drops a window with fewer than four scored offsets. It drops only a window with none, and a document of one token or none (`hamilton_sheet.rs:1880-1920`), so the rows should be every scored position.
      - I-5 did not say that no decision-trace option is passed. The trace is taken in the memory pass only, so it would make the two passes differ.
    - **The training data and the DEV leak:**
      - Sound: `--train-split` replaces TRAIN, and the in-run DEV evaluation leaves no state that training reads.
      - One MAJOR: the sheet called R8's `T` the nearest top-1. It is the writer's top-1 (`dev_top1_hits`, `in_space_long.rs:11586`), picked from the writer's admitted cells, not the whole table.
  - **No round returned a plain FIT,** and the sheet first said `TEST FITNESS: FIT`. Round 3 read only the round-2 delta, and no reviewer re-read its one applied sentence. The claims review caught this; the sheet now reads FIT WITH CHANGES.
- **The exact-head review found a silent fallback in `grade`.** A panel token at or above 4,096 keyed a row that T2 holds no counts for, and the grade fell back to the unigram without a word. No recorded number changes: no grade has run on real rows yet, and the census does not read T2. The four real panels and the three mixes all declare a vocabulary of 4,096 in their headers, and the corpus writer refuses an id past its header (`mmap_corpus.rs:498`), so the run's inputs cannot trip the new check. A wrong input would still have graded. `load_split` now refuses such a token by position, in TRAIN and the panels alike.
- **A quotation that was not Casey's.** The sheet and this REPORT quoted P5's line as Casey's words. It is the lab's list, which Casey adopted on 4 November. It is now unquoted.
  - **L-007 wording.** Row 6 read "Flat at these sizes" and closed the configuration. R3 read inside and behind alike as "does not beat T2". Both are now "not distinguishable at this power", and only a mechanism classification closes anything.
- **The new `clean=` refusal changed the readings' row sets.** A pair at D1 or D2 can no longer read D3's seen set. Every pair now reads the seen set of its cell X's TRAIN, X is always the larger mix, and step 8 runs `seen` against D1, D2 and D3. R1 to R3 still read D3's set. R6's adjacent data pairs are written larger-first.
- **The turn memory is out of the recipe.** Recipe (b) is run without it: the probe asks how the trunk scales, Wikipedia has no prompts to key on, and its host cost on 28M real tokens is unmeasured (claude-293: 0.67–0.75 GB per million real TRAIN tokens for the census's reply rows). A recipe `one-model-end-to-end` picks later is a new configuration.
- **The contrast cell's panel changed before the card was logged.** The first draft drew a new templated panel for it. It is now scored on DEV and on the same four real-text panels.
- **The seen share grows with the mix:** 8.8%, 12.1% and 15.6% of the panel's 1,460,697 scored positions (D1, D2, D3). So part of a gain R2 reads on all rows could be recall of contexts D3 holds and D1 does not. That is why R1 to R3 are also read on the unseen rows, and why a disagreement between the row sets is its own outcome.
- **Process slips:**
  - A builder stopped at its context limit with uncompiled code. I compiled it, and it was sound; a fresh builder finished the work.
  - `grade` first read `rows=NAME:all` as a file name. It now refuses it by name.
  - `uor-store put` printed "0/0 uploaded" and then "3/3 uploaded" for seven files. `uor-store ls` and the read-back show all seven stored.
  - The first Stop-rule fallback said "the widest width below 1,024 in steps of 64 (768)"; that width is 960, which needs 15 heads, an odd split. It now names 768 and says why.
  - The loop guard refused a patch file in the session scratchpad. The negative gates ran from the unit's `local/claude/claude-294/` instead.
  - **`loop pr-check` failed D32** with 6 sites in `lib.rs` and 4 in `tests/cli.rs`, where the base had none. They were not floats. Its lexer reads a tuple field taken straight after `]` or a space (`roots[0].1`, rustfmt's `s.0 .0`, a chain line opening `.1;`) as a `.5`-style literal. Each was rewritten to bind the element first, with the same behaviour; the suite passes.

## What changed

- **`docs/harness/prep/scale-probe-2026-11-05.md`:** the prep sheet, `TEST FITNESS: FIT`.
- **`docs/labs/scale-probe/instrument/`:** a standalone crate with its own `[workspace]` and bin `scale-probe`, counting integers only on the laptop.
  - `mix` writes the nested document-prefix mixes.
  - `seen` marks each panel row short, seen or unseen against one TRAIN, and writes `excluded.bin`.
  - `grade` scores a checkpoint's rows against its own TRAIN's unigram and T2 tables, and writes the hit bitsets.
  - `pair` gives `b`, `c`, `d`, the band and the verdict at two or three seeds, against a cell or a table, on all rows and, with `clean=`, on the unseen rows.
  - It refuses each inconsistency it checks by name; 58 tests.
- **`docs/roadmap/items/scale-probe.md`:** the Data and panel lines say real text (the card).
- **`docs/loop/tasks/scale-probe/claude-294/`:** the ORDER, with the card and the cancel probe, and this REPORT.

## Decision: the Data and panel lines (the ORDER's card)

Logged on #3472 as `[claude] owner log:`, on its own fresh decision review; Casey can reverse it.
- **Data:** claude-293's real-text corpus at fractions, human-written text only under the decided order's interim §3.10 reading. OASST2 is English threads only (`real_text.rs:574`), and Wikipedia is the English edition. The templated corpus stays as one contrast cell.
- **Panel:** the corpus's own held-out splits, cut by source id, with the 8-token overlap census against each mix. P-dev stays reserved.
- **Scores:** the decided order says "integer top-1 against unigram and T2 count tables". The card departs from those words: the deciding count is the float model's whole-table nearest top-1, not §4.8's integer top-1 through the D11 export. The D11 export's panel top-1 is owed before any width is served or any claim is made.
- **Reversal:** restore `docs/roadmap/items/scale-probe.md` from this unit's merge-base; "hold" stops any launch. Sealed mixes, census and results stay in the store, and a run already made stays as its own configuration.
- **The decision review** (fresh, non-author): STANDS WITH CHANGES. D1 (Data) and D2 (Panel) stand as written. D3 needed the Scores wording above and the reversal's restore source, and no part is owner-only. Its optional notes are not taken: the item's panel line could carry the row-set rule, and the card could cite claude-293's frozen-panel exclusion screen.

## Measurements

- **The mixes** (`scale-probe mix`, document prefixes root by root, so D1 ⊂ D2 ⊂ D3):

  | Mix | Tokens | Scored | Documents | Passes in 25.2M positions |
  |---|---|---|---|---|
  | D1 | 7,004,928 | 6,865,470 | 4,653 | 3.67 |
  | D2 | 13,989,169 | 13,708,035 | 9,053 | 1.84 |
  | D3 | 27,968,113 | 27,416,554 | 18,016 | 0.92 |

  Store `inputs/`, `SHA256SUMS` `56a7db05a8f35b01e7231e90f95c2e7a095415263e2c16adcd93a20968c60b9c`, fetched back and checked byte for byte.
- **The census** (`scale-probe seen`, every scored panel position against each mix):

  | Against | Seen | Short | Unseen |
  |---|---|---|---|
  | D1 | 129,173 | 4,935 | 1,326,589 (90.8%) |
  | D2 | 176,210 | 4,935 | 1,279,552 (87.6%) |
  | D3 | 227,312 | 4,935 | 1,228,450 (84.1%) |

  Store `census/`, `SHA256SUMS` `88929e87931bef0fd91213ffbeb4701038354aa6cceca8be3c1621f478399f67`, read back. "Seen" is a screen, not a finding of copying: a row whose 8-token context occurs anywhere in TRAIN.
## Checks run at the head

- **The instrument,** at `8041d8f0d70278cb70b3a93f2b0d36fe64eb5614`, the last commit that touches its code: 58 tests pass (`cargo test --offline` in its own workspace), and clippy with `-D warnings` is clean.
- **Negative gates run in this unit.** Each one failed with its fix set aside and passed once the fix was restored:
  - **The three-seed verdict and the three-seed spread:** set seed 3 aside.
  - **`clean=`'s TRAIN check:** set the check aside, and `pair_clean_refuses_other_train` fails.
  - **Seed 3 sharing its cell's TRAIN:** narrow `same_train` to two seeds, and `pair_refuses_cells_with_mixed_train` fails.
  - **The exact-head fixes:** set this round's `lib.rs` lines aside (diffed against `ec4704bbf1bb6cf12d9bf66bce87e2dd1b31e88a`). Exactly `grade_refuses_a_panel_token_beyond_vocab` and `pair_refuses_per_split_differences` fail; 56 pass.
- **Mutation gates,** asked for by the second-half review. Each made 4 tests fail, and the restored head passes 58:
  - `mcnemar_floor`'s `<` made `<=`: `floor_values` among them.
  - The verdict's `d > band` made `>=`, so a d equal to the band would read ahead: `band_spread_sets_the_band` and `band_reads_a_third_seed` among them.
- **Refusal coverage:** the data-integrity refusals have tests, and so do `pair`'s argument refusals. `mix`, `seen` and `grade` share the one parser behind them. Their own argument refusals (a repeated, unknown or missing `key=`) have no test of their own.
- **The harness,** at `334db02ad957b83e1320d37aed1f7eb2d0b88991`, with a clean tree. Each log starts with that SHA (`local/claude/claude-294/`, gitignored):
  - `loop report --check`: PASS.
  - `loop pr-check --local`: verdict PASS, with three advisory (D76) warnings: rule sources, read-path size and FILEMAP rows.
  - `uor-harness check`: exit 0, with 347 rules, 0 errors and 315 warnings. None of the warnings names this unit's sheet; the shared ones are disk, claims and branches.
  - `loop gates --full`: exit 0, with all 11 gates GREEN.
  - The commit after that head changes only this REPORT, and `loop pr-check --local` is re-run at the final head before the review record.
- **FIT:** `TEST FITNESS: FIT WITH CHANGES`, from three rounds of fresh non-author review, with every change applied by `2c3a96147c98791dfd01d3304c6b0554ae77a12e`. It is posted on #3472 (comment `6666302038`, its label corrected in a later comment) and recorded in the sheet.
  - No round returned a plain FIT. Round 3 read only the round-2 delta and returned one must-fix, applied verbatim; no reviewer re-read that sentence.
  - The harness reads FIT WITH CHANGES as FIT (`prep.rs:97-119`), as it did for the decode-cache sheet of 4 November.
  - The reviewers ran no builds; the tests and gates above are the author's.
- **Decision review:** STANDS WITH CHANGES (above). The card is logged on #3472 as `[claude] owner log:` (comment `6666457437`).
- **The in-unit exact-head review:** four fresh non-author reviewers on `ec4704bbf1bb6cf12d9bf66bce87e2dd1b31e88a`, who ran no cargo. Every finding below is applied unless named in Still open.
  - **`lib.rs`, first half: CHANGES.**
    - One MAJOR: a panel token at or above 4,096 fell back silently (above). `load_split` now refuses it.
    - Its minors: a TRAIN past `u32::MAX` tokens could overflow a bigram count (now refused); the README's build line lacked `--locked`; `main.rs` did not name `seen`.
  - **The tests: APPROVE.**
    - Three needles matched loosely; they are tightened.
    - A bad `table=` and `seen`'s existing out had no test; both do now.
    - `pair_refuses_row_identity_or_split_difference` reached only the pooled check; `pair_refuses_per_split_differences` now reaches the three split-by-split checks.
  - **`lib.rs`, second half: APPROVE.**
    - `pair` took each split's row count from a1 alone; a seed whose counts differ is now refused.
    - It asked for the two mutation checks above.
  - **The claims: CHANGES.**
    - Two MAJOR: the harness results were promised and not written, and the FIT was labelled FIT when every round returned FIT WITH CHANGES.
    - Four MINOR: no test count at the head; the P5 quotation was the lab's list, not Casey's words; the refusal coverage was unscoped; and the plain summary said "is scored" and claimed recall could be told apart.

## What it unlocks

The run unit (done-when 2 and 3):
- the D67 pre-run review, right before launch;
- the 18 grid runs and the 2 contrast runs, sealed and read back;
- grading on the laptop, then the Bar's table.

## Skills used

No skill applied. The loop's ORDER and REPORT procedure was used.

## Still open

- **The D67 pre-run review** is the run unit's, against the lease and the caps as they stand then.
- **The width-1,024 step time is unmeasured,** so the cost is a projection. I-2 measures it and cuts steps equally if the projection passes 20 GPU-hours.
- **The D11 export's panel top-1** is owed before any width is served or any claim is made.
- **The census reads 8-token contexts only.** A shorter shared phrase is not screened, which makes the unseen set an upper bound on unseen text, not a proof that the text is new.
- **Two exact-head minors not taken:**
  - `load_split` hashes a token file from one read and maps it with a second open, so a file changed between the two is not caught. The inputs are sealed and read back against their `SHA256SUMS` before any grade.
  - `pair clean=` checks `seen.json`'s SHA-256 and reads `excluded.bin` itself, but does not recount `seen.json`'s counts from it.

## Plain summary

This unit plans the project's first test of whether the model gets better as it gets bigger, and as it reads more. The model will be trained at three sizes on a quarter, a half and all of 28 million tokens of human-written text, twice each. Each copy will be scored on every position of four held-out sets that no model has read. The rules for what counts as better, and what each result leads to, are written down before any run. Three rounds of independent review each asked for changes, and all were made. A new counting tool also marks the test rows whose preceding eight tokens the training text already holds, so a gain on those rows can be read apart from a gain on the rest.
