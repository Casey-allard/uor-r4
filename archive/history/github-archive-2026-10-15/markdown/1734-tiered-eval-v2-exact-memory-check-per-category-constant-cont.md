# #1734 — Tiered eval v2: exact memory check, per-category constant controls (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-05T11:15:17Z
- **Updated:** 2026-10-09T13:08:48Z
- **Closed:** 2026-10-05T14:13:32Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1734
- **Merged:** not merged
- **Merge commit:** d20e26c4741f7de96322b85b28a2582cad764fa6
- **Base / Head:** codex/tiered-eval / UOR-Foundation:codex/tiered-eval-v2
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #820. Stacked on #1724 (base `codex/tiered-eval` at `f3737c28`). **Do not merge** until #1724 lands and an exact-head review approves.

This PR answers the five faults that the #1724 review and the #1733 re-score found, using new files. `conversational-v2*` and the hand-labelled `heldout-*-ids.txt` are unchanged. They stay as the record of what #1733 scored. The new panel is named `conversational-v3`, because `conversational-v2` is already the panel that #1733 re-scored. No model replied to v3, and no model was trained, run or graded.

## Fault → change

| # | Fault | Change |
|---|---|---|
| 1 | The grader cannot fail a memory reply that recalls the wrong fact | New `exact` check on all 30 memory rows. The reply must contain the expected value (case and punctuation normalised) and none of the forbidden values. Forbidden values are the distractors stated in the conversation and, for a closed class (colours, days, the numbers 2–12), that class's other members. A wrong value, a hedge naming both values, or no value fails. Acceptable = grader fluent and relevant **and** exact. |
| 2 | Multi-turn rows that do not depend on the earlier turn | All memory rows were rewritten. Each one states the expected value and a same-type distractor in earlier turns, and the last turn names only the key. `check` refuses a row whose value or forbidden value is in the last turn, or that has no distractor. Only memory rows are multi-turn now: the v2 topic follow-ups were replaced by single-turn ambiguous requests. Echoing the last turn or the whole history passes 0/30. Copying the first-stated or the last-stated candidate passes 15/30 each, and both copy controls are reported. |
| 3 | No per-category constant control; "I'm not sure…" beat every model | Every report has `per_tier.<tier>.headline`. For each category it gives the best of the constants (most acceptable rows), model minus best constant, the discordant rows, an exact McNemar test and `model_beats_constant`. The top-level `headline` lists only the categories where the model beats its best constant. A tier total is never the headline. `compare` gives McNemar per category. Unknowable rows use `abstain_exact`: the reply needs an abstain phrase and must assert no specific. A digit, a capitalised mid-sentence word the user did not write, or one of the row's answer-class words fails it, so "I'm not sure, but I think it is blue." fails. |
| 4 | Rows match M-world training templates | No v3 turn matches an M-world string literal whole or as a template, and none shares **4** consecutive words with one. A unit test enforces this on every build (`panel_v3_shares_no_m_world_phrasing`), and `chat-grade leak strict=` reports it. The same strict check finds 5 leaking rows in v2 (unk-04, story-20, unk-22, talk-24, do-26). The full corpus leak check also finds none (ft-mixed-mworld-chatv0 and TinyDialogues train, 257k documents). |
| 5 | K-ill-posed depended on model outcomes | `chat-grade ill-posed` applies one text-only criterion to all 200 heldout rows: *the request refers to material its text does not contain*. It is code, unit-tested, and reads only request text. Result: 67 ill-posed and 133 clean, now the embedded default. |

## Panel v3: rows by category

| category | rows | multi-turn | check |
|---|---|---|---|
| multi_turn_memory | 30 | 30 | exact (30) |
| self_contained_instruction | 30 | 0 | any (6) |
| clarify_or_on_topic | 24 | 0 | question (24) |
| unknowable_or_impossible | 24 | 0 | abstain_exact (24) |
| smalltalk_feelings | 26 | 0 | – |
| story_continuation | 26 | 0 | – |

Check-only controls (no grader): constants score 0/30 on memory. Constant 1 passes 24/24 clarify and 24/24 unknowable checks, so those two categories are read only against their best constant. `chat-grade check`: pass, worst case 336/384 positions.

## Frozen sha256 (before any model reply)

```
578e256448b9790d3164d88aea12b9a72d015b915dd1492868dd160f8b50a163  conversational-v3.json
425ad695463a7533f08a4a4ab6780815fe6c062d01cf76dcdb74f6a45744a9b8  conversational-v3-a.json
78e23f2a9bba865340a4a58f9cdc68cd34a55ae3612c2fdb8239a0f335b4ee22  conversational-v3-b.json
0823e316da1e1e6d46b0bba6ed596828919344ce24c337d830c71ad4fca0b0f9  conversational-v3-checks.tsv
8ef79a1609ab89aec06d6efcdcbbda4335d5e041e9d133362f112da4abd788a5  heldout-ill-posed-v3.tsv
2417ce8ce569724e8a6a79c39993c171f23c52107b293abfd87af6acacab4d37  heldout-ill-posed-v3-ids.txt
1ce055d48429a0cb51ca6b3931180befd3729d51802d8a1cf0653d822939865b  heldout-clean-v3-ids.txt
```

These values are also in `data/panels/MANIFEST.sha256`. Byte-identical copies are in `~/uor-r4-local/ladder/panel/`, with `SHA256SUMS.txt` and `README.txt` appended.

## The headline applied to the existing #1733 reports (v2 panel, no regrading)

`chat-grade tiers` on the seven sealed tier C reports, using the v2 checks and the v2 K list. This only shows the new comparison on old reports; it is not a v3 result.

- Categories where the model beats its best constant: 29M and 100m-B on `self_contained_instruction` and `smalltalk_feelings`; 100m-C, Cx and C12 on `self_contained_instruction` only; L and L12 on none.
- Every model is below its best constant on `clarify_or_on_topic` (−9 to −13) and on `unknowable_or_impossible` (−14 to −15).

## Checks run at this head

- `cargo test -p uor-r4-training --bin chat-grade`: 14 passed. New tests: exact scorer, abstention / fabricated-specific check, best-constant headline, frozen v3 panel structure and check-only controls, M-world leak, missing-material rule.
- `cargo fmt --check`: chat-grade.rs is clean. `relation_compiler.rs` already has a formatting diff on the base branch, and this PR does not touch it.
- `cargo clippy -p uor-r4-training --bin chat-grade`: no findings in chat-grade.rs. `--tests` stops on 3 existing `approx_constant` errors in `b3_e8_codecs.rs`, which this PR does not touch.
- `chat-grade check` on v3: pass. `chat-grade leak` on v3 at strict 4 (M-world) and with references and corpora: 0 leaking rows. `chat-grade ill-posed expect=`: no difference. `scripts/check_claim_wording.py`: pass.

## Caveats

- The `exact` rule is strict. A correct reply that also names the distractor ("Shelby, not Flash") fails. A model that copies one stated value at random passes about 15/30, so memory needs to beat the copy controls as well as the constants.
- `abstain_exact` uses lexical heuristics. A reply that suggests rather than asserts can fail ("maybe check your pocket" on rows with an answer-class list). A made-up name written in lowercase passes.
- Constant 1 still passes every clarify and unknowable check. On those categories a model can only beat its best constant on rows where the grader rejects the constant (constant 1 was acceptable on 18/24 and 15/24 in #1733).
- The missing-material rule labels 21 v2 "fragment" rows clean, because they refer to nothing missing: greetings without a trailing comma, headlines, list items. It also misses two v2 "absent" rows (heldout-112, heldout-137). K-clean therefore still holds some rows that are hard to answer. Its cue lists were written while looking at these 200 rows.
- The embedded K default changed from the 90-id hand list to the 67-id rule list. Re-tiering an old report without `ill_posed=` moves rows between K-clean and K-ill-posed.
- One author, no second annotator. No model has been graded on v3, so this PR makes no claim about any model's memory or conversation ability.

Cost: about 1 h 40 min of preparation, mostly writing code and panel rows. Compute was a dev build (2 min 35 s at -j 2), tests, two leak runs (15 s for the corpus run) and the rule run, all CPU with 2 threads, plus about 300 KiB of new storage (sealed leak roots under `~/uor-r4-local/ladder/panel-checks/`). No GPU, no ollama and no external compute. The job lock `claude-tiered-eval-v3` was registered and has been removed.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (4 entries; chronological: conversation comments, reviews, review comments)

### comment 5993476081 — 2026-10-05T11:25:28Z — Casey-allard

Adversarial review at 1b0390fcf3ad10352008e024bdae0cd01462341b:

**Verdict: changes requested.** The panel structure is sound, but the abstention check and the exact memory check both accept replies they are meant to reject. Each case below was reproduced with the PR's own `chat-grade check`, run from a `git archive` copy of this head and using `constants=<file>` to feed it adversarial replies.

**What I ran**
- `cargo test -p uor-r4-training --bin chat-grade` on a `git archive` copy in `$TMPDIR`: 14 passed, 0 failed.
- `chat-grade check requests=conversational-v3-a.json,conversational-v3-b.json constants=<adversarial file>` for the check-only pass counts below. These are check-only results; no grader and no model ran.

**Blocking**

1. **`abstain_exact` accepts replies that do not abstain.** `ABSTAIN_PHRASES` matches `can't`, `cant` and `cannot` anywhere in the reply.
   - `Yes! I can't wait to play with you!` passes **24/24** unknowable rows.
   - That count includes the impossible-action rows (unk-07 "come to my house and play", unk-08, 10, 12, 15, 17, 20, 23). On those rows the reply agrees to do the impossible thing and is still scored as an abstention.
   - The judge's relevance question ("responds sensibly to the user's last message") would plausibly say yes, so this reply would also count as *acceptable*.
   - "can't wait" is a very common TinyStories phrase, so a model trained on that corpus can pass by accident.
   - Fix: reject an affirmative opener (yes, sure, ok, of course) and the idioms "can't wait" and "cannot wait", or require the abstain phrase to come before any other clause. Add this reply as a check-only control.

2. **`fabricated_specifics` misses two kinds of made-up specific.**
   - (a) A capitalised name at the start of a sentence is exempt. `I'm not sure. Sam is the one.` passes **24/24**.
   - (b) A made-up common noun in lowercase passes on the 18 of 24 unknowable rows whose forbidden list is `-`. `I don't know, but they are under the bed.` passes **24/24**, which includes unk-03 (keys), unk-16 (backpack), unk-24 (who is behind me) and unk-02, 09 and 22.
   - The PR caveat names only "a made-up name in lowercase". The real gap is wider: a sentence-initial proper noun, and any lowercase specific on a row with no answer-class list.
   - Fix: check every capitalised token that is not "I" or "OK" and that the user did not write, wherever it sits in the sentence, apart from the reply's first word. Give every unknowable row an answer-class forbidden list, or treat "maybe X" and "X is" after an abstain phrase as an assertion.

3. **`exact` accepts a wrong binding.** The forbidden lists hold only the distractor *value*, never the distractor *key*. So a reply that attaches the correct value to the wrong entity passes. Each of these passes its row:
   - `The goldfish is Shelby.` (mem-01)
   - `Your dad drinks coffee.` (mem-25)
   - `Your uncle lives near the mountains.` (mem-18)
   - `Ella is playing the dragon.` (mem-03)
   - `You planted beans on the left side.` (mem-17)
   - `The red train goes to the city.` (mem-27; this reply passes 2 rows)
   - `Your scooter has a bell.` (mem-09)
   - `Kofi brought the kite.` (mem-10)
   - `Zara sits by the door.` (mem-19)
   - `The silver robot is Zip.` (mem-28)

   That is 10 for 10. The judge sees only "responds sensibly to the last message", so it will not catch these either. Memory is the headline category, and the PR asks whether a wrong-fact memory reply can pass: it can.
   - Fix: add the distractor key words to `forbid` (for example `dad` on mem-25, `mountains` on mem-18, `goldfish` on mem-01). That matches the existing rule that a reply naming the distractor fails. Also add a binding-swap control next to the copy-first and copy-last controls.

**Not blocking. Please disclose these or add a control.**
- Generic replies pass some memory checks. A TinyStories-style story (`Once upon a time, a girl with red boots and her uncle rang a bell. She ran to the lake.`) passes **5/30** exact memory checks, and a play line (`Let's run outside and play in the city by the lake!`) passes **3/30**. The reported "constants 0/30" holds only for the three default constants. Short common target words (red, run, lake, city, bell, boots, uncle, three, nine) give an unconditioned story a non-zero pass rate. Add a generic-story control.
- `do-26` accepts `great`, so `That sounds great!` passes. `do-03` (six) and `do-15` (three) have no forbidden list, so `Maybe 3 or 6.` passes both.
- `question` remains a check for a question mark only. This is already disclosed.

**Confirmed clean**
- All 30 memory rows are `recall`, each expected term is absent from the last turn, and echoing the last turn or the whole history passes 0/30.
- Copying the first-stated value passes 15/30 and copying the last-stated value passes 15/30, so the expected answer is balanced between first and last position.
- No single-turn row depends on history.
- The M-world leak test (`panel_v3_shares_no_m_world_phrasing`) passes at this head.
- `missing_material` and `ill-posed` read only the request text: there is no reply, grade or model input. The ill-posed split is text-only, with the cue-list caveat the PR already states.

Not merged. Review only.


### comment 5993768888 — 2026-10-05T11:45:15Z — Casey-allard

Review fixes pushed: head `b04baa473c406d96038f703ddbb3bc09610a37b6` (on top of `1b0390fc`).

**1. `abstain_exact` accepted non-abstentions ("Yes! I can't wait to play with you!" passed 24/24).**
- An abstain phrase followed by "wait" no longer counts.
- A reply with an agreement phrase fails: yes / yeah / of course / let's / here you go / no problem / "nothing is impossible" / ..., or "sure" unless the word before it is "not".
- Each impossible-action row (unk-07/08/10/12/15/17/20/23) now also has a list of words that perform or report the action ("on my way", "here's a hug", "it's open", "delicious", "beautiful", "poof", "turned off", "blast off").
- The review reply now passes 0/24.

**2. `fabricated_specifics` missed two kinds of made-up specific.**
- (a) A capitalised word at the start of a sentence now counts as a specific unless it is on a frozen opener list (pronouns, determiners, question words, auxiliaries, interjections, Sorry, Maybe, ...) or the user wrote it. "I'm not sure. Sam is the one." now passes 0/24.
- (b) All 24 unknowable rows now have a forbidden list; the 16 knowledge rows list their answer class (key places, pets, dream topics, backpack contents, song titles, who could be behind you, ...).
- Two structural rules also apply on every row: a guess marker (maybe, probably, I think, might be, ...) fails unless its sentence is a question or hands the question back ("tell me", "ask", "let me know", "show me"); and the clause after "but" must abstain again, hand the question back or be a question. "I don't know, but they are under the bed." now passes 0/24.
- Remaining limit, documented and pinned by a test: a plain assertion outside every pattern and outside the row's list still passes ("I don't know. It is the blue one." on unk-02).

**3. `exact` accepted a wrong binding.**
- The checks file has a new 6th column, `keys`: the distractor's key words, possessives included, on 26 of the 30 memory rows. A reply naming one fails.
- The other four rows have no key column. Three are updates (beach then lake, white then blue, Pebble then Biscuit), where the distractor is the superseded value of the same key. On the rule row mem-07, "allowed" also appears in a correct answer. On these four rows the swap is the other value, which `forbid` already fails.
- `chat-grade check` refuses a key in the last turn, or a key list with no key in an earlier turn.
- New control: `binding_swap` applies one authored wrong-binding reply per memory row (`data/panels/conversational-v3-swaps.tsv`). It passes 0/30. Your five examples are asserted row by row, and plain correct replies still pass.
- The copy controls still copy values only and are unchanged (15/15).

**Controls (`chat-grade check`, no model, ladder tokenizer, context 384).** The panel check passes; the worst case is still 336 positions (mem-16).
- Memory: constants 0/0/0, echo 0, copy 15/15, binding swap 0/30.
- Unknowable: constant 1 still passes 24/24, constants 2 and 3 pass 0, echo 0. Each of the three review replies (now frozen as `adversarial_abstentions` in every check report) passes 0/24.
- Clarify: unchanged.

**Re-freeze.**
- The panel rows (`conversational-v3*.json`) are unchanged.
- `conversational-v3-checks.tsv` was revised in place, because no model has replied to or been graded on v3, so no report is bound to the old file. Old sha256 `0823e316...` (recorded in the README with the reason), new `be82781146e500fe948e7a5a3062d09212f1d4c524c3537c1850faf1657c30c3`. New file `conversational-v3-swaps.tsv`: `30b521f1c34fdb91512753a02e6163d153e2a34fd1212f754b9153a6b0082714`. MANIFEST is updated.
- The v2 checks use only `any`, `abstain` and `question`, and `abstain` is unchanged, so the #1733 re-tiered headline is unaffected.
- Local copy in `~/uor-r4-local/ladder/panel/`: the old checks file is kept as `conversational-v3-checks-r1-superseded.tsv`, the new files are copied there, and SHA256SUMS.txt and README.txt have new lines appended.

**Checks run at this head:**
- `cargo test -p uor-r4-training --bin chat-grade`: 14 passed (the existing tests are extended; none added).
- rustfmt: chat-grade.rs is clean.
- clippy (`--tests -A clippy::approx_constant`, to get past the lint errors already in `b3_e8_codecs.rs`): chat-grade.rs has only the existing `type_complexity` warning.
- `chat-grade check`: pass.
- `check_claim_wording.py`: pass.

The rule is still lexical and has one author. No model has been graded on v3. Not merged.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


### comment 5993880884 — 2026-10-05T11:52:59Z — Casey-allard

**Re-review at exact head `b04baa473c406d96038f703ddbb3bc09610a37b6`: APPROVE (no blocking findings)**

I checked the three gaps from the earlier review against this head.

**Checks run at this head** (isolated worktree, shared target dir, `-j 2`):
- `cargo test -p uor-r4-training --bin chat-grade`: 14 passed.
- `rustfmt --check chat-grade.rs`: clean.
- `chat-grade check` on v3 (ladder tokenizer `claude-t4-1433-resume/bundle-learned-1`, context 384): passes. The check-only controls match the README table:
  - memory: binding_swap 0/30, copy first/last 15/15, echoes 0, constants 0.
  - unknowable: adversarial abstentions 0, 0 and 0 of 24; constant 1 passes 24/24.
  - clarify: constant 1 passes 24/24, echo 17/24.
- `shasum -c data/panels/MANIFEST.sha256`: all OK. The checks file is `be827811...` and the swaps file is `30b521f1...`, as stated. The panel JSONs did not change in this commit. The local `~/uor-r4-local/ladder/panel/` checks file is byte-identical to the new one.
- `scripts/check_claim_wording.py`: pass.
- I did not rerun clippy. The author reported that clippy only reaches this file with `-A clippy::approx_constant`, because of base-branch errors.

**Fixes verified:**
1. **abstain_exact agreement.** "can't wait" no longer counts as an abstention. Agreement phrases fail, and so does "sure" unless "not" comes right before it. The 8 impossible-action rows now forbid the phrases that perform or report the action.
2. **Fabricated specifics.** A capitalised word that starts a sentence now counts as a name unless it is on the opener list. All 24 unknowable rows have forbidden lists. Two rules are new:
   - a guess marker fails unless its sentence is a question or hands the question back to the user;
   - the clause after "but" must abstain again, hand the question back or be a question.
3. **Exact wrong binding.** The new `keys` column is validated: a key may not be in the last turn, and at least one key must appear in an earlier turn. The 5 review replies fail row by row. Plain correct replies pass. Every memory row has a swap reply, and the unit test asserts this.

**Non-blocking observations.** I probed with a temporary test that I removed afterwards. All of these make the check stricter, so they cannot inflate a model's score:
- "I don't know. Are you sure someone is there?" fails as `agrees`, because "sure" inside a question counts as agreement. Excluding "sure" in a question sentence would be a small refinement.
- "Turn around and look!" and "Mom might know?" fail as made-up names. This is the documented opener-list limit. "Mom" is a common sentence opener for this audience.
- "I cannot see you, but I hope you have fun!" fails as `asserts after but`. This is documented.
- "I'm not really sure" and "I'm not quite sure" are not abstain phrases. This predates this PR and also affects plain `abstain`.
- On the permissive side, "I don't know. They are under the bed." passes the generic rule. This is the documented lexical limit; on unk-03 the row's forbidden list catches it.

The documented limits and the in-place refreeze record (old sha `0823e316...`, made before any model replied) are adequate. The v2 headline is unaffected, because v2 uses only any/abstain/question.

I did not merge.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


### comment 5996253410 — 2026-10-05T14:13:31Z — Casey-allard

Included in #1740 (merged as 0f443ab3): this PR's head is an ancestor of the #1740 branch that was squash-merged into main, so its full content is on main. Closing as included rather than resolving conflicts, which would leave an empty diff. References #820


---
_Comment/review ids in this file: 5993476081, 5993768888, 5993880884, 5996253410_

