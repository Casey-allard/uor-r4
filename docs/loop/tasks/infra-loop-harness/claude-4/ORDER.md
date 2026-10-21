# ORDER: infra-loop-harness, claude-4 (loop PR 1: registries, lab check, build items)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md). Base: `fc96b4bea`. This is PR 1 of the build plan in [the loop page](../../../README.md#build-plan).

## Goal

Lay the ground the later loop PRs read. That means:
- one registry of the five labs, which the harness checks every lab name against;
- the owner-only paths and the rule-source files, written down as data;
- one roadmap item per remaining build PR, so each PR becomes an issue a lab can claim;
- a map from each part of the Loop Kit to the loop step and command that replace it, in our own words.

## Cancel probe (run first)

**Probe:** does the harness already check lab names?

**Cancel line, written before the probe:** if a check already exists, drop the registry check and only add the five-lab data.

**Result, from reading `origin/main` at `fc96b4bea`:**
- `known_labs` in `crates/uor-r4-harness/src/main.rs` only lists the labs that have a presence file.
- No command refuses an unknown `UOR_LAB`.
- Two hint strings name four labs: `main.rs` (`cmd_heartbeat`) and `receipt.rs` (`UNATTRIBUTED_WARNING`).

**The task stands.** Class: instrument.

**Dropped from the plan, because it was tried before:** the design's "`version` reports its build commit". `crates/uor-r4-harness/src/version.rs` records that two attempts to bake the build commit into the binary went stale. It reads the commit at runtime on purpose.

## Files in scope

- `docs/harness/labs.json`, `docs/harness/owner-paths.json`, `docs/harness/rule-sources.json` (new)
- `crates/uor-r4-harness/src/**`: a registry module, the lab check wired into every command that takes a lab name, and the hint strings
- `docs/roadmap/items/infra-loop-pr*.md` (new), `docs/roadmap/README.md` (re-rendered)
- `docs/loop/README.md`: the kit-to-loop map section only
- `docs/loop/tasks/infra-loop-harness/claude-4/**`

## Done when

- `uor-harness` refuses an unknown lab name and lists the five.
- The planted fault holds: a test that sets a sixth lab name expects the refusal.
- `cargo test -p uor-r4-harness` passes.
- `roadmap check` passes, with 74 plus the new items.
- The links, claim-wording and local copyright checks all pass.

## Stop and ask if

- a file outside scope needs to change;
- the lab check would block a command an owner runs without a lab name, because owner commands must keep working;
- a test would need weakening;
- a gate still fails after 3 attempts.
