# Prep sheet: the one model trained as served at full length, (b) and (k2) at the repaired head (one-model-end-to-end, claude-263 part B, 3 Nov 2026)

Lab: claude. Item: [one-model-end-to-end](../../roadmap/items/one-model-end-to-end.md) (#3407). Path step 4: one model trained as served, exported, served and scored end to end.

**Part B runs only if part A passes.** Part A is [I-3 at the repaired head](one-model-end-to-end-i3-repaired-2026-11-03.md), on claude-262's 256-step served checkpoints. If I-3 fails at either seed there, part B does not start, and the next unit localizes the fault.

TEST FITNESS: FIT - fresh non-author adversarial review (round 5) of this sheet at SHA-256 `9875bdb64294e37ce4c8b316f4eccd29fefd63cd8e9ba424bf8200ab462aa2de`, after rounds 1 to 4's findings were fixed. Round 5's two minor findings (an absent arm in the integer-side branch, and a (k2) smoke failure under a fixed pin) are fixed in this text. No bar moved.

PRE-RUN: PASS - fresh D67 review of `job-full.sh` SHA-256 `162386e0b67bcdcc6b15d2e77d1ac2a52262208e514d5d487559575baaa83539`, `watch-full.sh` and this sheet. Its two majors were watcher settings, not a defect: the watcher launches with `MAXSEC=25200 BUDGET=20`, since this sheet's own estimate is 5.8–6.3 h and about $15. Its minors, and what was done about each:
- a finished pod's lease now renews until the pod is down (`watch-full.sh` SHA-256 `687d8de689affd94a6c819d13701aaa75e8a1390fc4c67a39fdd76e5aed6d896`);
- the watcher's checkout is kept for the whole run;
- the job launches with two arguments only;
- seed 1's first arm has its memory read from `gpu.csv` while it runs;
- each pod's `/workspace/job.sh` is checked against the hash above before `uor-pod run`.

This sheet is claude-261's [copy-key sheet](one-model-end-to-end-copy-key-2026-11-02.md), FIT in round 4, re-run at the head where served training was repaired, with each arm alone on its card. **Where a section below says "unchanged", claude-261's text holds word for word.** Every change is stated in its section.

## Question

claude-261's question, asked at the repaired forward: does keying the Hamming read's copy heads on the two tokens before a position, instead of one, make the trained-as-served base model reply better on prompts phrased in ways TRAIN never used, at no cost in served speed? Two more questions apply to every arm:
- Does the model, trained as served for 8,192 steps, serve on the laptop exactly as it was trained (I-3)?
- What does it do there: its decode tok/s and its transcripts?

claude-261's answers cannot carry over:
- claude-262 found that claude-261's served forward was missing `PRODUCT_EXP`, and repaired it in `fac0131bc`. Every claude-261 reading went through the broken forward.
- Every claude-261 seed was cut to 2,048 steps by two arms sharing a card.

## Base artifact

This is a training comparison from scratch: the candidates are the checkpoints each arm produces.

- **The pin.** Training, smoke, I-0, scoring and snapshots all use `train-in-space` and `fbs-snapshot`, built `--locked --features cuda` at `0073cd07bd5d16021639041aa30c55132d2c6dc9`. That is the head part A checks: claude-262's repair, plus claude-261's copy key.
- **On each pod**, the job fetches that commit, checks it is the commit named and clean, builds, and records the binaries' SHA-256 values.
- **On the laptop**, `fbs-export` and `fbs-toks` are part A's: built `--features serving-wasm`, from a tree whose integer and tokenizer crates and `Cargo.lock` equal the pin.
- **The outputs** go under `results/claude/one-model-end-to-end-full-served-2026-11-03/sN/`.

## Arms

- **Both arms run claude-261's command unchanged, with only the key differing.** (b) has copy key 1 (the flag absent); (k2) has `--hamming-copy-key 2`.
  - The command is `train-in-space --data-root DATA --out OUT --objective cell --retain-rates 1 --decay-mult 1 --batch 16 --steps 8192 --eval-every 1024 --lr 0.001 --lr-end 0 --turn-memory 8 --turn-memory-reply 1 --turn-memory-dropout 500 --train-as-served 1 --seed S --width 256 --hamming-read 4 [--hamming-copy-key 2]`.
  - This is claude-262's served recipe. Its `config.json` (`served-s1`) differs from this command only in `steps` (256 there) and `eval_every` (64 there).
    - Its `samples` 6 and `reply_tokens` 48 are `train-in-space`'s defaults (`train-in-space.rs:252-253` at the pin), which this command also leaves unset.
    - Its `width` 256 is the default, which this command passes explicitly.
- **Arms in turn** (Casey, 3 Nov: "Arms in turn"). There is one 1 × RTX 5090 pod per seed (1, 2, 3).
  - On each pod, (b) and then (k2) run alone on GPU 0 at every stage: smoke, training, scoring and snapshot.
  - No two processes ever share the card.
- **deepseek-5's four runs are dropped.** claude-262 ran them at the repaired head.
- **Scoring** is claude-261's command, unchanged: `--score-rows CKPT --samples 1536 --reply-tokens 64 --retain-rates 1 --panel p-w=PW --panel p-v=PV --seed S`.
  - It is the serial scorer at the pin, always. #3583's batched scorer is not used here, even if it merges first: part A's E tests it on 256-step checkpoints only, and these samples files feed R1 to R5 and I-3.
- **Then** `fbs-snapshot CKPT SNAPSHOT.json`, as before.

## Readings

R1 to R5 are unchanged: claude-261's grade (`local/claude/claude-261/grade.sh`, SHA-256 `3cd7663c93d2d5b8b4f59b8b4425a832d3f3be9505a91ad6af222a2fc2796e29`), with p-w deciding and p-v confirming.

**R6 and R7 are read only on exports that pass I-3.** Both use the snapped export, `fbs-export --run-config CONFIG SNAPSHOT OUT` with no raw flags, since `--train-as-served 1` trains snapped. claude-261 used `--raw-gates --raw-transition`.
- **R6:** claude-261's timing protocol and speed condition, unchanged.
- **R7:** conv-a turn 3 under each arm's seed-1 export, and whether it names Maya. New here, reported only: the whole conv-a, conv-b and conv-c transcripts under each arm's seed-1 export, through `uor-chat --protocol 2`.

## Bar

claude-261's per-seed verdicts (better, worse, level, absent) and its four rows are unchanged, read in order, except for the two changes below.

**Row 1 also needs I-3.** Every one of the six exports, both arms at every seed, must pass I-3. That is this sheet's answer to its second question: the model trained as served for 8,192 steps serves exactly as it was trained.

**If I-3 fails for any export**, at any seed and for either arm, **no row is read in this unit.**
- **Why.** A parity fault can sit in the float forward: claude-262's missing `PRODUCT_EXP` did, and its follow-up 5 names two more float-side edges. So no configuration is closed, and no candidate or base is named, on readings from a forward that has just failed parity (CHARTER §4.1, §4.2, L-007).
- **What is reported.** The I-3 count for every export, the failing exports by name, and R1 to R5 marked "held, not read". R6 and R7 are unread for any failing seed-1 export.
- **What it means.** The second question is answered "no" for the failing exports. A miss is classed instrument (§4.2): a parity fault, not a model property.
- **The next unit localizes the fault**, as part A's fail path says. What follows depends on where the fault is:
  - **Integer side only:** that same unit repairs the integer side at a commit it names. It makes every export made here again, from the sealed snapshots, with the repaired `fbs-export` and `fbs-toks` and the laptop pin moved to that commit, and runs I-3 on each. If every one passes, it reads this table in full, row 1's I-3 condition included, with the held R1 to R5.
  - **Float side, or not localized:** R1 to R5 are void. The unit after the localization re-trains both arms under this sheet, at the repaired forward's commit, which replaces the pin `0073cd07b`. I-0 runs again at the new pin. That re-training replaces any row's unit.

**What counts as an I-3 failure.** Each of these is an I-3 failure, read as above:
- an `fbs-toks` refusal, or a non-zero exit;
- an order mismatch;
- a miss past 8 of 310;
- an `fbs-export` refusal. The exporter is on the integer side, and the laptop pin cannot move inside this unit.

"The base" in the rows' consequences now means the base trained as served at the repaired head.

## Instrument checks

- **I-0** runs `cargo test --release --locked --features cuda -p uor-r4-float-train --lib -- --test-threads=1` on every pod, before any training. It passes only if the output shows all of these:
  - at least one test ran, and none failed;
  - `copy_key_two_tells_two_facts_after_the_same_token_apart ... ok`;
  - `pair_halves_puts_the_first_token_in_each_copy_heads_low_half ... ok`.
- **I-1 (the smoke)** is unchanged, with two exceptions.
  - The two arms' 256-step smokes run one after the other.
  - A (k2) smoke failure stops the pod as an instrument result, like a (b) failure, since the pin cannot move inside this unit. The next unit repairs (k2) and re-trains both arms under this sheet at the repaired commit, with I-0 again, as the Bar's float-side branch does.
- **I-2** projects each arm alone. The 90-minute limit and the cut to the largest multiple of 2,048 steps are unchanged, and a cut applies to both arms at that seed. claude-262's Step P measured 167.5 ms per served step at `4465685531`, which was squash-merged as `fac0131bc` with identical trees for both crates. At that rate, 8,192 steps take about 23 minutes before evals, so no cut is expected. I-2 still reads each arm's own smoke.
- **I-3 is the door, on every export, not seed 1 only.** Each arm's export at every seed goes through I-3 exactly as part A defines it:
  - the snapped export;
  - DEV's prompts from the arm's own `samples-memory.json`;
  - `fbs-toks ARTIFACT --replies PROMPTS --out FILE --reply-tokens 64`.

  It passes only with exit 0, no order mismatches, and at most 8 of 310 first tokens differing. An I-3 failure, as the Bar defines it, is an instrument fault (CHARTER §4.2), never a model result.
- **I-4 (pairing) and I-5 (the copy key in the header)** are unchanged.

## Data split

Unchanged: claude-261's nine pins for TRAIN, DEV, the tokenizer, p-w and p-v, checked against their `SHA256SUMS`. No key was chosen on either panel.

## Steps

1. On each pod, fetch the pin, build, and record the binaries' SHA-256 values. Run I-0.
2. Run I-1's smoke for (b), then for (k2), and project I-2.
3. Train (b), then (k2), for 8,192 steps each (or I-2's equal cut).
4. Score (b)'s checkpoint, then (k2)'s, with memory on and memory out, on DEV, p-w and p-v. Then snapshot each.
5. Save every output with `SHA256SUMS` under the store root, read it back, and only then take the pod down.
6. On the laptop:
   - export each arm at each seed (snapped), then run I-3 and I-5;
   - grade R1 to R5 (if any export failed I-3, the grades are recorded and sealed, not read: the Bar);
   - on the exports that pass I-3, run R6 and R7;
   - if every export made passes I-3, apply the table in order. An arm absent at a seed makes no export there, and its seed goes to row 4 as the parent's rows say.

## Minimum test power

Unchanged: three seeds, with 1,202 paired p-w prompts and 393 p-v prompts per seed, so R1 detects a net shift of about 3% of the panel. Each I-3 has 310 paired prompts.

## Cost

Three Vast RTX 5090 pods at about $0.55–0.80 an hour. Each pod takes about:
- 10 minutes for I-0's whole suite (the parent's figure);
- 10 minutes to build and fetch;
- 15 minutes for the two smokes in turn;
- 2 × 30–45 minutes of training, alone;
- 2 × about 2 hours of serial scoring. claude-261 measured 7,106–7,591 s for one arm alone, over DEV, p-w and p-v with memory on and out. That was under the broken forward, so the figure is conservative.

That is about 5.8–6.3 hours a pod: about $10–15 in all, at most $2.40 an hour. That is inside 4 pods and $8 an hour with what other labs hold at launch (`uor-pod status`). Each lease is taken for 6 hours and renewed, so the top of the scoring range runs past the first lease on its renewal. A healthy run is extended, never cancelled.

## Stop rule

claude-261's stop rule holds, with its shared-card out-of-memory clause replaced: an arm that runs out of GPU memory alone on its card is absent at that seed (row 4). The rest is unchanged:
- an `fbs-export` or `fbs-toks` refusal is an I-3 failure (the Bar): an instrument fault, never counted, with no repair inside this unit;
- a divergence is classified when it happens;
- outputs are read back from the store before any pod goes down;
- the watcher ends on every terminal state, with a 90-minute stall window for the scorer's silent passes.

A build or I-0 failure stops that pod before any training: it is an instrument result.
