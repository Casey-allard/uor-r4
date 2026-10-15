# #1252 — Connect shared Hamming policy to contextual reads and byte emission

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-13T18:23:41Z
- **Updated:** 2026-10-09T13:17:36Z
- **Closed:** 2026-09-13T18:24:42Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1252
- **Merged:** 2026-09-13T18:24:42Z
- **Merge commit:** b267cfd2d958a76099cc8792e11c777c031895a0
- **Base / Head:** main / UOR-Foundation:codex/hamming-policy-integration
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Connect the Hamming refinement primitive to one shared deterministic LUT4 policy, exact typed session control, byte/EOS emission and observed-byte/result publication. The new frame retains both full selected spans and all four contextual start roots. A strict initialized artifact binds parameters, geometry, source and configuration. Whole-document finite parameter contrasts recompute downstream reads and observations, rather than retaining parent pointers.

This delivers initialized integration, not a learned or promoted model. Eleven focused release tests and a separate sealed report pass. Two fixed parameter interventions change later selected references and emitted predictions; artifact reload and finite-loss reference calculations agree. Actual initialized output is 0/24 complete answers and 1/460 teacher-observed symbols. The older panels are observational baselines only. Following owner clarification, the next learning gate is deliberately one contextual read plus a byte and EOS; dependent reads and accumulated regression qualification follow later.

Validation: `cargo fmt --check`; `cargo test --offline --release -p uor-r4-core --lib hamming_policy -- --test-threads=1 --nocapture`; separately invoked exact report driver; claim wording and source/evidence/preservation checks. Review found and corrected an active-byte bypass in PayloadMasked before the behavioral report. Both builds are charged: total model/build/report 239,769 ms within the recorded 360,000 ms step. Original dirty checkouts, retained model, old source and sealed evidence are preserved. Bounded snapshot allocations, a scalable optimizer, language qualification and normal retained-model dispatch remain unfinished. CI queue acknowledgments do not represent executed tests.

References #973. No issue closure or model promotion.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

