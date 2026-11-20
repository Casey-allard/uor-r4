# Prep sheet: split the S5 Pick buckets by pair distance (written-state-count-model, claude-441, 20 Nov 2026)

Lab: claude. Item: [written-state-count-model](../../roadmap/items/written-state-count-model.md) (#4063). Order: [claude-441](../../loop/tasks/written-state-count-model/claude-441/ORDER.md). Code: branch `claude/pick-distance-441` at the head the FIT reviews, named in the FIT history below. A later commit may change only documents, or repair a step-1, B1, B2, B3 or replication failure under the Stop rule. After any repair the run is made again into a new folder, and no number from the earlier build is reported. Compute: wick, Casey's own RTX 4090, at no rental. No pod is leased.

## The question

claude-439 put every fired Pick read at pair distance 1 or more into one class of buckets. EM gave Pick about 10‰ of the mixture there, against 190‰ at distance 0. Does a split by distance, into 0, 1, 2 and 3 or more, lower D87 DEV-tune bits beyond claude-439's three-class A1? And does the D11 codes' edge over a permuted code map (C3) change under the split?

## Steps

Every step runs on wick inside the `uor-r4-claude` container, as one job through `local/claude/wick/wick.sh`. It is built from a source tarball of the head above, with `UOR_NVCC_ARCH=compute_89` and `cargo +1.97.1`, as claude-439's run2 was.

The inputs are checked by sha256 before any step:
- claude-420's split v1 streams, with claude-439's four values (`train.tokens.u16`, `dev-tune.tokens.u16`, `dev-tune.novel.u8`, the id bytes);
- the trunk `seed1.d11`, `04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d`;
- claude-439's R/C split at `/workspace/out/c439/split-v1`, against claude-439's `run2/split.sha256`. The split is read, never rebuilt.

The run writes to a new folder, `/workspace/out/c441/run1`. No folder is reused (L-002). A completion watcher is armed before the job starts.

1. **Build and tests at the head, on wick:**
   - `cargo test --release -p uor-r4-discrete --features cuda --lib -- kn_lookup_cuda kn_score_cuda ngram_count_cuda kn_filter kn_ceiling pick`;
   - the instrument's tests with `--features cuda --bins`, and the ignored trunk-oracle test `host_pick_equals_the_trunk_oracle`.
   - All must pass, with at least one test run in each log.
2. **One run at 100% of C:** `--per-mille 1000 --check-host --profile-phases --pick-arms`, with the same positional arguments as claude-439's `pick-p1000` (N = 10, budget 2^24). Peak host memory is sampled once a second, and the GPU peak is read from `nvidia-smi`.
   - The run fits A0, A1, C1, C2 and C3 exactly as claude-439 did, and then A1s and C3s under the split rule.
   - `--check-host` runs B1 again at this head: the lookups against the host, A1 and C3's device rows against the host rows, C2 against the identity-code read, and A1 against the trunk oracle at every DEV-tune target.
3. **The replication check**, by `jq` in the job against claude-439's `/workspace/out/c439/run2/pick-p1000.json`. It compares these keys only, in this order:
   - for each of A0, A1, C1, C2 and C3: `.pick_arms.main.<arm>.b3` and `.pick_arms.main.<arm>.scored` must be equal;
   - the five bands `a1_minus_a0`, `a1_minus_c1`, `a1_minus_c2`, `a1_minus_c3` and `c1_minus_a0` must be equal;
   - `.pick_arms.b2` must be equal;
   - then A1s and C3s's `b3` are read: `last_at_most_first` and `scored_equals_last` must both be true. If either arm fails, the job stops with "B3 failure on a split arm": that is a B3 failure under the Bar, not a replication failure, because a split arm's B3 also sets `reading` to `instrument`;
   - only then `.pick_arms.reading` (`geometric_gain`) must be equal.
   - The job writes `replication: PASS` or `replication: FAIL` with the first differing key.
   - **Not compared, and expected to differ:** the whole arm objects. They now carry `weights_q32` (added on main after claude-439's run2) and `pick_stats.by_class_targets_hits` (added here). The check covers the commits on main since claude-439's run head that touched the fit code: `3f0aaa294` (#4107, claude-439's merge), `67d54aa0a` (#4123, codex) and `cfe019895` (#4124, codex), as well as this unit's own diff.
4. **Storage.** The JSON, the log, the memory samples and the step logs are copied to the laptop under `local/claude/c441/` and committed with the REPORT under `claude-441/run1/`. No count table, stream, mask, id list or Pick tape is copied off wick or committed.

**The split rule:** a row that did not fire is class 0. A row that fired takes class 1 + min(distance, 3), where the distance is the XOR + popcount distance the read returns. So distance 0 is class 1, as in claude-439, and distance 1, 2 and 3 or more are classes 2, 3 and 4. Each class holds one copy of the 176 base buckets: 880 buckets in all. A row whose split class, cut to 2, differs from its three-class class fails the run. That check runs on the host after the reads and B1, before any fit, so a failure there is an instrument stop under the Stop rule, not a crash to retry.

## Bar (pre-registered)

- **B1, B2 and B3** are claude-439's, unchanged. B3 now also covers A1s and C3s.
- **Replication (instrument):** step 3 passes. If it fails, the split is not read. The difference is the finding (L-007), and it is classified as instrument or object before anything else is said.
- **Bands,** with claude-439's method (2,670 DEV-tune records, 1,000 resamples, splitmix64 seed 4390, the 25th and 975th sums; "X beats Y" when the upper end is below 0). The four new ones are A1s − A1, A1s − C2, A1s − C3s and C3s − C3, overall and over the novel class, in Q12 sums and in micro-BPB.
- **The split reading,** from the overall bands only, written by the code as `split_reading`:
  - **`no_split_gain`:** A1s does not beat A1. The split closes as a configuration; the Pick expert stands as claude-440 chose it.
  - **`split_gain_geometric`:** A1s beats A1, and A1s beats C3s. The split helps, and the D11 codes still beat a permuted map under it.
  - **`split_gain_no_geometry`:** A1s beats A1, and A1s does not beat C3s. The split helps, and the codes' edge is not shown under it.
  - **`instrument`:** B2 or a B3 fails. A replication failure is read the same way, by the job and the REPORT.
- **What a gain does:** the REPORT recommends the split for S5, and a decision card under §5.1 makes the change. This run alone does not change S5.

**Recorded, with no bar:**
- A1s and C3s's DEV-tune micro-BPB, overall and novel, their fit bits on R and their bounds;
- the A1s − C2 and C3s − C3 bands;
- the fitted Pick weight per bucket, by class, for A1s and C3s;
- per class, the DEV-tune targets and the reads that hit (`by_class_targets_hits`) for every Pick arm;
- wall time, phases and memory.

## Base artifact

- **Code:** `claude/pick-distance-441` at the head the FIT names. The job writes the tarball's commit to `head.txt`.
- **Trunk:** `seed1.d11` as above, the copy claude-439 already placed on wick at `/workspace/in/c439/`, checked by sha256. `load_trunk` refuses any other file.
- **Reads:** the device Pick read is not changed at this head (`pick_device.rs` and `discrete.cu` are not in the diff), so A1 and C3's rows are the ones claude-439 measured. The split is made on the host from each row's distance.
- **Incumbent record:** claude-439's `run2/pick-p1000.json` on wick, also committed in claude-439's `run2/`.
- **Machine:** wick's RTX 4090 (24 GB), shared. claude-439's 100% run peaked at 24,071 MiB of whole-device memory, other jobs included. The job waits for 16,384 MiB free before it starts, as claude-439's did.
  - **Run note, 20 Nov:** `pick-p1000` crashed after 480 s with CUDA out of memory, at order 7 of the count build, before any Pick fit. Another lab's GPU job grew from 2,186 MiB at the start to 15,392 MiB, and the device peaked at 24,081 MiB. Under the Stop rule the run is repeated once as `pick-p1000-retry`, beside the crashed files, from the same binary. Only the start bar changes, to 21,504 MiB free: claude-439's 100% run used 20,189 MiB beyond the other jobs. No parameter of the run changes.

## Data split

- The same as claude-439: C is TRAIN's records whose index mod 128 is not 0, at 100%. R is TRAIN's records whose index mod 128 is 0 (4,060 records). Weights are fitted on R and scored, frozen, on DEV-tune (2,670 records, 10,807,417 targets).
- The split is claude-439's files, checked by sha256, not rebuilt.
- **DEV-test is not read by any step.** It stays sealed for gate (b).

## Minimum test power

- The bits are exact integer sums. The only sampling is the bootstrap over records.
- A1s and A1 use the same reads and differ only in their buckets, so their per-record differences are small and paired. claude-439's A1 − C3 band at 100%, from two arms that also share the read path, was 208 micro-BPB wide ([−232, −24]). The A1s − A1 band is expected to be about as wide [est]. A split effect much smaller than that would not be shown.
- If A1s − A1 holds 0, the REPORT gives its width in micro-BPB, as the smallest effect this run could have shown.
- One size is run. claude-439 already measured how the bands move with the size of C. Nothing is changed after seeing a number: the cut at 3, N, R, the passes, the start and the seeds are all fixed here.

## Cost

- **Rental:** none; wick is Casey's own GPU.
- **Time:** about 1 h 15 min [est], from claude-439's run2:
  - step 1 builds and tests, about 15 min;
  - claude-439's 100% run took 3,585 s;
  - the two split fits add about 275 s [est] (claude-439's five main fits took 688 s);
  - B1 adds about 6 min (the host lookups and checks took about 300 s at 1‰, and they do not grow with C).
- **Host memory:** claude-439's 100% run peaked at 20,076,956 KiB (VmHWM). The host lookups of `--check-host` hold one record-aligned chunk of at most 2^24 targets twice, about 2 GB more [est]. The split arms are fitted one at a time, like the others, and each table is freed before the next.
  - **Memory rule:** claude-439's rule was the estimate plus 4 GB. Here that is about 22 GB plus 4, so the container's cap is raised from 16 GiB to 26 GiB for this run when wick's MemAvailable is at least 28 GB beside the other jobs. Otherwise the job waits for that memory, under the six-hour rule below. The cap goes back to 16 GiB after the run. No parameter of the run changes for memory.

## Stop rule

- A failure in step 1 stops the job before step 2. It is repaired under this sheet.
- A crash is repeated once, under a new name (`pick-p1000-retry`) beside the crashed run's files, which are kept. A second crash needs a new sheet.
- A B1, B2 or B3 failure, or a replication failure, stops the reading. No split band from that build is reported. A split-class refusal is an instrument stop of the same kind.
- The run is made once. The first JSON written is its record.
- No DEV-tune number feeds back into any parameter this sheet fixes.
- If wick is unreachable, or its GPU or the memory stays unavailable for more than six hours, the unit asks for a rented 4090 or 5090 in the caps (§3.6) under a new sheet line. It does not wait.

FIT history:

- **Round 1, at `57bfb7035`:** a fresh-context non-author reviewer (claude subagent, no author context) gave FIT WITH CHANGES: no blocker, no major, three minor findings.
  1. A split-arm B3 failure sets `reading` to `instrument`, so step 3 would report it as a replication failure. Made: step 3 compares each original arm's `b3`, then reads A1s and C3s's `b3` before `reading`, and classes a failure there as B3.
  2. The sheet must name the keys step 3 compares, say the arm objects differ (`weights_q32`, `by_class_targets_hits`), and name the commits since claude-439's run head that touched the fit code. Made, in step 3.
  3. The split-class refusal must be stated as running before any fit, as an instrument stop. Made, under the split rule and in the Stop rule.
  - The author also corrected the memory cap to claude-439's estimate-plus-4 rule (26 GiB, at MemAvailable 28 GB or more).

TEST FITNESS: FIT - round 1's three changes are made: the B3 check before the reading comparison, the named keys and commits, and the split-class refusal as an instrument stop.
