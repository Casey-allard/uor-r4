# D30 port 4: `skill` is Rust

**Date:** 2026-10-13 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** the D30 order on #2028; port 4 of the series

## What moved

`crates/uor-r4-harness/src/skills.rs` — new, **10 tests**. `uor-harness skill list|show|match|use` is
Rust and **out of the delegated set**.

`skill use` is the one labs invoke every session: it writes **one receipt line** saying a skill was
actually loaded and deployed. Verified against the real registry and the real write path:

```
$ uor-harness skill use context-hydration --milestone m1 --lab deepseek
skill used: context-hydration (docs/harness/ledger/deepseek-2026-10.jsonl)

{"kind":"skill","lab":"deepseek","milestone":"m1","outcome":"started","schema":1,
 "skill":"context-hydration","ts":"2026-10-13T19:04:09Z"}
```

## The decision that shapes it

**An unknown skill refuses rather than recording.** A receipt naming a skill that does not exist is
**worse than no receipt** — the record would claim a skill was used when nothing was loaded. The
Python harness does the same, and the test asserts it.

The three id forms are kept because labs have already typed all three: **exact id, exact name, then a
unique prefix**. A prefix matching more than one resolves to the first in sorted order, which is
Python's behaviour — changing it here would make the two implementations disagree on a command run
every session.

**No YAML dependency.** Frontmatter is a flat `key: value` block with `[a, b]` lists, parsed directly.
A nested value is reported as unparsed rather than silently dropped.

## A bug in the glob matcher

`applies-to` patterns are globs. The first version stripped a trailing `*` and treated the rest as a
prefix — so **`docs/mechanisms/*.md` matched every file under `docs/mechanisms/`, `.md` or not** — and
its successor required the literal remainder `.md` as a *prefix*, matching nothing at all. The pattern
has to be split on `*` into a prefix **and** a suffix, and both checked. Tested for the glob case and
the plain-suffix case.

## Against the real registry

The test loads the **actual** `docs/harness/skills/`, asserts every skill has an id and a description,
and that **every id resolves to itself** — so a malformed skill file fails here rather than in a lab's
session.

```
cargo fmt --check                               -> clean
cargo clippy --all-targets -- -D warnings       -> 0 findings
cargo test -p uor-r4-harness                    -> 182 passed (10 new)
python3 -m unittest discover -s tests/harness   -> 319 passed
uor-harness check                               -> 0 errors
skill list vs Python                            -> both report 9 skills
unknown skill                                   -> both refuse
```

## Limitations, stated

- **`mistakes` is not ported.** It has its own registry, matcher and `capture` writer, and `capture`
  **writes a lesson file plus a guard** — that is a bigger surface than `skill`, and it is next.
- **`skill list` output differs from Python's.** Rust adds the version column and drops the truncation;
  same skills, different layout, so a script parsing the old columns breaks.
- **`--files` on `skill match` accepts comma-separated values**, which Python's argparse does not. A
  superset, but it is a difference.
- **`skills::load` reads the directory on every call.** Called once per command, so it does not matter
  today; it would if this ever lands on a hot path.
- **The `applies-to` matcher is a single-`*` glob, not a full glob engine.** A pattern with `**` or a
  character class is treated as a literal, silently.
