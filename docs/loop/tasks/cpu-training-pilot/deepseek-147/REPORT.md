# REPORT: cpu-training-pilot, deepseek-147

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `6c21d3133`.

## Status

Attempt 1 is complete at the registered preflight stop. Seed 2, held-out
generation and the one-worker repeat did not run. No capability claim is made.

## What did not work

- **The combined trajectory gate learned too few endpoints.** Seed 1 recalled
  **16/84** generated terminal rows against the registered **26/84** floor.
- **The shuffled control recalled more endpoints.** The combined gate emitted
  **11/1,484** false stops and its shuffled-label control recalled **18/84**
  with **13/1,484**. The control changed **58** leave-one-out firing decisions,
  so the failure was not a vacuous label permutation.
- **Adding the current code did not recover enough of the separate signals.**
  Current-code density previously recalled 19/84 and phase-only density 14/84.
  Their fixed 4,096-bit concatenation recalled 16/84 without any retuning.
- **The registered preflight stopped before generation.** Code, combined-density
  and shuffled-density complete replies and the one-worker repeat did not run.
  There is no exact-reply, BOS-completion, top-1 or depth result for this unit.
- **This is not a language or serving result.** Integer serving, `UORLUT01`,
  CUDA, float training and Claude-owned paths were unchanged.

## What changed

Generated termination rows now have an optional 4,096-bit combined state:

```text
current modulo-256 code || phase-aligned first-order transition
```

The transition is
`rotate_left64(current_code, popcount(previous_code) mod 64) XOR previous_code`.
Offset zero is all zero. Content selection continues to use only the current
code. The density fitter now accepts both the original 2,048-bit states and
the combined 4,096-bit state while rejecting mixed row widths.

The preflight's select-only report now includes an independently generated
code-prefix audit over the training samples. The audit checks every decision
before the first combined-gate firing against the code arm.

## Measurements

| Seed 1 reading | True combined | Shuffled control | Limit |
|---|---:|---:|---:|
| Signature bits | 4,096 | 4,096 | exact |
| Generated rows | 1,568 | 1,568 | exact |
| Terminal rows | 84 | 84 | at least 84 |
| Nonterminal rows | 1,484 | 1,484 | at least 1,484 |
| Terminal recall | **16/84** | 18/84 | at least 26/84 |
| Nonterminal false stops | **11/1,484** | 13/1,484 | at most 14/1,484 |
| Selected neighbours | 2 | 8 | at most 8 |
| Offset radius | 2 | 2 | exact |
| Margin | 775 | 1,805 | reported |
| Leave-one-out prediction changes | 58 |  | greater than zero |
| Ungated code-prefix audit | **0/1,145 mismatches** |  | zero |

Gate SHA-256 values:

- true: `844a30aec55cbf1fe0660e021e1fe4b936c56009b8ba1a9ac2d47be8d9f46871`;
- shuffled: `01320a61592b15731558f6d0afae8c4596ace62f49502a0ae9dd63a79f9dd6bb`.

Build and stop resources:

- release build: **19.96 s**, **269,722,017 bytes**, **1,198,408 KiB** peak;
- controls: **23.22 s** library, **5.55 s** driver and **11.06 s** Clippy;
- **261** library tests passed with 4 expected ignores; **10** driver tests
  passed; scoped Clippy and rustfmt passed;
- preflight wall: **0.95 s** at **242,824 KiB** peak RSS;
- driver binary: **3,295,552 bytes**, SHA-256
  `37add1af2846dec49289e710436ca2b4e00a9c29718d1be2a98c4193e00b02ac`;
- measured attempt compute: **60.74 s**; measured cumulative:
  **36,068.788816343 s**; admitted cumulative:
  **69,727.677487699 s**;
- the named model-time ledger remains `ABSENT` and is not treated as zero.

Exact stopped output is under
[`evidence-attempt-1/`](../../../../labs/cpu-training-pilot-trajectory-density-2026-11-04/evidence-attempt-1/).
Evidence `SHA256SUMS` is
`f56a5dc8be880189fb71776cac9f9979c6a7a0a6daa4a1c249230bcfa6c90467`.
Measured accounting is in the
[resource ledger](../../../../labs/cpu-training-pilot-trajectory-density-2026-11-04/resource-ledger.json).

## Classification

This is a **bounded configuration negative** for the fixed
current-code-plus-phase-transition state under the unchanged density rule.
The combined state was non-vacuous, preserved content before firing and kept
false stops inside the limit, but terminal recall remained far below the
26/84 floor.

The result closes this exact concatenation and fixed density objective. It does
not reject all trajectory geometry, learned transition metrics, terminal
prototypes, nonterminal-aware objectives or vector-space attention.

## Checks run at the head

- Fresh non-author FIT and GO preceded the registered build and preflight.
- Release build, **261** library tests, **10** driver tests, scoped Clippy and
  rustfmt passed.
- Source and all five public inputs matched their pinned identities.
- Combined packing, offset-zero history, previous-state generation, 4,096-bit
  fitting, width rejection, label control and code-prefix audit tests passed.
- The seed-1 preflight stopped at the registered recall floor and sealed all
  output and timing files.
- Claim wording, documentation links, formats, `uor-harness check`,
  `git diff --check` and the evidence manifest are recorded at delivery.
- No integer-session, serving, CUDA, float-training or Claude-owned source
  changed.

## Skills used

No system-provided Codex skill applies.

## Still open

- A terminal geometry that separates endpoints from ordinary continuation
  under both recall and false-stop limits.
- Complete replies, fresh phrasing and useful local chat.

## Plain summary

The 4,096-bit geometric state changed enough decisions to be real and did not
disturb content before stopping. It still learned only 16 of 84 endpoints, so
the preflight correctly stopped before spending time on generation.
