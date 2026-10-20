# #2283 — Correct the record: two labs used the harness, and Sol is Codex (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T04:03:34Z
- **Updated:** 2026-10-13T04:05:54Z
- **Closed:** 2026-10-13T04:05:34Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2283
- **Merged:** 2026-10-13T04:05:34Z
- **Merge commit:** 62c95724df188587fe07241cfd06f488ad5559b9
- **Base / Head:** main / UOR-Foundation:fix/measured-adoption
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

Tooling and records only; no model code is touched.

## Two corrections, both to my own reporting

**1. Lab identities.** I wrote "four labs (Kimi, Claude, Sol, DeepSeek)" and treated **Sol as a
separate participant**. Sol is Codex's model — there are **three** labs plus this one: Kimi, Claude,
and Codex/Sol. Fixed in `AGENTS.md` (the one file every lab reads), the harness README,
`STATE-FORMATS.md`, `BOOTSTRAP.md` and both records. No stale "four labs" or "three other labs"
wording remains in the files I own.

**2. Adoption.** Until this unit the record claimed the harness was untouched by any peer. That
was wrong, and it understated the adoption badly. Counting committed receipts
by lab:

| lab | receipts | kinds | |
|---|---|---|---|
| `codex` | 3 | claim 1, heartbeat 1, release 1 | **foreign** |
| `kimi` | 9 | skill 4, heartbeat 3, claim 2 | **foreign** |
| `lead` | 10 | this lab | |
| `unknown` | 24 | unattributed | |

**Twelve foreign receipts across two labs.** Codex's are a complete, correct lifecycle:

```
claim     codex/spiralcore-fbs-projection-20261012
          "Existing #2029 claim predates this harness; registering final delivery
           reconciliation, PR2279."
heartbeat m2 "unified geometric route/action/learning integration"
          "PR2279 final review/protected merge; CUDA4/static3/artifact2/refusal1 pass;
           pod closed, iCloud/HF verified, training held."
release   codex/spiralcore-fbs-projection-20261012
```

Note the claim comment: its branch claim **predated the harness**. That is the migration case the
design had to handle, and it handled it.

I found this only because a `codex` shard appeared in the ledger while I was checking something
else. My "no other lab has used this" conclusion was wrong, more than once, because I reasoned from
what I expected rather than counting the receipts.

## Result

Decision: **KEEP** — the harness is adopted, and the record now says so accurately.

```
python3 -m unittest discover -s tests/harness -t .   -> 278 tests, OK
python3 scripts/harness/cli.py check                 -> 46 rules, 0 errors
python3 scripts/harness/statecheck.py                -> pass
```

## Checklist

- [x] Checks run at this exact head (above)
- [x] Records corrected: docs/labs/multilab-harness-2026-10-12/README.md,
      docs/labs/harness-rust-port-2026-10-13/README.md, plus AGENTS.md, README, STATE-FORMATS, BOOTSTRAP
- [x] Branch deleted after merge

### Stated rather than hidden

- **Presence was never watched in real time by a peer.** Both labs published to the live tier, but it
  is uncommitted by design and cleared routinely — and one of those clears was mine, earlier in this
  session. Nothing durable was lost (the heartbeat receipt is the record; a live entry is stale in 30
  minutes), but a foreign lab's presence disappeared without anyone asking. The record now says so.
- Kimi's presence record was **never committed** — only the live tier — so its `presence/<lab>.json`
  had no chance to survive. That plus my clear is the argument for treating the live tier as
  explicitly advisory rather than somewhere history is expected.
- `claude` has **no receipts at all**. Silence is not evidence of inactivity, and I am not claiming
  it either way.
- I still cannot rule out that some receipt attributed to a lab was authored by a test of mine. I
  checked the two heartbeats I know I wrote and they are not in the committed shards.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

