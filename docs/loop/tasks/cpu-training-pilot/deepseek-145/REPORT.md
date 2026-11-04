# REPORT: cpu-training-pilot, deepseek-145

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `d5d173073`.

## Status

The accounting repair is complete. The corrected source reproduced the
registered seed-1 preflight byte-for-byte and stopped before generation. No
capability or serving claim is made.

## What did not work

- **The first trajectory implementation counted two different populations.**
  Its fallback predicate compared reference-aligned `decisions` with readout
  plus fallback counters over every generated decision. Existing sealed
  generation evidence shows why that fails: 973 reference-aligned decisions
  against `103 + 1,545 = 1,648` generated decisions.
- **The first unit's “exact fallback accounting” claim was premature.** Its
  synthetic unit test used matching handcrafted totals and did not exercise a
  capped trajectory beyond a shorter reference.
- **The mechanism result remains negative.** Corrected-source code top-1 was
  300/894 and trajectory was 297/894, so seed 2, generation and the repeat
  remain `NOT_RUN`.

## What changed

`ContentReadoutReading` now records `generated_decisions` independently of the
reference-aligned `decisions`. Full-generation fallback accounting compares
readout plus exact-writer fallback against that total. A regression constructs
a four-decision capped reply with a two-token reference horizon and verifies
the distinct populations. A second test verifies the driver predicate rejects
an unaccounted decision.

The trajectory construction, split, data, threshold rule, control and stop rule
are unchanged.

## Measurements

| Seed 1 reading | Code | Trajectory | Limit |
|---|---:|---:|---:|
| Training rows | 894 | 894 | at least 640 |
| Target values | 207 | 207 | at least 16 |
| Covered rows | 807 | 806 | at least 90% |
| Leave-one-out top-1 | **300** | **297** | trajectory >= code |
| Ambiguous rows | 277 | **254** | reported |
| Maximum distance | 537 | 1,080 | reported |
| Signature bits | 2,048 | 4,096 | trajectory 4,096 |

The corrected and original preflight reports have the same SHA-256,
`06311414…693bdc`; their structured failure files also match,
`17ac1246…b8879`. The target-mapping hashes remain `4f191527…7ffd046b` and
`a9b507a7…d32e23e`, with 632/633 effective assignments changed.

Build and stop resources:

- release build: **19.94 s**, **269,000,392 bytes**, **1,195,272 KiB** peak;
- controls: **23.35 s** library, **5.54 s** driver and **11.05 s** Clippy;
- 256 library tests passed with 4 expected ignores; 10 driver tests passed;
- preflight wall: **0.67 s** at **226,912 KiB** peak RSS;
- binary SHA-256:
  `62bed7d83cc1422cc5fe5c29895e6599567f4f66bd2f69723a43a84b4df62657`;
- measured repair compute: **60.55 s**; measured cumulative:
  **35,945.808816343 s**; admitted cumulative:
  **62,827.677487699 s**;
- the named model-time ledger remains `ABSENT` and is not treated as zero.

Attempt 1 is unchanged at its original seal. Attempt 2 is under
[`evidence-attempt-2/`](../../../../labs/cpu-training-pilot-trajectory-readout-2026-11-04/evidence-attempt-2/);
its `SHA256SUMS` is
`140cf3f54cf60931d149bd89359444e2caf67629f0e2485dcc4aa4ee5e288ee8`.
Accounting is in the
[repair ledger](../../../../labs/cpu-training-pilot-trajectory-readout-2026-11-04/resource-ledger-repair-145.json).

## Classification

This repair is an instrument positive and leaves the measured result as the
same bounded configuration negative for the 4,096-bit
`current code || first-order XOR` content representation. The transition
carried information but did not preserve exact content top-1, so the
registered preflight correctly stopped.

The result does not reject higher-order transition history, learned transition
geometry or a different content objective.

## Checks run at the head

- Focused capped-generation and fallback-accounting regressions passed.
- Release build, 256 library tests, 10 driver tests, scoped Clippy and rustfmt
  passed.
- Corrected-source preflight reproduced the registered reading and stop.
- Attempt 1 and attempt 2 evidence manifests verify independently.
- Source and all five public inputs match.
- Claim wording, docs catalog, roadmap schema/rendering, formats,
  `uor-harness check`, local PR checks and D23/D25/D32 are recorded at
  delivery.
- No integer-session, serving, CUDA, float-training or Claude-owned source
  changed.

## Skills used

No system-provided Codex skill applies.

## Still open

- A trajectory representation that preserves or improves exact content top-1.
- Full trajectory generation through the corrected accounting predicate.
- Complete replies, fresh phrasing and useful local chat.

## Plain summary

The review caught a full-generation accounting bug that would have rejected a
correct trajectory. The repair is pinned by a real capped-horizon regression,
and the corrected source reproduces the original negative result exactly.
