# REPORT: cpu-training-pilot, deepseek-146

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `cf0c16d58`.

## Status

Attempt 1 is complete at the registered preflight stop. No held-out generation
ran and no capability claim is made.

## What did not work

- **The phase-transition gate learned too few endpoints.** Seed 1 recalled
  **14/84** generated terminal rows against the registered **26/84** floor.
- **The shuffled control recalled more endpoints.** The phase gate emitted
  **7/1,484** false stops and its shuffled-label control recalled **16/84**
  with **11/1,484**. The control changed **48** leave-one-out firing decisions,
  so the failure was not a vacuous control.
- **The registered preflight stopped before generation.** Seed 2, writer/code/
  phase complete-reply arms and the one-worker repeat did not run. There is no
  exact-reply, BOS-completion, top-1 or depth result for these arms.
- **This is not a language or serving result.** Integer serving, `UORLUT01`,
  CUDA, float training and Claude-owned paths were unchanged.

## What changed

Generated termination rows now have an optional 2,048-bit phase transition
formed from `rotate_left64(current_code, popcount(previous_code) mod 64) XOR
previous_code`. The current code remains the content-readout key. Offset zero
is all zero and history resets per sample.

The existing full-row density rule uses the same `{1,2,4,8}` neighbour set and
offset radius two. A preflight control compares true and shuffled leave-one-out
firing decisions. A separate audit compares every phase-arm decision before the
first gate firing with an independently generated code arm.

## Measurements

| Seed 1 reading | True phase | Shuffled control | Limit |
|---|---:|---:|---:|
| Generated rows | 1,568 | 1,568 | exact |
| Terminal rows | 84 | 84 | at least 84 |
| Nonterminal rows | 1,484 | 1,484 | at least 1,484 |
| Terminal recall | **14/84** | 16/84 | at least 26/84 |
| Nonterminal false stops | **7/1,484** | 11/1,484 | at most 14/1,484 |
| Selected neighbours | 2 | 2 | at most 8 |
| Offset radius | 2 | 2 | exact |
| Margin | 938 | 39 | reported |
| Leave-one-out prediction changes | 48 |  | greater than zero |

Build and stop resources:

- release build: **20.66 s**, **269,494,925 bytes**, **1,192,288 KiB** peak;
- controls: **24.25 s** library, **5.53 s** driver and **11.07 s** Clippy;
- 258 library tests passed with 4 expected ignores; 10 driver tests passed;
- preflight wall: **0.73 s** at **236,912 KiB** peak RSS;
- binary SHA-256:
  `3017b738d3ab6e9b3d4c5a0767f94697fc7b9105871d6a828333929b5230ddfb`;
- measured attempt compute: **62.24 s**; measured cumulative:
  **36,008.048816343 s**; admitted cumulative:
  **66,427.677487699 s**;
- the named model-time ledger remains `ABSENT` and is not treated as zero.

Exact stopped output is under
[`evidence-attempt-1/`](../../../../labs/cpu-training-pilot-phase-termination-2026-11-04/evidence-attempt-1/).
Evidence `SHA256SUMS` is
`5b5af4570dd10bc1935bd52e70da80611b3de9a18674b05d9f9a99a123a97859`.
Measured accounting is in the
[resource ledger](../../../../labs/cpu-training-pilot-phase-termination-2026-11-04/resource-ledger.json).

## Classification

This is a **bounded configuration negative** for the fixed phase-aligned
first-order transition with the existing dense-neighbour termination rule. The
transition was non-vacuous and false stops remained inside the contract, but
terminal recall and control specificity were both weak.

The result does not reject all transition geometry, a nonlinear phase map, a
different terminal prototype construction or a nonterminal-aware objective.

## Checks run at the head

- Fresh FIT and GO preceded the registered build and preflight.
- Release build, **258** library tests, **10** driver tests, scoped Clippy and
  rustfmt passed.
- Source and all five public inputs matched.
- Phase packing, offset-zero history, generated phase history and non-vacuous
  label prediction changes passed. The full-generation code-prefix audit is
  `NOT_RUN` because the preflight stopped before generation.
- The preflight wrote structured failure evidence and sealed all outputs.
- Claim wording, docs catalog, roadmap schema/rendering, formats,
  `uor-harness check` and `git diff --check` are recorded at delivery.
- No integer-session, serving, CUDA, float-training or Claude-owned source
  changed.

## Skills used

No system-provided Codex skill applies.

## Still open

- A terminal geometry that separates endpoints from ordinary continuation
  under both recall and false-stop limits.
- Complete replies, fresh phrasing and useful local chat.

## Plain summary

The phase transition changed enough decisions to be real, but it did not learn
enough endpoints. The gate stopped with 14 of 84 terminal rows recalled, so no
generation was spent on a configuration already below its recall floor.
