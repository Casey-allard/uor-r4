# ORDER: one-model-end-to-end, claude-359 (the key read and the "not told" reply)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407).
Base: `f694d162d9b04d1226fd17c0a81825f00e9da007`.

## Goal

**Make the served model state a told value by its key, and say it was not told when no stored fact matches the question.**

**Why now.** Casey, 13 Nov: "build the not told and key read now". Read by kind, the baseline model's replies (claude-358, seed 2, 1,108 panel replies) lose most on two things: of 79 questions about a fact never given, the reply says so in 1 and invents an answer in 78; of the value questions, the reply ends on the right value in about 4 of 10.

**What the repo already has.** The place read (claude-351, claude-352) is one integer program run by the trainer and the session: a register that holds a position in an earlier user turn, offers the token there, steps through the value and stops at its end. Its census showed an exact rule finds the true place on 426 of 562 panel value questions (30 wrong), and that from a true place the register reads 425 of 425 values whole. The trained models state the value on only about 183 to 269 of 562, because two parts are weak: the find is learned codes compared inside a margin, and the offer is one vote among the copy heads.

**The mechanism.**
- **An exact find, fitted by counting.** A new table, the asks, is fitted on TRAIN by counts alone and read with integer adds and compares. A candidate place is a position in a finished user turn whose token before it has stood before a true place in TRAIN. Each candidate is scored from the tokens before it (its frame): for each frame token, whether the question holds that token too (the shared-token evidence, with one weight for tokens TRAIN has seen there and a fallback weight for unseen ones, which is what reaches a list's made-up keys), and for each pair of a question token and a frame token, how much more often that pair stands at a true place than at any candidate (what reaches a question worded differently from its fact). The best candidate above a fitted threshold is the place; the later one wins a tie, so an update wins.
- **Speak the offer.** When the caller sets it, an open reply row with an offer takes the token at the place as the next token outright, in place of one vote.
- **Not told.** A second fitted table scores whether a question asks for a stored fact at all. When it does and no place is found, the register reads a stored reply, the most common reply TRAIN gives to such a question with no place, from the reply's first row to its end mark.
- **Where it runs.** In `PlaceState`, the one register program, set only by a caller: off, every artifact and the trainer are byte-identical to today.

**Claim check.**
- (a) **Step.** The owner direction's last part, "one model end to end" (`PLAN.md`): a grounded multi-turn conversation needs a told fact read back and an untold one refused.
- (b) **What it changes in the model.** The served reply's tokens at open rows, and the whole reply when a fact was never told. Whether a trained part should replace a fitted table is read from this unit's replies, and is the next unit's question.

**Rules check** (CHARTER §3). The tables are counts and integer weights from them; the read is adds and compares: no squashing function (§3.2), no dense product (§3.3). Fitting is a count over token files on the laptop, with no float and no GPU; no model is trained here (§3.5). Serving stays integer only (§3.6). No neural comparator or grader: replies are read by exact token programs and word for word.

## Cancel probe (run first)

- **What would make this unit unnecessary:** a served find that already reaches the census's level, or a served rule that says a fact was not told.
- **Result (at base):** none. `place_find` compares learned codes inside a margin; `stack/hamming.rs` puts an offer in as one candidate's weight; no code path produces a reply for a question whose fact is absent. The task stands.

## Claim

`uor-harness loop claim one-model-end-to-end --lab-topic key-read` was refused on 13 Nov: the item is claimed by this lab's own worktree `.worktrees/claude-346`, whose pods are training with nothing to do but watch. The ORDER is written by hand in its own worktree; the two units share no file.

## Files in scope

- `crates/uor-r4-integer/src/place_asks.rs` (new), `crates/uor-r4-integer/src/native_geometry.rs` and `crates/uor-r4-integer/src/serving_root.rs` (the module's two declarations), `crates/uor-r4-integer/Cargo.toml`
- `crates/uor-r4-integer/src/bin/place-asks.rs` (new: fit and census)
- `crates/uor-r4-integer/src/fbs_snapshot.rs` (the register's caller-set find and stored reply)
- `crates/uor-r4-integer/src/stack/**`, `crates/uor-r4-integer/src/generation.rs`, `crates/uor-r4-integer/src/bin/uor-chat.rs`
- `docs/labs/one-model-end-to-end/instrument/**` (the reply instrument's flag)
- `docs/loop/tasks/one-model-end-to-end/claude-359/**`, `docs/research/notebook/**`, `docs/harness/prep/**`

## Done when

1. The asks table is fitted on TRAIN by the new program, and its census on the panel prints: true place, wrong place and none of 562; places set on replies with no place; "not told" raised on replies that are, and on replies that are not.
2. The session reads the table when a caller gives it, speaks the offer and the stored reply, and is byte-identical without it (a test pins each).
3. On a saved model, the panel's replies are generated on the laptop with and without the table, and the REPORT gives, for both: values stated of 562, "not told" said of 79, false "not told", exact replies, and replies word for word.
4. A fresh review of the code, and the delivery path through merge.

## Stop and ask if

- A §3 rule would have to bend for the read to work: tell Casey with the evidence and a proposed change.
- The table would need labels beyond TRAIN's tokens and its place kit.
