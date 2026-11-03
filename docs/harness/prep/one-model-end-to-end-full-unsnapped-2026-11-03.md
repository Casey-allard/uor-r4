# Prep sheet: the one model trained as served at full length on the unsnapped contract, (b) and (k2) (one-model-end-to-end, claude-266, 3 Nov 2026)

Lab: claude. Item: [one-model-end-to-end](../../roadmap/items/one-model-end-to-end.md) (#3407). Path step 4: one model trained as served, exported, served and scored end to end, with tok/s.

TEST FITNESS: FIT - fresh non-author adversarial review, round 2, of this sheet at `bde52c3cf`. Round 1, at `e4a352873`, was NOT FIT, with:
- a blocker: I-0 named a test by its file, `geometric_stack_native`, where cargo prints the module, `geometric_stack`;
- two majors: R6 could be reused from a failing export, and part B's absent-arm clause was dropped;
- six minors.

All are fixed. Round 2's three minors are fixed in this text: a pass is read as first tokens only, R6 runs on the raw export, and the crate is named for I-0's names. No bar moved.

PRE-RUN: PASS - fresh D67 review, round 3, of this sheet at `a283d74d8`, `job-266.sh` (SHA-256 `3e5f9a744860612568e752427132840186fc94e68f17901ff979723ba67186b9`) and `watch-266.sh` (SHA-256 `1f20f745a64c01be15a52572129a03b014831e8fcf973c99524a463c4077d9fc`).
- Round 1 failed on I-0's module path, the FIT blocker above, since fixed. Round 2 found no defect but left checks unread.
- Round 3 read the chain, the store inputs, the Bar's outcomes, the caps, the panel file names and both hashes.
- Its one major was that `uor-pod status` run without the Vast adapter shows Runpod only. With the adapter, 0 pods run on either provider, so it is cleared.
- Its two minors are fixed in the any-fail branch: an absent arm, and the readout the next unit needs.
- Launch: three 1 × RTX 5090 Vast pods, session claude-266, watched with `MAXSEC=25200 BUDGET=24 STALL=90`.

**What this run measures.** The object is the float model, trained as served on the unsnapped D11 contract for 8,192 steps, and its raw integer export. At 256 steps on this contract the export did not match the float model (I-3 missed on 32 and 14 of 310 against a bar of 8, [claude-265](../../loop/tasks/one-model-end-to-end/claude-265/REPORT.md)). While I-3 fails, the integer artifact is not the float model. So while any export fails I-3, every float reading here describes the float model only, and no reading is a served result or a step-4 claim. Even a pass supports only "first tokens match on DEV": I-3 does not test whole replies, which it reports beside its count.

**Why this run, under §4.4.** The I-3 line has three results: claude-263 (112 and 137 of 310), claude-264 (100 and 131) and claude-265 (32 and 14). None passes. §4.4's pivot note gives two choices, stop or one decisive run. This sheet is the one decisive run for step 4: the full-length model on the contract it is served on, with I-3 read on all six exports. No further 256-step I-3 unit runs on this line until the served-contract readout, the instrument the remaining parity sites need, is settled with Casey.

This sheet is claude-263's [part B sheet](one-model-end-to-end-full-served-2026-11-03.md) (FIT round 5, PRE-RUN PASS), which never ran. **Where a section below says "unchanged", part B's text holds word for word.** Three things change, each stated in its section:
- **the pin** moves to `59f7770ae2d936cfb3ac7337cd041cb3f25ead0f`, main with claude-265 (#3600) merged, so "trained as served" is the unsnapped D11 contract: gates passed through, transition unit-normalized;
- **the export** is raw (`--raw-gates --raw-transition`), the contract the model trains on, with I-6 checking each header;
- **an I-3 miss** no longer holds every reading unread. It still closes nothing, names nothing and changes no status (the Bar, under claude-266's decision card).

## Question

Part B's three questions, asked on the unsnapped contract:
- does keying the Hamming read's copy heads on two tokens (k2) make the trained-as-served model reply better on held-out phrasings than one token (b), at no cost in served speed;
- does the model, trained as served for 8,192 steps, serve on the laptop as it was trained (I-3);
- what does it do there: its decode tok/s and its transcripts.

The second answer is expected to be "not yet", so the Bar says what is read anyway, and how.

## Base artifact

A training comparison from scratch; the candidates are the checkpoints each arm produces.
- **The pin.** Training, smoke, I-0, scoring and snapshots use `train-in-space` and `fbs-snapshot` built `--locked --features cuda` at `59f7770ae2d936cfb3ac7337cd041cb3f25ead0f`. Its `crates/uor-r4-float-train`, `crates/uor-r4-integer`, `crates/uor-r4-tokenizer` and `Cargo.lock` equal claude-265's code commit `d4bb64ce2`. Each pod fetches the pin, checks it is that commit and clean, builds, and records the binaries' SHA-256.
- **On the laptop**, `fbs-export` and `fbs-toks` are built `--features serving-wasm`, and `uor-chat` `--release`, from a tree whose integer and tokenizer crates and `Cargo.lock` equal the pin's.
- **The outputs** go under `results/claude/one-model-end-to-end-full-unsnapped-2026-11-03/sN/`.

## Arms

Unchanged: (b) has copy key 1, (k2) `--hamming-copy-key 2`, both on part B's command at 8,192 steps. One 1 × RTX 5090 pod per seed (1, 2, 3); on each, (b) then (k2) run alone on GPU 0 at every stage ("Arms in turn"). Scoring is part B's serial command, then `fbs-snapshot CKPT SNAPSHOT.json`.

## Readings

- **R1 to R5:** claude-261's grade, unchanged (`grade.sh`, SHA-256 `3cd7663c93d2d5b8b4f59b8b4425a832d3f3be9505a91ad6af222a2fc2796e29`), p-w deciding and p-v confirming. They grade the float scores, so they describe the float model.
- **R6:** claude-261's timing protocol and speed condition (`timing.sh`), unchanged except that it runs on each arm's seed-1 raw export.
- **R7:** conv-a turn 3 under each arm's seed-1 raw export, and whether it names Maya. Reported only: the whole conv-a, conv-b and conv-c transcripts through `uor-chat --protocol 2`.
- **I-3** on all six raw exports, reported per arm and per seed: first tokens, whole replies and order mismatches against that arm's own `samples-memory.json`.

## Bar

**If every export made passes I-3** (exit 0, no order mismatch, at most 8 of 310 first tokens differing, and I-5 and I-6 hold), part B's table is read in order, unchanged: rows 1–4, with row 1's I-3 condition. That condition is read as "first tokens match on DEV", not as part B's "serves exactly as it was trained". An arm absent at a seed (it diverged, or ran out of memory alone on its card) makes no export there, and its seed goes to row 4, as part B's step 6 says.

**If any export made fails I-3:**
- **Nothing closes, nothing is named, no status moves** (§4.1, §4.2, L-007).
  - No row's consequence is applied, and no row, float or served, is named as met: no configuration is closed, and no base or candidate is named.
  - No PLAN, mechanism-board or notebook status changes on these readings.
  - No milestone claim or step-4 claim rests on them.
- **R1 to R5 are graded and reported** with their numbers and per-seed verdicts, each marked "float; served parity open".
  - They compare (b) and (k2) as float models, both through the same float forward.
  - No served ranking is inferred from the float ranking. An arm's I-3 count can differ from the other's, so the served models may not order as the float ones do.
  - A seed with an arm absent (it diverged, or ran out of memory alone on its card) has no per-seed verdict and is reported absent. It makes no export, and so no I-3 count.
- **R6 and R7** are read on each seed-1 raw export, each labelled by that export's own I-3 result:
  - an export that fails I-3: R6 "served tok/s of a raw export that fails I-3 (n/310); not the trained model's speed as trained", R7 "integer transcript, I-3 FAIL n/310; no capability claim";
  - an export that passes I-3 while another fails: R6 "served tok/s, I-3 PASS n/310; the table is unread", R7 "integer transcript, I-3 PASS n/310; the table is unread".

  Whether R7 names Maya is reported only. The float scorer reads DEV and the panels, not conv-a, so no float reply exists to print beside it.
- **The six I-3 counts** are the parity reading at full length, set beside claude-265's 32 and 14 at 256 steps.
- **The next unit localizes the fault.** Localizing needs the served-contract readout, which is with Casey ([#2028](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6607309869)). So the next unit on this line starts only once the readout is settled; other work goes on meanwhile. Part B's two branches hold, unchanged:
  - **Integer side only:** that unit repairs the integer side at a commit it names. It re-exports every arm at every seed from the sealed snapshots, with the repaired `fbs-export` and `fbs-toks` and the laptop pin moved to that commit, and runs I-3 on each. If every one passes, it reads R6 and R7 again on the re-exports, and reads this table in full with the R1 to R5 graded here, now unmarked. The labelled R6 and R7 read here are void for the table.
  - **Float side, or not localized:** R1 to R5 are void as served results; their float label stays. The unit after the localization re-trains both arms under this sheet at the repaired forward's commit, which replaces the pin, and runs I-0 again there.

**What counts as an I-3 failure:** unchanged from part B, with I-5 and I-6 added: an `fbs-export` or `fbs-toks` refusal or non-zero exit, an order mismatch, more than 8 of 310 first tokens differing, a header whose copy key is not the arm's (I-5), or a header without `raw_gates: true` and `raw_transition: true`, or with a `transport_snap` (I-6).

**Reversal.** Casey can say "hold": no further pod launches, and nothing is closed or named on these readings. The checkpoints, snapshots and float samples stay sealed under the results root above. The next parity unit re-reads them: the integer-side branch re-exports from the snapshots, and a served-contract readout, once Casey settles it, reads the same checkpoints.

## Instrument checks

- **I-0** runs the CUDA library suite at the pin on every pod, as part B. It passes only if at least one test ran, none failed, and each of these eight is `ok`:
  - claude-261's `copy_key_two_tells_two_facts_after_the_same_token_apart` and `pair_halves_puts_the_first_token_in_each_copy_heads_low_half`;
  - claude-264's `served_fbs_projections_round_to_the_residual_grid_on_cuda` and `served_projection_rounding_is_the_integer_gather_on_cuda`;
  - claude-265's four: `served_bridge_takes_the_unsnapped_d11_contract`, `the_training_forward_selects_and_restores_the_served_bridges`, `a_step_as_served_reaches_every_projection_after_a_requantized_activation` and `served_surrogate_gradients_are_exact_on_cuda`.

  Each name is defined once in `uor-r4-float-train` at the pin, and I-0 runs that crate's library only (`-p uor-r4-float-train --lib`), so each is matched as the last segment of cargo's whole line, `test <module path>::<name> ... ok`, whatever the module path. A file's module can differ from its name: `geometric_stack_native.rs` is `mod geometric_stack`.
- **I-1 (the smoke)** is unchanged.
- **I-2 (the projection)** is unchanged. claude-265's trainer clock read 30 s for 256 served steps on this contract (about 26,000 context tok/s), so 8,192 steps take about 16 minutes before evals, and no cut is expected.
- **I-3** is part B's, on raw exports: `fbs-export --raw-gates --raw-transition --run-config CONFIG SNAPSHOT OUT`, then `fbs-toks ARTIFACT --replies PROMPTS --out FILE --reply-tokens 64` on DEV's prompts from the arm's own `samples-memory.json`.
- **I-4 (pairing)** is unchanged.
- **I-5:** claude-261's, unchanged: (k2)'s header holds `"copy_key": 2` in its `hamming` record, and (b)'s holds no `copy_key`.
- **I-6:** each header's shape has `raw_gates: true` and `raw_transition: true`, with no `transport_snap`, as in claude-265.

## Data split

Unchanged: claude-261's nine pins for TRAIN, DEV, the tokenizer, p-w and p-v, checked against their `SHA256SUMS`. No key was chosen on either panel.

## Steps

1. On each pod, fetch the pin, build, record the binaries' SHA-256, run I-0.
2. Run I-1's smoke for (b), then (k2); project I-2.
3. Train (b), then (k2), for 8,192 steps each (or I-2's equal cut).
4. Score each in turn on DEV, p-w and p-v, memory on and out; snapshot each.
5. Seal every output with `SHA256SUMS` under the store root, read it back, and only then take the pod down.
6. On the laptop: export each arm at each seed raw; run I-3, I-5 and I-6; grade R1 to R5; read R6 and R7 on the seed-1 exports; then apply the Bar. An arm absent at a seed makes no export there, and its seed goes to row 4.

## Minimum test power

Unchanged: three seeds, with 1,202 paired p-w prompts and 393 p-v prompts per seed, so R1 detects a net shift of about 3% of the panel. Each I-3 has 310 paired prompts.

## Cost

Re-estimated, since part B's scoring figure came from the broken forward. Each pod takes about:
- 10 minutes for I-0's suite, and 10 to build and fetch;
- 6 to 8 minutes for the two smokes in turn (claude-265: 74 s wall to train 256 served steps);
- 2 × 18 to 30 minutes of training alone (16 minutes of steps at claude-265's rate, plus eight evals and start-up);
- 2 × 70 to 127 minutes of serial scoring. The low end scales claude-265's DEV-only scoring, 647–705 s for 310 prompts, to DEV, p-w and p-v's 1,905 prompts. The high end is claude-261's 7,106–7,591 s for one arm over the same three splits.

- about 20 minutes for the two snapshots, the seal, and the pull off the pod before it goes down.

That is about 3.8 to 6.1 hours a pod. At 15:05Z the cheapest verified 5090 offer was $0.975 an hour, so at $0.98–1.10 an hour a pod that is about $11–21 in all, and at most $3.30 an hour for the three pods together. At 15:05Z no pod was running, so this is inside 4 pods and $8 an hour; `uor-pod status` is read again at launch. The watcher's wall limit is 7 hours (`MAXSEC=25200`) and its spend limit $24. Each lease is taken for 6 hours and renewed. A budget or time stop wakes the session to extend a healthy run, never to cancel it.

## Stop rule

Unchanged from part B. In addition, an I-6 refusal is an I-3 failure, read as the Bar says.
