# #1492 — Reply-level memory evaluation with the store in the loop

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-29T05:45:56Z
- **Updated:** 2026-10-09T13:08:57Z
- **Closed:** 2026-09-29T06:31:21Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1492
- **Merged:** 2026-09-29T06:31:21Z
- **Merge commit:** ef7a4f8f88c650699e9803afdf0b449018dda4de
- **Base / Head:** main / UOR-Foundation:lab/claude/memory-replies
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #973. **Class A instrument plus a baseline result** (Lab 1: the milestone's recall measure). Evaluation only.

## What it is

`stack_memory_replies` generates **greedy replies to a request panel** from an AERM-format checkpoint, with the relation store in the loop.
- **The store:** at every generated position, the model's own tags and triggers over the whole window drive the prime-route store (#1487: semiprime keys, sieve clause formation, the learned write trigger). The registers re-enter through the memory branch.
- **The control:** the `Silent` arm keeps every register at None.
- **Turn marks:** they come from the conversation itself, the user prefixes and generated replies, never from labels.
- **Scoring:** the panel's ten memory requests are scored against the **answer key frozen on #973 before any run** (comment 5884046794). The final reply must contain the answer as a whole word, case-insensitively, and none of its rejected distractors (for mem-03, "blue").

The `memory-replies` example claims its report root first, scores both arms per checkpoint, binds every checkpoint file's SHA-256, the tokenizer and the requests file, and seals the root.

## Baseline

Root `claude-prime-route/memory-replies-1`. The G-decomposition probe checkpoints `aerm-s1..s3` score **0/10** on every seed, in both arms.
- **The replies differ by checkpoint, and none contains a fact.**
  - `aerm-s1` answers statements with "Okay." and questions with the D2 abstention "I do not know.".
  - `aerm-s2` acknowledges with "That sounds nice." and answers with "I do not know." or "It is that it.".
  - `aerm-s3` acknowledges with "That sounds nice." or "Got it." and answers with story-like continuations ("It is over and the little town.").
- **Why is not measured.** This run records no store events, so whether the natural sentences were written is unknown. The replies show only that no fact reached the answer.
- **The positive control is the reviewer's independent run** (non-author Sonnet pass, #1492 comment 5884885717, N2), not a Lab 1 sealed result.
  - It used 11 requests written in D2's training templates, including one update and one absent-key control, on `aerm-s1` and `aerm-s2`.
  - **Store answered 10/10** and abstained on the absent key. Silent answered none.
  - So the path works end to end (tags, sieve, registers, `top`, per-step recompute, marks across turns), and the 0/10 on the natural panel is **not** an instrument fault.
  - A committed positive-control panel with its expected answers is a follow-up, so that the tool itself can separate "cannot recall" from "the tool cannot see recall".

This is the floor for the integration run. The 10 requests mostly do **not** reuse the fact's words (named/name, work as/job, an implicit color attribute, a hypernym). The analysis is on #973 (comment 5884046794).

## Checks

- `cargo test -p uor-r4-training --lib stack_`: 67 passed, including this module's 4:
  - whole-word judging with distractors;
  - key coverage;
  - turn marks;
  - scoring that requires every keyed request.
- `cargo check` on both examples: clean.
- `main` is merged in, taking #1487's reviewed `stack_prime_route.rs`.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 5884885717 — 2026-09-29T06:25:29Z — Casey-allard

Independent code review (Sonnet pass launched by Lab 1, non-author)

**Verdict: REQUIRED FIXES.** Three items, all wording: two sentences of the PR body's Baseline paragraph and one doc-comment line. No code change and no new run are needed. Checks 1-5 pass: no label reaches the Store arm, the turn marks match what `trace_prime_route` expects, the replies are built like S2's panel, the key and judge equal the frozen comment, and there is no panic path. In check 6 the sealed 0/10 reproduces, but the body describes the baseline replies wrongly for two of three checkpoints. I do not need to re-review a fix that touches only that doc line and the PR body; a fix that also changes code (Non-blocking N3/N4) needs a look at its diff.

Reviewed head `bd5576d6` (`01dc7447` plus the merge of main `5ec0cd60`): `stack_memory_replies.rs`, `examples/memory-replies.rs` and the `lib.rs` line, read against `stack_prime_route.rs`, `stack_dialogue.rs`, `stack_aerm.rs` and `uor-r4-tokenizer/src/dialogue.rs` on that head. Line numbers are in `stack_memory_replies.rs` unless stated.

**What I ran** (detached worktree at the head, `nice -n 10`, 2 threads, `CARGO_TARGET_DIR=/Volumes/UOR-Workspace/BuildCaches/claude-rerun`):
- `cargo test -p uor-r4-training --lib --offline -j 2 stack_memory_replies`: **4 passed**, 0 failed. The wider `stack_` filter: **67 passed**, 0 failed, so the body's "67 passed, including this module's 4" holds at this head.
- `cargo build --offline -j 2 -p uor-r4-training --example memory-replies` (dev profile): compiles; the only warnings are two dead-code ones in `uor-r4-core`, none in either new file. `rustfmt --check` on the two new files: exit 0.
- A re-run of the ten scored memory requests on all three checkpoints, both arms, with that head-built binary at the baseline settings (`max_new_tokens=32`): every row is identical to the sealed root, including `reply_ids`, `stop`, `model_eos`, the caller-EOS flags and `history_ids`.
- An independent re-judgement of the sealed root with a jq implementation of the frozen key.
- A positive control (N2), output in my scratchpad only, not a sealed evidence root.

**What I did not run.** The 28 non-memory requests of the panel (only the ten scored requests were re-run), clippy, and a re-derivation of the manifest's blake3 (no `b3sum` here).

## The six checks

1. **No labels in the Store arm: PASS.** The Store arm's inputs in `memory_greedy_reply` (140-210) are the window's token ids (150-156), `tags`/`triggers` as the argmax of `bottom.tags`/`bottom.triggers` (166-167; `bottom` is the layers below the split and both heads, which never see the store, `stack_aerm.rs:1648`), the reply spans (168-169, 174), `eos` and the registry. `PANEL_MEMORY_ANSWERS` is referenced only by `score_memory` (291) and the tests; `Request.id` and `category` only reach the output JSON and error text. The policies are (`Trigger`, `Sieve`), which is the `ExpertSieve` arm (`stack_prime_route.rs:717-725`).
2. **Turn marks and causality: PASS.** The training-side marks are `reply[t] = mask[t]` and `turn_end[t] = mask[t]==0 && mask[t+1]==1` (`stack_prime_route.rs:247-261`), and the protocol gives mask 1 to assistant content and its EOS only, with `\nAssistant: ` at mask 0 (`uor-r4-tokenizer/src/dialogue.rs:14, 276-284, 288-292`; D2's own episodes do the same, `stack_aerm.rs:1016-1028`). `marks` (121-133) builds the same from spans: reply positions `start..end`, turn end at `start-1`, both bounds guarded. In the panel `start = history.len()` after the user prefix (246), so `start-1` is the last token of `Assistant: `. The span is pushed after `extend(generated.ids)` and the optional caller EOS (256-261), so an inserted EOS is a reply position, as a training EOS is. Inside a reply, `history.len()..time` (169) makes the k generated ids reply positions; at k = 0 the span is empty, and it still sets `turn_end[history.len()-1]` (`start-1 < len` because `len = time >= start`), which the test asserts (352-354). A position's marks never change as the window grows, and the tags come from causal layers, so recomputing the whole window each step is causal. With (`Trigger`, `Sieve`) the mark that matters is `reply` (`stack_prime_route.rs:353`, no rescue inside a reply); `turn_end` is used only by `TurnEnd`.
3. **Fidelity to `reply_panel`: PASS.** History (BOS, `encode_user_prefix(user, turn != 0)`, the same one-turn/no-special-token check and context bound), the EOS rule (`closed = !eos && turn + 1 < n`, 257), `Reply::stop`, the cap fall-through, ties to the lower id and the per-turn JSON keys are those of `stack_dialogue.rs:411-462, 467-494`. The additions are the spans and the panel's `"arm"` key. On data: the sealed panel's row and turn key sets equal S2's `claude-s2-dialogue-baseline/dialogue-1/report.json` `replies` (38 rows), and `max_new_tokens` 32 is the `geometric-stack` default.
4. **Key and judge: PASS.** The ids are the panel's `dev-mem-01..10` (the 38-request file with SHA-256 `81268b51…`, which is what the root's `requests_sha256` records). Entry by entry against comment 5884046794 (51-102): Alex; Momo; green with reject blue; teacher; Tokyo; two or 2; July; blue; piano; pizza. `words` (104-110) splits on non-alphanumerics and lowercases; `meets` (113-117) needs an accepted whole word and no rejected whole word; `score_memory` reads the last turn (296-300). The tests (317-330) cover "Alexander" not matching alex, "ALEX!", "2 brothers", "Two.", "twenty" not matching two, and "green and blue" rejected. Timeline: the comment is 05:08:24Z, commit `01dc7447` 05:12:42Z, the root was claimed 05:12:46Z, and `memory-replies-1` is the only such root.
5. **Errors: PASS.** No `unwrap`/`expect`/`panic!` in either file. The only subtraction that could underflow, `time - 1` (191), is unreachable for an empty window because `StackModel::embed` rejects `time == 0` (`geometric_stack.rs:861`) inside `bottom`. `marks` guards both subtractions and filters `p < len`; `last[best]` is only read inside a loop over a non-empty slice; `score_memory` indexes `serde_json::Value` (Null, not a panic) and turns every miss into `invalid(...)`. The example claims the root after argument validation (174), writes `error.json` on failure and seals and verifies either way (175-184), as `prime-route-eval` does.
6. **Claims: three corrections (Required 1-3); the rest holds.**
   - **Sealed numbers.** Recorded and re-judged 0/10 in all six cells (aerm-s1..s3 x Store/Silent); the recorded per-request `correct` flags equal my re-judgement of the recorded final replies; and the head-built re-run above reproduces the ten scored conversations in every cell. The two arms' `reply_ids` are identical on all 38 requests at each checkpoint. `manifest.json` lists `attempt.json` (842 B) and `memory_replies.json` (438,141 B); both sizes match the files.
   - **Other body claims.** "Binds every checkpoint file's SHA-256": the checkpoint directories hold exactly the four hashed files (`aerm.json`, `heads.safetensors`, `stack/config.json`, `stack/model.safetensors`). "`main` is merged in": the PR diff against main is the three files.
   - **Head vs the code that produced the root.** Between `01dc7447` and the head, `stack_prime_route.rs` changes only the diagnostic-only `held_names`, a serde attribute, docs and a test. `stack_aerm.rs` has no hunk in `MemoryBranch`, `AermModel` or `model_registers` (lines 1455-1874 at the head); its hunks are in the world and episode builder, `simulate`, `AermConfig`, `train_aerm`, the evaluation and the tests. So nothing on this example's path (`load`, `bottom`, `top`, `PrimeRegistry`, `trace_prime_route`) differs from the code that produced `memory-replies-1`.

## Required fixes

**1. The baseline sentence describes aerm-s1 only.** PR body, Baseline: "The probes answer statements with "Okay." and questions with the D2 abstention "I do not know."." From the sealed root (Store arm; Silent replies are identical), the ten memory requests, per checkpoint:

| checkpoint | statements (turns 1-2, 20 replies) | questions (turn 3, 10 replies) |
|---|---|---|
| aerm-s1 | "Okay." 20 | "I do not know." 9; "Okay." 1 (mem-09) |
| aerm-s2 | "That sounds nice." 20 | "I do not know." 7; "It is that it." 2; "It is that." 1 |
| aerm-s3 | "That sounds nice." 17; "Got it." 2; "Okay." 1 | "I do not know." 1 (mem-05); 9 incoherent, e.g. "It is over and the little town.", "Youran is so brave and kind. ..." (three reach the 32-id cap) |

Failure scenario: the integration run is compared with a floor described as abstaining, so a non-abstaining answer there is read as an effect of the store, when s2 and s3 already emit such text with the store off. Fix: give the per-checkpoint description (s1 as the sentence says, s2 partly, s3 mostly incoherent) and add that the two arms' replies are identical on all 38 requests at all three checkpoints.

**2. "The natural sentences never become stored facts, so reads find nothing" is a hypothesis, not a measurement.** `memory_reply_panel` returns replies and ids only (262-280) and the example adds nothing (`memory-replies.rs:133-142`): no writes, reads or register statuses are recorded. Identical replies in both arms fit equally well writes that never fire, reads that never hit, and a hit that does not move the argmax. Failure scenario: the sentence is cited as the measured cause when the D2-natural families and the G v2 key work are chosen, but this root cannot separate parse, read and emission failures, and a wrong choice there costs a fit. Fix: say "consistent with, not shown by", or add the trace (N5) and re-run.

**3. Line 8 says "sieve reads".** The module doc (7-8) lists "semiprime-expert keys, sieve reads, the learned write trigger". #1487's Required 1, fixed at `a8f890b2`, established that `ReadPolicy::Sieve` changes clause formation for reads and writes and that only the written-expert preference is read-only; this reintroduces the wording. Failure scenario: a later reader credits a Store-over-Silent gain to sieve reads, when a write is also rescued (turn 2 "Now Molly's pet is a cat." with Molly tagged other is stored only under the sieve; test `the_sieve_rescues_a_write_whose_entity_was_not_tagged`). Fix: "sieve clause formation (reads and writes)".

## Non-blocking

- **N1. Untested paths.** `memory_greedy_reply` and `memory_reply_panel` have no test, and the caller-EOS branch never ran in the root: 0 of the 120 non-final replies (20 per cell x 6 cells) needed it, and all six non-EOS stops are final turns. Its span (256-261) is verified only by reading; my control below and the re-runs did not reach it either (every non-final reply in them ends in EOS). A tiny random-init `AermModel` (the config of `stack_aerm`'s `the_memory_branch_starts_silent_and_the_split_matches_the_stack`) could pin three things: Silent equals `greedy_reply` on a zero-initialised branch; the panel's keys equal `reply_panel`'s; a forced no-EOS reply gets an EOS inside its span.
- **N2. Positive control (independent, scratchpad only).** I ran the file at the root's `argv[0]` (`BuildCaches/claude/release/examples/memory-replies`, SHA-256 `248af855…`, mtime 01:12:27, 19 s before the claim) on 11 requests I wrote in D2's training templates ("Note that Bob's pet is a cat." / a chit-chat turn / "What is Bob's pet?", one update, one absent-key control); a debug build of the head gave identical reply ids on aerm-s1. I reused `dev-mem-01..10` as ids only so that the tool's fixed key would not abort; its printed 0/10 means nothing for this panel. `max_new_tokens=16`, aerm-s1 and aerm-s2: **Store answers 10/10** ("It is a cat.", "It is green.", ..., "It is a duck." after the update) **and abstains on the absent key**; Silent answers none of the ten on either checkpoint (non-answers or abstentions). So tags, sieve, registers, `top`, the per-step recompute and the marks across three turns work end to end, and the 0/10 on the natural panel is not an instrument fault. These are D2 templates, so this shows the path, not that any natural phrasing works. Consider committing such a panel with its expected answers, because 0/10 in both arms alone cannot separate "cannot recall" from "the instrument cannot see recall".
- **N3. Fail fast, bind the panel.** `score_memory` errors if any of the ten ids is missing (295), but it runs after each generation (`memory-replies.rs:126`), and the report is written only at the end (154-157). A run over another panel therefore generates everything, aborts, and keeps only `error.json`. A same-named file exists: `dialogue-prefix-witness-inputs-1/development-requests.json` (3 requests, SHA-256 `e572344d…`) has `dev-mem-01` but not `dev-mem-02`. Conversely a panel that reuses the ids with other content is scored against this key silently. Check the ids (and the SHA-256 `81268b51…`) right after `load_requests`, before any model loads. That matters most for the integration run, where a wasted pass costs model-slot time.
- **N4. Bind the code and the key.** The root's `attempt.json` records `argv`, but the report has no executable hash and no key. This is #1487's N4, which `prime-route-eval` fixed with `executable_sha256` (compute it before the evaluation). Embedding the ten accept/reject lists, or their hash, makes the root self-describing.
- **N5. Attribution.** Record per request the store events (writes fired, reads taken, register status at the final position, tags on the entity and relation words), as #1487's trace does, so that a future 0/10 can be assigned to tags, trigger, read or emission. This is also the evidence Required 2 lacks.
- **N6. Silent is not the branch-free stack.** Lines 9-11 say "which for a zero-initialised branch is the plain stack". At initialisation only: `registers_side` adds the status row for every status including None (`stack_aerm.rs:1496-1499`), and in aerm-s1 the trained `memory.status` row 0 is non-zero (largest magnitude 0.083 over 128 values). Silent is the same weights with all registers None, which is what text training used, so it is a valid within-model control; say that, and do not compare it with S2's recorded panel as if the branch were absent.
- **N7. Library guard.** Only the example rejects a model without a memory branch (107-111). `top` ignores registers without one (`stack_aerm.rs:1683-1686`), so a library caller gets Store equal to Silent silently. Move the `has_memory()` check into `memory_reply_panel`.
- **N8. Class line.** The body says "Class A instrument plus a baseline result". Under `docs/integration/agent-execution-policy.md` ("Standing merge and review criteria") a measured number is class B. This review's reproduction of the sealed numbers (check 6) is the independent pass that class asks for; say "A + B" so the record matches.
- **N9. Nits.** The panel JSON writes `"arm": "store"` (serde snake_case) while the arms map keys are `Store`/`Silent` (Debug); pick one. `#[allow(clippy::too_many_arguments)]` on `memory_greedy_reply` (7 arguments) is unneeded. The reject list is matched as whole words (the frozen sentence says "whole word"; "bluish" would not reject), which `MemoryAnswer`'s doc could state, and a test could assert the key strings are lowercase. "This is the floor" is the metric's lower bound; the baselines pre-registered on #973 are S2 (0/10) and the native child (Alex, green, partly Tokyo), and this root is neither.

**Cost of this pass.** Test-profile build 15 min, example build 4 min, tests 84 s, and about 10 min of re-runs on the debug binary, all at nice 10 and 2 threads under a load average above 20. New storage: 5.5 GB of debug build cache in `BuildCaches/claude-rerun/debug` (left in place for reuse; safe to delete) and about 100 KB of reports in my scratchpad. The detached worktree is removed.

Not done: no review of the frozen key's substance beyond equality with the comment, of G-binding, or of the arm design.


### comment 5884914047 — 2026-09-29T06:28:05Z — Casey-allard

Independent re-check (Sonnet pass launched by Lab 1, non-author)

**Verdict: APPROVE at `5e457245`.** The three required fixes from comment 5884885717 are resolved. Scope: `git diff bd5576d6 5e457245` and the updated PR body; read-only, no cargo, no Python, no test or evaluation re-run.

- **Doc line (Required 3): resolved.** One commit, one file, +3/-2, every changed line a `//!` line; line 8 now says "sieve clause formation for reads and writes". Outside the `//!` block the file is byte-identical to `bd5576d6`, no other file changed, `git diff --check` is clean and `rustfmt --check` on the new file exits 0. My 4/67 test results and example build at `bd5576d6` therefore carry over.
- **Baseline replies (Required 1): resolved.** The body now describes each checkpoint. Re-read against the sealed root: s1 "Okay." then "I do not know." (9 of 10 questions); s2 "That sounds nice." then "I do not know." (7) or "It is that it." (2, plus one "It is that."); s3 "That sounds nice."/"Got it." then story-like text (the quoted reply is in the root).
- **Mechanism (Required 2): resolved.** "Why is not measured ... records no store events" is what the root supports. The positive-control paragraph matches my N2 and is attributed to my independent run, not to a Lab 1 sealed result.

Non-blocking leftovers: the body still does not state that Store and Silent replies are identical on all 38 requests at all three checkpoints (true in the root); s1's mem-09 answer is "Okay."; "This is the floor" (N9), the class line (N8) and N1-N7 remain as follow-ups. Approval is for the exact head `5e457245`; a changed head needs a fresh look.



---
_Comment/review ids in this file: 5884885717, 5884914047_

