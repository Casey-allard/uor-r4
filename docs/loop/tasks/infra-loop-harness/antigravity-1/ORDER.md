# ORDER: infra-loop-pr09-client-adapters, antigravity-1 (loop PR 9: loop guard, install and onboard for each client)

Lab: antigravity. Item: [infra-loop-pr09-client-adapters](../../../../roadmap/items/infra-loop-pr09-client-adapters.md) (#2914). Setup row: 9. Base: `7864b0516`.

## Goal

The item's done-when:
- `loop guard` (`uor-harness loop guard --client <C> --lab <L> --event <E>`) is the single hook entry point and exits at once outside the repository and its worktrees.
- Per-client subagent scoping from the P-1 probe results:
  - Claude Code (`claude`): scopes subagents by `agent_id` in `PreToolUse` against `local/<lab>/<topic>/orders/<n>.md`.
  - Antigravity (`antigravity`): records the root `conversationId` at `SessionStart` and identifies subagents by a differing `conversationId` in `PreToolUse`.
  - Kimi Code (`kimi`): tracks subagent depth between `SubagentStart` and `SubagentStop` per `session_id`.
  - Codex (`codex`) and DeepSeek (`deepseek`): evaluate the same path, scope and command guard on their hook or stream payloads.
- The guard never runs a release command for a lab (D56) and refuses forbidden commands (`--no-verify`, force push, push to `main`, `gh pr merge` without `--match-head-commit`, `git stash`, `pkill`, `killall`, credential reads).
- `loop install` lists every file it will write, backs up each config it edits to `local/loop/install-backups/`, refuses to replace a differing file without `--replace`, writes only to gitignored `local/` inside the repository, installs the six allowed Loop Kit skills (normalizing unquoted YAML `description:` and skipping `start-here`), and retires Kimi's second loop with `<= 1` KB pointers.
- `loop onboard --client <C>` prints `PASS`, `FAIL` or `REFUSED` for: `rules`, `skills`, `hooks`, `canary`, `identity`, `binary`, and ends with the lab's lane and first `PICKUP`. Writes and checks committed onboarding receipts in `docs/harness/onboarding/<client>.json`.
- `loop copyright` compares repository text against the owner's private Loop Kit zip (`8`-word windows), running on the laptop only and storing no kit text or hashes in the repository.
- The planted fault turns the check red: **each client's canary write is refused.**

## Cancel probe (run first)

At base `7864b0516`: `loop guard`, `loop install` and `loop onboard` exit 2 with `not built yet (see the build plan)`, and `docs/harness/onboarding/` is absent. The task stands. Class: instrument.

## Files in scope

- `crates/uor-r4-harness/src/loopguard.rs` (new)
- `crates/uor-r4-harness/src/lib.rs`, `crates/uor-r4-harness/src/loopgates.rs`, `crates/uor-r4-harness/src/doccheck.rs`, `crates/uor-r4-harness/src/main.rs` (dispatch and flag registration only)
- `crates/uor-r4-harness/tests/loop_guard.rs` (new)
- `docs/harness/onboarding/*.json` (committed onboarding receipts per client)
- `docs/harness/README.md` and `docs/loop/README.md`: usage and what is built
- `docs/loop/tasks/infra-loop-harness/antigravity-1/**`
- `docs/roadmap/items/infra-loop-pr09-client-adapters.md` (`status:` line only, at PR open) and `docs/roadmap/README.md` (`cargo xtask roadmap render`)

## Done when

- `loop guard`, `loop install`, `loop onboard` and `loop copyright` are implemented and wired under `uor-harness loop`.
- Each client (`claude`, `codex`, `kimi`, `antigravity`, `deepseek`) has a committed receipt in `docs/harness/onboarding/<client>.json`, and its canary write to a denied path is refused (the planted fault).
- All unit and integration tests pass, `loop gates --full` is GREEN, and `loop copyright` finds `0` matching 8-word runs against the private Loop Kit zip.

## Stop and ask if

- Any file outside scope or any owner-only path (`CHARTER.md`, `AGENTS.md`, `owner-paths.json`, `rule-sources.json`, `labs.json`) needs to change.
