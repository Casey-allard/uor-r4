# ORDER: m1-normalization-mechanism, claude-57 (the non-author FIT review the sheet asks for from Claude or Kimi)

Lab: claude. Item: [m1-normalization-mechanism](../../../../roadmap/items/m1-normalization-mechanism.md) (#2842). Base: `dea171338`. Item PR (partial: `References #2842`; the item stays `ready`).

## Goal

The draft sheet `docs/harness/prep/m1-normalization-mechanism-DRAFT-2026-10-22.md` asks for a non-author `TEST FITNESS` review from Claude or Kimi, because DeepSeek has authored text in it (deepseek-40). Claude has authored none of it. Review it in a fresh context at `dea171338` against CHARTER §4.5 and D22, and record the verdict.

1. Check each FIT block against the sheet and the code at the base: the bar, the base artifact, the data split, the steps, the stop conditions, the cost, the minimum test power and lawfulness.
2. Record the verdict with `uor-harness prep fit -m m1` (one receipt) and put the rendered `TEST FITNESS:` block in the sheet's Verdict section.
3. If the verdict is NOT_FIT, list every prerequisite in priority order, each with its evidence and what would clear it.
4. Make the sheet's top Status line name this review and its verdict.

This unit clears no compute, under any verdict (D22); a run still needs `m1-trunk-assembly-and-training`, PM dispatch and Casey's release.

## Cancel probe (run first)

Class: instrument (a docs verdict on the host; no GPU, no training). At the base, does the sheet already carry a Claude or Kimi verdict recorded after deepseek-40? If it does, the task is void.

## Files in scope

- `docs/harness/prep/m1-normalization-mechanism-DRAFT-2026-10-22.md` (the Status line and the Verdict section only)
- This task folder

## Done when

1. The sheet's Status line and Verdict section carry this review's verdict, its prerequisites in priority order and the rendered record.
2. Every claim in the verdict is checked by a command at the base (a grep, a hash, a test run), not repeated from the sheet.
3. The review changes no bar, hash, cost or stop rule. Anything that would change one is named as a card for the PM or the owner.
4. `loop report --check` and `loop pr-check --local` pass, and a fresh-context review record is posted at the head (docs-only).

## Stop and ask if

- Clearing a prerequisite would need a bar change or an owner value. That is a card, not part of this review.
- A file outside the scope above must change.
