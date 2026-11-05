# Prep sheet: three rules past the trained window, by held-out loss and reply agreement (integer-long-context, claude-287, 5 Nov 2026)

Lab: claude. Item: integer-long-context (#3471). Order: [claude-287](../../loop/tasks/integer-long-context/claude-287/ORDER.md), Casey's plan P4, unit 2 of 3.

TEST FITNESS: FIT - round 1, a fresh non-author review at `641ec3f57`. It found no blocker or major and re-derived every number it checked. Its eight minors are applied in this text ([Changes after FIT round 1](#changes-after-fit-round-1)).

**Why.** Casey's target (2 Nov) is a served 4,096-token conversation, with tok/s and RSS measured to 8,192. Every checkpoint is trained at one window W: the served a7r at 192, P2b's (L) at 2,048. The integer session must apply one rule past W, and none has a reading:
- the float scorer refused any context past a checkpoint's window ([cancel probe 1](../../loop/tasks/integer-long-context/claude-287/ORDER.md#cancel-probe-run-first));
- claude-286 found that carry needs no new forward, since no weight is sized by the window, and that its integer speed is measured (1,851 tok/s at 2,112);
- slide's and blocks' integer speeds are not measured.

This unit wires `--score-rows --past-window` and measures the three rules on two trained checkpoints. claude-288 serves the winner.

## Question

1. Past W, does **carry** hold the float model's held-out loss and greedy reply agreement within slide's band? Carry is one window over the whole conversation, with the trunk's state and the read's keys running on past W.
2. Does carry or **slide** keep anything from before W that **blocks** drops? Slide gives every target the last W tokens. Blocks cuts back-to-back windows of W and carries nothing over.
3. What does each rule cost: seconds a pass, and decode tok/s at W and at C?

## Base artifact

- **The pin:** this unit's head at run time, which carries the scorer's past-window code (Code, below). Without `--past-window`, every output and every refusal is the base's, and a test checks that.
- **The checkpoints**, each checked against its sealed run's sum file, pinned here by that file's SHA-256:

  | checkpoint | step | W | run objective | sum file | its SHA-256 |
  |---|---|---|---|---|---|
  | a7r seed 1 | `training-460fe46e/s1/r-drop-s1/models/step-016384` | 192 | `cell` | `s1/SHA256SUMS` | `aff46b2af26fc3da2530364b65d28bd42d8033af4f2d418c64e0d9d0dc40e9f6` |
  | a7r seed 2 | `training-460fe46e/s2/r-drop-s2/models/step-016384` | 192 | `cell` | `s2/SHA256SUMS` | `0950eba2fbd260488db747041bf8070fda7689dd67950c3d1dccc4e8c0fadd48` |
  | (L) seed 1 | `s1/L/models/step-008192` | 2,048 | `cell` | `s1/SHA256SUMS.files` | `ca0b8aeda4e23a6f793df8b503534a48ff090724632dc384c66a0b4b27424b23` |
  | (L) seed 2 | `s2/L/models/step-008192` | 2,048 | `cell` | `s2/SHA256SUMS.files` | `3c26a25280457e5b58c44e1bb7fc82dcd6a51b176fa86f49f6c5161a376598f3` |

  a7r is under `results/claude/one-model-end-to-end-2026-10-29/`, and (L) is under `results/claude/one-model-end-to-end-long-context-2026-11-04/`.
  - **Where W comes from:** the step's `config.json`, `context` 192 or 2,048. The scorer reads it, and refuses a `--context` other than W.
  - **Where the objective comes from:** the run's `config.json`, two levels up, with `objective` `cell`. Both runs have turn memory 8, and neither has `fbs_arm`, so no `--fbs-arm` is passed.
  - **Pod steps:** the job fetches each step folder and its run's `config.json`, then checks every file against the sum file's lines.
- **No training.** The object is the trained float model under the float scorer. No served, integer or milestone claim rests on this run. claude-288 measures the integer session.

## Data split

- **p-long** (claude-275, sealed; SHA-256s on [claude-276's sheet](one-model-end-to-end-long-context-2026-11-04.md#data-split)), for a7r:
  - `cell=dev,dev`, 160 conversations, 313,980 tokens and 609 queries;
  - query distances: 23 under 192, 586 at 192 or more, at most 1,994;
  - its longest conversation fits 2,048, so carry runs at C = 2,048.
- **p-long4k** (this unit), for (L):
  - **The command:** `heldout-panel out=… tokenizer=TOKENIZER cell=dev,dev conversations=160 seed=2771 protocol=2 chain=4096`, with the pinned tokenizer `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`. This is a data tool, not an evaluation.
  - **The draw:** 640,796 tokens, 640 queries and 0 metered mismatches. The redraws were 1,770 contradicting, 562 farewell, 94 mentioning, 1,280 too long and 0 dropped.
  - **The checks:** `check-queries` reports 640 checked, 0 failed.
  - **Distances:** 8 under 192, 250 under 2,048, 129 at 2,048–3,071 and 261 at 3,072 or more, at most 4,044. Replies are at most 21 tokens. Carry runs at C = 4,096.
  - **SHA-256s:**
    - tokens `ffefe084ae10d5d049ace0bb5acc49e9f7d2537b4a64038327f283ccf29933d9`;
    - mask `bccfdd7564d9795b4f4de4b260080defa488882a07c60d0a26809f0bb68db95d`;
    - `queries.json` `2f615b45439b6b2d590611420826594054a87451d0483d1099a8f45e4a7aa568`;
    - `user_turns.json` `0b94d34143caac7621cf37bcd0bcbe5fdc475aa865ebf6d6037ad738337452ee`;
    - `manifest.json` `97e7f14a29196cbbb63d5caa09227fe712f0b0185b0fc9b766bc081e56c70882`.
- **The census** is `train-in-space --census 1` against P2b's long TRAIN (`--train-split` with `61019954ef82d7b33d2d5a101f436fece29452f8580f8f3268e35229ccd0cb3c` and `29b34d2c9ed3a56bf638a3afd6863e5744b2c1d3f838a1034f19a77d1e02818d`), run on the laptop at this unit's code. `census.json` is `6c942e8879a23aecc1fe271f8c54d7fd21299513353868c87170379bd0e1244c`.
  - **Prompts:** p-long4k's exact prompt share is 0.0, and its template share at 1,024 is 0.0. So no prompt is shared with TRAIN, and `dev,dev` phrasings are never in TRAIN.
  - **Replies:** its exact reply share is 0.72, because the reply templates are shared by design.
  - **Answers:** 0 of 319 words are in TRAIN, and 12 of 321 numbers are, `found_queries` 31, 72, 328, 336, 340, 396, 452, 495, 555, 590, 591 and 620. p-long's census is [claude-281's](one-model-end-to-end-p2b-classification-2026-11-04.md#data-split): 0 of 319 words and 6 of 290 numbers.
- **The user turns** are recorded in `user_turns.json`. The long TRAIN was drawn before this panel and is not redrawn, so nothing is excluded from it (the manifest's `excluded: none`). The census's exact prompt share of 0.0 is the overlap check.
- **Stored** with a `SHA256SUMS`, read back 6 of 6, under `results/claude/integer-long-context-2026-11-05/inputs/{p-long4k,census}/`.

No model output was read to set any panel, reading, band or bar.

## Arms

There are 12 passes: two checkpoints, two seeds and three rules. Each pass writes memory on and memory out (L-010). The command is P2b's scoring command (claude-276) with the rule added and `--context` dropped:

`train-in-space --data-root DATA --score-rows CKPT --out DIR --samples 16 --reply-tokens 32 --retain-rates 1 --seed S --panel NAME=PANEL --panel-queries NAME=PANEL/queries.json --past-window RULE [--past-window-stride s]`

- **The decode:** the default greedy decode, as in P2b and claude-281; `--flock-end` is not passed. `--seed` is the checkpoint's own seed.
- **a7r** scores `p-long`, and **(L)** scores `p-long4k`.
- **blocks** runs `build_document_blocks` at W: the base path's back-to-back chunks of W, each read from its own block's start, so nothing carries over (changed at the pre-run review, below).
- **slide** uses stride s = W/16: 12 for a7r and 128 for (L). Each target past W then reads at least W − s + 1 tokens, 181 and 1,921, about 94% of W.
- **carry** sets the model's context to C, the panel's longest conversation rounded up to 2,048 or 4,096, and scores one window per conversation.
- **Queries:** each prompt ends at its reply, and starts where the rule says:
  - slide: the W positions before the reply, cut at `<|bos|>`;
  - blocks: the start of the reply's block;
  - carry: `<|bos|>`.

  The decode runs at W for slide and blocks, and at C for carry. A blocks reply that crosses its block's end slides from there (`blocks_crossing`, reported).
- **Carry's decode past C:** the greedy decode does not stop at `<|eos|>` (`in_space_long.rs:6298`), so with `--reply-tokens 32` it can pass C. That is 33 of p-long4k's queries, 345 steps a file, and those steps slide at C without the cache.
  - **Agreement is unaffected:** it compares only the first reference-length tokens (`past_window_query_bands`), and those fit inside C.
  - **The cost** is in Cost below.
- **DEV:** every pass also scores DEV at 192, as the identity check (I-1). Fresh V2 is refused with the option, and is not scored.
- **Order,** serial on GPU 0 of one pod with one binary, so each pass's tok/s is its own:
  - a7r seed 1: blocks, carry, slide;
  - a7r seed 2: blocks, carry, slide;
  - (L): blocks at seeds 1 and 2, then carry at seeds 1 and 2, then slide at seeds 1 and 2.

  (L)'s slide passes run last. Both read their stride from one file on the pod, written before the first starts (Stop rule).

## Readings

Every reading is per checkpoint, seed, rule and memory setting. Each comes from that pass's `past-window-NAME-{memory,no-memory}.json`, by a fixed `jq`. A row's band is `(position − 1 − doc_start) / W`, so band 0 is each conversation's first W targets. A query's band is its note's distance divided by W.

- **R0, tok/s first (reported).** Each queries file's decode tok/s, from the `decode` pair `--score-rows` prints, and each pass's wall seconds. Set beside claude-281's 137–148 tok/s at 2,048.
- **R1, loss past W (deciding).** The mean of the checkpoint's own `query_loss` over every target at offset W or more. The objective is `cell`, with no softmax or cross-entropy form (§3.2).

  ```
  jq '.window as $w | [.bands[] | select(.from >= $w)]
      | (map(.loss_sum) | add) / (map(.targets) | add)' past-window-NAME-memory.json
  ```
- **R2, reply agreement past W (deciding).** Over the queries whose note is at least W back, the reference tokens the greedy reply gives at the same place, both cut at `<|eos|>`, over the reference's tokens. That is 586 queries for a7r on p-long and 390 for (L) on p-long4k.

  ```
  jq '.window as $w | [.query_bands[] | select(.from >= $w)]
      | (map(.agreed_tokens) | add) / (map(.reference_tokens) | add)' past-window-NAME-memory.json
  ```
- **R3, the curves (reported):** each band's `loss_mean` and nearest-hit rate, and each query band's agreement, from band 0 out to C.
- **R4, hits past W (reported):** the nearest and writer hit rates over the targets at offset W or more.
- **R5, recall (reported, never deciding):** `stated` by query band, from the past-window file. P2b and claude-281 read recall at 0 for far notes, so it is not expected to separate the rules (claude-286's probe 5).
  - **The census-clean far words** come from `recall-NAME-memory.json` and the panel's census, as in claude-281: for p-long, claude-281's `census.json` under `results/claude/one-model-end-to-end-p2b-classification-2026-11-04/inputs/census/`; for p-long4k, this unit's.

  ```
  jq --slurpfile q PANEL/queries.json --slurpfile c census.json --arg p NAME --argjson w W '
    ($c[0].answers[] | select(.panel == $p) | .found_queries) as $E
    | [$q[0].queries[] | .answer | test("^[0-9]+$") | not] as $X
    | [.outcomes[] | select(.stated and $X[.query] and .distance >= $w
        and (.query as $i | ($E | index($i)) == null))]
    | length' recall-NAME-memory.json
  ```
- **R6, the memory (L-010, reported):** R1 and R2 with the memory out, and memory on minus memory out for each rule.

## Bar: the band, and the decision

**The band, an equivalence margin.** Within one seed, every rule reads the same model on the same targets (I-3) and the same queries (I-4), on one pod with one binary. So a rule's difference has no sampling noise of its own. The band is what two seeds of one recipe differ by, with a floor and a ceiling:

b_X = min(max(|X_slide,seed 1 − X_slide,seed 2|, f_X), 3 f_X)

- **f_loss** is 1% of the two seeds' mean slide R1;
- **f_agree** is 0.01, one point of agreement.
- **Why it is a margin:** a difference inside the band is no larger than retraining moves. It is not a paired test, and it rests on one draw of two seeds.
- **Why the ceiling:** an unusually wide seed spread cannot let a real loss under carry read as "holds".
- **Reported beside it:** the raw within-seed differences, carry − slide, carry − blocks and slide − blocks, for R1 and R2.

**The comparisons,** per seed and with the memory on (the served setting):
- **carry is worse** when R1_carry − R1_slide > b_loss, or R2_slide − R2_carry > b_agree. Otherwise **carry holds**.
- **a rule beats blocks** when R1_blocks − R1_rule > b_loss or R2_rule − R2_blocks > b_agree, and it is not worse than blocks on the other reading by more than its band.
- **a rule is worse than blocks** when R1_rule − R1_blocks > b_loss or R2_blocks − R2_rule > b_agree.

**The decision table** (claude-286's), read on each checkpoint across both seeds. The rows are tried in order, and the first that applies decides:

| # | reading | chooses |
|---|---|---|
| 1 | slide and carry are both worse than blocks, at both seeds | blocks |
| 2 | carry is worse, at both seeds | slide, for now. The carry training arm (2 Nov owner log: consecutive chunks, the trunk's state and the read's earlier keys carried, gradients inside a chunk) is the next unit, and P4's bound is checked then |
| 3 | carry holds at both seeds, and the same rule (carry, or slide) beats blocks at both seeds | carry |
| 4 | carry holds at both seeds, otherwise. This includes the tie, where neither beats blocks | carry, provisional, on serving cost. It is re-read on the first checkpoint that recalls far notes inside its window |
| 5 | anything else: the seeds disagree on carry | carry, provisional, with the disagreement named |

- **Blocks is the forgetting control.** A rule no better than blocks past W keeps nothing useful from earlier context. With recall at 0, the tie in row 4 is the likely outcome, since the panel then has not tested what the earlier context is for.
- **Named wherever it occurs, whatever row decides:**
  - carry worse than blocks at a seed. This can happen while carry holds against slide, when slide sits between them;
  - blocks beating both rules at one seed only.
- **Which row is served:** a7r's row decides the rule claude-288 serves, since a7r is the served model.
  - **(L)'s row** reads the same table from 2,048 to 4,096, the target's length.
  - **If (L)'s row differs from a7r's,** the served rule is still a7r's, but provisional. (L)'s row is named in the decision review as what the next long checkpoint re-reads.
  - **If (L) reads row 2,** the carry training arm is named for the long window, whatever a7r reads.
  - **What a7r's row cannot cover:** a7r is read only out to C = 2,048 (p-long), so its carry past 2,048 is unread. Serving a7r out to 4,096 rests on (L)'s row there, and the decision review says so.
- **With the memory out,** the table is read again (R6) and reported. If its row differs from the memory-on row, that is named.

Whatever the outcome, every reading is reported with its numbers. A failed instrument check leaves every reading unread (§4.1).

## Instrument checks

- **I-0, the suite:** the float crate's CUDA library suite passes at the pin on the pod, with `--no-fail-fast`. The past-window tests are CPU tests inside it: the windowers, the cut, the ruled query prompts, the refusals, the saved configs, the bands, and the base path unchanged.
  - **I-0n, on the laptop:** the pre-run review's one fix, blocks reading only its own block, is pinned by two tests. With the fix's dispatch line set aside they fail (a fresh build: 14 past-window tests passed, 2 failed), and with it they pass. They are CPU tests, so the laptop's gate stands for the pod's, and I-0 runs both by name.
- **I-1, DEV is the identity:** DEV is scored at 192 in every pass, and its rows file holds no context field.
  - **Rows:** per checkpoint and seed, `rows-dev-memory.json` and `rows-dev-no-memory.json` are equal byte for byte across the three rules, carry included. The job checks this on the pod after every carry and slide pass, against that seed's blocks pass, and stops on a miss.
  - **Why carry should match:** `set_context` changes only `config.context` (`geometric_stack_native.rs:1189`). `row_picks` sizes its batches from the window, not the model (`in_space_long.rs:1334`). The read pattern has no age term, so nothing in a forward of 192-token windows reads the model's context.
  - **How a mismatch is classed, decided now:** as an instrument fault. Something in the forward reads the model's context, so carry's panel forward is not the forward the other rules run. Every reading is left unread until the cause is found and repaired.
  - **Samples:** DEV's sample files are equal across slide and blocks. Under carry the model's context is C, so a DEV sample whose decode passes 192 does not slide; any difference is listed and classed, and reported only.
- **I-2, band 0 is the identity:** each conversation's first W targets read the same prefix from `<|bos|>` under every rule.
  - **slide and blocks** score them in the same first window, so band 0's `targets`, `nearest_hits`, `writer_hits` and `loss_sum` are expected to be equal exactly.
    - **One untested assumption:** at W = 192 a forward holds 16 windows, and slide's batches hold different neighbouring windows than blocks'. So exact equality assumes a row's result does not depend on the rest of its batch, which has not been measured.
    - **How a miss is classed, decided now:** a slide-against-blocks mismatch inside carry's tolerances below is classed as batch composition and recorded, not failed. A mismatch outside them fails I-2.
  - **carry** scores them in one longer forward. Its band 0 `targets` are equal exactly. Its hits are within 0.3% of band 0's targets of blocks', the scale of the 0.13–0.28% of top-1 picks claude-279 found differing between pods. Its `loss_mean` is within 0.1% of blocks'.
- **I-3, every target once:** per checkpoint, seed and memory setting, each band's `targets` is equal across the three rules.
- **I-4, the same queries:** each query band's `queries` and `replies` are equal across the three rules, and the `[queries]` line counts 0 past the window.
- **I-5, the loss is read:** every band has a `loss_sum`, the file's `objective` is `cell`, and the pass prints `[past-window] model reads W; loss under cell`, with C under carry.
- **I-6, carry runs at C:** carry's `[past-window]` line reads `run at 2048` for p-long and `run at 4096` for p-long4k, with 160 windows.
- **I-7, the inputs:** on the pod, every SHA-256 above equals its file's. `check-queries` passed on the laptop (Data split).
- **I-8, the grade can see a pass:** in every `recall-*.json`, `total.reference_states` equals `total.decoded`.

## Steps

1. On the pod:
   - fetch the pin, build `train-in-space` with `--features cuda --locked`, and record the binary's SHA-256;
   - check that `jq` is there;
   - run I-0.
2. Fetch the inputs: the driver data, p-long, p-long4k, and the four checkpoints with their run configs. Check every sum, then I-7.
3. Run the 12 passes in the order under Arms, with a watcher on the job.
   - Before each pass, the job writes its projection and stops at the bound (Stop rule).
   - After each pass, it prints the wall seconds and the `[past-window]` and `decode` lines.
   - It stops on a failed I-5 or I-6, and, after each carry and slide pass, on a failed I-1.
4. Seal every output with a `SHA256SUMS` under `results/claude/integer-long-context-2026-11-05/`, read it back, and only then take the pod down.
5. On the laptop: run I-1 to I-4 and I-8, compute R0 to R6, and read the table.

## Minimum test power: what the test can tell apart

- **Within a seed,** the rules are compared exactly, with no sampling noise (I-1 to I-4). The band is the seeds' own spread, so a difference inside it is no larger than what retraining moves.
- **Past W:**
  - every target at offset W or more in 160 conversations, about 283,000 positions in p-long and 313,000 in p-long4k before the mask (the masked count is in the files);
  - 586 queries for a7r and 390 for (L).
- **What it cannot tell:**
  - **Recall:** with recall at 0, agreement past W is mostly template tokens, which every rule can produce without the note. So a tie (row 4) says the panel did not test the earlier context, not that the rules are equal at recall. That is why row 4 is provisional, and is re-read on a checkpoint that recalls.
  - **a7r past 2,048:** a7r is read only to C = 2,048, so its carry from 2,048 to 4,096 is unread (Which row is served).
  - **Seed 2's configs** are not on the laptop. Their W and objective are checked on the pod, by the sum files' lines (I-7) and by each pass's `[past-window] model reads W; loss under cell` line (I-5).

## Cost

These are estimates from measured passes, not measurements:
- claude-276: a 2,048 scoring pass without the cache took 2,622 s, 2,563 s of it the queries, so about 0.18 s or less per 2,048-token row forward.
- claude-279 and claude-281: a cached 2,048 pass takes 305–441 s, and a 192 pass 78–91 s.

| arm | per seed | both seeds |
|---|---|---|
| a7r: blocks, slide (about 24,000 windows of 192, 16 a forward), carry (decode at 2,048) | about 1.5 + 3 + 8 min | about 26 min |
| (L): blocks, carry (forwards and decode at 4,096, and the uncached steps past C), slide (about 2,600 windows of 2,048, one a forward, memory on and out) | about 8 + 17 + 22 min | about 94 min |

- **Carry's decode past C** is 345 uncached steps a file, two files a pass. It adds about 2 to 9 min a pass, and the table carries 5 min.
- **Building, I-0 and fetching** add about 25 min, and sealing about 5 min.
- **In all,** about 2.5 h on one RTX 5090. At up to $1/h that is about $2.50, inside the ORDER's $3 bound, and at $0.56/h (today's cheapest verified 5090) about $1.40.
- **The job's pass estimates,** in seconds, are this table's: a7r 90 (blocks), 480 (carry) and 180 (slide); (L) 480 (blocks), 1,020 (carry), 1,320 (slide at 128) and 840 (slide at 256).

## Stop rule

- **The run stops** when all 12 passes are scored, the files are sealed and read back, and the pod is down.
- **The bound:** the job is started with the pod's price an hour and the time the pod came up. Before each pass, it projects the spend and stops if the projection passes $3. The passes left are then recorded as not run, and nothing is cut silently.
  - **The projection** is the spend so far, plus price × estimate × f / 3,600, plus $0.25 for sealing and `down`.
  - **The estimate** is the pass's estimate in Cost.
  - **f** is the pod's speed factor: the measured seconds of the passes done, over their estimates, and never below 1.
  - **The record:** each projection is written to `projection.jsonl`.
- **The stride first (the ORDER's stop row):** before (L)'s first slide pass, the job projects both slide passes at stride 128.
  - **If that passes $3,** (L)'s slide stride doubles to 256 for both seeds, and the choice is written to `projection.jsonl`. Each target past W then reads at least 1,793 tokens, 88% of W.
  - **No arm is cut before the stride.**
- **Carry at 4,096:** if the Hamming read refuses, or memory runs out, carry already runs one conversation a forward (`windows_per_forward` at 4,096 is 1). So it is recorded as an instrument limit, and (L)'s row reads not measured.
  - **This departs from the ORDER,** whose fallback is "(L) runs carry at 2,048 only". That cannot run on p-long4k: all 160 of its conversations are longer than 2,048, and `carry_windows_at` refuses a document cut into two windows.
- **A failed instrument check** is classed, repaired and re-run once inside the bound. Nothing is trained.

## Code in this unit

All of this is new code, with nothing changed on the base path:
- **`hamilton_sheet.rs`:** `build_document_windows_sliding` (slide), `build_document_blocks` (blocks), and `carry_context` with `carry_windows` (C, and one window per conversation).
- **`in_space_long.rs`:**
  - `PastWindowRule` and its refusals;
  - `past_window_cut` and `query_prompts_ruled`;
  - the bands, and `query_loss` by band in the rows' own batches, with no new arithmetic;
  - `checkpoint_window` and `run_objective`, reading the saved configs;
  - the `past-window-*.json` output.
- **`bin/train-in-space.rs`:** `--past-window` and `--past-window-stride`. A stride without a rule is refused, and so is the option without `--score-rows`.

## Changes after FIT round 1

Round 1 (`641ec3f57`) was FIT, with no blocker or major. It re-derived every number it checked: the panel's hashes, distances and C, the census, the counts past W, the window counts and strides, the cost inputs, and the sum files. It found the `jq` and the I-5 and I-6 strings matched the code. It left eight minors, each applied:
1. **The band** is now an equivalence margin with a ceiling of 3 f, and the raw within-seed differences are reported beside it.
2. **Row 4** is "carry holds, otherwise", and row 3 asks that the same rule beat blocks at both seeds. A carry-worse-than-blocks reading, and blocks beating both at one seed only, are named wherever they occur.
3. **a7r's carry past 2,048** is said to be unread, so serving a7r to 4,096 rests on (L)'s row.
4. **I-1:**
   - the job's on-pod DEV check is in Steps;
   - a carry DEV mismatch is classed in advance, as an instrument fault;
   - why carry should match is cited.
5. **I-2:** a slide-against-blocks band-0 mismatch inside carry's tolerances is classed in advance, as batch composition.
6. **Carry's uncached decode past C** is in Arms and in Cost.
7. **The bound** is now a projection checked before each pass, with the pod's speed factor, and (L)'s stride is chosen by the same projection. Each is written to `projection.jsonl`.
8. **Smaller points:**
   - the ORDER's carry-at-2,048 fallback is said to be impossible on p-long4k;
   - R5 names `recall-*.json` and gives its `jq`.

## Changes after the pre-run review

Two fresh read-only reviewers read the pod job, and the chain and the scorer, at `f2b179f18`. Both were GO with minors. Neither finished every item, and the open ones were closed here.

1. **Blocks is now a forgetting control (the one fix).**
   - **The defect:** the base cut, `build_document_windows`, reads each conversation's last, shorter chunk from the W tokens before its last target, which reaches back into the block before it. So the base path's last block carries part of the one before.
   - **Where it bites:** for a7r it is each conversation's tail. For (L), every p-long4k conversation is longer than 2,048 and at most 4,096 tokens, so all of its targets past W sit in such a chunk.
   - **Why it matters here:** blocks' queries already started at their block (Arms), so its losses and its replies read two rules, and a "blocks" loss past W would have read most of slide's past (lesson L-007).
   - **The fix:** `build_document_blocks` cuts the same back-to-back chunks and reads each one from its own block's start. A full block is the base cut's window, so band 0 and every full block are unchanged, and DEV still runs the base cut.
   - **The tests:**
     - `past_window_blocks_read_only_their_own_block`: each target reads only its own block, and full blocks equal the base cut.
     - `past_window_blocks_rows_and_replies_read_one_block`: each blocks row reads from its query prompt's start.
     - I-0n above.
   - **The class:** instrument, since no reading had been taken. The sheet's own Question already defined blocks this way ("carries nothing over"), so this brings the code to the FIT text.
2. **Closed here:**
   - the named tests (14, now 16) exist in plain test modules;
   - the `[past-window]` lines and the file names match the job's checks;
   - `uor-pod run` detaches with `nohup setsid`, and its ssh has a 10 s connect timeout and a 30 s keep-alive;
   - the Hamming read at 4,096 holds 16.8 M elements a row, under 2³²;
   - `set_context` refuses any change to a weight's shape, loudly.
3. **Accepted as they are:**
   - the job's signal trap waits for the running pass before sealing;
   - the pin is substituted and checked in the job's copy before upload.

## Changes after the run (claude-287's exact-head review)

The run at pin `be2defc00004daa8c020350c6a6f4baeb8aa2fda` is unchanged; these correct this sheet's text to what ran.
1. **Slide's query prompt.** Arms says slide's prompt is "the W positions before the reply". The code ends the W-token window at the reply's last input, as the base scorer's `query_prompts` does (`query_input_start`). So the prompt holds W + 1 − n tokens before a reply of n tokens: 172 to 187 at a7r and 2,028 to 2,043 at (L), for the panels' replies of 6 to 21 tokens. Under blocks a reply's first token reads anywhere from 1 to W tokens. Every slide query in the run used this.
2. **"Nothing changed on the base path"** (Code in this unit) rests on reading the code, by two exact-head reviewers. No CPU test pins `score_rows`' default path, which runs on the GPU. One refusal order changed: a panel that fails both the windowing and the query checks now reports the windowing error first.
3. **Refusals still owed,** filed as #3714 and touching none of this run: a missing run config, a losses vector shorter than the bands, carry over panels of different C, and blocks at W = 0. The laptop readout checks the first two on the sealed files: every band with targets has a numeric, non-zero `loss_sum` (156 of 156).
4. **Two headings renamed** to the names the harness's shape rule reads: "The band, and the decision" is now "Bar: the band, and the decision", and "What the test can tell apart" is now "Minimum test power: what the test can tell apart". Their text is unchanged.
