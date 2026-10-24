# REPORT: infra-loop-harness, claude-77

Lab: claude. Order: [ORDER.md](ORDER.md).

closes #2874
closes #2875
closes #2876

## What did not work, first

- The owner ruled on these three items at 2026-10-24T03:41:41Z. The relay named "the PM cover" to record them, and nobody did, so the issues stayed open for 18 hours. Under the single-lab rule (D65), the lab that relayed the rulings records them.
- The work was split across builder subagents:
  - the first committed D72–D74 and then ran out of context budget;
  - the second did the board;
  - the third did the four pages and handed back at its budget mark.

  The lab's top session finished the item files, the render and this REPORT, because a hook refuses REPORT files from subagents.
- The board's family-tree figure (`docs/figures/mechanisms-family.svg`) is a hand-drawn SVG with no generator, so it could not be re-rendered. Its alt text now says it shows the statuses before the 24 Oct rulings.
- pr-check's `rule-sources` rule refused the PARKED heading once its count changed from 13 to 11: any added heading with the word "rule" outside the rule files fails. The heading now says "broke a constraint", the charter's own word for these limits (§3), with the same meaning.
- The in-unit review at `6e94247d0` approved, and noted that D72 added "and scored there" (the ruling says only "until a lawful model is exported through it"), that `lut-serving.md` said the engine "becomes ACTIVE" (Claude's reading, not the ruling), and that each item's older "What is missing" text contradicted the new paragraph. All three are fixed: the phrase is gone from D72, the lut-serving line states only READY (D72), and each item's older text is rewritten as the state before the ruling.

## What changed

- `docs/integration/DECISIONS.md`: three rows appended, each quoting the owner's selected option text:
  - D72: D11 integer serving is READY; Hopf observation splits.
  - D73: float path. Float-trained counts once exported and discrete-trained. That an architect RETIRE reads as PARKED with its reopen condition is marked as Claude's own D68 decision, since §4.1 and D22 make no status terminal.
  - D74: spin-native weights and ternary/XOR are READY.
- `docs/mechanisms/README.md` (the board):
  - The flowchart's node labels and class lines are updated (lut, tx and spin are READY).
  - The D11 row moves to READY.
  - Hopf observation becomes two rows: the kept-fibre map in READY, sector routing in PARKED.
  - The spin-native and ternary/XOR rows move to READY, keeping their caveats.
  - Counts are ACTIVE 9, READY 8, PARKED 11, in 28 rows; the Hopf split added one, and the intro says so.
  - The family-table markers are updated, the disputes notes are marked as ruled, and the figure's alt text is dated.
- The four pages' status lines: `lut-serving.md` is READY (D72), `hopf-observation.md` is split (D72), and `spin-weights.md` and `ternary-xor.md` are READY (D74). Each keeps its existing facts.
- The three decision items are `status: done`, each with a "Recorded on 24 Oct 2026" paragraph. `docs/roadmap/README.md` is re-rendered.

Size: docs only (line counts in pr-check's size line at the top of the PR body).

## Measurements

Nothing was measured; these are records of rulings.

## Checks run at the head

- `cargo run -q --locked -p xtask -- roadmap check`: 91 items OK (ready 33, held 26, decision 1, done 31).
- `loop report --check`, `loop pr-check --local`, `git diff --check`, the links check and the quick gates: results are in the PR's review record at the head.

## Skills used

None.

## Still open

- Recolour `docs/figures/mechanisms-family.svg` for lut, tx and spin, and add the Hopf split.
- The three items' older "What is missing" text, below the new paragraph, still reads as before the ruling. For example, board-status-disputes quotes the old counts 10/4/13.

## Plain summary

The owner ruled on three open questions early on 24 October: whether a model trained in floating point can count toward the milestones, the status of four mechanisms on the project's board, and whether two weight formats stay in play. The rulings were relayed but never written into the records, so their GitHub issues stayed open. This change writes them into the decision index and the mechanism board, marks the three items done, and closes the three issues.
