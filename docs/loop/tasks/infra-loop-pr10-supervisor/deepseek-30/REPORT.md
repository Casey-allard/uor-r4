# REPORT: infra-loop-pr10-supervisor, deepseek-30 (repair A: PM-prompt cards on supervisor stops, and the close-out)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Setup row: 10. Base: `f3622f66a`. Item PR; exact-head review at the head.

## What did not work, first

- **The done-when card clause was undelivered, and the first close-out attempt was withdrawn for it.** #2987 delivered the real progress note and the checked model pin (merged `77fedb347`); this repair delivers the PM-prompt cards. The item stayed `ready` through both partial PRs; it flips only now that every clause is delivered.
- **The A ruling shapes the card set, and it is a PM ruling, not ours.** Kimi on #2028 at 18:19:09Z (comment 6348017248): "PM-prompt card on supervisor stops only (STOP, --max-sessions); a clean self-exit is the supervisor's restart path, not a stopped lab, and the pinned progressing-client test (zero cards) stays intact." So: `Next::Stop` cards; an **explicit multi-session** `--max-sessions` limit cards; a clean session exit and the default single-session unit do **not** card (the progressing-client test still asserts zero cards).
- **`--max-sessions` needed a precise boundary.** The default is one session, and carding it would break the pinned zero-card test and spam every normal unit. The card therefore fires only for an explicit limit above one, documented in the README; the multi-session test proves it.
- **The error exit cannot be asserted through the public run loop without a CLI harness.** The CLI's `Err` branch posts a best-effort card (a `let _` so a card failure never masks the exit code) and still prints the error; the new tests cover the STOP and limit cards, and the error card is visible in the diff and docs.
- **A transient test flake was observed once** while the suite ran in parallel (`a_leased_session_is_never_closed_and_notes_exactly_once_per_window`: an extra real-progress note), then passed alone and in two consecutive full-suite runs (24/24). It reads as timing under load, not a logic change from this PR.

## What changed

| File | Class | Lines | What |
|---|---|---|---|
| `crates/uor-r4-harness/src/loopdrive.rs` | code | +~20 | STOP branch calls `card` with a tagged reason; a clean multi-session limit cards once; the CLI error exit posts a best-effort failure card |
| `crates/uor-r4-harness/tests/loop_drive.rs` | test | +~115 | `a_supervisor_stop_cards_the_pm_for_the_next_prompt`; `reaching_an_explicit_multi_session_limit_cards_the_pm` |
| `docs/harness/README.md` | docs | +2 | the supervisor-stops card rule and the default-single-session exception |
| `docs/roadmap/items/infra-loop-pr10-supervisor.md` | docs | 1 line | `status: ready` → `status: done` |
| `docs/roadmap/README.md` | generated | render | the roadmap counts and row move to done |
| `docs/loop/tasks/infra-loop-pr10-supervisor/deepseek-30/{ORDER,REPORT}.md` | docs | small | this order and report, plus receipts |

**Size:** about 150 changed lines (ORDER estimate about 140, 1.07).

## Measurements

- **`cargo test -p uor-r4-harness --test loop_drive`:** 24 passed (22 + 2 new), twice consecutively.
- **The delivered done-when, end to end:** real progress note (`post_note_with`, fake-`gh` post test, #2987); model pin checked (`model_in_launch`, whole-argument cases, #2987); PM card on `STOP` and on an explicit multi-session limit (new tests); call the check-in, watcher and kill-switch tests from the original unit (18/18 at #2946) — every clause now has code and a test.
- **Planted fault unchanged:** the `loop-drive` gate's fault still turns the progress/close behaviour red with the item's own sentence; no gate edit in this PR.

## Checks run at the head

- `cargo fmt -p uor-r4-harness --check` and clippy (harness, `-D warnings`): GREEN.
- `UOR_LAB=deepseek uor-harness loop gates --full`: ten GREEN (the merge-gate `loop-review-changes-blocks` gate is on `main` now).
- `UOR_LAB=deepseek uor-harness loop gates --break all`: ten CAUGHT.
- `uor-harness loop report --check` and `loop pr-check --local`: PASS at the receipt head; `roadmap render --check` current.

## Skills used

- **solution-integrity**: the card set is derived from the PM ruling line by line, and the boundary that keeps the pinned zero-card test intact is tested both ways (clean exit none, explicit multi-session one).
- **holding-the-goal**: the item is closed only when every clause is code-plus-test, not when the paperwork looks ready.

## Still open

- **Nothing for this item.** The `--model` pin source remains the launcher until the owner registry stores model ids (documented, owner change).
- **PR 8 (#2982)** still waits on Kimi; unrelated to this item.

## Plain summary

- **The last PR-10 clause is built.** A supervisor stop (`STOP`, an explicit multi-session limit, or an error exit) now posts a PM card asking the lab to be prompted; a clean session exit and the default single unit do not card, so the progressing-client zero-card test stays intact.
- **The item is done, end to end:** note post and model pin were merged in #2987, and the stop cards land here with tests — so `status: done` is now honest rather than premature.