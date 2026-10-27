# REPORT: in-space-trunk-to-text, claude-146

Lab: claude. Order: [ORDER.md](ORDER.md). Research note: [RESEARCH.md](RESEARCH.md). Prep sheet: [in-space-trunk-to-text-hamming-recall-2026-10-27](../../../../harness/prep/in-space-trunk-to-text-hamming-recall-2026-10-27.md) (TEST FITNESS: FIT after two text fixes; light gates, per the owner's 27 Oct direction on #2028). The language run is read by claude-145's sheet, [in-space-trunk-to-text-content-2026-10-27](../../../../harness/prep/in-space-trunk-to-text-content-2026-10-27.md). The item stays open.

**Speed.**

- **Training,** at step `2,048` of the recall probe, with the three arms of a seed sharing one RTX 5090 (`df1d7892`, seed 2):
  - `12,187` context tokens/s with the Hamming read;
  - `14,439` with claude-145's content read;
  - `17,139` with no read.

  **The read costs about 29%** while the GPU is shared. Alone on the GPU, the language run's seed 3 trained at `32,013` context tokens/s with the read.
- **Decode,** in this runner (float, RTX 5090, 64 tokens after 16–192 context tokens; it re-runs the whole window per token, so it times the runner, not a served decode): `178–209` tokens/s with the read, against `372–390` for its no-read twin. **The read halves it here.** The read has no integer serving path yet, so there is no integer decode figure.

## What did not work, first

- **The language run lands in cell D at two seeds of three** (B at seed 1). The read does not clear claude-145's language bars: openings pass at seed 1 only (`298` against `283`), reach at none (`R16` at most `0.0327` against `0.05`). This closes the configuration, not the mechanism (CHARTER §4.1). The read's weight moved far from 0 (`0.72–1.01`), so by the sheet the start is not the cause: the key or the layer is. [Measurements](#the-language-run) has every number.
- **The replies still mostly ignore the prompt,** even at the seed that passes openings. "Remember, the code word is kandgind." gets "Got it, your friend is k know later!".
- **The first three forms of the read retrieved nothing.** Each closes its configuration, not the mechanism.

  | Commit | Form | Recall probe (hits per 512, by bucket) | Language |
  |---|---|---|---|
  | `9d4ca83b` | Keys and queries from the trunk's state, 4 heads, position spin | Seed 2: every arm at chance through step 4,096 | Openings `161` (the floor) at both seeds through step 8,192, stopped |
  | `67678fc3` | Keys from the previous token's point, copy-head queries from the current token's: an induction match. Payload: the kept position's residual, added after layers 2 and 4 | Seed 2: chance through step 2,560 (`[1,1,4,4]`); weight `0.026` at step 512 | not run |
  | `87ade492` | The same match. Payload: the kept token's own table point, added before the final norm | Seed 1: chance through step 1,024 (`[2,0,2,0]`); loss `1.00444` against no read's `1.00438` at step 512; weight `0.18` | not run |

  **Why, as classified.** Each is a mechanism fault at its configuration (the instrument worked: the same probe later scored `0.98–0.99` on the working form). The causes below are my readings of each fault, consistent with its numbers but not tested one by one:
  - **`9d4ca83b`, mechanism (at that configuration).** Hard top-K credits only the positions it picks, and with keys made from the trunk's own state, the right position is no nearer than any other at the start, so it is never picked and never learned ([RESEARCH](RESEARCH.md)).
  - **`67678fc3`, mechanism.** The match was right by construction (the copy test pins it). But the payload, added to the stream, reads to the trunk as its own input, not as its answer: the trunk predicts what follows the copied token, not the token.
  - **`87ade492`, mechanism.** The copied token's own point, added next to the stream's much larger parts and then normalised, barely moves where the prediction lands; and the weight that scales it grows only by the optimizer's step.
- **The no-read and content arms recall nothing, at any distance.** Reported, not barred. Even with the value 1–7 tokens back (the 2–8 bucket counts the distance between the two copies of the key, and the value sits one after the first), inside the trunk's own 8–12-token reach, the no-read arm stays at chance (`df1d7892`, seed 2, step 4,096: `[1,0,0,1]`). Trained on this objective at this length, at two seeds of this configuration, the recurrence shows no key–value binding. The content read, which brings back the kept position's own residual by design, does no better (`[0,3,1,2]`).
- **Recall slips a little after its peak.** Every run of the working form reaches `511–512` of 512 in every bucket by step 1,024–1,536 (`512/512` in three of the four), then loses up to 13 of 512 in the farthest bucket by step 4,096. It does so with and without the review fixes, so the shared rotation is likely not the cause (one comparison). The copy weight has no bound (by design: no squashing), so the likely cause is a weight past 1 on some coordinates, carrying the prediction beyond the copied point. Not tested.
- **The in-unit review found two faults in the first working form (`6d8bac25`), fixed at `df1d7892`, which every reported number comes from:**
  1. **Copy exactness held only at step 0.** The copy query and the key went through two separately learned rotations. Copy queries now go through the key's own rotation, so a repeated token is an exact match however the rotation learns.
  2. **The type heads were turned by a rotary position code** (RoPE's per-lane spin, as quaternions). The owner's direction is SpiralCore's transport spin for position: "we do not need rope, we have the spiralcore transport spin mechanisms" (13 Oct). The spin is gone; the trunk's recurrence carries order.

  The reviews also asked for the recall runner to save the claude-144 read with its model, to record its pairs per bucket (`recall.json`), and for the generator's test to check distinct values (it passed with that check removed from the generator; it now fails). All done.
- **A language run on the pre-review form was stopped at step 6,144,** so the reported language run is the merged form's. At step 6,144 it read openings `161` (seed 1) and `214` (seed 2, 3 distinct picks), against the twins' `161` and `189`; nearest top-1 `8,041` and `7,972` against `7,999` and `8,094`. Its folder carries a `STOPPED` note; its `EXIT 0` is the stop's trap, not a finish.
- **One run failed on my sequencing, not the code.** The first seed-2 test run at `87ade492` hit `CUDA_ERROR_OUT_OF_MEMORY` in two tests, because I launched it while another run's three arms held the GPU. The same commit passed all 112 tests on seed 1's run; every later recall run had the GPU alone.
- **The recall run departs from its sheet in three ways,** all recorded here:
  1. **Code.** The sheet pins `39f5cc57`; the read changed after it (the table above, and the review fixes). The arms, the task, the steps and the bar are the sheet's.
  2. **Gates.** Six negative gates instead of three: the margin's sign (kept); a row keeping itself (retargeted to the new choice); and four new ones, the previous-token key, position 0 as a candidate, the readout's move, and the copy query's shared rotation. The sheet's third gate, picks that lose their window's offset, is gone: gathers now run inside each window, so there is no offset to lose.
  3. **Pods.** Both seeds ran on one RTX 5090, one after the other, not one pod each.

## What changed

**The induction-keyed Hamming read**, `crates/uor-r4-float-train/src/flock_read_hamming.rs` (new), switched by `--hamming-read H` (0 is off). Two reads, each with `H = 4` heads matching on their own 64 coordinates: 2 copy heads, 2 type heads.

1. **Match.** The key at an earlier position `s` is the sign code of the shared table's point for token `x_{s−1}`, through a learned per-lane rotation. A copy head's query is the code of the current token `x_t`'s point through that same rotation. So "the position after the earlier copy of my token" is an exact match, Hamming distance 0, at every step: the induction circuit of Olsson et al. (2022), built in instead of grown. A type head's query is the residual through a second rotation. Candidates are `1 ≤ s < t`; the 16 nearest are kept, hard; each weighs `max(0, m − d) / m`.
2. **Payload.** A copy head brings back the table's point of token `x_s`, the token that followed the earlier copy. A type head brings back the rotated residual at `s`.
3. **Delivery.** The first read runs inside the stream after layer 2, adding both kinds through weights from 0. The second runs on the final norm's output, the points the nearest readout decodes. There, where a copy head found an earlier copy, the prediction moves toward the copied point, `n + w_c ⊙ (c − n)`, exact at `w_c = 1`.

**Why the move is the copy.** A transformer's head copies by adding to the logits, and a large addition wins whatever else is there. A nearest readout names the point a prediction is at. Adding a point moves that position little, but moving toward it is, at weight 1, the copy itself.

**The rest:**
- `geometric_stack_native.rs`: the two hooks in `hidden`, one after layer 2 and one after the final norm, each keyed on the input tokens' own points.
- `in_space_long.rs`: the MQAR-style recall task (`recall_window`, after `mqar-bench.rs`: four distance buckets, disjoint key, value and filler ranges, fresh pairings every window, with host tests of the generator) and its runner (`run_recall`); the read wired into both runners, saved and loaded with every checkpoint, and cleared by the arm guard.
- `flock_read.rs`: claude-144's choice and optimizer made reusable, its behaviour unchanged.
- `bin/train-in-space.rs`: `--recall PAIRS` and `--hamming-read H`.

## Measurements

### Recall

**The read retrieves, by the pre-registered bar, at both seeds.** Commit `df1d7892542dcdba1a45dd64387d4a123868055c`, binary sha256 `3093b347…` (the same in every `df1d7892` run: both recall seeds, both language jobs and the decode timing), step 4,096, 512 held-out queries per bucket per arm.

| Seed | Arm | 2–8 | 9–32 | 33–96 | 97–180 | Bar (33–96 and 97–180) |
|---|---|---|---|---|---|---|
| 2 | hamming | `0.994` | `0.998` | **`0.994`** | **`0.990`** | pass |
| 2 | none | `0.002` | `0.000` | `0.000` | `0.002` | |
| 2 | content | `0.000` | `0.006` | `0.002` | `0.004` | |
| 1 | hamming | `0.990` | `0.994` | **`0.984`** | **`0.986`** | pass |
| 1 | none | `0.002` | `0.000` | `0.002` | `0.000` | |
| 1 | content | `0.004` | `0.010` | `0.000` | `0.000` | |

The bar: at both seeds, `R ≥ 0.50` and `R ≥ R(none) + 0.10` in both far buckets. Before the review fixes, `6d8bac25` passed it too: far buckets `0.990`/`0.982` (seed 2) and `0.988`/`0.975` (seed 1). Every arm's step-0 reading equals the others' (I-2).

**When it turned on.** At step 512 recall is still near chance (`[3,4,3,2]`, seed 2), and at step 1,024 it is `512/512` in every bucket. The copy weight starts at 0, and AdamW moves it by at most about the learning rate per step. With the learning rate falling linearly from `0.001` to 0 over the 4,096 steps, the step sizes sum to `0.480` by step 512 and `0.896` by step 1,024, so the most it can reach is about `0.48`, then about `0.90`. Recall turns on when the moved prediction becomes nearer the copied token than any of the other 4,095 tokens of the 4,096-token vocabulary.

### The language run

`--hamming-read 4` at `df1d7892`, read against claude-145's no-read twins at the same seeds, at that sheet's settings, 16,384 steps. Seeds 1 and 2 shared one RTX 5090; seed 3 ran alone after them (`lang-s3.sh`, the same binary). Every step-0 reading equals its twin's.

| Seed | Nearest top-1 `T` (twin) | Openings `O` (twin) | Openings bar | `R16` (twin) | Reach bar | Cell |
|---|---|---|---|---|---|---|
| 1 | `8,687` (`8,607`) | **`298`** (`247`) | `283` | `0.1190 / 5.4619 = 0.0218` (`0.0080`) | `0.05` | **B** |
| 2 | `8,709` (`8,706`) | `200` (`229`) | `265` | `0.1783 / 5.4479 = 0.0327` (`0.0115`) | `0.05` | **D** |
| 3 | `9,233` (`8,587`) | `302` (`280`) | `316` | `0.1471 / 5.4980 = 0.0268` (`0.0116`) | `0.05` | **D** |

- **The reading is cell D, at two seeds of three.** Seeds 1 and 2 split, B and D, so seed 3 decided, as pre-registered. The configuration it closes: an induction-keyed Hamming read with two copy and two type heads, after layer 2 and on the readout, with no position signal, 16,384 steps.
- **Cell D's own split:** the read's weight moved far from 0 (mean absolute `0.81`, `1.01` and `0.72` at seeds 1–3), so the start is not the cause; by the sheet, the key or the layer is. My reading, not tested: for openings that points at the type heads' key, which nothing in the read's construction makes right (RESEARCH, credit problem (a)).
- **No flag F, and the fit holds or improves:** the read's nearest top-1 at step 16,384 is above its twin's at seeds 1 and 3 (`+80`, `+646`) and level at seed 2 (`+3`, within noise); seed 3's `9,233` is the line's highest. Along the way it led at seed 1 at every checkpoint, and at seed 2 at six of eight (by up to `785`, at step 4,096), trailing at steps 10,240 (`−134`) and 14,336 (`−41`).
- **Reach rises but stays under the bar:** `R16` is 2.3–2.8 times each twin's, against a bar of `0.05`.
- **Openings move late and unevenly.** Seed 1 leaves the floor at step 6,144 and reaches `298`; seed 2 stays at `161`–`169` until step 12,288, then `190` and `200`; seed 3 leaves it at step 4,096 and reaches `302`. The pre-review form, whose type heads were turned by position, had seed 2 at `214` at step 6,144, where this form has `161`: one seed at one step, so a hint, not a finding, that type heads ask partly by position.
- **The replies,** at step 16,384, seed 1:

  | Prompt | Model | True |
  |---|---|---|
  | "Remember, the code word is kandgind." | "Got it, your friend is k know later!" | "Okay, kandgind is the code word." |
  | "My lucky number is 5246." | "Got it, your friend is ,,,,, remember remember care." | "Got it, your lucky number is 5246." |
  | "Add 4 and 2." | "Got it, your friend is P! is st would." | "4 plus 2 is 6." |

  The likely reason, not tested: the copy heads find earlier copies of the current token, but in a reply the current token is "Okay," or "is", whose earlier copies are followed by other words, so the copy names the wrong next word. Copying a prompt word (`kandgind`, `5246`) needs a head that asks for that word by its role, which is the type heads' job.

  **One reply carries a prompt word.** Seed 3, for "Remember, the code word is kandgind.", writes "Got it, your friend is kandgr hope": the code word's first pieces, in order. The likely mechanism, not tested, is the copy chain: once one piece of the word is out, the copy heads find it in the prompt and bring back the piece that followed it. The reply's frame is still wrong ("Got it, your friend is"), so the role that should place the word is what is missing.
- **Decode** (this runner, float, RTX 5090, 64 tokens; it re-runs the whole window for each token and each read's whole sort): with the read `209`, `196` and `178` tokens/s after 16, 64 and 192 context tokens; its twin `390`, `377` and `372`. A served decode would cost one popcount pass over the context per head per token; that path is not built.

### Where the results are

- Store: `caseyallard/uor-r4-store:results/claude/hamming-read-2026-10-27/`, each set sealed and its file counts read back against the pod before it came down:
  - recall before the review fixes: `SHA256SUMS-claude-146-recall`;
  - the merged form's recall and the stopped pre-review language run: `SHA256SUMS-claude-146-df1d7892` (sha256 `7b4d1642aadcdf40e1666708cb2a74574b2bc8b1a6ac65b7090dd23e98f003c2`);
  - the language runs (seeds 1–3) and the decode timing: `SHA256SUMS-claude-146-lang` (sha256 `91e51d9838a1d4a85e11b31414b2e7ed59e126e2a1a0ecdea04a949ff98c5169`).
- Notebook: two rows in [float-training](../../../../research/notebook/float-training.md).
- Job scripts: `local/claude/hamming/` (`recall.sh`, `lang.sh`, `lang-s3.sh`, `decode.sh`), copied into the store beside the results.

## Checks run at the head

The code has not changed since the runs' commit, `df1d7892542dcdba1a45dd64387d4a123868055c`; later commits touch only this REPORT and the notebook.

- **CUDA library tests on an RTX 5090,** on each recall run's own build: `113 passed; 0 failed; 5 ignored`. The language job's own test step exited 0.
- **Six negative gates,** on each recall run: with one rule set aside at a time (the margin's sign, a row keeping itself, the previous-token key, position 0 as a candidate, the readout's move, the copy query's shared rotation), the named test fails (code `101`). Each gate first checks that its patch changed the file.
- **The generator's negative gate** (host): with `|| values.contains(&value)` removed from `recall_window`, `a_recall_query_names_the_value_after_its_keys_only_earlier_copy` fails; restored, it passes.
- **Host:** `cargo test --locked -p uor-r4-float-train --no-fail-fast` passes 71 library tests and every other target (12, 4 and 2 tests).
- **The stub-nvcc type check** of the CUDA-gated tests (`--features cuda --all-targets`) is clean. Clippy (`--no-deps -D warnings`) reports one error, `decode_speed`'s nine arguments, which is the same on `main`.
- **`uor-harness loop gates --full`:** 11 of 11 green.
- **In-unit reviews:** two fresh reviewers read the code at `6d8bac25` (the read and its hooks; the recall task and its runner). Their findings are fixed at `df1d7892` and listed above.

## Skills used

- **holding-the-goal.** The owner's order, recall first and language only on a pass, held through three failed forms of the read; each was stopped as soon as it read chance.
- **root-cause-debugging.** Each failure was classified before it was acted on:
  - the three forms' chance readings, as mechanism, each traced to its domain (where the read's output landed);
  - the out-of-memory test failure, as instrument (my launch beside another run), rerun alone;
  - the review's two faults, fixed and pinned by a new gate (`tie`), or removed.

## Still open

- **The type heads' key,** for openings: a head that names a prompt word by its role (the word or number after "is"), so the copy chain can start. Two cheap levers, each its own configuration:
  1. **position by SpiralCore's transport spin** in the type heads (the pre-review form's seed-2 hint);
  2. **the readout's copy weight clipped to [0, 1],** for the slow recall slip (a bound, not a squashing function).
- **The read's integer serving path** (popcount over 64-bit codes per head) is unbuilt, so its served decode cost on the M1 is unmeasured. The [pipeline map](../../../../model/pipeline-map.md) places it.
- **The item stays open.** Its done-when needs openings and reach together (cell A); this unit reaches neither at two seeds of three.

## Plain summary

The model now has a way to look back at its own prompt and bring a word forward, built from sign bits and bit counts instead of the usual attention arithmetic. On a pure memory test it works almost perfectly. It finds the right word up to 180 tokens back about 99% of the time, while the model without it never does. On real conversation it helps a little: the model predicts the next word at least as well as without it at every seed (clearly better at two), and at one seed of three it starts replies the way people do noticeably more often. But by the bar set before the run, it does not yet make the model answer from the prompt. The part that still has to learn is the one that decides *which* prompt word matters. That is the next piece of work.
