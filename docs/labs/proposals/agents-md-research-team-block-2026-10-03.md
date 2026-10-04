# Proposal: replace the stale "OpenCode research team" block in AGENTS.md

3 October 2026 (Eastern Time) · Support lab (Antigravity) · References #820 · **Proposal only — the owner decides.**
`AGENTS.md` changes need owner direction, so this PR is a draft and does not edit `AGENTS.md`.

## 1. What I found (please read first)

The block the brief calls the "OpenCode research team" section is **not in tracked `AGENTS.md`**.
- `origin/main` (`a621514c`) has no `research-kit:begin/end` markers, and `git log -S'research-kit:begin' -- AGENTS.md` returns nothing.
- It exists as a **13-line uncommitted insertion** at the end of `AGENTS.md` in the owner's checkout (`~/uor-r4`), next to untracked `.opencode/`, `.omo/` and an `AGENTS.md.bak.20260923-180746`. The backup filename is consistent with a 23 September install, before the Sep 25 and later direction changes.

So there is nothing on `main` to replace. This PR therefore (a) does not touch `AGENTS.md`, because an edit near the end of the file would collide with the owner's uncommitted insertion on their next pull, and (b) supplies the replacement text for the owner to paste into the checkout (or commit) if they want it.

I did not edit or read-modify the owner checkout. The diff above was read only.

## 2. Why the block is stale

| Statement in the uncommitted block | Problem today |
|---|---|
| Heading "OpenCode research team"; "`.opencode/OPENCODE-TEAM.md` maps them to OpenCode names"; "Start a milestone with `/proceed-uor-r4`" | OpenCode was removed on 3 October. DeepSeek now runs in its own harness. |
| "The lead (Sisyphus, DeepSeek V4.1 Flash) is the run's principal investigator" | Names an OpenCode agent. Lab leadership and ownership are set by D14/D19 and live GitHub claims, not by a harness-internal lead. |
| "`@uor-architect-reviewer` runs on Kimi K3 and is called only at consequential decision gates" | Conflicts with the AGENTS.md rule that DeepSeek is the default specialist and Kimi needs a new explicit owner request. |
| "No Claude or Codex on the automatic path" | Conflicts with D19: the Claude lab and the reauthorized Codex lab are active participants; D18 §9 recorded Codex/Kimi/Anti-Gravity changes that D19 then partly reversed. |
| Pointers into `.kimi-code/*` for "the team, protocol and tool routing" | `.kimi-code` is described as an optional workspace in the README; those files are not the lab protocol (`docs/labs/protocol.md`). |
| "Work in `~/uor-r4-worktrees/<topic>` … `codex/<topic>` … never edit the owner checkout" | **Still correct.** Kept. |
| "Git: stage named paths, never push to main … merging asks the owner" | Partly stale: the owner has authorized protected merges ("always merge your PRs"), but each lab still reviews at the exact head and uses the protected queue. Reworded. |
| "GitHub: use the authenticated `gh` CLI … never create or store a token" | **Still correct.** Kept. |
| "One cargo process at a time" | Still the local-host rule, but runners/pod rules changed on 3 October (see below). Reworded. |

Three stale mentions also sit in **tracked** `AGENTS.md` (not the block):
- line 4: "Codex rejoins Claude and OpenCode–DeepSeek …" (the 1 October banner);
- line 89: "RDC/DeepSeek/OpenCode sessions, inherit these rules";
- line 145: "Google, OpenCode/DeepSeek/Kimi and Claude" (inside the already-historical Sep 26 paragraph).
These are accurate historical statements about the labs on those dates; only line 4 and line 89 describe the present. Suggested wording is in §4. I have not applied it.

## 3. Proposed replacement block

Harness-neutral, short, and limited to rules the owner has stated. Items the owner must confirm are in square brackets.

```markdown
<!-- research-kit:begin -->
# Research labs and agent sessions

Everything above this block remains authority. Labs (Claude, DeepSeek, Codex and any lab the owner
authorizes) are peers; GitHub issues, claims and live process state decide who owns what. Each lab
runs in its own harness. OpenCode is no longer used (removed 3 October 2026). [Owner: name the
DeepSeek harness and its entry point, or point to the doc that does.]

- Read [README](README.md), [STATUS](STATUS.md), [ROADMAP](ROADMAP.md) and [current state](docs/integration/current-state.md); the [shared lab protocol](docs/labs/protocol.md) governs coordination.
- Specialist routing: DeepSeek is the default specialist model, including architecture review. Kimi/Moonshot needs a new explicit owner request. [Owner: confirm this still holds.]
- Whole-project rule: before changing anything, place it in the programme goal, the latest result and blocker, the mechanism's history and the callers of the code. State what else could be affected and check it.
- Work in `~/uor-r4-worktrees/<topic>` on a `codex/<topic>` branch; never edit the owner's checkout. Never delete unique research, artifacts or worktrees.
- Git: stage named paths; never push to `main`; use `--force-with-lease` only on your own branch. Deliver by protected PR with an exact-head review and the actual scoped checks recorded; do not push `codex/ci/*` branches (hosted runners are reserved for `main`'s required checks).
- Compute: training runs on the owner-authorized GPU pod or the M1; run exact-head checks locally. One cargo process at a time per host; register long jobs per the lab protocol.
- GitHub: use the authenticated `gh` CLI. There is no GitHub MCP and no token file; never ask for, create or store a GitHub token.
- Rust: apply "Rust Engineering Best Practices" (https://gist.github.com/auser/c3161f55a8393faa8af5ddda68c6befa) together with the invariants above.
<!-- research-kit:end -->
```

Notes for the owner:
- The block deliberately drops the Sisyphus/`@uor-architect-reviewer`/`/proceed-uor-r4` machinery. Those live in `.opencode/` and `.kimi-code/`; this proposal does not remove them.
- "Merging asks the owner" is replaced by the exact-head-review/protected-queue rule the owner stated on 30 September and 1 October. Confirm that is the intent.

## 4. Optional wording fixes in tracked `AGENTS.md` (not applied)

| Line | Current | Suggested |
|---|---|---|
| 4 | "Codex rejoins Claude and OpenCode–DeepSeek for the owner-authorized continuation" | "Codex rejoins Claude and the DeepSeek lab for the owner-authorized continuation" |
| 89 | "All agents, including RDC/DeepSeek/OpenCode sessions, inherit these rules" | "All agents, including RDC and DeepSeek sessions, inherit these rules" |
| 145 | historical Sep 26 paragraph | leave as-is: it is explicitly labelled historical |

## 5. Things left for the owner to decide (nothing moved or deleted)

- `.opencode/` and `.omo/` (untracked in the owner checkout; `.opencode/_retired` already exists) and the `AGENTS.md.bak.20260923-180746` backup.
- `docs/labs/prompts/opencode-deepseek.md`, `docs/labs/prompts/opencode-continuous-start.md`, and the OpenCode rows in `docs/labs/README.md`, `adapters.md`, `client-continuation.md`: rename/retarget for DeepSeek's own harness, or mark historical.
- `#1512`'s title ("OpenCode–DeepSeek") — noted in the #820 triage comment.
- The owner checkout's other uncommitted edits (`.cargo/config.toml`, `.kimi-code/agents/uor-architect-reviewer.md`, a deleted corpus manifest) were seen in `git status` and not touched.
