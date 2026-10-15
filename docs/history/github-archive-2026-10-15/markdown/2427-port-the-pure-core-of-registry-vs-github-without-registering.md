# #2427 — Port the pure core of registry_vs_github, without registering it (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-14T03:25:33Z
- **Updated:** 2026-10-14T03:36:22Z
- **Closed:** 2026-10-14T03:35:57Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2427
- **Merged:** 2026-10-14T03:35:57Z
- **Merge commit:** fa49e9e18bb7cc5db7b64b90e43dba884b5cf11b
- **Base / Head:** main / UOR-Foundation:deepseek/github-registry
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

**The pure core of `registry_vs_github` — ported, tested, and deliberately not registered.** Tooling only.

Decision: **KEEP** (tooling).

## What landed

**`githubcards.rs`** — `_newest_card`, `_card_reading` and `_registry_numbers`: **the part of the rule that decides anything.**

**Coverage does not move: 51 of 54.** That is the point of this unit.

## Why only the core, and why it is not registered

The rule is ~135 lines and **three of its parts need a network**: a cache under the state directory, one `gh issue view` per milestone, and comment pagination.

**I did not port those, so the rule stays in the Python set.** A half-ported rule claimed as coverage would be **the silent downgrade this port refuses** — the same reason `harness.documented_commands` is unregistered after I declined to approximate it.

**What is ported is where a bug would live**, and it **can be compared against Python without a network** — the only way to verify it honestly here.

## The capture corrected me on the strict form

The docstring calls `M<n>: <n>/<n>` the *"primary form"*, and I built a case asserting `"M1: 37/232 here"` reads as `milestone_number`.

**The capture returned `no_card`** — `_newest_card` requires a **card marker** (`status card`, `blocked card`, `pivot card`, `headline`) *before* `_card_reading` looks at the body at all.

**So the strict form is a refinement of a card, not a way to find one** — a distinction the docstring does not draw and that **only the capture made visible.** Both readings are now asserted.

## Details ported rather than improved

- **Sorting is by `createdAt` as a STRING**, reversed. A missing timestamp sorts last as the empty string; sorting by parsed time would reorder malformed timestamps — **and the order decides which card is read.**
- **The 120-character window is taken in CHARACTERS**, because a card quoting non-ASCII would slice mid-character — **the char-boundary panic this port has hit twice.**
- **`registry_numbers` reads `secondary` rows and skips incomplete ones** rather than reading a missing number as zero.

```
cargo test -p uor-r4-harness                    -> 384 passed (5 new)
cargo fmt --check / clippy -D warnings          -> clean / 0 findings
python3 -m unittest discover -s tests/harness   -> 332 passed
python3 scripts/check_no_softmax.py             -> OK — 1641, baseline 1641
parity.py                                       -> exit 0, 51 implemented — unchanged, deliberately
```

Record: `docs/labs/card-reading-port-2026-10-14/README.md`

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record + current-state entry carried, presence asserted before commit
- [x] Branch deleted after merge

### Stated rather than hidden

- **`check` still delegates, and this unit does not move the count.** Two rules remain: `harness.documented_commands` and `status.registry_vs_github` — plus `prlint.parse`.
- **The `gh` chain is untouched** — cache, calls, pagination, **and the decision that follows the reading.** So **only the reading is ported, and the ported code is a helper with no caller**, reachable from tests and nothing else.
- **`newest_card` clones every comment it inspects.** Python sorts in place; **the port copies bodies — fine at eight comments, wasteful at eight hundred**, and the rule runs per milestone.
- **A card whose marker appears after 120 characters reads as `headline_text` with no number**, since the window starts at the marker. **That is Python's behaviour and it is why the labs' cards put `headline` early** — a convention the code depends on and does not document.

References #2028


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6160268151 — 2026-10-14T03:32:31Z — Casey-allard

Superseded by the correction PR: this PR's CI failure is a test I wrote asserting an exact document count. The fix and the record are in the correction PR; this branch's ported code is unchanged there.


---
_Comment/review ids in this file: 6160268151_

