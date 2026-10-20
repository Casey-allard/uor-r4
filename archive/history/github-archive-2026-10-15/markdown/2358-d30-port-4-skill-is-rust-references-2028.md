# #2358 — D30 port 4: skill is Rust (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T19:05:58Z
- **Updated:** 2026-10-13T19:11:57Z
- **Closed:** 2026-10-13T19:11:33Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2358
- **Merged:** 2026-10-13T19:11:33Z
- **Merge commit:** f2f7246439db13afdefbe63962504a05ae6b1b6d
- **Base / Head:** main / UOR-Foundation:deepseek/port-skill
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

**D30 port 4.** Tooling only.

Decision: **KEEP** (tooling).

## What moved

`crates/uor-r4-harness/src/skills.rs` — new, **10 tests**. `uor-harness skill list|show|match|use` is Rust and **out of the delegated set**.

`skill use` is the one labs invoke every session — it writes **one receipt line** saying a skill was actually loaded. Verified against the real registry and the real write path:

```
$ uor-harness skill use context-hydration --milestone m1 --lab deepseek
skill used: context-hydration (docs/harness/ledger/deepseek-2026-10.jsonl)

{"kind":"skill","lab":"deepseek","milestone":"m1","outcome":"started","schema":1,
 "skill":"context-hydration","ts":"2026-10-13T19:04:09Z"}
```

## The decision that shapes it

**An unknown skill refuses rather than recording.** A receipt naming a skill that does not exist is **worse than no receipt** — the record would claim a skill was used when nothing was loaded. Python does the same, and the test asserts it.

The three id forms are kept because labs have already typed all three: **exact id, exact name, then a unique prefix**. A prefix matching more than one resolves to the first in sorted order — Python's behaviour, and changing it would make the two implementations disagree on a command run every session.

**No YAML dependency:** frontmatter is a flat `key: value` block with `[a, b]` lists, parsed directly. A nested value is reported as unparsed rather than silently dropped.

## A bug in the glob matcher

`applies-to` patterns are globs. The first version stripped a trailing `*` and treated the rest as a prefix — so **`docs/mechanisms/*.md` matched every file under `docs/mechanisms/`, `.md` or not** — and its successor required the literal remainder as a *prefix*, matching nothing at all. **The pattern has to be split on `*` into a prefix and a suffix, and both checked.**

## Against the real registry

The test loads the **actual** `docs/harness/skills/`, asserts every skill has an id and a description, and that **every id resolves to itself** — so a malformed skill file fails in CI rather than in a lab's session.

```
cargo fmt --check                               -> clean
cargo clippy --all-targets -- -D warnings       -> 0 findings
cargo test -p uor-r4-harness                    -> 182 passed (10 new)
python3 -m unittest discover -s tests/harness   -> 319 passed
uor-harness check                               -> 0 errors
skill list vs Python                            -> both report 9 skills
unknown skill                                   -> both refuse
```

Record: `docs/labs/port-skill-2026-10-13/README.md`

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record + current-state entry carried
- [x] Branch deleted after merge

### Stated rather than hidden

- **`mistakes` is not ported.** It has its own registry, matcher and a `capture` writer that writes a **lesson file plus a guard** — a bigger surface than `skill`, and it is next.
- **`skill list` output differs from Python's.** Rust adds a version column and drops the truncation; same skills, different layout, so a script parsing the old columns breaks.
- **`--files` accepts comma-separated values**, which Python's argparse does not. A superset, but a difference.
- **`skills::load` reads the directory on every call** — fine once per command, not on a hot path.
- **The matcher is a single-`*` glob, not a full glob engine.** `**` or a character class is treated as a literal, silently.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

