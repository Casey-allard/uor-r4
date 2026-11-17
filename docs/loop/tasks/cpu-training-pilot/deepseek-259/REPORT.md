# REPORT: cpu-training-pilot, deepseek-259

Lab: deepseek. Order: [ORDER.md](ORDER.md). Parent:
[deepseek-258](../deepseek-258/REPORT.md).

## What did not work, first

- The sixth fresh exact-head review found that the resource projection's five
  components summed to 3,603.25 seconds against a 3,600-second wall limit.
- The same review found the `NOT_RUN` record stale after Claude-334's sealed
  run merged. The original 06:07Z store probe had returned HTTP 404, but the
  merged Claude report records successful reads of both seed tarballs.
- The Linux nonregular-file test covered a directory but not a FIFO. The
  production open used `O_NONBLOCK`, yet no test protected that property.
- The seventh review found that this repair changed the deepseek-255 and
  deepseek-256 reports even though they were absent from the declared file
  scope. It also found that the exact `recall.jsonl` comparison was prose
  only, with no committed expected-versus-observed receipt.

## What changed

- Downloaded both Claude-334 seed tarballs and verified their recorded SHA-256
  values before extraction.
- Audited all eight real held checkpoints: `B2-aqH` and `B2-aH`, both seeds,
  at steps 4,096 and 16,384, with eight workers.
- Retained the eight-arm manifest and complete CPU report as text evidence.
  The original 404 inventory remains as a time-bounded observation.
- Replaced the stale `NOT_RUN` summaries in deepseek-254, deepseek-257,
  deepseek-258 and the notebook with the completed real audit.
- Reduced the projection's retry/setup allowance from `2,011.73` to
  `2,008.48` seconds so the five components sum to the admitted 3,600 seconds.
- Added a Linux FIFO regression with a ten-second bounded wait. It proves the
  nonblocking open fails closed without letting a blocking regression hang
  the test.
- [deepseek-260](../deepseek-260/ORDER.md) re-scopes the 255/256 report
  corrections and adds the exact gain/type-weight parity receipt.

## Measurements

- Claude-334 seed tarballs: seed 1
  `57cbd25a941f56458436caaa258b0afbe8a79a5520d5f31a371e7b33cf795d93`;
  seed 2
  `4d952047d69e4f80bcd1385d95c0189d268c1949d82503941c70e8395d2314fd`.
- Real audit: eight checkpoints in `0.03 s` wall at `520%` CPU and
  `171,240 KiB` peak RSS.
- Parity: all eight question/header/value/rest gain groups and both per-depth
  type-weight means matched Claude's `recall.jsonl` exactly.
- Temporary download, extraction and local report storage peaked at
  `582,545,293` bytes and is removed after delivery.
- Exact deepseek-254 through deepseek-259 task files plus the resource ledger:
  `371366` bytes.
- Final `pr-check` size: tests 0 files `+0/-0`; code 2 files `+986/-109`;
  docs 14 files `+888/-0`; generated 0 files `+0/-0`; other 6 files
  `+9955/-0`.
- Focused binary tests: 9 passed. Full CPU library suite: 480 passed, 5
  ignored.
- Artifact hashes: manifest
  `e1287b9b7f31766ab0545654fa6c4c4fb8b303f41ffcceadceb7c5d4e0b78c24`;
  report
  `497140c6a6730521b3337c8f155cd31771aade953203eb1ee4eab7d51ee58865`.
- Exact parity receipt:
  [real-eight-gain-parity.json](../deepseek-254/evidence/real-eight-gain-parity.json),
  SHA-256
  `be065856b20a7cc56a0a71b1dacd02d6a188927f00bb303521fdc3b4a1cce6d1`;
  all eight expected/observed arms compare equal.
- Test executable
  `87d481a87f003b19aa675a04d720ae058ca32bf08650b1b3e9080055d12cb665`;
  release executable
  `aef6083c6c47180d743abf3521716a12841316cecdb35b94d6768f8af171a31e`.

## Checks run at the head

- Scoped `rustfmt --check` and `git diff --check`: passed.
- Focused binary tests: 9 passed.
- Full `uor-r4-cpu-train` library suite: 480 passed, 5 ignored in 39.44 s.
- `cargo check -p uor-r4-cpu-train --all-targets`: passed.
- Scoped Clippy with `--no-deps`: passed with pre-existing crate warnings.
- Release build: passed.
- Report policy, local policy and all 11 gates: passed at the final receipt
  head.
- The seventh review requested changes; deepseek-260 repairs them. An eighth
  fresh exact-head review is requested after the final receipt.

## Skills used

No system-provided Codex skill applied.

## Still open

Claude-334 owns the B2 interpretation. The CPU report independently reproduces
its logged diagnostic fields but makes no carry, B2, capability, export or
serving claim. Future held checkpoints need a new manifest and hashes.

## Plain summary

The CPU audit now runs on the real sealed checkpoints and matches every logged
hold diagnostic exactly. This repair also makes the resource arithmetic exact
and protects the nonblocking FIFO behavior with a bounded test.
