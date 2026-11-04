# REPORT: cpu-training-pilot, deepseek-110 (memory-weight and target-arbitration replies)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Order: [ORDER.md](ORDER.md). Prep:
[FIT memory replies](../../../harness/prep/cpu-training-pilot-memory-replies-2026-11-03.md).
Base: `5707380fe`.

## What did not work, first

- **The first completion run had an invalid memory-off arm.** It passed the
  trained artifact to a complete-reply evaluator that always loaded memory, so
  `trained_memory_off` duplicated `trained`. The exact-head review caught this.
  The first raw roots are retained as
  `local/deepseek/memory-replies/deepseek-110/{seed-1,seed-2,seed-1-repeat}-invalid-memory-off`;
  a genuine
  `CpuFbsForward::disable_memory` path and point-key regression were added
  before rerun.
- **Neither memory arm produced a complete reply.** Every one of the 114 prompts
  on every arm hit the 16-token cap on both seeds. Exact-reference and
  BOS-complete counts were zero, with zero context stops.
- **The trained movement weights did not propagate.** Seed 1 moved the opening
  from 0 to 14 correct, but still reached zero complete replies and repeated
  4-grams rose from 26 to 91. Seed 2's trained and frozen arms were byte-identical
  in output, with opening 4.
- **Direct exact-memory targets did not carry the reply.** They raised opening
  correctness to 50/114 on both seeds and produced 15 distinct replies, but the
  subsequent autoregressive path still hit the cap with zero exact or
  BOS-complete replies. The shuffled target control reached 10 openings.
- **Direct target output was insensitive to trained fallback weights.** The
  `target_frozen` and `target_trained` output digests were identical on each
  seed, so the trained weights did not change the exercised target-arbitration
  trajectory.

## What changed

Added a bounded complete-reply evaluator to `uor-r4-cpu-train`:

- every prompt token is stepped once;
- reply start is marked before the final prompt token;
- generated tokens are fed back until shared integer BOS, cap or context stop;
- one trailing BOS is removed from the reference;
- exact-reference, reachability, opening, distinct-text, repeated-4-gram,
  generated-token and digest counters are deterministic;
- ordered per-row reachability/exactness survive into the report, allowing
  paired `b/c`, gain, integer threshold and pass fields for every registered
  comparison.

`cpu-memory-panel-driver` now accepts:

- `--reply-cap N`, which adds eight matched complete-reply readings;
- `--expected-trained-sha256 HEX`, which refuses artifact drift before outputs
  are used.

The arms are frozen memory, trained memory, equal-accepted-move random,
memory-off and trained-memory-off. Three direct-target arms use the stored
memory target at an exact key hit and the exact CPU writer only on a miss:
frozen, trained-fallback and trained-fallback with a deterministic
shuffled-target control.

Complete replies now have a genuine memory-disabled path. It retains the
artifact's trunk and hash, clears turn memory and the point fallback, and has a
finite test proving an artifact point key is ignored.

## Measurements

The 114 references hold 1,326 tokens after one trailing BOS is removed.
**104/114** fit the fixed 16-token cap and 192-token context. The other ten are
26–36 tokens and are counted unreachable.

Training reproduced deepseek-109 exactly on both seeds. Both runs accepted all
32 updates and reproduced trained artifact hashes
`c856ce21…b88de4` and `10f91f73…40e6`. Teacher-forced top-1 remained
`507→526` and `441→484`, with distances below frozen and random.

Source was `bccbf9295d011727d005e827f47cb1d0afde2c81`; release binary
`867a118bb21ced8f884d1a011d60a19bc129cc2b1c0ec559b79b4ed0994bdf01`;
host was the AMD Ryzen 9 9950X on Linux 7.0.0-42-generic with Rust 1.97.1.

Corrected complete-reply readings:

| Seed | Arm | Exact | BOS | Cap | Opening | Distinct | Repeated 4-gram |
|---:|---|---:|---:|---:|---:|---:|---:|
| 1 | frozen | 0 | 0 | 114 | 0 | 2 | 26 |
| 1 | trained | 0 | 0 | 114 | 14 | 3 | 91 |
| 1 | random | 0 | 0 | 114 | 0 | 2 | 26 |
| 1 | memory-off | 0 | 0 | 114 | 0 | 2 | 26 |
| 1 | trained-memory-off | 0 | 0 | 114 | 0 | 2 | 26 |
| 1 | target-frozen | 0 | 0 | 114 | 50 | 15 | 76 |
| 1 | target-trained | 0 | 0 | 114 | 50 | 15 | 76 |
| 1 | target-shuffled | 0 | 0 | 114 | 10 | 15 | 81 |
| 2 | frozen | 0 | 0 | 114 | 4 | 2 | 48 |
| 2 | trained | 0 | 0 | 114 | 4 | 2 | 48 |
| 2 | random | 0 | 0 | 114 | 4 | 2 | 48 |
| 2 | memory-off | 0 | 0 | 114 | 4 | 2 | 48 |
| 2 | trained-memory-off | 0 | 0 | 114 | 4 | 2 | 48 |
| 2 | target-frozen | 0 | 0 | 114 | 50 | 15 | 37 |
| 2 | target-trained | 0 | 0 | 114 | 50 | 15 | 37 |
| 2 | target-shuffled | 0 | 0 | 114 | 10 | 16 | 61 |

Generated-token totals and aggregate rates:

| Seed | Arm | Tokens | Generated tokens/s |
|---:|---|---:|---:|
| 1 | frozen | 1,824 | 24,043.1 |
| 1 | trained | 1,824 | 21,846.6 |
| 1 | random | 1,824 | 21,540.5 |
| 1 | memory-off | 1,824 | 25,044.8 |
| 1 | trained-memory-off | 1,824 | 25,080.3 |
| 1 | target-frozen | 1,824 | 21,168.0 |
| 1 | target-trained | 1,824 | 21,692.1 |
| 1 | target-shuffled | 1,824 | 25,003.3 |
| 2 | frozen | 1,824 | 21,622.2 |
| 2 | trained | 1,824 | 24,989.6 |
| 2 | random | 1,824 | 22,068.5 |
| 2 | memory-off | 1,824 | 23,361.3 |
| 2 | trained-memory-off | 1,824 | 25,000.4 |
| 2 | target-frozen | 1,824 | 21,061.6 |
| 2 | target-trained | 1,824 | 21,050.7 |
| 2 | target-shuffled | 1,824 | 24,646.6 |

Every paired exact-reference comparison covered 104 reachable rows, had
`b+c=0`, gain 0 against a threshold of 2, and failed the registered margin.
The five served query-movement arms retained zero mismatches over 4,333
decisions, 1,109,248 Q12 coordinates and 34,664 kept cells per seed.

The shuffle control changed **102/114** exercised opening mappings and
**773/795** exercised reply mappings while preserving keys and weights.

Walls were 37.762 s and 38.007 s; peak RSS was 395,808 and 386,892 KiB. A fresh
seed-1 process reproduced the complete report with all timing fields removed,
reproduced every artifact hash byte for byte, and ran in 38.099 s at 390,156
KiB peak RSS.

The detailed record is [comparison.json](evidence/comparison.json);
[SHA256SUMS](evidence/SHA256SUMS) binds the binary and generated artifacts. The
model-free ceiling remains in [reference-census.json](evidence/reference-census.json).

## Checks run at the head

- Eight focused reply-evaluator tests passed, including deterministic
  query-movement and direct-target replies over exported synthetic CPU
  artifacts and a point-key test proving the memory-off path ignores memory.
- The driver control test verifies the shuffled snapshot preserves keys and
  weights through artifact export/read-back while changing exercised targets.
- Full CPU release suite: 172 library tests passed, 4 release benchmarks
  ignored; every binary suite passed.
- Strict all-target CPU Clippy with `-D warnings`: passed.
- `cargo fmt -p uor-r4-cpu-train -- --check`: passed.
- `git diff --check`: passed.
- `uor-harness prep-check`: sheet completed at `FIT`; the corrected rerun ran
  against the exact FIT-reviewed source.
- Model-free `--select-only --reply-cap 16` reproduced 16 keys, 102 training
  examples, 114 evaluation examples and the 104-reachable-reference census.
- Both public seeds and the fresh seed-1 repeat completed under FIT.
- Both seeds reproduced the registered trained artifact hashes and exact
  five-arm CPU/serving parity.

## Skills used

The repository's `experiment-prereg` instructions governed the FIT sheet.
No system-provided skill applied.

## Reading

The registered complete-reply question is negative. Movement weights and direct
exact-memory targets can change the first token, but neither produces a
complete or naturally terminated reply in this configuration. Because every
paired exact-reference result is `0=0`, no larger fresh-phrasing run or serving
implementation is justified for this target-arbitration formulation.

This closes the fixed 16-key / 102-training / 114-evaluation, 32-update,
16-token greedy memory-target configuration. It does not reject every memory
key or every proposal path. The next work must change how reply continuation
and termination are represented, not add more movement-weight steps or another
width of this target arbiter.

## Still open

- A continuation mechanism that can escape the fixed 16-token cap.
- A new memory-key or proposal rule; the current direct-target formulation is
  closed.
- Serving implementation only if a future positive arm exists.
- Complete replies, fresh phrasing, useful chat and GPU qualification.

## Plain summary

Both public seeds and a fresh seed-1 repeat ran the five query-movement and
three direct-target arms through complete greedy replies. Memory weights raised
teacher-forced top-1, but every complete-reply arm hit the cap with zero exact
and zero BOS-complete replies. Direct targets raised opening correctness to
50/114 but did not complete a reply.

The result closes this full-path memory-target configuration. The next step is
a continuation and termination mechanism, not more coordinate updates or a
larger version of the same target arbiter.
