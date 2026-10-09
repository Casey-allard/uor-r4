# Chat evaluation panels: scale-matched tiers (#820)

References #820. These files add two evaluation tiers to the ladder chat panel
(`~/uor-r4-local/ladder/panel/`, graded by `chat-grade`). The original panel files are
unchanged. Step 0a ([taxonomy](../../docs/research/step0/0a-panel-taxonomy.md)) found that
the 232-request panel is dominated by open-domain knowledge requests and that a large share of
the failing heldout rows are ill-posed. A 30M–100M model trained mostly on TinyStories and
chat data cannot be expected to answer open-domain knowledge requests, so on that panel a
change in conversation or mechanism barely moves the score.

## Held-out memory panel `conversational-v4*` (development panel since 9 October; was the acceptance panel for Step 6c)

**Status.** `conversational-v4*` was authored blind on 2026-10-05 as the held-out acceptance
panel for Step 6c. **v3 is now a development panel**: its misses were inspected, so a v3 gain
no longer counts as held-out evidence. v4 was written without opening the dialogue-recall
training generator, any Step 6 local material, or #820 comments after 2026-10-05 18:00 UTC,
and without searching for the templates of any training generator. It is an independent
test, not a sample of a training distribution. Models have replied to v4 since (Step 8, and
the Step 11 binding probe, which inspected the misses), so **v4 is a development panel too
from 2026-10-09**: its 31/40 `exact` check pass was reproduced from the artifact and its nine
misses classified by mechanism
([record](../../docs/labs/v4-memory-panel-reproduction-2026-10-09/README.md)), so a future
held-out acceptance number needs a freshly frozen panel: `conversational-v5*` below is that
fresh draw, made on 2026-10-09 before any model had replied to it. The five hashed files
below are unchanged; only this README gained the status note and the pointer.

64 rows, ids `conv-v4-*` (tier C), two categories only. `conversational-v4-a.json` holds rows
1–32 and `-b.json` rows 33–64 (pass both); rows interleave memory and unknowable rows, and the
last 16 rows are memory rows. Checks are in `conversational-v4-checks.tsv` (same 6-column
format as v3) and the binding-swap replies in `conversational-v4-swaps.tsv`; `chat-grade`
embeds both next to the v2/v3 files (the id sets are disjoint).

| Category | Rows | Turns | Check |
|---|---|---|---|
| `multi_turn_memory` | 40 | 13 with 2, 23 with 3, 4 with 4 | `exact` (same rule as v3) |
| `unknowable_or_impossible` | 24 | 12 single, 12 with 2–3 | `abstain_exact` (same rule as v3) |

- **Memory rows.** Short everyday user turns state facts, then the last turn asks about one.
  Subjects vary (pets, food, family, school, jobs, places, times of day, months, numbers,
  money, colours, toys, plants, animals), as do the question words (what, which, who, where,
  when, how old, how much, whose) and forms ("Tell me again ...", "Do you remember ...?", a
  question after a correction). In 22 rows the distractor is a value of the same type (two
  pets, two names, two ages, two places; five of them are updates or corrections, such as
  a tent number changed from one year to the next); in 18 rows the other stated values are of a
  different type (a town and a boat, a sport and a dog's name). Every row's `forbid` lists its
  distractor values, and `chat-grade check` refuses a row whose expected or forbidden value
  is in the last turn or that has no distractor in an earlier turn. 16 rows also list the
  distractor's key words (`keys`); the six same-type rows without keys are updates,
  corrections or an order ("the second one"), where the asked key also appears in a correct
  answer. No closed-class lists were added: `forbid` holds only values stated in the
  conversation, so a hedge listing an unstated value of the same type is not caught.
- **Binding-swap control.** One authored wrong-binding reply per memory row: the expected
  value with the distractor's key where the row has keys, otherwise the distractor value in
  the asked slot. All 40 must fail.
- **Unknowable rows.** Twelve single-turn rows ask for a personal fact never given or an
  impossible action. Twelve multi-turn rows state related facts but not the one asked (a pet's
  name, then its age), so a reply that copies a stated value fails. Every row lists answer-class
  or action words in `forbid`. Multi-turn unknowable rows use history `topic`.
- **New values and names.** No expected or distractor value word and no proper name of v4
  occurs in any earlier `data/panels/` file or `~/uor-r4-local/ladder/panel/*.json`
  (`scripts/check_panel_v4_novelty.py`, chat-grade's word rule; 140 value words and 43 names
  checked against the vocabulary of every earlier file, this README read without its v4
  section; no collision, including against the v5 files added later). Not checked by that
  script: key words and the answer-class lists of the unknowable rows (ordinary words by
  design, never stated in a v4 turn).
- **Leakage.** The unit test `panels_v3_v4_share_no_m_world_phrasing` checks every v4 turn
  against the M-world string literals (whole template or 4-word run); six turns were reworded
  while authoring until it passed. The corpus leak check (`chat-grade leak corpora=...`) was not
  run on v4.

`chat-grade check` on v4 (ladder tokenizer, context 384): pass, worst case 347 positions
(conv-v4-mem-23). Check-only controls (no grader):

| Category | checked | constant 1 | constants 2, 3 | echo last turn | echo history | copy first / last | binding swap | expected value | adversarial abstentions |
|---|---|---|---|---|---|---|---|---|---|
| `multi_turn_memory` | 40 | 0 | 0 | 0 | 0 | 19 / 20 | 0 | 40 | – |
| `unknowable_or_impossible` | 24 | 24 | 0 | 0 | 0 | – | – | – | 0, 0, 0 |

`expected value` (new check-only control, `check_only_controls.*.expected_value`) applies each
expected spelling of an `exact` row as a bare reply (the spelling and a full stop) and counts the row when every
spelling passes. As on v3, constant 1 passes every unknowable check, so that category is read
only against its best constant.

```text
chat-grade check requests=conversational-v4-a.json,conversational-v4-b.json tokenizer=T.json context=384
python3 scripts/check_panel_v4_novelty.py
```

**Freeze statement (v4).** Written on 2026-10-05; only `chat-grade check`, the unit tests
and the novelty script have read it. sha256 (also in `MANIFEST.sha256`):

| file | sha256 |
|---|---|
| `conversational-v4.json` | `bea6743c4d6c7c97eead6232179b6d0620455ab26565bb813a863da41c9c5cd1` |
| `conversational-v4-a.json` | `f0029d57d8ab54e33709e933528e34a158b324c16cc53fb03ca75fbf47466ba0` |
| `conversational-v4-b.json` | `57cbe574d7c09a4e48101b036161a4a06966833a29e2f3ab954a03bc254b889d` |
| `conversational-v4-checks.tsv` | `b4238e112e5e66cfd773de75f71ae8eb39c9077c172fe8ff0f4ba8b33cc5793e` |
| `conversational-v4-swaps.tsv` | `465b5fd4f555facdddbd71ed96e03b958b8458d0aecb002468c293d69eeb381c` |

Any edit makes a new panel with a new name. Replies were not inspected row by row before the
Step 6c acceptance run; after that run v4 became a development panel, as recorded above.

## Held-out memory acceptance panel `conversational-v5*` (References #2029)

**Status.** `conversational-v5*` was drawn and sealed on 2026-10-09, before any model had
replied to any of its requests, as the held-out acceptance panel for acceptance criterion 1
of #2029. Its acceptance criteria are frozen in the
[canonical record](../../docs/labs/panel-freeze-2026-10-09/README.md) and in its own checks
file. `conversational-v4*` above is a development panel from 2026-10-09 — its misses were
read and classified by name — so no held-out acceptance number can be measured on it any
more; v5 is the fresh draw that replaces it for that purpose.

64 rows, ids `conv-v5-*` (tier C), the same two categories, the same six-column check format
and the same rule as v4. `conversational-v5-a.json` holds the first 32 rows and `-b.json`
the rest (pass both). Row ids are zero-padded to three digits (`conv-v5-mem-001`) on purpose:
the novelty script reads every file here as a bag of words, so a two-digit id suffix
tokenises to a bare number, and two such numbers happen to be value words of the v4 panel.

| Category | Rows | Turns | Check |
|---|---|---|---|
| `multi_turn_memory` | 22 with three, 14 with two, 4 with four | 2–4 | `exact` (same rule as v3 and v4) |
| `unknowable_or_impossible` | 12 single, 12 with two | 1–2 | `abstain_exact` (same rule as v3 and v4) |

- **Values are drawn, never chosen.** Every expected value, every distractor and every
  answer-class word is selected by the deterministic rule `panel-v5-draw/1`
  (`scripts/panel_v5_draw.py`) from a named, hashable source, and each is recorded in
  `conversational-v5-provenance.tsv` with its source, that source's sha256 and the sha256
  digest that selects it; the source line, pool index and pool size of every draw are in the
  canonical record. The sentence frames, the keys and the scene catalogue are authored in
  that script and are not values. No model was consulted and no value was invented.
- **Subjects.** Personal names (the `propernames` list), place names (capitalised Webster's
  headwords attested in running English) and two-digit numbers. That is narrower than v4's
  subject range, and it is a declared deviation: these are the value classes for which this
  workspace has a named, hashable, non-invented source. The check rule, the row shape, the
  category split and the control set are unchanged.
- **Copy controls are not vacuous.** Half the memory rows state the expected value first and
  half state it last, so the check-only controls read `copy_first_stated` 20 and
  `copy_last_stated` 20 of the memory rows, against 20 of 20 for a bare expected spelling, 0
  for every constant, 0 for echoing the last turn and 0 for echoing the history. Constant 1
  passes every `unknowable_or_impossible` check (24 of 24), so that category is read only
  against its best constant, and each of the three adversarial abstentions passes 0 of 24.
- **New values and names.** No expected or distractor value word and no proper name of v5
  occurs in any earlier `data/panels/` file (`scripts/check_panel_v5_novelty.py`, the v4
  script's rule). The v4 checker passes unchanged against the enlarged vocabulary.
- **Leakage.** No v5 turn matches an M-world literal whole or as a template, or shares four
  consecutive words with one; the port of that unit test is
  `scripts/check_panel_v5_conformance.py`, which also re-checks the row shape and every
  check-only control listed above.

```text
chat-grade check requests=conversational-v5-a.json,conversational-v5-b.json tokenizer=T.json context=384 \
  checks=data/panels/conversational-v5-checks.tsv
python3 scripts/check_panel_v5_novelty.py
python3 scripts/check_panel_v5_conformance.py
```

**An acceptance run must pass `checks=`.** The frozen `chat-grade` binary embeds the v2, v3
and v4 checks files only, and this round was restricted to panel construction with no grader
run at all, so the v5 checks are not embedded in it. A run that omits `checks=` leaves every
v5 row unchecked, and a multi-turn row with no check is a hard error. The graded report
records the checks path and its sha256, so the run stays bound to this file.

**Freeze statement (v5).** Drawn and written on 2026-10-09. Only `scripts/panel_v5_draw.py`
(no model is loaded or consulted), `scripts/check_panel_v5_novelty.py`,
`scripts/check_panel_v5_conformance.py` and `shasum` have read it. sha256 (also in
`MANIFEST.sha256`):

| file | sha256 |
|---|---|
| `conversational-v5.json` | `e766cfe9a311eb85c465dba7a9d5b74961f8ccb75cd9f93b3b3e9337b992c750` |
| `conversational-v5-a.json` | `9345b60d1762be56d8b488521887574bfc54ce1e588e31be386c012a2f9f6e3d` |
| `conversational-v5-b.json` | `4daa10eee83020014df4322d3593c419d838f1bb0865c9f3305f825e0177813d` |
| `conversational-v5-checks.tsv` | `737c4dfd5a65fe49c207afe40e88bf354c7cf9dd45fa173c54e639bb0aa0dbc8` |
| `conversational-v5-swaps.tsv` | `6e3ae51469863239ff30b733af4ec0423cf230213ae2673fe71b0e79a9a484c8` |
| `conversational-v5-provenance.tsv` | `60e72c498294c61d997f643f4305bad8b936d78fbd456217498cadd11754b254` |

Any edit makes a new panel with a new name. **No model has replied to any v5 request.** Three
things a held-out claim needs from here: the panel must not be inspected by any model before
the acceptance run; the acceptance run must be declared in advance and run once, with the
frozen criteria applied as written; and the first model that replies to it makes it a
development panel like v4, after which any further number on it is open-development
evidence. `shasum -a 256 -c MANIFEST.sha256` from this directory verifies every file.

## Deterministic row checks for the open reply panel (`reply-panel-*`, References #2029)

The open reply panel (`everyday-32.json`, `heldout-200-a.json`, `heldout-200-b.json`, 232 rows,
archived as `ladder.tar`) is **not in this directory** and is **not a held-out panel**: Step 0a read
every failing row, and Results 6 and 7 measured that its reading is judge-only and that its accepted
set is not separable from its rejected set by any judge-free property once one canned greeting is set
aside. These files add the deterministic component over the rows where a judge-free criterion exists:

| file | contents |
|---|---|
| `reply-panel-checks.tsv` | eighty-eight row checks: kind `any` on 21 content rows (row-specific required content), kind `abstain_exact` on 67 rows from the recorded ill-posed id list |
| `reply-panel-checks-provenance.tsv` | one provenance row per check: the anchor, its source, and why it is not answerable from the request |
| `reply-panel-canned.txt` | 13 observed memorised replies, the input to the canned-reply control (`chat-grade check constants=`) |

**Coverage is the point: eighty-eight of the 232 rows (37.9 %) are exactly determinable; the other 144 are
open-ended requests with no expected content, so no deterministic check exists for them.** The
control was run and passes: **zero of the eighty-eight checks are passed by any of the 13 memorised strings**,
including the greeting that carries 18.6 % of accepted rows at 29M and 23.9 % at 100M. Validation and
the frozen target are in
[docs/labs/reply-panel-deterministic-checks-2026-10-09](../../docs/labs/reply-panel-deterministic-checks-2026-10-09/README.md).
The panel's requests are unchanged; these files add checks over them.

## Tiered eval v2: panel `conversational-v3*` and the missing-material K split

**Status (5 October, later; 9 October):** development panel. Its misses were inspected. Held-out
acceptance is `conversational-v5*` above; v4, which took that role in between, is a
development panel too since 9 October.

The #1724 review and the re-score (#1733) found five faults in the first tiered instrument:
(1) the qwen grader cannot fail a memory reply that recalls the wrong fact; (2) many
multi-turn rows did not depend on their earlier turn; (3) there was no per-category constant
control, and the fixed reply "I'm not sure. Can you tell me more about what you mean?" scored
42/160, ahead of every chat model, mostly on clarify and unknowable rows; (4) several rows
matched M-world training phrasings; (5) the first K split depended on model outcomes. The
second version of the instrument answers them with new files. The `conversational-v2*` and
`heldout-*-ids.txt` files below are unchanged and stay as the record of what the #1733
re-score used. The panel version is `v3` because the name `conversational-v2` was already
taken by that re-scored panel.

### Tier C v3 (`conversational-v3*`, ids `conv-v3-*`)

160 rows; `conversational-v3-a.json` holds rows 1–80 and `-b.json` rows 81–160 (pass both).
Rows interleave round-robin by category. Checks are in `conversational-v3-checks.tsv`
(6 columns: `id kind history terms forbid keys`), embedded in `chat-grade` together with the
v2 checks (the id sets are disjoint, so a v2 report keeps its v2 checks). The binding-swap
control replies are in `conversational-v3-swaps.tsv` (embedded too).

| Category | Rows | Multi-turn | Check | Read against |
|---|---|---|---|---|
| `multi_turn_memory` | 30 | 30 | `exact` | best constant (all score 0 on the check), copy and binding-swap controls |
| `self_contained_instruction` | 30 | 0 | `any` on 6 rows (as v2) | best constant |
| `clarify_or_on_topic` | 24 | 0 | `question` | best constant |
| `unknowable_or_impossible` | 24 | 0 | `abstain_exact` | best constant |
| `smalltalk_feelings` | 26 | 0 | none | best constant |
| `story_continuation` | 26 | 0 | none | best constant |

- **Memory rows (all rewritten).** Each conversation states the expected value and at least
  one value of the same type for another key (a distractor) in earlier user turns; the last
  turn names only the key. Examples: turtle Shelby / goldfish Flash, ask for the turtle;
  piano Saturday / swimming Wednesday; four rows are updates (beach then lake). `exact`
  passes only when the reply contains a spelling of the expected value (words lowercased,
  punctuation ignored) and none of the forbidden values: the conversation's distractors and,
  for a closed class (colours, days, the numbers 2–12), that class's other members. A wrong
  value fails, a hedge naming both values fails, and a reply naming no value fails. The rule is
  strict: a correct reply that also names the distractor ("Shelby, not Flash") fails.
  26 rows also list the distractor's *key* words (`keys`: the goldfish when the turtle is
  asked, Grandma when Grandpa is asked, the dragon when the queen is asked); a reply naming
  one fails, so the asked value bound to the wrong key ("The goldfish is Shelby", "Your dad
  drinks coffee") fails. Possessives are listed separately (`giant|giant's`). The other four
  rows have no key list: on the three update rows (beach then lake, white then blue, Pebble
  then Biscuit) the distractor is the superseded value of the same key, and on the rule row
  (hopping allowed, running not) the key word "allowed" also appears in a correct answer;
  there the swap is the other value, which `forbid` already fails. The same strictness
  applies: "Jada, Malik's sister, is nine" fails on the word Malik.
  `chat-grade check` refuses an `exact` row whose value or a forbidden value is in the last
  turn, or that has no distractor in an earlier turn, so no row can be answered from the last
  turn or by echoing the history. In 15 rows the expected value is stated first and in 15 it
  is stated last, so a model that copies the first or the last stated candidate passes 15/30;
  these two copy controls are in every report (`check_only_controls.copy`). A third control,
  `binding_swap`, applies one authored wrong-binding reply per row
  (`conversational-v3-swaps.tsv`) and must pass 0/30.
- **Only memory rows are multi-turn.** The v2 topic follow-ups (clar-17/18/23) were replaced
  by single-turn ambiguous requests.
- **Unknowable rows (`abstain_exact`).** A reply passes only when all of these hold:
  1. it contains an abstain phrase that is not followed by "wait" ("I can't wait" is not an
     abstention);
  2. it contains no agreement phrase ("yes", "of course", "let's", "here you go", ...), and no
     "sure" except after "not";
  3. no sentence contains a guess marker ("maybe", "probably", "I think", "might be", ...)
     unless the sentence is a question or hands the question back ("tell me", "ask", "let me
     know", "show me");
  4. in each sentence, the clause after "but" abstains again, hands the question back or is a
     question;
  5. it has no digit and no capitalised word the user did not write, other than I or OK. A
     sentence may start with a capitalised word only from a frozen opener list (pronouns,
     determiners, question words, auxiliaries, interjections, "Sorry", "Maybe", ...);
  6. it contains none of the row's forbidden words. All 24 rows now have a list. The 16
     knowledge rows list their answer class (colours, number words, foods, places for keys,
     pets, song titles, people who could stand behind you, ...). The eight impossible-action
     rows (unk-07/08/10/12/15/17/20/23) list words that perform or report the action ("on my
     way", "here's a hug", "it's open", "delicious", "beautiful", "poof", "turned off", "blast
     off").

  "I'm not sure, but I think it is blue." fails; it passed v2's `abstain`. Three adversarial
  replies are applied to every unknowable row (`check_only_controls.*.adversarial_abstentions`)
  and each must pass 0/24. They come from the #1734 review, where each passed 24/24 under the
  first v3 rule: "Yes! I can't wait to play with you!", "I'm not sure. Sam is the one." and
  "I don't know, but they are under the bed." The rule is still lexical. A plain
  assertion outside every listed pattern and answer class ("I don't know. It is the blue
  one." on a row whose list has no colours) still passes. A reply that offers a guess as a
  question passes. A reply that starts a sentence with a word not on the opener list fails
  ("Imagine ...").
- **Leakage.** No turn matches an M-world phrasing whole or as a template, or shares 4
  consecutive words with any string literal of `milestone_world.rs` or `milestone_world_v2.rs`.
  The unit test `panel_v3_shares_no_m_world_phrasing` checks this on every build;
  `chat-grade leak strict=...` (default `strict_n=4`) reports it. The same strict check finds
  5 leaking rows in v2 (unk-04 "tell me the color of", story-20 "a big red ball", unk-22
  "What is my favorite song?", talk-24 "Nice to meet you", do-26 "word that means the"); all
  5 were rewritten in v3. The full corpus check (references at 6 words, corpus lines whole) also
  finds no v3 leak; four rows share an 8-word run with a corpus line (story-01/24/26, talk-17,
  TinyStories phrasing, as in v2), reported and not counted.
- **Reused rows.** Smalltalk, instruction and story rows are v2's, with three rewritten for
  leakage (talk-24, do-26, story-20); clarify rows are v2's single-turn ones plus three new;
  unknowable rows are v2's with unk-02/04/14/22 rewritten to avoid the M-world "What is my
  ...?" frames.

`chat-grade check` on v3 (tokenizer of the ladder models, context 384): pass, worst case
336 positions (conv-v3-mem-16). Check-only controls (no grader):

| Category | checked | constant 1 | constants 2, 3 | echo last turn | echo history | copy first / last | binding swap | adversarial abstentions |
|---|---|---|---|---|---|---|---|---|
| `multi_turn_memory` | 30 | 0 | 0 | 0 | 0 | 15 / 15 | 0 | – |
| `self_contained_instruction` | 6 | 0 | 0 | 0 | 0 | – | – | – |
| `clarify_or_on_topic` | 24 | 24 | 0 | 17 | 17 | – | – | – |
| `unknowable_or_impossible` | 24 | 24 | 0 | 0 | 0 | – | – | 0, 0, 0 |

Constant 1 still passes every clarify and unknowable check, so those two categories can show
a model gain only where the grader rejects the constant. They are read only through the
best-constant comparison below.

### Headline: per category, against the best constant

Every graded report (and `tiers`) now carries `per_tier.<tier>.headline`: for each category,
the constant reply with the most acceptable rows in that category (ties to the lower index),
the model's and the constant's acceptable counts, `model_minus_constant`, the discordant rows
and the two-sided exact McNemar p, and `model_beats_constant` (more model-only rows and
p < 0.05). For checked rows the same comparison is given for the check alone. The report's
top-level `headline` lists, per tier, the categories where the model beats its best constant.
A tier total is never the headline. Choosing the best constant after grading favours the
constant, so the comparison is conservative for the model. `compare` now also gives the paired
McNemar per category.

### Tier K by one text-only rule (`heldout-ill-posed-v3*`, `heldout-clean-v3-ids.txt`)

`chat-grade ill-posed requests=heldout-200-a.json,heldout-200-b.json` applies one criterion to
every heldout-200 request text: **the request refers to material that its text does not
contain.** The rule is code (`missing_material` in `chat-grade.rs`, unit-tested), reads only
the text, and never sees a reply, grade or model. It fires on one of: `open_end` (the text ends
with `:` or `,`; the panel keeps only a turn's first line, so the introduced material is
absent), `slot` (an unfilled `[placeholder]`), `constraints_only` (every sentence constrains
"your response" to a query the text does not hold), `deictic` ("the following", "below",
"this essay", "the passage", "the recipe", "your suggestion", a clause-final "this" in a
one-sentence request, ...; material not included), `own_material` ("I wrote", "here is", ...;
material not included) and `variable` (a bare x, y or n with no equation). Material counts as
included with a quoted span of 3 or more words, or 25 words or 3 sentences after the first
colon.

Result: 67 ill-posed (open_end 27, deictic 18, constraints_only 12, slot 6, own_material 3,
variable 1) and 133 clean. `chat-grade` embeds `heldout-ill-posed-v3-ids.txt` as the default
list. Against the hand-labelled v2 list (90): the 67 are a subset of the 90; the 23 rows
labelled only in v2 are 21 `fragment` rows that refer to nothing missing (salutations without a
comma, headlines, list items, quotations such as heldout-047 "Hey Alex!!!! 🚀") and two
`absent` rows the rule does not see (heldout-112 "has collected data ...", heldout-137 "the
animal joke series"). Under the one criterion they are clean; K-clean therefore still holds
some rows a reader may find hard to answer. The K rows of reports graded before this change
keep their recorded list; `chat-grade tiers` re-tiers them with the v3 list unless
`ill_posed=heldout-ill-posed-ids.txt` is passed.

### Freeze statement (v3)

`conversational-v3*` and `heldout-*-v3*` were written on 2026-10-05, before any model had
replied to any v3 request. Only `chat-grade check`, `chat-grade leak` (references and the
decoded corpora; no model), `chat-grade ill-posed` (heldout-200 request text only) and the unit
tests have read them. sha256 (also in `MANIFEST.sha256`):

| file | sha256 |
|---|---|
| `conversational-v3.json` | `578e256448b9790d3164d88aea12b9a72d015b915dd1492868dd160f8b50a163` |
| `conversational-v3-a.json` | `425ad695463a7533f08a4a4ab6780815fe6c062d01cf76dcdb74f6a45744a9b8` |
| `conversational-v3-b.json` | `78e23f2a9bba865340a4a58f9cdc68cd34a55ae3612c2fdb8239a0f335b4ee22` |
| `conversational-v3-checks.tsv` | `be82781146e500fe948e7a5a3062d09212f1d4c524c3537c1850faf1657c30c3` |
| `conversational-v3-swaps.tsv` | `30b521f1c34fdb91512753a02e6163d153e2a34fd1212f754b9153a6b0082714` |
| `heldout-ill-posed-v3.tsv` | `8ef79a1609ab89aec06d6efcdcbbda4335d5e041e9d133362f112da4abd788a5` |
| `heldout-ill-posed-v3-ids.txt` | `2417ce8ce569724e8a6a79c39993c171f23c52107b293abfd87af6acacab4d37` |
| `heldout-clean-v3-ids.txt` | `1ce055d48429a0cb51ca6b3931180befd3729d51802d8a1cf0653d822939865b` |

Any edit makes a new panel with a new name.

**Check revision (5 October, #1734 review).** The panel rows (`conversational-v3*.json`) are
unchanged. The checks file was revised in place before any model had replied to or been
graded on v3, so no report is bound to the first version. The first version
(`conversational-v3-checks.tsv`, sha256
`0823e316da1e1e6d46b0bba6ed596828919344ce24c337d830c71ad4fca0b0f9`) had three faults:
`abstain_exact` accepted agreement ("I can't wait"), a made-up name at a sentence start and
a lowercase made-up answer; `exact` accepted the asked value bound to the distractor's key.
This revision adds the `keys` column (26 memory rows), answer-class or action word lists for
the 18 unknowable rows that had none (and extends unk-12's), the stricter `abstain_exact`
rule above in `chat-grade`, and the binding-swap file. The v3 rows, the copy controls (15/15)
and the constant results (0 on memory, 24/24 for constant 1 on clarify and unknowable) are
unchanged.

```text
chat-grade check requests=conversational-v3-a.json,conversational-v3-b.json tokenizer=T.json context=384
chat-grade grade ... requests=conversational-v3-a.json,conversational-v3-b.json   # default constants
chat-grade ill-posed requests=heldout-200-a.json,heldout-200-b.json expect=data/panels/heldout-ill-posed-v3-ids.txt
chat-grade leak requests=conversational-v3-a.json,conversational-v3-b.json reference=... \
  strict=crates/uor-r4-training/src/milestone_world.rs,crates/uor-r4-training/src/milestone_world_v2.rs
```

## Tier C v2: conversational panel (`conversational-v2*`) -- record, superseded by v3

- **What it is:** 160 requests that need no world knowledge. Each one is answerable from the
  conversation itself or from general conversational sense.
- **Format:** the `chat-grade` request format (`id`, `category`, `user_turns`). Ids are
  `conv-*`, which is how `chat-grade` assigns tier C.
- **Files:** `conversational-v2.json` holds all 160 rows. A `chat-grade` panel file is capped
  at 128 requests, so `conversational-v2-a.json` (rows 1–80) and `conversational-v2-b.json`
  (rows 81–160) split it for loading; pass both together.
- **Row checks:** `conversational-v2-checks.tsv` (embedded in `chat-grade`; see below).
- **Row order:** rows interleave round-robin by category, so the derangement control's
  neighbouring row is always from a different category. The derangement therefore catches
  off-topic replies; generic replies are caught by the constant-reply control instead.

| Category | Rows | Multi-turn | Row check | Tests |
|---|---|---|---|---|
| `smalltalk_feelings` | 26 | 0 | none | greetings, feelings, small news |
| `multi_turn_memory` | 30 | 30 | `any` (27 recall, 3 derived) | the last turn needs a fact stated in an earlier user turn |
| `self_contained_instruction` | 30 | 0 | `any` on 6 rows with one determinate answer not in the turn | rewrite, list, count, rhyme, order words |
| `clarify_or_on_topic` | 24 | 3 | `question` on the 21 ambiguous single turns; `any` (topic) on the 3 follow-ups | ambiguous requests should get a clarifying question; follow-ups need the earlier topic |
| `story_continuation` | 26 | 0 | none | short TinyStories-register stories and continuations |
| `unknowable_or_impossible` | 24 | 0 | `abstain` | personal facts with no context, physically impossible asks |

Every multi-turn row depends on its earlier turns: no check term is in the last user turn
(`chat-grade check` refuses the panel otherwise), and for a `recall` row a term is in an earlier
user turn. No row's answer depends on the model's own earlier reply.

### Why v2 (v1 withdrawn before any model replied)

`conversational-160*` (commit 3564f9c1, review on #1724) was withdrawn. No model had replied
to it. v2 keeps v1's ids, categories and order and changes these rows:

- **Grader blind to history (review b).** The grader's relevance question asks only whether a
  reply answers the *last* message, so "Your name is Tom." passed a recall row. v2 adds the
  frozen row checks below; v1's keys sidecar was never read by `chat-grade`.
- **Rows that did not depend on the earlier turn (review b):** mem-03/08/13/16/19/24/25 were
  rewritten so the answer is a stated fact (mem-24 no longer depends on the model's own story);
  clar-17/18/23 now ask a follow-up that names nothing without the first turn; story-23/24 and
  unk-21 are single turn now. Multi-turn rows: 36 → 33, all history dependent.
- **Leakage into training text (review d):** rows matching M-world v1 intents (clarify, greet,
  how_are_you, thanks, farewell), relation templates (`My name is {v}.`, `I live in {v}.`,
  `What color is {x}?`, `My lucky number is {v}.`, ...), the AERM dev-mem panel or whole user
  lines of the training corpora were rewritten: talk-02/03/05/09/19/22, clar-01/03/05/08/14/19/21,
  mem-01/02/03/10/14/15/22/26/29, do-02/14, unk-04. clar-12/16 were made ambiguous so a question
  check applies. `chat-grade leak` found 15 leaking v1 rows and finds none in v2 (below).

### Row checks (`conversational-v2-checks.tsv`)

Tab-separated `id kind history terms`. A checked row is acceptable only when the grader judges
it fluent and relevant **and** it passes its check:

- `any`: the reply contains one of the `|`-separated words or phrases (words lowercased, split
  at anything not a letter, digit or apostrophe; a phrase matches consecutive words).
- `abstain`: the reply contains one of the fixed abstain phrases (`don't know`, `not sure`,
  `can't`, `cannot`, `no way to know`, ...; listed in every report).
- `question`: the reply contains `?`.

### Controls

Every graded row is also graded with (a) its reply placed after the next row's conversation
(the derangement) and (b) each of three constant replies in place of the model's
(`DEFAULT_CONSTANTS` in `chat-grade.rs`; `constants=none|FILE` overrides): "I'm not sure. Can you
tell me more about what you mean?", "That sounds nice! Thank you for telling me." and a
TinyStories opening. Controls go through the same checks. Each report gives, per tier and per
category, the paired McNemar of actual against the derangement and against each constant.
The check-only controls (no grader) apply each check to the row's last user turn and to all its
user turns joined.

`chat-grade check` gives the check-only pass counts before any model replies:

| Category | checked | constant 1 | constants 2, 3 | echo last turn | echo history |
|---|---|---|---|---|---|
| `multi_turn_memory` | 30 | 0 | 0 | 0 | 27 |
| `self_contained_instruction` | 6 | 0 | 0 | 0 | 0 |
| `clarify_or_on_topic` | 24 | 21 | 0 | 14 | 14 |
| `unknowable_or_impossible` | 24 | 24 | 0 | 0 | 0 |

So the check alone does not separate a model from constant 1 on `clarify_or_on_topic` and
`unknowable_or_impossible`: a generic "I'm not sure, can you tell me more?" is a correct answer
to most of those rows. Those two categories, and the unchecked `smalltalk_feelings` and
`story_continuation`, are read only through the paired test against the best constant for the
category; a category the model does not beat its constants on says nothing about the model.
An echo of the history passes most memory checks, so a memory pass also needs the grader's
fluent/relevant verdict on a reply that is not a copy.

### Leakage check

```text
chat-grade leak requests=conversational-v2-a.json,conversational-v2-b.json \
  reference=crates/uor-r4-training/src/{milestone_world,milestone_world_v2,stack_aerm,stack_dialogue}.rs,\
crates/uor-r4-core/src/answer_oracle.rs,docs/integration/paraphrase-review-2026-10-01/decisions.tsv \
  whole_only=everyday-32.json,stretch-32.json,chat-v0-20260925/panel-requests-dev.json \
  corpora=ladder/corpora/ft-mixed-mworld-chatv0,ladder/corpora/tinydialogues-train tokenizer=T.json
```

A turn leaks when its words equal a reference text's or a corpus line's (role label stripped;
a `{placeholder}` stands for 1–4 words in a template of at least 3 literal words), or when it
shares 6 or more consecutive words with a `reference=` text. The corpora are the decoded token
stores of the `ft-mixed-mworld-chatv0` fine-tuning arms (M-world paraphrases plus chat-v0 train,
147,359 documents) and TinyDialogues train (110,024 documents); this is the check that was
UNAVAILABLE in the review, made by decoding the local token stores. v1: 15 leaking rows. v2:
none. Four v2 rows share an 8-word run with a corpus line (story-01/24/26, talk-17; TinyStories
phrasing such as "once upon a time there was a little bunny"); that is reported, not counted.
Not checked: TinyStories train (story register by design) and the sealed §8 panel (never read).

**Freeze statement.** v2 was written on 2026-10-05 from v1 and the #1724 review, before any
model had replied to any v1 or v2 request. Only `chat-grade check` and `chat-grade leak`
(tokenizer, references and corpora; no model) have read it. The sha256 values are in
`MANIFEST.sha256` and posted on #820 before the first reply run. Any edit makes a new panel with
a new name.

## Tier K v2: hand-labelled split (`heldout-ill-posed*.tsv`, `heldout-*-ids.txt`) -- record, superseded by the v3 rule

- **What it is:** the 200 heldout-200 rows, split into K-clean (110) and K-ill-posed (90).
- **Criterion (one rule, request text only, applied to all 200 rows).** A row is ill-posed when
  its request text, read alone, cannot be answered as asked because:
  - `absent`: it points at specific material it does not contain and that is not common
    knowledge ("the following essay", "this code", "the recipe", "the region", "your
    suggestion", "Read the passage below ...");
  - `template`: it has an unfilled slot (`[city]`, `[event]`);
  - `constraints_only`: it holds only output-format constraints and no task;
  - `fragment`: it holds no request and reads as a piece of a document (a letter salutation,
    a post's opening line, a recipe step, a list item, a quotation, a citation, "Given the
    text: ..." with no question, a sentence cut off).
  A request about a named, widely known thing, a role-play set-up, or a request that includes
  its material is clean. Vague or knowledge-heavy requests are clean.
- **No model outcome was used.** v1 combined the Step 0a labels (which covered failing rows
  only) with a screen applied only to rows both models passed, so membership depended on the
  29M/96M outcomes (review e). v2 labels every row from its text alone. As a cross-check only:
  all 55 0a `ill=1` ids and all 16 v1 screen ids are ill-posed under the text rule; heldout-051
  was also in 0a; 19 rows are new (001, 016, 070, 086, 087, 097, 117, 123, 124, 128, 135, 139,
  141, 150, 174, 175, 176, 180, 195). Twins in form now share a tier: 002/117/144 and 030/128.
- **`heldout-ill-posed.tsv`:** each ill-posed id with its reason.
- **`heldout-ill-posed-ids.txt`:** the 90 ids of K-ill-posed. `chat-grade` embeds this file as
  its default list; `ill_posed=` overrides it.
- **`heldout-clean-ids.txt`:** the 110 ids of K-clean.

To grade only the clean rows, pass `exclude=data/panels/heldout-ill-posed-ids.txt` to
`chat-grade grade` or `reply`, or write a new panel file with
`chat-grade filter out=NEW.json requests=heldout-200-a.json,heldout-200-b.json exclude=...`.
Neither changes the original panel files.

## Scoring per tier

`grade` and `grade-replies` reports keep their fields (`actual`, `control_derangement`,
`paired_against_control`, `control_rule`, `per_category`, `rows`) and add `control_constants`,
`check_only_controls`, `per_category_controls`, `per_tier`, `tiers`, `checks` and `constants`.
`acceptable` includes the row check where a row has one; `fluent_and_relevant` is the old count.
Rows without a check (all K and everyday rows) score exactly as before.

```text
chat-grade tiers report=REPORT.json [checks=...] [out=NEW_ROOT]   # re-tier, no regrading
chat-grade compare a=A/report.json b=B/report.json [tier=C] [checks=...] [out=NEW_ROOT]
chat-grade check requests=conversational-v2-a.json,conversational-v2-b.json tokenizer=T.json context=384
```

`compare` pairs rows by id within each tier and gives two-sided exact McNemar results for
acceptable, fluent, relevant and check pass. Rows present in only one report are counted under
`unpaired`. Different graders give a warning and `same_grader: false`. Unknown categories are
an error.

## Copies

The copies in `~/uor-r4-local/ladder/panel/` are byte-identical to these files; see
`MANIFEST.sha256`. The withdrawn v1 files stay there, marked withdrawn in its `README.txt`.
**Path note (2026-10-09):** that ladder directory is no longer on the laptop. It is archived as
`icloud:UOR-R4/results/deepseek/ladder.tar` (3,306,919,424 B, md5 `ed9456d63472b7a424fe6ca62d36baa8`;
`~/.local/share/uor-r4/bin/cloud-store fetch ladder <dest>`), which is where the byte-identical copies
and the withdrawn v1 files now live.

## The open reply panel (232 requests) is not in this directory

Acceptance criterion 1 of #2029 (`≥ 116/232 acceptable`) is measured on a different instrument: the
open reply panel, `ladder/panel/everyday-32.json` (32 rows, sha256
`945c0c97196d39db430b5821c8c30a888a3c719c6e701c218fc7a914b350b480`) plus `heldout-200-a.json`
(`019cc6f6…d2c1`, 100 rows) and `heldout-200-b.json` (`5434cfd9…436c`, 100 rows). It has **no frozen
row checks** — no checks file lists its `talk-*`/`do-*`/`ask-*`/`follow-*`/`heldout-*` ids — so its
`acceptable` reading is the qwen2.5:7b judge alone, with no deterministic `check_pass` to be primary
against, unlike every panel on this page. Its files are in the `ladder.tar` archive above and in
`icloud:UOR-R4/results/deepseek/reply-panel-open-2026-10-09.tar`; the classification of its 189
failures, which are diffuse, is in
[docs/labs/open-reply-panel-2026-10-09](../../docs/labs/open-reply-panel-2026-10-09/README.md).
