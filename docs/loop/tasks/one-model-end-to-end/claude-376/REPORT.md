# REPORT: one-model-end-to-end, claude-376 (V0: the pure-vector baseline)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [ORDER.md](ORDER.md). PR #3970. Follows claude-374 (PR #3965) and the 14 Nov audit. This is stage V0 of its roadmap.

The order asked how much of the served model's replies come from the vector state alone. claude-358's three saved `p` models were served on the laptop over claude-374's panels P14 and P23 (2,218 rows) in two settings:
- **V:** the artifact's exact turn memory dropped;
- **VM:** the artifact as exported.

Neither setting uses the counted asks table, serve-again or the pair experts. There was no bar, so the notebook word is **inconclusive**, a number for the later stages to read against.

**The answer, at the scope of these panels:**
- **Exact replies.** The vector state alone gives about half: V gets 171, 173 and 211 exact replies of 2,218, and VM gets 377, 380 and 441.
- **Told values.** Read by whether the reply ends with the told value, the vectors give 75%, 96% and 43% of what the memory gives.
- **Facts.** Almost every fact the model gets right comes from the turn memory's table: legs, colour, sound, opposite.

## What did not work, first

1. **Without the turn memory, the model knows almost no facts.** Fact and number rows that ended with the answer word, P14 + P23, of 188: V 20, 17 and 11; VM 64, 61 and 54. Exact: V 14, 11 and 9; VM 32, 31 and 26. By kind, answer word, V then VM, summed over P14 + P23, seeds 1/2/3:

   | kind | V | VM |
   |---|---|---|
   | legs | 1 / 0 / 0 | 21 / 15 / 16 |
   | colour | 0 / 1 / 0 | 9 / 9 / 8 |
   | sound | 2 / 2 / 2 | 13 / 12 / 13 |
   | opposite | 0 / 0 / 0 | 4 / 3 / 3 |
   | capital | 1 / 1 / 1 | 3 / 5 / 2 |
   | bigger | 11 / 10 / 7 | 11 / 13 / 11 |
   | day after | 5 / 3 / 1 | 3 / 3 / 1 |
   | add | 0 / 0 / 0 | 0 / 1 / 0 |

   Only "bigger" and "day after" are answered as often by the vectors. Replies, P14, seed 1 (V, then VM):
   - "What's the capital city of Kenya?" → "Your name is and follow you you you you …" / "The capital of Kenya is is is is i".
   - "What is the opposite of tall?" → "That's great news about the opposite of of of of …" / "The opposite of tall is the opposite of of of …".
   - "Which day is after Thursday?" → "The day after Thursday is Friday." / "The day after Thurday." Here V is right and VM is wrong.
2. **The turn memory answers the same rows whatever the trained weights are.** With it on, several kinds score the same at all three seeds, exact, P14 + P23:
   - goodbye 32 / 32 / 32 (of 89);
   - first letter 9 / 9 / 9 (of 25);
   - greeting 4 / 4 / 4;
   - news of a thing 2 / 2 / 2.

   Without it, these vary by seed: goodbye 13 / 9 / 9, first letter 0 / 0 / 0. The memory is an exact table keyed on the turn's ids and counted from TRAIN. Equal counts at three seeds are consistent with it replaying the same stored reply on those rows; this unit did not read which reply the table holds. Replies, P14, seed 1 (reference, V, VM):
   - "Good night!": reference "See you later!"; V "Goodbye! Have a great day."; VM "See you later!".
   - "Thank you so much!": reference "You're welcome!"; V "Anytime!"; VM "You're welcome!".
   - "Tell me about yourself.": V "I'm a helpful assistant that runs on your computer."; VM the reference word for word.
3. **The memory never says "you haven't told me".** On the 163 rows whose reference says the value was never told, VM says so on 2, 1 and 3. V says so on 5, 17 and 87; seed 3's vectors learned that reply. Replies with the memory on, P14, seed 1:
   - "What is my lucky number?" → "It is 8stststststst…";
   - "What's my code word?" → "The code word is lucky for you.";
   - "What's my lucky number?" → "Your lucky number is 4 place.".
4. **Most of the exact gap on told values is wording, not the value.** On the 1,240 told-value rows, VM is exact on 204, 207 and 205, and V on 87, 110 and 40. Counted by whether the reply ended with the value word, by `subst-374.sh`'s rule, VM gets 518, 513 and 530, and V gets 389, 493 and 229. So at seed 2 the vectors find the told value almost as often as the memory does (493 against 513). The memory then adds the stock wording: V "It's numjast." against the reference "It is numjast." (both have the value).
5. **Both settings loop about as often.** Replies that did not end within 48 tokens, of 2,218: V 226, 221 and 150; VM 245, 210 and 161. A V loop on a told value, seed 1, P14: "What is my name?" → "Your name is justouldouldelant is justouldouldelant …". VM answers that row "You're welcome!".
6. **The timing is noisy.** The laptop was shared with other work; the load average reached 53 during seed 3. VM on seed 2 read prompts at 2,698 ids a second against about 4,400 in the other five runs, and that comes from the load. The re-timing below puts seed 2 back with the others. With the memory on, served tokens a second are 7% to 11% higher, not lower.
7. **Not run.** No model was trained without a turn memory: these weights were trained with it dropped from half the windows (`--turn-memory-dropout 500`), so V serves the same weights with the memory dropped, not a model trained without one. No pair experts, no counted asks table and no whole-conversation prompts (claude-367 read with all three). The panels are window prompts, as claude-374 scored them in float.

## What changed

- **`docs/labs/one-model-end-to-end/instrument/src/bin/raw-id-replies.rs`:** `--turn-memory 0|1` (default 1).
  - `0` calls `IntegerStackModel::drop_turn_memory` before serving, the call `uor-chat --no-turn-memory` makes, and refuses an artifact with no turn memory.
  - `--mark-reply 1` stays allowed with `0`. `StackChat::feed` marks every reply, and the copy heads read the reply opening (claude-356).
  - The receipt gains `"turn_memory_dropped": true`.
  - Test `turn_memory_zero_serves_the_artifact_with_its_memory_dropped`:
    - flag 1 gives the same replies and receipt as no flag;
    - flag 0 gives, for each row, the replay of a model whose memory was dropped;
    - some row's reply moves;
    - an artifact with no memory is refused, and so is a value other than 0 or 1.
  - With the drop call taken out (the refusal kept), the test fails on the replay comparison.
- The unit was renumbered from claude-375 while the reads ran: the docs session's PR #3969 had taken that number first. Nothing else in the unit changed.
- No recipe, model, panel, bar or other lab's file is changed.

## Measurements

**Models and artifacts.**
- claude-358's `p-s1`, `p-s2` and `p-s3` were fetched from `results/claude/one-model-end-to-end-copy-longest-2026-11-12/run/p-sN/try-1/`. They match their SHA256SUMS: tarballs `916a97e5…`, `d0e7b0d7…` and `17678d48…`.
- Their run config names the `p` recipe with the place read and its pull. It names seed N, 16,384 steps, and TRAIN `e25775f3…5f32`.
- They were exported at this unit's head with `fbs-export --legacy-artifact --raw-gates --raw-transition --run-config snap/config.json --context 8256`, as claude-366 did. Each artifact is 19,345,344 bytes, the size claude-366 and claude-367 served. SHA-256:
  - p-s1 `9a209883ff74cdeb6936fc11714d4dd4bb63947775609754c6a5f678e39df9bd`;
  - p-s2 `f4667934627e99d709533f30505058fdf07912ddf3cfc9ade6ded5f415e8c680`;
  - p-s3 `9615060f4db98a4cc5b21220e454428b816cda1d47256cf85091963692e83d20`.

**Panels.**
- P14 has 1,108 rows and P23 has 1,110 rows: claude-374's `pre` stage `samples-p-greedy-memory.json`.
- Their `[prompt, reference, position]` hashes are `6944401c8b70fac2` and `b40ce94f0ce85fff`, claude-374's.
- Each row was served greedy, 48 tokens, reply marked, with `--time-steps 1`. Each model's V then VM ran one after the other, one process at a time.

**Scoring.**
- Exact is `read-374.sh`'s rule: the reply equals the reference through its end-of-reply id.
- Fact and number rows use `subst-374.sh` (`8384c472…b791b691`), run as `subst-376.sh` with only the run names changed.
- The told-value and never-told counts use the same turn-kind rules, with the answer word taken by the same rule.

| model | setting | exact of 2,218 | ended | fact rows, answer word / exact, of 188 | told values, value word / exact, of 1,240 | never told, said so, of 163 |
|---|---|---|---|---|---|---|
| p-s1 | V | 171 | 1,992 | 20 / 14 | 389 / 87 | 5 |
| p-s1 | VM | 377 | 1,973 | 64 / 32 | 518 / 204 | 2 |
| p-s2 | V | 173 | 1,997 | 17 / 11 | 493 / 110 | 17 |
| p-s2 | VM | 380 | 2,008 | 61 / 31 | 513 / 207 | 1 |
| p-s3 | V | 211 | 2,068 | 11 / 9 | 229 / 40 | 87 |
| p-s3 | VM | 441 | 2,057 | 54 / 26 | 530 / 205 | 3 |

**Exact by kind** (P14 + P23; V s1/s2/s3 then VM s1/s2/s3; the kinds are `subst-374.sh`'s, applied to every row):

| kind | rows | V | VM |
|---|---|---|---|
| told value | 1,214 | 80 / 108 / 38 | 194 / 198 / 195 |
| repeat words | 212 | 13 / 10 / 90 | 17 / 18 / 84 |
| never told | 163 | 4 / 8 / 26 | 0 / 0 / 0 |
| fact | 150 | 4 / 3 / 2 | 22 / 21 / 17 |
| thanks | 116 | 17 / 15 / 13 | 32 / 34 / 34 |
| other | 98 | 24 / 8 / 18 | 46 / 44 / 45 |
| goodbye | 89 | 13 / 9 / 9 | 32 / 32 / 32 |
| number | 67 | 10 / 8 / 7 | 11 / 11 / 10 |
| first letter | 25 | 0 / 0 / 0 | 9 / 9 / 9 |
| greeting | 21 | 3 / 2 / 5 | 4 / 4 / 4 |
| a feeling | 18 | 3 / 2 / 2 | 8 / 7 / 8 |
| news of a thing | 18 | 0 / 0 / 1 | 2 / 2 / 2 |
| sentence with a word | 17 | 0 / 0 / 0 | 0 / 0 / 1 |
| list N | 6 | 0 | 0 |
| spell a word | 4 | 0 | 0 |

**Beside claude-367's pair experts.** That unit read the same three models on a different, 239-row everyday panel with the turn memory always on. Good replies were 137, 135 and 134 without the experts, 170, 171 and 170 with them held out, and 191, 193 and 194 as served. So the order of what each part adds, on its own panel, is:
- the vector state: this unit's V;
- the turn memory: about doubles exact here;
- the counted pair experts: about 40% more on 367's everyday rows.

The two panels are not the same rows, and the numbers are not added together.

**Speed** (`step_time` from the receipts, P14 + P23, 251,059 prompt ids read and 30,106 to 33,877 reply tokens served a run; load average at the start of the run):

| model | setting | prompt ids/s | served tokens/s | load |
|---|---|---|---|---|
| p-s1 | V | 4,510 | 1,448 | 6.2 |
| p-s1 | VM | 4,421 | 1,602 | 6.4 |
| p-s2 | V | 4,382 | 1,420 | 4.9 |
| p-s2 | VM | 2,698 | 924 | 4.6, rose during the run |
| p-s3 | V | 4,184 | 1,459 | 53.4 |
| p-s3 | VM | 4,221 | 1,617 | 18.8 |

**Re-timed: seed 2, P14, V and VM alternated twice, after the reads** (load average 3.8 to 5.4; every re-timed reply equals the read's, id for id):

| run | prompt ids/s | served tokens/s |
|---|---|---|
| V, 1st | 4,482 | 1,437 |
| VM, 1st | 4,398 | 1,597 |
| V, 2nd | 4,209 | 1,367 |
| VM, 2nd | 4,112 | 1,469 |

So seed 2's slow VM run was the machine. In every pair where the load held, VM serves 7% to 11% more tokens a second than V, and prompt reading is within 3%. This unit does not explain why the setting with more work per step is faster per token. That is a measurement only.

## Checks run at the head

| Check | Result |
|---|---|
| `cargo test --release --locked --manifest-path docs/labs/one-model-end-to-end/instrument/Cargo.toml --features integer-reader --bin raw-id-replies` | 18 passed |
| Negative gate: the drop call taken out of `raw-id-replies.rs`, the new test run | fails on the replay comparison; restored, 18 pass |
| The base reader (`origin/main`'s `raw-id-replies`, SHA-256 `df76dc15…1747`) on the six VM reads | all six VM reads equal the head's, id for id: the default is unchanged on the real artifacts |
| The head reader rebuilt after the renumber's comment edit | SHA-256 `f9665abd…9cce`, the binary that served every read |
| Panel identity hashes | P14 `6944401c8b70fac2`, P23 `b40ce94f0ce85fff`, claude-374's |
| Every read's exit | 0 (12 of 12) |

Binaries: `raw-id-replies` `f9665abd7fe3a6c26eed2787247bbd4151ae80291b8f599c9f3d35a4fb469cce` and `fbs-export` `6289f49885b31a3b2efdeb407711fe2ca8454f6be7d4f74e3fe2332a6e2d69e2`, built at `5bde4a297df0b17983c9d6ffbf7afa4f281bf8bc`.

## Skills used

None. The kit scripts are `local/claude/claude-376/serve-376.sh` and `rows-376.sh`, with `subst-376.sh`, whose only change from claude-374's is the run names.

## Still open

- **A model trained with no turn memory.** V serves weights that were trained with the memory present in half the windows. The vector state of a model trained without one may be better or worse; this unit does not measure it.
- **The same reads through `uor-chat` with the whole conversation as the prompt.** claude-367's chat setting; here the prompts are the 256-token windows the float score read.
- **A second scoring of told values that accepts wording.** The value-word count above is a description, not a bar.
- **For the roadmap.** The facts sit in the exact table, not in the vectors. This is where V3, knowledge in served vector rows, would have to land. The turn memory's lack of a "not told" answer is a defect of the table, not of the vectors.

## Plain summary

We served the same three trained models twice on the laptop: once using only their learned vectors, and once with their exact memory table switched on as well. With the vectors alone, a reply matches the expected reply word for word about half as often (171 to 211 of 2,218, against 377 to 441). Much of that gap is wording: the vectors often find the told value, say "It's numjast." where the panel wants "It is numjast.", or pick another polite phrase. But for facts like how many legs an animal has, almost every right answer comes from the table, not from what the model learned. The table also never says "you haven't told me"; the vectors sometimes do. Prompts are read at about 4,400 tokens a second either way; replies come out at about 1,450 tokens a second from the vectors alone and about 1,600 with the table, which is slightly faster.
