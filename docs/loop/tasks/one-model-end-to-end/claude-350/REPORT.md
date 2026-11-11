# REPORT: one-model-end-to-end, claude-350

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [ORDER.md](ORDER.md). PR #3933.

Stage 1 of the plan for the repaired arm, the place read: the question finds the place where the value starts, a register holds that place, and the reply reads on from it. Seven counts on the laptop route what stage 2 would build. No pod, no training, no model was run, and no bar or status changed. Stage 2 is the owner's call. Every claim below is [claude]'s.

Every fitted part is fitted on TRAIN's places and read on the panel's 562 recall replies. That reading is the one in the table. Every count is taken on the written reply's own tokens: nothing is generated.

| Row of the ORDER's rules | Reading | Routes |
|---|---|---|
| Instrument | every recall reply has a place: 562 of 562 | pass |
| Stop | 25 of 562 values are one token; B1 gives the exact value on 562 of 562 | B1: stop at a new word, at punctuation or at the turn's end. Nothing is learned. |
| Find | **misses its 0.80 line.** C1 to C3 at best: C3 on user turns, 419 of 562 (0.746). Best of the five forms: C3w on user turns, 436 of 562 (0.776) | by the rule's second clause C3w, masked to user turns, is built. **It cannot reach 126 replies (0.224).** |
| Arm | D1: the true place on 426 of the 436 reached (0.977); the register is set on 263 of 1,945 general replies (0.135) and 7 of 79 abstentions (0.089) | D1 |
| Open | under D1, right on 419 of 426 (0.984); wrong on 0.010, 0.005 and 0 of the reply rows of recall replies, general replies and abstentions | a learned gain per reply token |
| Label | **misses its 0.90 line.** The label read from the tokens equals the true place on 341 of 562 (0.607) | stage 2 takes its places from the oracle twin |
| Window | 113 of 562 places (0.201) are more than 192 tokens back; none is more than 256 | 256 |

**Two registered lines are missed, the Find's 0.80 and the Label's 0.90. No stopping clause applies and no row sends a plan change.** These are counts of integer rules with nothing trained.

## What did not work, first

- **The find misses its 0.80 line with every form.** One ask head reaches 419 of 562 at best (C3). Two heads reach 436 (C3w). The rule's second clause (at or above 0.50) builds C3w and has this page name the share it cannot reach: **126 of 562 replies (0.224)**.
- **What the find cannot reach is mostly questions about the user's own facts.** All 126 missed questions are listed in the output, each with the statement it asks about. 116 ask about the user ("Where am I from?" 13, "Where did I grow up?" 11, "What is my name?" 11, "What's my friend called?" 10), many in other words than the statement ("Where did I grow up?" against "I am from Jaistchukfield."). 10 ask for a name from a list. The causes are counted for all 126 together, not by group: on 93 no row of the question finds anything; on 21 rows find positions but the place fits none of them; on 12 the place fits a row and a later position fits it too. So on at least 104 of the 116, no row's question fits the place at all.
- **The key on main is not enough.** A zero row (the question's own token stands before the value, the key arm r's copy read uses) reaches 69 of 562. One learned target a token (C2) reaches 201. The two-token question does the work, and it is not on main.
- **The rule picks the arming rule that leaks more.** D1 passes its three lines, so it is the one routed. It leaves the register set on 263 general replies and at a wrong place on 43 recall replies, and with it the open rule opens 100 general reply rows. D3 (a learned gain per question token) sets it on 2 general replies and at a wrong place on 30, and opens 2 general reply rows. D3 is not better on every count: with TRAIN's fit it ties D1 on the true place (426) and is one reply behind on the whole-reply count below (413 against 414); with the panel's fit its true place is 421 against 428. Building D3 instead of D1 departs from the rule as written, so it is put to the owner.
- **A reply that opens the right row can open a wrong one first.** The value-start row is opened with the register at the true place on 419 recall replies under D1. On 414 of 562 (0.737) it is also the first row opened; on the other 5 a row before it opens too. Under D3 the two figures are 425 and 413.
- **The token label counted here misses its 0.90 line.** One rule was counted, the longest shared run, with stop lists of 0, 16, 64 and 256 tokens. With the registered list of 16 it holds the value on 557 of 562 but starts at the value on 341 (0.607), and the 0.90 line is on the start. It also fires on 587 of 1,945 general replies (0.302). That closes this rule with these lists, not every label that could be read from tokens. Stage 2 can train on TRAIN's twin (34,460 places). Conversations that come with no twin have no label yet.
- **The 192-token window is too short.** One place in five is further back. The register has to cover 256 tokens in training and in serving.
- **The panel's own fit is not a ceiling for C3w.** Fitted on the panel's 562 replies, C3w reaches 429; fitted on TRAIN, 436.
- **Count A cannot fail on its own.** A place is defined as a run in the turn equal to the reply's run, so the read-on from it reads the value by construction. A's information is the number of recall replies with a place and the cause of each one without; here every reply has one.
- **The open count stops at the value-start row.** When the register is at the true place, the rows after the value-start row are not asked. A row that would open again after the value is not counted, so no figure here is a count of replies read out whole.
- **What the counts cannot show.** Nothing is trained. No count shows that the question rows learn their targets, that a trained model takes the token the register offers, or what the read does to general replies it leaves alone. claude-339's carry did not move the language run, and its replies end in a repeated-token loop whose cause is unknown. A learned row is counted as one or two target tokens; a trained row is a point and may sit near more, or fewer.
- **The values are single made-up words and numbers.** B1 stops at a new word, so it would cut a value of two words after its first. This corpus has none.
- **An older helper orders its search.** The reply's run is looked for with a space before it first, then bare. A reply that writes its value bare and later with a space would be read at the later run. This is claude-349's code and was left as it is; how many panel replies state their value twice was not counted.
- **The program changed after its first run, twice.** Before review: the trace of the misses and the Open count under the two arming rules the Arm row did not choose were added. After review: each question row has one find (the first run let either head's find count, which is more generous than the ORDER's words); the Instrument row prints its decision; the window is measured from the value-start row; the whole-reply count was added; every miss is listed. No decision changed; the Instrument row's line is new, and the Open line now names its arming rule. The one form the row reading changed is C2w (220 on user turns against 236 with either head, 133 against 165 over every position); C3w is 436 either way. The first run's output was overwritten and no hash of it was kept. After the second review only a test and two comments changed in the program.

## What changed

- `crates/uor-r4-float-train/src/turn_census.rs`: the place census (`place_census`), after claude-349's census, which is untouched. Its parts are public and tested: `cases` (the replies counted, and each recall reply's true place), `targets` (what a learned question row asks for), `finds` and `rows` (what each question row finds under a form), `register` (the three arming rules), `open_rows`, `run_label`.
- `crates/uor-r4-float-train/src/bin/train-in-space.rs`: `--turn-census` takes a fourth folder, TRAIN's twin written `recall=oracle`, and prints the place census after claude-349's lines. Only that mode changed.
- [ORDER.md](ORDER.md): the second pre-count review's eight findings and three author's changes, committed before any count ran (`ccfacc9009b8b6b608090cf812af3ad231a80772`).
- One row in the [float-training notebook](../../../../research/notebook/float-training.md).

No bar, status, scope or stop rule changed. Nothing was trained and no pod was started.

## Measurements

Inputs: claude-349's three corpora (TRAIN and the panel written `recall=off`, the panel written `recall=oracle`) and TRAIN's twin, `conversations=60000 seed=13 recall=oracle`, written on the laptop by the same executable as TRAIN: 7,964,285 tokens, 34,460 value lines and 4,628 "none" lines. The panel has 562 recall replies, 1,945 general replies with an earlier user turn and 79 abstentions. TRAIN has 34,460, 115,929 and 4,628.

"TRAIN's fit" means the fitted part was fitted on TRAIN's places and read on the panel. "Panel's fit" is the same part fitted on the panel's own places.

### A. The instrument

The value's run in the reply, and the latest equal run in the latest earlier user turn that states it, are found on 562 of 562 panel replies and 34,460 of 34,460 in TRAIN. No reply writes its value in other tokens than its turn does.

### B. The step and the stop

| | of 562 |
|---|---|
| Values of one token | 25 |
| Exact value under B1 (stop where the token at the place starts a new word or is punctuation, or the turn ends) | 562 |
| Exact value under B2 (a majority rule on the token at the place), TRAIN's fit and the panel's | 562, 562 |

A read that names one token returns 25 values whole. The step is needed for the other 537.

### C. The find

A reply passes a form if, at some row of its question, the latest position that fits is the true place. With two heads a row's find is the later of the two heads' positions.

| Form | Every position, TRAIN's fit | User turns only, TRAIN's fit | Every position, panel's fit | User turns only, panel's fit |
|---|---|---|---|---|
| C1, zero rows | 39 | 69 | 39 | 69 |
| C2, one learned target a token | 133 | 201 | 150 | 225 |
| C3, C2 and the token before the row stands two before the place | 395 | 419 | 398 | 421 |
| C2w, two targets a token | 133 | 220 | 143 | 229 |
| C3w, two targets a token | 412 | **436** | 403 | 429 |

- The find over user turns only is better for every form, so the build masks it to user-turn positions.
- If either head's own find counts, C2w reads 165, 236, 170 and 246; C3w is unchanged.
- Three question rows carry 395 of the 436 reached replies: "?", " was" and " again", each asking for " is", with the name's last token two before the place (a statement of the form "woutdai is 93"). The next rows are " my" asking for " named" (12), " do" for " is" (6) and " love" for " like" (5). The twelve rows printed carry 428.
- The 126 misses: no row finds anything 93, the place fits no row 21, a later position fits too 12. By question: 116 about the user, 10 about a name from a list.

### D. The arming rule, with C3w on user turns

| Rule, TRAIN's fit | True place, of the 436 reached | Wrong place, of 562 | General replies with the register set, of 1,945 | Abstentions with it set, of 79 |
|---|---|---|---|---|
| D1, the last row that finds anything | 426 (0.977) | 43 | 263 (0.135) | 7 (0.089) |
| D2, the row whose find fits the fewest positions | 423 (0.970) | 46 | 263 | 7 |
| D3, the last row whose token fires under the majority rule | 426 (0.977) | 30 | 2 (0.001) | 5 (0.063) |

With the panel's fit, of the 429 reached: D1 428, 45, 262, 3; D2 426, 47, 262, 3; D3 421, 24, 2, 1. D1 is the simplest rule that meets all three lines (0.90, 0.15, 0.10), so the rule routes D1.

### E. The open rule

Right: the value-start row is opened, of the recall replies where the register is at the true place. Wrong: another row is opened while the register is set, of all reply rows of that kind (5,738, 21,438 and 922). First: recall replies, of 562, where the value-start row is opened with the register at the true place and no row before it is opened.

| Arming rule, TRAIN's fit | Right | Wrong, recall | Wrong, general | Wrong, abstention | First, of 562 |
|---|---|---|---|---|---|
| D1 (the Open row reads this line) | 419 of 426 (0.984) | 60 (0.010) | 100 (0.005) | 0 | 414 (0.737) |
| D2 | 416 of 423 | 63 | 100 | 0 | 412 |
| D3 | 425 of 426 (0.998) | 48 (0.008) | 2 | 0 | 413 (0.735) |

With the panel's fit: D1 414 of 428, 57, 93, 0, first 409; D3 421 of 421, 31, 2, 0, first 417. Under D1 the open rule fires on three reply tokens: " is", "'s" and " love".

The "First" column is the nearest this page has to an end-to-end figure, because the stop is exact on every value: under D1, on 414 of 562 recall replies (0.737) the register is at the true place, the first row opened is the value-start row, and the stop ends the read after the value's last token. Rows after the value are not asked, so it is not a count of replies read out whole, and it says nothing of a trained model.

For scale, on the same 562 replies claude-349's copy read took the whole value on 140 with the true turn admitted (0.249) and on 59 with no turn admitted. On its 488 labelled replies it took 135 (0.277). These three figures are from claude-349's notebook row, not from this output. Those are counts of a different read, and neither is a trained result.

### F. The label read from the tokens alone

The longest run of the reply equal to a run in an earlier user turn, starting outside the stop list (punctuation and TRAIN's most frequent tokens).

| Stop list | Split | Labelled | Equals the true place | Holds the value at the same offset | Fires on general replies | Fires on abstentions |
|---|---|---|---|---|---|---|
| 16, registered | panel | 562 of 562 | 341 (0.607) | 557 | 587 of 1,945 | 21 of 79 |
| 16, registered | TRAIN | 34,460 of 34,460 | 21,661 (0.629) | 34,207 | 34,713 of 115,929 | 1,412 of 4,628 |
| 0 | panel | 562 | 308 | 557 | 868 | 39 |
| 64 | panel | 490 | 234 | 439 | 227 | 5 |
| 256 | panel | 286 | 144 | 210 | 28 | 0 |

### G. The window

The distance in rows from the value-start row back to the place (a place 192 rows back counts as within 192), of the replies that have a place (all of them here).

| Split | Within 192 | 193 to 256 | 257 to 512 | More |
|---|---|---|---|---|
| panel | 449 | 113 | 0 | 0 |
| TRAIN | 27,487 | 6,973 | 0 | 0 |

## Checks run at the head

- `cargo test --release --locked -p uor-r4-float-train --lib turn_census`: 14 passed, 7 of them claude-349's. The seven new tests pin: one recall reply end to end through every count and `decide` line (traced by hand before the run); a find that is the latest fit and passes over a model turn when masked to user turns, and a learned row that asks for its target and not for its own token; the token label's longest run and its tie to the later turn; a space token that joins the label only when it stands before the run in both; two heads on one row giving one find at the later position; a recall reply to a conversation's first question kept as a listed miss; four recall replies through the census, where a row's two heads give one find and a reply that opens a row before the value-start row is not a first open.
- That last test was run against the code with the fix set aside, and fails both ways: with the one-find-a-row merge taken out where the census counts C and D, and with the first-open count loosened to any open.
- claude-349's lines reprint byte-identical: the output without the `place-census` lines has SHA-256 `a0856506ba182c15b4a3ded8de0569a09b539e9bc82ec575b3033d61f35afd5b`, the hash recorded for `local/claude/claude-349/census.out`.
- The count was run three times across rebuilds, the last with this head's program; the outputs are the same file: `local/claude/claude-350/census.out` on the laptop, SHA-256 `90ecd6881ae50f0af9dfc70c69530ef626bd55de329b7b7430b2becddf7b3bae`, 179 `place-census` lines, 126 of them the listed misses of the find. Every number above was checked against it.
- The census agrees with the writer's receipt for the twin: 34,460 value lines and 4,628 "none" lines. The panel's three kinds add to 2,586, its 3,586 replies less the 1,000 with no earlier user turn; TRAIN's add to 155,017, its 215,017 less 60,000. No recall reply or abstention answers a conversation's first question in either corpus.
- `cargo clippy` on the crate's library and `train-in-space`: nothing in `turn_census.rs`. `rustfmt --check`: `turn_census.rs` clean; in `train-in-space.rs` no changed line is reported.
- `uor-harness ratchet d23`, `d25`, `d32`: no additions.

## Skills used

None beyond the project loop. The fresh-context reviewers were subagents of this session. Two read the plan before any count: one its first text at `c85e30a8ae92946508071f2c68a0b1b5f7b6e196` (fit with eleven findings, all taken), one the revised text at `83b8cb21d2c302c1459f679304b6eb3fbe570780` (eight findings, all taken in `ccfacc9009b8b6b608090cf812af3ad231a80772`). Three read the census and this page at `cf85a4ab5082be1b09fdc81530f1ee35e447221f`, each fit with findings: the helpers and tests (four), the counts and rules (four), this page against the output (nine). All are taken in the commit after it, except the search order of the older helper, which is named above and left. Two more read those fixes and this page at `2b48c11585387b6753c5808c69d3109387eee4ef`, each fit with findings: four on the program (the merge was tested alone and not where the census uses it; the first-open count's denominator; what the window's distance is; a stale comment) and five on this page (a label claim wider than the one rule counted; "read out whole" claimed of the first-open count; the verdict word; the misses' wording; the source of claude-349's figures). All are taken, with one kept as it is and stated: the first-open count is of all 562 recall replies. The review record is on the PR.

## Still open

- **Stage 2 is the owner's call**, under the hold. As routed it builds: two ask heads with a two-token question, the find masked to user turns and reporting one position a row; the register, set by D1; the step, with the B1 stop; a learned gain per reply token to open; places from TRAIN's twin as labels; a 256-token window in training and serving. Stage 3's controls are named in the ORDER.
- **D1 or D3 is put to the owner.** The rule as written routes D1. D3 costs one learned gain per question token and leaves the register set on 2 general replies where D1 leaves 263.
- **The 126 replies the find cannot reach** need a question that asks for a statement worded differently. No form counted here does that, and no count sizes what would.
- **A label with no twin.** Real conversations come with no oracle twin. The token label is right on 0.607 and fires on 0.302 of general replies.
- Values of more than one word, a value stated twice in one reply, and a correction that restates a value are not counted.

## Plain summary

The plan was to have the question point at the place where the answer starts in what the user said, hold that place, and read the answer out from there one token at a time. Two of the plan's lines were missed. The question finds the right place on about three recall replies in four, not four in five, and the misses are almost all questions about the user's own facts, many asked in different words from the ones the user used. The one way tried of reading a training label off the text alone is right on about three replies in five, so training would lean on a twin copy of the data that marks the answers. The rest routes a build: reading on token by token is needed, because almost every answer is several tokens long; knowing where to stop is free; and the rule for when to read works on the replies the find reaches. Put together, on 414 of 562 recall replies the rules hold the right place, open first at the row where the answer starts, and stop where the answer ends. That is counted on the written replies, and the rows after the answer are not checked. The earlier read took the whole answer on 140 of the same 562 when handed the right turn. The simplest rule for when to hold a place also holds one on many ordinary replies; a slightly less simple rule nearly never does, and choosing it is the owner's call. Nothing is trained yet, so none of this shows what a model would say.
