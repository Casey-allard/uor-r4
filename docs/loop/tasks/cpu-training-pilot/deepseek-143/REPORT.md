# REPORT: cpu-training-pilot, deepseek-143

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `7dfd96644`.

## Status

Attempt 1 is complete at the registered preflight stop. No held-out generation
ran and no capability claim is made.

## What did not work

- **False stops missed the one-percent ceiling.** The true horizon predictor
  stopped **66/1,484** nonterminal rows against the registered **14**-row
  limit.
- **Terminal recall passed but was insufficient alone.** It recalled
  **40/84** endpoints, above the **26/84** floor, while its permuted control
  recalled 33.
- **The registered run stopped before held-out generation.** Because the
  preflight is a hard stop, seed 2, the no-gate/true/permuted generation arms
  and the one-worker repeat did not run. There is no exact-reply,
  BOS-completion, top-1 or depth result for this representation.
- **This is not a serving or language result.** The integer artifact,
  `UORLUT01`, the content-readout sidecar and Claude-owned paths were
  unchanged.

## What changed

The CPU content driver now has a separate `--stop-horizon-neighbors` arm. It
derives signed remaining-reference horizons for every generated code-signature
state, predicts with the lower median of exact-offset Hamming neighbours, fits
k by terminal recall/false stops/horizon error, and emits BOS only for a
predicted zero. A within-offset horizon rotation is the matched control.
Terminal-row and prediction-change controls use explicit leave-one-out
predictions over aligned original and permuted rows, so the query cannot match
itself.

The arm is isolated from the earlier prototype and density gates and does not
change serving.

## Measurements

| Seed 1 reading | True horizon | Permuted horizon | Limit |
|---|---:|---:|---:|
| Total rows | 1,568 | 1,568 | exact |
| Zero / nonzero horizon rows | 84 / 1,484 | 84 / 1,484 | exact |
| Terminal recall | **40/84** | 33/84 | at least 26/84 |
| Nonterminal false stops | **66/1,484** | 62/1,484 | at most 14/1,484 |
| Horizon absolute error | **7,184** | 9,525 | true < shuffled |
| Selected k | 2 | 1 | in `{1,2,4,8}` |
| Changed raw predictions | 806 | 806 | greater than zero |

The true and permuted gate hashes were `57cd8935…d6d5a5` and
`b3c2a42a…df7d557`. Candidate k=1 had the lowest horizon error at 6,979 and
recalled 36 terminal rows, but also produced 60 false stops. Candidate k=2
maximized terminal recall at 40 and produced 66 false stops; k=4 and k=8
produced 34/75 and 29/73 terminal/false counts. Selection followed the
registered ordering rather than optimizing only error.

Build and stop resources:

- release build: **33.02 s**, **268,973,135 bytes**, **1,164,624 KiB** peak;
- controls: **25.63 s** library, **6.01 s** driver and **11.91 s** Clippy;
- 254 library tests passed with 4 expected ignores; 8 driver tests passed;
- preflight wall: **0.60 s** at **236,760 KiB** peak RSS;
- binary SHA-256:
  `d97e7728398bb41eeb28d3f8d9222e487f998c9eac07fdec4d733c7228d9745e`;
- measured attempt compute: **77.17 s**; measured cumulative:
  **35,821.238816343 s**; admitted cumulative:
  **58,627.677487699 s**;
- the named model-time ledger remains `ABSENT` and is not treated as zero.

## Classification

This is a **bounded mechanism negative** for the exact-offset, lower-median,
leave-one-out continuation-horizon representation at the registered
one-percent false-stop limit. The horizon objective carries information: true
error was below the matched permutation and terminal recall passed. That
signal was not specific enough to separate endpoint states from ordinary
continuation under the required false-stop ceiling.

The stop rule correctly prevented a generation run whose expected false-stop
rate was already far outside contract. This does not reject a different
continuation parameterization, a horizon-distance threshold learned against
nonterminal density, or a trajectory representation that changes the state
before classification.

Exact stopped output is under
[`evidence-attempt-1/`](../../../../labs/cpu-training-pilot-continuation-horizon-2026-11-04/evidence-attempt-1/).
Evidence `SHA256SUMS` is
`162c781f6cf3abc206818551a36d0af47e11fe7152e34a7116a5102f698f5bfe`.
Measured accounting is in the
[resource ledger](../../../../labs/cpu-training-pilot-continuation-horizon-2026-11-04/resource-ledger.json).

## Checks run at the head

- Fresh FIT and GO preceded the registered build and preflight.
- Release build, **254** library tests, **8** driver tests, scoped Clippy and
  targeted rustfmt passed.
- Source identity and all five public inputs matched.
- Exact row accounting, raw LOO prediction changes and self-exclusion controls
  passed.
- The preflight wrote structured failure evidence and sealed all outputs.
- Claim wording, docs catalog, roadmap schema/rendering, prep checks,
  `uor-harness check` and `git diff --check` are recorded at delivery.
- No integer-session, serving, CUDA, float-training or Claude-owned source
  changed.

## Skills used

No system-provided Codex skill applies.

## Still open

- A continuation mechanism that separates endpoints from ordinary continuation
  under a false-stop ceiling.
- Complete replies, fresh phrasing and useful local chat.

## Plain summary

The horizon predictor found more endpoints than the earlier shuffled density
gate, but it also stopped ordinary continuation far too often. The registered
preflight stopped before generation, so this is a bounded negative rather than
a language result.
