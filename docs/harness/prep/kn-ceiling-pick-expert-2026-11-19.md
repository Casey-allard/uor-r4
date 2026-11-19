# Prep sheet: the S5 Pick expert, fitted on reserved TRAIN, against its controls (written-state-count-model, claude-439, 19 Nov 2026)

Lab: claude. Item: [written-state-count-model](../../roadmap/items/written-state-count-model.md) (#4063). Order: [claude-439](../../loop/tasks/written-state-count-model/claude-439/ORDER.md), with the corrections in its [STEP0](../../loop/tasks/written-state-count-model/claude-439/STEP0.md). Code: branch `claude/pick-expert-439` at the head the FIT reviews, named in the FIT history below. A later commit may change only documents, or repair a step-1, step-2, B1 or B2 failure under the Stop rule. After any repair every step re-runs from step 1, each into a new folder, and no number from the earlier build is reported. Compute: wick, Casey's own RTX 4090, at no rental. No pod is leased.

## The question

Does a session-copy expert lower D87 DEV-tune bits beyond the count experts? It is read by the Pick XOR + popcount over geometric codes. And does it do so beyond the same expert with its geometry removed (C2, exact token pairs) or scrambled (C3, permuted codes)?

Every arm's weights are fitted on a reserved TRAIN slice R and scored, frozen, on DEV-tune. claude-433 and claude-438 fitted on the set they scored, so this is the first out-of-sample reading of the count mixer.

## Steps

Every step runs on wick, as jobs through `local/claude/wick/wick.sh`, built from a source tarball of the head above with `UOR_NVCC_ARCH=compute_89`. The streams are claude-420's split v1 copies on wick. Their sha256 values are checked against claude-420's `train.json` and `dev-tune.json` before any step:
- `train.tokens.u16` `c6cba29ead4c745fb47b3f87007d982e72defc9f04102a268709ba0d34d51460`;
- `dev-tune.tokens.u16` `75c6d1ac479bf120bac3aa60ac160536e8146f8052aa49c002209500799b7d0a`;
- `dev-tune.novel.u8` `7df42437260bb767e2b1bdfd0cc55a242bd6e0b632a5bcd754852a7ab8aca6af`;
- id bytes `befb22f6908751ae522a80336afa584eb2c24e36a10b90ad3105e39271a48ac9`.

The trunk is `seed1.d11` (see Base artifact). N = 10, budget 2^24, the KN start and 96 EM passes for every fit, as in claude-438. Every run writes to a new folder under wick's `uor-r4-out/c439/`; no folder is reused (L-002). A completion watcher is armed before each job starts.

Where each part runs:
- the counts, the lookups and the Pick read run on the 4090;
- the EM fits over the per-target masses and the bootstrap sums run on wick's host, as claude-433's and claude-438's fits did on their pod's host.

1. **Build and tests at the head, on wick:**
   - `cargo test --release --features cuda --lib -p uor-r4-discrete -- kn_lookup_cuda kn_score_cuda ngram_count_cuda kn_filter kn_ceiling pick`;
   - the instrument's own tests.
   - All must pass.
2. **The 1‰ slice with the host checks:**
   - the lookups' host check, as `--check-host` in claude-438;
   - the CUDA Pick against the CPU oracle `observe_identity_pick` at every DEV-tune target of the slice. Each target's trace words `[pick, d(pick), ties, gate + 1, out root]` per block, and the successor, must be equal word for word.
   - C3's device read must equal the host Pick over the permuted codes row for row: class, hit, pick and distance at every target of the slice. A1 is checked the same way on both spans.
   - C2 runs on the host only. It must equal an independent host read: the code Pick over one distinct code per token, whose distance is 0 exactly at an exact pair. C2 must equal that read's fired rows and be empty elsewhere.
   - The lookups are checked one record-aligned chunk at a time. Every check fails the run if it compares no target.
3. **1% and 10% of C** (`--per-mille 10`, `--per-mille 100`): A0, A1, C1, C2, C3 and P+, with `--profile-phases`.
4. **100% of C**, the same arms, with peak host memory sampled once a second and the GPU peak read from `nvidia-smi`.
5. **Storage.** Each run's JSON, log and memory samples are copied to the laptop under `local/claude/c439/`, and committed with the REPORT under `run2/` (run1 was stopped; see the read amendment below). No count table, stream, mask, id list or Pick tape is copied off wick or committed.

**The arms, in ORDER indexing** (the target is at t, the query pair is (t − 2, t − 1), a candidate pair is (p − 1, p), and p + 1 < t):

| Arm | Pick expert | Bucket class from |
|---|---|---|
| A0 pure count | absent | none: claude-438's 176 buckets |
| A1 Pick | `seed1.d11`'s block codes, Identity transport | A1's own read |
| C1 read disabled | absent | A1's read (528 buckets, no Pick column) |
| C2 exact pair | the latest p with the same two token ids | C2's own match (did not fire, distance 0) |
| C3 permuted codes | A1's read over a seeded permutation of the token rows | C3's own read |
| P+ | A1 and A0 on the repeated streams below | A1's read |

- **Window.** Candidates lie inside one D87 record (BOS … EOS) and inside the trunk's context of 192: the read's prefix is at most 192 tokens ending at t − 1, cut at the record's BOS.
- **Fires** means the target has at least three earlier tokens in its record, counting BOS. When it fires, the expert puts 2^48 on the token at p + 1. Where it does not fire, the expert is absent from the bucket's simplex. Distance ties go to the latest p.
- **C3's permutation:** a Fisher–Yates shuffle of token rows 2 to 4,095 of the code table, driven by splitmix64 with seed 439. BOS (0) and EOS (1) keep their rows, so the code multiset is unchanged.
- **P+'s streams:** every record body of R and of DEV-tune is cut into 64-token chunks, and each chunk is followed at once by a verbatim copy of itself, so each copy's source is 64 tokens back, inside the window. A0 and A1 are fitted on R⁺ and scored on DEV⁺. The scored set for B2 is the second copies' targets.

## Bar (pre-registered)

- **B1 (identity):** step 2 passes. The CUDA Pick, C2 and C3 equal their host versions at every target of the 1‰ slice. The lookups pass their host check.
- **B2 (P+, the instrument can show a copy gain):** at every size, A1's bits on DEV⁺'s second copies are at most half of A0's bits on the same targets. If B2 fails, the instrument is broken and no arm's result is read (L-007). A plus span with no second-copy target, or A0 bits of 0 there, fails the run: B2 never passes on nothing compared. A second copy's targets at offset 2 or more sit at distance 0; the first two straddle the copy boundary, so their query pair has no earlier exact match and is not a copy target (about 2 in 64 of B2's targets). So B2 tests the distance-0 path only; the read at distance above 0 rests on B1's equality with the host oracle.
- **B3 (EM):**
  - each fit's last `fit_bits` is at most its first;
  - each fit's scored bits on R equal its last `fit_bits`;
  - `gap_bound_q12` is written for every fit; it is recorded, not tested.
  - A rise between passes is recorded but does not stop the run.
- **The reading, at 100% only,** from paired bootstrap bands over DEV-tune's 2,670 records:
  - For two arms X and Y, d_r = bits_X(r) − bits_Y(r), in Q12, per record r. The run fails unless the bands resample exactly 2,670 records; the JSON writes the count as `band_records`.
  - 1,000 resamples of the records, with replacement, are driven by splitmix64 with seed 4390. Each resample gives the sum of its d_r.
  - The band is the 25th and the 975th of the 1,000 sorted sums.
  - "X beats Y" means the band's upper end is below 0.
  - The bands written are A1 − A0, A1 − C1, C1 − A0, A1 − C2 and A1 − C3, overall and over the novel class, each in Q12 sums and in micro-BPB.
  - The overall bands decide the reading. The novel bands are recorded beside it.

**Reading map** (fixed before the run). The rows are tried in this order and the first that holds is the reading. The code writes the label as `reading` in the run's JSON; B1, step 1 and the stream check are read beside it.
- **I (instrument):** B1, B2 or B3 fails, step 1 fails, or the stream check fails. The JSON writes `instrument` when B2 or a B3 fails. The repair is made under this sheet, and no number from that build is reported.
- **No gain** (`no_gain`): A1 does not beat A0. This configuration is closed; the mechanism is not.
- **Bucket split only** (`bucket_split_only`): A1 beats A0, A1 does not beat C1, and C1 beats A0. The gain comes from splitting buckets, not from copying.
- **Not separable from the bucket split** (`not_separable_from_the_bucket_split`): A1 beats A0, A1 does not beat C1, and C1 does not beat A0. No geometry claim is made.
- **Geometric gain** (`geometric_gain`): A1 beats A0, C1, C2 and C3. S5's Pick counts as a geometry gain on P1.
- **Copy gain, no geometry** (`copy_gain_no_geometry`): A1 beats A0 and C1 but not both C2 and C3. A session copy helps, and these codes add nothing that token identity or any code map does not. The deployable P1 path takes the cheaper of the arms that tie.

**Recorded, with no bar:**
- each arm's DEV-tune micro-BPB, overall and novel;
- its fit bits on R and its bound;
- A0 against R\* = 1,432,318 micro-BPB, labelled "C is TRAIN minus R, not claude-422's TRAIN";
- the fire rate and the share of fired targets at distance 0, for A1, C2 and C3;
- the fitted Pick weight per bucket class;
- the 1% and 10% bands;
- wall time, phases and memory at each size.

## Base artifact

- **Code:** `claude/pick-expert-439` at the head the FIT names. The job writes the tarball's commit to `head.txt`.
- **Trunk:** `seed1.d11`, sha256 `04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d`. The job reads the same bytes from the HF store `caseyallard/uor-r4-store`, file `results/antigravity/m1-close-saved-model-2026-10-26/outputs/build-a/t16-frozen-zero-seed1-trunk.d11`, copied to wick without any token. `load_trunk` refuses any file with another sha256.
  - It is the trunk the count work already binds (`BASE` in `count-replay.rs` and `count-score.rs`; `TRUNK` in codex's `pick-reachability.rs`).
  - Its tokenizer gives the same ids as D87's (STEP0 §1).
  - No saved D11 artifact is in Pick mode. A1 reads this trunk's token blocks with Identity transport, through the same device path as `pick-reachability.rs` (`identity_pick_inventory`, `ReadSpec::Pick`, position word 3).
- **Streams:** claude-420's split v1, the files above, checked by sha256.
- **Machine:** wick's RTX 4090 (24 GB), shared with a transcode and with Codex. claude-433's 100% run peaked at 16,028 MiB of device memory. If the 4090 has less than that free when a size starts, the job waits for it and does not run beside the other jobs.

## Data split

- **C** (counts): TRAIN's records whose index mod 128 is not 0, cut at 1%, 10% and 100% of the stream at a record start, as `record_prefix` does.
- **R** (fit): TRAIN's records whose index mod 128 is 0, at full size for every cut of C. That is 4,060 of 519,580 records and about 16.3M targets. R and C share no record, and no R record enters any count table.
- The split is fixed by record index before any score is read.
- **Scoring:** DEV-tune, 2,670 records, 10,807,417 targets, weights frozen.
- **P+:** R⁺ and DEV⁺ are built from R and DEV-tune only.
- **DEV-test is not read by any step.** It stays sealed for gate (b).

## Minimum test power

- The bits are exact integer sums. The only sampling is the bootstrap over records, which is the unit the bar reads.
- The 1% and 10% runs give each band's width before 100% is read. Nothing is changed after seeing them: N, R, the bucket rule, the passes, the start, the seeds and the chunk length are all fixed here.
- Every band is written in micro-BPB beside its Q12 sums.
- If the 100% A1 − A0 band holds 0, the REPORT gives its width in micro-BPB, as the smallest gain this run could have shown.
- The REPORT calls a reading small when the band that decides it clears 0 by less than the width of the same band at 10%.
- B2's margin is wide on purpose: in P+ every second copy's source is in the window, so a working read cuts those bits far below half.

## Cost

- **Rental:** none; wick is Casey's own GPU.
- **Time, estimated from claude-433's 5090 run** (289 s raw count; 137 s for 24 passes of four fits over 10.8M targets, so about 1.4 s per pass per fit):
  - the raw count at 100% is about 7 min on the 4090;
  - each fit on R is about 3.5 min, five fits about 17 min;
  - P+'s two fits on R⁺ (about 32.6M targets) are about 14 min;
  - the Pick reads are minutes. (Wrong at 256 windows per call: about 5 h per size at run1's rate, about 13 h at the probe's. See the read amendment; at 4096 per call, about 1 h per size.)
  - So about 45 min at 100%, and under two hours for every size together. (Corrected by the read amendment: the three reads cover the whole query stream at every size, so about 2 h per size and about 8 h for every size together.)
- **Host memory:** the P+ span (R⁺ and DEV⁺, about 54M targets) is the largest. Its lookups are about 7 GB, and its 14-expert table about 6 GB, so the peak is about 15 GB.
  - wick has 59 GB, about 26 GB of it free on 19 Nov beside the other jobs.
  - The arms are built one at a time, and the main span is finished and freed before the P+ span is looked up.
  - The lookups go to the GPU in record-aligned chunks of at most 2^24 targets.
  - Before the arms, the job frees TRAIN, the counter and the KN tables. claude-438's 100% run, which held only DEV-tune, peaked at 10.3 GB.
  - **Memory rule:**
    - Claude's wick container is capped at 16 GB.
    - Each run writes its peak RSS. Before the 100% run, that peak is extrapolated linearly from the 1% and 10% runs.
    - If the estimate is 14 GB or less, the 100% run goes under the 16 GB cap.
    - If it is above 14 GB, the cap is raised to the estimate plus 4 GB for the 100% run only, when wick shows that much memory available beside the other jobs. Otherwise the job waits for that memory, under the six-hour rule below. The cap goes back to 16 GB afterwards.
    - No parameter of the run changes for memory.

## Stop rule

- A failure in step 1 or 2 stops the job before step 3. It is repaired under this sheet.
- A crash at any size is repeated once, under a new name (`<name>-retry`) beside the crashed run's files, which are kept. A second crash at the same size needs a new sheet.
- A B1, B2 or B3 failure stops the reading. It is an instrument fault, and no band from that build is reported.
- Each size is run once. The first JSON written at a size is its record.
- No DEV-tune number feeds back into any parameter this sheet fixes.
- If wick is unreachable, or its GPU or the memory a size needs stays unavailable for more than six hours, the unit asks for a rented 4090 or 5090 in the caps (§3.6) under a new sheet line. It does not wait.

## Read amendment, 19 Nov (before any band was computed)

- **What happened.** Run1 at `51be0e0a8aad48b04ecc6fa63cb0b5160ae2e768` passed step 1 on wick: 60 discrete CUDA tests, the instrument suite and the trunk oracle. Step 2 (1‰ with `--check-host`) was stopped by its pid at 19:01Z, 2 h 11 min in, inside A1's read of the main span. It wrote no JSON; its claimed, empty `pick-p1.json` and its log stay in run1. No arm was fitted and no band was computed, so no DEV-tune number was seen.
- **Why.** The read sent 256 windows per device call. A probe on wick (`device_read_is_batch_invariant`, at `8bf1ac29ef1039ba1eeb612706ceaee0536c8173`) timed 10,165 windows of the trunk: 4,390 ms at 256 per call, 1,221 ms at 1,000, 323 ms at 4,096. The forward call costs about 100 ms whatever it holds. At 256 the three reads (107M windows per size) take about 5 h per size at run1's rate (about 6,000 windows a second, read from its memory growth) and about 13 h at the probe's (432 µs a window), at every size. At 4,096 the probe's rate (about 32 µs a window) gives about 57 min per size.
- **The change.** The read sends `READ_BATCH` = 4,096 windows per call, the device's maximum (`MAX_BATCH_ROWS`). Each window's Identity `record` is still its ordinal in the stream, so no input to the device changes. The probe asserts the same rows and kept words at 256, 1,000 and 4,096. `the_read_fills_each_device_call` pins the constant and fails at 256.
- **What does not change.** Every arm, control, band, bar, parameter, seed and stop rule above. B1 at step 2 still compares the device read with the host at every DEV-tune target.
- **The rerun.** Steps 1 and 2 rerun at the new head into `run2`, a new folder (L-002), then steps 3 and 4. Run1 is not a size's record: a deliberate stop before any fit is neither a crash nor a record, and its empty claim file is not a JSON written. So the crash rule (`<name>-retry`) and "the first JSON written at a size is its record" both apply from run2 on.
- **The use of the constant.** `the_read_fills_each_device_call` pins `READ_BATCH`, not the call in `rows`, which a host test cannot reach. So step 2's job also requires `a1_main_rows_per_s` of at least 12,000 in the JSON (256 per call gave about 2,300 to 6,000).

FIT history:
- Round 1, 19 Nov, a fresh-context non-author reviewer at `83fd3737e14f6ade8bdefa5446659812d010c968`: FIT WITH CHANGES. Made at the next head: the code writes the reading label; C1 − A0 joins the bands, with the "not separable" row; the overall bands decide; each band is also in micro-BPB; B2's and B3's scope notes; the memory rule's two branches; the trunk is read from the HF store.
- Round 2, 19 Nov, a fresh-context non-author reviewer at `42872c98df2276d8197e7131ee035e24212689f9`: FIT WITH CHANGES. Every round-1 change was present, and the sheet and code agreed. It asked for B2 to fail on an empty second-copy set and for the bands' record count to be checked. Both are in the next head, each with a test that fails without it. Round 2 did not read `kn_pick.rs` or `pick_device.rs`; round 3 reads them.
- Note from round 2: every size writes a `reading` label, but the reading is taken at 100% only. The REPORT never quotes a 1% or 10% label as the reading.

- Round 3, 19 Nov, a fresh-context non-author reviewer at `2d5ac69b01fc1e3c22a53435af7c2616f0958625`, reading the two guards, `kn_pick.rs` and `pick_device.rs`: FIT WITH CHANGES. Its one change was to reword B2's scope note, because the first two targets of each second copy are not distance-0 copies. It said the reworded note turns the verdict to FIT, and the note is reworded at the next head, a document-only change. Its open minor: no test pins the `check_band_records` call site. So step 2's job checks that `band_records` is 2,670 in the JSON.

- Round 4, 19 Nov, a fresh-context non-author reviewer at `da9f4670742d4140feb66edcbe408cfd39e83743`, for the read amendment only: FIT WITH CHANGES. It found the device sees the same input for a window at every batch size (only the token ids are uploaded; padding and row plans are per row), and 4,096 inside every device limit (about 150 MB a call). Its changes, all made at the next head: a `.0` line start in `rows` is rewritten; the amendment's time figures are corrected; the stop-rule wording for run1 is added. Its minor: the test pins the constant, not its use in `rows`, so step 2's job checks the read rate.

TEST FITNESS: FIT - round 4's changes are made: the `.0` line start in `rows`, the corrected times, the run1 stop-rule wording, and the read-rate check in step 2's job.
