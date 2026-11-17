# Prep sheet: R\*, the sealed modified Kneser–Ney reference of bar Q (written-state-count-model, claude-422, 17 Nov 2026)

Lab: claude. Item: [one-model-end-to-end](../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [claude-422](../../loop/tasks/written-state-count-model/claude-422/ORDER.md). Code: branch `claude/kn-sealed-422`, off `main` at `9465b92d6`. Release cited at dispatch: D86 (§3.10), wick is the owner's GPU.

TEST FITNESS: FIT - round 2 (fresh-context non-author, at `57b85a395`) found no blocker. Its three should-fix findings and two notes are folded in below:
  - (1) Every run of the oracle test, not only step 4b, first checks the sha256 of its TRAIN and query streams.
  - (2) The stall rule also requires the job's CPU time to have stopped, because the oracle phase runs on the CPU and prints only at its end.
  - (3) The sealed run's failure cases are separated. A `--check-host` mismatch is an instrument failure, and only an exit before the "gpu scored" line is a crash. Replacing an invalid first JSON takes a decision card that amends the ORDER's seal, and the first JSON stays on record.
  - (Notes) B1's `jq -e` check is written out. Step 3's line also checks that its inputs are the ten runs of one sweep.

  Round 2 checked that every round-1 item is in the text, that the step-3 line is exact, that the 4a slice equals the oracle's, and that either negative reaches the equality assert.
- Round 1 (fresh-context non-author, at `f649077c4`): NOT FIT, two blockers and five should-fix findings, all folded in below:
  - (1, blocker) The N\* choice is now written out as one `jq` line in step 3. It compares `bits_q12` as numbers and asserts equal bytes.
  - (2, blocker) A later R\* never replaces the sealed one. A change to bar Q's reference is a decision card, and none may land once a model has been scored on DEV-test.
  - (3) The oracle test's query sha256 is checked to be DEV-tune's before step 4.
  - (4) A 2‰ instrument run checks that every order up to N\* is used, and fixes the negative's branch before it runs. The negative must fail at the equality assert.
  - (5) The oracle's peak memory and time are logged, with a 1‰ fallback.
  - (6) The 2-hour cap became a stall rule (AGENTS.md: never cancel a healthy run).
  - (7) R\*'s record names its code and the sha256 of its JSON.

  Notes kept: N naming and the key limit are fair; hashed exact counts stand in for suffix-array counts; the PPMd anchor is its own unit.

## The question

What is R\*'s BPB on sealed D87 DEV-test, on all targets and on the novel subset? Bar Q (claude-409 DECISION §2) asks a model for at most 0.80 × that number, so R\* must be sealed before any model is scored on DEV-test.

R\* is defined as follows. It is interpolated modified Kneser–Ney:
- three discounts per order, from that order's continuation count-of-counts n1..n4 (Chen–Goodman);
- full N1/N2/N3+ counts per context, unpruned;
- adjusted (continuation) counts below the top order; raw counts at the top order and for grams that start at BOS;
- the uniform 1/4096 floor under order 0;
- integer Q48 masses, with bits by `lawful_likelihood::bits_q12`.

It includes **claude-421's fallback rule**. An order whose n1..n4 holds a zero, or whose Chen–Goodman estimate D_i leaves [0, i], takes KenLM's `--discount_fallback` discounts (1/2, 1, 3/2) for the whole order. On all of TRAIN at N = 2, order 0 takes the fallback (n1..n4 = [7, 6, 11, 11]).

**N** is the longest context, in tokens: an N of k scores (k+1)-grams. The decision card expects 8–12 as n-gram length, which is N = 7..11 here. The key holds at most 10 context tokens (`NGRAM_MAX_ORDER`), so the sweep is N = 1..10.

The counts are exact. They are hashed per partition on the GPU and assembled in canonical order, not taken from a suffix array, but they give the same numbers. The digests and SHA are claude-418's.

## Steps

All on wick's RTX 4090, from a source tarball of this branch's head. The instrument is `docs/labs/kn-reference/instrument` (claude-421). The job script hardcodes every stream path under `/workspace/out/c420/run1/`.
1. **Build and CUDA tests.** `cargo test --release --features cuda --lib kn_score` passes (claude-421's 10 tests). The real-slice test also runs at its defaults (N = 2, 1%), to show the new settings left claude-421's check as it was.
   - **Done at `f649077c4`** (job `c422-build`), before this sheet's FIT: 10 / 10 pass. The defaults check gave claude-421's slice exactly: 107,280 query tokens, 107,257 targets, order-0 n1..n4 [58, 64, 59, 57]. That job named the DEV-tune path in its script, and a query of that size can only be DEV-tune's 1% slice. No score was read from it beyond equality with the oracle.
2. **Sweep on DEV-tune**, N = 1..=10:

   `kn-reference train.tokens.u16 dev-tune.tokens.u16 id-bytes.u8 dev-tune.novel.u8 N 134217728 sweep-N.json --check-host`

   This uses the same budget as claude-421's scale run (2^27 positions per partition). Each run counts all of TRAIN at orders 0..N and scores all of DEV-tune. The host scorer checks every mass. After each run, this check tests B1, and the sweep stops at the first failure:

   `jq -e --argjson n N '.per_mille == 1000 and .check_host == true and .sealed_run == false and .highest_order == $n and .budget == 134217728 and .inputs.train_tokens == 2091560001 and .inputs.train_sha256 == "c6cba29e…" and .inputs.query_sha256 == "75c6d1ac…" and .inputs.classes_sha256 == "7df42437…" and .inputs.id_bytes_sha256 == "befb22f6…" and .all.targets == 10807417 and .all.bytes == 21928475 and .novel.targets == 7566174 and .novel.bytes == 15995822' sweep-N.json`

   The script carries the full 64-digit shas. The sealed run's check is the same with `.sealed_run == true`, DEV-test's query and novel shas (`023784b7…`, `5bf3881b…`) and DEV-test's totals.
3. **Choice.** N\* is the N whose `all.bits_q12`, read as a number, is smallest. A tie goes to the smaller N. This is the line, run in the folder of the ten files:

   `jq -s -r 'if length == 10 and (map(.highest_order) | sort) == [1,2,3,4,5,6,7,8,9,10] and (map(.budget) | unique | length) == 1 and all(.[]; .per_mille == 1000) and (map(.all.bytes) | unique | length) == 1 then (sort_by((.all.bits_q12 | tonumber), .highest_order) | .[0].highest_order) else error("not the ten runs of one sweep") end' sweep-1.json sweep-2.json sweep-3.json sweep-4.json sweep-5.json sweep-6.json sweep-7.json sweep-8.json sweep-9.json sweep-10.json`

   `tonumber` is exact here: the totals are near 2^38, below 2^53.
4. **Oracle at N\*.**
   - **a. The slice's coverage.** First, `kn-reference … dev-tune.tokens.u16 … dev-tune.novel.u8 N* 16777216 slice-nstar.json --per-mille 2 --check-host` runs on exactly the oracle test's slice. Both cut at the first BOS at or before the 2‰ mark, by the same rule. Every order 0..N\* must have `kept_contexts` > 0, so the check reaches the top order.
   - **b. The check.** The ignored test `kn_score_cuda_real_slice_equals_the_cpu_oracle` with `UOR_KN_N=N*` and `UOR_KN_PER_MILLE=2`. It requires every GPU mass to equal `KnOracle`'s. `KnOracle` is the independent CPU oracle: ordered maps keyed by the context tokens, with its own discount rule.
     - The test itself has no seal guard. So **every** run of it (steps 4b and 4c, the Short reading, and any rerun) first logs the sha256 of the files in `UOR_KN_QUERY` and `UOR_KN_TRAIN`. They must be DEV-tune's (`75c6d1ac…`) and TRAIN's (`c6cba29e…`), or the job aborts before the test starts.
     - The job logs the test's peak resident memory and its time. If the oracle runs out of memory (the container's limit is 16 GiB), steps 4a and 4b run again at 1‰, with the same pass rule.
   - **c. Negative.** The branch is fixed from 4a's JSON before the negative runs. If some order has `discount_fallback` true and no zero in its `count_of_counts`, the out-of-range branch is live: `oracle_discounts`'s range test is replaced by `true`. Otherwise the oracle's gamma is set to 0.
     - The negative passes only if the test fails at its equality assert, "the GPU path differs from the CPU oracle". A panic elsewhere does not count. If the range negative ends in another panic (for example an oracle mass that does not fit u64), that is recorded as "branch live, comparison not reached", and the gamma negative is run as well and must fail at the equality assert.
     - Each negative builds into its own fresh target folder. Afterwards the oracle source is restored, `cmp`-identical to the tarball, and the check passes again.
5. **Reproducibility.** A second DEV-tune run at N\*, into a new file, must be `cmp`-identical to the first.
6. **Sealed run.** Run once only:

   `kn-reference train.tokens.u16 dev-test.tokens.u16 id-bytes.u8 dev-test.novel.u8 N* 134217728 sealed.json --check-host --sealed-run`

   It runs only after steps 1–5 pass.
7. **Storage.** The JSON files and logs are fetched to the laptop and committed under `docs/loop/tasks/written-state-count-model/claude-422/run1/`. They hold counts, digests and hashes, never a stream, mask or table. Wick keeps the outputs at `/workspace/out/c422`.

## Bar

This unit is a measurement: it produces R\*. Its bars decide whether the sealed number is valid, not whether it is good.
- **B1 instrument, every run:** exit 0, and host = GPU at every target.
  - Targets and bytes equal claude-420's counts:
    - DEV-tune: all 10,807,417 targets / 21,928,475 bytes; novel 7,566,174 / 15,995,822;
    - DEV-test: all 21,349,576 / 43,326,982; novel 14,795,227 / 31,328,319.
  - `train_tokens` is 2,091,560,001.
  - Every input's sha256 equals claude-420's:
    - TRAIN `c6cba29e…`;
    - DEV-tune `75c6d1ac…`;
    - DEV-tune novel `7df42437…`;
    - DEV-test `023784b7…`;
    - DEV-test novel `5bf3881b…`;
    - id-bytes `befb22f6…` (claude-420's `id_bytes_sha256`, and claude-421's scale-run input).
- **B2 oracle:** step 4a covers every order. Step 4b passes at N\*. Step 4c's negative fails at the equality assert, and the restored check passes.
- **B3 reproducibility:** step 5's files are `cmp`-identical.
- **Recorded, with no bar:**
  - for each N: all and novel BPB (micro-bits per byte) and Q12 bits;
  - for each order: grams, kept records, discounts and the fallback flag;
  - counting and scoring times, and peak device memory (the whole GPU, with any other user's share named);
  - the oracle's peak resident memory and time.

**Reading map** (fixed before the run):
- **P (sealed):** B1–B3 hold on DEV-tune, and the sealed run passes B1. R\*'s record holds:
  - the head commit (`head.txt`);
  - the sha256 of `sealed.json`;
  - N\*, and the DEV-tune JSON at N\*;
  - from `sealed.json`: DEV-test's all and novel `bits_q12`, `bytes` and `bpb_micro`, `masses_sha256`, the per-order digests, discounts and fallback flags, and the input shas.

  The JSON's `"unit": "claude-421"` names the instrument's unit, not the run's, and the REPORT says so. The REPORT also writes out the bar Q lines in micro-BPB, for all and for novel: 0.80 × R\* (the bar), and the 0.90 × and 0.95 × checkpoints. Each is rounded down to an integer.
- **I (instrument):** B1, B2 or B3 fails on DEV-tune. The instrument is repaired under this sheet, and DEV-test is not touched until all three pass. If the sealed run fails B1, its number is not R\*. The failure is classified, and a repaired sealed run needs a new sheet, which names the first run.
- **Edge:** N\* = 10, the key's limit. A longer context might score lower, which would make bar Q harder. R\* is still sealed at N = 10, labelled "edge-limited", and the REPORT gives DEV-tune's gain from N = 9 to 10.
  - The sealed R\* stays bar Q's reference. A later R\* with a wider key is reported beside it, never in its place.
  - Changing bar Q's reference takes a decision card, and no such card may land once any model has been scored on DEV-test.
- **Short:** N\* ≤ 3, below the card's expectation. This is not an instrument failure by itself. B2 must pass at N\* and also at N = 7 before the sealed run, to rule out a scorer fault that grows with N.

## Base artifact

- **Code:** this branch's head (`main` at `9465b92d6`, plus the test settings commit and this sheet). The job writes the tarball's commit to `head.txt`.
- **Streams:** claude-420's `/workspace/out/c420/run1/` on wick: `train.tokens.u16`, `dev-tune.tokens.u16`, `dev-test.tokens.u16`, `dev-tune.novel.u8`, `dev-test.novel.u8`, `id-bytes.u8`. These are read only. Their sha256 values are claude-420's committed `run1/*.json`.
- **Machine:** wick, NVIDIA GeForce RTX 4090, in Claude's container (`local/claude/wick/wick.sh`), with a 16 GiB memory limit.

## Data split

- **Split v1, by prompt id** (claude-411; claude-417 kept it): TRAIN gives the counts, DEV-tune chooses N, DEV-test is sealed.
- **Novel subset** (claude-411's rule, claude-420's masks): a target is novel when the 13 tokens before it in its record form no 13-gram of TRAIN. Bar Q gates on all targets and on the novel subset.
- **DEV-test** is scored once, in step 6. No earlier step reads it:
  - the instrument refuses its sha256 without `--sealed-run`;
  - every run of the oracle test checks that its query is DEV-tune's (step 4b);
  - nothing seen on DEV-test feeds back into N.

## Minimum test power

- R\* is deterministic. Counts, masses and bits are exact integers, so it has no seeds and no sampling error, and step 5 checks the run-to-run claim.
- N is chosen on 10.8 M DEV-tune targets. A difference of one Q12 unit decides between two N, so the choice is exact. Whether it generalizes is the usual DEV-tune-to-DEV-test question. The REPORT gives DEV-tune's BPB at N\* beside DEV-test's.
- The oracle check at N\* covers about 21,000 DEV-tune targets at 2‰ and requires more than 1,000 distinct masses. Step 4a shows that every order up to N\* is used.

## Cost

Wick is the owner's GPU, so there is no rental (D86). Claude-421 timed about 10 s per order on all of TRAIN. The sweep counts 2 + 3 + … + 11 = 65 orders, about 11 minutes, plus host checks of about 5 s per run. The oracle checks take a few minutes each; the second DEV-tune run and the sealed run about 2 minutes each. In all, about an hour. No pod.

`wick.sh gpu` is read before each job. Another user's process may share the GPU: the result is exact, so sharing changes only the times. A job starts only when at least 12 GiB of device memory are free, and the log names every other process on the GPU.

## Stop rule

- **Stall rule:** a job is stopped by its own PID and reported only when all three hold for 20 minutes:
  - its log gains no new line;
  - it has no GPU use of its own;
  - its CPU time (utime + stime in `/proc/PID/stat`, over the job's processes) does not advance.

  The oracle phase runs on the CPU and prints only at its end, so the CPU-time condition keeps it alive while it works. A run that is healthy but slow is never cancelled for its time.
- **A sweep run that fails B1** ends the sweep. It is classified (instrument or memory) and repaired, and the sweep is rerun from N = 1 under this sheet. DEV-test is untouched.
  - The partition budget may be lowered after a memory failure, but only if the per-order digests of the N values already run stay the same.
- **The sealed run is run once.** Its first written JSON is the record. The failure cases:
  - **A crash** is an exit before the log's "gpu scored" line, for example out of memory while counting. No mass was computed and no score seen, so the run is repeated once. A second crash needs a new sheet.
  - **A `--check-host` mismatch** (host ≠ GPU, which exits before the JSON is written) is an instrument failure, not a crash. It is not repeated. It goes to a repair and a new sheet.
  - **A JSON that fails B1** stays on record. A repaired run under a new sheet may give R\* only through a decision card that amends the ORDER's seal. The first JSON stays on record beside the new one.
- **No number seen on DEV-test feeds back into the choice of N.**
