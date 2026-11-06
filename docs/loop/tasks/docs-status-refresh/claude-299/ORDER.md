# ORDER: the README front page in plain terms, claude-299

Lab: claude. Item: [docs-status-refresh](../../../../roadmap/items/docs-status-refresh.md) (#3380). Base: `1f2bd76ce58503f25ef24e415dd193bf57086ddb`.

Casey's direct instruction of 6 Nov 2026 was to clean up the repository documentation, "including the readme so that the repo remains a professional open research project". README.md and CONTRIBUTING.md are Kimi's files and #3380 is Kimi's item. This unit is made on the owner's instruction and is noted to Kimi on #2028 (6688130273).

## Goal

Rewrite the README's "What this is", "Where we are" and research-question sections for an outside reader:
- define the unigram and T2 baselines in plain words;
- state M1's limitation next to its result;
- replace the chain of session-report links with the current open question, whether the model learns human-written text (claude-295, claude-297, #3407).

Correct CONTRIBUTING.md's milestone line to match the issue labels.

## Scope

- `README.md`
- `CONTRIBUTING.md`
- `docs/loop/tasks/docs-status-refresh/claude-299/ORDER.md`
- `docs/loop/tasks/docs-status-refresh/claude-299/REPORT.md`
- `docs/roadmap/items/docs-status-refresh.md`, only to add this unit's task folder to the item's scope

No other file may change. This unit has no model run, GPU, bar, status or rule change.

## Cancel probe

GO at 6 Nov 2026 15:40Z. Kimi has no open PR touching README.md or CONTRIBUTING.md (`gh pr list --state open` lists only #3754 and #3796, neither touching them).

## Done when

- Every number in the changed lines matches its source.
- Every relative link resolves.
- `uor-r4-claim-wording` passes.
- A fresh exact-head documentation review approves before the protected merge.

## Stop and report

Stop after the README and CONTRIBUTING edits, the review and merge verification. Do not move or rewrite STATE.md, STATUS.md, HANDOFF.md, ROADMAP.md or PLAN.md in this unit.
