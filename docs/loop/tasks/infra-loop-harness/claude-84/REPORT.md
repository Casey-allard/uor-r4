# REPORT: infra-loop-harness, claude-84

Lab: claude. Order: [ORDER.md](ORDER.md).
References #2904.

## What did not work, first

- **These two edits could not ride in #3170.** CI's policy job runs main's merge check, which before D76 failed `.github/workflows/*` on scope. `loop queue` needs that job to report `policy: pass`. So they waited for D76 to reach main as `0d489bb60`.

## What changed

- **`.github/workflows/docs.yml`:** the Links, Claim-wording and Generated catalog/figures steps get `continue-on-error: true`, each with a one-line D76 comment. Archive verify is untouched and still fails a PR.
- **`.github/workflows/relative-query-check.yml`:** deleted. `git grep relative-query -- ':!archive'` finds no workflow or code that uses it.

## Measurements

- None. The change is configuration only.

## Checks run at the head

- `loop report --check` and `loop pr-check --local`: PASS, with main's D76 binary.

## Skills used

- None.

## Still open

- **The ruleset.** Casey makes this change in GitHub settings, because the labs' token is not an admin on the repository. Turn off "branch must be up to date", and set the queue's minimum wait to 0.

## Plain summary

A broken link or a stale catalog no longer turns a pull request red. They are still reported. The one test that protects the archive still blocks.
