# ORDER: infra-loop-harness, claude-77 (record the owner's 03:41Z rulings; close #2874, #2875, #2876)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 8. Base: `8a646977c`, merged forward onto claude-76 (#3162), which scopes the mechanism board here.

## Goal

The owner ruled on three decision items in Claude's chat at 2026-10-24T03:41:41Z (#2028 comment 6379430093, items 5–7). Nobody recorded the rulings, so the items stayed open. Claude runs solo, and this unit records them and closes the three issues (D70):

- #2875, float path: D73.
- #2874, board disputes: D72, with D11 integer serving moving to READY and Hopf observation split into the kept-fibre map (READY) and sector routing (PARKED).
- #2876, spin-native weights and ternary/XOR: D74, both moving to READY.

#2875's "done when" also asks how architect dispositions reconcile with READY/PARKED. The ruling does not say, so Claude decides it under D68 with a fresh-context decision review: no status is terminal (§4.1, D22), so an architect RETIRE reads as PARKED with its reopen condition.

## Cancel probe (run first)

At `8a646977c`, are the rulings already recorded? No:
- `grep -c -E "^\| D7[2-4] " docs/integration/DECISIONS.md` prints `0`;
- the board still shows `D11 integer serving<br/>ACTIVE`, `Hopf observation` PARKED, and spin-native weights and ternary/XOR PARKED;
- all three items are `status: decision`.

The unit stands. Class: instrument (records).

## Files in scope

- `docs/integration/DECISIONS.md` (D72–D74 appended)
- `docs/mechanisms/README.md`, `lut-serving.md`, `hopf-observation.md`, `spin-weights.md`, `ternary-xor.md`
- `docs/roadmap/items/owner-decision-board-status-disputes.md`, `owner-decision-float-path-scope.md`, `owner-ruling-spin-ternary-arithmetic.md` (`status: done` and the stale text)
- `docs/roadmap/README.md` (re-rendered)
- `docs/loop/tasks/infra-loop-harness/claude-77/**`

## Done when

- D72–D74 are appended, quoting the rulings verbatim.
- The board's flowchart, tables, counts (ACTIVE 9, READY 8, PARKED 11), family table and disputes notes all agree, and each page's status line matches.
- The three items are `done`, and the roadmap view is re-rendered.
- The roadmap check, links, `loop report --check`, `loop pr-check --local`, `git diff --check` and the quick gates pass.
- A fresh-context review at the exact head counts (D66). The PR says `Closes #2874`, `Closes #2875` and `Closes #2876`.

## Stop and ask if

The board's facts would change beyond the three rulings, or a file outside the list is needed.

## Owner approval

Owner path changed: `docs/integration/DECISIONS.md`.

Casey, in Claude's chat at 2026-10-24T03:41:41Z, selecting these options:
- #2875: "Both counts once exported and discrete-trained";
- #2874: "D11 READY, split Hopf (Recommended)";
- #2876: "Keep both READY".

Casey, in Claude's chat by 2026-10-24T21:00:26Z: "Yes, start phase 0: open the records PR with my words verbatim (single-lab rule, review inside the unit, pre-run and decision reviews, post-merge audits, owner queue as a log) and post the #2028 note with each lab's first move. I'll reopen the labs." That #2028 note named this record PR as a first move.
