# REPORT: one-model-end-to-end, claude-365 (a question never told finds no place: a floor for each question form)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [ORDER.md](ORDER.md). PR #3955.

The order asked for fewer replies that answer a question never told with a told value, without losing the questions that were told. The served table now holds a floor for each question form seen often in TRAIN. On a panel written after the bar (seed 22), all three saved models say "not told" on 72 of 73 never-told questions, against 54 with the merged table, and state the told value on 11 more of 582 told questions. **The bar reads "Adds". The nine-turn conversation did not change: turn 8 is still wrong, and it is not this failure.**

## What did not work, first

1. **The conversation is word for word as before.** "What is two plus three?" is still answered "It's 18." The order named that turn as its reason, and the reason was wrong. On that turn the table finds nothing at all (`uor-chat --print-place`: `held=none offered=none`), because it does not class the question as one that asks for a stored fact: the asking score is about −225 against a floor of −87 (the tokens " two" and " three" weigh −111 and −113; "plus" is not a word of the training world). The reply is the model's own. A floor on places cannot change it.
2. **Where the turn's wrong reply comes from is only partly located.** With turn memory off the reply is the same "It's 18.", so it is not turn memory; it is the model's reply with no place offered. Turn 6's "Feeling sad is druvalk?" does come from turn memory: with it off the reply is "Okay, I'll remember that." (and turns 1 and 3 get worse). Neither is fixed here.
3. **The second form rule did nothing.** Leaving out tokens held by fewer than 64 of TRAIN's questions kept 952 tokens, merged 26 forms of 41,799 and gave the same count on every split. The threshold is far too low to leave out the pieces of an invented key. Only the first rule (the question's distinct tokens) is built. A question whose exact form TRAIN does not hold, such as one about a key never seen, keeps the one floor.
4. **What the floors leave.** By the table's count on the fresh panel: 1 of 73 never-told questions still gets a place, 13 told questions get a wrong place (unchanged), 5 are answered "not told" (16 before), and 10 other replies are answered "not told" (unchanged). On an older panel wrong places rose by 2 (seed 20: 11 to 13).
5. **The least count, 16 cases a form, was set once and not swept.** 678 of 41,799 forms have a floor.
6. **The builder stopped at its context limit before its last check.** The main session ran that check (the least-count rule set aside) and committed the work.
7. `cargo clippy` and `cargo fmt --check` were not run on the laptop.
8. **Whether floors counted from TRAIN may be served in a product** (CHARTER §3.1) is the owner's open question, as it is for the table they join.

## What changed

- **The table** (`crates/uor-r4-integer/src/place_asks.rs`): `PlaceAsks::ask_floors`, a floor for each question form (the question's distinct tokens, ascending). A table that holds none is written without the list, so a table of today reads and writes byte for byte as today. The index refuses a form named twice, out of order or past the vocabulary.
- **The find:** a question whose form holds a floor is picked against that floor; any other against `floor` or `floor_other`, as before. Nothing else in the find changed.
- **The count** (`fit_forms`): TRAIN's reply cases grouped by form; a form with 16 or more cases that have a candidate gets the floor the fit's own routine gives those cases alone.
- **The tool** (`place-asks forms`): the count for one of two rules, the census of TRAIN and of each panel given with and without the floors, and with `--out` the table with the floors added (rule `tokens` only; never written over).
- Table literals gain the empty list in `fbs_snapshot.rs`, `stack/tests.rs` and the instrument's `raw-id-replies.rs`. No change to the trained artifact.

## Measurements

**Step 1, the count before any build** (`local/claude/claude-365/forms-tokens.log`; `forms-frequent.log` is the same on every line). The table's find with the one floor, then with each form's own. Floors fitted on TRAIN only.

| Split | Told: right place | wrong place | false "not told" | Never told: place set | "not told" | Other replies: place set / "not told" |
|---|---|---|---|---|---|---|
| TRAIN (fitted on it), 34,460 / 4,628 / 175,929 | 33,234 → 33,570 | 594 → 630 | 583 → 211 | 1,198 → 212 | 3,430 → 4,416 | 14 / 480, unchanged |
| Seed 21, 564 / 76 / 2,884 | 542 → 542 | 16 → 16 | 5 → 5 | 20 → 6 | 56 → 70 | 0 / 12, unchanged |
| Seed 20, 581 / 82 / 2,953 | 563 → 565 | 11 → 13 | 6 → 2 | 26 → 3 | 56 → 79 | 0 / 5, unchanged |
| Seed 19, 557 / 75 / 2,906 | 530 → 536 | 11 → 11 | 14 → 8 | 20 → 3 | 55 → 72 | 0 / 7, unchanged |

**The go rule** (never-told questions with a place fall by 6 or more and told questions at their right place fall by no more than 3, on each read panel): falls of 14, 23 and 17; right places 0, +2 and +6. Cleared.

**Step 3, the bar, on the seed-22 panel** (written 2026-11-13T13:46:31Z, after the order's commit at 13:29:10Z; 975 rows read, 11 left out as equal to an older panel's; read once per arm). The three saved models served again, as merged. S is of 73 never-told rows, V of 582 told questions, O the other rows exact.

| Model | S: merged table | S: with floors | V: merged | V: with floors | O: merged | O: with floors | All replies exact, of 975 |
|---|---|---|---|---|---|---|---|
| p-s2 | 54 | 72 | 545 | 556 | 260 | 274 | 341 → 355 |
| p-s1 | 54 | 72 | 545 | 556 | 252 | 266 | 333 → 347 |
| p-s3 | 54 | 72 | 544 | 555 | 261 | 275 | 342 → 356 |

S rises by 18 on all three models; V and O rise. **Adds.** Repeat rows are exact on 81 of 84 in both arms. By the table's own count on this panel (read after the models): told questions at their right place 553 → 564, never-told with a place 19 → 1.

**The nine-turn conversation, p-s2** (`uor-chat` at this head; the same replies with the merged table and with the new one, and the same as claude-364's record):

1. "Keep these in mind: druvalk is 73, mibtorn is felsharp, quonzet is 18." → "Got it."
2. "What is mibtorn?" → "felsharp."
3. "What does a cow say?" → "A cow says moo."
4. "Say exactly this: trelvon dask pimmerholt zane" → "trelvon dask pimmerholt zane"
5. "What is druvalk?" → "That's 73."
6. "I'm tired today." → "Feeling sad is druvalk?"
7. "Tell me what quonzet was." → "It's 18."
8. "What is two plus three?" → "It's 18."
9. "What is my name?" → "I'm not sure, you didn't tell me."

## Checks run at the head

| Check | Result |
|---|---|
| `cargo test --locked --offline -p uor-r4-integer --features serving-wasm --lib --bin place-asks`, at the head | 602 passed, 1 ignored (594 at base; 8 new); the tool's 5 pass |
| The same with `--no-fail-fast` over every target, one commit before the head | library 601 passed; 8 integration targets fail on this laptop at their first line, loading a real bundle file that is not here (`No such file or directory`); this branch changes no integration test |
| The least count's edge: `<` changed to `<=`, the new test at exactly 16 cases | fails, as it must |
| G1, a table of today: the new table less its floors equals the merged table (`jq -S` of both); a table with no floors is written without the list (test) | holds |
| G1, the seed-21 replies of the three models with the merged table, read by this head's reader | equal claude-364's files, all three |
| G2, a form under the least count takes the one floor: the test with the rule set aside | fails, as it must |
| G3, the find takes a form's floor for that form only: two tests with the lookup set aside | both fail, as they must |
| The table's own floors are read by the index: the test with the load set aside | fails, as it must |
| The written table through `place-asks census` on seed 21 | the count's own lines (6 of 76 with a place) |
| G4, the fresh panel's time | after the order's commit |

**Review.** One fresh reviewer (`delegate:reviewer`, no part in the work) was given the order's rules and the code diff against `origin/main` at `d5d50acb8b71b0ba6c8b415377c70f437cb0fe13` (5 files, 399 lines added) and nothing else. It read the count against the fit's own routine line by line, the find, the order of every written and printed row, the load checks, the tool's four conditions for writing and the tests. Verdict: approve, no high or medium finding. Its three low findings are taken after the review: the tool now refuses `--rule frequent --out` before any count, the tool's header says what `--out` writes, and a test pins the least count at exactly 16 cases. It ran no test and no build, did not check whether another reader hashes a table that holds floors, and did not judge the second form rule on a form that mixes asking and other questions. The three changes after the review are small and were not reviewed again; the find is not among them, so the replies read above stand.

Head binaries (`local/claude/claude-365/bin/SHA256.txt`, built before the review's three changes; the find and the reader are the same code): reader `16a0d837…`, table tool `96a4d737…`, chat `7daf621d…`. Table with floors: SHA-256 `420e71c3b4a6fe0c1f958131f9c42f4b5064b90b49af19cb1bbc0860c33b53da`.

## Skills used

None.

## Still open

- **Turn 8 and turn 6 of the conversation: the model's own reply on a turn the table leaves alone.** Turn 8 states a told value for a question outside its world; turn 6 picks up a told key through turn memory. Next unit, sized by count first.
- Turn 2's bare value ("felsharp." with no lead-in).
- A question in a form TRAIN does not hold keeps the one floor; a rule that reads a question's wording apart from its key would cover it.
- The 13 wrong places and 15 false "not told" on the fresh panel.

## Plain summary

When someone asks about a thing they never told the model, the model should say so. Before this change it did on 54 of 73 such questions and gave a made-up answer on the rest; now it does on 72 of 73, on questions it had never been tested on, and it also answers 11 more of the questions it was told. The change is in the look-up table the model is served with, not in the trained model. Our hand-written nine-turn conversation did not get better: the turn we hoped to fix ("What is two plus three?" answered "It's 18.") turned out to be a different fault, in what the model itself says when the table has nothing for it. That is the next thing to fix.
