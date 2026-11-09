# REPORT: cpu-training-pilot, deepseek-248

Lab: deepseek. Order: [ORDER.md](ORDER.md). Parent: [deepseek-247](../deepseek-247/REPORT.md).

## What did not work, first

The fresh exact-head review found five material issues in deepseek-247:

- The README passed the original scratch root to the runner even though
  `uor-store get REMOTE/ DIR` preserves `REMOTE` below `DIR`.
- Duplicate arm IDs and archive paths were refused, but the same checkpoint
  could be repeated under new IDs and still pass as a full arm set.
- Missing or hash-mismatched checkpoints were counted with expected-value
  regressions and exited `2`, despite the REPORT saying source evidence
  exits `1`.
- The notebook generalized B2-a's scored-row failure to scored-row training
  generally, contradicting B1-a's 64/64 target-nearest result.
- The COMPLETE ledger recorded retained text and build storage but omitted the
  downloaded archives and extracted checkpoint tree.

## What changed

- The README now downloads below `SCRATCH/results/.../runs-2`, extracts under
  that run root and passes the same root to the runner.
- Manifest validation now rejects duplicate checkpoint paths and duplicate
  `(model_sha256, hamming_sha256)` pairs. The order test uses two distinct
  checkpoint files.
- Reports expose source and regression failure counts. The CLI writes the
  report before exiting `1` for any source failure or `2` for any observed
  summary mismatch.
- The notebook names B2-aq and B2-a precisely and retains B1-a's contrasting
  positive result.
- The ledger records 178,013,963 downloaded archive bytes, 378,392,745
  extracted checkpoint-tree bytes and 556,406,708 total temporary input bytes.

## Measurements

- Missing-checkpoint control: 1 source failure, 0 regression failures, exit 1.
- Mutated-summary control: 0 source failures, 1 regression failure, exit 2.
- Real rerun after repair: 8/8 arms pass in 0.10 s, 221% CPU, 183,260 KiB peak
  RSS; the 8-worker repeat is byte-identical and one-worker content matches.
- Full CPU library suite after repair: 478 passed, 5 ignored in 42.23 s;
  process wall was 42.32 s at 2,408% CPU with 692,448 KiB peak RSS.

## Checks run at the head

- Targeted regression library tests: 6 passed.
- Regression binary tests: 3 passed.
- Full CPU library suite: 478 passed, 5 ignored in 42.23 s; process wall was
  42.32 s at 2,408% CPU and 692,448 KiB peak RSS.
- `cargo check --offline --locked -p uor-r4-cpu-train --all-targets`: passed.
- Scoped Clippy: passed with only pre-existing warnings.
- Release binary rebuild: passed.
- Repaired real checkpoint manifest: 8/8 passed.
- Source and regression exit-code controls: passed as described above.
- 8-worker repeat was byte-identical; one-worker content matched.
- `loop report --check`, `loop pr-check --local` and `loop gates --full`:
  passed; all 11 gates were green.

## Skills used

No system-provided Codex skill applied.

## Still open

The second fresh exact-head review returned CHANGES on three residual
documentation and storage details. [deepseek-249](../deepseek-249/REPORT.md)
repairs them and owes the final review.

## Plain summary

This repair makes the documented path work, refuses accidental duplicate
checkpoints, distinguishes broken source evidence from a changed result, fixes
the research wording and records the temporary disk footprint.
