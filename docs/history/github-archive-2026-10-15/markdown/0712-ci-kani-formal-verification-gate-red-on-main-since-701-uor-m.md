# #712 — CI: Kani formal-verification gate red on main since #701 (uor-matmul-kernels global ASM)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-14T15:55:55Z
- **Updated:** 2026-08-14T21:40:34Z
- **Closed:** 2026-08-14T21:39:49Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/712
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Symptom
The **Formal Verification (Kani)** workflow (`cargo kani -p uor-r4-proof-model`) has failed on every push/PR since **#701 (#655-B1)**:

\`\`\`
error: Crate uor_matmul_kernels contains global ASM, which is not supported by Kani.
Rerun with \`-Z unstable-options --ignore-global-asm\` to suppress this error (**Verification results may be impacted**).
error: could not compile \`uor-matmul-kernels\` (lib) due to 1 previous error
\`\`\`

`uor-r4-proof-model` → `uor-r4-core` → `uor-r4-model-source` → `uor-matmul-*` (production dep since B1), and `uor-matmul-kernels` contains `global_asm!`, which Kani's codegen rejects.

## History (Formal Verification workflow)
- #699 (matrix census) and earlier: **success**.
- #701 (uor-matmul prod dep) onward — #703, #706, #708, #710: **failure**. Red on `main` HEAD.

So the deployed formal-verification invariant is currently **not being verified** — the gate compiles nothing. It is non-blocking (register/gates are the required checks), which is why PRs kept merging, but Kani coverage has silently lapsed.

## Options
1. Add `-Z unstable-options --ignore-global-asm` to the `cargo kani` invocation. Caveat from Kani: "Verification results may be impacted" — the ASM body is skipped, so any proof whose reachability crosses `uor-matmul-kernels` ASM is unsound. Acceptable only if no proof harness in `uor-r4-proof-model` actually exercises matmul kernels.
2. Ensure `uor-matmul-kernels`' `global_asm!` is behind a feature/`cfg` that the Kani build turns **off** (e.g. a scalar-fallback path), so the proof crate compiles a pure-Rust kernel under `cfg(kani)`.
3. Narrow the Kani build so `uor-r4-proof-model` does not pull `uor-matmul-kernels` into the proof graph (feature-gate the matmul dep out under `cfg(kani)`).

Recommend (2) or (3) to keep proofs sound; (1) only if the harnesses provably never touch the kernels. Filing per discovered-problem discipline; not blocking the #639 chain.

## Comments (1)

### comment 5298437285 — 2026-08-14T21:40:34Z — Casey-allard

Delivered by PR #717 (`4b1513ebd206b2a9c5f5ce1c844f7abc6bf415c2`).

Verification evidence:

- PR-side Formal Verification passed in 1m21s: `cargo kani -p uor-r4-proof-model --no-default-features --lib` proved both harnesses, including the corrected non-vacuous `ScoreQ::saturating_add` assertion.
- The minimal dependency-tree guard confirms the Kani build excludes `uor-r4-core`, `uor-r4-model-source`, `uor-r4-graph-compiler`, `uor-r4-graph-certify`, and `uor-matmul`.
- Default proof-model behavior remains enabled through `full-model`; 25 default-feature tests passed locally, while the no-default-feature library build/test passed without integration-test leakage.
- The merge-queue speculative commit passed the full required ladder: fmt/clippy/tests/no_std/κ, all-features tests, wasm, fuzz, Gate C trend, audit, and register conformance.

Queue run: https://github.com/UOR-Foundation/uor-r4/actions/runs/31842144053
Kani run: https://github.com/UOR-Foundation/uor-r4/actions/runs/31841876149/job/94900307700



---
_Comment/review ids in this file: 5298437285_

