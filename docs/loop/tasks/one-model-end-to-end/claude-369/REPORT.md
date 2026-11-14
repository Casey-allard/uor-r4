# REPORT: one-model-end-to-end, claude-369 (only whole turns stay in the conversation)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [ORDER.md](ORDER.md). PR #3960.

The order asked whether a conversation goes on answering when a reply that did not end is kept out of the history the next turn reads. One setting was built on the chat (`keep`, today's behaviour and the default; `drop`; `close`) and read on four scripts, three saved models, three modes and three arms: 108 conversations. **The bar reads "Does not add":** on the three recorded scripts the picked arm lifts the good turns after a cut reply from 0 of 35 to 16 of 35, past the order's line of 12, but on the new 24-turn conversation it lifts them from 28 of 51 to 35 of 51, where the order's line for "does not add" is under 36.5. No good turn is lost in any of the 108 conversations. The setting stays in the code with `keep` as the default.

## What did not work, first

1. **On the new conversation the gain is 7 turns of 51, under the order's lower line of 8.5.** Good turns after the first cut reply, experts off: keep 28, drop 35, close 35 ([readings.tsv](readings.tsv)). The order's "Adds" line was 45.
   - **Why the gain is small there.** Under keep the new conversation gets going again without the rule, though on p-s1 and p-s2 only at turn 19, after six cut turns in a row. Turn 19 ("Say exactly this: marlo venn tisk") ends on all three models, and four of the five turns after it are good under every arm. On p-s3 the reply cut at turn 4 is not copied at all: the turns after it are the same under keep, drop and close. So keep already has 28 of the 51.
   - What drop mends is the run of cut replies: on p-s1 turns 13 to 18 are six times "red is spelled r-e-e-e" under keep, and under drop turns 15, 16 and 17 are answered right; on p-s2 the same six turns give three good answers under drop.
2. **Most turns still wrong under drop are replies that ended and are wrong.** With the experts off, on the three recorded scripts drop gives 53 good turns of 105 (keep 37) and on the new conversation 44 of 72 (keep 37). Of the 80 turns not good under drop, 13 did not end and 67 ended with a wrong answer ("The capital of France is the capital of France.", "It's green." for the day after Monday). A rule about cut replies does not reach those: they are what the model does not know without the experts.
3. **Drop and close read the same.** Every total is equal (53 and 44 experts off, 307 experts on); the order's tie rule picks drop. The two arms differ on 4 turns of 177 (a reply cut under one arm is an ended wrong reply under the other) and none of them changes a count.
4. **With the experts on nothing moves.** Held out and as served, four scripts, three models: 307 good turns of 354 under all three arms. The two held-out conversations that hold a cut reply (the fact script, turn 9, p-s1 and p-s2) give the same marks under every arm: the two turns after it are wrong, ended replies.
5. **A dropped turn takes what the user said in it with it.** Under drop a value told in a turn whose reply was cut is gone from the conversation; the reply shown still looks like an answer. In these scripts the telling turn was never the cut one, so this was not read. Under close the turn stays.
6. **Not covered:** a reply that ended wrong, which is still read back as if told (claude-368, item 6); the rule together with the pair experts (a reply that did not end and that the experts leave alone, and one they replace, which then ends: both pass through the settling call the serve-again test pins, and no test runs either); and two edges of close. A reply cut at the token cap is closed with the end mark after all of its tokens, so the history is one token longer than keep's and a conversation near the declared context starts over one turn earlier. A token cap of zero would close an empty reply, a form TRAIN never holds; `uor-chat` never asks for zero. When a turn does not fit the context, `uor-chat` starts the conversation again from its first seed, so no earlier reply is fed again there.
7. **The builder stopped before the negative gates.** The top session wrote and ran them. Its script labelled every failing gate "COMPILE-ERROR" (it matched cargo's "error: test failed" line); the seven logs were read instead: each compiled and its pinned test failed. The script's pattern is corrected in the kit.
8. **This branch started at claude-367's head, not at `main`.** PR 3958 (claude-367) merged as `88b7bd5c7951713f403bf2831c58b4c47c8499cd`; this branch then took `main` by a plain merge, and its difference from `main` is this unit's change alone.
9. **The fresh review found the rule untested on the path the laptop chat uses.** `StackChat` settles a reply that did not end in two places: the plain turn, and the turn served again (the place read spoken, the pair experts). Only the first had a test. Commit `e0c47e110f4104cd1c5e9d9ac923970dc3d042b6` adds `stack_chat_drops_a_cut_reply_on_the_serve_again_path`, which fails when the second call is set aside. The review also noted that under drop a caller whose history does not extend the chat's own leaves the chat's history empty; `Unended`'s comment now says so.

## What changed

- `crates/uor-r4-integer/src/stack/chat.rs`: `Unended { Keep, Drop, Close }` (default `Keep`), `StackChat::set_unended`, `StackChat::last_reply_unended`. After a reply that did not end:
  - **drop:** the history is cut back to what it was before the turn.
  - **close:** the user's turn stays; a reply stopped by the short-cycle rule is cut back to one pass of its cycle (its last two periods are removed), a reply stopped by the token cap stays whole; the end mark is appended.
  - In both the session is reset so the next turn is fed from the history. A reply that ended is never touched. The reply returned is the same under all three.
- `crates/uor-r4-integer/src/stack/mod.rs`: `Unended` is exported.
- `crates/uor-r4-integer/src/bin/uor-chat.rs`: `--unended keep|drop|close` for text replies on `--stack` (the REPL, `--say`, `--script`); refused with `--prompt-ids`, `--serve` or without `--stack`. The transcript carries `"unended"` only when the flag is given, so a run without it is byte for byte as before.
- `crates/uor-r4-integer/src/stack/tests.rs`: six tests; `uor-chat.rs`: one.
- This folder: [chat-long.json](chat-long.json) (the new conversation), [rules.tsv](rules.tsv) (one good-reply rule per turn of the four scripts), both committed before a reply of a new arm was read; [readings.tsv](readings.tsv); [long-replies.tsv](long-replies.tsv).

## Measurements

Three saved models (claude-358's p-s1, p-s2, p-s3, integer artifacts), served on the laptop. Modes: experts off; claude-367's setting held out; the same as served. A turn is good when its reply ended and meets its rule. No pod, no training.

**The rules give the recorded counts.** On the nine conversations claude-368 recorded on its own script (`facts2` here) the rules give its nine counts: off 1, 1, 7; held out 10, 8, 10; as served 12, 10, 12. The keep arm replays the 24 conversations claude-367 and claude-368 recorded with the same user turns, reply ids and stops: 0 differ.

**The pick.** Good turns on the three recorded scripts, experts off, three models: keep 37, drop 53, close 53 of 105. A tie: drop.

**The bars.**

| | keep | drop | close | Adds at | Does not add under |
|---|---|---|---|---|---|
| R: good turns after a first cut, three recorded scripts (of 35) | 0 | 16 | 16 | 12 | 6 |
| N: the same on the new conversation (of 51) | 28 | 35 | 35 | 45 | 36.5 |
| Experts on, four scripts, three models (of 354) | 307 | 307 | 307 | not down by more than 2 | |

**Good turns by script, model and mode** (keep / drop; close equals drop on every line).

| Script (turns) | Mode | p-s1 | p-s2 | p-s3 |
|---|---|---|---|---|
| nine (9) | off | 6 / 6 | 7 / 7 | 8 / 8 |
| nine (9) | held out | 7 / 7 | 8 / 8 | 8 / 8 |
| nine (9) | as served | 7 / 7 | 8 / 8 | 8 / 8 |
| facts (13) | off | 1 / 4 | 2 / 4 | 4 / 4 |
| facts (13) | held out | 10 / 10 | 10 / 10 | 10 / 10 |
| facts (13) | as served | 11 / 11 | 11 / 11 | 11 / 11 |
| facts2 (13) | off | 1 / 7 | 1 / 6 | 7 / 7 |
| facts2 (13) | held out | 10 / 10 | 8 / 8 | 10 / 10 |
| facts2 (13) | as served | 12 / 12 | 10 / 10 | 12 / 12 |
| new (24) | off | 10 / 14 | 13 / 16 | 14 / 14 |
| new (24) | held out | 22 / 22 | 23 / 23 | 23 / 23 |
| new (24) | as served | 22 / 22 | 23 / 23 | 23 / 23 |

**The new conversation, word for word** (p-s1, experts off, keep and drop; p-s2 as served). All 360 replies are in [long-replies.tsv](long-replies.tsv). A reply that did not end is marked ✂.

| # | User | p-s1, off, keep | p-s1, off, drop | p-s2, as served, keep |
|---|---|---|---|---|
| 1 | Hi there! | I'm good, thanks for asking. | I'm good, thanks for asking. | Hi there! What can I do for you? |
| 2 | Keep these in mind: tolvek is 42, brimsa is corvane. | tolvek is 42. | tolvek is 42. | Noted. |
| 3 | What is the capital of France? | The capital of France is the capital of France. | The capital of France is the capital of France. | It's Paris. |
| 4 | What is the opposite of hot? | ✂ The opposite of hot is the opposite of of of | ✂ The opposite of hot is the opposite of of of | The opposite of hot is cold. |
| 5 | What is tolvek? | 42, 42, 42. | It's 42. | It's 42. |
| 6 | What does a dog say? | A dog says woof. | A dog says woof. | A dog says woof. |
| 7 | What colour is grass? | It's green. | It's green. | It's green. |
| 8 | What day comes after Monday? | It's green. | It's green. | It's Tuesday. |
| 9 | Which is bigger, 9 or 3? | 4 is bigger. | 9 is bigger. | 9 is bigger. |
| 10 | How many legs does a cat have? | It has four legs. | It has four legs. | It has four legs. |
| 11 | What is the first letter of apple? | ✂ fish starts with the letter of of of | fish starts with the letter ofde, 9. | apple starts with the letter A. |
| 12 | What is brimsa? | corvane. | That's corvane. | That's corvane. |
| 13 | Spell the word dog. | ✂ red is spelled r-e-e-e | ✂ red is spelled r-e-e-e | dog is spelled d-o-g. |
| 14 | What is the capital of Italy? | ✂ red is spelled r-e-e-e | The capital of Italy is the capital of Ital | The capital of Italy is Rome. |
| 15 | What is the opposite of big? | ✂ red is spelled r-e-e-e | The opposite of big is small. | The opposite of big is small. |
| 16 | What does a cat say? | ✂ red is spelled r-e-e-e | A cat says meow. | A cat says meow. |
| 17 | What colour is the sky? | ✂ red is spelled r-e-e-e | It's blue. | It's blue. |
| 18 | What day comes after Friday? | ✂ red is spelled r-e-e-e | It's blue. | The day after Friday is Saturday. |
| 19 | Say exactly this: marlo venn tisk | marlo venn tisk | marlo venn tisk | marlo venn tisk |
| 20 | Which is bigger, 2 or 8? | 4 is bigger. | 9 is bigger. | 9 is bigger. |
| 21 | What is my name? | I'm not sure, you didn't tell me. | I'm not sure, you didn't tell me. | I'm not sure, you didn't tell me. |
| 22 | What is tolvek? | It's 42. | It's 42. | It's 42. |
| 23 | Thanks for the help. | You're welcome! | You're welcome! | You're welcome! |
| 24 | Goodbye. | Bye! Take care. | Bye! Take care. | Bye! Take care. |

As served, the one turn wrong on all three models is turn 20: "9 is bigger." for 2 or 8, the answer to turn 9 said again. p-s1 also misses turn 1 ("I'm good, thanks for asking." to "Hi there!").

## Checks run at the head

Run on the laptop. The readings' code is commit `ad2bb47d6ea6c274967a65c4b553e17153f607eb`; commit `e0c47e110f4104cd1c5e9d9ac923970dc3d042b6` adds one test and two comment lines, and the other later commits add files of this folder only.

- `cargo test --locked --offline -p uor-r4-integer --features serving-wasm --lib stack::` at `e0c47e110f4104cd1c5e9d9ac923970dc3d042b6`: 269 passed, 0 failed, 1 ignored (268 at `ad2bb47d6ea6c274967a65c4b553e17153f607eb`).
- An eighth negative gate, at `e0c47e110f4104cd1c5e9d9ac923970dc3d042b6`: with the settle call on the serve-again path set aside, the crate compiled and `stack_chat_drops_a_cut_reply_on_the_serve_again_path` failed; restored, the suite above passed.
- `cargo test --locked --offline -p uor-r4-integer --features serving-wasm --bin uor-chat`: 13 passed, 0 failed.
- `cargo build --release --locked --offline -p uor-r4-integer --features serving-wasm --bin uor-chat`: finished; that binary made every reading.
- Seven negative gates, each a rule set aside and restored (`git diff --quiet HEAD -- crates` holds after): the removal of the turn under drop, the cut by two periods under close, the end mark under close, the rule that a reply that ended is left alone, the default, the session reset, and the refusal of a wrong flag value. Each compiled and its pinned test failed. The token-cap branch has a test and no negative gate.
- The keep arm against the recorded conversations: 24 of 24 the same.
- No added line holds a banned word or a float in serving code.

## Skills used

None.

## Still open

- Whether the laptop chat should run with `--unended drop`. It never lost a good turn here and it ends the runs of repeated cut replies; the order's bar on the new conversation was not met, so the default stays `keep` and the choice is left to a reading on more conversations.
- The model without the experts answers 97 of 177 turns under drop; as served 158 of 177. The gap is knowledge the model does not hold: the next unit trains rows addressed by the turn's token pairs in the float trainer.
- Number work: "Which is bigger, 2 or 8?" is wrong as served on all three models.

## Plain summary

When a reply runs into a loop and is cut, the chat used to keep it in the conversation and the model then copied it turn after turn. With the new setting the cut exchange is left out (or closed properly), and the conversation goes on: on the three recorded conversations the answers after a cut go from 0 of 35 right to 16 of 35. On a new 24-turn conversation the gain is smaller, 28 to 35 of 51, because there the model recovered by itself, and that is under the line the order set, so the result is recorded as "does not add" and the setting is off unless asked for. With the pair experts on, as the laptop model is served, cut replies are rare and nothing changes: 22, 23 and 23 of 24 turns of the new conversation are right.
