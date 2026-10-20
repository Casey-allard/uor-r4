# #2277 — Make the harness discoverable to every lab, and stop unattributed receipts passing silently (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T03:44:21Z
- **Updated:** 2026-10-13T03:46:21Z
- **Closed:** 2026-10-13T03:46:02Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2277
- **Merged:** 2026-10-13T03:46:02Z
- **Merge commit:** 6a8fe8d3f5db72908400568d3c7e82ebb5592538
- **Base / Head:** main / UOR-Foundation:harness/discoverable
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

Tooling only: this touches the harness and one instruction file, neither of which runs model code.

## What and why

Two gaps, both the same shape: a lab not being told something it needed to know.

**1. `AGENTS.md` never mentioned the multi-lab surface.** It is the one file every lab reads first,
and its harness section listed `status`, `brief`, `prep`, `check`, `skill use` and `mistakes` — all
single-lab commands. It said nothing about identity, presence, claims, or the fact that three other
labs share this repository. A lab could follow that section completely and stay invisible to
everyone, which is exactly what happened: no other lab has ever used this harness.

It now opens with the four-lab context, the one-line identity export, a pointer to
[BOOTSTRAP.md](docs/harness/BOOTSTRAP.md), and the three cross-lab commands — before the single-lab
ones, because cross-lab visibility is the thing a new session has to get right first.

**2. `skill use` and `team dispatch` recorded unattributed with no warning.** The `heartbeat` path
warned; the receipt path did not. That is how 13 receipts became orphans — nothing told the session
that its record could not name its author. Both now warn once per process:

```
uor-harness: WARNING - no lab identity, so this record is attributed to nobody and no
             other lab can see who did it. Fix it for this session with:
             export UOR_LAB=<kimi|claude|codex|deepseek>
```

Once per process rather than per record, so a session that deliberately runs unattributed is
informed rather than spammed.

## Result

Decision: **KEEP** (tooling).

Two new tests assert the properties that matter: an unattributed receipt warns and names the fix,
and the warning is not repeated for every subsequent record.

```
python3 -m unittest discover -s tests/harness -t .   -> 276 tests, OK
python3 scripts/harness/cli.py check                 -> 43 rules, 0 errors
python3 scripts/check_claim_wording.py               -> passed
```

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling and instructions only; no capability or headline changed
- [x] `python3 scripts/check_claim_wording.py` passes
- [x] Record: docs/labs/multilab-harness-2026-10-12/README.md (the unit this completes)
- [x] Branch deleted after merge

### Stated rather than hidden

- **Discoverability is not adoption.** This makes the harness impossible for a lab to miss if it
  reads `AGENTS.md`; it does not make any lab read it, and no lab has run this yet.
- The 13 already-orphaned receipts stay orphaned. Nothing can recover which lab wrote them, and
  rewriting them would be inventing evidence — the same mistake as the fabricated presence records
  removed in #2275.
- The warning is one line per process. A session that runs unattributed in a loop sees it once.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

