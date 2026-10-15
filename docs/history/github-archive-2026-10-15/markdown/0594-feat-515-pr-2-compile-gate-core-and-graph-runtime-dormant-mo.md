# #594 — feat(#515): PR-2 — compile-gate core and graph-runtime dormant modules (default OFF)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-10T20:44:57Z
- **Updated:** 2026-10-09T14:17:25Z
- **Closed:** 2026-08-10T22:55:29Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/594
- **Merged:** not merged
- **Merge commit:** 24001e43510e51df25af4c2f3953e43da8262d83
- **Base / Head:** main / UOR-Foundation:r5/gate-core-runtime-dormant
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## #515 PR-2 — compile-gate core & graph-runtime dormant modules

Fourth tranche of preserve-and-gate. Modules the build proves are off the serving path are now behind **default-off** cargo features:

- **uor-r4-core**: `lie_jordan` (`lie-jordan`) + its operator algebra `endomorphism` (`endomorphism`). Coupled (endomorphism is referenced only by lie_jordan), so `lie-jordan = ["endomorphism"]`.
- **uor-r4-graph-runtime**: `packed_kernels` (`packed-routing`) — the placeholder ROUT-bytecode / typed-transition evaluators (`packed-routing-dormant`).

Default build + test-compile exclude them; `--all-features` compiles all and its lib tests pass (32 core + 13 runtime); clippy clean at `--all-features`.

### Build-test correction — three modules are actually wired
Left **un-gated** (gating breaks the default build — the proof they're on-path). Claims are untouched in this PR; a later pass will refine their wording:

- `bott_fock`, `cd_space` — reachable from shipped `graph-cli` via the `cd-compile` / `quantum-eval` subcommands. Dormant in the *scoring* path (the accurate part of their claims) but compiled into the CLI.
- `patch_chain` — held by `R4G1Runtime` (`crate::patch_chain::PatchChain`), on the serving runtime path.

No ledger/CONFORMANCE change here. Refs #515.


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5246947234 — 2026-08-10T22:55:28Z — Casey-allard

Closing without merge. Build-testing against the FULL workspace test compile (which my earlier gauntlet skipped) shows all three modules this PR gated — lie_jordan, endomorphism, and packed_kernels — are referenced by the root BDD validation suite (tests/bdd.rs): lie_jordan (universal_product_u8, LieJordanSplit + a source-embed test), endomorphism (EndomorphismAlgebra), and packed_kernels (PackedFrontier/PackedShortlist/StepOutput). Gating them default-off breaks the default `cargo test --test bdd` build.

Per the corrected preserve-and-gate rule (only modules with zero default-build reference — code or tests — are gated; modules with live default-suite coverage stay registered-but-compiled, like the CLI-reachable bott_fock/cd_space), none of these three are gatable. Their `open` ledger claims (lie-jordan-dormant, endomorphism-dormant, packed-routing-dormant) remain accurate — dormant in the serving path, with BDD validation harnesses — and are unchanged on main. Nothing to ship here. Refs #515.


---
_Comment/review ids in this file: 5246947234_

