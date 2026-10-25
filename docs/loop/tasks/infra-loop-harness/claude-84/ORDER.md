# ORDER: infra-loop-harness, claude-84 (the D76 workflow steps)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 5. Base: `0d489bb60`.

## Goal

D76 (#3170) made six merge-check rules advisory. Its two workflow parts waited for D76 to reach main, because main's pre-D76 scope rule failed `.github/workflows/*` in the CI policy job. This unit lands them:

1. **`.github/workflows/docs.yml`:** the Links, Claim-wording and Generated catalog/figures steps get `continue-on-error: true`. Archive verify stays hard.
2. **`.github/workflows/relative-query-check.yml`:** deleted. It is tied to a September branch, and no workflow, code or config outside `archive/` uses it. Only audit and move records describe it.

## Cancel probe (run first)

At `0d489bb60`, `docs.yml` has no `continue-on-error`, and `relative-query-check.yml` exists. The unit stands. Class: instrument (gates).

## Files in scope

- `.github/workflows/docs.yml`, `.github/workflows/relative-query-check.yml`
- `docs/loop/tasks/infra-loop-harness/claude-84/**`

## Done when

- The three steps are advisory, archive verify is unchanged, and the old workflow is gone.
- `loop report --check` and `loop pr-check --local` pass.
- A fresh-context review at the exact head counts.
- The PR says `References #2904`.

## Stop and ask if

Anything else in `.github/` would need to change.

## Owner approval

Owner path changed: `.github/**`.

Casey, in Claude's chat by 2026-10-25T00:03:59Z, answered "Yes, change them (Recommended)" to the gates question. The question included "docs/link checks advisory". Recorded as D76.
