# REPORT: infra-loop-harness, deepseek-73 (refresh two stale roadmap item sections)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Item #2904 (umbrella). Base: `5844d911c`.

## What did not work, first

- **The first prepared patch is obsolete.** The earlier `item-text-refresh-2.patch` also rewrote `m1-address-as-read`, but Kimi's #3144 (`875a2f309`) reconciled that item first (consumer/panel-v2/BPB pinned, remaining checkpoint/FIT/dispatch/chat approval/GPU run). That patch failed `git apply --check` on current main and was retired (kept as `.obsolete`); this unit covers only the two still-stale items.
- **Roadmap sync reports two issue updates.** `xtask roadmap sync` (dry run) says 2 issues would be updated from the changed item text; issues are synced by the PM's tooling, not by this docs PR, and no issue was touched.
- **This is text only.** No bar, threshold, checkpoint, panel, hold or item status is changed; the m2 panel is still `frozen: false` pending its maintainer upload and M3 rates are still `NOT_RUN`.

## What changed

| File | Change |
|---|---|
| `docs/roadmap/items/m2-fresh-heldout-panel.md` | "fresh test is NOT_RUN / no noise stated" replaced with the staged m2-fresh256 facts (pins, 0/128 controls, measured noise, owner bar, deepseek-43 FIT, upload-only freeze, M2 HELD) |
| `docs/roadmap/items/m3-nonrelation-prose-panel.md` | "no pre-registered set exists" replaced with the frozen 50-turn panel, grader + four controls, NOT_FIT sheet with pinned world=v2 compiler, and the remaining FIT/PM/owner/chat-approval steps (rates NOT_RUN) |
| this folder | ORDER and REPORT |

## Measurements

| Check | Result |
|---|---|
| m2 sealed artifact pins | confirmed earlier from the local sealed copy (`0ed1ab98…`, `1de2fc28…`, `3b5c5477…`, `7c33f3f7…`); this patch only records them |
| m3 panel | `7c29c665…`, 50 turns, 25 heldout with zero 4-word runs against the world sources (independently re-computed earlier on #2856) |
| roadmap check/render/sync | check OK (91 items); render produced no diff; sync dry-run: 2 issues would update |

## Checks run at the head

`xtask roadmap check`/render/sync (dry run), `loop report --check`, `loop pr-check --local`, `loop gates --quick`, `git diff --check` — see the closing line.

## Skills used

- `solution-integrity`: the obsolete patch was retired after failing apply-check; both replacement paragraphs cite merged evidence and keep the remaining gates explicit.
- `scope-guard`: two roadmap item text lines plus the task folder.

## Still open

- Both items remain at their current statuses (`ready`); this records reality, it advances no step.
- m2-fresh256 still needs the maintainer HF upload before `frozen: true`; m3 still needs a fresh non-author FIT, PM/owner confirmation and chat approval before the held baseline run.

## Plain summary

- Rewrote the two remaining stale "What is missing" sections to the delivered state and remaining gates, without touching any status or value.
