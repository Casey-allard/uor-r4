# #2294 — The doc checker reads every document, not a list I wrote (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T04:58:34Z
- **Updated:** 2026-10-13T05:00:58Z
- **Closed:** 2026-10-13T05:00:43Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2294
- **Merged:** 2026-10-13T05:00:43Z
- **Merge commit:** 892253741358d8154ff1a5296ff098e05b6f1ae5
- **Base / Head:** main / UOR-Foundation:harness/doccheck-discovery
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

Tooling only.

## What and why

The `doccheck` rule shipped in #2292 with a **hand-written list of seven documents**. That list was
already incomplete when I wrote it: `CONTRIBUTING.md` alone carries nine `cli.py` commands and was
never checked, and any new document would have been invisible until someone remembered to add it.

```
before:  documented commands: 40 checked
after:   documented commands: 96 checked      (across 913 documents)
```

**56 commands were unchecked.**

## Result

Decision: **KEEP** (tooling).

Discovery now covers every tracked Markdown file under `docs/` and the repository root. It found one
immediately — `docs/integration/current-state.md` naming the broken `install-hooks` from the previous
unit — and that turned out to be a **false positive**, which produced the more useful rule.

### Prescriptive versus descriptive

A document that **tells a lab what to run** is checked. A document that **records what was run** is
not. That is the line the exclusions draw:

| excluded | reason |
|---|---|
| `docs/history/` | archived evidence, kept verbatim |
| `docs/labs/<topic>-<date>/` | dated records of work done — rewriting them to satisfy a parser would falsify the record |
| `docs/integration/current-state.md` | an append-only log that records commands |
| `research/`, `.harness/`, `local/`, `target/`, `.worktrees/` | evidence or scratch, not documentation |

The changelog case is the interesting one. An entry necessarily names the thing it reports fixed —
*"a top-level `install-hooks` that is a `git` subcommand"* — so checking it against the parser flags a
**description of a bug as if it were guidance**. A check that cries wolf on the project's own log is
one people learn to turn off, and that costs far more than the coverage is worth.

Five tests pin the discovery rules, including that `AGENTS.md` is checked first (the one file every
lab reads, so a wrong command there costs the most) and that archived, scratch and append-only
material stays out.

```
295 tests pass on 3.15 AND 3.9
python3 scripts/harness/cli.py check                 -> 45 rules, 0 errors
python3 scripts/harness/doccheck.py                  -> 96 commands across 913 documents, all exist
```

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record: docs/labs/harness-doccheck-2026-10-13/README.md, plus a current-state entry
- [x] Branch deleted after merge

### Stated rather than hidden

- **The coverage claim is bounded by the exclusions.** `docs/labs/` dated records could contain a
  command that no longer exists, and this will not tell you — deliberately, since those are records.
- Every tracked Markdown document is scanned per run. It is fast today; if the tree grows much
  larger this may need a cache, and nothing measures that yet.
- `doccheck` still cannot catch a **false description of a real command** — the fourth error that
  prompted it. It checks shape, not truth.
- Kimi's new rules have not landed on `main`; when they do I will run them and report rather than
  assume what they say.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

