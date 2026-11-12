# REPORT: one-model-end-to-end, claude-356

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [ORDER.md](ORDER.md). PR #3946.

A laptop-only test, approved by the owner on 12 Nov ("Yes, test it"): the three stage 3 place-read models of [claude-352](../claude-352/REPORT.md) (p-s2, p-s1, p-s3) and their control (n-s2), served with a copy rule that takes the longest match at every row except the one that chooses a reply's first token, where it keeps the one-token match the models were trained with. Read on a fresh panel (generator seed 15), none of whose prompts equals a prompt of the old panel. No pod, no training, no float pass; no artifact is changed and nothing served by default is changed.

**By the reading fixed in the ORDER before any reply to a fresh prompt existed: better on recall.** On the fresh panel at the 256 window, the new rule against each model's own: 23, 26 and 33 more recall replies of 560 state the value (245 to 268, 278 to 304, 237 to 270), with 1, 6 and 2 lost; the repeat-a-string replies equal to the wanted string go from 0 to 2 of 102 on each seed; the other replies equal to the wanted reply fall by 1, 1 and 2 of 322.

**What that does not say.** The models still state the value on about half the recall rows (48%, 54%, 48%). The control gains as much (78 to 110), which points to the copy rule and not the place read as the source of the gain. Repeat-a-string replies stay near zero. No model was trained with the rule.

| Gate | Reading | Status |
|---|---|---|
| G1 | Each form of the rule, broken on its own (fourteen breaks, gates a to n below, the session's passing of the reply mark among them), fails the rule's tests; the integer crate's stack tests pass at the head. | **met** |
| G2 | On the old prompts, all sixteen reply files with no rule set and under `longest-exact` are byte-identical to claude-355's. | **met** |
| G3 | The programs that made the fresh panel remake the old one: seed 14's tokens with the old SHA-256, and its mask, its oracle twin, its places kit and its 256 and 192 prompts byte for byte. | **met** |
| G4 | The counts for the four artifacts, both windows, both panels and three rules, and the changed replies word for word, are under Measurements; how decision 1 reads is said above. | **met** |

## What did not work, first

1. **Recall is still right on about half the rows.** Under the new rule 268, 304 and 270 of 560 fresh recall replies state the value. The rule adds 23 to 33 replies a model; it does not make the models recall.
2. **The control gains as much as the place-read models**: 78 to 110 of 560 (32 gained, none lost). The place read's margin over its control does not grow: it shrinks on p-s2 and p-s1 (167 to 158 replies, 200 to 194) and moves by one on p-s3 (159 to 160).
3. **Repeat-a-string replies stay near zero**: 2 of 102 on each seed (0 before). As decision 2 said beforehand, this unit tests nothing new on them: the new rule and `longest-exact` give the same two replies. A copied string is still followed and seldom stopped.
4. **Four other replies are lost on every place-read seed, the same four rows** (fresh rows 135, 159, 456, 767: "which is bigger" questions answered with a number). Their replies open with a bare space token, and the first word is the second token. The rule spares only the row that chooses the first token, so at the second token the two-token match (`:` and the space) brings back the first word of an earlier reply that opened the same way: "19 is bigger." becomes "Yes." after an earlier "Yes." (table 5). On every row whose second token changed after a bare space (12, 12 and 10 a model) the new second token is that follower, and the old one was the follower of the latest earlier lone space (`pair-356.jq`). 43 to 47 of the 984 replies open with a bare space; on them the right replies fall (18 to 17, 18 to 17, 19 to 17) while on the other rows they rise (315 to 340, 347 to 375, 307 to 342).
5. **The first reply token still changed on one row of 984 on p-s3 and on the control** (none on p-s2 and p-s1). The rule fixes which position the copy heads read at the opening row; it does not fix what the row is computed from, which has been through prompt rows read under the longest rule. Why the token moved on these rows was not traced further.
6. **The scripted conversation is still wrong on four of nine turns under the new rule** (below): asked "What is druvalk?" (told 73) it says "That's 603."; "I'm tired today." and "What is two plus three?" get "I'm a helpful assistant that runs on your computer."; "What is my name?" (never told) gets "You're welcome!". Turn 2 states the right value and runs on into the next one; turn 4 says the string and then a stray "A". The models' own rule garbles turns 2 to 6. Against `longest-exact` it differs on turns 4, 6 and 8: turn 4 now holds the string, and turns 6 and 8 are wrong under both.
7. **Replies that run to the 48-token cap fall but remain**: 107 to 101, 158 to 143 and 127 to 106 of 984.
8. **The pinned `train-in-space` of claude-352 could not make the prompts** (it refuses `--panel-prompts` with `--context`; that came later than the pin). A build of this branch's source made the kit and the prompts; gate G3 is what shows it is the same program for this purpose.
9. **The first run of the sabotage gates was stopped at its baseline**: the new session test itself failed, because its replay half asked a snapped artifact (one whose transport picks the nearest icosian, a choice the trace does not record) for a decision trace (the per-step record of what each discrete read chose, which the forced replay reads back), which the reader refuses. That half was rewritten on unsnapped artifacts. The gates reported here are of the rewritten test, run to the end.
10. **Not shown at all:** how a model trained with the rule would reply. Under either new rule the laptop copy reads differently from its trained model on purpose; no float-against-integer agreement is claimed.

## What changed

- **The integer reader** ([`hamming.rs`](../../../../../crates/uor-r4-integer/src/stack/hamming.rs), [`session.rs`](../../../../../crates/uor-r4-integer/src/stack/session.rs)): a fifth copy rule, `CopyRule::LongestAfterOpening` (`longest-after-opening`). At the row that chooses a reply's first token it takes `latest-exact`'s position; at every other row `longest-exact`'s. The session names that row by its reply mark (the row before the marked reply start); with no mark every row reads as `longest-exact`. The served read, the trace and the forced replay all take the same flag. An artifact keeps its own rule unless a caller sets this one.
- **What the rule does and does not spare:** only the reply being generated. Earlier replies inside the prompt are prompt rows and are read under the longest rule. At a readout row where the place read is open the copy rule does not apply, as before. `uor-chat`'s memory-line wording follows the artifact's own rule, not the one set by the flag.
- **`raw-id-replies`** ([source](../../../../labs/one-model-end-to-end/instrument/src/bin/raw-id-replies.rs)): `--copy-rule` takes the new name and records it in the receipt. With `--mark-reply 0` no row is marked, so the rule is `longest-exact` there; a test pins that.
- **`uor-chat`**: the message for a wrong `--copy-rule` name lists the fifth name.
- **[chat-script.json](chat-script.json)**: the nine user turns fixed in the ORDER.
- No change to the float side, to any artifact, to a bar or to a status. claude-352's registered reading stays NOT_RUN.

## Measurements

"Own" is the rule each artifact was exported with (`latest-exact`); "longest" is `longest-exact` (claude-355); "new" is `longest-after-opening`. A recall row is one the whole-word scorer scores (a stated value is asked for); a repeat row is one whose last user turn opens with "Say exactly", "Copy these", "Repeat exactly" or "Please repeat"; the rest are other rows. "Right" on a recall row is the scorer's `right`; on the others it is a reply equal to the wanted reply token for token, end included. A reply "runs to the cap" when it has no end token in 48. Every cell is own, longest, new; in brackets the replies gained and lost by the new rule against own.

**The fresh panel.** `m-world-recall-corpus conversations=1000 seed=15` (the old panel is seed 14, TRAIN seed 13): 560 value rows. Prompts by `--panel-prompts`, 984 at the 256 window and 1,106 at the 192 window. Prompts equal, token for token, to a prompt of the old panel at either window: 0 and 0, so no row is left out. The same filter finds all 990 and all 1,108 old prompts when the old panel is given to it as the new one.

**Table 1. Fresh panel, 256 window, 984 prompts (the window and panel the reading is fixed on).**

| Model | Recall replies that state the value, of 560 | Repeat replies equal to the wanted string, of 102 | Other replies equal to the wanted reply, of 322 | Run to the cap, of 984 | Replies changed (longest, new) | First reply token changed (longest, new) |
|---|---|---|---|---|---|---|
| p-s2 | 245, 248, **268** (24 gained, 1 lost) | 0, 2, 2 | 88, 87, 87 (3, 4) | 107, 100, 101 | 211, 160 | 54, 0 |
| p-s1 | 278, 279, **304** (32, 6) | 0, 2, 2 | 87, 87, 86 (3, 4) | 158, 139, 143 | 237, 184 | 57, 0 |
| p-s3 | 237, 244, **270** (35, 2) | 0, 2, 2 | 89, 86, 87 (2, 4) | 127, 107, 106 | 220, 175 | 53, 1 |
| n-s2 (control, no place read) | 78, 107, 110 (32, 0) | 3, 4, 4 | 87, 86, 88 (2, 1) | 116, 110, 111 | 237, 198 | 51, 1 |

**The reading (ORDER, decision 1), the new rule against the model's own, each place-read seed:**

| Part | p-s2 | p-s1 | p-s3 | Holds on each seed |
|---|---|---|---|---|
| (a) At least 10 more recall replies state the value | 23 more | 26 more | 33 more | yes |
| (b) Repeat replies equal to the wanted string are not fewer | 0 to 2 | 0 to 2 | 0 to 2 | yes |
| (c) Other replies equal to the wanted reply fall by no more than 10 | 1 | 1 | 2 | yes |

**Better on recall.** The control, the old panel, the 192 window and `longest-exact` are reported and are no part of the reading.

**Table 2. Fresh panel, 192 window, 1,106 prompts.**

| Model | Recall, of 560 | Repeat, of 102 | Other, of 444 | Run to the cap, of 1,106 | Replies changed (longest, new) | First reply token changed (longest, new) |
|---|---|---|---|---|---|---|
| p-s2 | 195, 207, 218 (24, 1) | 0, 2, 2 | 118, 115, 115 (3, 6) | 128, 119, 118 | 228, 179 | 56, 0 |
| p-s1 | 214, 225, 239 (30, 5) | 0, 2, 2 | 115, 115, 113 (3, 5) | 171, 151, 156 | 254, 201 | 61, 0 |
| p-s3 | 191, 203, 217 (27, 1) | 0, 2, 2 | 117, 115, 115 (2, 4) | 134, 116, 117 | 220, 179 | 54, 1 |
| n-s2 (control) | 70, 92, 93 (23, 0) | 3, 4, 4 | 116, 113, 116 (2, 2) | 119, 116, 121 | 236, 194 | 54, 1 |

**Table 3. Old panel (seed 14, the panel the rule was chosen on), 256 window, 990 prompts.** Description only.

| Model | Recall, of 562 | Repeat, of 96 | Other, of 332 | Run to the cap, of 990 | Replies changed (longest, new) | First reply token changed (longest, new) |
|---|---|---|---|---|---|---|
| p-s2 | 238, 239, 259 (26, 5) | 0, 2, 2 | 77, 73, 75 (1, 3) | 124, 113, 115 | 207, 163 | 50, 0 |
| p-s1 | 269, 280, 298 (31, 2) | 0, 2, 2 | 78, 74, 76 (1, 3) | 159, 133, 135 | 218, 172 | 54, 0 |
| p-s3 | 223, 242, 256 (36, 3) | 0, 0, 0 | 77, 73, 75 (1, 3) | 126, 105, 109 | 203, 165 | 45, 1 |
| n-s2 (control) | 76, 110, 114 (39, 1) | 6, 9, 9 | 76, 71, 75 (1, 2) | 133, 114, 120 | 226, 178 | 53, 0 |

**Table 4. Old panel, 192 window, 1,108 prompts.** Description only.

| Model | Recall, of 562 | Repeat, of 96 | Other, of 450 | Run to the cap, of 1,108 | Replies changed (longest, new) | First reply token changed (longest, new) |
|---|---|---|---|---|---|---|
| p-s2 | 183, 188, 201 (22, 4) | 0, 2, 2 | 107, 103, 105 (1, 3) | 139, 127, 131 | 219, 182 | 51, 0 |
| p-s1 | 209, 223, 233 (26, 2) | 0, 2, 2 | 108, 103, 106 (1, 3) | 173, 144, 146 | 236, 189 | 56, 0 |
| p-s3 | 170, 190, 196 (29, 3) | 0, 0, 0 | 104, 99, 102 (1, 3) | 135, 112, 118 | 208, 171 | 46, 0 |
| n-s2 (control) | 69, 98, 98 (30, 1) | 6, 9, 9 | 105, 98, 104 (1, 2) | 144, 124, 134 | 228, 171 | 58, 0 |

The old-panel own and longest columns are claude-355's numbers again, from byte-identical reply files (G2). On both panels and both windows the recall replies lost under `longest-exact` sit on the rows whose first reply token changed (fresh 256: 20 of 21, 27 of 33, 24 of 26); under the new rule that token does not change and those losses are gone.

**Table 5. Counts added after the replies were read; no part of the fixed reading.** Fresh panel, 256 window, own against new, all three kinds of row together (`second-356.jq`).

| Model | Replies that open with a bare space token, of 984 | Right on those rows (own, new; lost, gained) | Right on the other rows (own, new; lost, gained) | Rows whose second reply token changed, the first being the same | of them, after a bare space | Replies lost in all, and of them with the second token changed |
|---|---|---|---|---|---|---|
| p-s2 | 43 | 18, 17 (4, 3) | 315, 340 (1, 26) | 17 | 12 | 5, 4 |
| p-s1 | 45 | 18, 17 (4, 3) | 347, 375 (6, 34) | 17 | 12 | 10, 4 |
| p-s3 | 47 | 19, 17 (4, 2) | 307, 342 (2, 37) | 15 | 10 | 6, 4 |
| n-s2 (control) | 46 | 19, 20 (1, 2) | 149, 182 (0, 33) | 10 | 6 | 1, 1 |

A reply whose first word has no token of its own with the space joined (a number, "Yes") opens with the space as a token by itself. On row 456 the prompt holds an earlier reply "Yes." and the question "Which is bigger, 1 or 19?". Under the models' own rule the reply is the tokens ` `, `1`, `9`, ` is`, ` bigger`, `.`; under the new rule it is ` `, `Yes`, `.`: the first token is the same and the second is the earlier reply's first word.

### The changed replies, word for word (fresh panel, 256 window, own then new)

Printed by claude-355's `show-355.jq` from the reply files named in [runs.txt](runs.txt). OWN lines are cut at 150 characters.

**Every recall reply lost under the new rule (1, 6 and 2).**

| Model, row | Asked | Wanted | Own | New |
|---|---|---|---|---|
| p-s2, 326 | What is takmomstou? | Takmomstou is 555. | Takmomstoukle is 555. | Takmomstouksar is 55. |
| p-s1, 5 | What is sustvaim? | It's demplost. | Sustvaim is demplost. | Sustvaim is 1874. |
| p-s1, 18 | Do you remember what mitkloukfirk was? | Mitkloukfirk is laimjechouk. | That's laimjechouk. | That's laimjechoukfirk. |
| p-s1, 83 | Who is my friend? | Your friend is Gostshourkpaik. | Your friend's name is Gostshourkpaik. | Your friend's name is Gostshourk |
| p-s1, 90 | Tell me what droukde was. | That's zundranveend. | It's zundranveend. | It's zundructions. |
| p-s1, 111 | Tell me what grusgrair was. | Grusgrair is jistzeetchun. | It's jistzeetchun. | It's jist it it it it it (to the cap) |
| p-s1, 326 | What is takmomstou? | Takmomstou is 555. | Takmomstou is troustou is 555. | Takmomstou is troustou is 55. |
| p-s3, 18 | Do you remember what mitkloukfirk was? | Mitkloukfirk is laimjechouk. | That's laimjechouk Bird saw dog e | That's laimjechoukfirk bringrous. |
| p-s3, 28 | Tell me what kakheem was. | Kakheem is graisshundpes. | It's graisshundpes. | It's graissh,, I Iant: It's's's's (to the cap) |

**The first three recall replies gained on each seed (of 24, 32 and 35).**

| Model, row | Asked | Wanted | Own | New |
|---|---|---|---|---|
| p-s2, 78 | Tell me what rouhan was. | It's naschest. | It's naschestructions. | It's naschest. |
| p-s2, 106 | What is my friend's name? | Your friend's name is Tourkwum. | Your friend is Pinund know. | Your friend is Tourkwum. |
| p-s2, 183 | What is my friend's name? | Your friend's name is Shendweendwail. | Your friend is T meet you didn't tell me. | Your friend is Shendweendwail. |
| p-s1, 48 | Where is my home? | Your home is in Oslo. | You live in in Os for telling me. | You live in in Oslo. |
| p-s1, 78 | Tell me what rouhan was. | It's naschest. | It's naschestructions. | It's naschest. |
| p-s1, 98 | Do you remember what trondseem was? | Trondseem is chaldrost. | That's chaldrostructions. | That's chaldrost. |
| p-s3, 5 | What is sustvaim? | It's demplost. | Sustvaim is,,,,,,,, (to the cap) | Sustvaim is is demplost. |
| p-s3, 30 | Do you remember what stumhilzairk was? | Stumhilzairk is 74. | That's 7. | That's 74. |
| p-s3, 64 | What is dindrorheed? | That's 8983. | That's 898. | That's 8983. |

**The repeat replies that became right (2 on each seed; the same under `longest-exact`).**

| Model, row | Asked | Own | New |
|---|---|---|---|
| p-s2, 348 | Repeat exactly: nousbemgoust steschekchut grastbound destgra plerkgritked dageerhait | nousbound destgritked dageerhait | nousbemgoust steschekchut grastbound destgra plerkgritked dageerhait |
| p-s2, 711 | Repeat exactly: plirchoul keestjastzou mistgrod houndbriklim | plirchou mistgrod houndbriklim, a new line, "Assia s s s s" (to the cap) | plirchoul keestjastzou mistgrod houndbriklim |
| p-s1, 131 | Say exactly this: brusreed jeedbourmound stukbraist seelmeest | brusreed jeedbraistundundmeest, a new line, "Assht toappy,ven well runs follow follow" (to the cap) | brusreed jeedbourmound stukbraist seelmeest |
| p-s1, 826 | Copy these words: jetweerstik drindtret fudzensa bertrukvourk | jetweerstik drindtret fudzensa bertruk | jetweerstik drindtret fudzensa bertrukvourk |
| p-s3, 131 | Say exactly this: brusreed jeedbourmound stukbraist seelmeest | brusreed jeedbraistantantantantantind am lunch co co you you (to the cap) | brusreed jeedbourmound stukbraist seelmeest |
| p-s3, 826 | Copy these words: jetweerstik drindtret fudzensa bertrukvourk | jetweerstik drindtret fudzensa bertrukk Now well well well (to the cap) | jetweerstik drindtret fudzensa bertrukvourk |

**The other replies lost (4 on each seed, the same four rows and the same new replies on all three) and gained (3, 3 and 2).**

| Row | Asked | Wanted | Own | New |
|---|---|---|---|---|
| 135, lost on all three | What is the bigger number, 12 or 15? | 15 is bigger. | 15 is bigger. | 8 is bigger. |
| 159, lost on all three | Which number is larger: 7 or 14? | 14 is bigger. | 14 is bigger. | 7 is bigger. |
| 456, lost on all three | Which is bigger, 1 or 19? | 19 is bigger. | 19 is bigger. | Yes. |
| 767, lost on all three | Which is bigger, 18 or 19? | 19 is bigger. | 19 is bigger. | Yes. |
| 145, gained on all three | Which number is larger: 7 or 6? | 7 is bigger. | 6 is bigger. | 7 is bigger. |
| 777, gained on all three | Which number is larger: 16 or 2? | 16 is bigger. | "2 is bigger." (p-s2), "2, 2, 2, 2, 2, 2, 2." (p-s1), "2?" (p-s3) | 16 is bigger. |
| 360, gained on p-s2 and p-s1 | What is 4 plus 7? | 4 plus 7 is 11. | "4 plus 4 is 11." (p-s2), "4 plus 4 is 12." (p-s1) | 4 plus 7 is 11. |

**The rows whose first reply token changed under the new rule (`firstrow-356.jq`).** Fresh p-s3, row 793, "What was trustedhar again?", wanted "Trustedhar is 89.": "Noted." becomes "That's 89.". Fresh control, row 227, "What was kainjisthem again?", wanted "That's filtisdrast.": "Your name is grurks!" becomes "That's great news about theirven follow follow" (to the cap). Old p-s3, row 893, "What was belplon again?", wanted "It's voumet.": "Rome is voukgor." becomes "It's voukgor.".

### The scripted conversation (p-s2, nine turns, 48 tokens a reply, the memory helper off)

`uor-chat --stack art/p-s2-try-2-c8256.uorlut --script chat-script.json --max-tokens 48`, then with `--copy-rule longest-exact` and `--copy-rule longest-after-opening`. The same command with claude-355's script and no rule gives claude-355's reply ids again: the nine reply-id arrays of the two records are identical (`chatids-356.jq`, SHA-256 `4838541b…4e2bee` for both; the line is in [runs.txt](runs.txt)).

| Turn | User | Own rule | `longest-exact` | New rule |
|---|---|---|---|---|
| 1 | Keep these in mind: druvalk is 73, mibtorn is felsharp, quonzet is 18. | Got it. | Got it. | Got it. |
| 2 | What is mibtorn? | Mibtorn is felsharp, quonzet is a a a (stopped on a cycle) | Mibtorn is felsharp, quonzet is 18. | Mibtorn is felsharp, quonzet is 18. |
| 3 | What does a cow say? | Mibtorn is roof Iededed (cycle) | A cow says moo. | A cow says moo. |
| 4 | Say exactly this: trelvon dask pimmerholt zane | Mibt zane, a new line, A Ied, a new line, helpful't't't (cycle) | I'm a helpful assistant. | trelvon dask pimmerholt zane, a new line, A |
| 5 | What is druvalk? | Mibt zane, a new line, A Ied, a new line, helpful't't't (cycle) | That's 603. | That's 603. |
| 6 | I'm tired today. | Mibt zane I. | That's 603. | I'm a helpful assistant that runs on your computer. |
| 7 | Tell me what quonzet was. | It's 18. | It's 18. | It's 18. |
| 8 | What is two plus three? | I'm a helpful assistant that runs on your computer. | It's 18. | I'm a helpful assistant that runs on your computer. |
| 9 | What is my name? | You're welcome! | You're welcome! | You're welcome! |

Under the new rule: turns 1, 3 and 7 are right; turn 2 holds the right value and then the next pair; turn 4 holds the string and a stray "A"; turns 5, 6, 8 and 9 are wrong. It is one conversation and no part of the reading.

## Checks run at the head

Head of the code: `c231cccddee6b35a51662f9cf15b771d349973c5`. Everything ran on the laptop.

| Check | Result |
|---|---|
| `cargo test --locked --release -p uor-r4-integer --features serving-wasm --lib --no-fail-fast` | 549 passed, 0 failed, 1 ignored |
| G1 baseline: the 15 tests the gates run (the copy rule tests and the session test), nothing broken | 15 pass |
| gate a: the opening is not spared (`at` gives the longest rule at the opening) | 3 fail: the rule's unit test, the forced-replay test, the session test |
| gate b: the latest match at every row | 3 fail: the same three |
| gate c: the served choice does not read the opening | 3 fail: the same three |
| gate d: the forced choice does not read the opening | 2 fail: the forced-replay test, the session test |
| gate e: `mean_at` drops the opening | 3 fail: the same three as gate a |
| gates f, g, h, i, j: the stream read, the readout read, the trace, the stream replay and the readout replay each drop the opening | 1 fails under each: the session test |
| gates k, l: the session names no opening; the session names the row after | 1 fails under each: the session test |
| gates m, n: the plain step and the timed step each drop the mark | 1 fails under each: the session test |
| every gate | exactly 1 line changed; all 15 builds read at their "Finished" line, no compile error |
| the two source files after the gates, against their saved copies (`cmp`) | identical; the 15 tests pass |
| `cargo test --locked --release --features integer-reader --bin raw-id-replies` (instrument crate) | 13 passed, 0 failed |
| `raw-id-replies` and `uor-chat` built again from this head, SHA-256 against the programs that made every reply here | identical (`88de2162…10fd46`, `b76b1f73…5190d0`) |
| G2: the 16 old-panel reply files under the own rule and `longest-exact` against claude-355's (`cmp`) | identical, all 16 |
| G3: the old panel made again by the fresh panel's programs, six files (`cmp`) | identical, all six |
| fresh prompts equal to an old-panel prompt (`equal-356.jq`) | 0 of 984 and 0 of 1,106; the filter finds all 990 and 1,108 old prompts when given the old panel |
| every one of the 48 runs | exit 0; 984 and 1,106 rows fresh, 990 and 1,108 old |
| `cargo check --locked -p uor-r4-integer` (default build) and `-p uor-r4-training` | both finish |
| `cargo clippy` on the integer crate (all targets, `serving-wasm`) and on the instrument | no warning in `hamming.rs`, `session.rs` or `raw-id-replies.rs`; 5 in `uor-chat.rs` and 12 in `tests.rs`, none on a line this change added |
| `uor-harness ratchet d23`, `d25`, `d32` | OK, no site added (4,451; 0; 31, each its baseline) |

Not shown able to fail: the instrument test's last new assertion (with no reply marked, the new rule's reply file equals the longest rule's) was not run under a gate, and the reviewer notes its small fixture may not tell the two apart. What it states is pinned in the integer crate by the session test (gates b, k and l). Not run: any float pass, any pod, `cargo fmt` (the integer crate and the instrument are not formatter-clean on main).

The laptop kit, not committed (`local/claude/claude-356/`): `panel-356.sh`, `run-356.sh`, `count-356.sh` and `count-356.jq`, `equal-356.jq`, `second-356.jq`, `firstrow-356.jq`, `pair-356.jq`, `chatids-356.jq`, `gates-356.sh`, `head-356.sh`, the four pinned programs, both panels, every reply file and the counts.

## Skills used

None beyond the project loop. Review inside the unit: three fresh-context reviewers, none given the plan or the reasoning.

- **Code**, given the ORDER and the diff: approved. It followed the opening through every place a copy head chooses (the served read, the trace, the forced replay, both reads, both steps), found no off-by-one in the session's test for the opening row and no way a stale mark is read as an opening again, and no change to what an artifact serves by default. Its notes: the new rule's own arm in the two choosing functions cannot be reached (the rule is resolved first; the arm is there because the match must name every rule); the instrument assertion named above may not be able to fail; `reset()` does not clear the reply mark (so before this change; listed under Still open).
- **Numbers**, given the report and the raw files and told to write its own counting program before reading mine: every cell of table 1 and of the reading confirmed, the changed rows the same, and fifteen quoted replies decoded from their ids match the text here. By its own numbers decision 1 reads "better on recall".
- **Text**, given the ORDER, the report, the run record and the count files: approved; every cell of tables 2, 3 and 4 and of the conversation matches. Its six wording notes are all taken: the margin on p-s3 moves up by one and is now said so; the source of the gain is worded as what the control points to; the notebook row counts the conversation's turns as three right, two partly, four wrong; the plain summary says the repeated string is followed by a stray "A"; the claude-355 check has its result and hash; "snapped artifact", "decision trace" and "the place read" are explained.

One round each; no finding asked for a code change, so no second round was run.

## Still open

- **A retrain with the rule.** Every model here was trained with the one-token rule and only read with the new one. Whether a model trained with it recalls better than these is not known and is the question this result raises. It needs the rule in the float trainer, a pod, and the owner's word (ORDER, decision 6).
- **The opening is two tokens long when a reply opens with a bare space** (point 4, table 5). A rule that also spares the row after a bare-space first token was not run: it was seen in these replies, so it would need its own ORDER and a panel not read yet. It concerns 43 to 47 replies of 984 and four lost replies a model.
- **Stopping after a copied string**, the value's first token, questions about things never told, sums and facts: untouched, as the ORDER said. Turns 5, 8 and 9 of the conversation are of these kinds.
- **Serving the rule by default.** Not done: an artifact keeps its own rule unless a caller sets this one. Making it the served rule of these artifacts, or of an export, is a separate decision.
- **`uor-chat --script` with `--copy-rule` still prints its notice line on standard output before the JSON record** (noted in claude-355; not changed here).
- **`reset()` on a session does not clear the reply mark** (so before this change). Every caller here clears the mark before a new prompt, so nothing reads wrongly today; under the new rule a caller that resets and does not clear would get one row read as an opening. Not changed here.
- No milestone moves. claude-352's registered reading stays NOT_RUN.

## Plain summary

Three of the four models have the place read: when a question names something said earlier, it finds where in the conversation that was said and offers the model the words that followed. The fourth is the same model without it, the control. All four copy a remembered value by finding where the current word last appeared and reading what came after it. claude-355 tried matching two words back instead of one: more values came out right, but about as many were lost, and the lost ones sat at the very first word of a reply, where the two words before it are always "Assistant" and ":" and the match is simply the previous reply.

This unit matched two words everywhere except at that first word. It was tested on a new set of 984 questions the models had not been read on. On each of the three models 23, 26 and 33 more answers of 560 state the remembered value and 1, 6 and 2 are lost; nothing else gets worse by more than 2 replies. By the rule written down before the run, that is "better on recall".

It is a modest gain. The models still state the value only about half the time (48%, 54%, 48%), the control model without the place read gains just as much, and a model asked to repeat a string still almost never stops in the right place. In a nine-turn conversation the model now answers "A cow says moo." and repeats the string it is given (then adds a stray "A" instead of stopping), and still says "That's 603." for a value that was 73. Nothing was trained; the models were only read differently on the laptop.
