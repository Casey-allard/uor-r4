# ORDER: infra-loop-harness, claude-2 (PR 0, the premise probes)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md). Base: `fedde5eae` (main with PR A, #2895).

## Goal

Test the four premises the loop build rests on before anything is built on them. Each premise's cancel line is written below, before its probe runs.

## The premises, their probes and their cancel lines

| Id | Premise | Probe | Cancel line (written before the probe) |
|---|---|---|---|
| P-1 | Hooks fire in headless runs and inside subagents, for each client, and the client exposes a compaction event. | In a scratch folder, give the client a project-level hook that appends a line to a marker file. Then run one headless session that calls a tool and starts one subagent that calls a tool. | If a client's hooks do not fire inside its subagents, those subagents get read-only tools and the policy gate checks their output. If hooks do not fire headless at all, that client has no fence and relies on the gate and the queue check. |
| P-2 | A supervisor can run each client headless, with login persisting, no prompts, the model pinned, and the process stoppable by its pid. | One headless run per client with an explicit model and a turn limit, started and stopped by pid. | If any part fails, that client stays interactive-only until it passes. |
| P-3 | Lab tags survive the squash merge and appear on claims, PRs, reviews and comments. | Read the squash commit of #2895 on `main`, the harness claim record, and the review comments on #2895. | If the squash drops the tag, the tag moves into the PR body and the review record only. |
| P-4 | The policy result can feed a check that is already required, with no ruleset change. | Read how `.github/workflows/ci.yml` publishes the five required contexts, and the ruleset's integration pin. | If not, the policy gate stays advisory and the local queue check is the only lock (D54). |

## Who runs what

- **claude:** P-1 and P-2 for Claude Code, plus P-3 and P-4. This is the claude-2 order.
- **codex, kimi, antigravity and deepseek:** P-1 and P-2 for their own clients. Each posts a tagged result on #2028, with the commands and their output. Their cells read "owed by <lab>" until they post.

## Files in scope

- `docs/loop/tasks/infra-loop-harness/claude-2/**`, for this ORDER and its REPORT.
- `docs/loop/README.md`: the probe table's results column only.

Scratch runs stay in `local/claude/loop-probes/`, which is not committed.

## Done when

- P-3 and P-4 have a recorded result with evidence.
- P-1 and P-2 have a recorded result for Claude Code.
- The four other clients' cells name the lab that owes them.
- The REPORT lists what did not work first.

## Stop and ask if

- a probe needs a change to any client's global settings (`~/.claude`, `~/.codex`, `~/.gemini`, `~/.kimi-code`, `~/.dsh`); that is the owner's step;
- a probe would print or read a secret;
- a probe would use a GPU;
- any file outside scope needs to change.
