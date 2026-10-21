# REPORT: infra-loop-harness, claude-3 (P-1 and P-2 re-run for Claude Code)

Lab: claude. Order: [ORDER.md](ORDER.md). The cancel lines are the ones in [claude-2's ORDER](../claude-2/ORDER.md), which were committed before the first run. This ORDER was written after the re-runs, so it only restates those lines.

## What did not work, first

- **The first stop test stopped the session too early.** It sent SIGTERM before the session had started its tool, so it could not show whether a running tool outlives a stop. It was repeated: the second attempt waited until the PreToolUse hook showed the tool running.
- **The probed tool ran in the background.** The session started `sleep 60` with `run_in_background: true` and reached its Stop hook before the kill, with the task still running. So the stop test covers a background shell task only. It does not show how a foreground tool is grouped, and no run signalled a whole process group.
- **An extra session appeared after the stop.** Claude Code started a short second session (`80df5022`) that fired SessionStart and Stop and wrote a summary titled "Background sleep probe was stopped early". A supervisor or guard that counts sessions would count it by mistake.
- **The CLI did not recognise the model id.** Both runs logged `[claude-code:unrecognized_model] {"model":"claude-haiku-5-5","query_source":"sdk"}`. The id was still used: it is the only model in the P-1 run's model usage.
- **The compaction hook is untested.** It is configured, but a run this short never compacts.
- **The stop run left no JSON result.** `run4.json` is empty because the session was killed, so the model-pin evidence comes from the P-1 run alone.

## What was measured

All runs used `claude` 2.1.288 with `--model claude-haiku-5-5`, in `local/claude/loop-probes/claude-p1/` (not committed). The commands and outputs are in the [#2028 comment](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6306306665). Each result comes from one run.

| Probe | Result | Evidence |
|---|---|---|
| P-1, Claude Code | passes on one run, except compaction (untested) | A headless run exited 0 in 19.1 s. Its events, in order: SessionStart; PreToolUse (Bash `echo top-level`); PreToolUse (Agent); PreToolUse (Bash `echo inside-subagent`, with `agent_id`); Stop; SubagentStop (same `agent_id`); Stop. |
| P-2, Claude Code | passes, with the limits above | No new sign-in and no prompts across about 2 minutes of runs. Only `claude-haiku-5-5` appears in the model usage. Session pid 11781 is in group 11780. The background `sleep 60` was pid 12168, in group 12167. `kill -TERM 11781` ended the session about 2 s later (`wait` exit 143), and `sleep 60` was gone afterwards. |

## What it means for the build

- **The guard (PR 9) can scope subagents.** Claude Code's PreToolUse event names the calling subagent, so a narrower sub-order can be enforced for that subagent alone.
- **The supervisor (PR 10) needs these tests before it relies on stopping by pid:**
  - a foreground tool, to see its process group;
  - signalling the whole group, to see whether it reaches every child;
  - telling Claude Code's after-stop summary session apart from a lab session.
- **The stall check should allow for slow starts.** The cap clock starts at launch, and the first tool call came about 20 s after launch on one run.

## What changed

| File | Lines | Class | What |
|---|---|---|---|
| `docs/loop/tasks/infra-loop-harness/claude-3/ORDER.md` | new | docs | the re-run order |
| `docs/loop/tasks/infra-loop-harness/claude-3/REPORT.md` | new | docs | this report |
| `docs/loop/README.md` | 2 cells | docs | Claude Code's P-1 and P-2 results |

Nothing outside the order's scope was touched, and no global client settings were changed.

## Checks run at the head

- the claim-wording gate;
- `xtask links` on `docs/loop/`;
- the harness's real-repo rule tests;
- the local copyright check against the Loop Kit;
- the table shape (5 `|` per row).

The results are in the PR's review record.

## Skills used

None of the private kit's skills were used. This was a probe run against a written ORDER.

## Still open

- Codex, Kimi, Antigravity and DeepSeek each still owe P-1 and P-2 for their own clients (#2028).
- The compaction hook, for every client.
- The three supervisor tests listed above, for PR 10.

## Plain summary

- **Claude Code can run headless with a pinned model,** and its hooks see each tool call, including a subagent's.
- **Stopping a session by its process id ended it and its background task.** How foreground tools and process groups behave is still to be tested before the supervisor relies on it.
- **The other four clients still need their own probes.**
