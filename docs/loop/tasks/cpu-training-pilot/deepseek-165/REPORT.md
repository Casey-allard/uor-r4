# REPORT: cpu-training-pilot, deepseek-165

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `7018b09cd`.

## Status

Complete and merged. The registered geometric copy hypothesis is **REJECT**.

## Question

Can sign-Hamming agreement between the current Q12 writer query and each
token's shared vector-table row choose a better next token among the exact-copy
candidates than depth-first longest-32 selection?

## What did not work

Two independent FIT reviews blocked the first implementation states: the
distance test reused the implementation as its oracle, and the stateful
geometric replay plus random-32 behavior lacked direct coverage. Both were
repaired with finite tests before FIT and GO.

The experiment itself failed its bar. Ranking exact-copy candidates by static
sign-Hamming distance from the Q12 writer query to the token's shared
vector-table row was non-vacuous but substantially worse than depth-first copy.

## What changed

- Added `GeometricCopy32` over the existing exact-copy candidate set.
- Added deterministic ranking by distance, deeper depth and lower token id.
- Bound the selected distance into the decision digest.
- Added eight-arm evaluation output, oracle naming and a geometric-change
  control against longest-32.
- Added independent sign-Hamming, distance-ordering, tie, replay and random-32
  control tests.
- Registered the bounded CPU development screen and recorded independent FIT
  and GO.

## Measurements

- implementation pin: `2378b0414c674923a2c62de44865260f3b778abc`;
- FIT reviewer: fresh-context Codex subagent `fit_geometric_review_3`;
- GO reviewer: fresh-context Codex subagent `go_geometric_review`;
- GO head:
  `8c941102764f61538e4aa4a9beb4669d87a53901`;
- fresh panel: 128 conversations, seed 2762, protocol 2, `dev,dev`;
- reachable replies: **453 train / 443 fresh** per artifact seed;
- decisions: **28,992 train / 28,352 fresh** per arm and seed;
- geometric body positions: **174 and 195 / 4,661** train, **270 and
  286 / 4,822** fresh;
- longest-32 body positions: **409 / 4,661** train and **624 / 4,822** fresh
  on both seeds;
- fresh controls on body positions: `k2>k1` **547**, fixed depth 1 **218**,
  random-32 **414**, writer **49 and 60**;
- geometric changed **4,017 and 3,927** train decisions and **5,106 and
  4,891** fresh decisions versus longest-32;
- random-32 changed **2,295** train and **2,921** fresh decisions;
- every geometric copy decision carried a selected distance; every fallback
  preserved its writer token;
- all arms had zero exact replies, zero BOS-complete replies and zero premature
  BOS stops;
- fresh-process repeats were byte-identical after timing fields;
- fresh throughput was about **60.4k-60.9k** generated tokens/s at 32 workers.
- reviewed head: `50f22f86962078d8b2a310f07f3c6ee23ff074fc`;
- protected merge: `38ab1357832be444bba248b729e4aa7b46dac7a0`;
- fresh-main CPU library: **295 passed**, 4 expected ignores;
- fresh-main CPU binaries: **82 passed**.

The geometric arm passed only the fixed-depth-1 and writer comparisons on
fresh dev. It failed longest-32, longest-4, `k2>k1` and random-32 on both
artifact seeds. It also failed the train non-inferiority floor. The sealed
outcome is **REJECT**.

## Checks run at the head

- CPU library: **295 passed**, 4 expected ignores.
- All CPU-training binary test targets passed.
- Focused copy-reply tests: **19 passed**.
- Independent sign-Hamming oracle: **1 passed**.
- Scoped Clippy with `-D warnings`: passed.
- D23/D25/D32 ratchets: no growth.
- rustfmt and `git diff --check`: passed.
- Fresh output root and seed 2762: absent before compute.
- Local sealed evidence manifest and all copied evidence: verified.
- Published evidence sums and the no-`.u16` corpus boundary: verified.

## What remains

No delivery work remains for this bounded experiment. The next geometric-copy
unit must learn or derive the candidate ranking from context rather than use
raw writer-row sign proximity. Exact-copy selection, learned termination and
terminal-token reconstruction remain open.

## Size

Against base `7018b09cd`, the merged result changes 43 paths (**+1,337 / -8
textual lines**) plus gzipped evidence blobs. No `.u16` corpus is committed.

## Skills used

No system-provided Codex skill applies. Project delivery and research
procedures were used.

## Still open

Useful local chat remains open. This experiment changes no serving artifact
and makes no complete-reply, language or serving claim.

## Plain summary

Static token-row sign agreement is not a useful arbitration rule for exact-copy
candidates. It changed thousands of decisions, selected candidates with
recorded distances, and remained deterministic, but lost most of the body
accuracy of longest-32. The next attempt needs contextual learned geometry,
not another threshold on this distance.
