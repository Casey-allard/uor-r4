# Prep sheet: m1-d11-artifact-scorer GPU qualification (claude-116, 25 Oct)

Item [m1-d11-artifact-scorer](../../roadmap/items/m1-d11-artifact-scorer.md) (#3248); ORDER [`claude-116`](../../loop/tasks/m1-d11-artifact-scorer/claude-116/ORDER.md), done-when stage 4. This run qualifies an **instrument**. It is not step 6's measurement: the A8 checkpoint stands in for step 6's own checkpoint (#2839) and its numbers are never reported as step 6's.

## Question

On one RTX 5090, does the D11 artifact scorer's ahead-of-time CUDA build (`docs/labs/m1-d11-artifact-scorer/instrument`, `--features cuda`) score every row of the registered 14,287-row DEV panel bit-identically to `IntegerDiscreteModel::evaluate_prefix`, on a real exported checkpoint?

## The one arm: base artifact and data split

- **Instrument:** the PR head pushed before the pod, built on the pod with nvcc (`UOR_NVCC_ARCH=compute_120`, cubin `sm_120`, PTX admitted by the discrete crate's integer rules).
- **Checkpoint:** A8 FrozenZero `a8-dose/run-state-frozen-zero/continuation-05248.json`, inside the store's `results/antigravity/a8-full-dose-rerun-2026-10-25/a8-full-dose-rerun-20261025.tar.gz`. Relation read (kept 8, 1 sweep), vocabulary 4,096, 4 blocks, context 192, 8 kept cells, epoch 274 = 274 accepted updates.
- **Rows:** the DEV split of `docs/labs/a8-training-configs-2026-10-19/evidence/a8-frozen-zero-seed1.json`, built by `rows`/`score` from `uor_r4_discrete::corpus` (the `eval-corpus` walk), with only the two DEV file paths rewritten to the pod's copies; the pins are unchanged.

## Steps (the probe ladder)

The pod: `scripts/pod/uor-pod up --lab claude --session claude-116 --gpu 5090 --count 1 --hours 2 --ref <full pushed sha> --card <#3248>`. A single job script runs every step and stops at the first failed check. Its literal commands are below. It is uploaded with the results.

0. **Inputs, pinned.**
   - `/root/uor-store get` the tarball, then check its sha256.
   - Untar only `a8-dose/run-state-frozen-zero/continuation-05248.json`, then check its sha256.
   - `/root/uor-store get` the DEV folder. Copy its two files to the config's own paths, `/workspace/uor-r4/local/driver/data/ft/dev/{tokens.u16,response_mask.u8}`, so the config is used unmodified, and check both sha256s and the config's.
   - Build in `docs/labs/m1-d11-artifact-scorer/instrument` with `UOR_NVCC_ARCH=compute_120 CARGO_TARGET_DIR=/root/build/target-d11 cargo build --release --locked --features cuda --target x86_64-unknown-linux-gnu`. Copy the binary to `/workspace/d11-bin/` with its sha256.
1. **(a) Finite GPU test**, minutes: `UOR_DEV_CONFIG=<the config> cargo test --release --locked --features cuda --target x86_64-unknown-linux-gnu --no-fail-fast -- --include-ignored`. Without `UOR_DEV_CONFIG`, the panel test fails rather than skips. The log must show both of these as `ok`:
   - `gpu_scores_are_bit_identical_to_the_reference_and_the_host_build`: Route, Flock and Relation, the 16-block 243-context bound, and 4,100 rows across a launch-chunk boundary.
   - `the_registered_dev_panel_has_14287_rows`.
2. **(b) Throughput**, seconds: `score` times its own GPU pass (`score_elapsed_us`). The laptop's `evaluate_prefix` takes about 64 µs per 192-token row, so the reference check over 14,287 rows projects to about 1 s. Nothing here approaches an hour.
3. **(c) The qualification**, from the copied binary:
   - `d11-scorer export continuation-05248.json A8.lut EXPORT.json`, then check `A8.lut`'s sha256.
   - `d11-scorer rows CONFIG ROWS.json`, then check its `rows_sha256`.
   - `d11-scorer score A8.lut CONFIG SCORE.json SCORE-ROWS.jsonl --reference`, then check the pass row's fields in `SCORE.json`.

## Bar and decision table

| Outcome | Meaning | Unlocks |
|---|---|---|
| (a) passes, and `SCORE.json` shows `rows` = 14,287, `rows_sha256` `d933c533…`, `artifact_sha256` `42671d44…` (the sha256 of the file it scored), `refused` = 0, and `reference.checked` = `reference.agreed` = 14,287 with no `first_mismatch` | the instrument is qualified | stage 4 of the done-when; the #2839 note maps sheet lines `:32`, `:81` and `:82` to `export`, `score` and `toks`; the step-6 unit scores its own checkpoint with them |
| any row differs, in (a) or (c) | an instrument fault (D31) | fix the port and re-run; parity is never relaxed |
| `agreed` = `checked` but `refused` > 0 | an instrument or data fault: the scored rows are not the panel's (both engines refused them) | never counted; find why rows were refused, re-run |
| a pin, the rows hash or the artifact hash differs | wrong inputs | never counted; fix the inputs, re-run |
| cargo, the link, nvcc, PTX admission or the cubin target fails | an instrument or tooling fault | fix the build or the kernel source, re-run |
| the pod, the driver or the store fails | infrastructure | a new pod; nothing is counted |

## Instrument checks (each must hold before (c) counts)

- `export` on the pod writes the artifact the laptop wrote: sha256 `42671d4422397217eb58b7c23763618a7eb007965476775c68054d3b3b09082f`, 542,260 bytes.
- `rows` on the pod gives 14,287 rows with `rows_sha256` `d933c533b8b95b4ed6cb54cf96fe83c25a221e3e3226f9a212f52c7bfc1a7074`, as on the laptop.
- The device's capability matches the cubin's `sm_120` (the scorer refuses otherwise, with no PTX fallback); the report records the cubin and PTX sha256 and the ptxas resource report.
- The reference is the product's own serving engine, `IntegerDiscreteModel`, the clean-room integer evaluator: an oracle for parity, never a score of its own.

## Data and checkpoints (sha256)

| Input | sha256 |
|---|---|
| A8 tarball (store) | `4a744188f14c934f89e57a1be83ecc8f2cb211620290a6539217837f809bae55` |
| `continuation-05248.json` | `70935d37234a868ceeeff66a132fb4584e27a67710f06ec7b87db39358fbc36b` |
| its `model_sha256` | `0a26abb5c4ddce94f9782f5c8f3e5d602bf297eb2c6fcd0c1c2f839e664df9c8` |
| config `a8-frozen-zero-seed1.json` | `cf7ebcc85d2e463ae8f6691b63d90a0e005701cb38b8249a8d0d5460d8508479` |
| DEV `tokens.u16` (store `w2-inputs/2026-10-25/driver-data/ft/dev/`) | `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b` |
| DEV `response_mask.u8` (same folder) | `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078` |

## Cost and stop rule

One RTX 5090 through `scripts/pod/uor-pod` (Vast, about $1.11/h at the time of writing), a 2-hour lease, at most about $2.50. One other session's pod ran at $2.17/h when this was written, inside the caps of 4 pods and $8/h. Stop at once and classify on the first parity mismatch. The outputs go off the pod to the store's `results/claude/d11-artifact-scorer-2026-10-25/` with a `SHA256SUMS` before `release` and `down`, and the pod never stays up after the job.

## Minimum test power

The comparison is exhaustive, not sampled. Every one of the 14,287 rows is checked field by field: the read vector, the admitted cells, the top-8 and the prediction. So any port fault that changes the output of even one panel row fails the run with certainty.

A fault that changes no panel row is covered only by the finite tests. Those tests ran on the GPU in step (a). On the laptop, each of nine planted bugs made them fail.

## Test fitness

TEST FITNESS: FIT. A fresh-context, non-author reviewer passed this sheet at `d2089be9c47b71a7669f2b67362e2fc30a8d0433` (#3248 comment `6404635561`), with four should-fix findings, all fixed in `932779e32d1fedee85a6343ca06cf64bc0d2b136`. The D67 pre-run review returned GO (#3248 comment `6404635800`). The run used the sheet as it stood at `932779e32`.

After the run, this section and the one above were added, and two headings were renamed to the blocks `prep.sheet_shape` requires. Neither change touches the test.
