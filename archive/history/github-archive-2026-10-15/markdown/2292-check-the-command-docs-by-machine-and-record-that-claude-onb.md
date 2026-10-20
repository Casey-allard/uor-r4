# #2292 — Check the command docs by machine, and record that Claude onboarded (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T04:52:32Z
- **Updated:** 2026-10-13T04:56:18Z
- **Closed:** 2026-10-13T04:55:59Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2292
- **Merged:** 2026-10-13T04:55:59Z
- **Merge commit:** b874393134ad0e65c5503943d97bf30df5749cd7
- **Base / Head:** main / UOR-Foundation:harness/doccheck
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

Tooling only.

## Result

Decision: **KEEP** (tooling).

### 1. The command docs are checked by machine

`doccheck.py` validates every `cli.py` command in the harness docs against the real argparse tree.
Registered as `harness.documented_commands`, so it runs in `check` and therefore in CI. It **parses**
rather than executes — safe against a live repository, since `heartbeat` or `team claim` would mutate
state if invoked.

```
$ python3 scripts/harness/doccheck.py
documented commands: 40 checked against the real argument parser
  every documented command exists, with the flags it claims
```

Four documentation errors shipped in the previous three units. Every one was found by **running the
command by hand while writing a record** — a poor detector, since it only works if the author happens
to try that exact string.

| error | caught? |
|---|---|
| `heartbeat --dry-run` — no such flag (it is `--print`) | **yes** |
| a top-level `install-hooks` — it is a `git` subcommand | **yes** |
| `team list` — the subcommand is `roster` | **yes** |
| "`prep new` changes no research state" — it **writes** a sheet | **no** |

That last row is deliberate. It is a claim about behaviour and no argument-parser check can see it.
A validator that is silently incomplete is worse than one whose limits are written down.

### The first version had a hole in the exact case it exists for

`cli.py install-hooks` — error #2 — **was not detected**. `git` was in the tokeniser's skip-list, so
the walk stopped at `git` and never reached `install-hooks` to find it missing. **The check would
have passed the very error it was written for.**

```
cli.py install-hooks        -> FAIL   (a top-level command that is really a subcommand)
cli.py git install-hooks    -> pass
```

A check that passes everything looks identical to one that works, and only a known-bad input tells
them apart. All 13 tests in `test_doccheck.py` are either an error that actually shipped or a shape
the docs really use — including the shell substitution in `--repo "$(git rev-parse --show-toplevel)"`,
whose inner spaces would otherwise split into apparent command words.

### 2. Claude onboarded, and closed the two remaining gaps

The onboarding prompt ran successfully in a lab that did not write it:

| path | foreign labs before | after |
|---|---|---|
| `prep new` | *(none)* | **claude** |
| `team dispatch` | *(none)* | **claude** |
| claim, release | codex, kimi | claude, codex, kimi |
| heartbeat | codex, kimi | claude, codex, kimi |

`prep new` was the one that mattered — the D22 gate standing between a lab and prohibited compute,
now exercised by a lab that did not write it.

Claude's records are unmistakably its own work: a receipt for
`docs/harness/prep/m4-starling-read-2026-10-13.md`, a dispatch of the `evidence-auditor` role, a claim
and release of `onboard/claude`, and a heartbeat reading
`M4: softmax at runtime: yes -> target no (D23 prohibits softmax project-wide)` — real project state.

**One thing that looked like a defect and is not.** The prep receipt names a sheet that does not
exist on disk. That is **correct**: step 8 tells a lab to delete the sheet if it will not use it.
A receipt pointing at a since-deleted sheet records a registration that was made and withdrawn, not
corruption.

```
python3 -m unittest discover -s tests/harness -t .   -> 290 passed on 3.15 AND 3.9
python3 scripts/harness/cli.py check                 -> 45 rules, 0 errors
```

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record: docs/labs/harness-doccheck-2026-10-13/README.md, plus a current-state entry
- [x] Branch deleted after merge

### Stated rather than hidden

- **`doccheck` cannot catch a false description of a real command** — which is exactly the fourth
  error. It checks shape, not truth.
- It reads seven named documents. A command in a new doc is unchecked until that doc is added to the
  list, and nothing enforces the list staying complete.
- **Claude's run proves the prompt is followable once, in one lab, on this machine.** Not that a lab
  on a different runtime, or with no Rust toolchain, gets the same result.
- Whether a withdrawn `prep` registration should leave a receipt pointing at a deleted sheet is a
  design question this PR does not settle.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

