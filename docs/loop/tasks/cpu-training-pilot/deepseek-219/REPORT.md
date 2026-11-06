# REPORT: cpu-training-pilot, deepseek-219

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Prep:
[prefix-length sheet](../../../../harness/prep/cpu-training-pilot-prefix-length-2026-11-06.md).
Base: `86996114f`.

## Status

Complete as **KEEP** for retrieved-prefix length completion.

The four-token retrieved prefix plus the nearest stored reply's length produced
**9/459** exact replies on artifact seed 1 and **10/459** on seed 2. It retained
body positions over model-only length completion and beat shuffled-prefix
length completion on both artifacts. Every reachable row emitted by length
without a BOS stop.

This is a development-only CPU mechanism KEEP, not a serving promotion. The
absolute exact rate is about 2%, and full retrieval remains stronger at
**33/459** exact replies on each artifact.

## What did not work

- No arm emitted BOS. The model still did not learn or derive a native stop;
  all 459 rows in every length arm stopped at the retrieved or oracle budget.
- Model-only generation with the same retrieved length remained at **0**
  exact on both artifacts, and shuffled-prefix length completion also remained
  at **0**.
- The tiny exact rate is not enough to displace exact retrieval in serving.
  Full retrieval stayed at **33/459** on each artifact.
- Independent review remained unavailable under the current delegation policy.
  FIT, GO and review are explicitly local exact-head self-reviews.

## What changed

Registered retrieved-prefix continuation with neighbor-length completion.
The existing experiment binary now has an opt-in length mode with model-only
cap, model-only retrieved-length, retrieved-prefix length, shuffled-prefix
length, full-retrieval and reference-length-oracle arms. Length and BOS stops
are counted separately, and length-stopped token identity can count as exact.

The implementation adds **362 code lines** in one binary. It does not change
the default continuation mode, serving default, sidecars, `UORLUT01`, CUDA,
float training or Claude-owned source.

## Measurements

Fresh panel:
`local/deepseek/prefix-length/2026-11-06/panels/dd-2782`, seed 2782,
`dev,dev`, protocol 2, 128 conversations, **459** reference-reachable replies
per artifact.

- tokens:
  `e9c8c4fd074e92960883e442ca97911d46d985145a170e86d15fa220943ae3ff`;
- mask:
  `91212bde0f1a825389e7cc6500dbf8e159c2c971d4ff02c661d5dff49ac047e6`;
- manifest:
  `e34af1d20ca48075dda43965a7283fbbff4e507a11581adb21d8b79a18fffa5e`.

| Artifact | Arm | Exact | Opening | Body | Length stops | BOS |
|---|---|---:|---:|---:|---:|---:|
| seed 1 | model-only, 64 cap | 0 | 0 | 70 | 0 | 0 |
| seed 1 | model-only, retrieved length | 0 | 0 | 49 | 459 | 0 |
| seed 1 | retrieved prefix 4, retrieved length | 9 | 106 | 452 | 459 | 0 |
| seed 1 | shuffled prefix 4, retrieved length | 0 | 21 | 72 | 459 | 0 |
| seed 1 | full retrieval | 33 | 106 | 762 | n/a | 0 |
| seed 1 | reference-length oracle | 0 | 0 | 70 | 459 | 0 |
| seed 2 | model-only, 64 cap | 0 | 21 | 58 | 0 | 0 |
| seed 2 | model-only, retrieved length | 0 | 21 | 41 | 459 | 0 |
| seed 2 | retrieved prefix 4, retrieved length | 10 | 106 | 427 | 459 | 0 |
| seed 2 | shuffled prefix 4, retrieved length | 0 | 21 | 80 | 459 | 0 |
| seed 2 | full retrieval | 33 | 106 | 762 | n/a | 0 |
| seed 2 | reference-length oracle | 0 | 21 | 58 | 459 | 0 |

Paired comparisons:

- seed 1 exact vs model-only length: **9 vs 0**, two-sigma threshold 2;
- seed 1 exact vs shuffled length: **9 vs 0**, threshold 2;
- seed 1 body vs model-only length: **452 vs 49**;
- seed 2 exact vs model-only length: **10 vs 0**, threshold 2;
- seed 2 exact vs shuffled length: **10 vs 0**, threshold 2;
- seed 2 body vs model-only length: **427 vs 41**.

Three fresh-process runs per artifact alternated artifacts and repeated
byte-for-byte after timing removal. All used an explicit 32-thread Rayon pool.

Resources:

- generator build: **13.69 s**, 1,192,540 KiB peak RSS;
- experiment binary build: **30.04 s**, 1,338,704 KiB peak RSS;
- panel generation: **0.58 s**, 7,524 KiB peak RSS;
- selector walls: **1.35-1.43 s**;
- maximum selector RSS: **206,024 KiB**;
- retained evidence: **128,485 bytes**;
- sealed local `SHA256SUMS`:
  `1cee8582ca5f374371f2e1f0cd7249b56babf0b39770b879187379ab6baedc41`.

Compact evidence: [result.json](evidence/result.json).

## Interpretation

The stored reply length supplies the missing completion budget after a correct
retrieved opening. It is a useful fallback mechanism, but it does not produce
native termination and only a small fraction of rows become exact. Full
retrieval remains the stronger and simpler behavior on this panel.

The next termination work should derive a stop from the continuation state.
Another length retune or longer prefix would not address the measured zero-BOS
result.

## Checks run at the head

- seed-2782 panel and output roots were absent at registration;
- focused length-completion tests: 4 passed;
- full `uor-r4-cpu-train` library: 399 passed, 4 expected ignores;
- scoped Clippy passed for the changed binary with the pre-existing library
  `type_complexity` lint exempted;
- scoped rustfmt and `git diff --check` passed;
- source and release binary hashes frozen before GO;
- fresh panel matched the requested seed, cell, protocol and conversation
  count;
- six alternating selector processes emitted all 459 reachable rows;
- repeated decisions and replies matched exactly after timing removal;
- every length arm recorded positive bounded budgets and zero BOS stops;
- sealed 24-file local evidence checksum verified;
- full harness check: 284 rules, 0 errors and 251 repository-wide advisory
  warnings;
- D23, D25 and D32 ratchets unchanged, claim wording passed, and prep-check
  reports FIT with compute allowed;
- `xtask links`: 1,277 pre-existing repository-wide broken links and none among
  the newly added prefix-length evidence links;
- `xtask docs-catalog --check`: 1,175 records, 20 lines, zero missing paths;
- `xtask roadmap check` remains blocked by the pre-existing
  `infra-desktop-client-tauri` status/issue mismatch, which this diff does not
  touch;
- implementation review: local exact-head self-review at `f5cb4f7a9`.

## Delivery

Pending review, protected queue merge and fresh-main verification.

## Skills used

No system-provided Codex skill applied.

## Still open

- a model-derived stop that removes reliance on a retrieved length;
- a serving integration only if a later check shows unique complete replies
  over full retrieval;
- Claude's independent end-to-end serving track.

## Plain summary

The stored reply length fixes the observed zero-BOS failure as a completion
budget: a correct retrieved opening now produces a few exact complete replies,
and the mechanism passes both matched controls on both artifacts. Native
termination and absolute response quality remain the larger blockers.
