# ORDER: m3-nonrelation-prose-panel, deepseek-41 (non-author FITNESS review of the draft sheet)

Lab: deepseek. Item: [m3-nonrelation-prose-panel](../../../../roadmap/items/m3-nonrelation-prose-panel.md) (#2856). Base: `dea171338`. Item PR (partial: `References #2856`; the item stays `ready`).

## Goal

Act as the non-author `TEST FITNESS` reviewer for antigravity's draft `docs/labs/m3-nonrelation-prose-panel/preregistration-DRAFT-2026-10-22.md` (CHARTER §4.5, D22). The design note, frozen 50-turn panel (`7c29c665…`) and deterministic grader were already merged in `#2952`; this unit checks whether the sheet actually pins the configuration a FIT would clear, and records the verdict. No compute is cleared.

## Cancel probe (run first)

Class **instrument** (docs verdict; no GPU, no training). At the base: the sheet says `TEST FITNESS: PENDING`; no non-author FIT review is recorded on #2856; the panel hash matches the file and `data/panels/MANIFEST.sha256`; the grader's 4 finite controls pass. A prior reviewer has not already recorded a verdict. Task stands.

## Files in scope

- `docs/labs/m3-nonrelation-prose-panel/**` (the draft sheet and lab record)
- this task folder

## Done when

- `uor-harness prep fit` recorded the verdict (one receipt) and the rendered `TEST FITNESS: ...` block is pasted into the draft sheet, with the status line matching.
- The verdict names every concrete prerequisite it requires, with evidence: the owner bar must be quoted with its ISO-UTC time (it is registered but the sheet still says pending), the saved world=v2 compiler artifact/tokenizer must be pinned (the sheet names no artifact hash), and the grader should be cited by commit.
- The panel hash, manifest entry and the 4 finite controls are verified by command, not repeated from the sheet.
- `loop report --check`, `loop pr-check --local`, `loop gates --full` and a fresh-context review record at the head pass (docs-only PR).

## Stop and ask if

- A bar or an artifact hash would have to be invented (they do not: the bar is registered and the artifact belongs to the author's next repair).
- A file outside the scope above must change.