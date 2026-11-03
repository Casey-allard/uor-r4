# REPORT: batched scorer, deepseek-9

Lab: deepseek. Item:
[one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Order: [ORDER.md](ORDER.md). Base: `b6ebf7002`.

## What did not work

- **The first host-only reading was incomplete.** This workstation has no GPU,
  so the original unit could not establish CUDA parity itself. Claude's
  independent RTX 5090 run supplied the missing equality and timing evidence.
- **Scoped, forced-opening and served-key arms remain serial.** Their batch state is not yet rectangular or independent per sample, and weakening exactness for them would defeat the unit.
- **Mixed prompt lengths are never padded.** They remain separate groups. Padding changes recurrent history and memory-row indices, so a larger apparent batch is not worth a different model.
- **The GPU reading is one checkpoint, seed and panel.** It does not cover seed
  2, P-dev, scoped reads, forced-opening, served-key or the seven ignored CUDA
  tests. The flag stays off by default.

## What changed

`train-in-space --score-rows --batched-decode 1` now groups full-scope nearest decodes by exact prompt length. Each group is split under the existing evaluation batch/context budget and runs one rectangular recurrent forward per decode step. Results are restored to prompt order, and Flock randomness still uses each prompt's original global sample index.

Terminated rows receive a BOS placeholder for tensor shape only and are never read again. Active rows keep their exact histories. `Greedy`, `FlockRank`, `FlockCell` and `CycleStep` share the path; every unsupported arm falls back to the existing one-at-a-time decoder.

## Measurements

- Pinned DEV produced 310 prompts over 75 distinct lengths.
- Exact-length grouping reduces them to 75 groups; the largest has 32 prompts and the mean is 4.13.
- Host grouping/fallback controls: **3 passed**. Every prompt appears once, all members of a group have the same length, grouping is deterministic, wide groups respect the `16 × 192` evaluation-token budget, and every unsupported branch stays serial.
- The complete float-train library suite passed on this host.
- On one RTX 5090, the CUDA float-train library suite passed **300 tests, 0
  failed, 7 ignored**, including
  `exact_length_batches_match_one_at_a_time_decodes`.
- Claude used claude-262's served seed-1 step-256 checkpoint and the same
  `--score-rows --samples 1536 --reply-tokens 64 --retain-rates 1 --seed 1`
  command with `--batched-decode 0` and `1`. All five output files were
  byte-identical:
  `rows-dev-memory.json`, `rows-dev-no-memory.json`,
  `samples-dev-flock-memory.json`, `samples-memory.json` and
  `samples-no-memory.json`.
- Full-scope decode through the whole-table nearest read was faster while
  exact:

  | Samples file | Tokens | Serial | Batched | Speedup |
  |---|---:|---:|---:|---:|
  | `samples-memory.json` | 19,465 | 223.6 s | 63.3 s | 3.53× |
  | `samples-no-memory.json` | 19,465 | 222.6 s | 63.0 s | 3.53× |
  | `samples-dev-flock-memory.json` | 11,305 | 130.7 s | 49.9 s | 2.62× |

  Whole `--score-rows` wall time fell from 581 s to 180 s.

The parity run used the tested tree's exact `train-in-space` blob and Claude's
expected `in_space_long` blob. After merging current `main`, the latter differs
only by Claude's `claude-265` assertion that the served transition is unsnapped;
the batched production path is unchanged. The sealed evidence is HF
`results/claude/pr-3583-parity-2026-11-03/p3583.tar`, SHA-256
`e13ac5a544f5e50742ef1f1eb2c8181d56ff3828a55b4cc9eba8d5df795f2601`.

The exact-head review then found one boundary defect: group size used the
initial prompt width even though a near-context prompt grows toward
`SHEET_CONTEXT` during decoding. Grouping now uses the widest window a batch can
reach, and a near-context host test enforces the evaluation token budget. This
can change group boundaries but not any row's decoding arithmetic. No RTX 5090
offer was available for a fresh current-blob run; the measured parity above is
therefore retained as the tested-path reading, while a current-blob rerun stays
open and the feature remains off by default.

## Checks run at the head

- Fresh-target `cargo test --locked -p uor-r4-float-train --lib`: **217 passed,
  0 failed**.
- `cargo check --locked -p uor-r4-float-train --bin train-in-space`: passed.
- `git diff --check`: passed.
- Package `cargo fmt --check` is not clean on current `main`: it reports a
  pre-existing 6,000-line formatting diff across this crate. The unit does not
  reformat unrelated source.
- Strict no-dependency Clippy reaches pre-existing `deny` failures in the
  package. The one new lint from the grouping fix was removed; the remaining
  findings predate this unit.
- CUDA test `exact_length_batches_match_one_at_a_time_decodes`: **passed on the
  RTX 5090**. It compares complete `DecodedReply` fields under all four rules
  with Hamming reads and opening/reply memory rows active, and requires a
  non-empty reply.

## Skills used

No available Codex skill applied. The serial decoder, failure surface, prompt/corpus controls and integer serving path were read directly.

## Still open

- Repeat parity on seed 2, P-dev, scoped reads and the forced-opening and
  served-key arms that still run serially.
- Re-run complete parity on the current blob after the near-context grouping
  fix when RTX capacity is available.
- Make batching the default only after GPU equality and a reviewed speed reading both hold.

## Plain summary

The scorer can now batch prompts of exactly equal length without padding, while
every uncertain arm stays on the original serial path. Host grouping and the
full float library pass. On one RTX 5090 checkpoint and seed, serial and batched
decodes produced five byte-identical files and batching ran 2.62–3.53× faster.
The feature remains opt-in and off by default until more arms and panels are
covered.
