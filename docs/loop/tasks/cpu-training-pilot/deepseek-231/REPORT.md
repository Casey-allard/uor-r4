# REPORT: cpu-training-pilot, deepseek-231

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Prep:
[joint-branch sheet](../../../../harness/prep/cpu-training-pilot-joint-branch-2026-11-06.md).
Base: `b4a7aac9f`.

## Status

Complete as **INSUFFICIENT_POWER** before fresh-panel generation.

The TRAIN preflight could not supply endpoint supervision for the fresh
false-boundary problem. No seed-2787 panel was generated and no fresh result is
claimed.

## What did not work

- The first generated-spine preflight used the full TRAIN suffix and boundary
  tables. It produced **77** supervised groups and **273** candidate rows, but
  all **77** first-boundary choices were already exact. It therefore had zero
  examples of choosing a later endpoint.
- The fixed document-grouped four-fold preflight derived memory from three
  document folds and inspected reference-prefix branches in the held-out fold.
  It produced **0** candidate rows and **0** supervised groups across all
  **453** TRAIN rows.
- Reference prefixes alone do not reproduce the fresh generated-prefix false
  stops: suffixes that are terminal in a fresh panel can remain nonterminal in
  every TRAIN held-out document. The original supervision design therefore
  stopped below its registered floor of 96 groups and one baseline-wrong group.

This is not a fresh mechanism negative and not a control fault. The tested
configuration has insufficient TRAIN supervision.

## What changed

Added `joint_branch.rs`, a bounded integer whole-branch selector:

- one full-sequence geometric content spine records every boundary endpoint
  without committing a stop;
- 15 integer features combine whole-prefix content evidence with endpoint
  boundary evidence;
- a deterministic pairwise perceptron ranks complete candidates;
- arms cover learned selection, matched shuffled labels, first boundary,
  longest spine, terminal-volume and reference endpoint;
- document-grouped four-fold TRAIN fitting and a `--preflight` gate stop before
  fresh compute when power is absent.

Added the `cpu-joint-branch-driver` and six focused tests. The implementation
adds **1,703** code and test lines across the two new Rust files plus 13 lines
of crate integration, above the item's current code estimate. No serving
default, artifact format, `UORLUT01`, CUDA, float path or Claude-owned source
changed.

## Measurements

| Preflight | TRAIN rows | Groups | Baseline wrong | Candidates | Outcome |
|---|---:|---:|---:|---:|---|
| full memory, generated spine | 453 | 77 | 0 | 273 | below power |
| four-fold, reference prefix | 453 | 0 | 0 | 0 | below power |

Final preflight timing:

- processes: one, 16 requested Rayon workers, 249% observed CPU;
- wall: **0.35 s**;
- user/system: **0.83 / 0.04 s**;
- peak RSS: **112,152 KiB**;
- retained evidence: **32,035 bytes** including the checksum manifest.

Inputs:

- TRAIN tokens:
  `140cf50d638d6066aec0a31345183712b6fc5d95502384223b91af9504b48f91`;
- TRAIN mask:
  `d0aaa2b1007e62098ab943187f25e13bddbaed3508dce881a176c2827052c0ad`;
- TRAIN manifest:
  `d570caedbb628167c59b96b14746863b99998c2f67289d1deca16dbebd402ba5`;
- artifact seed 1:
  `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd`.

Compact result: [result.json](evidence/result.json). Raw preflight:
[preflight.json](evidence/preflight.json). Raw earlier generated-spine
receipt: [preflight-full-memory.json](evidence/preflight-full-memory.json).
The default no-flag run also stopped before arm evaluation:
[auto-stop.json](evidence/auto-stop.json).

## Interpretation

The previous boundary guards failed because fresh generated trajectories stop
at suffixes that are continuation-only in TRAIN. Training a selector on the
TRAIN reference trajectory cannot observe that mismatch: the first boundary is
already the reference endpoint everywhere the pipeline has enough TRAIN
support.

The next endpoint unit needs supervision generated under the same distribution
shift that causes fresh false boundaries, or a reference-free objective that
does not ask a pairwise ranker to infer an unobserved preference. Another
feature set over the same TRAIN groups would repeat the insufficient-power
fault.

## Checks run at the head

- focused joint-branch tests: **6 passed**;
- driver test: **1 passed**;
- full CPU library: **414 passed, 4 expected ignores**;
- scoped strict Clippy passed with the pre-existing `type_complexity` and
  `needless_borrow` exemptions;
- Rustfmt check on the two new Rust files and `git diff --check` passed;
- the fresh seed-2787 panel and output were not created.

## Skills used

No system-provided Codex skill applied.

## Still open

- reference-free or distribution-matched supervision for generated-prefix
  false boundaries;
- training content and termination jointly over a representation that exposes
  the fresh shift;
- Claude's independent end-to-end local-chat track.

## Plain summary

The complete-branch code and controls are implemented, but TRAIN never shows
the selector a wrong first stop to learn from. The registered power gate
stopped the unit before a fresh panel, so the honest result is insufficient
power rather than another fresh negative or a tuned retry.
