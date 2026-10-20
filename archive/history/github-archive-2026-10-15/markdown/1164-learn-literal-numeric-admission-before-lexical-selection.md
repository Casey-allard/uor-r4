# #1164 — Learn literal numeric admission before lexical selection

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-07T03:55:01Z
- **Updated:** 2026-10-09T13:17:36Z
- **Closed:** 2026-09-07T03:56:20Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1164
- **Merged:** 2026-09-07T03:56:20Z
- **Merge commit:** 8ffd04e5dcd1ac19bf6f6dcfd46229a4d7349124
- **Base / Head:** main / UOR-Foundation:codex/joint-typed-admission
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Literal numeric proposals could execute before lexical selection, causing supported locations and Rust identifier completions to emit numbers. Add a learned Numeric/DeferToLexical decision before literal execution, reusing the existing two-lane signed-H4 router and Rust learner. The complete d590 parent, numeric operand ranking, computed roles, source/NoRead router and emission remain fixed. Parent reconstruction, optional serialization, exact pre-admission overflow support and work counters preserve the existing runtime contracts.

Retain artifact `433e3807` at bounded construction-repair scope: 558/615 → 563/615 complete responses, five gains and no lost correct or changed remaining wrong output. Disabling admission restores all 615 parent texts/stops. Open 4/6 and fresh 8/12 are unchanged; matched equality ties angular. Fresh location cases do not reach the gate, so lexical-rejection transfer is unestablished. All prior output/write/session preservation passes, including the prior 20/20 source set. Documentation points next at order-sensitive entity-to-operand binding and preserves the separate source-abstention negatives.

Validation: offline release example/CLI and final core test targets compile; actual artifact parent/mutation, causal commitment, checkpoint and zero-allocation checks pass; 49 boundary arithmetic pairs pass; integer-kernel source scan passes; three exact generated Rust functions pass 15 assertions. Formatting, architecture-policy and claim-wording checks pass. Eighteen warm predict/observe samples measure 0.133 ms median, 0.182 ms maximum, excluding loading/encoding/ingestion/checkpoints; no end-to-end, energy or matched-speedup claim. Full release/Clippy/WASM and new public-CLI generation were not run. Required CI names remain compatibility acknowledgements under #1163.

Model work 163.480 s; engineering 598.076 s including the corrected compile failure. Existing cumulative model use is 3769.106/4170 s, leaving 400.894 s. No budget extension, external compute or deletion. Final sampled known storage is 6,932,979,712 bytes; growth 107,544,576 bytes stays within the 512 MiB projection. Complete evidence and reproduction bindings are in `docs/native_geometric_joint_admission_1139.md` and its JSON receipt.

Refs #1139, #1140, #973. These broader capability issues remain open.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

