# REPORT: cpu-training-pilot, deepseek-247

Lab: deepseek. Order: [ORDER.md](ORDER.md).

## What did not work, first

- `uor-harness loop claim cpu-training-pilot --lab-topic real-ask-regression`
  refused on the known umbrella-scope false overlap with an unrelated open
  one-model-end-to-end order. That order and this unit share no file; this
  ORDER is hand-written and the policy gate checks the actual delivery.
- The first push was blocked because the ORDER landed before its REPORT. The
  REPORT skeleton was added before the draft PR; no product code existed at
  that point.
- A workspace-wide `cargo fmt --all -- <files>` invocation formatted unrelated
  files in this fresh worktree. Those accidental modifications were identified
  from the clean worktree's status and reverted exactly; all later formatting
  used `rustfmt` on the two new files only.
- Strict `clippy -D warnings` still fails on ten pre-existing warnings in
  `prompt_reply_rank.rs` and `suffix_arbitration.rs`. The normal scoped Clippy
  run passes and neither new file adds a warning.
- The regression proves the stored ask-selection geometry only. It does not
  measure carry from the question row to `H4`, B2, answer quality or serving.
- The first fresh exact-head review returned CHANGES. The reproduction root
  disagreed with `uor-store`'s path-preserving download, duplicate checkpoint
  identities were accepted, source failures were classified as regressions,
  the notebook overgeneralized B2-a, and the ledger omitted temporary inputs.
  The [repair ORDER](../deepseek-248/ORDER.md) fixes all five.
- A second fresh review confirmed those repairs but found three record
  residuals: archive failure behavior was described as retained, 18,191
  downloaded metadata bytes were omitted and the ledger timestamp preceded its
  measurements. [deepseek-249](../deepseek-249/ORDER.md) fixes them.
- A third fresh review confirmed every code/evidence repair and found only that
  the final documentation changed the task-byte total after the ledger update.
  [deepseek-250](../deepseek-250/ORDER.md) re-measures the final tree.

## What changed

- Added `ask_offset_regression`, a strict manifest model for multiple
  checkpoints. It validates the source store/path, archive and checkpoint
  hashes, unique arm ids, archive references, checkpoint paths and checkpoint
  hash pairs, plus plain relative paths, relation settings and expected
  relation counts before work starts.
- Added `cpu-ask-arm-regression`. It verifies every sealed archive by streaming
  SHA-256, audits independent checkpoint pairs through a bounded Rayon pool,
  preserves manifest order, compares every summary field and writes one
  deterministic report only to a new path. A field mismatch exits `2` after
  the report; malformed source evidence is retained in the report and exits
  `1`.
- Added tests for a passing real-shape checkpoint, deterministic repeats,
  named expected-field mismatch, wrong checkpoint hash, wrong archive hash,
  missing Hamming data, manifest order, unsafe paths, duplicate ids, duplicate
  archives and argument refusal.
- Added the eight real Claude-330 checkpoint cases already measured: B1-a,
  B1-a0, B2-a and B2-aq at seeds 1 and 2. The manifest pins both store archives
  and all eight `model.safetensors` / `hamming_read.safetensors` hashes.
- Added reproduction instructions whose root agrees with `uor-store`'s
  full-path download layout, and a compact report. No checkpoint,
  tarball, generated model artifact or executable is committed.
- No float-training, carry, serving, schema, GPU or other-lab file changed.
- Size: `pr-check` reports 17 changed paths: code 4 files `+1,082/-0`,
  documentation 10 files `+574/-0`, and other text evidence 3 files `+806/-0`
  (`+2,462/-0` total). The ORDER gave no line estimate.

## Measurements

The release run verified archives `e2e67a8e…` (89,069,111 bytes) and
`2e8ec2b3…` (88,944,852 bytes), then audited all eight checkpoint pairs with
**8 workers** in **0.10 s**. It consumed **221% CPU**, peaked at **183,260 KiB**
RSS and passed **8/8** arms.

| Arm | Seed 1 / seed 2 target nearest | Reading |
|---|---:|---|
| B1-a | 64 / 64 | Found-copy pull trains selection |
| B1-a0 | 9 / 1 | Task loss alone is near chance |
| B2-a | 0 / 3 | Scored-row pull does not train question selection |
| B2-aq | 64 / 64 | Question-row pull trains selection |

- A fresh 8-worker report was byte-identical.
- A one-worker run matched every report field except `workers`.
- Changing B2-a seed 2's expected target-nearest count from 3 to 4 produced a
  named mismatch and exit `2`, while the other seven arms still passed.
- Replacing one checkpoint path with a missing path produced one source
  failure, zero regression failures and exit `1`.
- Manifest SHA-256:
  `5a5ad55c7759942b82ff5b94d658453433faa22fdf3bd8000c67b6933766b136`.
- Regression report SHA-256:
  `3f497f58059c152954599433b8ccc860a89e7bb74e327b3472873b9be4dbbc9b`.
- Release executable SHA-256:
  `2b53623d48b15ce9b8fa0ab4f0bee4863a6d9364ca22914ce9fe942b28bb92b6`.

Resources are in the
[D84 ledger](../../../../labs/cpu-training-pilot-real-ask-regression-2026-11-09/resource-ledger.json).
The previous admitted total `107,567.677487699 s` plus this unit's 1,800-second
projection gives `109,367.677487699 s`.

## Checks run at the head

- `cargo test --offline --locked -p uor-r4-cpu-train --lib
  ask_offset_regression::tests`: 6 passed.
- `cargo test --offline --locked -p uor-r4-cpu-train --bin
  cpu-ask-arm-regression`: 3 passed.
- `cargo test --offline --locked -p uor-r4-cpu-train --lib`: 478 passed,
  5 ignored in 42.23 s; the timed process took 42.32 s at 2,408% CPU and
  peaked at 692,448 KiB.
- `cargo check --offline --locked -p uor-r4-cpu-train --all-targets`: passed.
- `cargo clippy --offline --locked --no-deps -p uor-r4-cpu-train --bin
  cpu-ask-arm-regression --lib`: passed with only pre-existing warnings.
- `cargo build --release --offline --locked -p uor-r4-cpu-train --bin
  cpu-ask-arm-regression`: passed.
- Targeted `rustfmt --check` and `git diff --check`: passed.
- `uor-harness loop report --check`, `loop pr-check --local` and
  `loop gates --full`: passed; all 11 full gates were green.
- `uor-harness check`: 363 rules, 0 errors and 329 pre-existing warnings,
  predominantly merged-claim and orphan-branch hygiene outside this unit.

## Skills used

No system-provided Codex skill applied. The repository's CPU-research, task
loop and delivery instructions were followed directly.

## Still open

- Claude-333 Unit A merged as `d7cc2b930`. Its CUDA checks, Unit B readings and
  carry run remain in the next Claude unit and are outside this order.
- Once carry code produces checkpoints, this runner can prove that the ask-table
  split did not drift; a separate carry evaluator must locate any remaining
  loss between the question and `H4`.
- The downloaded checkpoint data stays only in the gitignored local scratch
  root and is removed after delivery.

## Plain summary

This unit turns the earlier one-off audit into a reusable test. It first checks
the exact sealed archives and checkpoint files, then asks the same
question-key geometry question for four trained arms on two seeds, in
parallel. It expects the saved answer counts exactly and names the field when
one changes.

The result reproduces B2-aq's question-row selection and B2-a's scored-row
failure, while preserving B1-a's positive found-copy result. It does not make
the selected value survive the next four token positions. That carry remains
the current model blocker for Claude's separate track.
