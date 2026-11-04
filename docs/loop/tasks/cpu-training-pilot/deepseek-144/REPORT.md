# REPORT: cpu-training-pilot, deepseek-144

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `d5d173073`.

## Status

Attempt 1 is complete at the registered preflight stop. No held-out generation
ran and no capability claim is made.

## What did not work

- **Trajectory top-1 fell below the existing code readout.** Code reached
  **300/894** leave-one-out rows, while trajectory reached **297/894**.
- **Its lower ambiguity was not a capability gain.** Trajectory ambiguity fell
  from 277 to 254 rows, but it did not preserve the exact top-1 floor.
- **The registered run stopped before generation.** Seed 2, all public
  generation arms and the one-worker repeat did not run.
- **This is not a serving or language result.** No integer serving,
  `UORLUT01`, CUDA, float-training or Claude-owned source changed.

## What changed

The content crate now exposes a 4,096-bit trajectory mode:
`current code || (current code XOR previous current code)`, with a zero
transition at offset zero. The existing nearest-example fitter and threshold
fallback operate over it, and the driver exposes `--trajectory-readout`,
true/shuffled mappings, generation arms and a separate bar.

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

The trajectory true/shuffled mapping hashes were `4f191527…7ffd046b` and
`a9b507a7…d32e23e`; 632 of 633 effective assignments changed.

Build and stop resources:

- release build: **20.64 s**, **268,980,357 bytes**, **1,194,684 KiB** peak;
- controls: **24.79 s** library, **5.98 s** driver and **11.97 s** Clippy;
- 255 library tests passed with 4 expected ignores; 10 driver tests passed;
- preflight wall: **0.64 s** at **226,616 KiB** peak RSS;
- binary SHA-256:
  `c45a4c7c688a0ede4845d17b9edc88a1e58fbd8c42a5cc94f82e19aa8b5c8ec7`;
- measured attempt compute: **64.02 s**; measured cumulative:
  **35,885.258816343 s**; admitted cumulative:
  **61,927.677487699 s**;
- the named model-time ledger remains `ABSENT` and is not treated as zero.

## Classification

This is a **bounded mechanism negative** for the 4,096-bit
`current code || first-order XOR` content representation. The transition
carried information: ambiguity fell by 23 rows, coverage stayed within one
row of code and the shuffled mapping changed. It nevertheless lost three
exact top-1 rows, so it is not a replacement for the positive 2,048-bit code
under the registered preflight contract.

The result does not reject higher-order transition history, learned transition
geometry or a different content objective. The stop rule correctly prevented
generation from a representation that had already fallen below its fixed
content baseline.

Exact stopped output is under
[`evidence-attempt-1/`](../../../../labs/cpu-training-pilot-trajectory-readout-2026-11-04/evidence-attempt-1/).
Evidence `SHA256SUMS` is
`1294285ad6174e6f84b69a4197058175ff3a204a84f9d91021cf492ec7c21c21`.
Measured accounting is in the
[resource ledger](../../../../labs/cpu-training-pilot-trajectory-readout-2026-11-04/resource-ledger.json).

## Checks run at the head

- Fresh FIT and GO preceded the registered build and preflight.
- Release build, **255** library tests, **10** driver tests, scoped Clippy and
  targeted rustfmt passed.
- Source identity and all five public inputs matched.
- Trajectory packing/history, target shuffle, threshold pairing and exact
  readout/fallback accounting passed.
- The preflight wrote structured failure evidence and sealed all outputs.
- Claim wording, docs catalog, roadmap schema/rendering, prep checks,
  `uor-harness check` and `git diff --check` are recorded at delivery.
- No integer-session, serving, CUDA, float-training or Claude-owned source
  changed.

## Skills used

No system-provided Codex skill applies.

## Still open

- A trajectory representation that preserves or improves exact content top-1.
- Complete replies, fresh phrasing and useful local chat.

## Plain summary

The first-order trajectory representation reduced ambiguity but lost three
top-1 rows, so the registered preflight stopped before generation. This closes
that construction, not trajectory modeling generally.
