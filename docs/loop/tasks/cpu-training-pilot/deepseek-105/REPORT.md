# REPORT: cpu-training-pilot, deepseek-105

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Sheet:
[cpu-training-pilot-point-neighborhood-2026-11-03.md](../../../harness/prep/cpu-training-pilot-point-neighborhood-2026-11-03.md).
Base: `79843cca8`.

## What did not work

- **The ranking signal is below its three-sigma threshold on both seeds.**
  Eight-row voting beat the single nearest row by 1 and 2 openings of 60,
  against thresholds of 5 and 3. The primary strict-above-controls screen
  passed, but this is not statistically strong opening evidence.
- **The first FIT review was not fit.** The sheet lacked the harness's required
  Base artifact and Steps sections, the extracted-nearest parity check had only
  a runtime guard, and the driver did not emit the pre-registered vote-versus-
  nearest threshold. All three were repaired before compute.
- **The first focused test target hit the repository's known duplicate**
  `uor-r4-integer` artifact collision. A fresh target produced the valid 8/8
  focused result.
- **The first full gate run hit the known fake-`gh` `ETXTBSY` flake** in
  `loop_drive`; 27 of 28 gate tests passed. The immediate rerun passed all 11
  gates. Both attempts are retained.
- **This rule is not served.** It is an offline screen over the same 48
  artifact rows. No integer artifact field, schema, session path or browser
  path implements eighth-nearest voting yet.

## What changed

Extended `cpu-point-transfer-driver` with `--neighbor-vote N`:

- extracts the opening Q12 query from the memory-miss served trunk;
- verifies the extracted single-nearest answer equals the existing served point
  artifact;
- computes exact L1 distances to every point row;
- orders candidates by `(distance, row index)`;
- counts the first eight targets and chooses the commonest target, ties to the
  smaller token id;
- compares the same vote over point and shuffled-target tables; and
- reports vote-minus-nearest against a separately computed three-sigma
  threshold.

The finite tests cover a wrong nearest row overridden by the neighborhood,
smaller-target tie breaking, oversized `k`, deterministic split selection,
non-vacuous shuffled targets, and extracted-nearest versus served parity.

## Measurements

| Seed | Point-off | Nearest | Point k=8 | Random-target k=8 | Vote - nearest / 3σ |
|---|---:|---:|---:|---:|---:|
| 1 | 0 | 2 | **3** | 2 | 1 / 5 |
| 2 | 0 | 1 | **3** | 2 | 2 / 3 |

The primary screen passed on both seeds: the point vote was strictly above
point-off, nearest and the shuffled-target vote. The separate ranking threshold
did not.

The existing served arms retained zero query-coordinate, token and kept-cell
mismatches over 2,110 positions, 540,160 Q12 coordinates and 16,880 kept cells
per seed. The extracted nearest matched the served point answer on 60/60
openings per seed.

Full teacher-forced point arms remained unchanged from deepseek-104:
171/169/169 on seed 1 and 164/163/166 on seed 2 for point/off/random. Those
figures do not evaluate the offline vote.

## Reading

This is a **positive mechanism screen, not a capability result**. Eight-row
target voting recovers one to two additional unseen-key openings over nearest
row lookup and also beats the shuffled vote, on both seeds. The effect is too
small to pass the separate variance threshold and does not establish useful
conversation.

## Checks run at the head

- Focused `cpu-point-transfer-driver` suite: 8 passed.
- Both registered public runs exited 0; one used 32 Rayon threads and about
  1.07 s wall.
- Every registered key overlap and exact-memory-hit check was zero.
- Extracted nearest matched served point naming on 120/120 openings across the
  two seeds.
- Full package suite: 161 library tests passed with 4 release benchmarks
  ignored; existing drivers passed 6/1/5 and the extended point binary passed
  8 focused tests.
- Strict package Clippy with `--all-targets --no-deps -- -D warnings`, package
  formatting, evidence checksums and FIT validation passed.
- `uor-harness loop gates --full`: all 11 gates passed on the immediate rerun
  after the retained `ETXTBSY` flake.
- Report validation and local PR policy are recorded after the documentation
  head is frozen.

The gate attempts are retained in [evidence/loop-gates.txt](evidence/loop-gates.txt).

## Skills used

No available Codex skill applied. The point-key image, served query, CPU FBS
forward and exact-distance operations were inspected and tested directly.

## Still open

- An artifact-bound integer `vote_k` field, serving implementation, session
  integration and exact served audit for this rule.
- A stronger candidate that clears the variance threshold and transfers beyond
  this corpus.
- Complete replies, fresh reply wording, useful local chat and GPU
  qualification.

## Plain summary

On both public model seeds, voting across the eight nearest training point rows
beat the single nearest row and a shuffled-target vote on 60 unseen-key
openings. The gain was only one to two openings, below the registered
three-sigma signal. The candidate is worth a separate serving-schema unit, but
it is not served yet and is not language capability.
