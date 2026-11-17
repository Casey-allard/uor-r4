# REPORT: cpu-training-pilot, deepseek-255

Lab: deepseek. Order: [ORDER.md](ORDER.md). Parent:
[deepseek-254](../deepseek-254/REPORT.md).

## What did not work, first

- The first review found that deepseek-254 validated one canonical checkpoint
  path, discarded it, and resolved the path again in each worker. A directory
  symlink changed between those steps could escape the manifest root. Child
  files also were not checked separately.
- Four review findings followed: no assertion pinned the rest-token gain mean,
  raw path strings rather than canonical identities guarded duplicates,
  single-checkpoint mode silently ignored `--workers`, and the recorded diff
  size was stale by one line.
- The second exact-head review confirmed the containment repair, then found
  that a basename-only manifest path canonicalized an empty parent. It also
  caught stale resource-ledger bytes and notebook counts.
- The seventh review found that deepseek-259 updated this report outside its
  declared file scope. [deepseek-260](../deepseek-260/ORDER.md) re-scopes and
  records the correction.

## What changed

- Manifest validation now canonicalizes the root once and returns the exact
  canonical checkpoint directories it checked.
- Workers audit those retained directories instead of resolving untrusted
  manifest strings again.
- Each `model.safetensors` and `hamming_read.safetensors` is canonicalized,
  required beneath that root, opened as a regular file, and on Linux checked
  through `/proc/self/fd` before its bytes are read. Other platforms fail batch
  mode closed because they cannot provide this pinned-descriptor check.
- Canonical checkpoint paths are the duplicate identity. Single-checkpoint mode
  now rejects batch-only arguments. The synthetic test directly asserts the
  rest-token mean.
- A child-file symlink to a checkpoint outside the manifest root has a focused
  refusal test.

## Measurements

The retained synthetic reports are unchanged in their readings; only the test
executable identity changed:

- single SHA-256:
  `1d2014a91240db057728ce27e6554dffef1774bc2a8a6322237d2e060ed09f42`;
- batch SHA-256:
  `95b39b12dd3684abe23c67d32f51b86dd2f5ffe83abf0191cf22d256726a4673`;
- release executable SHA-256:
  `aef6083c6c47180d743abf3521716a12841316cecdb35b94d6768f8af171a31e`.

Focused binary tests passed 9/9. The full CPU library suite passed 480 tests
with 5 ignored in 47.22 s.

## Checks run at the head

- Scoped `rustfmt --check` and `git diff --check`: passed.
- Focused binary tests: 9 passed.
- Full `uor-r4-cpu-train` library suite: 480 passed, 5 ignored.
- `cargo check -p uor-r4-cpu-train --all-targets`: passed.
- Scoped Clippy with `--no-deps`: passed with only pre-existing crate warnings.
- Release build: passed.
- `loop report --check` and `loop pr-check --local`: passed.
- All 11 `loop gates --full` gates: green.
- The second fresh review requested changes; [deepseek-256](../deepseek-256/ORDER.md)
  repairs its findings.

## Skills used

No system-provided Codex skill applied.

## Still open

The original store probe was `NOT_RUN` at 06:07Z. After Claude-334 published,
deepseek-259 audited all eight real held checkpoints and reproduced their
recorded gain groups and type-weight means. A moved gain still does not by
itself prove carry, B2, export, serving or language capability.

## Plain summary

This repair makes the batch audit read only the root-bounded checkpoint files
it validated, rather than resolving the manifest path a second time. It also
closes the review's test and interface gaps.
