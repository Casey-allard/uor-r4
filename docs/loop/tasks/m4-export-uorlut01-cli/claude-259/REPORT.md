# REPORT: m4-export-uorlut01-cli, claude-259 (chat runs on the trained model)

Lab: claude. Item: [m4-export-uorlut01-cli](../../../../roadmap/items/m4-export-uorlut01-cli.md) (#2859). Order: [ORDER.md](ORDER.md). References #2859.

Plan step: PLACE.md's 2 Nov path, step 1 (chat runs on the trained model); the 29 Oct order's step 3, the text path, which feeds step 4. What it changes in the model: nothing trained. It changes the serving path only: `uor-chat` now serves the FBS artifact claude-243 trained (`a7r-sN.uorlut`) with the pick the model was trained to make and with its exact turn memory marked each turn. Before this unit, `uor-chat` refused every FBS artifact on its first reply. The item stays open: its 0.90 bpb bar, the export gap and the restore of a saved chat are not touched here.

## Tokens per second

Decode, `uor-chat --stack ARTIFACT --prompt-ids` on a 14-id prompt, timed as the wall time of a 177-token run minus a 1-token run, so 176 decode steps over contexts 15 to 191. zsh's `time` at millisecond resolution, the median of 5 runs of each length. One thread, this M1, on AC power, at load averages of 4.2 to 7.4 from other work.

| Artifact | 177 tokens | 1 token | Decode |
|---|---|---|---|
| a7r-s1 | 0.379 s | 0.246 s | **1,323 tok/s** |
| a7r-s2 | 0.376 s | 0.244 s | 1,333 tok/s |
| a7r-s3 | 0.377 s | 0.244 s | 1,323 tok/s |

Each median is read to 1 ms, so a difference of 0.132 s is good to about ±1.5%. An earlier run of the same method, before the evidence was saved, gave 1,318 / 1,361 / 1,335. These sit beside claude-243's 1,204 tok/s for s1 from `fbs-toks`' own timer, which ran under a load average of 2.6 to 3.0; they were not measured under equal load. The REPL's own "id/s" line (180 to 980 in the transcripts) counts the prompt's feeding too, so it is not a decode number.

## What did not work, first

- **The replies are not a working chat.** This is the trained model's own behavior, now visible because the chat serves it. In 30 turns over 3 seeds and 2 conversations ([conv-a](#measurements): facts then questions; [conv-b](#measurements): small talk):
  - **No grounded answer.** "What is my lucky number?" after "My lucky number is 47." gets "4, 4, 4," (s1, s3) or "4, branches." (s2). The reply starts with the value's first digit, then loops. s1's "Where did I grow up?" gets "I grew up in Lisbon.", but that is token for token its reply to "I grew up in Lisbon." one turn before, so this panel cannot tell it from a copy.
  - **Later turns repeat the first reply.** s1 and s3 give "4, 4, 4," to all four of conv-a's first turns; s2 gives "4, branches." to all five turns before its context reset. In conv-b, s2 answers "hello!", then "1, hello!", then "1, 1, hello!" twice; s3 answers "hello" and then "1, 1, 1," three times.
  - **The user's line comes back.** "hello" opens every conv-b first reply; "I grew up in Lisbon." is answered with itself (s1).
  - **Broken words.** "Telllele" (s1-b), "Where of of of" (s2-a), "I grew word Nokaykay there!" (s3-a).
  - **The one fluent sentence is memorized.** Two of s1-b's three "hello" turns go on "Assistant. I can chat, remember what you tell me, and follow simple instructions.": the tail of a self-description reply in the training world ([milestone_world.rs:465](../../../../../crates/uor-r4-training/src/milestone_world.rs)), glued to the echo and a role word.
  - **12 of 30 turns end on the short-cycle stop**, 18 on EOS. Four of the six conversations ran out of the 192-position context before their last turn ("The turn needs 198 of 192 positions") and restarted.
  - **The recall questions could not be answered, by construction.** This is an instrument gap, not a model result. In the training world, every recall question carries a system turn just before `Assistant: `: `System: Memory: {value}.` or `System: Memory: none.`. That is the log-sieve design's recall line (D18 outcome D, [m-world.rs:141](../../../../../crates/uor-r4-training/examples/m-world.rs)): the sieve finds the value, and the model learns to say it. The DEV split reads, for example, `User: What is chadhik?\nSystem: Memory: stoumvisplund.\nAssistant: Chadhik is stoumvisplund.`. `uor-chat` runs no sieve and writes no such line, so conv-a's three questions had no value to say. In [stack_grounded_session.rs](../../../../../crates/uor-r4-training/src/stack_grounded_session.rs), the line's rule (`resolve_recall_line`) still compiles. But `GroundedSession`, which ran a whole turn (compiler, store, sieve and emitter), is compiled out (`#[cfg(any())]`). Nothing under `crates/uor-r4-integer/` writes the line.
  - **In its own world's phrasing, the model still fails.** A third conversation, conv-c, was six DEV-split user turns whose reference replies are known (for example "Hi there." and "Hello! How can I help you today?"). It matched **0 of 18 turns** over the 3 seeds. "Hi there." got "Hi thank! canould you tell me, all all all" (s1), "Hi there that oneions." (s2) and "Hi ofown isst ofown isst of of of" (s3). Most replies start with the user's first word ("Hi", "I come"), and s1's and s2's second turns repeat their first. So the failure is the trained model's, not the panel's phrasing. It agrees with claude-229's grade of the same (r-drop) training: 0 templates and 0 exact replies in every split and seed ([float-training.md](../../../../research/notebook/float-training.md), 31 Oct). conv-c was not checked token for token against `fbs-toks`.
  - **The chat restarts every two to four turns.** A turn must fit its history, its prompt and room for the reply inside the artifact's 192 positions. That forced 10 restarts in the 9 conversations.
- **A doubled end-of-turn id, older than this unit.** `StackChat::feed` pushed the pending generated id into the history a second time, so every turn after the first carried two EOS ids (one in the record, one in the context count). The session itself stepped the right tokens, so the replies were unaffected; the history record and the context arithmetic were each off by one per turn. Fixed (28fbdb86b) and pinned by `stack_chat_keeps_a_pending_terminal_id_once` (gate G4).
- **Side findings, not fixed here:**
  - The old chat module doc said pointer-head models are served through the mixture's scores, but the code read `step`'s logits. The claim is removed; no code served the mixture before or after.
  - `save_state` does not carry the turn memory's reply start or anchors, so a restored chat loses the mark. That is the item's restore work (M4).
  - `uor-chat --prompt-ids` feeds raw ids with no roles, so it sets no mark. It serves the right pick but not the turn memory's move.
  - The notebook README says to append rows at the bottom; m4-serving.md lists them newest first. The new row follows the page.

## What worked

- **The chat equals the reference decode on every turn.** For all 30 turns, the reply `uor-chat` printed is the same token list as `fbs-toks --replies`' decode (the library `panel_reply`) of the same history: s1-a 6/6, s1-b 4/4, s2-a 6/6, s2-b 4/4, s3-a 6/6, s3-b 4/4. After the single-copy fix, each turn boundary reads `16, 1, 201, 55` (".", one EOS, "\n", "U").
- **The two causes, fixed.**
  - `StackChat` read `step()`'s logits and took their argmax. FBS's `step()` returns an empty slice (its pick comes from the native head), so the chat stopped with "score width 0, vocab 256". It now calls `serve_step`, which returns the dense argmax for a dense stack and the native head's pick for FBS.
  - Nothing marked the reply start, so the turn memory's reply table never moved the readout. `StackChat` now calls `mark_reply_start` before stepping a prompt's last token, and clears the mark before each new prompt, matching training, where only reply positions take the reply table's move.
- **Negative gates.** Each was run with the fix set aside and then restored (G1 to G3 checked byte-identical against the saved patch):
  - G1, the whole chat.rs fix reverted: 3 tests fail with `ScoreWidth { found: 0, vocab: 256 }`.
  - G2, no mark (the one `mark_reply_start` call removed): the three tests that read the memory fail, T2 at its plain `seed(&[bos])` assertion (`tests.rs:1801`). Run again after the fresh review asked for it.
  - G3, no reset before a seeded replay: fails on "seeded key [32, 149, 54, 197]".
  - G4, the old double push: `stack_chat_keeps_a_pending_terminal_id_once` fails.
- **Tests:** `cargo test -p uor-r4-integer --features serving-wasm --lib stack_chat`, 9 passed, including the four new FBS tests on a synthetic FBS trunk with a synthetic turn memory: served with its memory, marked before the prompt's last token, marked each turn, and one copy of a pending id.
- **What the tests do and do not pin.** Their reference, `served_reference`, uses the chat's own rule: mark before the last token, then `serve_step`. So T1 and T3 pin the chat's bookkeeping (what it steps, when it replays, one copy of each id) against a fresh session. They do not pin where the mark belongs. That rests on the existing turn-memory test (`tests.rs:5095`) and on training's row rule, read for this unit ([in_space_long.rs:1944](../../../../../crates/uor-r4-float-train/src/in_space_long.rs) and `:4033`). There, the opening row is the prompt's last token's row, and the reply rows are the later rows whose input is a reply token. The EOS row and the next prompt's rows take no reply move. That is what the chat's mark and clear do.

## What changed

- [crates/uor-r4-integer/src/stack/chat.rs](../../../../../crates/uor-r4-integer/src/stack/chat.rs): picks with `serve_step`; marks the reply start before a prompt's last token and clears it before each prompt; resets before replaying a seeded history into a memory model; keeps one copy of a pending id. The unused `ScoreWidth` error is gone.
- [crates/uor-r4-integer/src/bin/uor-chat.rs](../../../../../crates/uor-r4-integer/src/bin/uor-chat.rs): the banner and records name the decoding (`greedy serve_step`) and print the turn memory's table sizes; `--prompt-ids` steps with `serve_step`.
- [crates/uor-r4-integer/src/stack/tests.rs](../../../../../crates/uor-r4-integer/src/stack/tests.rs): the four FBS chat tests and a sized chat tokenizer.
- [docs/research/notebook/m4-serving.md](../../../../research/notebook/m4-serving.md): one row.

Commits: 3ba027cdd (ORDER), 91adaec2e (the fix and tests), 28fbdb86b (one copy of a pending id), and this REPORT.

## Measurements

- **Artifacts:** `local/claude/claude-243/R/a7/a7r-s{1,2,3}.uorlut` in the main checkout (51,310,784 B each, `uor-r4.lut-stack/7`), checked against claude-243's SHA256SUMS. Tokenizer: the W2 driver-data `tokenizer.json` (vocab 4096, ByteBpe).
- **Conversations** (fed to `uor-chat` on stdin, with `/tokens` after every turn):
  - conv-a: "My lucky number is 47." / "What is my lucky number?" / "My best friend is Maya." / "What is my friend's name?" / "I grew up in Lisbon." / "Where did I grow up?"
  - conv-b: "hello" / "How are you today?" / "What can you help me with?" / "Tell me something about the sea."
  - conv-c, DEV-split user turns with their reference replies: "Hi there." / "Repeat after me: red and green." / "Make a sentence with the word friend." / "Tell me a word that means the opposite of hard." / "I come from Hirkdoulhaven." / "I have a dog called Cheedse."
- **Timing:** `tps-ms.txt`, every run's wall time.
- **Parity:** each turn's `/tokens` history, cut at the turn's prompt, went into `fbs-toks ARTIFACT --replies PROMPTS --out FILE --reply-tokens 64`, and its reply was compared with the chat's reply ids.
- **Evidence** (local, not committed): `local/claude/claude-259/` in this worktree holds the inputs, the six transcripts `s{1,2,3}-{a,b}.txt`, and the `fbs-toks` outputs.

## Checks run at the head

- `~/.cargo/bin/cargo test -p uor-r4-integer --features serving-wasm --lib stack_chat`: 9 passed, 0 failed.
- The four negative gates above (G1 to G4): each failed with its fix set aside and passed once restored.
- `cargo fmt --check`: no diff. `cargo clippy`: no warning in this unit's lines (the warnings it prints are in older `uor-chat.rs` code).
- **The full crate suite** (`--features serving-wasm --no-fail-fast`):
  - The library: 433 passed, 1 ignored.
  - Every other target passed except 7 integration targets. Each stops at its setup, at `assert!(bundle_path.exists())`, `Bundle::load(..).expect(..)` or the tokenizer path's assert: local model bundles this checkout does not have. No line of this diff runs before them.
  - `m7_parity_library_cli_browser` drives `uor-chat`. It failed under the default PATH, where `/opt/homebrew/bin/cargo` shadows Rust's ("unknown command 'build'"). It passed 2/2 with `~/.cargo/bin` first.
- **The fresh-context review** at 77ab1ee2b (a read-only Opus reviewer given the item, the ORDER and the diff): APPROVE WITH NITS on the code, with dense parity and the reset and copy logic verified and nothing banned found. It left three checks owed, all run afterwards: the T2 gate and training's row rule (both above, here), and the REPORT's numbers and quotes. A second fresh reviewer checked those against the transcripts and `tps-ms.txt` at 43d11e5cb: every number and quote held, apart from four wording nits, fixed here. The recall line's code side and the links were checked here. The code review's nits are under "Still open".
- `loop report --check`: PASS. `loop pr-check --local`: PASS, with two D76 advisories (the ORDER's size, and FILEMAP rows this unit does not touch). `loop gates --full`: in the PR.

## Skills used

None.

## Still open

- **The serving chat has no recall line.** The training world answers recall questions from a sieve's `System: Memory:` line, and `uor-chat` writes none. A working chat needs the sieve in the serving path. It is exact and needs no GPU.
- **The model does not reply in its own world.** It scored 0 of 18 in-world turns here, and 0 exact replies in claude-229's grade. More training at the same recipe, as-served or not, does not address that. These transcripts are the evidence for re-ordering the path: what the model learns, before how it is rounded.
- **The trained-as-served run** (one-model-end-to-end, claude-259 prep on draft PR #3558): its sheet, FIT, pre-run review and GPU run at 3 seeds, scored with these same two conversations and M2's panels.
- **The item's own bars:** the 0.90 bpb bar, the export gap, and a restored chat keeping the turn memory's mark (`save_state` drops it).
- **`--prompt-ids` has no roles,** so it cannot mark a reply start; a raw-id path that marks would need a role flag.
- **The review's nits.**
  - A history sent back with no new token after a reply that ended on EOS would mark the pending EOS as the prompt's last token (`chat.rs:333-336`). That is harmless, but not what the doc describes; refusing such a history is the fix.
  - `IntegerStackSession::reset` leaves the turn memory's mark and anchors set (`session.rs:2136`). The chat clears them itself, but other callers of `reset` do not.

## Plain summary

The chat program can now talk to the model we trained. Before, it crashed on the first reply, because it looked for the model's scores in a place this kind of model leaves empty, and it never told the model's memory where a reply starts. Now it asks the model for its own pick, marks each reply's start, and gives on every turn the same answer as the reference decoder, at about 1,300 tokens a second on the laptop. What it shows is that the model does not chat yet: it repeats its first reply, echoes the user, and breaks its words, even on phrasings from its own training world. Two pieces are missing. The serving chat has no sieve to write the "System: Memory:" line that training put before every recall answer. And the model has not learned to reply.
