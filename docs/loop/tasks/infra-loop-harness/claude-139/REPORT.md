# REPORT: infra-loop-harness, claude-139 (done-when audit)

Lab: claude. Order: [ORDER.md](ORDER.md). References #2904.

## What did not work, first

- **The bar's review clause predates D66.** It asks for "review by another vendor". Since D66 (24 Oct 2026), CHARTER §5.5 asks for the lab's own fresh-context exact-head review, and the charter wins.
  - Antigravity, DeepSeek and Kimi each have a verified unit reviewed by another vendor (table below).
  - Claude's verified units are under D66, with its own fresh reviews.
  - Claude's one unit reviewed by another vendor, claude-72 (#3141, reviewed by DeepSeek), has no Claude verify note on record. Its merged note was posted by DeepSeek.
- **The status stays `ready`, although the bar is met under the charter's rule.** This item is the only claimable vehicle for item-file maintenance and the shared records its `scope:` holds: `PLAN.md`, `docs/roadmap/items/*.md`, the mechanism pages and more. Seven units have used it for that: claude-106, -108, -121, -137 and -138, and antigravity-80 and -109. A `done` item cannot be claimed. Closing #2904 needs a maintenance item first; see "Still open".
- **Row 5's "one fixture PR per rule fails" is shown only by test names.** All 12 `pr-check` rules have at least one test in `tests/loop_prcheck.rs`, matched by name. I did not read each test to confirm it is a failing fixture for its rule.
- **The render picks up another unit's change.** `docs/roadmap/README.md` also takes `m1-fresh-panel-transfer`'s held → ready from #3311 (antigravity-109), which merged without re-rendering. The counts move to ready 13, held 11.

## What changed

Size: 2 files, +30/−8, plus this task folder.

- **`docs/roadmap/items/infra-loop-harness.md`.**
  - **Title:** "Build the loop harness (the loop subcommands of uor-harness)". "five-lab" is dropped.
  - **"Why it matters":** four labs (five until Codex left on 26 Oct), and the review is the lab's own fresh-context one (D66). It keeps the history of D59 and of cross-vendor review.
  - **"What is missing":** for the build, nothing, with this audit as the evidence. It also says plainly that the item is the maintenance umbrella, with the units that used it.
  - **"How to start":** how a maintenance unit claims the item, and that shared files never go in `scope:`. The build steps stay, marked as history.
  - **"Rules that apply":** the delivery path under D66.
  - **Unchanged:** `scope:`, `status: ready` and the done-when text.
- **`docs/roadmap/README.md`:** the re-render, with the new title and #3311's status change.

## Measurements

The audit, at base `168712ac3`.

**1. Build plan.** Every row has a merged PR, and its planted fault is held by a test that passes at the base:
- harness suite, `cargo test -p uor-r4-harness --no-fail-fast`, all 33 targets: 1,155 passed, 0 failed;
- the `xtask` tests: 6 passed.

| Row | PR | Planted fault | Test that holds it (passes) |
|---|---|---|---|
| A | #2895 | charter text, by gates | `claim-wording` gate; `xtask roadmap check` (95 OK) |
| 1 | #2898 | a sixth lab name is refused | `labs.rs`: `an_unregistered_lab_is_refused` |
| 2 | #2920 | an item scoped `**` is refused | `xtask/src/roadmap.rs`: `roadmap_loop_scope_refuses_every_root_wide_glob` |
| 3 | #2923 | a hand-edited STATE fails | `loop_render.rs`: `a_hand_edited_state_fails_check` |
| 4 | #2921 | `--break all` turns each gate red | `loop_gates_run_rules.rs`: `break_all_turns_each_gate_red_for_its_own_reason` |
| 5 | #2924 | one fixture PR per rule fails | `loop_prcheck.rs`: per-rule tests, at least one per rule (by name) |
| 6 | #2926 | untagged, same-lab-not-fresh, non-Claude setup record refused | `loop_review.rs`: the three `planted_fault_*` tests |
| 7a | #2922 | an overlapping claim is refused | `loop_claim.rs`: `an_overlapping_claim_is_refused` |
| 7b | #2925 | a REPORT missing a section is refused | `loop_report.rs`: `a_report_missing_a_section_is_refused` |
| 7c | #2931 | a later CHANGES blocks queueing | `loop_queue.rs`: `planted_fault_a_later_counted_changes_refuses` |
| 8 | #2982 + follow-ups | a planted broken link fails | `loop_migrate.rs`: `broken_link_in_a_migrated_view_fails`; `docs_links.rs` broken-link tests |
| 9 | #2932 | each client's canary write is refused | `loop_guard.rs`: `each_clients_canary_write_is_refused` |
| 10 | #2946, #2949, #2987, #2993 | a stalled client is closed and carded; a progressing one continues | `loop_drive.rs`: `the_soft_check_in_continues_real_progress_and_closes_a_stall`, `the_fake_clock_closes_a_silent_client_and_cards_it` |
| 11 | #2930 | the window gives up after 5 minutes | `loop_gates_watch.rs`: `the_window_gives_up_after_five_minutes_without_change` |
| 12 | #2935 | a commit without trailers is refused | `git_hooks.rs`: `a_lab_branch_message_without_trailers_is_refused` |

**2. Onboarding.** `loop onboard --check-receipts`, from this base's build, gives `PASS onboarding receipts: 4 verified and in-process canary writes refused (antigravity, claude, deepseek, kimi)`.

**3. One end-to-end unit per lab.**

| Lab | Unit | PR | Review | Verify note |
|---|---|---|---|---|
| antigravity | antigravity-32 | #3061, merged 23 Oct | `[kimi] uor-review v1` APPROVE (another vendor) | #2904, 23 Oct 12:35Z |
| deepseek | loop PR 11 | #2930, merged 22 Oct | `[claude] uor-review v1` APPROVE (another vendor) | #2916, 22 Oct 00:19Z |
| kimi | kimi-8 | #2959, merged 22 Oct | `[claude] uor-review v1` APPROVE (another vendor) | #2906, 22 Oct 13:10Z |
| claude | claude-137 and -138 | #3307 and #3309, merged 26–27 Oct | own fresh-context reviewers (D66) | #2904, `loop close --verify` |

## Checks run at the head

The exact-head review record on the PR covers these:
- `xtask roadmap check` (95 items OK) and the render;
- `loop report --check`;
- `loop pr-check --local`;
- the suite above.

## Skills used

- **holding-the-goal.** The question was whether the item is done, not whether its text looks finished. So every clause got a test that runs, or a record on GitHub. Where the bar's wording and the charter disagree, that is stated rather than smoothed over.

## Still open

- **A maintenance item, then closing #2904.** Close the umbrella in a later unit, once a dedicated item takes the maintenance paths: `PLAN.md`, `docs/roadmap/items/*.md`, the mechanism pages and the rest of this item's non-loop globs.
- **The owner's queue:**
  - the bar's "review by another vendor" wording against D66;
  - `AGENTS.md`'s `UOR_LAB` line, which still names codex;
  - refreshing the shared `~/.cargo/bin/uor-harness`.
- **GitHub issue #2904's title** follows the item's at the next roadmap sync.

## Plain summary

This is the item that built the project's work loop. Every piece on its build list is finished, and each piece's own "does it catch the mistake it should" test passes today. All four labs are signed up and have each finished a full piece of work. The item stays open on purpose for now. The labs also use it as the place to make small fixes to the project's planning files, and closing it would leave them nowhere to do that.
