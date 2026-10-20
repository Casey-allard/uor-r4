# #2382 — D30 check-port 6 of 10: the arena rules (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T21:35:12Z
- **Updated:** 2026-10-13T21:39:20Z
- **Closed:** 2026-10-13T21:39:01Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2382
- **Merged:** 2026-10-13T21:39:01Z
- **Merge commit:** 4cb76c74a682c09a492e3c992f1e5985a834b4a9
- **Base / Head:** main / UOR-Foundation:deepseek/arena-rules
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

**D30 check-port 6 of 10.** Tooling only.

Decision: **KEEP** (tooling).

## All four `arena` rules, exact agreement

| rule | severity | Python | Rust |
|---|---|---|---|
| `arena.rows_valid` | error | all pass | 0 findings |
| `arena.parents_exist` | error | all pass | 0 findings |
| `arena.decisive_power` | warn | all pass | 0 findings |
| `arena.knows_what_it_cannot_measure` | warn | all pass | 0 findings |

Coverage reads **22 of 40 rules** (arena 4, harness 3, mistakes 2, multilab 6, prep 5, status 2).

## The two that carry the weight

**`decisive_power` turns D22's minimum into a check.** A row asserting `decisive` confidence below the declared floor — two independent repeats, or two hundred fresh rows — is a provisional reading **wearing a decisive label, and the label is what a reader trusts.** The rule cannot tell whether the work happened; it can tell whether the claim is *entitled*, which is what a machine can decide.

**`knows_what_it_cannot_measure` is the one that ages best.** A row list with no stated limits **reads as complete.** **Forty characters, because a shorter one is a gesture** — `"none"`, `"see docs"`, `"TBD"` each satisfy "there is a limits field" while stating nothing. All four are tested as rejections.

## Three decisions worth naming

**An absent arena is reported, not passed.** `load` returns an empty arena for a missing file, which would make every rule report nothing — **and silence reads as a pass.** `rows_valid` reports it as its own case.

**The duplicate-id check runs first, because it invalidates everything after it.** With two rows of one id, every lookup resolves to whichever was read first.

**A cycle is reported once per entry point** — the Python's behaviour, kept, because reporting it once would **hide the second row leading into the cycle, which is the row a reader has to fix.** The walk is explicitly tested for termination, since a cycle is what a naive parent walk hangs on.

```
cargo fmt --check                               -> clean
cargo clippy --all-targets -- -D warnings       -> 0 findings
cargo test -p uor-r4-harness                    -> 267 passed (12 new)
python3 -m unittest discover -s tests/harness   -> 319 passed
parity.py                                       -> exit 0, rule inventory: 22 implemented
```

Record: `docs/labs/port-check-arena-2026-10-13/README.md`

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record + current-state entry carried
- [x] Branch deleted after merge

### Stated rather than hidden

- **`check` still delegates.** 6 of 10 modules.
- **`rows_valid` reads `candidates` only.** A renamed key reads as an empty arena and reports *"missing or has no rows"* — **wrong in the loud direction, but wrong.** Python has the same coupling.
- **`limits` is measured in characters, not meaning.** A forty-character string of filler passes — **the rule can require someone to write something; it cannot require them to mean it.**
- **`decisive_power` trusts the row's own `seeds` and `rows` counts.** A recorded count of two hundred rows that never happened passes: **it checks the claim's arithmetic, not its truth.**
- **The agreement measurement is a hand-run probe.** Nothing in CI compares per-rule counts — **and this probe has now caught one real bug** (`guard_covered`, last unit).

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

