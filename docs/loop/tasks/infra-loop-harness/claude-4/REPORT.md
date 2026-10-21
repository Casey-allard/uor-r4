# REPORT: infra-loop-harness, claude-4 (loop PR 1: registries, lab check, build items)

Lab: claude. Base: `fc96b4bea`. Order: [ORDER.md](ORDER.md). The ORDER was committed (`913123e90`) before any work, with the cancel probe's result: no lab-name check existed, so the task stood.

## What did not work, first

- **One planned piece was dropped.** The design had "`version` reports its build commit". `crates/uor-r4-harness/src/version.rs` records two earlier attempts to bake the build commit into the binary; both went stale through cargo's cached build output. The command reads the commit at runtime on purpose, so this PR leaves it alone.
- **Two map rows matched the private kit.** The local copyright check found two 8-word runs in the new map rows on the loop page. Both rows were reworded (`f86b7d3ad`), and the whole PR now has 0 matches.
- **The docs builder ran out of room.** It stopped at its context limit after writing the 13 items. The main session wrote the kit-to-loop map.
- **The first head had a real bug.** At `33d64d03a`, a fresh-context review found that `labs.json` was read from the current directory, not the repository root. A recording command run from a subdirectory with a valid lab was therefore refused; the reviewer reproduced this.
- **The same review found three smaller issues:**
  - read-only `team` and `multilab` actions also refused a bad lab name;
  - no test pinned the exit-2 contract;
  - `onboard::KNOWN_LABS` was a second roster that still allowed unlisted labs.
- **All four were fixed in `7bad586f8`:**
  - The registry is found from the git top level, then from each ancestor folder.
  - Only the actions that record a lab name resolve it: team `claim`, `release` and `dispatch`, and multilab `touch`.
  - New integration tests in `crates/uor-r4-harness/tests/lab_registry_cli.rs` pin the subdirectory case, read-only commands accepting any name, and the refusal: exit 2, the five names, nothing written.
  - Onboarding reads `labs.json`. Its one changed assertion now expects FAIL instead of WARN for an unlisted lab, to match the refusal.
- **The second head borrowed a parent repository's registry.** A round-2 fresh-context review at `168092c91` found that the lookup kept climbing past a git top level that had no `labs.json`, into whatever repository enclosed it. So an old worktree under `.worktrees/` would have borrowed the primary checkout's registry instead of refusing, and the rollout note below was false. The reviewer reproduced this.
  - The fix: inside a git work tree, only that tree's top level counts. The ancestor walk now runs only outside git.
  - A new test, `a_nested_repository_without_a_registry_refuses_instead_of_borrowing_one`, failed with the fix set aside (`lab_registry_cli.rs:116`: exit 0, expected 2). With the fix restored, it passes.
  - The same review found a pasted line that had corrupted a comment in `main.rs`. It is restored.
- **The subdirectory fix is shown to be needed.** With it set aside, `the_registry_is_found_from_a_subdirectory` failed with "cannot read the lab registry …/crates/deep/er/docs/harness/labs.json"; with the fix restored, the test passed.

## What changed

| File | Class | What |
|---|---|---|
| `docs/harness/labs.json` (new) | data | The five labs: vendor, client, headless command and open-PR limit. Kimi is marked as PM. |
| `docs/harness/owner-paths.json` (new) | data | The files only the owner changes, each with its reason. |
| `docs/harness/rule-sources.json` (new) | data | The only files allowed to state a rule. It is enforced in a later PR. |
| `crates/uor-r4-harness/src/labs.rs` (new), `lib.rs` | code | `load_labs`, `check_lab`, `resolve_lab`. A missing, malformed or empty registry is refused, with the reason. |
| `crates/uor-r4-harness/src/main.rs` | code | Every command that records under a lab name goes through one resolver, `lab_or_refuse`. These are `heartbeat`, `receipt`, `team`, `multilab`, `notes`, `ledger-sync`, `skill use` and `prep fit`. An unknown name exits 2 and lists the five. `team dispatch` now honours `--lab` as well as `UOR_LAB`, a small behaviour change. |
| `crates/uor-r4-harness/src/receipt.rs`, `main.rs` | code | The hint strings name all five labs. |
| harness integration fixtures | tests | Four fixtures that recorded under unregistered probe labs now carry a `labs.json` in their own scratch repository. One assertion that counted the files in the repository now expects `milestones.json` and `labs.json`. No test was weakened. |
| `docs/roadmap/items/infra-loop-pr02…pr12*.md` (13, new) | docs | One item per remaining build PR, each with dependencies, the team under D52, and its planted fault. |
| `docs/roadmap/README.md` | generated | Re-rendered: 87 items. |
| `docs/loop/README.md` | docs | A new "Kit-to-loop map" section. |

Size is reported here, not capped. Everything stayed inside the ORDER's scope.

## Measurements: gates at the head

- **Harness tests:** `cargo test -p uor-r4-harness --no-fail-fast`: 667 passed, 0 failed, at the fix head.
- **Planted fault:** `a_sixth_lab_is_refused` passes. With `check_lab`'s comparison forced to accept everything, it failed (`test result: FAILED. 0 passed; 1 failed`, from the builder's run), and it passed again once restored.
- **Live refusal:** `UOR_LAB=gemini uor-harness heartbeat -m m1` printed the refusal naming the five labs and exited 2. It ran against a scratch live directory, and nothing was written.
- **Existing records:** every lab name already in the committed ledgers is one of the five (codex 987, antigravity 779, kimi 751, deepseek 115, claude 13, plus `unknown` from runs with no lab set). No current lab is newly refused.
- **Other checks:** `roadmap check` gives 87 items OK, and `render --check` says current. The claim-wording gate passed. `cargo fmt --check` is clean, and clippy shows nothing in the new code. `xtask links` reports 0 broken links in the changed files. The local copyright check against the Loop Kit, over the whole PR, finds 0 matches.

## Skills used

None of the private kit's skills were invoked. The ORDER's cancel probe was run by reading the code first.

## Still open

- `owner-paths.json` and `rule-sources.json` are recorded but not yet enforced. The policy check comes in PR 5, and the rule-source check later.
- **Rollout.**
  - Every lab must rebuild `uor-harness` after this merges.
  - A worktree branched before the merge has no `labs.json`, so the new binary refuses recording commands there. This is by design. The lab merges `main` into that worktree, or starts a fresh one.

## Plain summary

- **The harness knows the five labs and refuses any other name,** so no record can be attributed to a mistyped or unknown lab.
- **Every remaining build step is a roadmap item.** After the next roadmap sync, each becomes an issue a lab can claim, with its team and its planted fault written down.
- **The loop page maps each part of Ron's kit to what replaces it here,** in our own words.
