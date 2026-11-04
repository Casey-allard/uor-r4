# Prep sheet: the exact decode cache in scoring, on one GPU (one-model-end-to-end, P3 cache unit 2, claude-279, 4 Nov 2026)

Lab: claude. Item: [one-model-end-to-end](../../roadmap/items/one-model-end-to-end.md) (#3407). This is P3 (faster scoring) of Casey's 4 November plan, cache unit 2 of 2: scoring's decode steps from [claude-278](../../loop/tasks/one-model-end-to-end/claude-278/REPORT.md)'s exact step cache wherever the window fits.

TEST FITNESS: FIT WITH CHANGES, from a fresh non-author review of this sheet at `15a60735` ([#3407, 6638239076](https://github.com/UOR-Foundation/uor-r4/issues/3407#issuecomment-6638239076)). It found no blockers and five instrument SHOULDs, all folded in here: I-4 (coverage), I-3b (a failure is not noise), the logs beside `--out`, plain-text lines, and what the gate leaves out.

PRE-RUN: GO in round 2, from fresh non-author reviews of this sheet, the ORDER, the job and the grade ([#3407, 6638389538](https://github.com/UOR-Foundation/uor-r4/issues/3407#issuecomment-6638389538)). Round 1 was NO GO, until I-3b and I-4 were in the scripts. Round 2 read the sheet at `58fa1523`, the job `c0d48b27`, the grade `7188e1db` and the watcher `7735c645`.

RUN 1 (4 Nov, 18:17–18:34Z, `vast-59143240`, about $0.30): EXIT 5 at I-2, an instrument fault. `on-L192` decoded with 0 prefills and 0 steps (6,797 whole-window forwards), so the cache never ran.
- **Cause:** the scorer sets the one-atom FBS arm on every pass (`score_rows`), and the cache refused any arm. The GPU tests had left the arm unset.
- **Repair (`11aed950`):** that arm projects through the same `plan.forward` as no arm (`spiralcore_fbs_arms::project`), so the cache now carries it, and refuses only the arms with kernels of their own. The CPU refusal test fails with the repair set aside. The CUDA decode-switch test now sets the arm as the scorer does.
- **What held:** I-0 (the CUDA library suite: 338 passed, 0 failed, 8 ignored; the 17 named tests) and I-1.
- **Sealed** in the store under `claude-279/run1/`: 155 files, read back 152 of 152 against `SHA256SUMS.files` (SHA-256 `e5151c7626bbf641fdcdfb2f2634e4f929937e5bb6fff700cc6cb02f6f257a6e`).
- **Run 2** runs the new pin on the same pod, into fresh folders and under `claude-279/run2/`, after a fresh pre-run review of the change.

OUTCOME: **Exact and faster** (run 2, 18:49–19:57Z, EXIT 0). See the [REPORT](../../loop/tasks/one-model-end-to-end/claude-279/REPORT.md).
- **G** held at L192, S192 and L2048, and **G0** at L192 and S192. I-0 to I-4 hold.
- **The 2,048 queries files:** 98.66 and 98.81 tok/s with the cache on, against 16.68 and 16.75 off.
- **Steps:** 99.2% of the L2048 forwards after the prefills.
- **`--decode-cache`** stays at its default, 1.
- **Run 2's seal:** `claude-279/run2/`, 335 files, read back 335 of 335. `SHA256SUMS.files` has SHA-256 `2f184e2ec21631075d6497236e380a5486c555ffec5f4aa8320f31d877f8c20b`.

**What this run measures.** Today every decode step of a scoring pass re-runs the model over the whole window: 2,048 positions at P2b's long window, to compute one new position. At 2,048 the two `p-long` queries files are about 98% of a pass (claude-277). Of their 18,879 decode steps after the first, 18,721 (99.2%) fit the window, so they can step from the cache instead; at 192, 6,105 (32%) do.

The build ([ORDER](../../loop/tasks/one-model-end-to-end/claude-279/ORDER.md)) adds three things:
- the Hamming read's step;
- the decode switch in the batched decode, which prefills each group's prompt, steps while the window fits, and re-runs the whole window from the first step that would slide it, or that the cache refuses;
- `--decode-cache 0|1` on `train-in-space`, default 1.

Each is exact by construction: a step runs the same ops on the same tensors as the whole window, and cuts only the query rows. This run checks that on the GPU, bit for bit in the tests and byte for byte in the scored files.

**What the gate leaves out.** The serial decode (`decode_reply_with_mode`) keeps the full re-run. It serves the forced, keyed and served arms (files named `samples-*-forced-*`, `samples-*-keyed-*` and `samples-*-served-*`) and the read scopes. G compares their files too, which confirms they did not change, but it does not test the cache on them. R3 gives their share of the pass's decode seconds. It re-scores P2b's sealed seed-1 checkpoints, unchanged, with P2b's commands. No training runs. The object is the scorer's decode, not the model, and no model reading rests on this run.

## Question

1. Does every step the cache takes equal the whole window's position, bit for bit? This covers P2b's two Hamming reads with random parameters, the turn memory, the retention reading and the served surrogates, through a window slide.
2. Are P2b's scored files byte-identical with the cache on and off, at 192 and at 2,048, and identical to the files the base commit writes?
3. How many decode tok/s does each queries file gain at each window?

## Base artifact

- **The pin** is `11aed950ae7a1ae83f9fff672d27c54f02b33590`, the build's commit on draft PR #3673. Run 1 ran the first pin, `06b225d17eb9f9bdaac30fa27efe8953aa50c57e`; the repair above moved it. `train-in-space` is built there with `--locked --features cuda`. The pod checks that the commit is that one and clean, and records the binary's SHA-256. A later commit on the PR that touches only docs needs no rebuild. One that touches code moves the pin, and this sheet names the new one before the job runs.
- **The base** is `f34f0b17c76cac6c28e82fbc754163023948ba65`, `main` at claude-278's merge, the ORDER's base. Its `train-in-space` has no `--decode-cache`: it re-runs the whole window at every step, and its Hamming read has no step. It is built into the same target folder after the pin's binary is copied out.
- **The checkpoints** are P2b's seed 1, under `results/claude/one-model-end-to-end-long-context-2026-11-04/s1/`:
  - (L) `L/models/step-008192/`;
  - (S) `S/models/step-021845/`.

  The seal file `SHA256SUMS.files` there has SHA-256 `ca0b8aeda4e23a6f793df8b503534a48ff090724632dc384c66a0b4b27424b23`. The pod checks the ten checkpoint files against its lines.
- **The outputs** go under `results/claude/one-model-end-to-end-decode-cache-2026-11-04/claude-279/` (`run1/`, `run2/`), a new folder beside claude-278's sealed `run1/` and `run2/`, which stay untouched.

## Arms

Every pass runs P2b's scoring command on GPU 0: `--score-rows CKPT --out DIR --samples 16 --reply-tokens 32 --retain-rates 1 --seed 1 --panel p-long=PLONG --panel-queries p-long=PLONG/queries.json`. The passes at 192 add `--context 192 --panel p-v=PV`. The passes at 2,048 add `--context 2048`. Each pass writes to its own new folder, and runs alone: the passes are serial.

- **base:** the base binary, as P2b scored.
- **off:** the pin with `--decode-cache 0`, which re-runs the whole window at every step.
- **on:** the pin with `--decode-cache 1`, the new default.

The order on the pod is:

| # | Pass | Binary | Window |
|---|---|---|---|
| 1 | base-L192 | base | 192 |
| 2 | off-L192 | pin, 0 | 192 |
| 3 | on-L192 | pin, 1 | 192 |
| 4 | off-L192-again | pin, 0 | 192 |
| 5 | base-S192 | base | 192 |
| 6 | off-S192 | pin, 0 | 192 |
| 7 | on-S192 | pin, 1 | 192 |
| 8 | on-L2048 | pin, 1 | 2,048 |
| 9 | off-L2048 | pin, 0 | 2,048 |

The base runs first, so it takes the driver's cold start of the PTX (built ahead by nvcc for `compute_120`, no NVRTC), not a pass that is compared for speed. on-L2048 runs before off-L2048, so a slow off pass cannot hold up the reading the cache is for.

## Readings

- **G (the gate): on equals off, byte for byte.** For each of L192, S192 and L2048, take the SHA-256 of every file under on's `--out` folder and off's. They must be equal file for file, with the same file names.
  - Each pass's printed lines (`scored.jsonl`) must also be equal, once each line's `decode[1]` and `search[1]` are dropped. Those are the wall seconds of a samples file's decode and of a served key's searches. Every other field is a count, a hash or a window. A printed line that is not a JSON object (`[recall] ...`, `[trace] ...`) is compared as it stands.
  - **No clock reading is inside `--out`.** At the base, `score_rows` writes its files only through `write_new(&out.join(..))`: positions, targets, picks, reply texts and recall counts. The seconds go to stdout. claude-277 checked this for the same command and the same 19 files.
  - **The logs sit beside `--out`, never inside it.** Each pass's stdout, stderr, exit code and clock readings go to `DIR.log/`, next to its `--out` folder `DIR`.
  - The laptop's grade recomputes every SHA-256 from the files read back off the pod. It never trusts the pod's own lists, and a missing or empty folder never compares as the same.
- **G0: the whole window is as before.** base-L192 equals off-L192, and base-S192 equals off-S192, byte for byte, with the same rule as G. The pin's Hamming read routes a whole window through the call the base makes, and the rest of the whole-window path is unchanged, so G0 checks that nothing else moved.
- **R1 (tok/s first): decode tok/s per file, both sides.** Each samples and queries file prints a `decode` pair (tokens, seconds). R1 gives tok/s = tokens / seconds for every file of every pass, and on's tok/s over off's per file.
  - **The deciding reading is L2048's two `p-long` queries files.** P2b timed one pass at 2,048 within 2 s across two pods, so a single pair reads cleanly there.
  - At 192, P3(a)'s two serial repeats differed by 31% (claude-277), so the 192 ratios are reported but decide nothing.
- **R2: the forwards.** Each pass of the pin prints `decode_forwards` on stderr: the cache's prefills, its steps, and the whole-window forwards. On each on pass, steps / (steps + whole) is set against the count from `queries.json` (99.2% of steps after the first at 2,048, 32% at 192). On each off pass, prefills and steps must both be 0 (I-2). I-4 gates the coverage these counts show.
- **R3: the whole pass.** Each pass's wall seconds and GPU memory peak, on against off, and the serial decode's share of its decode seconds.
- **Reported, not gated:** base-L192, base-S192 and off-L2048 against P2b's sealed files from another pod (claude-277 found two 5090 pods disagreeing on 26 of 15,961 top-1 picks with one binary).

## Bar

The first outcome that applies is the reading.

- **Unread (instrument):** I-0 to I-3, or I-3b, fails. No reading counts. The REPORT classifies the failure. The fault is repaired, and the run is repeated.
- **The cache is not exact (G fails):** any scored file or printed line differs between on and off, at a window whose off pass repeats itself (I-3 at L192, I-3b at S192 and L2048).
  - `--decode-cache` defaults to 0 in this unit's PR, and the REPORT names the first file and row that differ.
  - P4 scores with the full re-run.
  - The cache is repaired in a new unit, and this run is repeated there.
- **Unread (coverage) (G holds, I-4 fails):** the on passes did not step as the switch predicts, so G did not test the cache where it claims to. `--decode-cache` defaults to 0, and the REPORT names the refusal that took the steps, from R2.
- **The whole window moved (G0 fails, G holds):** the pin's off pass differs from the base. Since I-3 holds, the scorer is deterministic on this pod, so the build changed a whole-window result.
  - The PR does not merge until the cause is found and fixed, and the run is repeated.
  - The REPORT names the first file and row.
- **Exact and faster (G and G0 hold, and on's tok/s is above off's on both L2048 queries files):** the default stays 1. From P4 on, this line scores with the cache. The REPORT gives R1 to R3.
- **Exact but not faster (G and G0 hold, and on's tok/s is at or below off's on either L2048 queries file):** `--decode-cache` defaults to 0, since an exact cache that gains nothing only adds a path. The REPORT names where the time goes, from R2 and R3. The step's own cost is the next question.

Whatever the outcome:
- no model, recall or served reading rests on this run;
- the REPORT gives every number, negatives first.

**Reversal.** Casey can say "hold": no further pod launches, and `--decode-cache 0` restores the full re-run on any later pass. The outputs stay sealed under the results root.

## Instrument checks

- **I-0: the CUDA library suite at the pin,** `cargo test --locked --features cuda -p uor-r4-float-train --lib --no-fail-fast -- --test-threads=1`. It passes when at least one test ran, none failed, and each test named here is `ok`.
  - **The cache's gates:**
    - `the_step_cache_matches_the_whole_window_bit_for_bit` (G1, 96 cases: with and without P2b's two random Hamming reads, served or plain, with and without retention, with and without the reply table, prompts of 4, 7 and 33, batch 1 and 3, 12 steps each);
    - `the_step_cache_carries_the_two_token_key_and_the_type_spin` (G1b, 8 cases);
    - `the_step_cache_steps_to_the_window_and_refuses_the_slide` (G2);
    - `a_flipped_state_bit_or_a_zeroed_input_row_fails_the_compare` (G3: a scan-state bit, a convolution input, and a bit in a carried read residual);
    - `the_two_position_scan_kernel_is_the_whole_scans_next_position`;
    - `the_decode_cache_decodes_the_same_replies_through_a_window_slide` (the decode switch: a random model with the reads and the turn memory, four decode rules, plain and served, at a 16-position window);
    - `exact_length_batches_match_one_at_a_time_decodes` (now with the cache on).
  - **P2b's nine scoring tests,** as named in claude-277's sheet.
  - G3 is then run alone with `--nocapture`, so its ungated line on the old lowest-bit flip is kept.
- **I-1 (the inputs):**
  - `sha256sum -c SHA256SUMS` passes in each input folder;
  - every SHA-256 named in the Data split equals its file's;
  - each of the ten checkpoint files matches its line in P2b's seal.
- **I-2 (the passes):**
  - every pass exits 0;
  - each pass's `scored.jsonl` holds a `decode` pair for both `p-long` queries files;
  - each pin pass's stderr holds one `decode_forwards` line;
  - each off pass reads 0 prefills and 0 steps, and each on pass more than 0 steps. The zero is a hard check, not a formality: the switch and its counts are per thread, so a decode moved onto another thread would ignore `--decode-cache 0`, and only this check would show it.
- **I-3 (the gate can read):** off-L192 and off-L192-again are byte-identical, file for file, by G's rule. If they are not, neither G nor G0 can tell the cache or the build apart from the scorer's own noise.
- **I-3b (a failure is not noise):** S192 and L2048 have no repeated off pass by default. If G or G0 fails at either window, the job runs one more off pass there (off-S192-again, off-L2048-again) before the seal. If it differs from the first off pass, the scorer is not deterministic at that window, and the run is unread. If it equals it, the failure stands.
- **I-4 (the gate tested the cache):** read only where G holds.
  - On each window, the on pass takes as many forwards as the off pass, one per decode step: prefills + steps + whole on equals whole off.
  - At 2,048, at least 90% of on-L2048's forwards after the prefills are steps, as predicted: the queries files are about 98% of the pass and 99.2% of their steps fit.
  - At 192, more than 0 steps (I-2). The REPORT gives the share.
- **The base is a different binary:** the job stops if the base's `train-in-space` is byte-identical to the pin's, so a stale copy in the shared target folder cannot pass as the base.
- **The signal trap.** If the job is stopped, it first stops the pass that is running, waits for it, and only then seals what it wrote.

## Data split

The scoring inputs are P2b's, fetched from the store and checked by their SHA-256s, as in claude-277's job:
- **Driver data,** `w2-inputs/2026-10-25/driver-data/`:
  - `tokenizer.json` `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`;
  - `mw-balp/train/tokens.u16` `3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8`;
  - `mw-balp/train/response_mask.u8` `b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3`;
  - `ft/dev/tokens.u16` `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b`;
  - `ft/dev/response_mask.u8` `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078`.
- **p-v,** `inputs/claude/heldout-panel-p-v-2026-10-30/p-v/`:
  - `tokens.u16` `3488eef5bba82c3d3b97633fbd92f568c237636e6bb44d9decbdb8886d4196f0`;
  - `response_mask.u8` `c170fd0ac9568f98fa3e2ab5f22af0fe31b94ea14bbbf802daf446e20d5ca023`.
- **p-long,** `results/claude/one-model-end-to-end-long-context-2026-11-04/inputs/p-long/`:
  - `tokens.u16` `8b2529b1a1385112ef41a42eca50dc982dc474f3b40e8948b8887b6b4158ab36`;
  - `response_mask.u8` `b17ff53fc37dfe2910b1ffca820538ca5473d1b0b707da3f06eb83c25c58fed0`;
  - `queries.json` `cf5c30b96e98f7770314f96be5114cf5eeb21e3f9657aeb84b4c1d6168ca2390`;
  - `user_turns.json` `4d21bc632073c6384e0193960c862ddbb326ecedf4ed27b6af30811319207800`.

Nothing is held out and nothing is graded for recall, so no split is spent.

## Steps

1. On one RTX 5090 pod, fetch the pin, build, record the binary's SHA-256, and run I-0 and G3's line.
2. Fetch the base, build it into the same target folder, and record its binary's SHA-256.
3. Fetch the inputs and the two checkpoints, and run I-1.
4. Run the nine passes in the Arms' order, one at a time. Record I-2 after each.
5. Seal every output with `SHA256SUMS.files`, put it in the store, read it back, and only then take the pod down.
6. On the laptop: check I-3, G and G0, then compute R1 to R3 and apply the Bar.

## Minimum test power

G, G0 and I-3 are exact compares, so a single pass of each side decides them. One differing byte anywhere fails the gate. The tests in I-0 are exact compares too.

R1 decides only one thing: whether on is faster than off at 2,048. The cache replaces about 99% of that window's queries steps with steps of one position. So a real effect is far larger than the 2 s spread P2b saw between pods on one 2,624 s pass.

## Cost

The pod takes about:
- 10 minutes to fetch and build the pin, and 15 for I-0, whose G1 now runs 96 cases;
- 5 minutes to build the base;
- 5 minutes to fetch the inputs and checkpoints;
- 15 minutes for the six passes at 192;
- 5 to 20 minutes for on-L2048, and 45 for off-L2048 (P2b's pass took 2,624 s);
- 15 minutes to seal and read back;
- only if G or G0 fails at S192 or L2048 (I-3b): 5 more minutes for off-S192-again, and 45 for off-L2048-again.

That is about 2 to 2.2 hours, or about 3 if I-3b runs both repeats. Verified 5090 offers ran from about $1 an hour at this sheet's writing, so it costs about $2 to $2.4, inside the ORDER's $3. `uor-pod status` is read at launch, inside 4 pods and $8 an hour. The watcher's wall limit is 4 hours (`MAXSEC=14400`), its spend limit $4, and the lease is taken for 4 hours and renewed. A budget or time stop wakes the session to extend a healthy run, never to cancel it.

## Stop rule

- **A failed instrument check** (I-0 to I-2) ends the job at once, and it seals what it wrote. The fault is repaired, and the run is repeated at the repaired job.
- **A failed I-0 cache gate** (G1, G1b, G2, G3 or the decode switch) also ends the job, before any pass. That is the ORDER's first stop rule: if a step's bits differ because of an op whose result depends on the launch's shape, the Hamming read stays refused in the step, and the REPORT names the op.
- **off-L2048 running past the wall limit** is extended, never cancelled. If the pod dies during it, the on pass's reading stands, and off-L2048 is run again alone at the same pin. G at 2,048 is then read against it.
- **A pod that fails to start** is replaced by one more, once, inside the caps.
- **The pod passing about $3** is the ORDER's third stop rule: the session extends a healthy run and notes it for Casey.
