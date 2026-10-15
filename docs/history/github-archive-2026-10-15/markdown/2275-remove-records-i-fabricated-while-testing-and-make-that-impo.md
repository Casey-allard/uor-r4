# #2275 — Remove records I fabricated while testing, and make that impossible to repeat (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T03:35:03Z
- **Updated:** 2026-10-13T03:37:15Z
- **Closed:** 2026-10-13T03:36:55Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2275
- **Merged:** 2026-10-13T03:36:55Z
- **Merge commit:** 12840562f2b18c58a52d94d1bf42b0b71e54f2d2
- **Base / Head:** main / UOR-Foundation:fix/fabricated-records
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

Tooling only: this touches the harness, which runs no model code. The figures below measure the
harness, taken here.

## What and why

**First, a correction.** PR #2274 committed `docs/harness/presence/kimi.json`,
`presence/claude.json` and `ledger/claude-2026-10.jsonl`. Those were written by **my smoke tests**,
which set `UOR_LAB=kimi` and `UOR_LAB=claude` by hand to exercise the two-tier reader. No other lab
has ever run this harness.

I traced it before deleting: the values echoed the registry, and the registry was written by my own
harness commit `0b9977323`, not by either lab. So this was not peer activity misattributed by
accident — it was entirely my own test data, placed in the committed record, where it was
indistinguishable from two labs genuinely checking in. A reader would have concluded that Kimi and
Claude had each published a live check-in, with line counts that neither had reported: the values
echoed the registry, which my own harness commit had written. Neither lab has ever run this tool.

That is precisely the failure this coordination layer exists to prevent, so this PR fixes it three
ways and adds the test that would have caught it.

## Result

Decision: **KEEP** (tooling).

**1. The fabricated records are removed**, along with the machine-local live tier copy.

**2. [BOOTSTRAP.md](docs/harness/BOOTSTRAP.md)** — a ten-minute, no-compute script to paste into
**one** lab that has never run the harness. It exports `UOR_LAB`, runs the tool on that lab's own
interpreter, claims and releases a scratch branch to prove claims are visible to peers, checks in
once, and confirms its own presence appears `via live`. It commits nothing, and it includes what
each possible observation *means* — so a confusing result is reported rather than swallowed. This
is the adoption test: every test so far is the harness exercising itself.

**3. The identity warning is loud.** `heartbeat` without `UOR_LAB` used to drop a note into a JSON
field nobody reads:

```
uor-harness: WARNING - no lab identity. Receipts and presence will be unattributed.
             export UOR_LAB=<lab>   (kimi | claude | codex | deepseek)
```

Absence is visible; misattribution is not. An unattributed receipt cannot answer "which lab did
this", which is the whole purpose of the layer.

**4. `presence/README.md` forbids test data** and gives the safe alternative — point
`$UOR_HARNESS_LIVE` at a temp directory, which is what the suite now does for every test class.

The session contract also names the multi-lab surface (`export UOR_LAB`, `multilab presence`,
`multilab claims`), so a session discovers it rather than being told.

```
python3 -m unittest discover -s tests/harness -t .   -> 269 tests, OK
python3 scripts/harness/cli.py check                 -> 43 rules, 0 errors
```

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] `python3 scripts/check_claim_wording.py` passes
- [x] Record: docs/labs/multilab-harness-2026-10-12/README.md (the unit this corrects)
- [x] Branch deleted after merge

### Stated rather than hidden

- The harness has **still never been run by another lab.** This PR makes that testable; it does not
  make it tested. The bootstrap is the next step and it needs a human to hand it to a session.
- The two records were live in `main` for about 20 minutes. Anything cloned in that window saw two
  labs that had not checked in.
- I cannot prove no *other* fabricated record exists from earlier work — only that these two did,
  and that I have now traced and removed them.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

