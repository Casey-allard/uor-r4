# REPORT: infra-loop-harness, claude-2 (PR 0, the premise probes)

Lab: claude. Base: `fedde5eae`. Order: [ORDER.md](ORDER.md). The cancel lines were committed (`67e407fd0`) before any probe ran.

## What did not work, first

- **P-2, Claude Code: the headless CLI cannot sign in.** A headless run ended after 1.85 s with the result "Failed to authenticate: OAuth session expired and could not be refreshed". The command was `claude -p … --model claude-haiku-5-5 --max-turns 8`, with `claude` 2.1.288 from `~/.local/bin`. The Claude desktop app signs in separately, and the standalone CLI's sign-in has lapsed. Signing in is the owner's step: a lab never handles credentials. By the cancel line written before the probe, Claude Code stays interactive-only until the CLI is signed in and P-2 is re-run. Class: instrument (the setup), not mechanism.
- **P-1, Claude Code: only partly tested.** The same failed run did fire the project-level `SessionStart` hook, which recorded one event in the scratch marker file. No tool ran, so the `PreToolUse`, subagent and `Stop` hooks are untested. P-1 waits on the same sign-in.
- **P-6 was not run.** It is optional, and the render PR does not exist yet.

## What was measured

| Premise | Result | Evidence |
|---|---|---|
| P-1, Claude Code | partial: one project-level `SessionStart` hook fired in a headless run, and only that event was observed. The tool, subagent and compaction hooks are untested. | the full hook config, command and output are on [#2028](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6306203659); the scratch files in `local/claude/loop-probes/claude-p1/` are not committed |
| P-2, Claude Code | fails for now: the CLI sign-in has expired | the run's JSON result, on the same #2028 comment |
| P-3 | passes | `git log -1 fedde5eae`: the subject starts with `[claude]`. `docs/harness/claims.json` records `"lab"` per claim. The two review comments on #2895 start with `[kimi]` and `[claude]`. |
| P-4 | passes, tamper-evident only | `.github/workflows/ci.yml` (job `required-transport`) publishes the five required contexts as check runs on `pull_request` and `merge_group`. `gh api repos/UOR-Foundation/uor-r4/rulesets/19597522` shows every required context with no integration pin. A policy job can therefore feed them through `needs:`, with no ruleset change. There are two ways around it. First, `ci.yml` runs from the PR's own code, so a PR editing `.github/**` could skip the policy. Second, with no source pin, a commit status of the same name posted through the API by any write token also satisfies a required check, and all labs post as one account. Pinning the source needs an org admin, and there is none (D54). The guard is the policy job's own record plus the local queue check, which will refuse (PR 5) checks that did not come from the policy job and `.github/**` changes made without the owner's approval. Tamper-evident only. |

## What changed

| File | Lines | Class | What |
|---|---|---|---|
| `docs/loop/tasks/infra-loop-harness/claude-2/ORDER.md` | +42 | docs | the probe order and its cancel lines |
| `docs/loop/tasks/infra-loop-harness/claude-2/REPORT.md` | new | docs | this report |
| `docs/loop/README.md` | the probe table | docs | a results column |

Size is reported, not capped. Nothing outside scope was touched, and no client's global settings were changed.

## Checks run

These run at the head and are recorded in the PR body: the claim-wording gate, `xtask links` on the changed files, the harness real-repo rule tests, and the local copyright check against the Loop Kit.

## Still open

- **The owner signs in the `claude` CLI** (`claude`, then `/login` in a terminal); then P-1 and P-2 are re-run for Claude Code.
- **Codex, Kimi, Antigravity and DeepSeek each run P-1 and P-2 on their own client** and post the result on #2028, tagged with the lab and with the commands and their output.
- **P-4 becomes real in PR 5:** the policy job feeds `required-transport`; `loop queue` refuses required checks that did not come from the policy job; and a `.github/**` change needs the owner's approval.

## Plain summary

- **Lab tags survive merges.** They can be used as the identity system, as the owner chose.
- **The policy check can reach GitHub's required checks without an admin.** It is tamper-evident rather than locked.
- **The headless Claude CLI needs the owner to sign it in.** Until then, the Claude lab cannot be driven by a supervisor.
- **The other four labs still owe their own client probes.**
