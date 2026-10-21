# ORDER: infra-loop-pr05-policy-check, claude-16 (loop PR 5: loop pr-check and the policy job)

Lab: claude. Item: [infra-loop-pr05-policy-check](../../../../roadmap/items/infra-loop-pr05-policy-check.md) (#2908). Setup row: 5. Base: `24130f6df`.

## Goal

The item's done-when: `loop pr-check` checks order, scope at the base, commit order, review record, REPORT, owner paths, rule sources, the rendered-file lock, read path and FILEMAP. The policy job runs base code on PR events and feeds `required-transport` in `ci.yml`. `pr-check` refuses a PR whose required checks did not come from the policy job, or that changes `.github/**` without the owner's approval. The planted fault: **one fixture PR per rule fails.**

## Cancel probe (run first)

Does `loop pr-check` or `.github/workflows/gate.yml` already exist? Result at `24130f6df`: no. `loop` has `gates`, `render`, `claim`, `suborder` and `ask`. The task stands. Class: instrument.

## Owner approval for `.github/**` (an owner path)

Asked in this lab's chat on 21 Oct 2026. The answers were recorded at 2026-10-21T15:25:16Z, right after they arrived.
- "Do you approve these .github changes?" (add `gate.yml`; make `required-transport` run after the policy job). Casey: "Approve (Recommended)".
- "How should the policy verdict feed the five required checks?" Casey: "Record only (Recommended)".

## Architecture calls (Claude, D52)

1. **Inputs.** The base and head commits, read with `git` only. The rules read the registries, items and budgets at the base, so a PR cannot loosen its own check.
2. **Exit codes.** 0 means every rule passes; warnings are allowed. 1 means a rule failed. 2 means the check could not run. Size is printed first, by class.
3. **Files a later PR creates.** A rule whose governing file does not exist yet (FILEMAP, the rendered views) reports "not applicable" and names the PR that creates it. Budgets fail only when the PR makes things worse.
4. **GitHub rules.** The review record and the required-check source go through the `Gh` trait and its fake. `--local` skips them.
5. **CI.** `gate.yml` holds `policy` and `tests`. `ci.yml` calls it as a reusable workflow. `required-transport` runs `always()`, after the policy job. The checks stay green, and each check's summary records the policy verdict and run id.

## Files in scope

- `crates/uor-r4-harness/src/**` and `crates/uor-r4-harness/tests/**`
- `.github/workflows/gate.yml` (new) and `.github/workflows/ci.yml` (approved above)
- `docs/harness/README.md` and `docs/loop/README.md`: usage, and what is built.
- `docs/loop/tasks/infra-loop-harness/claude-16/**`

## Done when

- Each of the ten rules, the required-check source, and the `.github` approval has a fixture PR that fails it, and a green fixture passes.
- On this PR's own CI run, the policy job runs base code and its verdict appears in the five checks' summaries.
- The full harness suite passes, as do claim-wording, links and the local copyright check.

## Stop and ask if

- A rule needs a GitHub write, or a test needs the network.
- Any file outside scope needs to change.
- The CI change could stop the five required checks from being published.
