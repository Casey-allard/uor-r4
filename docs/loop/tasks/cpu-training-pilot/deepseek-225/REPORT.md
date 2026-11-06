# REPORT: cpu-training-pilot, deepseek-225

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Prep:
[aligned endpoint sheet](../../../../harness/prep/cpu-training-pilot-content-prefix-endpoint-2026-11-06.md).
Base: `07349193e`.

## Status

Complete as **REJECT**. Both artifacts and fresh-process repeats ran.

## What did not work

- The aligned retrieved endpoint did not produce complete replies. On either
  artifact only **206/229** of 471 reachable replies reached an endpoint or
  BOS, while **67/85** stopped prematurely.
- The content and quality counts did not improve over the unchanged boundary:
  both made **17** exact replies and retained **644/642** body positions.
- The aligned true-retrieval guard did not beat the shuffled-retrieval guard
  on exact replies. Both made 17 exact replies, so the paired comparison failed.
- The unconditional stored length still emitted all 471 replies without
  premature BOS, but its body count fell to **497/495** from the boundary
  **644/642**. This reproduces the deepseek-223 content-loss finding.

## What changed

Added a reference-independent aligned retrieved endpoint to suffix
arbitration. The generator continues along deepseek-198's unchanged content
and boundary path, suppresses a boundary stop only while the complete generated
reply is a prefix of the retrieved TRAIN reply, and stops when that reply is
reached. On first divergence, the original boundary path resumes.

The driver now distinguishes stop-only changes from token-decision changes.
This repair was required after the first TRAIN-only preflight falsely reported
`CONTROL_FAULT`; both attempts are retained in the evidence root.

## Measurements

Seed 2784 contained 128 conversations, 16,768 tokens, 5,370 response tokens
and **471** reachable replies.

| Arm | Exact | Body positions | Premature BOS | Endpoint stops |
|---|---:|---:|---:|---:|
| aligned true endpoint | 17 | 644 / 642 | 67 / 85 | 23 / 23 |
| original boundary | 17 | 644 / 642 | 73 / 91 | 0 / 0 |
| shuffled-retrieval guard | 17 | 644 / 642 | 73 / 91 | 1 / 1 |
| unconditional stored length | 11 | 497 / 495 | 0 / 0 | 0 / 0 |
| local longest + aligned endpoint | 0 | 481 / 481 | 7 / 7 | 0 / 0 |
| full retrieval | 50 | 955 / 955 | 0 / 0 | 0 / 0 |
| reference endpoint oracle | 27 | 644 / 642 | 73 / 91 | 27 / 27 |

The guard changed **23/23** complete outcomes from boundary and **24/24** from
the shuffled guard, without changing a content token. It reduced premature BOS
by only six on each artifact. Candidate deeper-global decisions were
**2,118/2,275**.

Panel pins:

- tokens:
  `510e11010ecdeb4bdf2ea70d76c6d2e1d6da5c34fa1b2705b84180664a4e82d8`;
- mask:
  `ed16c32bacf5fbcdb9ad556dd75cf5ad735fd75fc1092f2a29c187403c8e540a`;
- manifest:
  `03a01091a2907ec7dd3252964894c6ebae83797fa691d1f9ece3f9027fa0a68c`.

Report hashes:

- seed 1:
  `2134cd86cf8f23887aa6ef00c95ad2486e9b607a703911279d205ab1ce2b8832`;
- seed 2:
  `6a267ef1b4f91f59729118a4bc6dd0a5703fe3e5a93a7a71ab46c46c2dce1a20`.

Each repeat is byte-identical to its first report. Raw `SHA256SUMS` was
`eacbdbfc0f82f72d1f79574d9aef7b8b113cd32eebf8edc16718c6e6ef525340`.
Retained evidence was **174,195 bytes**.

Resources:

- four scored processes completed in **7.99-8.91 s** at **1,286-1,447% CPU**;
- peak scored-process RSS was **280,420 KiB**;
- panel generation took **0.71 s** and peaked at **7,596 KiB**;
- the release driver build took **22.32 s**.

Compact evidence: [result.json](evidence/result.json).

## Interpretation

The guard is active but does not solve termination. It changes a small number
of boundary stops to retrieved endpoints and leaves exact/body quality
identical to boundary. Most replies either diverge before the retrieved
endpoint or never reach BOS, so a prompt-level retrieved reply cannot be used
as a reliable stopping state for a generated trajectory.

This closes the fixed full-prefix alignment strategy as a termination repair.
It preserves the diagnosis that stop training must use the generated state's
own branch evidence; a retrieved full reply is too brittle after divergence.

## Checks run at the head

- focused aligned-endpoint tests: 2 passed;
- driver usage test: 1 passed;
- full `uor-r4-cpu-train` library: 406 passed, 4 expected ignores;
- scoped Clippy passed with the pre-existing `type_complexity` lint exempted;
- changed implementation files pass rustfmt and `git diff --check`;
- TRAIN-only preflight attempt 2 passed the activity audit and reached
  `REJECT`, not `CONTROL_FAULT`;
- source, driver, retrieval and release binary hashes match FIT/GO;
- both artifacts have 471 reachable rows and deterministic repeats;
- retrieval-reply shuffle changed 450 panel associations;
- suffix shuffle changed 75,093 targets with zero BOS transitions.
- D23, D25, D32 and claim-wording ratchets passed;
- prep-check reports the aligned-endpoint sheet FIT and complete;
- `loop report --check` and `loop pr-check --local` passed;
- `loop gates --full`: 11/11 green;
- `xtask docs-catalog --check`: 1,175 records, 20 lines, zero missing paths;
- documentation links report zero broken targets added by deepseek-225;
- full harness check: 287 rules, zero errors, 252 repository-wide warnings;
- `xtask roadmap check` remains blocked by the pre-existing
  `infra-desktop-client-tauri` status/issue mismatch, which this diff does not
  touch;
- implementation review was a local exact-head self-review because independent
  delegation was unavailable in this session.

## Skills used

No system-provided Codex skill applied.

## Still open

- generated-state branch or endpoint evidence that survives content divergence;
- a stop signal trained on generated trajectories rather than teacher-forced
  suffixes;
- Claude's independent end-to-end serving track.

## Plain summary

The aligned retrieved endpoint is active but insufficient. It converts 23
boundary stops per artifact into retrieved endpoints while preserving the
boundary arm's exact and body counts, yet most replies still stop prematurely
or fail to terminate. The fixed full-prefix endpoint guard is rejected.
