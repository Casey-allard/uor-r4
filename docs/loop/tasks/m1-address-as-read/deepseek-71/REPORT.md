# REPORT: m1-address-as-read, deepseek-71 (reconcile stale FIT-source wording)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Item #2833.

## What did not work, first

- **The stale sentences were not merely old history.** The sheet's status line keeps the deepseek-38 note "next FIT must come from a lab other than deepseek", and the **live** lab README repeated it as current guidance; the deepseek-41 review and the claude-71 update make that impossible to satisfy, because every lab has now authored sheet text or a panel. Rewriting history would be wrong, so the sheet's sentence carries an explicit "superseded by the claude-71 update" pointer and the README is corrected instead.
- **The README's prerequisite list was stale in the other direction too:** it still listed the panel hash as open and did not mention the now-pinned fresh-V2 BPB stream (#3139); both are recorded as done in the corrected sentence.
- No code, test, verdict, bar, tolerance, checkpoint, panel, hold or item status touched.

## What changed

| File | Change |
|---|---|
| `docs/harness/prep/m1-address-as-read-oracle-DRAFT-2026-10-22.md` | deepseek-38 sentence marked superseded by the claude-71 fresh-context rule (historical text kept) |
| `docs/labs/m1-address-as-read/README.md` | remaining prerequisites corrected: held checkpoint SHA, fresh-context non-author FIT, PM dispatch, typed release; panel-v2 and #3139 BPB pin recorded as done |
| this folder | ORDER and REPORT |

## Measurements

| Check | Result |
|---|---|
| stale sentence occurrences | before: sheet status + README (2 live); after: README none, sheet one marked superseded |
| invariants | NOT_FIT, checkpoint PENDING (#2844), 0.01 tolerance, panel-v2 pin, no holds — unchanged (diff review) |

## Checks run at the head

`git diff --check`, `loop report --check`, `loop pr-check --local` — see the closing line.

## Skills used

- `solution-integrity`: history kept rather than rewritten; the live guidance is the thing corrected; the stale prerequisite list is fixed in the same pass.
- `scope-guard`: the two in-scope docs plus the task folder.

## Still open

- The held checkpoint SHA (#2844), the fresh-context FIT, PM dispatch and Casey's typed release remain required; no compute is cleared.

## Plain summary

- Removed the contradictory "not deepseek" FIT guidance from the live README and marked the historical sheet sentence as superseded, and corrected the README's prerequisite list; no rule changed.