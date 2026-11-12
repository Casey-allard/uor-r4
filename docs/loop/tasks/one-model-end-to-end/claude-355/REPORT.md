# REPORT: one-model-end-to-end, claude-355

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [ORDER.md](ORDER.md). PR #3945.

A laptop-only test, approved by the owner on 12 Nov ("Yes, build it"): the three stage 3 place-read models of [claude-352](../claude-352/REPORT.md) (p-s2, p-s1, p-s3) and their control (n-s2), served with the copy read taking its longest match (two tokens, else one) in place of the one-token match they were trained with. No pod, no training, no float pass; no artifact is changed and nothing served by default is changed.

**By the reading fixed in the ORDER before any reply was generated, the replies did not get better.** The part that failed: on seed 3 no repeat-a-string reply equals the wanted string under either rule (0 of 96, then 0 of 96). The other parts passed, by little: 1, 11 and 19 more recall replies of 562 state the value, 2 and 2 repeat replies of 96 become right on the other two seeds, and the other replies equal to the wanted one fall by 4 on each seed.

| Gate | Reading | Status |
|---|---|---|
| G1 | Each of the rule's forms, broken on its own (five breaks, gates a to e below), fails the rule's tests; the integer crate's tests pass at the head. | **met** |
| G2 | With no rule set, all eight reply files are byte-identical to claude-353's. | **met** |
| G3 | The counts for the four artifacts at both windows under both rules, and the changed replies word for word, are under Measurements. | **met** |
| G4 | Whether the replies got better, and what did not change, is said above and under "What did not work, first". | **met** |

## What did not work, first

1. **The replies did not get better by the fixed reading** (ORDER, decision 2): repeat-a-string replies on seed 3 stay at 0 of 96.
2. **Where a count rose, it rose by little, and nearly as many replies were lost as gained.** On p-s2, 27 recall replies newly state the value and 26 that stated it no longer do (238, then 239 of 562).
3. **The recall replies lost sit on the rows whose first reply token changed** (table 4): 21 of 26, 16 of 18 and 17 of 20. At a reply's first token the two tokens before it are the end of `Assistant` and `:`, and their latest earlier match is the previous reply's opening, so under the new rule the copy read is given that reply's first token there. On the rows whose first reply token changed (50, 54 and 45 a place-read model at the 256 window), the new first token is that token on 46, 43 and 38. Recall replies that are, word for word, an earlier reply of the same conversation and not the wanted one go from 2, 2 and 3 a model to 27, 27 and 23. This points at the opening as where the harm is; it is not shown by a run that leaves the opening alone, and none was made.
4. **A copied string is followed further, and the reply then seldom stops.** On the 96 repeat rows the reply says 10 to 13 tokens of the wanted string before leaving it (4 to 5 under the models' own rule; the strings are 28 tokens on average), and the whole string comes first on 15 to 22 replies (4 to 6 before). Only 0 to 2 of them end there.
5. **The count made before the ORDER did not carry over to replies.** On the panel's true text the two-token rule names the next value token on 1,580 of 1,954 against 1,373. That count has the true words before every token; in a reply the model's own earlier tokens stand there, and a reply's first token (point 3) is not in it.
6. **In the scripted conversation one turn of nine changed, for the worse** (below): asked fifth to say a string, the model said the string under its own rule and "You're welcome!" under the new one. The panel hardly tests this on repeat rows: 93 of the 96 have no reply before them, and on no repeat row did the first reply token change.
7. **The ORDER said 1,108 prompts at both windows.** The 256 file has 990 (562 recall, 96 repeat, 332 other); the 192 file has 1,108 (562, 96, 450).
8. **The 18-turn script of claude-345 is no longer on the laptop.** A nine-turn script in the panel's own forms was written for step 5: [chat-script.json](chat-script.json).

## What changed

- **The integer reader** ([`hamming.rs`](../../../../../crates/uor-r4-integer/src/stack/hamming.rs)): a fourth copy rule, `CopyRule::LongestExact` (`longest-exact`). Among the positions whose key equals the query exactly, it takes the latest whose token before the key also equals, by the head's code, the token before the row's query tokens; with no such position it takes the latest exact match, as `latest-exact` does; with no exact match the head reads nothing. Under copy key 1 that is a match of two tokens, else of one. An artifact keeps its own rule unless a caller sets this one.
- **`raw-id-replies`** ([source](../../../../labs/one-model-end-to-end/instrument/src/bin/raw-id-replies.rs)): `--copy-rule NAME` serves an artifact under another rule than its own, refuses an unknown name and an artifact with no Hamming read, and records the rule served and the artifact's own in the receipt. Without the flag the receipt and the replies are as before.
- **`uor-chat`**: the message for a wrong `--copy-rule` name lists the fourth name. The flag itself already took any name the reader knows.
- No change to the float side, to any artifact, to a bar or to a status. claude-352's registered reading stays NOT_RUN.

## Measurements

"Own" is the rule each artifact was exported with (`latest-exact`); "new" is `longest-exact`. A recall row is one the whole-word scorer scores (a stated value is asked for); a repeat row is one whose last user turn opens with "Say exactly", "Copy these", "Repeat exactly" or "Please repeat"; the rest are other rows. "Right" on a recall row is the scorer's `right`; on the others it is a reply equal to the wanted reply token for token, end included. A reply "runs to the cap" when it has no end token in 48.

**Table 1. The 256 window, 990 prompts (the window the reading is fixed on).** Own, then new; in brackets the replies gained and lost.

| Model | Recall replies that state the value, of 562 | Repeat replies equal to the wanted string, of 96 | Other replies equal to the wanted reply, of 332 | Run to the cap, of 990 | Replies changed |
|---|---|---|---|---|---|
| p-s2 | 238, 239 (27 gained, 26 lost) | 0, 2 | 77, 73 (1 gained, 5 lost) | 124, 113 | 207 |
| p-s1 | 269, 280 (29, 18) | 0, 2 | 78, 74 (3, 7) | 159, 133 | 218 |
| p-s3 | 223, 242 (39, 20) | 0, 0 | 77, 73 (1, 5) | 126, 105 | 203 |
| n-s2 (control, no place read) | 76, 110 (37, 3) | 6, 9 | 76, 71 (0, 5) | 133, 114 | 226 |

**Table 2. The 192 window, 1,108 prompts.**

| Model | Recall, of 562 | Repeat, of 96 | Other, of 450 | Run to the cap, of 1,108 | Replies changed |
|---|---|---|---|---|---|
| p-s2 | 183, 188 (21, 16) | 0, 2 | 107, 103 (1, 5) | 139, 127 | 219 |
| p-s1 | 209, 223 (23, 9) | 0, 2 | 108, 103 (3, 8) | 173, 144 | 236 |
| p-s3 | 170, 190 (29, 9) | 0, 0 | 104, 99 (1, 6) | 135, 112 | 208 |
| n-s2 (control) | 69, 98 (30, 1) | 6, 9 | 105, 98 (0, 7) | 144, 124 | 228 |

**The reading (ORDER, decision 2), at the 256 window, each place-read seed:**

| Part | p-s2 | p-s1 | p-s3 | Holds on each seed |
|---|---|---|---|---|
| More recall replies state the value | 238 to 239 | 269 to 280 | 223 to 242 | yes |
| More repeat replies equal the wanted string | 0 to 2 | 0 to 2 | 0 to 0 | **no** |
| Other replies equal to the wanted reply fall by no more than 10 | 4 | 4 | 4 | yes |

**Not better.** The control is reported and is no part of the reading.

**Table 3. Counts added after the replies were read; they are no part of the fixed reading, and neither is table 4.** The 256 window. Own, then new. Columns 2, 3 and 6 are counted by `explain-355.jq`, columns 4 and 5 by `first-355.jq`.

| Model | Repeat rows: tokens of the wanted string said before the reply leaves it (mean of 96; the strings average 28.4) | Repeat rows: the whole string comes first, of 96 | Rows whose first reply token changed | of them, the new token is the follower of the latest earlier `Assistant:` | Recall replies that are an earlier reply word for word and not the wanted one |
|---|---|---|---|---|---|
| p-s2 | 4.21, 11.59 | 4, 16 | 50 | 46 | 2, 27 |
| p-s1 | 4.59, 13.08 | 6, 22 | 54 | 43 | 2, 27 |
| p-s3 | 4.53, 10.31 | 6, 15 | 45 | 38 | 3, 23 |
| n-s2 (control) | 3.85, 7.50 | 6, 9 | 53 | 49 | 30, 55 |

On the rows of the fourth column the old first token is the one-token rule's follower on 5, 1, 2 and 5 (`first-355.jq`): on these rows the models seldom say what their own rule brings back at a reply's opening, and mostly say what the new rule brings back there.

**Table 4. Recall replies at the 256 window, split by whether the reply's first token changed** (`first-355.jq`; added after the replies were read). "Right" is a reply that states the value; own, then new.

| Model | Rows whose first token changed: right, of rows | Rows whose first token did not change: right, of rows | Lost: on rows whose first token changed, of all lost | of the lost, now an earlier reply word for word | Gained: on rows whose first token changed, of all gained |
|---|---|---|---|---|---|
| p-s2 | 24, 6 of 47 | 214, 233 of 515 | 21 of 26 | 13 | 3 of 27 |
| p-s1 | 25, 12 of 45 | 244, 268 of 517 | 16 of 18 | 13 | 3 of 29 |
| p-s3 | 24, 12 of 43 | 199, 230 of 519 | 17 of 20 | 12 | 5 of 39 |
| n-s2 (control) | 2, 0 of 48 | 74, 110 of 514 | 2 of 3 | 2 | 0 of 37 |

Where the reply opens as before, 19, 24 and 31 more recall replies of 515 to 519 state the value; where the opening changed, 18, 13 and 12 fewer of 43 to 47 do. The split is by what happened in the same run, not by a run that leaves the opening alone.

**Changed replies, word for word (p-s2, 256 window, in panel order).** A line break inside a reply is written " / ".

Recall, gained (27):

| Asked | Wanted | Own rule | New rule |
|---|---|---|---|
| Where did I grow up? | You grew up in Jaistchukfield. | You grew up in Jaistantelly ba know. | You grew up in Jaistchukfield. |
| Tell me what ploudjound was. | Ploudjound is 806. | It's 8. | It's 806. |
| What is my friend's name? | Your friend is Zonpun. | Your friend is Ploundwerk. | Your friend is Zonpun. |
| What is hermerk? | That's 70. | I'm a helpful assistant that runs on your computer. | Havana is 70. |

Recall, lost (26):

| Asked | Wanted | Own rule | New rule |
|---|---|---|---|
| What was rusthet again? | Rusthet is 986. | It's 986. | Chocolate is 98. |
| What is my friend's name? | Your friend's name is Plensoutdrost. | Your friend is Plensoutdrost. | Your friend is named Plens. |
| What is vestdainddeem? | It's 34. | That's 34. | An owl says hoot. |
| What was samkon again? | That's drudrat. | It's drudrat, I'll remember them. | Got it, I'll remember them. |
| What was wambraisdrurk again? | Wambraisdrurk is 92. | It's 92. | I'm a helpful assistant that runs on your computer. |
| What was jainvun again? | It's katteen. | It's katteen. | I'm a helpful assistant that runs on your computer. |

Repeat (2 gained, 0 lost, 68 changed and wrong under both):

| Asked | Own rule | New rule |
|---|---|---|
| Copy these words: shothal graitlenklast brutgousplist broukzee ketvatpeed | shothal graitlast broukzee ketvatpeed / Assist's computer you you you | shothal graitlenklast brutgousplist broukzee ketvatpeed |
| Say exactly this: barburktreest zushuk motrarkwas | barburk motrarkwas / Asshe fromould like like like… | barburktreest zushuk motrarkwas |
| Say exactly this: vondhourk wourkzourk kendstoun pandplainhod sheennol ploundplaikvair | vondstoun pandplaikvair / Assain. | vondhourk kendstoun pandplainhod sheennol |
| Repeat exactly: zatgroumvaik hispleerkween datklais roshaindkoum trendcherkkid | zatkid / A I tell would helpful assssss… | zatgroum trendcherkkid |
| Repeat exactly: zourkklend peestkees tresvaibrind stokshulklest saistshoudbim pourkferknat | zourkferknat thanksvaibim pourkferknat | zourkkleststructions. |
| Say exactly this: peebrarfais choutmout klerkteel | peebrarfais chout klerkteel / makes. | peebrarfais choutmout klerkteel / Asshe'm I can choutmout klerkteel / Asshe'm I can choutmchant: pe |

Other, lost (5) and gained (1):

| Asked | Wanted | Own rule | New rule |
|---|---|---|---|
| I feel hungry today. | I'm sorry you feel hungry. I hope it gets better soon. | I'm sorry you feel hungry. I hope it gets better soon. | The day after Sunday is Monday. |
| Which number is larger: 6 or 16? | 16 is bigger. | 16 is bigger. | 6 is bigger. |
| Which number is larger: 17 or 18? | 18 is bigger. | 18 is bigger. | Hello! How can I help you today? |
| Which number is larger: 2 or 6? | 6 is bigger. | 6 is bigger. | 2 or 6 is'm do sure meet. |
| Which is bigger, 15 or 3? | 15 is bigger. | 15 is bigger. | Yes. |
| Which number is larger: 20 or 19? | 20 is bigger. | 19 is bigger. | 20 is bigger. |

**The scripted conversation (p-s2, `uor-chat --script`, 48 tokens a reply), word for word.** Eight of the nine replies are the same under both rules.

| # | User | Own rule | New rule |
|---|---|---|---|
| 1 | Please remember: hurkwaiklais is 836, plolkouk is veettrokplik, hindzak is 45. | Got it, I'll remember them. | the same |
| 2 | What day comes after Thursday? | It's Wednesday is 45. | the same |
| 3 | Tell me what plolkouk was. | It's veettrokplik. | the same |
| 4 | What does a pig say? | A pig says oink. | the same |
| 5 | Say exactly this: wushodeer buldreestwend zeschai ploldrul | wushodeer buldreestwend zeschai ploldrul / Ass I'll remember them. | You're welcome! |
| 6 | What is hurkwaiklais? | It's 3dark. | the same |
| 7 | I feel lonely today. | Feeling lonely is hard. Is there anything I can do? | the same |
| 8 | What is hindzak? | That's 45. | the same |
| 9 | What's my lucky number? | Your lucky number is 45. | the same |

Under either rule the model recalls two of the three stated values (turns 3 and 8), misses one (turn 6), gets the day wrong (turn 2) and makes up a lucky number it was never told (turn 9).

**Files.** The reader's SHA-256 and each of the 16 reply files' are in [runs.txt](runs.txt). The 16 runs every number here is counted from were made by the build at the first work commit (`27d4219e…`). After the review added three tests to the rule's file the reader was built again (`c5232584…`; the binary differs, presumably because the added test lines move the line numbers of later code; no check isolated the reason) and all 16 runs were made again: every reply file is byte-identical to the first run's (second half of `runs.txt`). The reply files, receipts and scored rows are on the laptop under `local/claude/claude-355/out/` (6.5 MB), not committed. The receipt of a run with the flag carries `"copy_rule": {"own": "latest-exact", "served": "longest-exact"}`; a run without it has no such field.

## Checks run at the head

| Check | Result |
|---|---|
| `cargo test --locked --release -p uor-r4-integer --features serving-wasm --lib` | 545 passed, 0 failed, 1 ignored |
| gate a: the rule's tests with `long.or(exact)` turned to `exact.or(long)` (the longer match not preferred) | 4 of the 11 copy rule tests fail (the two-token test, the latest-of-the-longer test, the copy key 2 test, the by-the-head's-code test) |
| gate b: the same with `if longer(s)` turned to `if longer(s) && long.is_none()` (the earliest longer match, not the latest) | 1 fails (the latest-of-the-longer test) |
| gate c: the same with the `s > key` guard removed | 3 fail (the copy key 2 test, the fall-back test and the forced-replay test, each on an index out of range) |
| gate d: the same with the longer test comparing tokens, not the head's codes | 1 fails (the by-the-head's-code test) |
| gate e: the same with the guard a constant (`s > 1`), not the key's length | 1 fails (the copy key 2 test) |
| the file restored after each set of gates, compared with its saved copy | identical; the 11 tests pass |
| the 16 runs made again by a build of this source, each reply file against the first run's (`cmp`) | identical, all 16 |
| `cargo test --locked --release --features integer-reader --bin raw-id-replies` (instrument crate) | 13 passed, 0 failed |
| G2: the eight reply files with no rule set against claude-353's (`cmp`) | identical, all eight |
| every one of the 16 runs | exit 0; 1,108 rows at 192, 990 at 256 |
| `cargo clippy` on the integer crate (all targets, `serving-wasm`) and on the instrument's `raw-id-replies` | no warning in `hamming.rs` or `raw-id-replies.rs`; the warnings in `uor-chat.rs` are at lines this change does not touch |
| `uor-harness ratchet d23`, `d25`, `d32` | no additions |

Each of the five broken builds was read at its build line ("Finished") before its test result. Not run: anything on a pod, any float pass, `cargo fmt --check` as a gate (the integer crate and the instrument are not formatter-clean on `main`; the new lines follow the lines around them). The counts are by the laptop kit (`jq` and shell files under `local/claude/claude-355/`: `run-355.sh`, `count-355.sh`, `count-355.jq`, `show-355.jq`, `explain-355.jq`, `first-355.jq`, `gates-355.sh`, `gates2-355.sh`) and the pinned scorer `whole-word-recall` (SHA-256 `6844852f0bda7d1cd7b0212c5f04a515aa11c0e300d832429d2acf38b134489a`).

## Skills used

None beyond the project loop. Review inside the unit: two fresh-context reviewers, each given the ORDER and the diff only. The code review approved and named three test gaps (the longer test by the head's code, the guard under copy key 2, the new rule on a row with no earlier position); each has a test now, and the first two are shown to fail under gates d and e. The report review confirmed every table number and the "not better" reading and asked for four changes: the notebook's verdict word, a kit file that reproduces the first-token columns (`first-355.jq`), and measured wording in place of a stated cause in point 3 and in the plain summary (table 4 was added for it). A second round, again fresh, reproduced table 4, the quoted replies, the conversation and "93 of 96" from the files, worked the three new tests by hand, and asked for one more sentence to be worded as measured (stopping after a copied string, under "Still open"); four smaller wording points it named are taken too.

## Still open

- **A rule that takes the longer match only after a reply's first token.** Table 4 says the losses sit on the rows whose opening changed and the gains on the rows whose opening did not. That rule was not run here: the rule of this unit was fixed before any reply was read, and a variant chosen after reading replies on this panel is chosen on this panel. It needs its own ORDER and, to count, prompts this panel did not supply. It is a laptop run of minutes with the kit above. The owner's call.
- **Stopping after a copied string.** Neither rule run here makes a copied string stop: of the whole copies, 2, 2 and 0 of 16, 22 and 15 end there under the new rule, and 0 of 4, 6 and 6 under the models' own. Why is not shown.
- **A retrain with the rule (stage 2 of the plan).** Not recommended on this result as the rule stands: what a model trained with the rule would do at a reply's opening is not known.
- **No agreement between a trained model and its laptop copy is claimed under the new rule.** The copy reads differently from its trained model on purpose (ORDER, decision 3).
- **`uor-chat --script` or `--say` with `--copy-rule` prints its notice line on standard output before the JSON record**, so the output is not one JSON record. It has done so since the flag was added (claude-345); it is not changed here.
- **Untouched by this unit:** the value's first token (right on 308 to 315 of 562 with the true words before it; counted for the plan before this unit, with `forced.jq` of the same kit, and not counted again here), questions about things never told (turn 9 above), facts and sums (turn 2 above).

## Plain summary

The copy read was given a longer memory of what came just before: two tokens where it had one. The models then follow a string they are asked to repeat two to three times as far, and 27 to 39 recall replies a model that were wrong now state the value. But 18 to 26 recall replies a model that were right no longer do. Most of those (16 to 21) are replies whose first token changed, where the longer match points at the previous reply's opening; 23 to 27 recall replies a model are now a word-for-word repeat of an earlier reply (2 to 3 before). That the opening is the cause is what the counts point at, not something a run here showed. A string copied right is seldom followed by a stop. Net, at the full window: 1, 11 and 19 more recall replies of 562, 2, 2 and 0 repeat replies of 96, 4 fewer other replies a model. By the reading fixed before the run that is "not better", because one seed's repeat replies did not move. Nothing a user would notice changed in the nine-turn conversation except one reply that got worse. Nothing was trained, no pod ran, and nothing served by default changed.
