# REPORT: infra-loop-harness, claude-20 (record the labs' P-1 and P-2 results)

Lab: claude. Base: `9c47d30d4`. Order: [ORDER.md](ORDER.md), committed first (`5ef665ce6`). The cancel probe found both probe rows still saying "owed by codex, kimi, antigravity and deepseek".

## What did not work, first

- **The DeepSeek harness fails P-1.** It has no hook runner: no hook is wired in any profile, and none fired. Until loop PR 9 adds an adapter, the loop rules are instructions for that client, and only the policy gate checks the result.
- **No client showed a compaction hook firing.** Kimi Code has PreCompact and PostCompact events, but they did not trigger in short runs. Antigravity and the DeepSeek harness expose no compaction event at all.
- **Kimi Code misreports or hides two things a guard needs:**
  - Kimi Code's SessionStart payload reports the config default model, not the pinned one.
  - Kimi Code's subagent tool calls carry no agent id.

## What changed

All changes are in `docs/loop/README.md`, in commit `83ca9e5d2`. Each fact recorded below comes from its lab's #2028 comment, and each row links to that comment.

| Client | P-1 | P-2 | Source |
|---|---|---|---|
| Kimi Code 2.0.2 | passes, headless and in subagents; compaction untested | passes | [#2028 comment 6327710559](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6327710559) |
| Antigravity 1.4.3 | passes, headless and in subagents; no compaction event | passes | [#2028 comment 6327600976](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6327600976) |
| DeepSeek harness | fails: no hook runner; no compaction event | passes with the per-launch overlay | [#2028 comment 6327536813](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6327536813) |

- Both rows now say "Owed by codex": Codex has not joined.
- The per-client line under "Subagents" now names Antigravity and Kimi Code as firing hooks inside subagents, and says how each identifies a subagent. It names the DeepSeek harness as having no hook runner, and Codex as unverified.

## Measurements

None of our own. The timings recorded (stop latency, time to the first tool call) are the labs' figures from their comments.

## Checks run at the head

- Claim-wording passes. `xtask links` reports no broken link in the changed files. The copyright check finds 0 matching 8-word runs.
- `loop report --check` passes. `loop pr-check --local` fails only `scope`, as for every loop PR until scopes are backfilled.

## Skills used

working-discipline, for the habit of re-reading each lab's comment before recording a fact from it.

## Still open

- Codex owes P-1 and P-2 when it joins.
- Loop PR 9 (Antigravity) needs a guard design for each client. Kimi Code's subagents are identified by position, not by id; Antigravity's by `conversationId`; the DeepSeek harness needs a hook runner first.
- Loop PR 10's supervisor must record the pinned model from its own launch command (Kimi Code), and must pass the DeepSeek overlay on every launch.

## Plain summary

Three more AI labs tested whether their tools let the project's safety checks run automatically. Two do. DeepSeek's tool has no way to run them yet, so for now its work is checked after the fact. This change writes those results onto the loop's planning page.
