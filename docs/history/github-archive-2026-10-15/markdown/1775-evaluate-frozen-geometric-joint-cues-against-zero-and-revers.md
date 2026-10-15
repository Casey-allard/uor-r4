# #1775 — Evaluate frozen geometric joint cues against zero and reversed controls

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-06T01:18:14Z
- **Updated:** 2026-10-09T13:17:22Z
- **Closed:** 2026-10-06T22:45:06Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1775
- **Merged:** not merged
- **Merge commit:** 1d5161bf983003f5d9c2333d5aaf4e48316c0556
- **Base / Head:** codex/geometric-cue-joint-learning-20261006 / UOR-Foundation:codex/geometric-cue-joint-transfer-20261006
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The previous joint-cue fit improved development completion59→74/512, but lacked an independently frozen transfer comparison. This adds a four-arm evaluator (original unary, exact zero joint replay, learned ordered joint, reversed frozen joint) and a fresh-only Rust panel author that binds the actual old training plan and preserves28 unique exposure exclusions without producing an unused512-row development panel.

At numerical head8d09c53e62ac84825de0622e8c92e44739d6cb6e, prospective completion is flat20/128 original→learned (3 gains/3 losses), first-source68→67, paired complete3→4/64; reversed completes18. Zero overlays exactly replay original numerical behavior. The exposed diagnostic is reported separately. No gradients, fitting, updates or arm selection occur. Retain the candidate; no transfer, general-chat or geometry-advantage qualification.

Independent saved-trace review finds distinct consumed query/relative/joint descriptors in all64 opposite-role pairs, yet learned ranking picks the same source for both questions in33 pairs. This supports investigating geometric ranking/credit coadaptation, while later continuation failures remain separate. DeepSeek owns softmax/tensorcore training.

Validation: focused Rust plan tests, formatting and release build of fitter/plan/panel examples PASS (202.370s, CPU8, peak RSS5,006,172,160B); actual preparation4.063s and frozen evaluations18.728s/17.876s, CPU16, Linux x86_64 with CUDA hidden. Executable SHA292df8ca01599170f2d7b45de4ad256921675968b6c05285e688a283418f7168; fresh panel manifestce9d0f039c896e67c72c0e3125576903bc05892193058902d012a59281e60f87. Preserved setup refusals happened before model loading and are not model-quality evidence. Final documentation-only commit retains identical numerical source; claim-wording gate and diff check PASS.

Durable sealed evidence: /workspace/uor-r4/codex/cue-joint-transfer/{evaluate-diagnostic-2,evaluate-fresh-2}; paired audit analysis-paired-first-read-1/audit.json. Stacked on PR1772; not delivered to main.

References #820.


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 6007348155 — 2026-10-06T01:19:33Z — Casey-allard

Exact-head checks at 48e8a6adf0a520b6eb051d8c160c343851bf5677 PASS on Linux x86_64 pod, CPU8, CUDA hidden: formatting, integer/ordered-carrier and artifact checks, both example suites, and optimized fitter/plan/panel builds. All7 commands exited0; complete checks elapsed 174.223s; peak own-process-group RSS 3130540032 bytes. Native fitter binary SHA25619bfedf298d97912193f900552c18857ece6e88703afe2f2e1aec18854cf5eb7. No transfer predictions or new training run.

Independent source/data audit verified8 construction-plan manifest identities and115 blocked ordered banks. Remaining disjoint banks do not support another full512/128 authoring draw. Retain all exclusions. Next dependency: explicit fixed fresh-only128 preparation, with literal coverage checked against hashbound existing training rows; source reproduction suggests capacity, but Rust preparation is NOT_RUN. DeepSeek's softmax/tensorcore offline training seam remains untouched.


### comment 6026819275 — 2026-10-06T22:45:03Z — Casey-allard

RECORD: frozen geometric joint-cue transfer controls left prospective completion20/128→20/128; no transfer qualification. Retained result/four controls: https://github.com/UOR-Foundation/uor-r4/pull/1775. Branch preserved.


---
_Comment/review ids in this file: 6007348155, 6026819275_

