# `team roster`: attempted, NOT delivered — and the three findings that make the next attempt cheap

**Date:** 2026-10-14 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** the first of the six remaining items on the measured scope

## This is not a delivery, and it is recorded as one

**I attempted `team roster`, did not get it verified, and reverted the wiring.** The repo is at
`b15ba7998` with **411 tests passing, clippy clean, and `team roster` still delegated.**

**Why revert rather than ship:** the command's output differed from Python's in ways I had found but
not yet corrected, **and a `roster` that lists nine roles in an order Python would not print is a wrong
answer that looks like a right one** — the class of defect this port has spent four units on.

**The library work was removed too**, because with the dispatch reverted **it has no caller** — and an
uncalled function that clippy is told to ignore is worse than no function.

## Finding 1: `serde_json` did NOT preserve insertion order

**The one worth keeping.** I read a JSON object as a map, reasoning that `serde_json`'s default
feature preserves insertion order. **The comparison disproved it:**

```
python order: architecture-reviewer, experiment-designer, test-methodologist, systems-cost
rust   order: architecture-reviewer, delivery-reviewer, evidence-auditor, experiment-designer
```

**`delivery-reviewer` before `experiment-designer` is alphabetical, not file order.**

**Why it matters beyond one command**: `docs/harness/roles.json` is a JSON **object keyed by id**, so
any code that walks it must decide its order deliberately. **Assuming the convenient behaviour is
exactly how this port produced two wrong constants earlier** — here it produced a plausible, wrong
roster.

## Finding 2: Rust `String` order is BYTE order, Python's is CODE-POINT order

With a sort restored, the order **still** differed, for a sharper reason: **Rust's `Ord for String`
compares UTF-8 bytes; Python compares code points.**

**For ASCII they agree — and all nine current ids are ASCII, so this would never show up in today's
data.** It would appear the moment a role id carried any non-ASCII character, **as a silent reordering
in one implementation and not the other.**

**The fix is explicit**: `a.id.as_bytes().cmp(b.id.as_bytes())` is what Python does — **and writing it
explicitly is the point, because the default `cmp` reads as correct and is correct only by coincidence
of the alphabet.**

## Finding 3: the doubled newline, for the third time

Python's `f"{n} dispatchable roles (…)\n"` through a writer that adds the line's own newline gives
**one** blank line. The port used `println!` with `\n` in the string (**two**), then `println!()` on
its own line (**also two**).

**This port has now produced a doubled newline three times** — the `lint format` body, the `check
--json` object, and this header. **Each time the fix was one character and each time only comparing
output found it**, never reading either implementation.

## What is known to work, for the next attempt

* **The shape**: `{schema, note, roles}`, where `roles` is an **object keyed by id** with `title`,
  `seat`, `route{provider,model,reasoning}`, `purpose`, `skills`, `when`, `read_only`.
* **`Role.route` renders `provider/model`, or `(inherit)` when there is no provider** — deliberate,
  **because a blank column would read as a missing value rather than as a decision to take the
  caller's route.**
* **`title` falls back to the id, `seat` to `specialist`**, so a minimal role still renders a full row.
* **Nine roles**; columns are `{id:<24} {seat:<11} {route:<32} {title}` with ` [read-only]` appended,
  then the purpose truncated to 100 characters.

## Verification of the state I left

| check | result |
|---|---|
| `git status` | clean; `main` = `origin/main` = `b15ba7998` |
| branches / worktrees | **0 / 0** |
| `cargo test -p uor-r4-harness` | **411 passed, 4 ignored** |
| `cargo fmt --check`, clippy | clean, 0 findings |
| `python3 scripts/check_no_softmax.py` | OK — 1641, baseline 1641 |
| **`delegate_to_python` sites** | **5 — unchanged; `team roster` still delegates** |
