# REPORT: cpu-training-pilot, deepseek-246

Lab: deepseek. Order: [ORDER.md](ORDER.md).

## What did not work, first

- No Claude-330 checkpoint is available in this branch, so fitting a real
  B2-trained ask table, measuring held-out pulls or making any carry/capability
  claim is `NOT_RUN`.
- The synthetic B2-style fixture reaches a zero objective because it contains
  the exact target-key offset as a candidate. That 8-to-0 bit change verifies
  the objective, candidate construction, sidecar and audit path; it is not
  evidence of learned generalization.
- The release control uses synthetic identity rotations and a deterministic
  two-target corpus. Its 37 ms wall measures local search throughput only, not
  quality on a trained checkpoint.
- The first validation pass covered malformed pull corpora but did not yet pin
  the malformed and non-finite checkpoint cases required by this order. Those
  controls, deterministic metadata serialization and output-path collision
  refusal were added before delivery.
- Fresh exact-head review then found four material gaps: two-file output was not
  transactional under failures, custom pull corpora were identified only by
  path, the evaluation cap omitted candidate scoring, and the accepted
  checkpoint inventory was only the subset needed by this solver. A public
  helper also had unchecked token addition. All four are repaired and covered
  by tests at this head.
- Review against Claude-330's actual save path found two more compatibility
  defects hidden by the first fixtures. Candle saves rotations as
  `(width/4,4)`, not flattened `(width,)`, and the real loader uses the fixed
  Hamming depths `[2,6]` and rejects two-token copy keys on four-coordinate
  head halves. The solver and existing audit now validate those actual shapes
  and invariants.
- A later exact-head review found three delivery defects. The solver accepted
  ask tables at only some depths, the retained safetensors sidecar was a
  committed binary, and the evidence-retention test could overwrite a sealed
  prior root with `fs::copy`. The source reader now requires ask tables at
  either none or every depth; evidence is retained by hash rather than as a
  committed binary; and retention claims a fresh root and uses the same
  no-replacement pair writer as the CLI.
- The next review found that numeric parsing also accepted noncanonical depth
  aliases such as `d02.key_rotation`, and that the test-only evidence helper
  added a production-build warning. Canonical `d2`/`d6` names are now enforced,
  and the helper is compiled only for tests.
- Repository-wide `cargo fmt --all -- --check` is already red across many
  untouched files. The two new Rust files and their edited module are
  targeted-format clean; this unit did not reformat unrelated code.
- A strict `clippy -D warnings` run sees ten pre-existing warnings in
  `prompt_reply_rank.rs` and `suffix_arbitration.rs`. The normal scoped Clippy
  run passes, and neither new file adds a warning.

## What changed

- Added `AskOffsetSolveBasis`, `AskPullCorpus` and `cpu-ask-offset-solver`.
  The solver reads `embedding.weight`, `heads` and every
  `d{depth}.key_rotation`; it needs no trunk forward.
- The default corpus is the fixed B2 relation map `key -> key + 16`, tokens
  `16..32` to `32..48`. An optional weighted JSON corpus accepts repeated and
  multiple targets per question. Its raw bytes are SHA-256 bound in the report;
  `--expected-pulls-sha256` can require a pre-registered value.
- For each question/depth, candidates are the zero offset, every distinct
  exact target offset and a weighted coordinate median. Up to four best starts
  run a deterministic coordinate search at steps
  `16384,4096,1024,256,64,16,4,1`; an exact zero objective stops early.
- The objective is weighted XOR/popcount Hamming distance over the ask half of
  the heads after `hamilton_gate_f32` rotation and sign packing. There is no
  dense matrix multiplication or softmax in this path.
- Independent question/depth cells run through the existing Rayon pool.
  Ordered parallel collection and explicit sorting produce stable readings;
  tensor names are sorted before serialization.
- `--max-evaluations` now bounds every objective call per question/depth. A
  reading records candidate, coordinate and total objective evaluations,
  search starts and whether the budget stopped candidate selection or search.
- The output is a new complete `hamming_read.safetensors`: all original
  tensors and metadata are preserved, existing ask tables are replaced and
  every solved depth receives a full F32 `(vocab,width)` ask table. Safetensors
  header metadata is canonicalized for deterministic fresh-process bytes.
- The checkpoint reader now requires the complete Claude-330 per-depth
  query/key/value rotations, margins and weights, plus valid scalar or mixed
  copy keys in the actual `[width/4,4]` quaternion layout. It requires the
  fixed depths `[2,6]`, rejects unknown tensor names and enforces the
  canonical numeric depth spelling and the two-token key's eight-coordinate half
  rule. Existing ask tables must cover either none or every depth. Empty or
  zero-weight pulls, out-of-vocab tokens, bad shapes, invalid head geometry,
  partial depths, checked-overflow failures, non-finite values, type-spin mixing
  and invalid copy-key records fail closed.
- Output names are normalized through their real parent directories so aliases
  collide before solving. Candidate and report bytes are written to synced
  sibling temporaries and promoted without replacement. If the second
  promotion fails, the first promoted output is removed. This is rollback for
  ordinary synchronous failure; two separate final paths cannot be one
  crash-atomic filesystem operation, and process termination between promotions
  can leave the candidate without its report.
- The CLI reopens the candidate with `TrainedAskOffsets`, audits the fixed B2
  relation map, verifies preserved candidate metadata and non-ask tensor bytes,
  then re-audits every selected pull from the serialized candidate. It writes
  candidate/report hashes plus per-cell zero/solved/improvement readings.
- The deterministic evidence test creates its retention root exclusively and
  publishes the report and candidate through synced, no-replacement promotion.
  The repository retains the text report, hashes and measurement; the generated
  safetensors sidecar is not committed under the charter's binary hygiene rule.
- No serving, artifact-schema, training-default, GPU or Claude-owned file was
  changed.
- Size: `pr-check` reports 13 changed paths: code 6 files `+2,274/-6`,
  documentation 3 files `+285/-0`, and other/evidence/claim files 4 files
  `+1,082/-0` (`+3,641/-6` total). The ORDER gave no line estimate. Inline unit
  tests are counted in the code class.

## Measurements

The synthetic B2-style run used 64 tokens, width 8, two Hamming depths and one
copy/ask head pair. Over 16 questions and two depths it reduced total
weighted search bits from **8 to 0**, improved **2/32** readings and used **96**
total objective evaluations, all during candidate selection; no cell exhausted
its budget. The selected-corpus audit reproduced the same 8-to-0 result across
all 32 readings. The existing relation audit then measured **32/32** exact
codes and **32/32** target-nearest cells, versus **30/32** target-nearest for
the zero offsets. A repeated invocation with fresh output paths in the same
process was byte-identical. The selected-corpus verification makes another 64
objective calls after solving; those audit calls are outside the solver-search
budget and are not included in `total_evaluations`.

- Synthetic report SHA-256:
  `39ed4fcea8e8a73ba8507e5eb90c5340fdec237c7a4187918e2df83e850a7704`
- Generated candidate Hamming sidecar SHA-256 (not committed):
  `ef785c378684f997d24d91b026840854803c61c1a1d61b26ec35bbbc650d0d4a`
- Synthetic model input SHA-256:
  `2a7a8ed21c2a3b5d07c3a4c9e2a35a30c53fa91f0c261bb8da18fde546b49549`
- Synthetic Hamming input SHA-256:
  `f5891af995d5d3a46853d70133f64dc601d95b5ec175d5e9357e42855fda9e88`
- Default pull corpus SHA-256:
  `5b0b129803110a3efb90d063ca6c84bcdc9ac6ac0079ad2ccf733a09c61a80e0`
- Binary SHA-256 recorded by the retained report:
  `62d88fa129231fd10d429f2a27c6278315c2873f803045bb3f677895b4da7426`
- Resource measurement SHA-256:
  `12345a560898bf82b8da3da4ae89756971f5cf445c1736e53c567ed42b6785b1`
  ([measurement](evidence/resource-measurement.json)).

The isolated release control used width 256, two depths, 16 questions and two
targets per question. It completed **393,344** total objective evaluations in
**37 ms** across 16 workers, with no exhausted cell. Maximum resident set size
was 60,716 KiB for the benchmark and 236,360 KiB for the evidence test. The
full library suite peaked at 755,468 KiB. This is a throughput measurement, not
a trained-model or capability result.

Resources are recorded in the
[D84 ledger](../../../../labs/cpu-training-pilot-ask-offset-solver-2026-11-09/resource-ledger.json).
The previous admitted total `105,767.677487699 s` plus this unit's 1,800-second
projection gives `107,567.677487699 s`.

## Checks run at the head

- `cargo test --offline --locked -p uor-r4-cpu-train --lib ask_offset_solver::tests
  -- --skip release_width_256_two_depth_solver_benchmark`: 7 passed.
- `cargo test --offline --locked -p uor-r4-cpu-train --bin
  cpu-ask-offset-solver`: 4 passed.
- `cargo test --offline --locked -p uor-r4-cpu-train --lib`: 472 passed,
  5 ignored in 43.12 s; the timed process took 43.21 s and peaked at 755,468 KiB.
- After merging `origin/main` at `35f55d334`, the same library suite again
  passed 472 tests with 5 ignored in 37.91 s; the timed process took 38.01 s and
  peaked at 734,648 KiB. Both binaries' four-test suites also passed.
- `cargo test --offline --locked -p uor-r4-cpu-train --bin
  cpu-ask-offset-audit`: 4 passed.
- `cargo check --offline --locked -p uor-r4-cpu-train --all-targets`: passed.
- `RAYON_NUM_THREADS=16` on the release test binary with
  `--ignored --nocapture --exact
  ask_offset_solver::tests::release_width_256_two_depth_solver_benchmark`:
  passed at 393,344 total evaluations and 37 ms.
- The evidence-retaining binary test, run with 16 workers into a fresh root,
  reproduced detached report and candidate outputs in 5.96 s, peak RSS 236,360
  KiB.
- `cargo clippy --offline --locked --no-deps -p uor-r4-cpu-train --bin
  cpu-ask-offset-solver --bin cpu-ask-offset-audit`: passed with only
  pre-existing warnings in untouched files.
- Targeted `rustfmt --check` and `git diff --check`: passed.
- `uor-harness loop report --check`, `loop pr-check --local` and
  `loop gates --full`: passed.
- `cargo fmt --all -- --check`: failed on pre-existing formatting outside this
  unit.

## Skills used

No system-provided Codex skill applied. The repository CPU-research,
task-loop and delivery instructions were followed directly.

## Still open

- Run the solver against the exact Claude-330 B2 checkpoints when their paths
  and hashes exist, then compare held-out target-nearest and pull distance to
  the zero-offset checkpoint.
- Use the optional weighted corpus for real prompt-conditioned pulls; the
  built-in B2 relation map is a deterministic initializer, not a learned
  conversational objective.
- Treat the output only as a checkpoint-compatible ask-table candidate. B2
  must pass before implementing the Q12 ask marker/exponent and serving parity
  sequence already described in `deepseek-245`.
- If B2 is positive, run the same candidate through the trained-offset audit,
  CPU/integer serving construction and a held-out reply panel. None of those
  steps ran in this unit.

## Plain summary

This unit gives the workstation a fast way to construct Claude-330-compatible
question offsets from token geometry and supplied pulls. For each question
token and Hamming depth, it tests the zero offset, exact offsets to requested
target tokens and a robust median, then improves the best starts by exact
weighted Hamming distance over rotated sign codes. It writes a complete
checkpoint-like sidecar rather than mutating the source checkpoint.

The synthetic run proves that a valid candidate can be made, preserved through
safetensors, reopened by the existing audit and repeated byte-for-byte. It does
not prove that a real checkpoint learns well, that B2 carries the required
information or that a reply becomes conversational. Those remain gated on a
real Claude-330 checkpoint and the project's separate training/serving bars.
