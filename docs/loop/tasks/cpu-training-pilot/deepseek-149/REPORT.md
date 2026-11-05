# REPORT: cpu-training-pilot, deepseek-149

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `4e42ffca5`.

## Status

Complete at a positive bounded screen. The longest exact suffix key across
depths four to one beat both the `k2>k1` and random-depth controls on two newly
generated panels. This selects a read construction for the next trained unit;
it does not train or qualify a model.

## What did not work

- **The first two attempts were input-path instrument faults.** Attempt 1 used
  normalized panel filenames for public DEV; attempt 2 corrected token and mask
  but still used `manifest.json` instead of `dev-manifest.json`. Both stopped
  before writing any target reading and remain sealed.
- **The successful gain is small.** `longest_4_to_1` gained eight correct
  positions over `k2>k1` on each new panel, about half a percentage point of
  copyable positions. It is a candidate selected by the registered paired
  rule, not a large capability improvement.
- **This is not a trained-read result.** The screen uses exact token matches
  before the model; it does not prove that a trained copy read improves
  openings, answers or complete replies.

## What changed

Added `uor-r4-cpu-train`'s `cpu-copy-key-depth` binary and its exact
`copy_key_depth` module. It reproduces Claude-267's 192-token BOS/window
semantics, scans fixed suffix depths one through four, and compares:

- `k2>k1`;
- longest available depth, four to one;
- fixed depth one, the shortest control;
- a deterministic random available-depth control.

Candidate matching excludes the current query position, never crosses a BOS
document, and uses `x[0]` for missing startup history. The public panel is
pin-only parity. Two new deterministic m-world-v2 panels were generated under
the registered seeds after FIT and GO.

## Measurements

Public DEV reproduced the instrument exactly: **8,243** copyable positions,
depth 1 **6,306 / 3,859**, depth 2 **4,383 / 3,506**, and `k2>k1`
**4,388**.

New `train,train` panel, seed 2501:

| Arm | Right / copyable |
|---|---:|
| `longest_4_to_1` | **815 / 1,508** |
| `k2>k1` | 807 / 1,508 |
| random depth | 790 / 1,508 |

The candidate beat `k2>k1` with 9 candidate-only and 1 control-only decisions
against threshold 2. It beat random depth with 32 candidate-only and 7
control-only against threshold 6.

New `dev,dev` panel, seed 2502:

| Arm | Right / copyable |
|---|---:|
| `longest_4_to_1` | **826 / 1,594** |
| `k2>k1` | 818 / 1,594 |
| random depth | 798 / 1,594 |

The candidate beat `k2>k1` with 8 candidate-only and 0 control-only decisions
against threshold 2. It beat random depth with 33 candidate-only and 5
control-only against threshold 5.

Both new panels had more than 200 eligible positions, the random control
changed 49 and 48 copyable predictions respectively, and the dev-panel repeat
matched byte for byte.

Resources:

- release build **0.06 s**, peak **54,212 KiB**;
- generator build **0.05 s**, peak **44,592 KiB**;
- panel generation **0.41 s + 0.47 s**, peaks **7,760 / 8,120 KiB**;
- all three attempts measured **21.42 s** in total;
- driver binary **1,232,848 bytes**, SHA-256
  `7e83410218a111f4f741bf86946209eb3efb8da1d393f7e22246cb04964f26ce`;
- **267** library tests passed with 4 expected ignores; 6 focused copy-key
  tests, Clippy and rustfmt passed;
- admitted cumulative projection: **70,927.677487699 s**.

Exact output is under
[`evidence-attempt-3/`](../../../../labs/cpu-training-pilot-copy-key-depth-2026-11-05/evidence-attempt-3/).
The raw sealed `SHA256SUMS` is
`bec19988e1b161973de1037f444ffc1516093f2bd33f54df4a35ee52a09c3a22`.
The two failed attempts and their hashes remain in the same lab record.
Generated corpora are omitted from Git; the committed evidence verifies with
`PUBLISHED-SHA256SUMS`, while the committed panel manifests pin both corpus
hashes and generator seeds.

## Checks run at the head

- Fresh FIT and independent GO covered the corrected attempt-3 recipe.
- The command block passed `bash -n`; source and generator pins matched.
- Public DEV parity, all input hashes, both ≥200-row power floors, both random
  non-vacuity controls, RSS limits and fresh-process determinism passed.
- The three evidence manifests verify independently.
- No serving, integer, CUDA, float-training or Claude-owned source changed.
- Exact-head adversarial review passed at `4daadc58b`; PR #3690 merged as
  `ab0a759cd`.
- Fresh `main` passed **267** library tests with 4 expected ignores, the
  copy-key binary test, evidence-manifest verification and result-field checks.

## Skills used

No system-provided Codex skill applies.

## Still open

- Train copy key depths three and four, or a longest-first backoff, through the
  float model and export/serve path.
- Compare that trained read against `k2>k1` on complete replies and fresh
  phrasing.
- Measure whether the small exact-match gain survives learned weighting,
  export and integer serving.
- Complete replies, useful chat and model capability remain open.

## Plain summary

Using the longest exact history match, up to four tokens, was better than using
two tokens and falling back to one. The gain held on two newly drawn panels and
beat a random-depth control, but it was only eight more correct next tokens per
panel. It is worth testing in the trained copy read; it is not yet a language
or serving result.
