# #593 — feat(#515): PR-2 — compile-gate graph-compiler dormant surfaces; correct 2 wired

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-10T20:35:49Z
- **Updated:** 2026-10-09T14:17:25Z
- **Closed:** 2026-08-10T23:12:54Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/593
- **Merged:** 2026-08-10T23:12:54Z
- **Merge commit:** 78f5b724016d321d45c2197b17a90bb50296a18d
- **Base / Head:** main / UOR-Foundation:r5/gate-compiler-dormant
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## #515 PR-2 — compile-gate graph-compiler dormant surfaces

Third tranche of preserve-and-gate. The eight genuinely off-serving-path
`graph-compiler` modules are now behind **default-off** cargo features, one per
`open` ledger claim:

`behavioral_probes`, `graph` (build_graph construction), `monograph`,
`patch_induction`, `perturbation`, `quantum_cover`,
`rate_distortion_compression`, `reference_compiler_ir`.

Default workspace build compiles and links without them; `--all-features`
compiles all eight; clippy clean both ways; nothing on the default path (incl.
tests) references them.

### Build-test correction — two claims from #590 were wrong
Feature-gating these breaks the **default** build, which is the build-level proof
they are on the serving path. They are left un-gated and their `open` claims are
removed (CONFORMANCE.md regenerated, 29 ids, model==doc):

- `future_state_planner` — `use crate::future_state_planner::PlanTrajectory` in `semantic_emission_decoupling.rs`.
- `stage_dag` — `verify_compiler_stage_dag_compliance` (shipped `pub fn`) in `uor-r4-proof-model/structural_guarantees.rs`.

audit-limits: clean. Refs #515.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

