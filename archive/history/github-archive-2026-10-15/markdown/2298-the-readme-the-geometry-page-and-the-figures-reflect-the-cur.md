# #2298 — The README, the geometry page and the figures reflect the current path (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T05:20:28Z
- **Updated:** 2026-10-13T05:23:19Z
- **Closed:** 2026-10-13T05:22:51Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2298
- **Merged:** 2026-10-13T05:22:51Z
- **Merge commit:** 3778e1d6d5f17611ab5a13929c13a4ffdf912ce9
- **Base / Head:** main / UOR-Foundation:claude/readme-geometry-2026-10-13
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Owner request: redo the README and the geometry page, with their pictures and the mechanisms, so they reflect the current path, and make the README read as a presentable research project with good bookkeeping.

Line: no model line (documentation) · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged). References #2028.

## What changed

- **README.md** (518 → 302 lines). Always visible:
  - question and architecture;
  - milestones M1–M8 with headline numbers;
  - the current path, as an operator-phase checklist;
  - rules D11 and D22–D27;
  - mechanisms with status;
  - scoped key results;
  - a bookkeeping table;
  - how the project is run (PM R9 roster);
  - quick start.

  In `<details>`: all results, negatives and retractions; models and data; the repository map; statistics. Pre-D23 float-form results are labelled *historical*.
- **docs/geometry.md** leads with the compliant next-token path (rung 0's executed path) and the four trunk operators: the G2 Hamilton gate, the SpiralCore/FBS projection, the geometric recurrence gates and the rank-margin objective. It states what is missing: the head (OP3a), device qualification and the read layers. Math sections 1–10 are kept, with status lines corrected:
  - §2: the geometric gates;
  - §7: the softmax route mix is marked historical;
  - §8: the finished VSA test;
  - §9 and §10: SpiralCore's signed action is now the trunk projection (#2279).
- **docs/history/geometry-float-form-2026-10.md**: the complete previous page, preserved intact with re-pointed links.
- **xtask figures** (D27): the seven status figures are generated in Rust. Check mode fails on a stale SVG, and an overflow guard refuses labels wider than their box. Regenerated: `architecture`, `milestones`, `training-pipeline`, `serving-pipeline`, `geometry-stack`. New: `geometry/compliant-path`, `geometry/operators`.
- **docs/mechanisms/README.md** gains a Status column, checked against each brief.
- **docs/bookkeeping.md** (new, *proposed* for the PM's plan): maps each README surface to its source of truth and to what changes it, sets five rules, and splits the work between labs, the PM and a README keeper.
- Record: `docs/labs/readme-geometry-2026-10-13/README.md`; a current-state entry; the branch claim and the Claude ledger shard.

## Checks at this head

| Check | Result |
|---|---|
| `cargo run -p xtask -- figures` | `all 7 generated figures are current` |
| negative: append to `milestones.svg`, then run figures | `gate failed: stale generated figures … milestones.svg`, rc 1; passes again after restore |
| `cargo clippy -p xtask -- -D warnings` | rc 0, no `allow` attributes |
| `cargo fmt -p xtask --check` | rc 0 |
| `python3 scripts/check_claim_wording.py` | passed |
| `python3 scripts/check_no_softmax.py` | OK 1649 (baseline 1649) |
| `python3 scripts/check_no_matmul.py` | OK 93 (baseline 93) |
| `python3 scripts/harness/cli.py check` | rc 0. Warnings: pre-existing panel/orphan items, plus `branch_claimed`, because the checker reads the claim from the main checkout (see below) |
| local link and anchor check over the six changed pages | 0 missing |

All seven figures were also inspected rendered in a browser.

## For the harness owner (DeepSeek)

`team claim` writes `docs/harness/claims.json` into the **main checkout** (`multilab.main_worktree`). Here the main checkout is the owner's `~/uor-r4`, which should stay clean. I reverted that write and put the claim in this PR instead, as R1 says. The status check reads the main checkout, so it keeps warning until this merges.

## Not done here

- Porting `docs/figures/geometry/generate.py`, the math figures, to Rust.
- Wiring `xtask figures` into the PR checks.

Both are listed as follow-ups in `docs/bookkeeping.md`.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 6138836595 — 2026-10-13T05:20:45Z — Casey-allard

APPROVE at exact head `6966bdd8a4bc87cb0b5d16e3f1aaa2611bc88404` (Claude lab self-review, owner merge rule of 30 September). Checks run at this head are in the PR body: figures current (plus the stale-SVG negative), clippy `-D warnings` with no allows, fmt, claim wording, no-softmax 1649/1649, no-matmul 93/93, harness check rc 0, local link and anchor check 0 missing. Docs and the xtask generator only; no model code, no compute.

### comment 6138851733 — 2026-10-13T05:22:03Z — Casey-allard

APPROVE at exact head `cd5d1523cb95b32a365e4ba8ba037f347d5a7e0c` after rebasing onto #2296 (`c1ba70de2`). Conflicts were in `claims.json` (main's version plus this branch's claim) and `current-state.md` (both entries kept, newest first). Re-run at this head: figures current, clippy `-D warnings` rc 0, no-softmax 1649/1649, no-matmul 93/93, claim wording passed, harness check rc 0.


---
_Comment/review ids in this file: 6138836595, 6138851733_

