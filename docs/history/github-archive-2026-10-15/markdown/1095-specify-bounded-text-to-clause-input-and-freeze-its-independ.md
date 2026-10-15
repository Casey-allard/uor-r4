# #1095 — Specify bounded text-to-clause input and freeze its independent comparison

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-03T04:50:56Z
- **Updated:** 2026-10-09T14:16:52Z
- **Closed:** 2026-09-03T04:52:15Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1095
- **Merged:** 2026-09-03T04:52:14Z
- **Merge commit:** 3e894820c520f3b7803a48c6a2eeeb5b7d7021c5
- **Base / Head:** main / UOR-Foundation:issue-1085-clause-segmentation
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The accepted learned reference still requires callers to supply five token clauses and their lengths. This specifies `R4TextToClausesV1`: bounded raw-text lexing, delimiter recovery and boolean grammar admission while preserving the #1077 reader, #1073 core, #1079 R4 path, known lexicon/query and four-fact context.

The contract fixes typed request/result/refusal fields, ordered raw/tensor identity, core-vocabulary output decoding, and a separate independent adapter-versus-oracle-segmentation comparison. Parked #1094 owns implementation and pre-observation input/runtime binding. Its numeric fidelity/refusal criteria, replay isolation, resource caps and divergent failure actions are declared before any population preparation or evaluation.

Independent review inspected the actual reader/R4 sources and identified corrections to schema, artifact precedence, preflight failure handling and bounded evidence storage; these are incorporated. The accompanying original-source audit records pinned NEMESIS/W33/UOR relevance without importing parser/model/proof claims. Current roadmap, continuation and claim pointers retain #1079's weak-control result, #1082's descriptive limits, #973 open and #954 blocked.

Validation: independent contract/source review; claim-wording check; changed-document local links and claim-reference/digest integrity; `git diff --check`. No parser implementation, model load, fit, population preparation, evaluation, replay or broad QA ran. Required CI/queue statuses are transport acknowledgements only.

References #1085. Close with the merged commit and DoD evidence after protected delivery; #1094 remains parked.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

