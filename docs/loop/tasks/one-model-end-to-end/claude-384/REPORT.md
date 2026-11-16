# REPORT: one-model-end-to-end, claude-384 (V3: serve the pair rows as integer rows)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [ORDER.md](ORDER.md). PR #3983. Follows claude-370 (PR #3961), claude-373 (PR #3964), claude-374 (PR #3965) and claude-376 (V0, PR #3970). This is stage V3 of the 14 Nov roadmap.

The order asked for claude-370's pair rows (`--pair-rows 2`: 187,413 rows of 256, one per unordered pair of different tokens seen in at least 2 TRAIN user turns, addressed by the product of the two tokens' primes) to reach the integer artifact as i16 rows at 2^-12, and for the served stack to read them. Then it asked whether claude-370's float gain survives serving.

**The bar reads "Served":** over P14 + P23 (2,218 rows), served exact replies go from 423 (S-b) to 524 (S-pair) on seed 2 and from 409 to 516 on seed 3. G_s is 4.69 points against a line of 2.3, and each seed's gain is above zero. claude-370 measured 4.60 points in float on the same panels (431 to 527, 413 to 521): the served gain is the float gain. The replay instrument bar passed first: the served picks equal the trainer's at every one of 253,277 prompt positions, and at the last position on 4,436 of 4,436 prompts, as raw row numbers and raw addresses.

This is a reading on panels and in one nine-turn conversation. It is not a milestone claim, and the row values are an F32 to Q12 export gap, not D85's native path.

## What did not work, first

1. **The first served reads used a rule the trainer does not use.** The trainer's reply runs to the first token of the next user marker, so it adds no rows while that marker is being written; the first served reader kept the rows until the whole marker had arrived. The unit's own review found it. Checked at every prompt position (`pair-positions`, below), the first rule disagreed with the trainer at 12,088 positions a seed (5,986 on P14, 6,102 on P23), every one at the first or second token of a user marker (`[55, 2728, 28]`). The last-position replay (4,436 of 4,436) could not see it, because a prompt ends on the assistant header. The fix (commit `72229241c`) closes the reply while the stream ends with a beginning of the user marker; every S-pair read was served again with it (serve-2), and every number below is serve-2's. The fix moved 25 of 4,436 S-pair replies (9, 10, 4 and 2 by read), cost seed 3 one exact reply (517 to 516) and changed one conversation turn. The first reads are kept in the kit as `serve-1`.
2. **The facts the rows were first aimed at are still wrong.** Fact and number rows (188) exact: 43 and 46 with the rows, 33 and 27 without. Capital: 0 and 1 of 34 exact with the rows, 1 and 0 without. Adding: 0 of 21 in every artifact. What moved is legs (11 and 11 of 29, against 4 and 8), colour and day after; claude-370 saw the same in float.
3. **The served conversation is not better on values.** In claude-356's nine-turn script, with pair experts off, S-pair answers "What is druvalk?" (told 73) with "That's 66." on seed 2 and "That's 666" (a short cycle) on seed 3; S-b answers "Your cake 18." and "That's 65.". Seed 3's S-pair then breaks: the cycled reply stays in the conversation, and the next two replies run into a made-up user line ("That's 66 / User: O spellucuc remember remember remember"). S-b seed 3 does not break this way. "What is my name?" (never told) is answered "You're welcome!" by three artifacts and "You're." by S-pair seed 2 (it was "You're welcome!" under the first rule).
4. **The model leans on the rows, served as in float.** Served with the rows dropped (S-cut, claude-370's I-8), exact falls to 322 and 316: about 100 below the plain `b` models, and replies that did not end fall from 2,058 and 2,039 to 1,817 and 1,593. A served artifact whose rows are lost or damaged is worse than one trained without them.
5. **The artifact is six times larger.** 116,050,880 bytes with the rows against 19,345,344 without: the rows add 96.7 MB (187,413 × 256 × 2 bytes of values and 187,413 u32 addresses). That is the cost the next step (compression: rows shared by the pair's E8 address, or a higher MIN) has to cut. Compression did not run in this unit.
6. **Speed is not settled at the same moment.** The serve-2 reads ran at 1,505 to 1,648 served tokens a second with the rows and at 1,427 (S-b) and 1,415 (S-cut) for the seed-2 P14 check reads, minutes apart under a falling load (42 to 8). The serve-1 reads, under a load of 11 to 35, ran at 177 to 1,123. No read timed S-pair and S-b side by side at the same moment.
7. **The row values do not meet D85's default path.** They were trained in F32 by claude-370 and rounded once to Q12. The rounding is at most half a step (1.221e-4) and the served replies agree with the float replies about as often with the rows as without them (below), but this is historical float/export evidence: the values are not trained in the served representation.
8. **Overlapping marks were let through until the fix's reviews.** A header holding a beginning of the user marker after its first token, at its end (user `[4, 7]`, header `[2, 4]`) or whole inside it (user `[1, 9]`, header `[2, 1, 9, 3]`), would hold a reply back or open a turn inside a header the scan takes whole; the per-position checks would have hidden the first and called the second a disagreement only on marks they never draw. `PairMarks::check` now refuses such marks, in the trainer and in the reader. The production marks pass it (the header `[201, 35, 560, 652, 714, 28]` ends with `28`, not `55`), so no read changed; the refusal came after serve-2's binaries were built.

## What changed

- **One shared operator for the picks** (`crates/uor-r4-integer/src/fbs_pair_rows.rs`, new, declared in both integer crate roots). The prime address of a token pair, the dictionary (ascending u32 addresses, checked), the marks (user marker and assistant header ids, refused when one begins the other or the header ends with a beginning of the user marker), the turn scan, `open_picks` (the rows of the open turn's pairs, sorted and unique, or none when no turn is open, the turn has no rows, or the stream ends with a beginning of the user marker after the reply position, where the trainer's reply has already ended), the streaming `PairTracker` the served session feeds token by token, the Q12 rounding (`quantize_q12`, refusing any value outside i16) and the plain-data `PairRowsImage` with its rows file (magic, sizes, order and digest checked). The float trainer's `pair_rows.rs` now calls this module for every pick and every address; it has no picks of its own.
- **Snapshot** (`fbs-snapshot`). A checkpoint with pair rows is no longer refused. Weighed or boosted rows (claude-373, claude-374) and rows beside address rows are still refused. The rows are rounded to Q12 into `{out}.pair-rows`, with a census line and a note in the snapshot.
- **Artifact** (`fbs-export --pair-rows ROWS`). The rows file is added as a UORLUT01 table under `lut-stack/15` (`add_pair_rows_uorlut01`), so a reader from before refuses the artifact by schema instead of serving it without its rows. The flag needs the snapshot's note and refuses an address table beside it. Pair rows are also refused beside any native Q31 record (`q31`, `q31_lanes`, `q31_read`), when either is added and when a header names both; `pair_rows_are_not_served_with_a_q31_read` pins it, and it fails with the rule set aside. The served measurements below were made under `lut-stack/12`, before `main` gave that number to the native Q31 state (codex-1) and took `/13` and `/14` too; the merge moved pair rows to `/15`, and the artifacts were exported again and served again (below).
- **Served read** (`stack/pair_rows.rs`, `stack/session.rs`). The model loads the rows from the artifact; the session carries a `PairTracker`, reset and restored with the rest of its state. At layer 0, after the Hamilton block and any address rows, before the Hamming read, the bag (the sum of the picked i16 rows, shifted by 4 into the i32 residual at 2^-16) is added at every step whose turn is open with picks. Both the plain and the timed fbs steps do this.
- **Replay instrument.** `pair-picks` (float crate, new) writes the trainer's plan for each panel prompt from the checkpoint's own dictionary. `pair-positions` (float crate, new) compares the trainer's plan with the served tracker at every position of every panel prompt, from the rows file alone. `raw-id-replies --pair-rows 0|1` names each reply's picks and addresses, counts reply steps whose picks moved, and drops the rows with 0.
- **Lockfile.** The instrument's `Cargo.lock` gains `uor-r4-native-vector`, which `main` gave the integer crate without refreshing it.

## Measurements

**Models and snapshots.**
- claude-370's four step-16,384 checkpoints on `wick` (`/workspace/out/c370/try-1/{pair-s2,pair-s3,b-s2,b-s3}`).
- Snapshotted on `wick`'s RTX 4090 by `fbs-snapshot` built at `bd66ff21d3b8b2db0f82e65a9b54859f55d11f2b` (binary `e7e99bb3…6d74`), an export step beside the V1 session's run. Census:
  - pair-s2: largest |value| 2.246155, largest rounding 1.221e-4, 344,040 zeros of 47,977,728; rows file `5d35efc6…2e37`;
  - pair-s3: largest |value| 2.150860, largest rounding 1.221e-4, 342,295 zeros; rows file `210d6cbb…a8a6`.
- Exported on the laptop with `fbs-export --legacy-artifact --raw-gates --raw-transition --run-config <snap>/config.json --context 8256`, the pair models with `--pair-rows`. SHA-256 and bytes:
  - pair-s2 `8f804dcc9f8504958308818a0940d4591aeb76a0ac88e9bf0f6313df77580db6`, 116,050,880;
  - pair-s3 `5f6e95f5c35cc736ba06b0ac231f0dc2f64a961bf79ad4e80e078ca506dbfd7e`, 116,050,880;
  - b-s2 `6ee0efe59c1276715140a4ebf2a10e8efe0813b41a8fb8e2be817e66e9bb2469`, 19,345,344;
  - b-s3 `793a30244e1b53eb63e2b8bbd6a085ff56f1c855a0d553fbb87ae275ad2992e9`, 19,345,344.
- Those pair artifacts declare `lut-stack/12`. After the merge of `main` (a3c31a227) the four snapshots were exported again with the same flags (`serve3-384.sh`, log `serve-3/`). The b artifacts are byte for byte the ones above. Each pair artifact differs from its `/12` one in one byte, header offset 851, the schema's `2` become `5`: pair-s2 `41c6ff461fce47662917c928ea837679745aec8989d3806ff74d7b97564dc444`, pair-s3 `2a9f8ec2be833a5b0eb4e4ef8687b3f5e64082987f518d21835d64c42d00c38c`. Served again by `raw-id-replies` built at a3c31a227, every reply of the four S-pair reads (4,436), of S-b seed 2 P14 (1,108) and of S-cut seed 2 P14 (1,108) equals serve-2's id for id, and every read exits 0. The numbers below are serve-2's and hold for the `/15` artifacts.

**Panels.** P14 (1,108 rows) and P23 (1,110 rows), claude-374's `samples-p-greedy-memory.json` (`33bdd07a…178d`, `ddccde6f…7d99`), identity hashes `6944401c8b70fac2` and `b40ce94f0ce85fff`, checked by `rows-384.sh` before scoring. Every row served greedy, 48 tokens, `--mark-reply 1`, turn memory as exported, no asks table, no serve-again, no pair experts, `--time-steps 1`, one process at a time.

**The replay (instrument bar, passed).** Two checks, both as raw u32 row numbers.

*At every prompt position* (`pair-positions`, float crate, new): the float crate's own `pair_rows::plan` of each whole prompt (one window, as the scoring pass reads it) against the shared `PairTracker` fed the prompt token by token, as a served session is. The counts are the same at both seeds, because the picks read only the dictionary and the marks:

| panel | positions | with rows | agree | disagree | broken beginnings | first rule: disagree |
|---|---|---|---|---|---|---|
| P14 | 125,221 | 40,245 | 125,221 | 0 | 0 | 5,986 |
| P23 | 128,056 | 40,751 | 128,056 | 0 | 0 | 6,102 |

A broken beginning would be a position where a beginning of the user marker is not finished by the text after it: there the trainer adds the rows and a causal reader cannot know to. The panels hold none; the random-stream test draws them and pins the rule there.

*At each prompt's last position* (`pair-picks` on `wick` against the serve-2 reads of `raw-id-replies` on the laptop, prompt by prompt):

| run | panel | prompts | rows agree | addresses agree | with picks |
|---|---|---|---|---|---|
| pair-s2 | P14 | 1,108 | 1,108 | 1,108 | 1,108 |
| pair-s2 | P23 | 1,110 | 1,110 | 1,110 | 1,110 |
| pair-s3 | P14 | 1,108 | 1,108 | 1,108 | 1,108 |
| pair-s3 | P23 | 1,110 | 1,110 | 1,110 | 1,110 |

In every served read the picks held from a reply's first id to its end (`picks_moved` 0).

**The export gap.** Served replies equal to claude-370's float scoring-pass replies, id for id (the float reply cut after its first end mark):

| model | P14 | P23 | of 2,218 |
|---|---|---|---|
| pair-s2 | 962 | 962 | 1,924 |
| pair-s3 | 971 | 989 | 1,960 |
| b-s2 | 943 | 953 | 1,896 |
| b-s3 | 907 | 927 | 1,834 |

The artifacts with rows agree with float at least as often as the ones without (87–88% against 83–85%), and the fix raised both pair counts by 4. The Q12 rounding of the rows adds no gap that this count can see beyond the trunk's own export gap.

**The number.** Exact is `read-374.sh`'s rule; fact and number rows are `subst-374.sh`'s, run as `subst-384.sh` with only the run names changed.

| model | setting | exact of 2,218 | ended | fact and number rows, exact / answer word, of 188 |
|---|---|---|---|---|
| pair-s2 | S-pair | 524 | 2,058 | 43 / 85 |
| b-s2 | S-b | 423 | 1,960 | 33 / 69 |
| pair-s2 | S-cut | 322 | 1,817 | 23 / 49 |
| pair-s3 | S-pair | 516 | 2,039 | 46 / 86 |
| b-s3 | S-b | 409 | 2,048 | 27 / 56 |
| pair-s3 | S-cut | 316 | 1,593 | 20 / 43 |

Gains: +101 (s2) and +107 (s3); G_s = 104 / 2,218 = **4.69 points**, each seed above zero: **Served**. Float (claude-370): +96 and +108, 4.60 points. Under the first rule: 524 and 517, 4.71 points.

Rows won and lost, S-pair against S-b of the same seed: seed 2 won 66 + 67 and lost 18 + 14 (P14 + P23); seed 3 won 78 + 64, lost 14 + 21. Most won rows are "Say exactly this", "Copy these words" and "Please repeat" rows said back in full, as in float.

**Fact and number rows by kind** (P14 + P23, exact / ended with the answer word):

| kind | of | pair-s2 | pair-s3 | b-s2 | b-s3 | cut-s2 | cut-s3 |
|---|---|---|---|---|---|---|---|
| capital | 34 | 0/5 | 1/4 | 1/6 | 0/2 | 0/0 | 0/2 |
| colour | 19 | 8/13 | 8/13 | 5/9 | 4/8 | 4/8 | 5/9 |
| day after | 28 | 3/9 | 5/12 | 3/9 | 0/1 | 0/1 | 0/1 |
| legs | 29 | 11/27 | 11/27 | 4/16 | 8/19 | 4/14 | 0/6 |
| opposite | 6 | 5/5 | 5/5 | 5/5 | 0/0 | 0/0 | 0/0 |
| sound | 28 | 6/11 | 6/12 | 6/12 | 6/13 | 6/14 | 6/13 |
| add | 21 | 0/1 | 0/0 | 0/0 | 0/1 | 0/1 | 0/1 |
| bigger | 23 | 10/14 | 10/13 | 9/12 | 9/12 | 9/11 | 9/11 |

**The conversation** (`uor-chat --stack ART --tokenizer <claude-370's> --script claude-356/chat-script.json`, pair experts off, turn memory as exported; `uor-chat` `540cd1fa…ec34` for S-pair, serve-1's `4e19b04c…6603` for S-b, which does not read the tracker):

| turn | user | pair-s2 | b-s2 | pair-s3 | b-s3 |
|---|---|---|---|---|---|
| 1 | Keep these in mind: druvalk is 73, mibtorn is felsharp, quonzet is 18. | Got it. | Got it. | Got it. | Got it. |
| 2 | What is mibtorn? | Mibtorn is felsharp. | Mibtorn?? | Mibtorn is felsharp. | Mibtorn is felsharp. |
| 3 | What does a cow say? | A cow says moo. | A cow says moo. | A cow says moo. | A cow says moo. |
| 4 | Say exactly this: trelvon dask pimmerholt zane | trelvon dask pimmerholt zane | the line, then "Assistant: trelvon dask pimmerholt zane" six more times to the token limit | trelvon dask pimmerholt zane | trelvon dask pim |
| 5 | What is druvalk? | That's 66. | Your cake 18. | That's 666 (short cycle) | That's 65. |
| 6 | I'm tired today. | I'm tired today. | I'm tired today. | That's 66 / User: O spellucuc remember remember remember (short cycle) | I'm tired today. |
| 7 | Tell me what quonzet was. | It's 18. | It's 18. | That's 66 / User: Tell me what quonzet is 18. | It's 18. |
| 8 | What is two plus three? | 18. | That's 18. | 18. | Hibtorn is felsharp. |
| 9 | What is my name? | You're. | You're welcome! | You're welcome! | You're welcome! |

The rows' gain shows in the conversation where the panel shows it: the repeat turn is said back exactly by both S-pair seeds and by neither S-b seed. Values and arithmetic are wrong in all four.

**Speed** (`step_time` from the serve-2 receipts; 124,113 prompt ids on P14 and 126,946 on P23 a run, 15,112 to 19,078 reply tokens; load average at the start of each read; one process at a time):

| read | prompt ids/s | served tok/s | load |
|---|---|---|---|
| pair-s2, P14, S-pair | 4,021 | 1,533 | 42.3 |
| pair-s2, P23, S-pair | 4,299 | 1,648 | 23.3 |
| pair-s3, P14, S-pair | 3,792 | 1,505 | 15.4 |
| pair-s3, P23, S-pair | 4,131 | 1,593 | 13.2 |
| b-s2, P14, S-b (check read) | 4,032 | 1,427 | 9.0 |
| pair-s2, P14, S-cut (check read) | 3,895 | 1,415 | 7.8 |

The check reads equal serve-1's reads id for id (1,108 of 1,108 each); the other S-b and S-cut reads are serve-1's, which do not read the tracker. Under serve-1's load (11 to 35) the same reads ran at 177 to 1,123 served tokens a second, and four re-timed seed-2 P14 reads (S-pair and S-b alternated, the first rule's binary) ran at 542 to 880, the runs of one setting differing more than the settings. With the rows the reads are no slower than without them at this resolution; the cost per reply step is one gather and sum of the picked rows (a reply's picks are fixed for the whole reply). A side-by-side read at the same moment would settle it.

## Checks run at the head

Run on the laptop at `a3c31a22714f2802543df721192914903d559f49`, the merge of `main` and the last commit with code (`local/claude/claude-384/gates-head.sh`, log `gates-merge.log`; the run at `d3ec2fbc4` before the merge is `gates-head.log`):

| Check | Result |
|---|---|
| `cargo test -p uor-r4-integer --features serving-wasm --lib -- address pair_rows q31` | 58 passed |
| `cargo test -p uor-r4-integer --features serving-wasm --lib` (the whole library, after the merge) | 691 passed, 1 ignored |
| `cargo test -p uor-r4-integer --lib -- pair_rows address` (the default build) | 21 passed |
| `cargo test -p uor-r4-integer --features serving-wasm --bin fbs-export` | 6 passed |
| `cargo test -p uor-r4-float-train --lib -- pair_rows fbs_export` | 41 passed |
| `cargo check -p uor-r4-float-train --bin fbs-snapshot --bin pair-picks --bin pair-positions` | builds |
| `cargo check -p uor-r4-training -p uor-r4-cpu-train` (the integer crate's dependents) | builds |
| `cargo test --release --locked --manifest-path docs/labs/one-model-end-to-end/instrument/Cargo.toml --features integer-reader --bin raw-id-replies` | 19 passed |
| `rustfmt --edition 2021 --check` on the five new files | clean |
| `pair-positions` at both seeds, P14 and P23 | disagree 0, broken beginnings 0, exit 0 at both seeds |
| Negative gates for the fix: the close taken out of `open_picks` (`Some(rows)`); the overlap refusal taken out of `PairMarks::check`; the refusal narrowed to a header's end | the first fails `every_position_of_plan_reads_what_open_picks_names_of_its_prefix`; the second and third each fail `open_picks_reads_the_float_rule` and `marks_where_one_begins_the_other_are_refused`; restored, all pass |
| Negative gates, run when the served read was built (`bd66ff21d`): the layer-0 bag call taken out of the session; then the pair tracker's reset taken out of `restore_state` | each fails a `stack/tests/pair_rows.rs` test (the served-step test; the same-length restore in the picks test); restored, all pass |
| Refusals of a damaged rows file (length, order, magic, digest) | `fbs_pair_rows::tests::damaged_files_are_refused`; in the artifact, a short values table, a changed record hash and one changed value byte (`stack/tests/pair_rows.rs`) |
| The replay (above) | every position; 4,436 of 4,436 last positions |
| Panel identity hashes | P14 `6944401c8b70fac2`, P23 `b40ce94f0ce85fff`, claude-374's |
| Every serve-2 read's exit | 0 (6 of 6) |
| Negative gate for the merge's rule: the native Q31 refusals taken out of both adders and the parse | `pair_rows_are_not_served_with_a_q31_read` fails; restored, it passes |
| The `/15` artifacts served again (`serve-3/`) and the conversation run again (`chat-3/`) | 6,652 of 6,652 replies equal serve-2's id for id, every read exits 0; both pair models' nine replies equal `chat-2/`'s |

Binaries that served serve-2 and the conversation, built at `72229241c7e8f648ac9e5ab2d83f126c945090d2`: `raw-id-replies` `e44fbbc842542e75f4578e067a9b7dba8c8b41a7983c9ecb214bcd51716b39fd`, `uor-chat` `540cd1fa1d3eb7d1013be5776280577ac4fc36aaf8df85dcfd6dbeb20488ec34`. The artifacts were exported by `fbs-export` `5af2cc461ec60b4548843681fc1b13df9bbeb68dda113db1dbb870a097c16ed7`, built at `44a07a527eec6fe12e3f3d3af5b3ea62b0ead58f`; the fix did not touch the export. On `wick`, at `bd66ff21d`: `fbs-snapshot` `e7e99bb3…6d74`, `pair-picks` `95fcb98b…c568c343`. The serve-1 reads used `raw-id-replies` `874139a9…86a6` and `uor-chat` `4e19b04c…6603`, built at `44a07a527`. serve-3 and the second conversation, after the merge, used binaries built at `a3c31a227`: `raw-id-replies` `62c38b5f1b58490788b4e101585b8357cba34f846133da93baa1de229ea54c61`, `uor-chat` `8f99e4d6629c62aff8fa1bcbf44fdaa582ef217183f6a6bf789516558be348e9`, `fbs-export` `0f7ed7b5cbd0374bb3fc89ebaf37a6d36f4d39c65929071e95ab03375c3f1807`.

## Skills used

None. The kit is `local/claude/claude-384/`: `snap-384.sh` (the `wick` job), `export-384.sh`, `serve-384.sh` (serve-1), `serve2-384.sh` (serve-2), `serve3-384.sh` (the `/15` export and serve-3), `replay-384.sh`, `rows-384.sh` with `subst-384.sh` (claude-374's, run names changed), `retime-384.sh`, `chat-384.sh` and `gates-head.sh`.

## Still open

- **Compression, the next unit.** The rows are 96.7 MB of a 116 MB artifact. Rows shared by the pair's E8 address, or a higher MIN than 2 turns, would cut it; each needs its own float training and the same served bar.
- **The values in the served representation.** The rows are trained in F32 and rounded once; a D85 unit would train them in Q12 in the shared space.
- **Conversations that break after a cycle.** Seed 3's S-pair keeps a cycled reply in the conversation and breaks the next two turns; the panel reads each row from a clean window and cannot see this.
- **A broken beginning in a live conversation.** Where a reply's own text writes a beginning of the user marker and then something else, the trainer added the rows at that position and the served reader holds them back for one step. The panels hold no such position; a conversation could.
- **Speed at the same moment.** S-pair and S-b timed side by side, as `raw-id-replies --time-steps 1` allows.
- **The packet-parity bin.** `cargo check --tests` on the float crate fails at base on the `packet-parity` bin, which needs `uor_r4_cpu_train`; the checks above build `--lib` and the named bins instead. Not this unit's to fix.

## Plain summary

The model claude-370 trained in float learns a table with one row for each pair of words that appears together in users' messages. When the user writes, the rows for every pair of their words are added into the model's thinking. That table made the float model about 4.6 points better at giving exactly the expected reply. This unit puts the table into the integer file that runs on the laptop, so the laptop model reads it too, and checks that the laptop picks exactly the same rows as the trainer at every position of every test prompt. The first version did not: it kept adding the rows for two tokens after the next user message had begun, where the trainer had stopped. That was found and fixed, and every reading was taken again. On the laptop the model gets 524 and 516 of 2,218 replies exactly right with the table, against 423 and 409 without it: the same gain as in float. Most of the gain is the model repeating the user's words back correctly. Facts like capitals and arithmetic did not improve, and the file is six times larger, which is what the next step has to fix.
