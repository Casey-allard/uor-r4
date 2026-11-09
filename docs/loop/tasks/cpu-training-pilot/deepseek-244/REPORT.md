# REPORT: cpu-training-pilot, deepseek-244

Lab: deepseek. Order: [ORDER.md](ORDER.md).

## What did not work, first

- The first synthetic window used question row `183`, which is not a registered
  B2 tail row. The registered rows are `168, 174, 180, 186`; the probe now uses
  the far slot at `186`, scores `H4` at `190`, and pins both facts in a test.
- The first retention summary collapsed every ask head into one all-head
  boolean. Partial success was therefore invisible. The report now keeps the
  exact `(depth, head)` cells retained for each relation.
- The first fitter allocated two width-sized vectors on every candidate
  evaluation. It now reuses point buffers for the entire bounded fit.
- A fresh review found that the first weighted-retention pass used the saved
  type-head margins. Ask heads start with copy-head margins of `head_width / 4`
  bits, not the trained type margin. The oracle now reports ask-initialized and
  type-margin weights separately. The corrected ask-weight totals are `0/64`
  before fitting and `40/64` after fitting on each seed.
- The shared-offset search reached zero Hamming bits on `0/16` relations. It
  retained the target in the nearest list in `49/64` and `48/64` cells, but only
  `40/64` and `40/64` had nonzero ask-initialized selector weight. It is not a
  learned ask query yet.
- Global `cargo fmt -p uor-r4-cpu-train -- --check` failed on pre-existing
  ordering elsewhere in `src/lib.rs`. Only the new files were formatted; they
  pass `rustfmt --check`.

## What changed

- Added `AskHeadOracle`, built directly from the served integer table and
  Hamming rotations. It constructs the question-keyed query, exposes exact and
  bounded shared offsets, computes total target-key Hamming bits, and returns
  nearest-kept positions with the existing tie rule.
- Added `cpu-ask-head-oracle`. It binds both artifact hashes, models the B2
  far tail slot with one fact, runs all 16 relation/query pairs in parallel,
  reports per-depth/per-head target distances, nearest-list retention and
  nonzero selector weight, and refuses replaced reports.
- Exposed the existing sign-code helpers inside the CPU crate only. No serving
  schema, training branch, serving default or Claude-owned file changed.
- Added five library tests and three binary tests for an independently signed
  copy-query parity check,
  exact targeting, deterministic bounded fitting, nearest retention,
  positive-weight retention, malformed/read-less artifacts, the valid B2
  tail/H4 geometry and SHA-256 argument validation.
- Changed 3 tracked source files and added 2 source files, plus the REPORT,
  two evidence reports, one resource ledger and one notebook row. The change is
  about 950 code/test lines; the ORDER gave no line estimate.

## Measurements

The inputs were Claude-322 seed 1
`86e51d5ee2c00a4041bcde4d2183884e351099f28f155fb1e6a7516f13064dfe`
and seed 2
`8838cd1044ae7152934d8ec09016e268f2dabb0c702dd45e6ab06d7b5198bdae`.
The release executable was
`1620773790737878288e753900ae1e808553cc9820ebd6326c6a352b2de449db`.
Each run used 16 Rayon workers and this command, substituting the seed, hash
and output:

```sh
RAYON_NUM_THREADS=16 target/release/cpu-ask-head-oracle \
  --artifact <b64L-seed.uorlut> --expected-artifact-sha256 <hash> \
  --out <report.json>
```

| Seed | Nearest list: before / exact / fitted | Ask-init weight: before / exact / fitted | Saved type margin: before / exact / fitted |
|---|---:|---:|---:|
| 1 | 3 / 64 / 49 of 64 | 0 / 64 / 40 of 64 | 3 / 64 / 49 of 64 |
| 2 | 2 / 64 / 48 of 64 | 0 / 64 / 40 of 64 | 2 / 64 / 48 of 64 |

| Seed | Start bits mean | Fitted bits min / median / max | Fit evaluations | Budget exhausted |
|---|---:|---:|---:|---:|
| 1 | 122.375 | 10 / 57.5 / 112 | 82,432 | 0 |
| 2 | 126.5625 | 12 / 30.5 / 116 | 83,968 | 0 |

Exact offset to the found-copy key produced zero bits for all `16/16`
relations and retained the target with nonzero ask-initialized weight in all
`64/64` cells on both seeds. The bounded shared fit put the target in the
nearest list for `15/16` relations on seed 1 and `14/16` on seed 2. With
ask-initialized margins it contributed nonzero weight in `13/16` and `11/16`
relations. Across relations, fitted ask-weight retention by cell was:

| Seed | depth-2 head 2 | depth-2 head 3 | depth-6 head 2 | depth-6 head 3 |
|---|---:|---:|---:|---:|
| 1 | 13 / 16 | 7 / 16 | 13 / 16 | 7 / 16 |
| 2 | 10 / 16 | 10 / 16 | 11 / 16 | 9 / 16 |

Both fresh-process reports were byte-identical on repeat. Seed report hashes
are `d72f2d03…ca33` and `61f9dd0e…4484`; complete per-relation cells and
per-depth/per-head starting, exact and fitted Hamming bits are in
[`seed-1.json`](evidence/seed-1.json) and
[`seed-2.json`](evidence/seed-2.json).

The two `time -v` runs each took about `0.02 s` wall with 9,008 KiB maximum
RSS. This is a coarse sub-second timing, not a throughput qualification.
Resources are recorded in the
[D84 ledger](../../../../labs/cpu-training-pilot-ask-head-oracle-2026-11-08/resource-ledger.json).
The previous admitted total `102,167.677487699 s` plus this unit's
1,800-second projection gives `103,967.677487699 s`.

## Checks run at the head

- `cargo test --offline --locked -p uor-r4-cpu-train --lib
  ask_head::tests`: 5 passed.
- `cargo test --offline --locked -p uor-r4-cpu-train --bin
  cpu-ask-head-oracle`: 3 passed.
- `cargo test --offline --locked -p uor-r4-cpu-train --lib`: 461 passed,
  4 ignored.
- `cargo check --offline --locked -p uor-r4-cpu-train --all-targets`: passed.
- `cargo build --release --offline --locked -p uor-r4-cpu-train --bin
  cpu-ask-head-oracle`: passed.
- `rustfmt --edition 2021 --check` on both new source files: passed.
- `cargo clippy --offline --locked --no-deps -p uor-r4-cpu-train --bin
  cpu-ask-head-oracle`: passed with only pre-existing warnings in untouched
  files; the new files add none. `-D warnings` fails on that existing debt.
- Both seed reports ran successfully; fresh repeats were byte-identical.
- `UOR_LAB=deepseek uor-harness loop report --check`: passed.
- `UOR_LAB=deepseek uor-harness loop gates --quick`: 3 green.
- `UOR_LAB=deepseek uor-harness loop gates --full`: 11 green.
- `git diff --check`: passed.

## Skills used

No system-provided Codex skill applied. The repository's CPU-research and
delivery instructions were followed directly.

## Still open

- This is a one-fact, query-row oracle. It does not run B2, score `R_ask`, or
  show that a retained value position survives the four-token carry to `H4`.
- Exact geometric targeting is not a learned mechanism. Claude-330 must train
  the shared ask offsets or equivalent query transform and then pass B2 under
  its registered sheet.
- The coordinate fit is a diagnostic local search, not a global optimum or a
  training procedure. No relation exhausted its 200,000-evaluation cap.
- A trained ask-head artifact and a full B2 trace remain required before any
  capability or serving claim.

## Plain summary

This unit asks a narrow geometric question. Suppose a fact stores a relation
word as a key and its answer at the next position. B2 later shows a different
question word, such as `Q(K)`, with four header tokens before the answer is
scored. The oracle computes the ask query as the sign pattern of the question
word's vector plus a learned offset, rotated in quaternion blocks. It compares
that pattern with the earlier key patterns and keeps the nearest 16 positions.

With an exact offset calculated only for the experiment, the question query
can match the key perfectly and retain the answer position in every ask head.
With an offset learned by a bounded shared search, many but not all heads keep
it. Those retained positions are only the first half of B2: the model still has
to train the offset and carry the chosen value through the header. The value of
the result is that it gives Claude a concrete CPU target and a per-head gap to
close without spending GPU time to discover the same geometry.
