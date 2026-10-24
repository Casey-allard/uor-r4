# REPORT: m1-address-as-read, deepseek-72 (repair after the counted CHANGES on #3140)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Item #2833. Repair for PR #3140.

## What did not work, first

- **The reviewer's two findings were correct.** My deepseek-71 pass fixed only one live FIT-source sentence and left the "PM still selects the data source" clause contradicting the new frozen-panel statement; it also missed the sheet's verdict paragraph repeating the old "from Claude or Kimi — deepseek has authored" rule. Fixed at all three sites; no other rule touched.
- **The sheet's older registration history stays as history.** The deepseek-38 status sentence remains with its supersession marker; the verdict parenthetical now points at the claude-71 rule rather than restating the obsolete one.

## What changed

| File | Change |
|---|---|
| `docs/labs/m1-address-as-read/README.md` | PM selection/freeze made past tense, panel-v2 frozen recorded |
| `docs/harness/prep/m1-address-as-read-oracle-DRAFT-2026-10-22.md` | :148 PM clause past tense with frozen panel-v2; :150 FIT-source parenthetical marked superseded by claude-71 |
| this folder | repair ORDER and REPORT |

## Measurements

| Check | Result |
|---|---|
| live "PM still selects/freezes" statements | 0 after the fix (README + sheet) |
| live "from Claude or Kimi" FIT-source rules | 0 after the fix (sheet :150 now points at claude-71) |
| invariants | NOT_FIT, checkpoint PENDING (#2844), 0.01 registered, panel-v2 pin, no holds — unchanged |

## Checks run at the head

`git diff --check`, `loop report --check`, `loop pr-check --local` — see the closing line.

## Skills used

- `solution-integrity`: the first repair was verified against the reviewer's exact line numbers; the missed occurrences were fixed rather than argued.
- `scope-guard`: the two in-scope docs plus the task folder.

## Still open

- Held checkpoint SHA (#2844), fresh-context FIT, PM dispatch and Casey's typed release; no compute is cleared.

## Plain summary

- Removed the last two contradictions about the PM freezing the panel and the old "not deepseek" FIT rule; wording only.
