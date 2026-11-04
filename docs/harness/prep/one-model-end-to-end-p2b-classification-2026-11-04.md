# Prep sheet: P2b's Negative, instrument or mechanism? Trained phrasing against held-out phrasing (one-model-end-to-end, claude-281, 4 Nov 2026)

Lab: claude. Item: one-model-end-to-end (#3407). Order: [claude-281](../../loop/tasks/one-model-end-to-end/claude-281/ORDER.md), THE ORDER (#2028 6634218223) step 4.

TEST FITNESS: FIT - round 2, a fresh non-author review at `eb40dcd99`. It found no blocker or major and re-derived every number it checked. Three of its four minors are applied in this text; the fourth needs no text. Round 1 at `63856b54e` was NOT FIT, with two majors. What changed in each round is at the end.

**Why.** [P2b](../../loop/tasks/one-model-end-to-end/claude-276/REPORT.md) read Negative. (L), trained at 2,048, stated a held-out note's value in 0 (seed 1) and 1 (seed 2) of 603 paired p-long queries. That closes the configuration only (§4.1). Before a next configuration is chosen, §4.2 asks for a classification, instrument or mechanism. p-long changes two things at once:
- **The phrasing:** p-long is `cell=dev,dev`. Its note and query user turns use only the `development` templates (`milestone_world_v2.rs:986-1015`), which TRAIN never contains. The reply templates are shared.
- **The values:** p-long's values are DEV values too.
- **No positive control was scored.** The instrument writes `cell=train,train`, drawn as TRAIN was, but none was drawn for P2b.
- **The commonest reply** in P2b's seed-1 queries file is the note acknowledgement (82 of 609), as if the query were read as a note.

So the Negative is either "the trained model cannot recall a note across turns" (mechanism: the read, or the recipe), or "it never learned to recognize a question it was not trained on" (a phrasing generalization, which is data and curriculum, not the read).

## Question

1. In the trained phrasings (`train,train`), does (L) at 2,048 state the note's value where it does not in the held-out ones (`dev,dev`)?
2. Which change carries it: the phrasing (`dev,train` against `train,train`) or the value (`train,dev` against `train,train`)?
3. When the reply fails, what does it say: a reply template with the wrong value, or a note acknowledgement?

## Base artifact

- **The pin:** this unit's head, which carries claude-279's decode cache, its four refusals (Code, below) and the census's `found_queries`. `--score-rows` steps from the exact decode cache where the window fits. claude-279 found the files byte-identical with the cache on and off.
- **The checkpoints:** P2b's sealed (L) at seeds 1 and 2, `results/claude/one-model-end-to-end-long-context-2026-11-04/s{1,2}/L/models/step-008192/`, checked against each seed's `SHA256SUMS.files` before scoring.
- **No training.** The object is the trained float model's recall under the float scorer. No served, integer, step-4 or milestone claim rests on this run.

## Data split

Three new panels, written on the laptop with the pinned tokenizer `d36d3e87…`. Each is `heldout-panel out=NEW tokenizer=TOKENIZER cell=CELL conversations=160 protocol=2 chain=2048 seed=SEED`, a data tool, not an evaluation. Each gives 0 metered mismatches, and `check-queries` reports 0 failed.

| panel | cell | seed | changes against TRAIN | tokens | queries | words / numbers |
|---|---|---|---|---|---|---|
| q-tt | `train,train` | 2761 | none (the positive control) | 314,924 | 617 | 298 / 319 |
| q-td | `train,dev` | 2762 | values only | 314,180 | 610 | 301 / 309 |
| q-dt | `dev,train` | 2763 | phrasings only | 312,879 | 590 | 286 / 304 |
| p-long (P2b's, sealed) | `dev,dev` | 2751 | both | 313,980 | 609 | 319 / 290 |

- **Distances** (under 192 / 192–511 / 512–1,023 / at least 1,024; at most):
  - q-tt: 31 / 70 / 143 / 373, at most 2,002;
  - q-td: 23 / 88 / 144 / 355, at most 2,002;
  - q-dt: 35 / 79 / 138 / 338, at most 1,999;
  - p-long: 23 / 86 / 138 / 362, at most 1,994.

  Replies are 6–21 tokens.
- **SHA-256s** (tokens, mask, `queries.json`, `user_turns.json`):
  - q-tt: `84f1eb1d01a8de327153635e4367a69148e89b6e9ca55cc6c1dad6fc3197fe63`, `9359992750d3866d9b7ff30da52920f12b3648e0b8a0ddf5436b912333616978`, `e40f1cfe5e45553270547420bbe556347ff8f8ea6d13a5418fbc7dd598c21b36`, `c86cfca51f5ccc62ee5a02c90fc10fa66beb947b19669249282a4744435d4200`;
  - q-td: `cbdfa95e132fe79e7a06831fcccd2fc7b5bab9cfac61ae215598742b70d89139`, `4e4f46c07052bf3637f97ac306701067bb53a5622a0bebcd860aa9b9d0a253b2`, `ac88ec02e0b825200ed52bee8f406782ac118c82f2094568f06672b252350f74`, `50adeaa4884d07f776a818c1d93c33a731beb50867e4f489263a04197d58a232`;
  - q-dt: `80368dad3b3e7d8e2a3852ea33f6dc38420c4c0ef2008ba0aca71640bdadb3af`, `1172b6a5607d9f3a033135ed1ce6533b22b545dc1f9784d6e78261506f776fa9`, `7f7938f0258c85725752c22159e593685dbecbb148731f1793e0aec13a46c8a7`, `21b6d5bef2753d57c0631a1b265d89c008da4c8559c8a2c239eda93103fdb95b`;
  - p-long: [claude-276's sheet](one-model-end-to-end-long-context-2026-11-04.md#data-split), unchanged.
- **The census** is `train-in-space --census 1` against P2b's long TRAIN, `--train-split` with its pinned SHA-256s `61019954…` and `29b34d2c…`. It ran on the laptop at this unit's code. It counts each panel's answers found as words of TRAIN's replies, and lists the queries (`found_queries`):

  | panel | words in TRAIN | numbers in TRAIN | queries set aside |
  |---|---|---|---|
  | p-long | 0 of 319 | 6 of 290 | 6: queries 0, 42, 93, 401, 471, 487 (P2b's six) |
  | q-tt | 3 of 298 | 318 of 319 | 321 |
  | q-td | 0 of 301 | 10 of 309 | 10 |
  | q-dt | 2 of 286 | 303 of 304 | 305 |

  A trained value that is a number comes from TRAIN's own pool, so in q-tt and q-dt a reply can state a number without reading its note. The invented words are new even in the trained-value cell. So **the deciding readings are words against words, census-clean:** 295 in q-tt, 301 in q-td, 284 in q-dt and 319 in p-long, of which 283, 287, 270 and 306 are at a distance of at least 192.
- **The value's split** is `split_of` (`milestone_world_v2.rs:427`), a keyed hash of the whole word, over one syllable generator (`word()`, `:566`). So a dev word and a train word are drawn alike, and only the hash tells them apart.
- **Overlap:** TRAIN was drawn with `exclude=` p-long's user turns. The new panels are new draws (new seeds). A `train` user turn can equal a TRAIN user turn word for word, which is what a positive control drawn as TRAIN was looks like. Keys and values are drawn fresh per conversation.
- **Stored,** with a `SHA256SUMS` read back: `results/claude/one-model-end-to-end-p2b-classification-2026-11-04/inputs/{q-tt,q-td,q-dt,census}/`. This is this unit's own root, never P2b's sealed one (L-002).

No model output was read to set any panel, query, reading or bar.

## Arms

Each seed's (L) is scored in four passes of P2b's scoring command, one panel a pass:

`--data-root DATA --score-rows CKPT --out DIR --samples 16 --reply-tokens 32 --retain-rates 1 --seed S --context 2048 --panel NAME=DIR --panel-queries NAME=DIR/queries.json`

- **p-long's pass:** P2b's own, the same command and flags, unchanged. I-1 then compares P2b's sealed recall files with the output of the very same command.
- **q-tt's, q-td's and q-dt's passes:** the same command, each with its own panel in place of p-long. One panel a pass, as in P2b, so no panel's grade can depend on another scored before it in the same pass.

`--seed` is the seed's own, as in P2b. The eight passes run serially on GPU 0, on one pod with one binary, and each writes memory on and out.

## Readings

Every count is read from the panel's `recall-NAME-memory.json` (memory on), per seed, by this fixed `jq` over `outcomes[]`. `outcomes[].query` is the query's index in `queries.json`, the index `found_queries` uses too. A query counts when four things hold:
- its outcome is `stated`;
- its answer in the panel's `queries.json` is a word (not `^[0-9]+$`);
- its `distance` is at least 192;
- its index is not in the census's `found_queries` for that panel.

```
jq --slurpfile q PANEL/queries.json --slurpfile c census.json --arg p NAME --arg k stated '
  ($c[0].answers[] | select(.panel == $p) | .found_queries) as $E
  | [$q[0].queries[] | .answer | test("^[0-9]+$") | not] as $W
  | [.outcomes[] | select(.[$k] and $W[.query] and .distance >= 192
      and (.query as $i | ($E | index($i)) == null))]
  | length' recall-NAME-memory.json
```

`--arg k decoded` gives each panel's n. Tested on P2b's sealed `score-L-2048/recall-p-long-memory.json` with p-long's census: s_dd = 0 at seed 1 and 0 at seed 2, each of n = 306. Over every query, the seeds' stated counts are 0 and 1; seed 2's one is query 20, a number.

- **R1 (deciding): trained cell against held-out cell, far words.** s_tt (q-tt) against s_dd (p-long). The panels are drawn apart, so the test is unpaired. R1 passes where s_tt ≥ 10 and s_tt − s_dd > 2√(s_tt + s_dd). It is reported with each panel's n.
  - **Far only:** a note at least 192 tokens back is past the window P2b's short arm (S) trained at, and past the window P2b's control scored (L) at ([P2b's sheet](one-model-end-to-end-long-context-2026-11-04.md#arms)). The deciding reading is recall across turns, not recall of a note in the reply's own neighbourhood.
- **R2: which change carries the gap, far words.** Two gaps, each against q-tt, each passing where s_tt − s_x > 2√(s_tt + s_x):
  - **the phrasing gap:** q-dt (phrasing changed, value kept);
  - **the value gap:** q-td (value changed, phrasing kept).

  R2 picks the next step when R1 passes (Bar). The value gap is expected near 0 for words, since `split_of` draws dev and train words alike (Data split), but it is read, not assumed.
- **R2n: numbers, reported only.** Each panel's stated count over its number answers, census-clean and in all, set beside its census count. In q-tt and q-dt nearly every number is TRAIN's, so their number counts are never read as recall.
- **R3: the reply form.** In each panel's `samples-NAME-queries-memory.json`, the text of each reply before its first `<|eos|>`, classed by this fixed `jq` test:
  - **ack:** `test("^ Okay, I.ll remember")`;
  - **template opening:** `test("^ (It|That).s ")` or `test("^ [A-Z][a-z]+ is ")`;
  - **template shape:** the same openings ending `[^ ]+[.]$`;
  - **other:** the rest.

  Tested on P2b's sealed s1 L2048 queries file: 82 ack, 135 open "It's/That's", 1 opens "{k} is", 391 other. Reported only.
- **R4: the memory (L-010).** Each panel's far clean words, memory on against out, paired by query (b, c as P2b's R2). Reported only.
  - **Why it is not in the Bar:** the turn memory is built from TRAIN and holds none of these new words, so it cannot be where a stated word comes from.
  - **What the Bar claims instead:** recall from the window by the trained model, read included. It is not a claim that the Hamming read alone carries the recall.
- **R5: tok/s first.** Decode tok/s for each queries file at 2,048 with the cache, from the `decode` pair `--score-rows` prints, set beside P2b's 15.2.
- **The distance bins** (under 192, 192–511, at least 512) for R1, R2 and R4, with each panel's median far distance. The under-192 bin (12 to 14 clean words a panel) is reported, never read. R1's rule inside the 192–511 bin and inside the at-least-512 bin is reported beside it, since the panels are separate draws.

## Bar

Read per seed, then across both:
- **Generalization (P2b's Negative is a generalization fault):** R1 passes at both seeds. The trained model then states a new word from a note at least 192 tokens back in its trained cell, and not in the held-out cell. The read and the recipe stay as they are, and the next unit changes the data. Which change follows R2 at both seeds:
  - **only the phrasing gap passes:** held-out phrasings mixed into TRAIN, or the real-text corpus (THE ORDER step 6);
  - **only the value gap passes:** an instrument flag, not a data change. For words, the two value splits differ only by a keyed hash the model cannot see (Data split), so a gap there points at a panel difference. The next step is Inconclusive until a panel check, comparing the two panels' distances and templates, finds it;
  - **both pass:** both changes are named, and the data unit carries both;
  - **neither passes, or the seeds disagree on R2:** only the double change fails. Both changes are named, and which one carries it is Inconclusive.
- **Mechanism (it does not recall across turns even as trained):** R1 fails at both seeds with s_tt < 10 at each. The trained model then does not recall a far note even in its trained cell, on panels drawn as TRAIN was.
  - **What it closes:** P2b's configuration (copy key 1, the earliest 16 matches averaged, `flock_read_hamming.rs`) at this recipe. It never closes the read (§4.2).
  - **What it cannot say:** this test does not separate the read's configuration from the recipe.
  - **The next unit's change** (copy key 2, or the latest match) is chosen on claude-267's oracle, which found key 1 right at 3,859 of 8,243 copyable DEV positions with the latest match. It is not shown by this test.
- **Inconclusive:** anything else, the seeds disagreeing on R1 included. Nothing is named or closed.

Whatever the outcome, R2 to R5 are reported with their numbers. A failed instrument check leaves every reading unread (§4.1).

## Instrument checks

- **I-0:** the float crate's CUDA library suite at the pin passes, `--no-fail-fast`, with claude-275's query tests and claude-279's decode-cache tests named. That includes `the_decode_cache_decodes_the_same_replies_through_a_window_slide`, which carries this unit's tests for the row cut and `first_refusal`. **I-0n:** that test also fails with the row-cut fix set aside, by a reverse patch of the fix's own lines, and passes again once it is restored.
- **I-1 (the scorer did not move):** the re-scored `recall-p-long-{memory,no-memory}.json` equal P2b's sealed ones byte for byte, seed by seed. Only the recall grades are compared across pods: claude-279 found that 19 of 609 query replies and 0.13–0.28% of top-1 picks differ between this lab's 5090 pods, while the recall files stayed identical. Every cell-against-cell reading is scored on one pod with one binary, so no reading crosses pods. p-long's pass is P2b's own command, so I-1 never depends on what other panels in a pass would do.
- **I-2 (the panels):** `check-queries` passed for each panel on the laptop (Data split). On the pod, every SHA-256 in the Data split equals its file's.
- **I-3 (the grade can see a pass):** in every `recall-*.json`, `total.reference_states` equals `total.decoded`.
- **I-4 (the cache ran):** each pass's `decode_forwards` has `steps` > 0. Its `first_refusal` is reported.
- **I-5 (the pairing):** each panel's memory-on and memory-out recall files list the same queries.
- **I-6 (the grade agrees with the text):** a second grade, made on the laptop from each queries samples file, agrees with `stated` on every query of every panel, memory on and out.
  - **The second grade,** by `jq`: the query's answer as a whole word (no letter or digit beside it) in `reply_text` before its first `<|eos|>`.
  - **Any disagreement** is listed and classed before any reading is made.
  - **Tested on P2b's sealed p-long queries files:** it agrees on 609 of 609 at each seed, seed 2's one stated reply (" That's 97.", query 20) included.
- **I-7 (the passes repeat), reported only:** each pass also scores `dev`. Its five `dev` files (`rows-dev-*`, `samples-memory.json`, `samples-no-memory.json`, `samples-dev-flock-memory.json`) are compared byte for byte across a seed's four passes on one pod with one binary. A difference is reported and classed. It is not in the Bar.

## Steps

1. On the pod, fetch the pin, build the float crate's `train-in-space` with `--features cuda`, record the binary's SHA-256, and run I-0, then I-0n (set aside, run, restore, rebuild).
2. Fetch the inputs: the three panels, p-long, the census, both seeds' checkpoints and the pinned driver data. Check every `SHA256SUMS`, and run I-2.
3. Score seed 1's (L), then seed 2's, at 2,048, one panel a pass: p-long, q-tt, q-td, q-dt (Arms). After each p-long pass, check I-1 on the pod and stop on a mismatch.
4. Seal every output with `SHA256SUMS` under `results/claude/one-model-end-to-end-p2b-classification-2026-11-04/`, read it back, and only then take the pod down.
5. On the laptop: check I-1 and I-3 to I-7, compute R1 to R5 and the bins, and apply the Bar.

## Minimum test power

R1's rule needs s_tt ≥ 10 on q-tt's 283 far clean words, a stated rate of about 3.5%. P2b's s_dd on p-long's 306 was 0 at both seeds. With s_dd near 0, R1's chance to pass, taking s_tt as Poisson with mean 283 × rate (computed with `jq`):

| rate | one seed | both seeds |
|---|---|---|
| 3% | 0.35 | 0.12 |
| 4% | 0.69 | 0.48 |
| 5% | 0.90 | 0.81 |
| 6% | 0.97 | 0.95 |

So the test reliably sees a recall rate of 5% or more at both seeds. Below about 4%, a miss reads Inconclusive (seeds disagree) or Mechanism, and that is the floor of what it can tell apart.

## Code in this unit (claude-279's review SHOULDs, and the census)

Each refusal returns the group to the whole-window re-run, and the census field is additive. None changes a scored token: I-1 checks that the p-long recall files still equal P2b's. Each fix ships with a test that fails with the fix set aside.
- `step_instrument` (`flock_read_hamming.rs`) refuses the Hamming read's tie rule being off. CPU test; set aside, it fails.
- `step_refusal` (`geometric_stack_native.rs`) refuses reads set since the prefill, by an epoch the prefill records, not only by site count. Reads set and cleared again passed the count. CPU test; set aside, it fails.
- The decode switch (`in_space_long.rs`) clears a row cut left on the thread before it asks the cache. CUDA test (I-0, I-0n).
- `DecodeForwards` names the first refusal's kind. CUDA test (I-0).
- The census lists `found_queries`. CPU tests; set aside, both fail.

## Cost

- **The laptop:** three panels and the census, done (about a minute each).
- **The pod:** one RTX 5090.
  - Eight passes at 2,048, with the cache, four per seed, one panel each. The scale is claude-279's on-L2048 pass, 441 s, with the queries file about 98% of it. So a seed takes about 4 × 441 s ≈ 30 min, and both seeds about 1 h.
  - Building, I-0 and I-0n add about 20 min.
  - About 1.4 h at up to $1/h is about $1.40, inside a $3 bound.

## Stop rule

The run stops when all eight passes are scored, the files sealed and read back, and the pod is down. A failed instrument check is repaired and re-run once inside the bound. Nothing is trained.

## Changes after FIT round 1

Round 1 (`63856b54e`) was NOT FIT, with two majors and three minors. Every number it spot-checked was right, and so was the `jq`.

**The majors:**
- **M1 (L-010): the Phrasing branch claimed "the read works" without needing the memory to make a difference.** R1 also counted notes in the reply's own neighbourhood.
  - **The fix:** R1 now reads far words only (distance at least 192: 283 in q-tt, 306 in p-long), and the near bin is reported only.
  - **What the Bar claims now:** recall from the window by the trained model, not the Hamming read alone.
  - **R4 stays reported, not in the Bar:** the turn memory holds no new word, so it cannot be where a stated word comes from.
- **M2: R1 changes the phrasing and the value at once, but its pass was called "Phrasing" and fixed the next step as held-out templates.**
  - **The fix:** the pass is now "Generalization", and R2's two gaps pick the next step.
  - **The value split, checked:** it is a keyed hash of the whole word over one syllable generator (`milestone_world_v2.rs:427`, `:566`). The value gap is therefore expected near 0, but it is read.

**The minors:**
- **m1:** the power section gives Poisson chances, not a cut-off.
- **m2:** the Mechanism branch says the next read configuration is chosen on claude-267's oracle, not shown by this test.
- **m3:** new I-6, a second grade from the reply text, tested on P2b's sealed files. Also, I-2's `check-queries` is the laptop's (the pod checks the SHA-256s).

## Changes after FIT round 2

Round 2 (`eb40dcd99`) was FIT. It found M1 and M2 fixed in substance, re-derived the far counts, the Poisson table, I-6 and `split_of`, and left four minors:
- **The value-gap-only branch** is now an instrument flag, Inconclusive until a panel check, never a change to TRAIN's values.
- **The bins** report each panel's median far distance, and R1's rule inside 192-511 and inside 512 and over.
- **The 192** cites P2b's arms: (S) trained at 192, and the control scored (L) at 192.
- **I-6's positive** on P2b's files is one number reply. Stated words first show on q-tt, and I-6 covers every panel, so this needs no text.

## Changes after the pre-run review

The first pre-run review, at `1a96a5c4f`, ran out of context before it finished. The gap it named first was that nothing showed that scoring four panels in one pass leaves p-long's recall files byte-identical to P2b's sealed ones, and I-1 depends on that.
- **The fix:** each seed is now scored in two passes, P2b's own p-long command unchanged and then the three new panels, so I-1 compares one command with itself.
- **What it leaves unchanged:** no panel, reading, bar or line of code, so the FIT stands and the pin's I-0 and I-0n are unchanged.
- **The cost:** the same, about 1.4 h on one 5090.
- **The next step:** a fresh pre-run review reads this sheet and the job.

The second pre-run review, at `cdd4f9ec8`, was split into four narrow reviewers.
- **The chain:** GO.
- **The scorer:** GO. It left one point open: in a pass over three panels, state left by one panel could reach the next and bias R2's gaps, though not R1.
  - **The fix:** one panel a pass, as in P2b. This also costs nothing, since the queries file is about 98% of a pass.
  - **New, reported only:** I-7, from the `dev` files every pass also writes.
- **The pod side:** FIX FIRST, with two cheap majors, both now fixed in the job:
  - the job checks for `jq` before the build;
  - the watcher stops, and wakes the session, when a lease renewal fails, since a lapsed lease lets the reaper take an idle pod before its results are saved.

None of this changes a panel, reading, bar or line of code.

## Run 1: stopped at I-0n; the test repaired

Run 1 ran at `04b547c70` on `vast-59164039`. It passed I-0, then stopped at I-0n (EXIT 4) before it fetched any input: the slide test passed with the row-cut fix set aside. This is classed as an instrument fault, never a result.
- **The cause:** every decode forward clears the row cut after it. So a cut left on the thread meets only the first exact-length group's first step. That group is the 3-position prompt, and `prefill_refusal` refuses its prefill as a short prompt (`time` below `CONVOLUTION_WIDTH`, 4) before it asks about the cut. The test could not fail.
- **The repair:** test only, at `64475a830`. The row-cut check now decodes the prompts from length 5 up. On the same pod it passes with the fix in place, and fails with the fix set aside on "a row cut left on the thread changed the forwards" (prefills 2, the first refusal being the row cut). No scored path changes.
- **Run 2:** the sheet's one re-run, at `64475a830`, in new folders. Run 1's 26 files are sealed in the store under `run1/`.

## Outcome (run 2, at `64475a830`)

Every instrument check passes: I-0, I-0n and I-2 on the pod, I-1 (the four p-long recall files equal P2b's sealed ones), and I-3 to I-7 on the laptop.
- **The Bar reads Mechanism at both seeds.** s_tt is 0 of 283 at each, under 10, and s_dd is 0 of 306.
- **R2:** q-td (0 of 287) and q-dt (0 of 270) read 0 too, so both gaps fail.
- **What it closes:** P2b's read configuration at this recipe, never the read.

R2 to R5 and the bins are in [the REPORT](../../loop/tasks/one-model-end-to-end/claude-281/REPORT.md).
