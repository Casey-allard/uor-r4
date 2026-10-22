# ORDER: infra-loop-pr09-client-adapters, antigravity-4 (repair: three required items from 8a093dc review)

Lab: antigravity. Item: [infra-loop-pr09-client-adapters](../../../../roadmap/items/infra-loop-pr09-client-adapters.md) (#2914). Setup row: 9. Base: `d00431587` (`origin/main`), repairing `antigravity-3` (PR #2932) after the counted `CHANGES` record at `8a093dc16fb2401d4872e3747e2584f7e25c5b81` (`claude`, `uor-review v1`, fresh context).

## Goal

Fix all three required items from the `8a093dc` `CHANGES` review record plus the cheap follow-ups:
1. **HIGH 1 — Preserve existing `~/.claude/CLAUDE.md` content between marker lines:** Instead of replacing the user's whole `~/.claude/CLAUDE.md` with the 3-line pointer on `loop install --apply --replace`, merge the UOR-R4 pointer block between `<!-- uor-r4-loop-start -->` and `<!-- uor-r4-loop-end -->` markers so existing user instructions survive intact and re-runs replace only the marked block.
2. **MEDIUM 2 — Timestamped per-run backup folder under `local/loop/install-backups/`:** Write backups for each `loop install --apply --replace` run into a run-stamped directory `local/loop/install-backups/<epoch_ms>-<pid>/` so subsequent runs never overwrite earlier backups.
3. **MEDIUM 3 — Isolate `tests/loop_guard.rs` install tests from the real repository checkout:** Pass a temporary repository fixture instead of `repo_root()` whenever a test invokes `loop install` with `--apply` or `--replace`, so tests never write into the real checkout's `local/` folder.
4. **Follow-ups:** Refuse `loop install --apply` whenever `UOR_LAB` is present in the environment (even if whitespace-only), note in `docs/loop/README.md` that a shell deny-list can never be complete (e.g. `echo 'pkill x' | sh`), and add regression assertions for Codex/DeepSeek hook merging and `UOR_LAB` refusal text.

## Cancel probe (run first)

Is `8a093dc` still `CHANGES` on PR `#2932`, and does `loop install --client claude --apply --replace` on `8a093dc` overwrite prior content in `CLAUDE.md` while writing backups to a fixed path under `local/loop/install-backups/`? Result: yes (`loop review --check --pr 2932` exits `1`, and `loopguard.rs:3269-3276` and `:3762-3778` reproduce both defects). Class: instrument.

## Files in scope

- `crates/uor-r4-harness/src/loopguard.rs`, `crates/uor-r4-harness/tests/loop_guard.rs`, `docs/harness/onboarding/**`
- `docs/loop/README.md`, `docs/harness/README.md` (shared docs)
- `docs/loop/tasks/infra-loop-harness/antigravity-{1,2,3,4}/**`

## Done when

- All 3 required items and the follow-ups have regression tests in `tests/loop_guard.rs` that fail on `8a093dc` and pass on HEAD, and `loop gates --full`, `loop report --check` and `loop pr-check --local` pass.

## Stop and ask if

- Any file outside the scope above must change.
