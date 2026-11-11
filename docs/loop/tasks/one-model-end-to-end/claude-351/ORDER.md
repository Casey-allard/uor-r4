# ORDER: one-model-end-to-end, claude-351

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `999cdc087e67c58c8de0a6ce8dac2333f744a7b3`.

## Goal

**Build the place read (arm p) as stage 1 routed it, with arming rule D3: in the float trainer and in the integer serve, with the export, float against integer parity, gradient checks and a one-page sheet. No pod and no training in this unit.**

**Why now.** [claude-350](../claude-350/REPORT.md) counted what the build must contain and put one choice to the owner. On 11 Nov (UTC), in chat, the lab asked "Build stage 2 of the place read, and with which arming rule?" and the owner answered **"Build with D3"**. That approves this build, with D3. It does not approve a pod: stage 3 runs only when the owner says run. The hold of 10 Nov stands for every other arm (PR #3913 stays a draft).

## What is built

Each row is stage 1's routing ([claude-350's ORDER](../claude-350/ORDER.md), "The arm" and "What stage 1 decides").

| Part | As routed | New here |
|---|---|---|
| Find | Two ask heads. Each asks with two halves: the previous token's half plainly, the current token's half with the head's learned row (C3w). | The two-half question for ask heads. |
| One position | Each head reports its nearest position, the later on ties. A row's find is the later of the two heads' positions. | The rule for ask heads; today they report a mean of 16. |
| Mask | The find sees user-turn positions only, an exact mask from the role markers. | The mask. |
| Arm (D3) | A row's find sets the register only if the row's token fires: one learned gain per question token, firing when it is above zero. The last firing row of the turn wins. | The gain and its rule. |
| Hold | One place register per conversation, an integer. The next user turn clears it. | The register. |
| Open | A reply row opens the read when its token's gain is above zero and the register is set: one learned gain per reply token. | The gain. |
| Read | At an open row the register offers the shared table's point of the token at its place, through the copy read's readout mix. | The source. |
| Step | At an open row, when the reply's token equals the token at the place, the place moves on by one and the next row is open. A miss closes that row only. | The step. |
| Stop (B1) | After a step, the read ends where the token at the place starts a new word or is punctuation, or the turn ends. The register then clears. Nothing is learned. | The stop. |
| Label | Places from TRAIN's oracle twin, as stage 1 read them. | A places file the trainer reads. |
| Window | 256 tokens in training and in serving. | The window. |

## Decisions the build fixes

Stage 1 left these to the build. Each is stated here before any code, and none changes a bar.

- **One register, one program.** The hold, the arm, the open, the step and the stop are one integer program. The trainer runs it teacher-forced over the window's tokens and the serve runs it over the emitted tokens. No gradient passes through it.
- **When a head finds.** A head finds when its nearest user-turn position is inside the head's margin, the test the serve's `LatestNearest` rule already makes. Else the head finds nothing at that row.
- **Two heads, one pull.** Each row of a labelled question pulls one head toward the key of the labelled place: the head whose question is nearer that key. Only the question rows learn, as in the pull on `main`.
- **Both gains are counts.** Each gain is a signed count per token that saturates, read by its sign, as stage 1's majority rules were. The arm count goes up at a question row whose find is the labelled place, and down at a row whose find is another place or stands before a reply with no label. The open count goes up at the value-start row when the register is at the labelled place, and down at any other reply row while the register is set; rows the step opens are left out. No gradient reaches either.
- **The source replaces, it does not add.** At an open row the register's token is the mix's source in place of the copy heads' mean. The mix's weight `w_c` is the only part of the read the training loss moves.
- **One table for roles and the stop.** A class per token (starts a user turn, starts a reply, ends a turn, stops a read) is built from the tokenizer at training and written to the artifact, so the serve reads roles and the stop from the token stream alone.
- **Off is main.** With the arm off, every output and every artifact is byte-identical to `main`.

## What a pass is

| # | Gate | Check |
|---|---|---|
| G1 | The register program reproduces stage 1. Driven by the census's own finds, it gives D3's line: the true place on 426 of the 436 reached, a wrong place on 30, set on 2 general replies and 5 abstentions; opened right on 425 of 426; first open on 413 of 562. | a test on the panel of claude-349, on the laptop |
| G2 | Float against integer: on a small model built with no GPU, the trainer's forward and the serve give the same register, the same open rows and the same read at every row. | a parity test |
| G3 | Gradients: a finite-difference check of every new differentiable part (the two-half question's row, the pull aimed at the labelled place, the open gain, the mix with the new source). | tests |
| G4 | Off is main: the crate's suites pass unchanged, and an artifact written with the arm off equals one written from `main`. | tests, and a byte compare |
| G5 | The export round trip: a model with the arm on is written, read back and served, with G2 holding on the read-back artifact. | a test |
| G6 | `uor-harness ratchet d23`, `d25`, `d32`: no additions. | the three commands |
| G7 | A one-page sheet for stage 3 with a FIT from a reader who is not its author. | the sheet and the reader's verdict |

## Amendments made during the build (11 Nov, claude's own decision review; logged for Casey, who can reverse them)

Each is a fact the build met, the change it forces, and why no bar moves.

1. **The one program lives in `crates/uor-r4-integer/src/fbs_snapshot.rs`.** The integer crate has two builds that share no engine code: the serving build (`--features serving-wasm`, the `stack` engine) and the default build the float trainer links. `fbs_snapshot.rs` is the one module both compile, and it is in the item's `scope:` line. So the register (`PlaceState`), the turns (`PlaceTurns`), the pick (`place_pick`) and the artifact's record (`PlaceMarks`, `PlaceImage`) are there, and `stack` re-exports them. This is "One register, one program" made literal: the trainer and the serve call the same compiled functions.
2. **"One table for roles and the stop" is a record and one table.** Role markers are several tokens long ("User:" after a newline), so a class per token cannot name a turn. The artifact's header carries the marker token runs (`PlaceMarks`: begin, end, separator, the three role markers) and one section, `hamming.d{depth}.place`, carries three bits per token: stops a read, arms the register, opens the read. The serve still reads everything from the token stream alone.
3. **G3 has no open-gain check.** The decision "Both gains are counts" makes the arm gain and the open gain saturating counts with no gradient. G3's list named "the open gain" in error. G3 checks the three differentiable parts: the two-half question's row, the pull aimed at the labelled place, and the mix with the register's source.
4. **G1 is read on the dynamic program, and it is exact.** The census counts each reply row against the register as the reply's start left it; the program's register also steps and stops. Run token by token over the panel with the census's own finds, the program gives D3's line exactly (426, 30, 2, 5; 425 of 426; 413 of 562), no reply's register had left the true place at its value-start row, and all 425 opened values were read whole, none short and none long. The output with the program's lines removed hashes to claude-350's `census.out`.
5. **G2 and G3 split at the GPU.** The vendored candle CPU backend refuses every tensor operation (D24, D29), so the trainer's forward and its gradients cannot run on the laptop, and this unit has no pod. The repo's standing form for such checks is a `#[cfg(feature = "cuda")]` test run on a pod.
   - **G2a, on the laptop:** the register, the open rows and the offered token come from the same compiled program in the trainer and the serve, so they agree when its inputs agree. The tests are: the packed codes the trainer hands the program equal the codes of the exported artifact for every token and head; and the integer mix at an open row equals the float form `n + w(c − n)` within half a step.
   - **G2b and G3, on the pod:** the trainer's forward against the served artifact row by row, and the three finite-difference checks, are written here as `cuda` tests and compile under the stub. They run as stage 3's first step, before any training, and a failure there stops stage 3. **This unit does not run them and does not claim them.**
6. **Files in scope gain `crates/uor-r4-integer/src/fbs_snapshot.rs`**, already in the item's `scope:` line.
7. **G2a is corrected.** Amendment 5 named a laptop test in which the trainer's packed codes equal the exported artifact's. The trainer's codes come out of its forward, which needs the GPU, so that equality belongs to G2b. G2a on the laptop is three things:
   - the trainer and the serve call the same compiled `place_find`, `place_first` and `PlaceState`;
   - six host tests cover what the trainer adds around them: a row's bits packed as the served code, the nearer head on a tie, the votes, a window's labels, the saturating count and the saved image;
   - the two mix tests in `stack/tests.rs` hold the integer mix at an open row to the float form.
8. **G2b has two parts, both on the pod.** A `cuda` unit test runs the trainer's forward on a hand-built conversation and checks the open row, the offered place and every other row against the program. The bins road then compares `fbs-snapshot`'s float readouts with `fbs-compare` on the exported artifact. On a smoke's checkpoint the register does not arm yet, so the sheet reads float against integer again on the trained checkpoint.
9. **G3's true form.** Two of the three differentiable parts pass their gradient straight through a sign: the question's row and the place pull. A finite difference of a sign is zero almost everywhere. So those two are checked by where the gradient lands and by descent: the pull falls to zero in four steps and the step's own find then votes for the place. The mix is smooth, and is checked against its closed form and a finite difference.
10. **Decisions the trainer fixes.** None changes a bar.
    - Under the place read the readout read's ask heads write nothing. Their margins start at an eighth of a head's width, in bits.
    - Both counts saturate at 64.
    - The place pull is on the learned half of each ask head, at every row of a labelled question, for the nearer head (the lower on a tie), with the key and the rotation held fixed. The older found-copy pull skips a read that has the place read.
    - Labels come from a kit file written by `--turn-census … --places-out`. The kit names its corpus by the SHA-256 of its tokens, and the trainer refuses a kit read from another corpus. The flags are `--hamming-place KIT` and `--hamming-place-pull λ`.
    - A checkpoint without the place read gains one at zero counts, a warm start. A checkpoint with one must have the kit's marks and stop tokens. `--recall` and the modes that do not train refuse the flag.
    - When a step is split into micro-batches, the pull is weighted by each part's share of the step's windows, not of its question rows.
    - Each checkpoint adds one line to `place.jsonl` in the run's folder, the reading since the last one: tokens armed, tokens opening, the arm votes and the open votes up and down, and the mean pull. (Corrected by amendment 14.)
11. **The serve's find looks over the whole session, not 256 tokens.** The routing's row "Window" set 256 in training and in serving. The trainer's window is its `--context`. The serve keeps every token of the conversation up to the artifact's context, as it does for every other read (arm r trained at 192 and is served at 8,256), and its find sees every user-turn position among them. A session refuses a step at its context and never drops a token, so a place never moves under the register. Finds further back than a training window are not measured by any count so far. The sheet names this as a risk; no bar moves.
12. **Files in scope gain the two exporter binaries,** `crates/uor-r4-float-train/src/bin/fbs-snapshot.rs` and `crates/uor-r4-integer/src/bin/fbs-export.rs`, both in the item's `scope:` line, and the sheet, `docs/harness/prep/one-model-end-to-end-place-read-2026-11-11.md`, under the item's `docs/harness/prep/one-model-end-to-end*`. `turn_census.rs` is in this ORDER's list and not in the item's line, as it was for claude-350.
13. **G4's byte compare of a real export moves to the sheet.** No trained snapshot is on the laptop. Here G4 rests on four things: the suites pass unchanged, among them `main`'s own byte tests (`read_free_fbs_exports_are_unchanged_by_the_hamming_format`, `snapshots_without_hamming_reads_are_unchanged`); the artifact test shows an artifact without the place read has no place record and that adding the read leaves every earlier section's bytes alone; a snapshot without the read is written by the same call as before; and the census output without the new lines hashes as claude-350's. The sheet's check I-o exports arm r's sealed snapshot with the pinned exporter and compares it with arm r's sealed artifact.

### After the first review round (six fresh reads of the spec and the diff)

14. **The place reading was wired to a run that refuses the place read.** The reading had been added to the recall run's record, and `--recall` refuses `--hamming-place`, so no training run would have written it. It now goes to `place.jsonl` in the training run's folder, one line a checkpoint, and the recall run's record is as on `main`. The wiring itself cannot run on the laptop; the sheet's check I-s reads the file on the smoke.
15. **What D3 means when a firing row finds nothing.** Such a row records nothing, so the register is set by the last firing row that found a place. The census reads D3 the same way, which is why G1 is exact.
16. **Which reply rows vote on the open count.** With the register at the labelled place, the value-start row votes up, each row before it votes down, and the rows after it do not vote. With the register anywhere else, at another place or in a reply with no label, every row of the reply votes down. A row the step opened does not vote. A label's value-start row may lie past the window; it is only compared with a row.
17. **A window with no known corpus start teaches nothing.** The trainer cannot see that window's labels, so "no label" there would be a guess. Before this amendment such a window voted every find down.
18. **Votes are cast at each forward.** A training step split into several forwards runs its later forwards on the counts its earlier ones moved. This is kept as it is: the counts are a running tally, and the split is a memory setting, not a part of the method.
19. **A reply answers the latest user turn once.** A second model turn with no user turn between asks nothing and sets nothing. A system turn between the question and its reply changes nothing.
20. **Refusals added.** Marks the turn reader cannot tell apart are refused when an artifact is read, when the read is added and when the trainer builds its tables: an empty separator, user marker or assistant marker; one token for the start and the end mark; a role marker that ends with another. Forced codes are refused on an artifact with a place read, because an open row's readout takes the place and would pass over them. `--places-out` refuses a path that exists. A checkpoint folder with a place read's state and no reads beside it is refused; the place state is written first, so a save cut short cannot leave reads that have silently lost their place read.
21. **What G1 checks, and what it does not.** The program on the panel is driven by the census's own finds, arm tokens and open tokens. So G1 checks the program's turns, arm, hold, clear, open, step and stop against stage 1's counts. It does not check the finds, which both sides take from the same code.
22. **Two findings kept as they are, with the reason.** The kit's corpus hash stays the caller's to give: it is the hash of the `tokens.u16` file, header included, which is what a training run checks its corpus by, so it cannot be computed from the tokens alone. The found-copy pull's per-depth reading is one entry short under the place read; its one caller, the recall run, refuses the place read.

### After the second review round (four fresh reads: the sheet, the fix range, the GPU tests, the integer tests)

23. **The census and the kit pair a reply as the register does.** Amendment 19 changed the register and left the census's pairing as it was, so on a document with two model turns in a row the kit would have labelled a reply the register never arms. The census now gives such a reply no user turn, so it is no case and has no label. A new test fails with the old pairing. On the real corpus nothing moves: the program's four lines, the count output's hash and the kit's bytes are the same, so no such reply is a case in TRAIN or on the panel.
24. **The kit refuses bad marks when it is built.** `place_kit` runs the marks' check with the vocabulary it is given, so `--places-out` no longer writes a kit that a run would refuse at its first step. The real marks pass it.
25. **A checkpoint's place line has a host test.** The line is now `PlaceRead::line`, tested on the laptop: the reading and the step, and no mean pull when no step pulled since the last line. Its call from the training run is still unrun (sheet, I-s).
26. **The GPU tests, read by hand.** A fresh reader traced the three GPU tests and left three points open; the author closed them from the code, and none is run. The pull's pairs are every row of the question, a range, so the gradient reaches both question tokens. A closed row runs the same arithmetic with and without an armed register, so its values are equal bit for bit. After descent only the "?" row finds the place: the row before it stands after the user marker, so the second token of its question matches no candidate, and one token gets the vote.
27. **Round-2 findings kept as they are, with the reason.** No test pins the order of the two writes in a save, and a step folder that is written again and cut between them would load new counts beside old reads; a run writes each step folder once, and this is listed as open. The session hands the register's offer to the readout in both of its step paths; that is read, not pinned by a test, and the timed step is not run with a place read. Two round-1 tests cover behaviour and pin no change. One integer test is renamed to what it pins.
28. **The sheet's second fitness read was NOT FIT** on one point: the two hand reads named neither the replies nor what the count decides. The sheet now fixes both, and answers the read's four smaller points. The read is in the sheet word for word.
29. **The sheet failed CI's harness tests and a third fitness read, and then passed a fourth.** At `bfde1cbe040a025e1dbfae4327bb5c6198c965e1` the sheet had no "Steps" section and no `TEST FITNESS:` line, which the harness reads from every sheet; both are added, and the harness's sheet tests pass on the laptop. The third read, of amendment 28's changes only, was NOT FIT on two points in the retake those changes added: the whole-word score counted a value inside a longer number, and two of the four artifacts were rescored. The score is now a rule on the reply's text with a fourth test reply (check I-w), and a retake rescores all four artifacts once. The fourth read, of those changes only, was FIT with no blocking point. Three wordings it asked for were made after it and are marked in the sheet as read by the author only. All four reads are in the sheet, the last three word for word.

## Not established by this unit

- Nothing is trained. No gate shows that a trained model takes the register's token, that the question rows learn their targets, or that the gains settle. Only stage 3 can.
- G1 drives the register with the census's exact finds, not a model's. It checks the program, not the find.
- The trainer's forward and its gradients are not run here (amendment 5): G2b and G3 are written and compiled, and run on the pod as stage 3's first step.
- The 126 recall replies the find cannot reach stay out of reach.

## Cancel probe (run first)

- **What would make this unit unnecessary:** a read on `main` that holds a position across rows.
- **Result (at claim):** none. `git grep -n -i -E "place[_ ]register|place[_ ]read" HEAD -- crates/uor-r4-float-train/src crates/uor-r4-integer/src/stack` gives lines in `turn_census.rs` only, stage 1's counts.

## Claim

Written by hand, as claude-347's to claude-350's were: claude-346 (PR #3913, a draft under the hold) holds the item for `uor-harness loop claim`.

## Files in scope

- `docs/loop/tasks/one-model-end-to-end/claude-351/**`
- `crates/uor-r4-float-train/src/flock_read_hamming.rs`, `geometric_stack_native.rs`, `in_space_long.rs`, `fbs_export.rs`, `turn_census.rs`, `bin/train-in-space.rs`
- `crates/uor-r4-integer/src/stack/hamming.rs`, `mod.rs`, `session.rs`, `format.rs`, `synthetic.rs`, `tests.rs`, and `generation.rs`
- `crates/uor-r4-integer/src/fbs_snapshot.rs` (amendment 1)
- `docs/research/notebook/**` (one row a result)
- A new source file only if the item's `scope:` line gains its path in the same commit.

## Inputs, all on the laptop

- The panel and TRAIN of claude-349 under `local/claude/claude-349/`, and TRAIN's twin and the count output of claude-350 under `local/claude/claude-350/` (`census.out`, SHA-256 `90ecd6881ae50f0af9dfc70c69530ef626bd55de329b7b7430b2becddf7b3bae`). No model reads them in this unit.
