# A deterministic sub-reading for the open reply panel: 88 row checks, and a canned reply cannot pass one

References #2029 (M1 acceptance criterion 1). Lab: DeepSeek. 2026-10-09. **CPU only: no pod, no
generation, no training, no grading. $0.** One source build of `chat-grade` from main (5m37s) and
`chat-grade check`.

## Question

Results 4–7 measured four defects in the reply panel's reading, one after another: its cap is not
the constraint (5); its failures are diffuse across two classifications (3, 6); it is judge-only,
with no frozen row checks at all; and Result 7 showed the accepted set is **not separable from the
completed-but-rejected set by any of fourteen judge-free properties** once one canned greeting is
set aside — that greeting being **18.6 % of all accepted rows at 29M and 23.9 % at 100M**. A
milestone cannot be gated by a judge whose line the text does not show.

The question this piece answers is whether a deterministic component can be built for that panel,
in the shape the v4/v5 memory panels use, such that **a canned or memorised reply cannot pass it** —
and if so, over how much of the panel.

## The headline is the coverage, and it is a finding, not a methods note

**Measured before a single check was authored**, from the 232 requests themselves:

| | rows | share |
|---|---:|---:|
| **total panel** | **232** | 100 % |
| **exactly determinable, checks built** | **88** | **37.9 %** |
| — ill-posed rows (`abstain_exact`) | 67 | 28.9 % |
| — content rows (`any`, required content) | 21 | 9.1 % |
| **no judge-free correct answer exists** | **144** | **62.1 %** |

The 144 are open-ended requests — "What are the key components of a successful marketing campaign?",
"Write a detailed description of a character's journey to becoming a famous fashion designer",
"Develop a catchphrase for a bike shop." For those there is **no expected content to check**: any
fluent, on-topic reply is as correct as any other, so "acceptable" is irreducibly a judgement.

**This is a stronger statement than Result 6's.** Result 6 said the instrument is judge-only and the
judge's line is not in the text. This says something more basic: **for most of this panel there is no
judge-free correct answer to check, because the request is open-ended.** A panel two-thirds of which
cannot be scored deterministically cannot gate a milestone no matter how good its judge is.

**So the panel can carry a deterministic SUB-READING, not a deterministic replacement for
116/232.** And criterion 1 remains **NOT MET**: a new instrument is not a met milestone.

## What was built

Three files in `data/panels/`, sealed in `MANIFEST.sha256` (29 files verify from `data/panels/`):

| file | sha256 | contents |
|---|---|---|
| `reply-panel-checks.tsv` | `ad4e9a142a0c864865678fa1c2b8f7d099fd38f02a475c3ff8c0afd44a1b05a3` | 88 row checks |
| `reply-panel-checks-provenance.tsv` | `01de5bb28198b05554ce64a35efa401b573fdb1dfa8e81ef54a9e9a3d3211f9d` | 88 provenance rows |
| `reply-panel-canned.txt` | `ef8bb14718d4ab58f9e47850d38a4099a43b2c2d89c5d94f72ea7d4d091e86a7` | 13 memorised replies (control input) |

The panel's requests are **unchanged**: these are checks laid over the existing sealed panel, which
remains development evidence.

**Two kinds, because the panel has two kinds of row.**

- **Content rows, kind `any`** (21 rows): the reply must contain at least one of the row's anchors,
  which are the content the answer must contain — `ask-02` → `honey`; `ask-06` → `cow|chicken|pig|
  horse|sheep|goat|duck|animal`; `heldout-043` → `96`; `heldout-145` → `extract|website|html|
  requests|beautifulsoup|automat`. **The anchors differ per row and none occurs in any canned reply,
  so a single canned string cannot pass across rows**: it would have to contain every row's content
  word at once. That is the mechanism, not a promise.
- **Ill-posed rows, kind `abstain_exact`** (67 rows): taken from chat-grade's own recorded
  `data/panels/heldout-ill-posed-v3-ids.txt`, so the row set is selected by the instrument's existing
  rule rather than by hand. The request's text does not contain the material it refers to; the
  correct behaviour is to ask for it or decline, and the defect is inventing it. `abstain_exact` is
  that rule: it requires an abstention and **rejects a fabricated specific** (no digit and no
  capitalised word the user did not write).

**Authoring discipline, auditable in the provenance file.** Anchors were written from the *request*
and from what the answer must contain, never from what the model replied; the observed replies were
used only to build the control list. `scripts/reply_panel_checks_build.py` drops any anchor that
occurs in the row's own last user turn (a row answerable by echoing the request is not a check) and
any anchor that occurs in a canned reply. One anchor was dropped by those rules and the drop is
recorded: `heldout-132`'s `lemon` is in the request.

## The canned-reply control — run, not promised

Run **by `chat-grade check constants=`**, so the control arm is the frozen grader's own
`RowCheck::passes` (via `check_only_controls`) rather than a reimplementation of it:

```text
chat-grade check requests=everyday-32.json,heldout-200-a.json,heldout-200-b.json \
  tokenizer=tokenizer.json context=384 \
  checks=data/panels/reply-panel-checks.tsv constants=data/panels/reply-panel-canned.txt
```

**Result: PASS. Thirteen memorised strings, 88 checked rows, zero rows passed.**

| control reading | value |
|---|---|
| total canned-constant passes, all categories | **0** |
| `Hello! How can I help you today?` (18.6 % / 23.9 % of accepted rows) | **0 of 88** |
| `I'm good, thanks for asking.` | 0 of 88 |
| `I'm a helpful assistant that runs on your computer.` | 0 of 88 |
| `I understand your request and will provide a response that meets the specified constraints.` | 0 of 88 |
| the remaining nine memorised replies | 0 of 88 each |

The greeting that carries a fifth to a quarter of the accepted set passes **none** of the
deterministic checks. That is the measurement Result 7 predicted, and it is what makes any future
number on this sub-reading interpretable.

## Structural validation — `chat-grade check`, from a source build at main

| | |
|---|---|
| `check_panel` | **pass** |
| requests | 232 (8 multi-turn) |
| **checked rows** | **88 of 88** — `checked_rows_by_history {"none": 88}`, `checks_without_loaded_request: []` |
| by category and kind | `heldout_first_turn/abstain_exact` 67, `heldout_first_turn/any` 14, `simple_question/any` 7 |
| **worst-case context position** | **168** (`follow-02`, of `context=384`) |
| checks source | `data/panels/reply-panel-checks.tsv`, 88 rows, sha256 `ad4e9a14…` |
| novelty | `check_panel_v4_novelty.py` **pass** (2,558-word vocabulary, 26 files) and `check_panel_v5_novelty.py` **pass**, both unchanged and re-run with the new files present |
| manifest | `shasum -a 256 -c MANIFEST.sha256` from `data/panels/` → **29 files, 29 OK** |

Note on the binary: the **frozen macOS artifact `a5071cfe…` predates the `check` subcommand** — its
usage line is `extract|grade` — so this validation used a build of `chat-grade` from current main
(5m37s, CPU, no model). The `check` subcommand is dispatched at
`crates/uor-r4-training/src/bin/chat-grade.rs:216`. This is a small, real gap in the frozen artifact
and is recorded; it does not affect the sealed runs, which used `grade`.

## The frozen target, and the first reading

**Frozen before any model is run against these checks.** The primary deterministic reading is
`check_pass` over the **21 content rows**, because that is the component a canned reply cannot pass;
the 67 ill-posed rows are reported secondarily and read against their best constant, because a
constant abstention passes `abstain_exact` by design — the same convention v3 uses for its clarify
and unknowable categories. Any future number on this sub-reading is reported per kind and per
category, with the failures named.

**First reading, retrospective** — measured on the already-sealed 29M replies (`chat-29m-B-lr5e-4`,
43/232 by the judge), so it is a reading of the new instrument on old data, not a new model result:

| population | checked | `check_pass` |
|---|---:|---:|
| **content rows (`any`)** | **21** | **2** |
| abstain rows (`abstain_exact`) | 67 | **0** |
| **total** | **88** | **2** |

The two that pass are `ask-03` (the reply contains "hydrated") and `ask-06` (it names a horse). The
rest are substantive misses, not formatting: `ask-01` answers "Why do leaves change colour?" with
"The fall is change color in the fall."; `ask-07` answers "Why is the sun hot?" with "The sun is
hot."; `heldout-103` answers "Write down all the prime numbers from 1-12, and add up their total"
with "1, 2, 3, 4, 5, 6, 7, 8, 9, 10."; `heldout-043` never states the divisor count of 27720.

**The 0/67 on the ill-posed rows is a real diagnostic, with one measured limit.** No reply in the
sealed set both abstained and avoided fabrication: **0 of 67 abstain strictly, against 27 of 67 that
name a fabricated specific** — the model invents content for requests whose material is missing.
The limit: `abstain_exact` does not credit a *clarifying question*, and **8 of 67 replies ask one**,
which is legitimate behaviour. It is used here because the alternative is not safe — see below.

## The grammar gap, measured

The intended third component — **forbidden distractors and the distractor's key words** — is **not
expressible for a single-turn row with the current grammar**, and this is the piece's main
unfinished part:

- `forbid` and `keys` are parsed only for `exact` and (forbid only) `abstain_exact`.
- `exact` **cannot validate on a single-turn row**: `validate_checks` requires
  `history=recall`, a check term in an *earlier* user turn, and a distractor value stated in an
  earlier turn — a 224-of-232 single-turn panel has no earlier turn.
- So the content checks can require expected content but cannot forbid a distractor, and the
  ill-posed checks get their negative component from `abstain_exact`'s built-in
  `fabricated_specifics` rule rather than from a per-row `forbid` list.

And the natural repair for the ill-posed rows is **unsafe as it stands**, which is why it was not
used: a `question`-kind check ("the reply asks for the missing material") **would be passed by the
canned greeting on all 67 rows**, because "Hello! How can I help you today?" contains a question
mark and names no fabricated specific. A hypothetical "asks a question AND names no fabricated
specific" kind measures **8 of 67** — but it is canned-reply-passable, so it is not acceptable as a
deterministic check. The exact grammar change needed is an additive kind —
`reply_exact <history=none> <terms> <forbid> [keys]`, `exact`'s rule without the recall/multi-turn
requirement — so a single-turn row can require expected content, forbid distractors, and forbid a
canned reply, and so a `clarify` variant can require the missing material to be named without being
satisfiable by a generic greeting.

## Cost

Laptop CPU only: a 5m37s `chat-grade` build, `chat-grade check` (seconds), and two Python scripts.
**$0.00, no pod.**

## Limitations

- **62.1 % of the panel is unchecked and cannot be checked.** This is the finding, not a shortfall
  of effort: those requests have no expected content.
- **No `forbid`/`keys` component** for the reasons above; the ill-posed rows rely on
  `abstain_exact`'s built-in fabricated-specific rule.
- **`abstain_exact` does not credit a clarifying question** (8 of 67 would ask one), so 0/67
  understates how often the model behaves acceptably on ill-posed rows. It is reported as the check's
  reading, not as a model score.
- **The content anchors are a necessary-content check, not a sufficient one.** A reply containing
  `honey` may still be wrong; it cannot be *right without it*. That is the same character as the
  memory panels' `exact` check and is stated rather than glossed.
- **The 21 content rows were authored by reading the requests**, so they are an authored instrument
  over a panel whose failures are already known. The anchors are request-derived and mechanically
  filtered, and the panel is development evidence either way — no held-out claim is made.
- **The first reading is retrospective**, on replies that already existed.

## Recommendation — the restatement, and what I would choose

**Restate criterion 1's reply half as a deterministic sub-reading over the 37.9 % that is exactly
determinable, and give the open-ended remainder no gate at all.** Not a second instrument for the
remainder. The reasons, in order:

1. A gate on the open-ended 62.1 % would be **the judge again** — the reading Result 6 and Result 7
   showed is not interpretable, and a quarter of which is one memorised string.
2. Building a second instrument for open-ended conversation is **a different and much larger
   project** than this line can justify, and Step 0a already recorded that the lever for this panel
   is the float model's corpus/knowledge/capacity, which no bounded readiness instrument reaches.
3. The determinable third is **a real target with a real defect in it**: 67 ill-posed rows where
   "ask for the missing material or decline" is checkable, and where the model currently invents a
   fabricated specific on 27 of them and abstains on none; plus 21 content rows where it currently
   passes 2.

**If the sub-reading is adopted, its first move should be the additive `reply_exact` kind above**,
because that closes the two gaps this piece could not: a per-row forbidden-distractor component, and
a canned-reply-proof clarify criterion for the 67 ill-posed rows.

## The `reply_exact` grammar addition: discrimination is now expressible on a single-turn row

The first piece of this record could require expected content but **could not forbid a distractor**,
because `forbid` and `keys` parse only for `exact` (which `validate_checks` requires to be a
multi-turn recall row) and for `abstain_exact`. 224 of the panel's 232 rows are single-turn, so the
three-component shape — required content, forbidden distractors, distractor keys — could not be
written for them. With anchors alone a reply passes by containing the right content word; it says
nothing about whether it **also** contains the wrong one.

**What was added, and nothing broader:** `CheckKind::ReplyExact` — `exact`'s matching rule without
the recall precondition, permitted with `history=none`. Eight sites in
`crates/uor-r4-training/src/bin/chat-grade.rs`: the enum arm, `name()`, `passes()` (sharing the exact
arm), the parser (with the same `terms != "-"` guard), `forbid` and `keys` acceptance, the
expected/forbidden/key overlap refusals, and a `validate_checks` arm that requires the terms to be
absent from the last user turn and the distractors and keys to be absent from **every** user turn —
a distractor the user wrote is not a distractor. No existing kind's behaviour changed.

**Focused tests, exercising the changed path** (`cargo test -p uor-r4-training --bin chat-grade`,
**17 passed, 0 failed**), new test `reply_exact_is_exact_without_the_recall_requirement`:

| case | result |
|---|---|
| accepts a reply containing the required content | pass |
| rejects a reply missing the required content | pass |
| rejects a reply containing a forbidden distractor ("They make honey, not vinegar") | pass |
| rejects a reply naming the distractor's key ("The wasp makes honey") | pass |
| **validates on a single-turn row, where the same row as `exact` is refused** | pass |
| refuses a term in the request, a distractor in the request, a key in the request | pass |
| refuses a missing distractor, a term that is also forbidden, a term that is also a key | pass |

### Distractors: five rows, not eighty-eight

A distractor no plausible reply would contain is noise, so the column was authored only where a
wrong-but-plausible alternative genuinely exists — a value a confused or hedging reply would
actually contain, that is not the required content, is not in the request, and is not itself a
correct answer. **Five of the 21 content rows got one; the other 16 did not.**

| row | required content | forbidden distractor | why it is plausible |
|---|---|---|---|
| `ask-06` | a farm animal | `lion|tiger|elephant|penguin|shark|dolphin` | a wild animal named as a farm animal is a real error |
| `heldout-022` | `heapq|heapify|heappush|heappop|push` | `max heap|maxheap` | the request asks for a **Min** heap; max-heap is the wrong polarity |
| `heldout-043` | `96` | `48` | 27720 has 96 divisors; 48 is the half-count error |
| `heldout-103` | `28` | `17|55` | the primes 1-12 total 28; 17 forgets 11 and 55 sums 1..10 |
| `heldout-126` | `printf|stdio|include|main(` | `cout|System.out|console.log|print` (keys `java|javascript|python|cpp`) | a C request answered in C++, Java or JavaScript |

**Two candidate rows were rejected, and the rejection is the interesting part.** `ask-07` ("Why is
the sun hot?", whose misconception is "a ball of fire") was dropped because **negating** the
misconception is a normal correct phrasing — "it is not a ball of fire, it is nuclear fusion" —
so a distractor check would fail correct replies. `ask-02` ("What do bees make?") was dropped
because bees really do make wax and royal jelly, so the near alternatives are correct answers. The
remaining 14 content rows have no specific wrong word: their wrong answers are **missing or vague
content**, which the anchored `any` check already catches.

### Re-seal and re-validation at the changed head

| | before this piece | after |
|---|---|---|
| checks file sha256 | `ad4e9a14…` | **`d24ae404be11d25b…`** |
| kinds | `any` 21, `abstain_exact` 67 | **`any` 16, `reply_exact` 5, `abstain_exact` 67** |
| checked rows | 88 | **88** (37.9 % of the panel, unchanged) |
| `check_panel` | pass | **pass** |
| worst-case context position | 168 of 384 | **168** (`follow-02`) |
| `checks_without_loaded_request` | `[]` | **`[]`** |
| `shasum -a 256 -c MANIFEST.sha256` from `data/panels/` | 29 OK | **29 files, 29 OK** |
| v4 and v5 novelty checkers | pass, pass | **pass, pass** |
| **canned-reply control (13 strings)** | 0 passes | **0 passes** |
| `abstain_exact` on the 67 ill-posed rows | — | **unchanged** |

**The canned control still reports zero, run by the grader's own `RowCheck::passes` through
`chat-grade check constants=`.** The new component did not become a path by which a canned reply
passes: `reply_exact` requires required content *and* excludes the distractors, and the memorised
strings contain neither. The safety property measured in the first piece — that a `question`-kind
clarify check would be passed by the canned greeting on all 67 ill-posed rows — is untouched, because
those rows keep `abstain_exact`.

### First reading with discrimination, and the honest result: no verdict changed

| population | checked | `check_pass` |
|---|---:|---:|
| content rows, `any` | 16 | 1 |
| content rows, `reply_exact` | 5 | 1 |
| ill-posed rows, `abstain_exact` | 67 | 0 |
| **total** | **88** | **2** |

**The total is 2 of 88, exactly as before the addition, and no row changed verdict.** The one
`reply_exact` row that passes is `ask-06`, which names a horse and no wild animal; `heldout-043`
still never states 96, `heldout-103` still answers "1, 2, 3, 4, 5, 6, 7, 8, 9, 10." for the primes,
and `heldout-126` still does not write the program.

That is a finding, and it is the one to take from this piece: **on this artifact the model's wrong
answers are wrong by *omission*, not by naming a competing value.** The 64 short wrong answers of
Result 6 fail because the required content is absent, not because a wrong value is present — so a
discrimination component adds expressiveness without moving a single verdict here. It is worth
having for the instrument (a reply that says "honey, not vinegar" now fails a check that would have
passed it), but it does not by itself explain the failures, and it must not be reported as if it did.

**Criterion 1 remains NOT MET and 43/232 is unchanged.** This makes the instrument express
discrimination rather than only presence. It does not move the criterion.

## Next

Build the additive **clarify** kind for the 67 ill-posed rows — "the reply asks for the missing
material AND is not a canned reply", which the first piece measured as unsafe with the existing
grammar because a `question`-kind check is passed by the canned greeting on all 67 rows. `reply_exact`
now supplies the machinery it needs (required terms, forbidden distractors, keys, no recall
precondition); what it still cannot say is "asks a question", so the clarify kind needs a required
question form plus the same `forbid`/`keys` exclusions. Then restate criterion 1's reply half as a
deterministic sub-reading over the checked rows, reported per kind and per category with the failures
named, and **do not gate the open-ended 62.1 % on a judge.** Criterion 1 remains **NOT MET** and
43/232 is unchanged: this is a new instrument, not a met milestone.
