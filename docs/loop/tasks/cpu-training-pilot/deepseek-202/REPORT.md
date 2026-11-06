# REPORT: cpu-training-pilot, deepseek-202

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Prep:
[continuation-closure sheet](../../../../harness/prep/cpu-training-pilot-continuation-closure-2026-11-05.md).
Base: `d56427955`.

## Status

Complete as **REJECT**, with a non-vacuous continuation mechanism. Whole-reply
closure reduced premature stops relative to tokenwise content, but both arms
produced 17 exact replies and treatment still made 71/64 premature stops
against the four-stop ceiling.

The positive content selector remains intact. This closes the fixed
strict-majority whole-continuation closure configuration, not the broader
idea of learned continuation.

## What did not work

- Treatment produced **17/17** exact replies, tied with tokenwise content
  **17/17**; the paired exact comparison therefore failed on both artifacts.
- Treatment made **71/64** premature stops, below tokenwise **74/98** but far
  above the registered maximum of four.
- Body positions tied `content_no_boundary` at **682/678**, above local
  **478/478** and closure-only **238/243**.
- Closure-only produced only **12/12** exact replies with poor body positions,
  so content/local arbitration remains necessary.
- The shuffled closure produced **0/0** exact replies and **602/598** body
  positions.
- Most closure events preserved tokenwise trajectories: only **8/39** complete
  replies changed versus tokenwise, although **187** closure rows differed
  from tokenwise or local output when the union was counted.

## What changed

- Added `continuation_closure.rs`, which stores complete remaining replies for
  full-history suffixes at depths four through 32.
- Added strict-majority whole-sequence confidence, same-depth source coupling,
  first-token agreement, empty endpoints, cap/context fallback and complete
  continuation digests.
- Added source closure, tokenwise content, closure-only, shuffled closure,
  no-boundary, local, writer and oracle arms.
- Added a deterministic equal-depth continuation shuffle and tests for
  confidence ties, empty endpoints, cap/context fallback, source-depth
  isolation, digest binding, decision accounting and shuffle invariants.

No serving, CUDA, float-training, exporter or Claude-owned source changed.

## Measurements

Fresh panel:
`local/deepseek/continuation-closure/2026-11-05/panels/dd-2776`, seed 2776,
`dev,dev`, protocol 2, 128 conversations, 449 reference-reachable replies per
artifact.

- tokens:
  `a1a5306ed32f3f7332f54c5025c0b8a43070cbf5ebd0f3fc90467f45eebd6240`;
- mask:
  `652c3c69b89422e2ef17c4eb0c69d30936ea59d2cabd982117ca2f91218df5a3`;
- manifest:
  `cd5052645a346440d89b323b00b93ba7c7d60f43e6cf97bf1f062eeed2c3b28e`.

The closure table contained **80,728** suffixes, **111,656** observations,
**715,428** continuation tokens and **11,083** empty endpoints. The shuffle
changed **79,384** continuation assignments while preserving per-depth
observation and token totals.

| Seed | Arm | Exact | Premature | Opening | Body | BOS |
|---:|---|---:|---:|---:|---:|---:|
| 1 | source closure | 17 | 71 | 57 | 682 | 187 |
| 1 | tokenwise content | 17 | 74 | 57 | 682 | 194 |
| 1 | closure only | 12 | 78 | 15 | 238 | 186 |
| 1 | shuffled closure | 0 | 18 | 57 | 602 | 22 |
| 1 | content no-boundary | 0 | 0 | 57 | 682 | 0 |
| 1 | local reply boundary | 0 | 3 | 35 | 478 | 8 |
| 1 | writer | 0 | 0 | 0 | 57 | 0 |
| 1 | content oracle | 30 | 0 | 57 | 682 | 449 |
| 2 | source closure | 17 | 64 | 57 | 678 | 187 |
| 2 | tokenwise content | 17 | 98 | 57 | 678 | 225 |
| 2 | closure only | 12 | 19 | 27 | 243 | 54 |
| 2 | shuffled closure | 0 | 18 | 57 | 598 | 22 |
| 2 | content no-boundary | 0 | 0 | 57 | 678 | 0 |
| 2 | local reply boundary | 0 | 3 | 35 | 478 | 8 |
| 2 | writer | 0 | 0 | 12 | 56 | 0 |
| 2 | content oracle | 30 | 0 | 57 | 678 | 449 |

Treatment beat closure-only and shuffled closure on exact replies, but tied
tokenwise content and failed the premature-stop and complete exact bar.

Resources:

- release build: **0.52 s**, 218,388 KiB peak RSS;
- generator build: **10.86 s**, 1,258,252 KiB peak RSS;
- TRAIN preflight: **3.48 s**, 873,216 KiB peak RSS;
- panel generation: **0.65 s**, 7,884 KiB peak RSS;
- seed 1 / seed 2: **3.62 / 3.73 s**, **897,012 / 917,012 KiB** peak RSS;
- seed 1 / seed 2 repeat: **3.65 / 3.68 s**;
- normalized repeats matched exactly;
- retained evidence: **10,045,094 bytes**;
- sealed `SHA256SUMS`:
  `899bbca44564d63705496103846096ec74f2698a2aaf859865afe9e7372d6669`.

Compact evidence: [result.json](evidence/result.json).

## Interpretation

Completion-level prediction preserves the content trajectory but does not
repair its endpoint decisions. Removing the separate BOS-only table changes
some whole replies, but the confidences and continuation endings still encode
the same premature endpoint signal.

The next termination work must add genuinely new state or supervision, such as
prompt-conditioned writer geometry, generated-state continuation distance, or
an explicit response-length objective. Another suffix-key or complete-mode
permutation over the same exact history would repeat this closed configuration.

## Checks run at the head

- source review: APPROVE at `d628faa94`;
- non-author TEST FITNESS: FIT at `414ec5f0a`;
- separate D67 pre-run review: GO at `607f67655`;
- focused continuation-closure tests: **9 passed**;
- full CPU release suite: **389 passed**, 4 expected ignores;
- strict scoped Clippy and formatting: passed;
- source/input/generator pins, panel manifest and reachability: passed;
- shuffle, closure power and all control gates: passed;
- both fresh-process repeats matched after timing removal;
- failure-path compression and retained-size enforcement: passed;
- final result seal: valid.
- final exact-head review: **APPROVE** at `42536cb25`;
- all required PR checks: pass.

## Delivery

PR #3759 was reviewed at exact head
`42536cb25443336e1cd7b057486be262da62acf3` with a counted
`uor-review v1` APPROVE and all required checks green. The protected queue
merged it as `d2313599a94f84e57e40d2fa9c72d1cf38d07fb7`.

Fresh `main` has identical content for all 14 changed files. The CPU package
passed **389 tests** with four expected ignores, source and compact evidence
hashes match the registration, and the pre-registration is complete.

## Skills used

No system-provided Codex skill applied. Repository pre-registration, FIT/GO,
D84 CPU research and protected-delivery procedures were used.

## Still open

- termination conditioned on a new continuation or geometric state signal;
- fresh-phrasing generalization beyond exact-memory recall;
- Claude's independent scale, export and serving track.

## Plain summary

Whole-reply continuation lookup reduced premature stops somewhat and preserved
body quality, but it tied the tokenwise baseline on exact replies and still
over-stopped heavily. This closes the fixed strict-majority continuity mode.
The next stopping experiment needs a new state signal rather than another
representation of the same suffix history.
