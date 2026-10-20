# #1724 — Tiered chat evaluation: conversational panel, clean open panel, per-tier scoring (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-05T06:37:24Z
- **Updated:** 2026-10-09T13:08:47Z
- **Closed:** 2026-10-05T14:13:29Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1724
- **Merged:** not merged
- **Base / Head:** main / UOR-Foundation:codex/tiered-eval
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #820. Follows Step 0a (#1708). Do not merge without an exact-head review.

## Why

Step 0a found that the 232-request open panel is mostly an open-domain knowledge test. Knowledge-absent was the largest failure class (35% of 96M failures), recall was 2.2%, and only 8 of 232 rows are multi-turn. About a quarter of the failing heldout rows are ill-posed: they refer to material that is not in the request. A 30M–100M model trained on TinyStories plus chat cannot answer open-domain knowledge requests, so on this panel a conversational or mechanism change barely moves the score. This PR adds evaluation tiers sized to what a model of this scale should be able to do. The original panel files are unchanged.

## What

**Tier C: `data/panels/conversational-160.json`** (split into `-a`/`-b` because a `chat-grade` panel file holds at most 128 requests)
- 160 new requests that need no world knowledge, in the existing request format (`id`, `category`, `user_turns`). Ids are `conv-*`.
- 36 rows are multi-turn (2–4 user turns). The model answers each earlier turn with its own greedy reply.
- Rows interleave round-robin by category, so a row's derangement-control neighbour always comes from a different category.

| category | rows | multi-turn |
|---|---|---|
| smalltalk_feelings | 26 | 0 |
| multi_turn_memory | 30 | 30 |
| self_contained_instruction | 30 | 0 |
| clarify_or_on_topic | 24 | 3 |
| story_continuation | 26 | 2 |
| unknowable_or_impossible | 24 | 1 |

- **Freeze:** the panel was authored before any model replied to these requests. Only the no-model `chat-grade check` has read them. Hashes are in `data/panels/MANIFEST.sha256`.
- `conversational-160-keys.tsv` records the expected recalled content for the memory rows, for a later exact-recall check; the grader does not read it.

**Tier K: `heldout-ill-posed-ids.txt` (71) and `heldout-clean-ids.txt` (129)**
- 55 ill-posed ids come from the 0a `ill=1` labels: the union of the 29M and 96M labels, which never conflict.
- 16 more come from a request-text-only screen of rows 0a never labelled, because both models were accepted on them. Examples are salutations, `[placeholder]` templates, and "Given the text:" with no text.
- `heldout-ill-posed.tsv` gives the source of every id.

**chat-grade changes**
- `grade` and `grade-replies` reports keep every existing field and add two:
  - `per_tier`: for tiers C, K-clean, K-ill-posed, everyday and stretch, the actual tally, the derangement control and the paired exact McNemar against the control, with per-category tallies inside each tier.
  - `tiers`: the tier rule, plus the source and sha256 of the ill-posed list. The list is embedded in the executable by default; `ill_posed=` overrides it.
- `exclude=IDS.txt` on `grade` and `reply` drops those requests before any reply is generated.
- New subcommands:
  - `check`: runs the turn and context check against a tokenizer, with no model.
  - `filter`: writes a new panel file and never edits its inputs.
  - `tiers`: re-tiers an existing report without regrading.
  - `compare a= b= [tier=]`: pairs two graded reports by id within each tier and gives exact McNemar results for acceptable, fluent and relevant. Rows present in only one report are counted under `unpaired` and left out of the test. It warns if the two reports used different graders.
- `compare` and `tiers` with `out=` claim a new report root and seal it.

## Checks at this head

- `cargo test --offline -p uor-r4-training --bin chat-grade`: 4 passed. Three are new unit tests, covering the id-list filter and tiers, per-tier/control aggregation (including an unparsed or missing verdict), and compare pairing (McNemar, unpaired rows, turn mismatch, repeated id).
- `cargo fmt --check -p uor-r4-training` with this PR's files: clean. One caveat: `relation_compiler.rs` on main is not rustfmt-clean, and that file is not touched here.
- The chat-grade binary builds with no new warnings.
- `chat-grade check` with the ladder tokenizer, context 384, 64 new ids:
  - Tier C passes. The worst-case history is 323/384 positions, on `conv-mem-28` (4 turns).
  - The open panel with `exclude=` passes with 161 requests.
- `chat-grade tiers` on the existing 96M-C 7b report gives:

| tier | rows | acceptable | derangement control | McNemar p vs control |
|---|---|---|---|---|
| K-clean | 129 | 14 | 0 | 1e-4 |
| K-ill-posed | 71 | 24 | 4 | below 1e-4 |
| everyday | 32 | 8 | 0 | 0.0078 |

- `chat-grade compare` of 96M-C against 29M-B lr5e-4, acceptable rows (paired McNemar):

| tier | 96M-C | 29M-B | p |
|---|---|---|---|
| K-clean | 14/129 | 12/129 | 0.82 |
| K-ill-posed | 24/71 | 22/71 | 0.79 |
| everyday | 8/32 | 9/32 | 1.0 |

No model has replied to tier C yet.

## Caveats

- **The grader judges fluency and relevance only.** A memory row whose reply is fluent but recalls the wrong fact can still pass. The keys sidecar is there for a later exact check.
- **Ill-posed rows score higher than clean rows.** On 96M-C, 24/71 ill-posed rows are acceptable against 14/129 clean ones, because the grader accepts generic replies to salutations. Removing them lowers the open-panel score. The M4 target (≥ 60/232) needs restating per tier before it is read.
- **One author.** The 16 screen ids and the whole tier C panel are a single author's judgement (Claude), with no second annotator.
- **Control neighbours can cross tiers.** The per-tier derangement control is the grade of each row's own reply placed against the next row's conversation in the loaded panel, so that next row can belong to another tier.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (4 entries; chronological: conversation comments, reviews, review comments)

### comment 5989511894 — 2026-10-05T06:45:53Z — Casey-allard

Adversarial review at 3564f9c191299ee30988d23104473cf533e090b8:

Verdict: **request changes.** The tooling is correct as code. The checks I ran are below. Tier C does not yet measure the things it is named for, and the K-tier split depends on the outcomes of the two models it is used to compare.

What I ran at this head: `cargo test --offline -j 2 -p uor-r4-training --bin chat-grade` gives 4 passed. `shasum -c MANIFEST.sha256` is OK for all 7 files. I reproduced the ill-posed set against `docs/research/step0/0a-panel-taxonomy/labels-{29m,96m-pass1}.tsv` (column `ill`) and checked leakage with a template regex over every string literal in `crates/uor-r4-training/src/*.rs`. No model or grader was run.

## Blocking

1. **(b) The grader cannot fail a reply that ignores the earlier turn.** `RELEVANT_QUESTION` asks "Does the last Assistant reply respond sensibly to the user's **last message**?", and the examples are single-turn. On conv-mem-01 ("My name is Lily." / "What is my name?"), the reply "Your name is Tom." is fluent and responds sensibly to the last message, so it passes. The same holds for all 30 `multi_turn_memory` rows. The same gap affects `unknowable_or_impossible`: an invented answer ("You ate pancakes.") is just as "sensible" as "I don't know". `conversational-160-keys.tsv` is not read. As it stands, tier C's memory and abstention categories score fluent, on-topic replies, not memory or abstention. Requested fix: before any model replies to tier C, either add a third judged question for these categories that sees the whole history and the key/abstention expectation, or add a deterministic key check from `keys.tsv` (exact or normalized containment). Then freeze that rule.

2. **(b) Some multi-turn rows are still not turn-dependent even with a correct grader.** A reply that ignores turn 1 would still be correct on these: conv-mem-08 (the last turn names the dragon itself), conv-mem-13 (any cat fact), conv-mem-16 ("maybe you are tired"), conv-mem-19 (any drawing idea), conv-mem-25 (any cheering line), conv-mem-03 (any green object, if the reply guesses), clar-17, clar-18, clar-23, story-23, story-24 and unk-21. conv-mem-24 has a key that depends on the model's own first reply, so it cannot be checked offline. About 23 of the 36 multi-turn rows are genuinely dependent. Report a dependent-subset count separately, or rewrite these rows into a new panel name.

3. **(c) No control catches a generic canned reply.** The derangement control uses the neighbouring row, which is always from a different category because of the round-robin order. It therefore catches off-topic replies but never a generic one. Under the current relevance-only grader I estimate a single canned reply would pass most of these:
   - all 24 `clarify_or_on_topic` rows with "Could you tell me more about what you mean?", which is the intended answer;
   - most of the 24 `unknowable_or_impossible` rows with "I'm not sure, I can't know that.";
   - most of the 26 `smalltalk_feelings` rows with "That sounds nice! Tell me more."

   That puts a constant-reply model at roughly 50–70 of 160 (30–45%), and each canned line would also beat the derangement control within its own category. Add a constant-reply control for each category (one fixed generic reply graded on every row) to the tier C report, and read tier C as a margin over it.

4. **(d) Leakage into M-world v1 social intents, which are trained into the `ft-mixed-mworld-chatv0` arms.** In `milestone_world.rs` the `clarify` intent has train phrasings "Can you help me with it?" (an exact match to **conv-clar-01**), "Can you fix it?" (≈ conv-clar-05), "Tell me more about it." (≈ conv-clar-21) and "Is it good?" (≈ conv-clar-19). It also has canned replies ("Could you tell me more about what you mean?"). The `greet`, `how_are_you`, `thanks` and `farewell` intents closely match conv-talk-02/03/05/09/19/22 ("Hi there.", "Good evening!", "Good night!", "I have to go now. Bye!" / "Bye! I have to go eat dinner now."). `animal_sound` (cow → moo) makes conv-mem-29 a reversed training fact. Other exact template matches:
   - conv-mem-01: "My name is {v}." / "What is my name?"
   - conv-mem-02: "What is my dog's name?"
   - conv-mem-10: "My best friend is {v}." (`milestone_world_v2.rs`)
   - conv-mem-14: "I live in {v}."

   Matches to the frozen `stack_aerm.rs` dev-mem panel strings: conv-mem-03 ("My favorite color is green.") and conv-mem-15 ("I am learning to play the piano.").

   Leakage against chat-v0 (smoltalk, ultrachat, everyday-conversations) is **UNAVAILABLE**. The raw jsonl lives on the unmounted SSD and only `tokens.u16` is local. Please either report tier C scores separately for M-world-touched rows, or replace those rows in a renamed panel.

5. **(e) The ill-posed set uses two criteria, chosen by model outcome.** The 0a part matches exactly: 55/55 ids, with no source mismatch, no 29M/96M conflict, and the 71 ill-posed + 129 clean ids partitioning all 200. The text screen, however, was applied only to rows that **both** models passed, while failing rows keep 0a's `ill=0` verdict. That produces direct inconsistencies:
   - heldout-117 (a `[forbidden_words]` template with no task) is **clean**, while its twins heldout-002 and heldout-144 are ill-posed.
   - heldout-128 ("Given the text: ..." with no question) is **clean**, while heldout-030, identical in form, is ill-posed.

   Tier membership is therefore a function of 29M/96M outcomes. All 16 screen rows were passed by both models by construction. That inflates K-ill-posed acceptance (the "24/71 vs 14/129" caveat is largely this artifact) and makes the 96M-C vs 29M-B K-tier compare circular. Fix: apply one text-only criterion to all 200 rows, blind to replies and outcomes, and keep 0a's labels only as an agreement check. If that is not done, mark the K split as invalid for any model whose failures fed the 0a labels.

## Checked and correct

- **(f)** `mcnemar_exact` is the correct two-sided exact binomial (2·P[X≤min(b,c)], capped at 1). Hand-checking (10,2) gives 0.0386 and (6,2) gives 0.289, both matching the tests.
- `tier_summary` pairs actual with control per row. `compare_rows` pairs by id, refuses rows whose user turns or category differ, refuses repeated ids, and counts one-sided rows as unpaired. The tests cover these cases.
- `tier_of` matches the documented rule.

## Nits

- `tier_of` sends every unknown category to `everyday`, so a typo or a new panel silently lands in that tier. Make unknown categories an error.
- `write_sealed` seals the root even when `fs::write` failed. Write first, return on error, then seal.
- The module doc (line 24, from before this PR) still says the grader scores "from 1 to 5"; it gives yes/no.
- Per-tier `per_category` tallies only actual replies, with no control per category, so a category with a control-level pass rate cannot be seen.
- (a) No row needs encyclopedic knowledge. The rows that need everyday lexical or common-sense knowledge or simple arithmetic are fine for this model scale but should be named: conv-do-02, do-08, do-14, do-15, do-19, do-21, do-23, do-24, do-26, do-30, mem-03, mem-09, mem-22 and mem-29. mem-29 is the one memory row whose answer needs world knowledge (moo → cow).
- (g) Freeze evidence is the single commit 3564f9c1 (2026-10-05 02:36 -0400) with `MANIFEST.sha256`. I found no `conv-*` rows in `~/uor-r4-local/ladder/{grades,evals}`, which is consistent with no use yet. The freeze statement is self-attested by the author. Post the hashes to #820 before the first reply run, and make every fix above under a new panel name, as the README requires.

Not merged; no changes made by the reviewer.


### comment 5990160334 — 2026-10-05T07:37:06Z — Casey-allard

Review fixes pushed: head `f3737c2830b8b06cd87d50144299528e79d088dd` (one commit on top of 3564f9c1). This answers the [review](https://github.com/UOR-Foundation/uor-r4/pull/1724#issuecomment-5989511894); the PR stays unmerged.

## Blocking items

**(b) The grader ignored history.** `chat-grade` now applies frozen per-row checks from `data/panels/conversational-v2-checks.tsv`. The file is embedded in the binary, and `checks=` overrides it. There are three kinds:
- `any`: the reply contains one of the listed words or phrases.
- `abstain`: the reply contains a fixed phrase such as "don't know", "can't" or "not sure".
- `question`: the reply contains `?`.

A checked row is acceptable only if it is fluent, relevant **and** passes its check, so "Your name is Tom." now fails a recall row. Reports also keep `fluent_and_relevant`, the old count. 84 of the 160 rows are checked:
- all 30 memory rows
- all 24 unknowable rows
- all 24 clarify rows
- the 6 instruction rows that have one determinate answer not stated in the turn

The keys sidecar is gone; the checks replace it.

**(b) Multi-turn rows that did not depend on the earlier turn.**
- mem-03/08/13/16/19/24/25 were rewritten so the answer is a stated fact. mem-24 no longer depends on the model's own story.
- clar-17/18/23 now ask a follow-up that names nothing without the first turn. Their checks are topic words that are not in the last turn.
- story-23/24 and unk-21 are now single-turn.
- There are now 33 multi-turn rows: 27 recall, 3 derived, 3 topic.
- `chat-grade check` refuses a panel in which a multi-turn row has no history-dependent check, a check term appears in the last user turn (an echo would pass), or a recall term appears in no earlier turn.

**(c) No constant-reply control.** Every row is also graded with three fixed replies in place of the model's (`constants=none|FILE` overrides them):
1. "I'm not sure. Can you tell me more about what you mean?"
2. "That sounds nice! Thank you for telling me."
3. A TinyStories opening.

Reports give the paired McNemar of the actual replies against each constant, per tier and per category, and the per-category derangement tallies (nit). There are also check-only echo controls, which need no grader. The checks do **not** separate a model from constant 1 on clarify (21/24) or unknowable (24/24), because a generic clarifying "I'm not sure" is a right answer there. The README says those categories, and the unchecked smalltalk and story rows, are read only through the paired test against the best constant. An echo of the history passes 27/30 memory checks, so a memory pass also needs the grader's verdict on a reply that is not a copy.

**(d) Leakage.** New `chat-grade leak` compares every user turn with three sources:
- Reference texts: string literals of `milestone_world.rs`, `milestone_world_v2.rs`, `stack_aerm.rs` (dev-mem), `stack_dialogue.rs` and `answer_oracle.rs`, plus the paraphrase-review TSV. A turn counts as matching if it equals a text, fits a template with at least 3 literal words, or shares 6 or more consecutive words with a text.
- Whole-turn matches only: everyday-32, stretch-32 and the chat-v0 dev panel.
- The **decoded token stores** of `ft-mixed-mworld-chatv0` (M-world paraphrases plus chat-v0 train, 147,359 documents) and TinyDialogues train (110,024 documents). This is the check that was UNAVAILABLE in the review.

v1 has 15 leaking rows by this rule. Besides those, every row you named as an intent match (greet, how_are_you, thanks, farewell, clarify) was rewritten. **v2 has 0 leaking rows.** Four story/smalltalk rows share 8-word TinyStories phrasing with corpus lines ("once upon a time there was a little bunny"). Those are reported but not counted. Not checked: TinyStories train, and the sealed §8 panel (never read).

**(e) The ill-posed split depended on model outcomes.** Tier K was re-labelled from the request text alone, with one written criterion applied to all 200 rows (`absent` / `template` / `constraints_only` / `fragment`; definitions in the README).
- Result: **90 ill-posed, 110 clean**, with no model outcome used.
- As a cross-check only: all 55 0a ids and all 16 v1 screen ids fall under the rule, and 19 rows are new.
- The twins now share a tier: 002/117/144 are ill-posed, and so are 030/128.
- New sha256 values are in `data/panels/MANIFEST.sha256`.

Re-tiering the existing reports with the new split (`tiers` and `compare`; no regrading):

| tier | 96M-C acceptable | derangement | p vs control | 29M-B lr5e-4 | p 96M-C vs 29M-B |
|---|---|---|---|---|---|
| K-clean | 12/110 | 0 | 4.9e-4 | 10/110 | 0.80 |
| K-ill-posed | 26/90 | 4 | 3e-6 | 24/90 | 0.81 |
| everyday | 8/32 | 0 | 0.0078 | 9/32 | 1.0 |

## Nits
- `tier_of` returns an error on an unknown category, and on a `conv-*` id outside the six tier C categories.
- `write_sealed` writes first and returns before sealing if the write fails. There is a test for this.
- The module doc now describes yes/no verdicts, not a 1–5 score.
- On (a): mem-29 (moo → cow) is gone. The remaining instruction rows that need everyday lexical knowledge are unchanged.
- On (g): the hashes go to #820 now, before any reply run.

## New panel name
v1 (`conversational-160*`) is withdrawn and removed from `data/panels`. It stays in git at 3564f9c1. The local copies remain in `~/uor-r4-local/ladder/panel/`, with a WITHDRAWN note in `README.txt`, and the v1 heldout split files there were renamed `v1-withdrawn-*`. Tier C is now `conversational-v2(-a,-b).json` plus `conversational-v2-checks.tsv`. It keeps v1's ids, categories and order. No model has replied to v1 or v2. Only `check` and `leak` have read them, and neither uses a model.

## Checks run at f3737c28
- `cargo test --release -p uor-r4-training --bin chat-grade`: 8 passed. 5 tests are new or extended: checks parse/validate/judge, checks and constants in the summary, unknown-category errors, leak matching, and `write_sealed`.
- `cargo fmt --check`: chat-grade.rs is clean. `relation_compiler.rs` on main is still not rustfmt-clean and is untouched here.
- The chat-grade build adds no new warnings.
- `chat-grade check`, tier C v2, context 384: pass. Worst case is 323/384 positions, on conv-mem-28. The embedded checks have sha256 5589462f…
- The open panel with `exclude=` the new ill-posed list: pass, 142 requests.
- `chat-grade leak` on v2: 0 leaking rows. On v1: 15.

Not done: no model has replied to tier C, nothing has been graded with the new controls, and no ollama job ran. The panel and the tier K labels still have one author (Claude) and no second annotator.


### comment 5992819558 — 2026-10-05T10:39:39Z — Casey-allard

**For Codex (owner request): tier C needs another fix before it can serve as a headline score.**

I re-scored all seven chat models at head `f3737c28`, with the same settings as the existing `powered-7b` reports. Results and sealed roots are in #1733.

- **The constant control beats the models.** The constant "I'm not sure. Can you tell me more about what you mean?" is acceptable on **42/160** tier C rows. The models score **25–35/160**. Five of the seven are significantly *below* the constant (paired exact McNemar p = 0.014–0.049), and none is above it.
- **Where the constant scores:** clarify 18/24 and unknowable 15/24. The models score 0–1/24 on unknowable.
- **What the controls do show:** every model beats the derangement control and constants 2 and 3, so the replies are about the request.
- **Memory:** 0–3/30 for every model.

Requested fix, before tier C totals are used:
1. Report tier C per category against the best constant for each category, instead of one total.
2. Or rescore clarify and unknowable so that one fixed abstention cannot pass both kinds of row: abstaining when the answer is unknowable, and answering or staying on topic when it is answerable.

The README already says these categories must be read against the constant. This run shows that a reader of the tier C total would otherwise be misled.


### comment 5996252414 — 2026-10-05T14:13:28Z — Casey-allard

Included in #1740 (merged as 0f443ab3): this PR's head is an ancestor of the #1740 branch that was squash-merged into main, so its full content is on main. Closing as included rather than resolving conflicts, which would leave an empty diff. References #820


---
_Comment/review ids in this file: 5989511894, 5990160334, 5992819558, 5996252414_

