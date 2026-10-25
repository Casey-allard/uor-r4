# Full-vector address metric on the A8 placement: zero-update GPU pre-flight

- **Date:** 2026-10-25. **Lab:** claude (claude-110, #3230). **Item:** `m1-address-key-full-vector` (#2834), M1 Stage 1.
- **Supersedes** antigravity-37's draft, `m1-address-key-full-vector-DRAFT-2026-10-23.md` (`b8aa3aefd`). It keeps that draft's design, and adds:
  - the kernel;
  - the pinned tables;
  - the configs;
  - the CUDA control;
  - the grader;
  - a matched shuffled control for each metric.

  Registration follows D67: the lab's own fresh pre-run review replaces the draft's retired owner/PM registration.
- **Rules:**
  - zero training updates;
  - integer arithmetic only, with no softmax or squashing (§3.2) and no matmul (§3.3);
  - GPU for every model quantity (§3.5);
  - host counting of stored outputs for grading only (D24);
  - a negative closes a configuration (D22).

## Question

The writer's two-block outer key has 5 distance levels. Suppose it is replaced by a full-vector cell metric, on a full-vector placement of the same 4,096 tokens into the same 256 cells. Does starting reach at checkpoint 0 then rise above A8's 1,351 of 3,847 P_succ rows, by more than the same metric gains on a shuffled placement?

## Arms (checkpoint 0, zero updates, seed 1, `Relation { kept: 8, sweeps: 1 }`, `ActiveLeafV1`, interleaved schedule)

| Arm | Config (sha256) | Pinned table (file sha256) | Metric (`s[15]`) | K | Role |
|---|---|---|---|---:|---|
| A8-Outer | `docs/labs/a8-training-configs-2026-10-19/evidence/a8-relation-seed1.json` (`469a2ff0fe17a6a80f5875a077b3e6a9436553efdf1c6e7884541220553e9d5e`) | `address_structured_v1` (`6600bdb1306dbaea4d77f5bf021e717a6bef791e44c1ec7d7c9bb9f0e8cd3cd1`) | `outer_l1_v1` (0) | 8 | exactness control |
| FV8-Env | `docs/labs/m1-address-key-full-vector/configs/fv8-env.json` (`56c3fafe0b8cdb7800f57d0a6ba577b2964356de5765b350a701546cace6e534`) | `address_full_vector_v1` (`2c886736c858778677d6a7b87253af188d8787cedc5fe5acd9b3c899a10a90e2`) | `cell_envelope_l1_v1` (1) | 8 | treatment 1 |
| FV8-TwoScale | `…/configs/fv8-two-scale.json` (`1cdb80615d1467bbe062bc1a70804630b3f69af5c580ce3b0d2b5d7407a89a28`) | `2c886736…` | `full_vector_cell_l1_v1` (2) | 8 | treatment 2, the prior primary |
| FVS8-Env | `…/configs/fvs8-env.json` (`b3b5fb60a904eaecb17d36f3ff5263ff4f83d6846612380df5ce8a5aef20830e`) | `address_full_vector_shuffled_v1_seed_20261019` (`5e5abf336bfb039d6c3bde336cf5d03f207ae9ff7bc8cfb21e71691c622313c4`) | (1) | 8 | treatment 1's shuffled control |
| FVS8-TwoScale | `…/configs/fvs8-two-scale.json` (`05d907d0d6e2d3526642b32ead7edf7a9db422c02334ff3ed9aa03939950c834`) | `5e5abf33…` | (2) | 8 | treatment 2's shuffled control |
| FV16-TwoScale | `…/configs/fv16-two-scale.json` (`1590e52d9d663ef27725bdeec55be7e3968c65060becd5bd02191e24fd2220a5`) | `2c886736…` | (2) | 16 | bracket only, against A16's 6,023 / 14,287 and 2,178 / 3,847; decides nothing |

The two tests that pin the arms:
- `tests/address_metric_configs.rs`: each config equals A8's except for its table, its metric and (FV16) K, and every K8 arm has its own scope.
- `attention_preflight` `the_dev_b3_lock_admits_every_k8_address_metric_arm`: `attention-dev-b3` admits every K8 config.

## Bar

1. **G-Exact (instrument):** A8-Outer reproduces, exactly:
   - 4,244 of 14,287 full-DEV target-admitted rows;
   - 1,351 of 3,847 P_succ admitted;
   - full-DEV `top1_correct` 133.
2. **G-Q (instrument):** `device::address_metric_device_tests::device_admission_matches_the_host_oracles_for_each_metric` passes on the pod.
   - The device's admitted cells on 16 rows equal the `address_cells.rs` oracles, recomputed on the device's own table and query vectors, for all three metrics.
   - The table is a patterned 1,024-token one, with 4 tokens of distinct interior blocks in every cell. It replaced the seeded table after run 1; see below.
   - G-Q also runs each metric through the device's evaluation path (row metrics and aggregation), which it added after run 2.
   - The three metrics must also admit different cells, so no metric is computed as another.
3. **G-Occ (instrument):** each pinned table's `coverage-census --token-cells … --bar-rows 1941` reports:
   - 4,096 distinct vectors and 256 occupied cells;
   - zero tokens in alias groups, and zero DEV rows targeting a non-representative;
   - alias ceiling 14,287.

   That covers the FV table and the shuffled table. The tables are pinned and built on the host, so a miss is a table-build fault, never a mechanism result.
4. **Starting reach (primary).** Treatment T (FV8-Env or FV8-TwoScale) and its same-metric shuffled control S:
   - **reach:** B3(P_succ, T) > 1,351, and on the 3,847 P_succ rows, b − c > F = max(24, 2·√(b + c)) against A8-Outer. b counts rows T admits and A8 does not; c the reverse.
   - **geometry:** the same rule holds for T against S.
   - **capacity:** min(alias ceiling, T's full-DEV admitted) > 1,941. With a ceiling of 14,287, this reduces to T's own admission. A8 has 4,244, so this is near-vacuous: it is recorded, not evidence.

   Each treatment needs reach and geometry, against two controls. That holds the family-wise false-pass rate near 4.6% (two roughly two-sigma one-sided tests), and the label names which treatment passed.

| Outcome | Condition | Unlocks |
|---|---|---|
| INSTRUMENT | G-Exact, G-Q or G-Occ fails | no treatment number is read; fix and rerun |
| PASS | a treatment has reach, geometry and capacity | Stage 1 is met: #2834 is done; `m1-wider-code-carrier` (#2847) and Stage T's decision review (#2836) take it as input |
| PARTIAL | reach and capacity, no geometry | the gain is packing balance, not predecessor geometry; #2834 stays open, and a decision review picks the next metric |
| FAIL | no treatment has reach | closes (256-cell outer partition, B = 4 full-vector metric, K = 8) under D22; routes to #2847 |

**The grader** is `address-key-grade MANIFEST OUT` (`src/address_key_grade.rs`). It:
- joins rows by `row_ordinal`, never by file order;
- refuses a repeated or missing row, or a panel other than 14,287 / 3,847;
- grades each treatment against A8-Outer and its same-metric shuffled arm;
- decides F exactly in integers: d > 24 and d² > 4(b + c).

Its host tests fix b and c on known fixtures. The table's outcome order is INSTRUMENT, then PASS by either treatment, then PARTIAL, then FAIL.

## Base artifact

None is trained. Each arm is `init-corpus` at checkpoint 0 from its pinned table. The tables were built on the host (D24 data preparation) from TRAIN by `address-cells-codebook`, which rebuilds A8's `6600bdb1…` byte for byte. They are sealed in the HF store at `results/claude/address-key-2026-10-25/tables/`, with a SHA256SUMS listing all three file shas above.

## Data split

- **TRAIN** `mw-balp/train`: tokens `3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8`, mask `b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3`. Used for table construction only.
- **DEV** `ft/dev`: tokens `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b`, mask `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078`, tokenizer `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`. That is 14,287 rows, of which 3,847 are P_succ. Each arm scores it once, on the GPU.
- No FRESH slice is read.

## Steps

1. **Done:**
   - the host metric (antigravity-37);
   - the kernel dispatch, the zero-update gate (one guarded capability constructor), the CUDA control, the configs and the grader (claude-110).
2. **Source commit:** the run builds `77123dd995be44d5b0a5f482c8b82e270dd9aa42`, which fixes all code. A later commit that touches `crates/` re-pins this line before the run.
3. This sheet's non-author FIT, then the D67 pre-run review.
4. **On one leased RTX 5090, in this order** (the job is `local/claude/address-key-job.sh`; it stops at the first miss, so no treatment number exists unless every instrument check passed):
   1. Check that free disk is at least 30 GiB. Stage the data and tables, and check every sha above.
   2. **G-Q:** `cargo test --release --locked -p uor-r4-discrete --features cuda --lib address_metric_device_tests -- --ignored`.
   3. `init-corpus` for every arm.
   4. **G-Exact:** run `eval-corpus` and `attention-dev-b3` on A8-Outer, and check 4,244 / 1,351 / 133 on the pod.
   5. **G-Occ:** for the FV and FVS tables, run `attention-export-token-cells`, then `coverage-census --token-cells … --bar-rows 1941`, and check G-Occ on the pod.
   6. Only then the treatments: `eval-corpus` for every other arm, and `attention-dev-b3` for the K8 arms.
5. Archive everything, with SHA256SUMS, to `results/claude/address-key-2026-10-25/run/` before `uor-pod down`.
6. Grade on the Mac with `address-key-grade manifest.json grade.json`, run from inside the downloaded run folder (the manifest's paths are relative to it), then record one notebook row.

**Changed after the FIT.**

By its reviews, before any run:
- the source commit was re-pinned for the grader's review fixes and rustfmt;
- the grader now binds each arm to its pinned config and requires G-Q's log;
- G-Q's fixture grew to 1,024 tokens;
- the job (`docs/labs/m1-address-key-full-vector/address-key-job.sh`) copies itself into the run folder;
- the cost is re-priced.

By its two instrument-fault runs (below):
- after run 1, G-Q's fixture became the patterned table, because the 1,024-token seeded table still could not separate the envelope from two-scale;
- after run 2, the two-scale hinge keeps the envelope's scale, and G-Q runs the evaluation path.

None of these changes an arm, a bar or a pin of data.

**G-Q's first pod run (25 Oct, 17:27Z, `vast-57698566`) stopped as an instrument fault, as the stop rule requires; no arm ran.**
- The device matched the host oracle for all three metrics on every row.
- The separation check then found envelope and two-scale admitting identical cells: the seeded device table gives same-cell tokens one vector, so the two orders cannot differ.
- The fixture is now a patterned table, with 4 tokens of distinct interior blocks in every cell, and the source is re-pinned. The output is kept as `address-key-2026-10-25-fault1`.

**The second run (17:34Z) passed G-Q, G-Exact (4,244 / 1,351 / 133) and G-Occ on both tables, then stopped as an instrument fault in FV8-TwoScale's `eval-corpus`; no treatment number is graded.**
- The evaluation aggregator refused it with status 12. Its row contract bounds the address hinge (at most 72) on the outer key's scale, and two-scale distances run about 256× larger.
- The two-scale metric now keeps the envelope's scale for its hinge, since its sum term only breaks ties; admission and rank still use the two-scale order.
- G-Q now also runs the evaluation path for every metric.
- No arm, bar or data pin changes. The output is kept as `address-key-2026-10-25-fault2`.

## Stop rule

- **G-Exact, G-Q or G-Occ fails:** stop as an instrument fault (D31), never a mechanism negative.
- **Projection:** stop if `eval-corpus` on any arm projects past 10 minutes.
- **Hard ceiling:** 45 minutes wall clock on the pod. Free disk under 30 GiB stops the job before it stages anything.

## Cost

One RTX 5090, under 45 minutes: about 15 minutes to build and qualify, then a few minutes for each of 6 arms. At the cheapest verified 5090, $0.989/h (D67 review, 25 Oct), that is **about $0.75**, with a cap of $1.

## Minimum test power

Every arm is scored on all 14,287 DEV rows and all 3,847 P_succ rows (at least 200 per CHARTER §4.3). The paired floor, F = max(24, 2·√(b + c)), is a two-sigma one-sided discordance test, with a floor of 24 rows (0.6% of P_succ). The floor dominates until b + c > 144.

## Test fitness

TEST FITNESS: FIT - M1 #2029 m1-address-key-full-vector
sheet: docs/harness/prep/m1-address-key-full-vector-2026-10-25.md
reviewer: claude, fresh-context non-author review (round 1 NOT FIT at 1cb179f92; round 2 FIT)
note: https://github.com/UOR-Foundation/uor-r4/issues/2834#issuecomment-6400663507
date: 2026-10-25
A FIT verdict clears compute for the configuration in that sheet only (D22).
