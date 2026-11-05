# REPORT: cpu-training-pilot, deepseek-193

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Prep:
[suffix-memory sheet](../../../harness/prep/cpu-training-pilot-copy-suffix-memory-2026-11-05.md).
Base: `1f1e3a756`.

## Status

Complete as **REJECT** for the fixed trained-reply-suffix plurality/backoff
configuration.

The fresh oracle had enough power: **14** exact bodies on each artifact seed.
The treatment nevertheless reached only **57** and **86** correct body
positions, while local `Longest32To1` reached **514** on both seeds. The
treatment also exceeded the premature-BOS ceiling, stopping **14** and **227**
replies early. This closes the tested content-plus-termination suffix table, not
exact reply memory generally.

## What did not work

- **The primary body comparison rejected on both artifact seeds.** Seed 1
  scored `57` bodies versus local longest's `514`, with `48` candidate-only
  against `70` control-only and a threshold of `17`. Seed 2 scored `86` versus
  `514`, with `65` against `69` and the same threshold.
- **The depth-one control also rejected the longer table on seed 2.** Seed 2
  scored `86` against `89` for depth one, below the paired threshold of `14`.
  Seed 1's `57` against `48` passed that isolated comparison, so longer context
  supplied no replicated body gain.
- **Termination was too eager.** Treatment stopped all `447` reachable replies
  with BOS and made `14` and `227` premature stops against the registered
  maximum of four. The shuffled table stopped `447` and `443` replies early,
  and depth one stopped `298` and `333` early.
- **The table cannot carry the opening under this construction.** It inspects
  generated reply tokens only, so its first decision has no suffix and always
  falls back to the writer. Opening correctness was `0/0` on seed 1 and
  `19/19` on seed 2, against local longest's `34/34`.
- **Frequent short terminal suffixes are not a safe fresh continuation
  signal.** The table learned `16,150` suffixes and `4,644` BOS transitions
  from 128 TRAIN conversations. After the fallback opening, the treatment made
  `12,963` and `4,023` known-suffix decisions, but those decisions displaced the
  local exact-copy body with a much weaker fresh distribution.
- **The first TEST FITNESS review was NOT_FIT.** It found that seed 2771 had
  already been used, non-vacuity was not yet restricted to reachable rows,
  `distinct_targets_by_depth` counted successor entries rather than unique
  targets, and no complete-reply integration test exercised the new selector.
  All findings were repaired before FIT or compute.
- This is not a rejection of suffix memory, learned continuation, geometric
  attention or useful chat. It closes the exact plurality, longest-backoff,
  full-content-plus-BOS table trained from this small TRAIN panel.

## What changed

- Added `suffix_memory.rs` with TRAIN-only suffix-to-next-token counts at depths
  one through 32, plurality selection with smaller-token ties, decoder-BOS
  targets, writer fallback and a successor-multiset-preserving shuffle.
- Added `cpu-copy-suffix-memory` with six arms:
  `trained_suffix_32`, `trained_suffix_depth_1`, `shuffled_suffix_32`,
  `local_longest_32_boundary`, `writer_boundary` and a reference-length oracle.
- Added seven focused tests, including complete-reply writer override,
  BOS prediction, unknown-suffix fallback, shuffled-trajectory separation and
  per-depth target-multiset preservation.
- Added the FIT/GO pre-registration and this compact evidence record.

No serving, CUDA, float-training, exporter or Claude-owned source changed.

## Measurements

Fresh panel:
`local/deepseek/copy-suffix-memory/2026-11-05/panels/dd-2772`, seed 2772,
`dev,dev`, protocol 2, 128 conversations, 447 reference-reachable replies.

- tokens SHA-256:
  `6bbbc8ed23e3b67f73889c277fbabe6593ba366bc4c021c4f2d237aea8eefec7`;
- response-mask SHA-256:
  `82e3b29e9a5fb3e4680d4734b4c8c39f2eba67cfc4410d04c9bfd84def1b4009`;
- manifest SHA-256:
  `03fa58eeac0981e7332a427fc98377a02d4215277fcf6a6677f1628a2f8989ba`.

The true table contained **16,150** suffixes, **33,284** transitions, **4,644**
BOS transitions and **710** suffix keys with more than one observed target.
The deterministic shuffle changed **15,601** suffix-to-successor associations
while preserving each suffix's observation count and every depth's target
multiset.

| Seed | Arm | Exact | BOS | Premature | Opening | Body | tok/s |
|---:|---|---:|---:|---:|---:|---:|---:|
| 1 | local longest-32 | 0 | 17 | 9 | 34 | 514 | 67,919 |
| 1 | trained suffix-32 | 0 | 447 | 14 | 0 | 57 | 45,208 |
| 1 | trained depth-1 | 0 | 447 | 298 | 0 | 48 | 15,891 |
| 1 | shuffled suffix-32 | 0 | 447 | 447 | 0 | 0 | 2,468 |
| 1 | writer | 0 | 0 | 0 | 0 | 52 | 70,457 |
| 1 | reference oracle | 14 | 447 | 0 | 34 | 514 | 21,532 |
| 2 | local longest-32 | 0 | 17 | 9 | 34 | 514 | 67,889 |
| 2 | trained suffix-32 | 0 | 447 | 227 | 19 | 86 | 19,828 |
| 2 | trained depth-1 | 0 | 447 | 333 | 19 | 89 | 12,960 |
| 2 | shuffled suffix-32 | 0 | 447 | 443 | 19 | 23 | 4,862 |
| 2 | writer | 0 | 0 | 0 | 19 | 51 | 67,954 |
| 2 | reference oracle | 14 | 447 | 0 | 19 | 514 | 21,796 |

Treatment changed `13,062/4,394` reachable steps from local longest and
`447/894` from the shuffled table on seeds 1 and 2. The controls were therefore
behaviorally non-vacuous.

Resources:

- TRAIN preflight: **1.85 s**, 340,208 KiB peak RSS;
- seed 1 / seed 2: **1.87 / 1.81 s**;
- seed 1 / seed 2 repeat: **1.91 / 1.82 s**;
- maximum measured process RSS: **341,748 KiB**;
- every non-timing field and reading repeated exactly;
- the sealed local `SHA256SUMS` read back clean.

Compact evidence: [result.json](evidence/result.json).

## Checks run at the head

- first non-author TEST FITNESS: **NOT_FIT**, findings repaired;
- corrected non-author TEST FITNESS: **FIT**;
- separate D67 pre-run review: **GO**;
- focused suffix-memory tests: **7 passed**;
- full CPU release suite: **370 passed**, 4 expected ignores, all binary tests
  passed;
- strict scoped Clippy with `--all-targets --no-deps -- -D warnings`: passed;
- formatting, `git diff --check`, `prep-check`, input/source pins and output
  sealing: passed;
- both artifact repeats matched after timing removal;
- no fresh panel was generated before FIT and GO.

## Reading

The suffix table is an active exact-memory mechanism: it generated known-suffix
decisions, predicted BOS, differed from its shuffle and was deterministic.
Those facts are not enough. On fresh phrasing, plurality over the full TRAIN
reply table rapidly selected terminal or weak continuations and destroyed the
local exact-copy body advantage. The result closes this table construction.

The next continuation work should either:

1. separate content and stopping into different memories and test content
   without BOS targets;
2. require fresh-support evidence or a learned confidence gate before a global
   suffix may override longest local copy; or
3. train on a materially larger reply corpus where the suffix distribution has
   coverage outside the small development panel.

Do not retune this depth, target tie or shuffle on seed 2772.

## Skills used

No system-provided Codex skill applied. Repository pre-registration, FIT/GO,
D84 CPU research and protected delivery procedures were used.

## Still open

- complete-reply/fresh-phrasing capability on the one-model path;
- Claude's exact export/greedy-parity and scale-probe work;
- a continuation mechanism that preserves local copy while improving
  completion rather than displacing it.

## Plain summary

A global exact suffix table learned from 128 TRAIN conversations was active,
but its plurality continuation displaced the stronger local exact-copy body
and stopped fresh replies too early. The oracle supplied enough exact bodies to
make the rejection decisive rather than underpowered.
