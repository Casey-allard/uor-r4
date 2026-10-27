# REPORT: infra-loop-harness, claude-138 (take codex out of the lab registry)

Lab: claude. Order: [ORDER.md](ORDER.md). References #2904.

## What did not work, first

- **`loop claim` refused the item.** claude-137 (#3307) had put `docs/harness/README.md` and `docs/loop/README.md` in `scope:`. Both are shared registration files, so the claim refused. That was my slip: it passed `xtask roadmap check` and both of claude-137's fresh reviews. This ORDER was written by hand, and the item fix (edit 6) removes both globs. A candidate rule, logged in claude's friction log: `roadmap check` should refuse a shared file in `scope:` as `loop claim` does.
- **The first run of Casey's command did not land.** Casey's first reply said the edit was pushed. But the branch, the worktree's file and every `claude/*` branch were unchanged, and his Terminal panel did not show the command. On the second run, from this session, it printed `true` and pushed `e08e8848f`.
- **The installed binary fails the receipt check on the new registry.** `~/.cargo/bin/uor-harness` is built from `origin/main`'s code. On this branch, `loop onboard --check-receipts` prints "FAIL onboarding receipts: codex is not in docs/harness/labs.json". That is the old gate walking `CLIENTS`, and it shows why edit 3 is needed. Until the shared binary is refreshed after this merges, every lab running that check from it sees the same FAIL. Rebuilding it is not a lab's step.
- **I misread one failing test as environment-only, and CI caught it.** The test is `each_clients_canary_write_is_refused` (`tests/loop_guard.rs`). It runs every client's canary on the real repository root and accepts only a CHARTER, owner-only or R10 refusal.
  - **Locally,** from a lab worktree, antigravity came first and was refused for a worktree reason: "cross-worktree write refused: .worktrees/claude-drop-codex-registry belongs to another lab". I classed that as an instrument fault and stopped there.
  - **CI's plain checkout at `e08e8848f`** showed the real failure: "codex: unexpected refusal reason: lab \"codex\" is not in docs/harness/labs.json". With codex out of the registry, its canary is refused before its payload reaches the path rule.
  - **Fix:** the test now runs in `make_uor_fixture`, whose registry lists every adapter. All five adapters, codex included, are refused by the path rule there, and the worktree failure is gone too. The registered labs' canaries still run on the real repository, inside the receipt test. That test checks that each one is refused, not why: from a lab worktree the reason is the cross-worktree rule. So the real-repository path-rule reason is no longer asserted (round-2 review, advisory).

## What changed

Size, without this task folder: 12 files, +348/−42.
- Harness code: 4 files, +34/−23.
- Harness tests: 2 files, +97/−11.
- Docs, registry and item: 4 files, +8/−8.
- Archive: 2 new files, 209 lines.

The task folder adds the ORDER and this REPORT. The ORDER gave no estimate.

- **Registry (Casey's edit, `e08e8848f`).** `docs/harness/labs.json` lists antigravity, claude, deepseek and kimi, and its note says "four labs". `jq -e '.labs | keys == [...]'` printed `true`.
- **Refusing codex.**
  - `labs.rs`: the tests list four labs, and `codex` is refused by `check_lab` and `resolve_lab`.
  - `lab_registry_cli.rs`: `heartbeat` as `codex` exits 2, and both no-identity hints read exactly `export UOR_LAB=<antigravity|claude|deepseek|kimi>` (`receipt.rs`).
  - `onboard.rs`: the unknown-lab refusal names the four and not codex.
- **The receipt gate follows the registry (the D68 decision recorded in claude-137).**
  - `check_all_receipts` walks `docs/harness/labs.json`. A registered lab with no client adapter is refused, and a new test pins that.
  - `CLIENTS` keeps codex's adapter, and its doc comment says why.
  - In `tests/loop_guard.rs`, the fixture registers every adapter in its own registry copy, so the codex-adapter tests keep running. The real-repo receipt test expects the registry's labs.
- **Docs.**
  - `docs/harness/README.md` says `--check-receipts` covers every registered lab.
  - `docs/loop/README.md` has its title, opening line and labs table at four. Codex's departure is a dated note quoting Casey, and its history rows stay.
- **Archive.**
  - `codex-d63-finite-probe-scope-20261026.patch` holds #3169's three commits (sha256 `0eae76bb98c0f49be180f5f23cd65f947b473ffe64bbbc6f90d96a27d5b7f631`), and a record file sits beside it.
  - From base `71d9247a87ba45588ea00a583d5321ae2a3d6082`, `git apply --cached` into an isolated index and `git am` in a scratch worktree both give tree `8f0f86882ae254f21b548836ab85dd955f74e77c`, the PR's head tree.
- **Item.** `scope:` loses the two shared-file globs.
- **`loop drive --client codex`.** No change was needed. Every `loop …` command resolves the lab through the registry first (`lab_or_refuse` in `main.rs`). With `UOR_LAB=codex`, `loop drive` exits 2 with "lab \"codex\" is not in docs/harness/labs.json".

## Measurements

None of model behaviour. These are the change's own checks, at head `e08e8848f` unless stated.
- **Harness suite** (`cargo test -p uor-r4-harness --no-fail-fast`, all 33 targets): 1,153 passed and 1 failed. The failure was the canary test above; after its fix, and with the test added after the first review, `tests/loop_guard.rs` passes 14 of 14. These new or changed tests pass:
  - `the_real_registry_lists_exactly_the_four`
  - `an_unregistered_lab_is_refused`
  - `an_unregistered_lab_is_refused_by_heartbeat_with_nothing_written`
  - `the_no_identity_hints_list_the_registered_labs`
  - `an_unknown_lab_fails_and_names_the_registered_labs`
  - `committed_onboarding_receipts_are_valid_and_canaries_refuse`
  - `a_registered_lab_without_a_client_adapter_fails_the_receipt_check`
  - `an_adapter_whose_lab_left_the_registry_is_refused_inside_the_repo`, added after the first review. With codex left registered in its fixture, it fails on the owner-path reason instead.
- **Before the registry edit, the tests pinned it.** At `37837239e`, with codex still in `labs.json`, five of them failed:
  - CI's `harness rules and tests`: the two `labs.rs` tests and the onboard test, with 674 passing;
  - locally: the codex heartbeat check in the CLI test.
- **Negative gate for the receipt gate.** I set aside `loopguard.rs`'s diff (a 50-line patch). The real-repo receipt test then failed: `"codex is not in docs/harness/labs.json"`. Restored, it passes. The crate was rebuilt both times.
- **`loop onboard --check-receipts`:**
  - the branch build gives `PASS onboarding receipts: 4 verified and in-process canary writes refused (antigravity, claude, deepseek, kimi)`;
  - the installed main build gives `FAIL … codex is not in docs/harness/labs.json`.
- **Codex refused:** `UOR_LAB=codex uor-harness loop drive …` and `heartbeat` both refuse with the registry's message. `loop drive` exits 2.
- **Archive:** both reproductions give tree `8f0f86882ae254f21b548836ab85dd955f74e77c`.

## Checks run at the head

The exact-head review record on the PR covers these:
- the suite and negative gate above;
- `xtask roadmap check` (95 items OK);
- `loop onboard --check-receipts` from the branch build;
- `loop report --check`;
- `loop pr-check --local`, including `owner-paths` (the ORDER's `## Owner approval` names `docs/harness/labs.json` and quotes Casey);
- `loop gates --full`.

## Skills used

- **holding-the-goal.** Both refusals were read as the lock working:
  - the claim refusal led to fixing the scope, not to working around the claim;
  - the owner-path refusal kept the registry line as Casey's own edit.
- **solution-integrity.** The receipt gate was changed to follow the registry rather than shrinking `CLIENTS`, so codex's adapter keeps its tests.

## Still open

- **For the owner:**
  - `AGENTS.md`'s `UOR_LAB=<antigravity|claude|codex|deepseek|kimi>`, an owner path his instruction did not name;
  - the item's title and "Why it matters" prose ("five-lab");
  - refreshing the shared `~/.cargo/bin/uor-harness` after this merges, so `--check-receipts` passes from it.
- **The candidate `roadmap check` rule** (a shared file in `scope:`).
- **The item itself stays `ready`.** Its done-when asks for every PR in the build plan and every lab's end-to-end task, which this unit does not finish.

## Plain summary

Codex now works on a different project, so Casey asked for it to come off the list of labs that work here. Casey made the one-line change to the list himself, because the tools only let the owner touch it. This change does the rest. The checks and messages that name the labs now name four. The check that every lab has completed its setup now reads the list instead of a fixed five. Codex's last unfinished piece of work is kept in the archive exactly as it was.
