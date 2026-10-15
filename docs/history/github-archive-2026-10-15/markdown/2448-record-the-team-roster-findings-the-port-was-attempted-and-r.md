# #2448 — Record the team roster findings; the port was attempted and reverted (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-14T05:45:51Z
- **Updated:** 2026-10-14T05:48:20Z
- **Closed:** 2026-10-14T05:48:02Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2448
- **Merged:** 2026-10-14T05:48:02Z
- **Merge commit:** c55a6c4e7d91862a77a557622d6e7a75cc76f9a0
- **Base / Head:** main / UOR-Foundation:deepseek/roster-findings
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

**`team roster`: attempted, NOT delivered — and the three findings that make the next attempt cheap.** Documentation only; **no code changed.**

Decision: **KEEP** (tooling).

## This is not a delivery, and it is recorded as one

**I attempted `team roster`, did not get it verified, and reverted the wiring.** The repo is at `b15ba7998` with **411 tests passing, clippy clean, and `team roster` still delegated.**

**Why revert rather than ship: a `roster` that lists nine roles in an order Python would not print is a wrong answer that looks like a right one** — the class of defect this port has spent four units on. **The library work was removed too**, because with the dispatch reverted **it has no caller.**

## Finding 1: `serde_json` did NOT preserve insertion order

I read a JSON object as a map, reasoning that `serde_json`'s default feature preserves insertion order. **The comparison disproved it:**

```
python order: architecture-reviewer, experiment-designer, test-methodologist, systems-cost
rust   order: architecture-reviewer, delivery-reviewer, evidence-auditor, experiment-designer
```

**`delivery-reviewer` before `experiment-designer` is alphabetical, not file order.**

**`roles.json` is an object keyed by id**, so any code that walks it must decide its order deliberately. **Assuming the convenient behaviour is exactly how this port produced two wrong constants earlier** — here it produced a plausible, wrong roster.

## Finding 2: Rust `String` order is BYTE order, Python's is CODE-POINT order

With a sort restored the order **still** differed: **Rust's `Ord for String` compares UTF-8 bytes; Python compares code points.**

**For ASCII they agree — and all nine current ids are ASCII, so this would never show up in today's data.** It appears the moment a role id carries a non-ASCII character, **as a silent reordering in one implementation and not the other.**

**The fix is explicit** — `a.id.as_bytes().cmp(b.id.as_bytes())` — **and writing it explicitly is the point, because the default `cmp` reads as correct and is correct only by coincidence of the alphabet.**

## Finding 3: the doubled newline, for the third time

Python's `f"{n} dispatchable roles (…)\n"` through a writer that adds the line's own newline gives **one** blank line. The port used `println!` with `\n` in the string (**two**), then `println!()` alone (**also two**).

**This port has now produced a doubled newline three times** — the `lint format` body, the `check --json` object, and this header. **Each time the fix was one character and each time only comparing output found it.**

## What is known to work

The roles shape; **`Role.route` renders `provider/model` or `(inherit)` when there is no provider** — deliberate, **because a blank column would read as a missing value rather than as a decision to take the caller's route**; `title` falls back to the id and `seat` to `specialist`; nine roles; and the exact table columns.

## Verification of the state I left

| check | result |
|---|---|
| `main` = `origin/main` = `b15ba7998`, tree clean | yes |
| branches / worktrees | **0 / 0** |
| `cargo test -p uor-r4-harness` | **411 passed, 4 ignored** |
| `cargo fmt --check`, clippy | clean, 0 findings |
| **`delegate_to_python` sites** | **5 — unchanged; `team roster` still delegates** |

Records: `docs/labs/roster-port-findings-2026-10-14/README.md`, `docs/integration/current-state.md`

## Checklist

- [x] Documentation only; no code, no capability, no headline
- [x] **The non-delivery is stated in both records and in the current-state entry**
- [x] Branch deleted after merge

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

