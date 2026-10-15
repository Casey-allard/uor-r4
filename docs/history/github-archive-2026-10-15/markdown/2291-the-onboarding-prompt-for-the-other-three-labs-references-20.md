# #2291 — The onboarding prompt for the other three labs (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T04:44:59Z
- **Updated:** 2026-10-13T04:46:48Z
- **Closed:** 2026-10-13T04:46:39Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2291
- **Merged:** 2026-10-13T04:46:39Z
- **Merge commit:** f387345d41fe5c416687a3aafe40e096f86f1620
- **Base / Head:** main / UOR-Foundation:harness/onboard-prompt
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

Docs and one instruction file. No model code.

## What and why

You asked for a prompt to give the other three labs. [`ONBOARD-PROMPT.md`](docs/harness/ONBOARD-PROMPT.md)
is paste-ready: **one line changes per lab** (the `UOR_LAB` value), everything else is literal.

It leads with `uor-harness onboard` — the Rust verifier from the previous unit — and then has each lab
do something none of them has done yet.

## Result

Decision: **KEEP** (tooling).

### Why step 4 is the important one

Every other path already has foreign receipts:

| path | foreign receipts |
|---|---|
| claim, release | Kimi, Codex |
| heartbeat | Kimi, Codex |
| committed presence | Kimi (#2278), Codex (#2279) |
| `skill use` | Kimi |
| **`prep new`** | **zero** |
| **`team dispatch`** | **zero** |

`prep new` is the **D22 gate** — the pre-registration sheet that must reach `TEST FITNESS: FIT` before
any compute. If it misbehaves for a lab that did not write it, the failure mode is a lab doing compute
it should not. So the prompt asks specifically whether it is obvious what must be filled in before a
milestone can be called FIT, and treats that answer as worth more than the rest of the exercise.

### Two errors in my own draft, found by running it rather than reading it

1. **I wrote `team list`.** The subcommand is `team roster`. A prompt with a wrong command wastes a
   lab's time and teaches it to distrust the doc.
2. **I claimed nothing here changes research state.** `prep new` **writes** a sheet under
   `docs/harness/prep/`. It is the only write in the exercise, it is the whole point of step 4, and
   the prompt now says so and asks the lab to delete the sheet if it is not going to be used.

Both were caught the same way the two broken install commands were caught last unit — by running the
thing. `scripts/harness/doccheck.py`, which would catch this class automatically, is **still not
shipped**: it works but reports false positives, and a checker that cries wolf is worse than none.

`AGENTS.md` now points at the prompt from the harness section, so a lab finds it without being told.

```
python3 -m unittest discover -s tests/harness -t .   -> 281 passed
python3 scripts/harness/cli.py check                 -> 0 errors
```

## Checklist

- [x] Every command in the prompt run before it was written down
- [x] Tooling and docs only; no capability or headline changed
- [x] Record: docs/labs/harness-onboard-2026-10-13/README.md (the unit this documents)
- [x] AGENTS.md links it from the harness section
- [x] Branch deleted after merge

### Stated rather than hidden

- **The prompt is a request, not a guarantee.** Whether the labs run it is outside my control, and
  the two unproven paths stay unproven until one does.
- If a lab reports that `prep new` looks clear and correct, that is **one** lab's judgement of a
  gate — not a validation that the gate cannot be satisfied wrongly.
- A lab with no Rust toolchain uses the Python fallback in step 2. It still exercises everything
  that matters; it just does not get the verifier.
- I removed a receipt my own earlier test had left in `docs/harness/ledger/unknown-2026-10.jsonl`
  rather than commit it — the same synthetic-record problem I have had to correct twice.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

