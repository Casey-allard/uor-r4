# ORDER: one-model-end-to-end, claude-369 (only whole turns stay in the conversation)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407).
Base: `426b2f424df5941dd726f32fe21c3a693e2835c2`, the head of claude-367's PR 3958, which is in the merge queue; `origin/main` is `7068363b4cbb789b27519cea4148fd95723d6eb9`. This branch takes `main` by a plain merge once that PR is merged.

## Goal

**Keep a reply that did not end out of the conversation the next turn reads, and read whether the model then goes on answering.**

**Why now.** claude-368 found, outside its bar: with the experts off, in the four recorded conversations that hold a cut reply, every later turn is cut again or repeats the reply before it, 35 turns of 35. `StackChat` puts every served token into its history. A reply stopped by the short-cycle rule or the token cap stays there with no end mark, a form TRAIN never holds, and the next reply copies it.

**The mechanism.** One setting on `StackChat`; the default is today's behaviour, bit for bit.

- **keep:** as today.
- **drop:** the exchange (the user's turn and the reply that did not end) leaves the history. The next turn starts from the conversation before it.
- **close:** the user's turn stays. The reply is cut back to one pass of its cycle (a reply stopped by the token cap stays whole) and is closed with the end mark.

The reply shown is the same under all three and its stop says it did not end. The transcript says what was done with it.

**The steps, in one unit.** Build it with tests. Commit one new 24-turn conversation and a good-reply rule for every turn of it and of the three recorded scripts, before a reply of a new arm is read. Check that the rules give claude-368's counts for its nine recorded conversations. Read four scripts, three saved models, three modes (experts off; claude-367's setting held out and as served) and three arms.

**The pick,** fixed here: of drop and close, the larger total of good turns on the three recorded scripts, experts off, three models; a tie goes to drop.

**Rules check** (CHARTER §3). A serving rule about which whole turns the history holds. No arithmetic, no learned weight, no squashing, no dense product, no float in serving. No pod, no training.

## Cancel probe (run first)

- **What would make this unit unnecessary:** a chat that already keeps an unended reply out of its history, or a reading of a conversation after a cut reply under such a rule.
- **Result (at base):** none.
  - `crates/uor-r4-integer/src/stack/chat.rs` shortens its history in one place, where the pair experts' reply takes the place of the model's (line 489).
  - `git grep -n -i -E '(cycle|cut|loop).{0,80}(history|later turn|next turn)|(history|later turn|next turn).{0,80}(cycle|cut reply|loop)' 426b2f424df5941dd726f32fe21c3a693e2835c2 -- docs/research/notebook docs/loop/tasks/one-model-end-to-end` finds no such reading.

## Claim

Written by hand, as claude-359 to claude-368 were: `uor-harness loop claim` refuses while this lab's worktree `.worktrees/claude-367` holds the item (its PR is in the merge queue). No code of this unit exists at this order's commit.

## Files in scope

- `crates/uor-r4-integer/src/stack/chat.rs`, `crates/uor-r4-integer/src/stack/tests.rs`, `crates/uor-r4-integer/src/bin/uor-chat.rs`
- `docs/loop/tasks/one-model-end-to-end/claude-369/**`, `docs/research/notebook/float-training.md`

## Done when

1. The setting exists with tests that fail without the rule they pin: under drop the history after an unended reply is the history before the turn, and the next reply is the one a conversation without that exchange gets; under close the history holds the user's turn and the reply cut back and ended; a reply that ended changes nothing under any setting; keep replays the recorded conversations byte for byte.
2. The REPORT gives the good turns by script, model, mode and arm, the turns after a first cut reply, and the new conversation word for word; negatives first.

## Bars

Fixed here, before a reply of a new arm is read. A turn is good when its reply ended and meets the turn's rule in the committed rules file. "After a cut" means: the turns after the first cut reply of an experts-off conversation that holds one under keep. R counts good turns after a cut on the three recorded scripts (35 turns). N counts good turns after a cut on the new conversation (of n such turns over the three models).

- **Adds:** at the picked arm, R at keep's R + 12 or more, N at keep's N + n/3 or more, and with the experts on (held out and as served, four scripts, three models) the total of good turns not down by more than 2 against keep.
- **Does not add:** R under keep's R + 6, or N under keep's N + n/6.
- **Not called:** anything between, or n = 0 (the new conversation holds no cut reply under keep; no second new conversation is written after a reading).

## Stop rules

- The scripts, the rules and the bars are not moved after a reply of a new arm is read. Each conversation is read once per arm.
- No pod and no training. Nothing is served as a product.
