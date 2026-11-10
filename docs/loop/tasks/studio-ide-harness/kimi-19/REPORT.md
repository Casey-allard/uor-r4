# REPORT: register the browser-owned Studio item, kimi-19

Lab: kimi. Item: [studio-ide-harness](../../../../roadmap/items/studio-ide-harness.md) (#3569, registration parent only). New item: `studio-cursor-hosted`; official issue sync is pending. Order: [ORDER.md](ORDER.md). Base: `d4f1e3470ed2681847f33028de3258b5053b63f8`. Draft PR: #3914. References #3569; leaves its status and acceptance unchanged.

## What did not work

- Official `cargo run -p xtask -- roadmap check` and `roadmap render` are RED on four errors in two unchanged base items: `float-kernel-fusion.md` lacks `## What is missing`; `infra-desktop-client-tauri.md` uses `status: in_progress`; their two `issues.json` ids consequently have no valid parsed item. These files and `issues.json` match `origin/main`. The 11 harness gates do not execute this validator.
- Official issue sync uses the same checked loader and cannot run until that base condition changes. No manual issue creation, generated-file edit, validator weakening, unrelated item repair, or alternate render path was substituted.
- The first draft placed its ORDER under the new item. Local policy correctly refused that structure because the item cannot exist at its own registration PR's merge base. The records now use the established parent-item registration route, with a top-session D76 decision limited to the expected new-item scope advisory.
- Agent-98 returned CHANGES on the initial acceptance: an unsatisfiable global-check requirement, a literal closing-keyword/#3569 ambiguity, undefined PR-metadata provenance, overlapping-path coordination, and missing fresh-main verification. Corrections are applied: global check/render/sync is BLOCKED/NOT_RUN with the exact base error set and no-new-errors requirement; #3569 wording uses References only; PR metadata is local/imported or companion-supplied without credentials; overlapping active ORDERs must be disjoint; fresh-main verification is required.
- No application, model, API, service, browser or Tauri implementation was performed in this unit.

## What changed

- Added `studio-cursor-hosted` as a separate ready, host-checked M7 item with browser-owned task/runtime acceptance, optional tool-only Tauri execution, IndexedDB persistence, review/editor workflows, and explicit local acceptance limits.
- The older #3569 acceptance remains open and unchanged. This registration does not fulfil it. No other open PR touches Studio/Tauri/product-document paths; future active ORDERs must keep overlapping paths disjoint without changing #3569's status.
- Added binding rulings for a real `ComposerSurface`, one typed task list/send path, independent 1023/1024 drawer behavior, mode-specific skip links, transport experimentation, native approval authority, explicit `run.resumed`, truthful empty states, provider-key exception, quota/lease behavior, canonical argument rejection and serial red-first task reviews.
- Only the new item and this parent task folder changed; no existing source or generated file changed. Initial ORDER/item commits were `23f936db3`/`117b642b3`; the corrected parent registration ORDER is `0dcfd5cc0`. Size is reported, not capped.

## Measurements

- The new item is 57 lines. The ORDER is trimmed below the 4 KB read-path guidance. Final HEAD/tree are bound in the PR and local evidence after checks, not guessed here.
- Disk at probe: 34.2 GiB free. No model artifact, paid compute, credential or deployment was used.

## Checks run at the head

- `cargo test -q -p xtask roadmap::tests`: **33/33 PASS**; validator unit tests, not whole-roadmap validation.
- `git diff --check`: PASS before the item commits.
- Base identity: #3569, the two malformed roadmap items and `issues.json` match `origin/main`; no diff.
- `cargo run -q -p xtask -- roadmap check` and `roadmap render`: **FAIL**, exactly the four pre-existing errors above; no output generated. Global check/render/sync is BLOCKED/NOT_RUN, not PASS.
- `uor-harness check`: exit **0**; log begins `414 rules, 382 failed (0 error, 381 warn)` and includes existing multi-lab/status/prep warnings plus this then-unclaimed branch. This command does not replace the roadmap validator or loop gates. Log: `local/kimi/cursor-implementation/item-check.log`.
- The first `loop pr-check --local` failed the new-item-at-merge-base ORDER rule. At corrected `b838e6e93f2faaef89bfe4414a010a7e42d42e4e` / tree `afd3ebfed0b16e4b5f68a1bd91009819a2492712`, `loop report --check` and `loop pr-check --local` **PASS**, retaining only the explicitly accepted new-item scope advisory, historical commit-order advisories and existing FILEMAP advisories. Log: `local/kimi/cursor-implementation/item-policy.log`.
- At `b838e6e93`, all **11/11 full gates are GREEN**, exit 0 (`local/kimi/cursor-implementation/item-final-gates.log`). Required GitHub checks are green at that head; historical acknowledgement-only names are not presented as test evidence.
- This REPORT-only correction is followed by final `report --check`, `pr-check --local` and full-gate runs. Their actual results and final HEAD/tree are bound in the PR body and `local/kimi/cursor-implementation/final-*` evidence rather than asserted before execution.

## Skills used

- Isolated worktree, working discipline, solution integrity, writing-plans correction review, subagent-driven task preparation, systematic root-cause investigation, receiving-code-review verification, and protected delivery.
- Agent-98 independently reviewed acceptance and returned CHANGES; its findings are addressed above. Its re-review at `b838e6e93` returned **APPROVE**, with a non-blocking pointer correction now applied in the item's How to start. No subagent decided acceptance or scope. Fresh exact-head review of the final commit is separate.

## Still open

- Official roadmap check/render/sync remains BLOCKED/NOT_RUN pending the unchanged base records. No issue number or generated output is claimed. All 23 implementation tasks remain pending.
- Corrected-head policy/full gates, fresh exact-head approval, required GitHub checks, actual protected merge, fresh-main source/tree verification, #3569-open verification and cleanup are pending until observed.
- Hosted service/integrations, real R4 generation, useful-model evaluation, production deployment and full Cursor parity are NOT_RUN and outside this local implementation authorization.

## Plain summary

This is a registration unit, not a product capability delivery. It creates a separate browser-owned Studio item through the existing parent item and leaves #3569's acceptance untouched. The new documents are in draft PR #3914. The roadmap validator and renderer fail only on two pre-existing malformed items; that failure is explicit and no out-of-scope repair or substitute pass is claimed. No model, hosted service, production release or full-parity claim is made.
