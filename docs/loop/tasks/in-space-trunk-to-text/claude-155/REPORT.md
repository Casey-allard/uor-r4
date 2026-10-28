# REPORT: in-space-trunk-to-text, claude-155

Lab: claude. Order: [ORDER.md](ORDER.md). Prep sheet: [in-space-trunk-to-text-prompt-register-2026-10-28](../../../../harness/prep/in-space-trunk-to-text-prompt-register-2026-10-28.md). TEST FITNESS: FIT, and PRE-RUN: PASS, by a fresh non-author reviewer (owner log on #2028, comment `6477012870`).

The readings, by the sheet:
- **(r), the learned prompt register (8 slots), is cell D with flag F at both seeds.** Training collapses: `T` falls to `684` and `1,246` of 14,287, and openings to `0`.
- **(g), the plain bag of the user's turn, meets the item's whole done-when at both seeds**, on its final checkpoints. That is openings `≥ 197` with `T ≥ 7,002`, `R16 ≥ 0.05`, and the replies recorded. **The item closes.** Its openings (`Ō = 234.7 / 266.3`) do not beat claude-152's (a) (`302.0 / 228.0`) by the sheet's margin.
- **Pivot note (CHARTER §4.4):** four results on this line (claude-152 to claude-155) have not moved openings past (a). The line stops tuning reads on the float trunk. The diagnosis and the next step are below.

**Speed.** Context tokens/s in training, each arm alone on one RTX 5090:

| Arm | Seed 1 | Seed 2 |
|---|---|---|
| (r) register | `12,573` | `12,641` |
| (g) bag | `28,560` | `21,430` |
| (a), claude-152 | `32,049` | `32,329` |

- **The register costs about 61% of the speed.** It gathers `(rows, slots, picks, width)` per forward.
- **The bag costs about 11%** at seed 1. Seed 2's pod was slower in every phase, decode included.
- **Decode** in this runner (float, 64 tokens after 16, 64 and 192 context tokens, re-running the window each token), seed 1 / seed 2:
  - (r) `141 / 135 / 126` and `142 / 135 / 126`;
  - (g) `125 / 119 / 111` and `96 / 90 / 75`.

  These mix host speed with the reads' host-side work. There is no integer path for either read.

## What did not work, first

- **The learned register collapses training at both seeds (flag F).** By checkpoint:

  | Step | `T` seed 1 / seed 2 | Copies | Train loss, seed 1 |
  |---|---|---|---|
  | 2,048 | `6,341 / 5,579` | `583 / 446` | `0.985` |
  | 4,096 | `4,031 / 3,081` | `231 / 146` | `0.883` |
  | 8,192 | `1,222 / 1,474` | `48 / 94` | `0.968` |
  | 16,384 | `684 / 1,246` | `5 / 26` | `1.022` |

  `Ō` is `0.0` and `3.3`. Training loss turns back up after step 4,096, so the model unlearns in training as well as on DEV. The replies degenerate: "Note these down: zundpim is 832, grarkfud is 4217." gets "ile doctor doctor doctor …".
- **Classified: configuration, not instrument.** Every instrument check passed at every seed:
  - I-1: `145 passed`;
  - I-2: step 0 equals (a)'s, and the configs are right;
  - I-3 and I-4;
  - I-5: dropping every slot changes the reading.

  Two causes are visible in the code and the drops. Neither is tested here.
  1. **The keys send gradient into the trunk.** A position's key is the sign of the live stream after layer 2, straight through, and every pick's weight `relu(m − d)` pulls the trunk's states toward the slot queries: 8 slots × 8 picks per position. Dropping every slot from the final checkpoint does not restore the trunk; it reads `11` and `518`. So the trunk itself was bent.
     - The Hamming read builds its keys from the detached token table and trains normally.
     - claude-144's flock read took keys from the trunk's final point and lost no fit.
     - The fix is to detach the keys' input, or take keys at the readout.
  2. **The slots are not distinct.** Their starting query codes coincide in groups, and identical slots get identical gradients, so they never separate. The per-slot drops are identical within each group: slots `{0, 2, 6}`, `{1, 7}` and `{3, 5}`. The 8 slots act as 4 distinct slots, in groups of 3, 2, 2 and 1. The fix is distinct starts, checked by a test.
- **The bag does not beat (a) on openings.** `Ō` is `234.7` (`212, 275, 217`) and `266.3` (`252, 272, 275`). That is `−67.3` and `+38.3` against (a) seed for seed, and under bar 1 (`347.3`).
- **No cell for (r) is better than D:**
  - bar 1 is `347.3`;
  - bar 2 is the best control, (g) seed 2, `266.33 + 43.32 = 309.7`;
  - (r) reads `0.0 / 3.3`.

## What worked

- **The item's done-when is met by the bag, at both seeds,** on the final checkpoints:

  | (g) | Openings (≥ 197) | `T` (≥ 7,002) | `R16` (≥ 0.05) |
  |---|---|---|---|
  | seed 1 | `217` | `8,993` | `0.166` |
  | seed 2 | `275` | `9,253` | `0.155` |

  Greedy replies to the six DEV prompts are recorded at every checkpoint (`samples.md`, `samples.jsonl`). So done-when 1, 2 and 3 hold on the same checkpoints, as the sheet reads them. Two limits:
  - its replies still mostly template ("Got it, I'll remember them." for three of the six at seed 1);
  - its openings do not beat (a)'s.
- **The bag is used.** Dropping it from the final checkpoint takes `T` from `8,993` to `7,479` and `9,253` to `7,605`, and openings from `217` to `28` and `275` to `39`. The model's first reply token now depends on the whole user turn: `R16` is 8 to 16 times (a)'s `0.020 / 0.010`. The opening read in claude-154 was barely used (`25 / 19` tokens).
- **The bag is cheap:** about 11% of training speed. Its running sum is a doubling scan, with no matrix product.

## The pivot (CHARTER §4.4)

**Four results, one pattern.**
- claude-152's margin objective lifted copying (cell C).
- claude-153's width 512 lifted fit (cell D).
- claude-154's opening read was ignored (cell D).
- claude-155's bag was used but did not lift openings past (a), and the learned register collapsed.

Routing the prompt into the trunk works. Copies rose, and the bag is used heavily. Openings, which need stored associations ("Note these down" → "Noted."), do not move.

**The measured reason.** The trunk and token table hold `1,112,576` learnable numbers (the Hamming read and the register add a few thousand more): the shared token table holds `1,048,576`, and the six-layer trunk only `64,000`. Each routed projection holds 1,024 to 2,048 numbers (8,448 per layer); they are signed routings, not weight matrices, which is what makes them free to serve as integers. A lookup of the prompt's first 8 tokens names `607` openings from the same TRAIN, which holds `26,450` distinct prompt starts. The trunk has nowhere to store that many associations.

The charter's mission is geometry **and exact memory**, and this line has tested geometry alone. Other labs' exact addressed memory is built and passes its oracle (true address `80/80`, no memory `0/80`), but it is not connected to this trunk.

**The owner's decision,** 28 Oct 2026, in claude's chat, after this diagnosis: *"ok yes, my vote is to pivot to your diagnosed recommended new roadmap"*. On serving format, he then kept determinism: train in float, export exact integers (below, in Still open).

**Next on this line, in order:**
1. **A per-stage profile** of one training step and one served token. Estimated from shapes, the whole-vocabulary nearest search costs about `1,000,000` operations per token, against about `25,000` for the trunk, in serving and in the training loss alike. Measured wall-clock by stage is missing.
2. **A mechanism map** for the owner's open refocus direction. It lists every stage from first token to emitted word, with its job, its mechanism, its status, its float form, its integer form and the export between them.
3. **One decisive run: route plus memory.** The bag forms a key from the user's turn and reads a sparse exact memory built from TRAIN, against a no-memory control (L-010).
4. **Fixes recorded for whoever reopens the learned register:** detach its keys' input, give its slots distinct starts, and test both.

## What changed

- **`crates/uor-r4-float-train/src/flock_read_register.rs`** (new, `06dd98549`): `PromptRegister`.
  - Spans by the turn anchor.
  - Keys: sign codes of each lane's turned real part, straight through.
  - Learned ±1 slot queries, Hamming distances, and a top-K of the span chosen on the host (detached, ties to the earlier position).
  - Relu-margin weights and per-slot value turns, giving weighted means.
  - Per-slot zero-start weights at the stream after layer 2 and at the readout.
  - `--register-select all`, the bag: one slot, uniform over the span, by a doubling scan with no `cumsum`.
  - Tests: 4 host (spans, picks, save/load and refusals, settings); 4 CUDA (zero weights, a slot's weighted mean against a reference, the bag's uniform span, gradients).
- **Wiring** (`7e445dae8`), beside the opening read:
  - two hooks in `StackModel::hidden`, with the slots carried from the stream to the readout and cleared on every forward;
  - `LongConfig.prompt_register`, `register_picks` and `register_select`, all left out of `config.json` when off;
  - the flags `--prompt-register Q`, `--register-picks K` and `--register-select learned|all`;
  - start, its own optimizer, save, load and cleanup;
  - in `--inspect`, `register_weight_mean_abs` and, on the last checkpoint, `register_drop`: the reading with each slot, and every slot, zeroed;
  - 3 host tests.
- **The item file:** `status: done`, with the met done-when.
- **The prep sheet, this REPORT and its notebook row.**

With no new flag, every file a run writes is unchanged (host tests pin the config keys).

## Measurements

### The runs

`train-in-space --objective cell --hamming-read 4 --retain-rates 1 --decay-mult 1 --batch 16 --steps 16384 --eval-every 2048 --lr 0.001 --lr-end 0 --seed S` on the pinned TRAIN at width 256, plus each arm's flags: (r) `--prompt-register 8`, (g) `--prompt-register 1 --register-select all`. One arm per RTX 5090. Every pod built and tested its own binary from `7e445dae83eacf3c3e5c6a7f51c3108ef00c3479`.

At step 16,384, seed 1 / seed 2:

| Arm | `T` (of 14,287) | `O` (of 1,226) | `Ō` (last 3) | Copies (of 4,130) | `R16` | Register weight | Train loss |
|---|---|---|---|---|---|---|---|
| (r) | `684 / 1,246` | `0 / 0` | `0.0 / 3.3` | `5 / 26` | `0.094 / 0.084` | `1.138 / 0.845` | `1.022 / 1.012` |
| (g) | `8,993 / 9,253` | `217 / 275` | `234.7 / 266.3` | `1,158 / 1,560` | `0.166 / 0.155` | `0.268 / 0.276` | `0.607 / 0.619` |
| (a), claude-152 | `9,328 / 10,573` | `306 / 245` | `302.0 / 228.0` | `1,517 / 2,933` | `0.020 / 0.010` | none | `0.604 / 0.618` |
| (o), claude-154 | `9,320 / 9,487` | `224 / 148` | `206.0 / 156.3` | `1,820 / 2,081` | `0.029 / 0.028` | (opening read) | `0.634 / 0.638` |

- `R16 = shift(16) / shift(0)` from `--reach`. At step 0 it is `0.746` and `0.730`. (r)'s final `R16` is high only because its trunk barely learned; it does not meet done-when 1.
- **Every slot dropped**, at the final checkpoint:
  - (r): `T` `684 → 11` and `1,246 → 518`;
  - (g): `8,993 → 7,479` and `9,253 → 7,605`; openings `217 → 28` and `275 → 39`.
- **The bars:**
  - bar 1 is `302.0 + 45.3 = 347.3`;
  - bar 2 is `266.33 + 43.32 = 309.7`, set by the best control, (g) seed 2, above (o)'s `206.0`;
  - flag F is `T < 8,828`.

**Instrument checks,** at every seed:
- **I-1:** `145 passed; 0 failed; 5 ignored`, as pre-registered, on the first pod's CUDA run and on each of the four pods' builds.
- **I-2:** every arm's step 0 equals (a)'s at the same seed (`2/0/730`, `5/3/681`), on each probe and each arm. Each `config.json` records the register's settings, 4 Hamming heads, width 256, no opening read and the pinned TRAIN.
- **I-3:** both weights are above 0 at every arm (the table). The Hamming read's weight in (r) grew to `1.48 / 1.30`, against `0.48 / 0.55` in (g).
- **I-4:** all 17 losses of every arm are finite: 8 training-loss means and 9 DEV losses (the job log's I-4 line).
- **I-5:** at every arm, the reading with every slot dropped differs from the reading with the register.

### Evidence

- **Location:** `caseyallard/uor-r4-store:results/claude/prompt-register-2026-10-28/`, one tar per pod, named by pod and arm:
  - `pod1-r-s1.tar`, holding the first pod's CUDA test run too;
  - `pod2-r-s2.tar`, `pod3-g-s1.tar` and `pod4-g-s2.tar`.

  `job.sh`, `cuda-tests.sh` and `watch.sh` are beside them.
- **Seals:** `SHA256SUMS-claude-155`, sha256 `e81bf9ae11d71fd02554d513304a9021c140345a74aa6c352c412a69e2aad816`, lists:
  - `pod1-r-s1.tar` `3a0e0fc32f1c54e3a9a2a0eaaa7336e24ff389c576a47ac77a59b56aba36e918`;
  - `pod2-r-s2.tar` `76baa03c17286d2fd698d415f05ddabb21b8f7b5349609d1836d4594c7f73822`;
  - `pod3-g-s1.tar` `e60a17d4302c6e0b9bb4321dfbdf62633bbc54dbbf1fb26363cc8d816e9597ad`;
  - `pod4-g-s2.tar` `501c81a2fc9ef50ee4408cc618a7e53ecda903dffabd105af773b37e9bee3f96`.

  Every tar was read back and checked against its pod's hash before that pod came down.
- **Cost:** four Vast RTX 5090 pods, 4.29 pod-hours, about `$3.14`. All are down.
  - Pods 1 and 2: `$0.80` and `$0.77` at `$0.563/h`.
  - Pod 3: `$0.66` at `$0.975/h`.
  - Pod 4: `$0.90` at `$1.106/h`.

## Checks run at the head

**The runs' code is `7e445dae`.** After the runs, one commit corrected `new`'s doc comment in `flock_read_register.rs`. It now states the known start-rule flaw instead of claiming the starts differ; behaviour is unchanged.

- **CUDA library tests on an RTX 5090 at `7e445dae`:** `145 passed; 0 failed; 5 ignored`, on the first pod's CUDA run and on each of the four pods' builds, the register's 4 CUDA tests among them.
- **Host:** at `7e445dae`, 90 library tests passed and every host target of the crate passed, 0 failed.
- **Negative gates,** each set aside, run and restored:
  - picks from the whole window fail the picks test (exit 101);
  - the register's `load_reads` line removed fails the checkpoint-load test (exit 101).
- **`uor-harness loop report --check`:** PASS. **`loop gates --full`:** 11 of 11 green.
- **In-unit reviews,** each by a fresh reviewer that never saw the plan:
  - **FIT and pre-run review:** FIT and PASS, with their notes applied (listed in the sheet).
  - **The module code review: APPROVE.** It confirmed both run findings from the code:
    - the start rule repeats every 6 lanes, so slot `j + 6` equals slot `j` and slot `j + 3` is its opposite;
    - the keys' straight-through path into the live stream.

    It found no defect in what the module computes: spans, incremental top-K, gather indices, doubling scan, span sums and zero-weight divide. Its low notes are an f32 cancellation in the bag's prefix sum over long windows, `load` not checking the dtype, and a start-rule test to add before any rerun.
  - **The wiring code review: APPROVE.** No site is missed; slots are cleared on every forward; the drop restores the register. Its low notes: the forward wiring and the drop have no unit tests (the vendored candle computes nothing on the CPU), and a trunk of fewer than 2 layers would ignore the register silently.
  - **The claims check of this REPORT, its notebook row and the item's closing note: APPROVE.** It confirmed every cell, bar, flag and done-when claim against the sealed tars, the pod log and the checkpoint's header. Its six minor slips are fixed:
    - two decode figures cut off, not rounded;
    - the slot groups (3, 2, 2 and 1);
    - the parameter count's scope;
    - the projections' sizes;
    - an unmeasured "slower host";
    - bar 2's displayed sum.
  - **A fresh re-review at the final head:** APPROVE.

## Skills used

- **holding-the-goal.** Casey asked for "learned query slots over the user's turn, kept as separate vectors and reaching the readout, with the opening read and a plain bag as controls". That is what ran, and the sheet decided it. When the results and the parameter count pointed past this line, the REPORT says so and names the pivot the charter requires.
- **root-cause-debugging.** The register's collapse was classified before it was read. Every instrument check passed; then the code (a straight-through key path into the trunk) and the per-slot drops (identical slots) gave two concrete, untested causes, each with a fix.

## Still open

- **The next steps of the pivot,** in order: the per-stage profile, the mechanism map for the owner's refocus direction, and the route-plus-memory decisive run.
- **Serving format, answered by the owner** (28 Oct, claude's chat): *"determinism is still probably the goal but like you said we can do the training in float and maybe just export the integer exact... either way it is worth looking into the archived hopf golden ratio complex - riemannian bridge we used for the original prime router or angular router"*. Training stays float and serving stays integer, through an exact export, as CHARTER §3.6–3.7 already say; no charter change. Every new piece needs an exact integer form, and the archived prime and angular router is to be read into the mechanism map. He added: *"I don't care what we serve at long as it is cheap"*. So the map ranks each stage by its cost per token (operations and bytes moved), and the number format is chosen under that.
- **The vector writer's admission** (bounded output candidates) is built and does not work: its address hinge cannot be met (the item file's 27 Oct state). The whole-vocabulary search is the largest per-token cost, so this is a separate item.
- **The learned register's two fixes:** detached keys, and distinct slot starts with a test.
- **No read since claude-144 has an integer serving path.**

## Plain summary

Casey asked for learned slots that read the user's turn, kept separate and reaching the output, against two controls: claude-154's opening read and a plain average of the turn (the bag).

- **The learned slots broke training.** The model unlearned after the first eighth of the run. Two causes are visible in the code: their keys pulled on the trunk's own states, and several slots started identical and stayed that way.
- **The plain bag met this item's goal at both seeds:**
  - first-word accuracy above the minimum (`217`, `275`);
  - overall accuracy above the count table;
  - a model whose output depends on words 16 positions back.

  The model uses it heavily. Its first words are still no better than the best earlier model's.
- **The bigger finding:** the model's learning part is 64,000 numbers, and 94% of the model is the word table. It can route the prompt's words but has nowhere to store what each question should produce. The project's design puts that storage in exact memory, which this line never connected.

So this line stops tuning reads. Next come a speed profile, a map of every mechanism from first token to last word, and one decisive test of routing plus exact memory. The runs cost about `$3.14`, and all pods are down.

## Done when

1. **Met.** The register is built:
   - its 4 host tests pass, and 2 negative gates bite (picks from the whole window; the checkpoint load removed);
   - its 4 CUDA tests pass on every pod (`145 passed`);
   - with no new flag, the config is unchanged.
2. **Met.** `--inspect` prints `register_drop` on the last checkpoint: each slot, and every slot, zeroed.
3. **Met.** (r) and (g) ran at seeds 1 and 2 under one sheet with a fresh FIT and pre-run review. Openings were read as the mean of the last three checkpoints, against (a) and claude-154's (o).
4. **Met on merge.** This REPORT leads with tokens/s, and the result has one notebook row.

**The item's own done-when is met** by (g) at both seeds, so the item closes with this unit.
