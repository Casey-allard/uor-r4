# #1578 — Add typed temporal conversation evaluation

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-01T16:12:47Z
- **Updated:** 2026-10-09T13:17:35Z
- **Closed:** 2026-10-01T17:21:26Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1578
- **Merged:** 2026-10-01T17:21:26Z
- **Merge commit:** eb999998a9f38db12a4d6e9d677c17447b909690
- **Base / Head:** main / UOR-Foundation:codex/grounded-script-eval-20261001
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Grounded conversation previously had no scripted evaluator for historical views, scope changes and selected write interventions. This adds `grounded-session evaluate`, retaining exact authored cases and scoring typed actions, memory receipts and whole-answer membership separately. Reload continuity requires successful matching outcomes and exact restored state; failed pairs stay failures.

Gold annotations remain scoring-only. Frozen MWorld2 is unchanged. The tiny CLI witness deliberately cannot emit the accepted names under its two-token cap, keeping failed answer scores visible rather than treating store receipts as language success. Defaults bound input and event work; checkpoint copies remain a separately disclosed storage cost.

Validation: source formatting, diff and claim wording passed locally. Exact-head free CI and independent source review pending. No selected trained-model evaluation or external SSD writes performed.

References #962, #1552.


## Timeline (3 entries; chronological: conversation comments, reviews, review comments)

### comment 5935546187 — 2026-10-01T16:15:26Z — Casey-allard

Independent nonauthor whole-head source review completed by principal mathematician at fffbe32675465cfb0870b289f5bd5b094c9ab08f against3144a383: SOURCE APPROVED, no material blocker. Reviewed oracle/evaluator/CLI/tests/current-state. Gold enters scoring only, view/record/status distinctions retained, NoRead does not fake absence, original/intervention answers separate, errors stay denominators and cannot pass reload continuity. Scope: finite exact answer membership may reject unlisted paraphrases; disk reload is in-process, not process-resume qualification. Reviewer ran no build/tests/model. Exact-head CI remains queued, not PASS.

### comment 5936104846 — 2026-10-01T16:46:23Z — Casey-allard

Actual execution receipt for exact PR head `b8f7679178ff086c6b090b320dd317214d120d13`.

[Evidence run 36892057309](https://github.com/UOR-Foundation/uor-r4/actions/runs/36892057309) completed **SUCCESS**. Its operational workflow commit is `095400c5e62ab45edd468cb4273769f345f0530d`; both checkout and `UOR_BUILD_SOURCE_COMMIT` pin the PR head, and the executed identity check passed. Documentation/policy checks, formatting, and compilation of the training library, `m-world`, `grounded-session`, and `temporal-compiler` passed.

Executed test results:

- `grounded_session_eval::tests`: **4 passed**, 0 failed.
- `answer_oracle::tests`: **6 passed**, 0 failed.
- `temporal_cli`: **1 passed**, 0 failed, 47.02 seconds; this invokes actual fit/score/session/evaluate CLI subprocesses.
- `stack_grounded_session::tests`: **12 passed**, 0 failed, 1 helper ignored in the outer harness. The parent fresh-process test explicitly invoked that helper in a child process, where it separately **passed**.

The executed CLI test asserts evaluator scores of action **53/55**, memory **54/55**, frozen complete-answer membership **0/22**, intervention complete-answer membership **0/1**, and joint **33/55**. It also asserts six successful reload comparisons, refusal of the deliberately failed-turn reload (one failed pair), distinct NoHistory/Evicted outcomes, disabled recall without changing the typed memory result, input-byte/hash preservation, and a sealed baseline-precondition refusal. These are the expected fixture results, including intentional wrong-relation/error sentinels and insufficient tiny-emitter outputs. They verify evaluation and integration behavior; they do not establish useful complete-answer language or general conversation capability.

The prior [failed run 36890276933](https://github.com/UOR-Foundation/uor-r4/actions/runs/36890276933) is retained. The only repair was converting the malformed-input test's string into a `serde_json::Value` at line 1105; no assertion or runtime behavior was weakened. Cache save completed successfully in the replacement run. No local Cargo/model execution or external SSD writes were used for this repair.

Delivery status at this receipt: the separate delivery publisher failed with `Not Found` because no exact-head envelope exists at `delivery/pr-1578/b8f7679178ff086c6b090b320dd317214d120d13.json` on `codex/lab-state`. The live `main` rules require the five historical compatibility statuses and the protected merge queue; `mission-delivery-gate` is not among the required contexts. D19 §5 retains actual checks and recorded exact-head review while treating delivery-evidence as advisory. This comment records execution, not a merge or a substitute for the final source review. No protection bypass is requested.

Authorship disclosure: this reporter authored the FrozenAnswers addition and the one-line test repair; this execution receipt is not represented as a whole-change independent review.


### comment 5936141434 — 2026-10-01T16:48:27Z — Casey-allard

Independent nonauthor source approval retained at exact head `b8f7679178ff086c6b090b320dd317214d120d13` by the principal mathematician reviewer.

The reviewer verified HEAD and the clean worktree, and independently compared the complete diff with previously approved `fffbe32675465cfb0870b289f5bd5b094c9ab08f`. The only change is `grounded_session_eval.rs:1105`, assigning `"a".into()` instead of `"a"` to a `serde_json::Value`. This preserves the deliberately unknown field and its rejection assertion, fixes the RHS type, and changes no production behavior or scoring semantics. **SOURCE APPROVED; no blocker.** The reviewer performed no build/test/model execution; the separate [exact-head execution receipt](https://github.com/UOR-Foundation/uor-r4/pull/1578#issuecomment-5936104846) records those results.

All five required compatibility contexts now pass. Proceeding through the normal protected merge queue under D19 and the owner's standing merge authorization, with the exact head matched; no advisory-publisher repair or protection bypass.



---
_Comment/review ids in this file: 5935546187, 5936104846, 5936141434_

