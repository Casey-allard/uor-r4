# #1503 — M-world v1: a typed-intent world for the milestone's three categories, with a frozen oracle

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-29T10:48:57Z
- **Updated:** 2026-10-09T13:08:56Z
- **Closed:** 2026-09-29T20:12:39Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1503
- **Merged:** 2026-09-29T20:12:39Z
- **Merge commit:** 4bed6f6c00648212bc2861329ce1883b02a30759
- **Base / Head:** main / UOR-Foundation:lab/claude/m-world
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #973, #1508. **Class A (Lab 1): a training world and evaluator.** R1's instrument (the plan of record, #1507).

## What it is

**`milestone_world.rs`** renders conversations from typed intents in the §8 milestone's three categories: responsive chat, a closed knowledge base, ten simple-instruction types, and in-context relations with updates and abstentions.
- Each intent fixes the reply it trains on and a frozen oracle.
- User phrasings and relation values are split between training and development.
- The qualification panel is authored separately.

**`examples/m-world.rs`** has three modes:
- `corpus`: chat-v0 unchanged plus M-world, excluding every development-panel turn, self-validated through `DialogueSplit::load` and `EpisodeIndex`;
- `evaluate`: greedy replies with the model's own history, judged per category and intent. The M-world report is written before the panel;
- `rejudge`: re-scores saved replies after checking every user turn.

## R1 used this world, and what that taught us

- **R1 corpus:** `claude-r1/corpus-3`, manifest `0ad2342b…`, 15,932 M-world responses.
- **Scores:**
  - development: Responsive 0.52, Instruction 0.27, Relation 0.01;
  - trained phrasings: 0.93, 0.65, 0.59.
- **The value pools were closed** (about 10 per relation), so R1 memorized values instead of copying them from context. This v1 is kept as R1's sealed instrument.
- **M-world v2** (#1508, A1) opens the pools and adds MQAR and copy curricula.
- **The re-review's 4 oracle regressions** are recorded as known limits of R1's scoring and fixed in v2: https://github.com/UOR-Foundation/uor-r4/issues/1516.

## Checks

- `cargo test -p uor-r4-training --lib milestone_world`: **7 passed.** They cover:
  - the oracle's semantics and the review cases;
  - a filler reply failing every intent;
  - every training reply passing its own checks;
  - split disjointness, including instruction phrasings;
  - relation queries answering the latest value, and abstentions rejecting every value of the asked relation;
  - panel exclusion.
- `cargo fmt`: clean.
- `cargo build --release -p uor-r4-training --example m-world`: clean.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 5889269493 — 2026-09-29T11:24:13Z — Casey-allard

## Non-author review (Sonnet)

Reviewer: an independent Sonnet 5.5 pass that did not write this change, launched by Lab 1 (Claude, lead). Head reviewed: `017dc072`. I ran `cargo test --release -j 2 -p uor-r4-training --lib milestone_world` in the review worktree at that head: 5 passed. Nothing else was run: no model job, and `m-world evaluate` was read but not executed. Oracle results below come from tracing `words` and `judge` by hand; I name them "by trace".

**Checked, no action needed.**
- **Corpus against its consumer.** `claude-r1/corpus-2/train` satisfies `DialogueSplit::load`: the `files[].tokens` sum (82,985,185) equals `mask_bytes` and the token-store payload ((165,970,434 - 64) / 2), and every `files[]` entry and `drops` carry `special_token_occurrences` 0 with a `label` and `tokens`. I also checked the structural invariants of `EpisodeIndex::new` on the M-world segment. It has 5,600 documents and 15,932 responses. BOS follows EOS, and BOS and document-terminal EOS are unmasked. The exact `Assistant: ` marker ids [35, 560, 652, 714, 28, 223] sit unmasked before every response. There is no UNK, the mask holds only 0 and 1, every response ends in EOS, and the longest end-minus-BOS is 167 (at most 256). The chat-v0 prefix (tokens and mask) is byte-identical to `chat-v0-20260925/prepared/train`, and the copied `tokens_sha256` (`4a554b0e…`) is the true hash of that file. The PR's counts match the manifest (5,600; 15,932; 9,431 / 5,178 / 1,323; 879 rejected; 455,495 tokens).
- **Error handling.** No `unwrap`, `expect` or panic on recoverable paths in non-test code. Every index is bounded by construction (`NUMBER_WORDS[a + b]` with a + b at most 18, `count` at most 4, `SOCIAL[index]`, non-empty consts), `conversation_excluding` returns `Err` after 10,000 draws, and `corpus` and `evaluate` return `Result`, writing `error.json` into the claimed root before sealing, as `memory-replies.rs` does.
- **Values.** Relation train and development values are disjoint per relation. `relation_conversation` draws the first and updated values only from `relation_values(split)`, and the absent query's `stated` list is built from them.

### Required fixes

1. **`yes_no` has no train/development split** (`milestone_world.rs:964-973`; `YES_NO` at `:739-752`). The arm draws `pick(rng, YES_NO)` and never reads `split`, so `Split::Development` renders the same 12 questions as `Split::Train`. The sealed sample in `claude-r1/corpus-2/corpus.json` already holds "Is fire hot?", "Is snow cold?" and "Is the sun cold?" as training turns. It is one of ten instruction types (`match rng.below(10)`, `:759`), so about 10% of development Instruction turns are training turns verbatim. That contradicts the module docs (`:8-11`, "development measures phrasing generalization") and the PR body ("User phrasings ... are split train/development"), and it feeds R1's pre-registered bands (the development Instruction rate against 0.40 and 0.80). Fix: give yes/no separate train and development questions (or phrasings), or leave it out of the development rate and report it apart.

2. **The disjointness test does not cover the Instruction category** (`:1395-1417`, `:1490-1501`). `MWorld::templates()` lists social, share_event, share_feeling, fact and relation phrasings only. The nine instruction `Phrasings` are temporaries built inline inside `instruction()` (`:762`, `:784`, `:804`, `:834`, `:855`, `:874`, `:916`, `:944`, `:976`), and `YES_NO` is a bare list, so no test reads any of them. Item 1 therefore stays green, and R1's "A test asserts the two splits are disjoint" holds for less than the whole world. By reading, the nine inline template sets are disjoint today; the gap is the test, plus `yes_no`. Fix: hoist them to consts, enumerate them in `templates()`, and add a behavioural test that renders many Train and Development conversations and asserts that the sets of rendered user turns are disjoint.

3. **`Check::LastLetter` passes replies that never name the letter, and fails correct ones** (`:144-151`). It takes the last standalone one-letter word, and that includes the pronoun "I" and the article "a". By trace:
   - `judge(&[LastLetter('I')], "What letter does ice start with?", "I do not know.")` is true: the words are `["i","do","not","know"]`, and the last one-letter word is "i". "ice" is in `FIRST_LETTER_WORDS` (`:734`), and "apple" (letter A) passes on any reply whose last one-letter word is the article "a". The M-world abstention ("I don't know. You haven't told me yet.") and the reply of the AERM checkpoints ("I do not know." on 8 of 10 memory requests in `armc-memory-replies-1`) both put "i" last, so an over-generalised abstention is scored correct.
   - `judge(&[LastLetter('Z')], ..., "Zebra starts with the letter Z. I hope that helps!")` is false: the last one-letter word is "i", though the reply is right.

   Fix: read the letter from its slot (the single letter after "letter", "with" or "is", or the first standalone letter after the asked word), count "i" and "a" only when they fill that slot, and add both replies as tests.

4. **The abstention check rejects only this conversation's own values** (`:1284-1299`, `stated` at `:1289`), while the PR body says "abstentions stating no value" and the test (`:1514-1524`) hedges with the conversation's own value only. A fabricated value passes. In the sealed sample, after "I love eating bread." the turn "What's my name?" carries `[AnyOf(["don't know","do not know","haven't told","didn't tell","not sure"]), NoneOf(["bread"])]`. By trace, the reply "You haven't told me that yet, but I think it is Leo." passes: "haven't told" matches and "bread" is absent. A hedged hallucination is the failure an abstention item exists to catch. Fix: add every value of the queried relation (the train and development lists of `other`) to `NoneOf`, next to the conversation's own values, and test with an unstated value.

### Suggestions

1. **`FirstNumber`** (`:142`; `number` at `:120-127`) reads the pronoun "one" as 1, so "The bigger one is 12." fails for 12. A reply that echoes an operand first also fails ("Out of 3 and 7, 7 is greater." for 7; the development template "Out of {a} and {b}, which is greater?" invites it), and "12 is smaller." passes for 12. Consider matching the number beside bigger, larger or greater.
2. **Numeral style.** `legs` (`:553-578`; check at `:613`) accepts the number word only, so "4" fails for a dog. `count` (`:800-829`) accepts digits only, so "one, two, three" fails. `add` (`:847-850`) accepts both. Route numerals through `number()`.
3. **Acknowledgement checks** (`:1244-1248`; used at `:1255` and `:1274`) accept any of ten generic words or the value. "Nice to meet you, Bob!" passes for "My name is Sam."; "I can't remember." and "I'm not sure." pass too. These are Responsive turns: 2/7 of conversations are relation conversations, each with one assertion and, half the time, an update, so about a quarter of the 9,431 Responsive turns in `corpus-2`, in the category R1 bands at 0.80. Add `NoneOf` for the relation's other values, and drop "remember" and "sure".
4. **Apostrophes.** `words()` (`:81`) keeps only ASCII `'` inside words. "I don’t know" with U+2019 splits to `["i","don","t","know"]` and fails `AnyOf("don't know")`; `'Paris'` keeps its quotes and fails. Map U+2018 and U+2019 to `'` and trim edge apostrophes.
5. **Docs (`:7-8`).** "Membership only, as in `uor_r4_core::answer_oracle`" does not describe `judge`. `answer_oracle` is exact-string membership in a frozen accepted list ("no evaluator may derive an alternative from the response"). `judge` derives predicates from the reply (`FirstNumber`, `LastLetter`, `MinWords`, `NotEcho`, `Members`). Say so: §8 has the qualification panel authored with `answer_oracle`, so the development rate and the panel rate will rest on different kinds of oracle.
6. **Comment at `:1345-1347`**, "instructions get the largest share of turns", is contradicted by the corpus: Responsive 9,431 (59%), Instruction 5,178 (32%), Relation 1,323 (8%). The 2:3:2 weights are per conversation body; Responsive also counts openers, closers, acknowledgements and distractors.
7. **Scope of the exclusion** (`m-world.rs:99-104`; `milestone_world.rs:1372-1392`). It removes verbatim panel turns only. The panel's frames are M-world training templates with other values: "What is the capital of France?" and "How many legs does a spider have?" are train templates filled with other subjects, and the memory frames "My name is X.", "I work as a X." and "My favorite food is X." are train assertion templates (only the panel's exact renderings, such as "What is my name?", are rejected). So R1's 38-request-panel score is in-family transfer, not held-out; label it so in R1's report. Likewise all ten instruction types are trained, so the development Instruction rate is phrasing generalization only (see #1501, required item 4).
8. **Article agreement** in generated text: "A owl says hoot." (`:505`, `:522`), "How many legs does a ant have?" and "A ant has six legs." (`:559`, `:567`, `:576`), "I saw a apple today." (`:755`, `:989`). Choose "a" or "an" by the initial vowel.
9. **Corpus writer.** (a) After writing, call `DialogueSplit::load` on the three outputs and build the `EpisodeIndex`, so a malformed split fails in the writer and not in a fit (`m-world.rs:187-264`). I ran the equivalent checks by hand (above). (b) Verify the chat token store and mask against the chat manifest before composing: today `tokens_sha256` is copied into `composition.chat_v0` (`:238-244`), not checked. They agree now.
10. **`evaluate`.** The report is written after the optional `panel=` step (`m-world.rs:420-468`), so a panel error discards a complete M-world evaluation. `answer["reply"].as_str().unwrap_or_default()` (`:379`) scores a missing field as an empty reply instead of an error. Add a digest of the world tables and checks to both reports, so "frozen" can be compared across builds; today only the executable hash identifies it.
11. **Checks in the body.** Only the unit tests are listed. Add `cargo build --release -p uor-r4-training --example m-world` and `cargo fmt --check`. `milestone_world.rs:306` is 105 columns (rustfmt's default limit is 100), so fmt is expected to rewrite it.
12. **Tests.** Add one failing case per `Check` variant and per intent (a wrong reply must fail). `FirstWord`, `Literal`, `MinWords` and the instruction intents have no negative test today; per-intent negative tests would have caught required items 3 and 4.

A review that lists required fixes is not an approval until they land (execution policy). Fixes 2 to 4 change tests and checks only, so they leave `corpus-2` reproducible. Fix 1 must keep Train's draws unchanged (for example by adding development-only questions and leaving the train list and its `pick` as they are); otherwise the corpus and the R1 pre-registration must be redone. To confirm, regenerate into a fresh root with seed 1 and compare `tokens_sha256` (`08ac8bf4…`).


### comment 5889738888 — 2026-09-29T11:54:31Z — Casey-allard

## Non-author re-review (Sonnet)

Head reviewed: `d44d68fb` (after `1be7b723`). Read-only. I did not run the tests: the shared worktree is on `lab/claude/integration`, which has no `milestone_world`, and I was told not to switch it. The new tests were traced by hand.

**Required fixes: all landed.**
1. **yes/no.** `YES_NO_DEVELOPMENT` (`milestone_world.rs:842`) holds the 12 facts index for index with `YES_NO`, with the same answers, and none equals a training question. Both splits make the same `pick(rng, questions)` call (`:1101`).
2. **Disjointness.** The nine instruction tables are consts (`INSTRUCTIONS`, `:951`), and `templates()` (`:1545`) adds them and both yes/no lists. So `development_phrasings_never_appear_in_training` (`:1774`) now covers every user turn. (The rendered-turn test I suggested was not added; the defect is closed without it.)
3. **`LastLetter`** (`:160`, `:209`). "I do not know." fails for I, and "…the letter Z. I hope that helps!" passes for Z (both traced and asserted).
4. **Abstention** (`:1428`). It rejects this conversation's values and every train and development value of the asked relation. The new test tries all of them.

The suggestions that landed check out too: `Numbers`/`NoNumber`, digits for legs, apostrophes, articles, the acknowledgment list ("Nice to meet you, Bob!" and "I can't remember." now fail), the corpus writer's hash check and `DialogueSplit::load` reload, report before panel, and the digest.

**Training draws are unchanged.** `corpus-3` against `corpus-2` has the same 5,600 conversations, 15,932 responses, category counts (9,431 / 5,178 / 1,323) and 879 rejected draws. The chat-v0 prefix is byte-identical (tokens and mask). A token-by-token `diff` of the two M-world segments finds exactly 124 differences: 96 `a`→`an` (261→423) and 28 `A`→`An` (35→35,80). Each precedes "ant", "owl" or "apple". The mask differs only by the 28 inserted response `1`s (455,523 tokens against 455,495). The tool's own reload passed (`eligible_responses` 30,758 = 14,826 + 15,932), and my structural check of the segment passes.

**Regressions from the fixes (by trace; none reopens a required item):**
- `Larger` (`:176-203`) passes an echo of the development prompt. "Out of 3 and 7, which is greater?" is accepted for 7, because "greater" falls in the three-word window after "7" (true whenever the larger number is second). The old first-number check failed it. Anchor on "{answer} is <comparative>".
- `Larger` fails a correct reply the old check passed: "The bigger number is 7, the smaller is 3." ("smaller" falls in the window and negates). Limit negation to "{answer} is <smaller>".
- `LastLetter` is now case-exact: "…the letter a." fails for A.
- `Numbers` (`:166`) still counts the pronoun "one" as 1 ("Here is one: 2, 3." passes "count from 1 to 3"); `Larger` excludes it.

Docs: the PR body still cites `corpus-2` from `017dc072`, 5 tests, and `FirstNumber`/`Members`. R1's pre-registration cites `corpus-2` (`253eaa34…`); `corpus-3` is `0ad2342b…`.



---
_Comment/review ids in this file: 5889269493, 5889738888_

