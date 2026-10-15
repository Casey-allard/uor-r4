# #1806 — mqar-bench: land the parameter-matched ordinary control on main

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-06T22:17:10Z
- **Updated:** 2026-10-09T13:19:35Z
- **Closed:** 2026-10-06T22:29:56Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1806
- **Merged:** 2026-10-06T22:29:56Z
- **Merge commit:** ae7808426b9e5b543dff52f6d53310e9a7cae053
- **Base / Head:** main / UOR-Foundation:deepseek/control-arm-to-main
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Carries `5a37b981` ("mqar-bench: ordinary parameter-matched control arm (D20 s2 conditions 1 and 3)", *Casey Allard*, 6 Oct 00:42) onto current `main` by cherry-pick **`-x`**, authorship preserved.

## Why

**D20 §2 condition 1 is not runnable from `main`.** The comparator behind every C6 number — **0.2534 in-class, the control in the verdict** — lives only on `deepseek/d20-control`, and `git grep 'mlp=matched' origin/main` returns zero hits. The verdict cites a comparator a trunk reader cannot execute.

## The conflicts, and why they were mechanical

Three hunks, all resolved as **union**:

| hunk | resolution |
|---|---|
| 1 — doc comment | main's lineage list (`conv8\|carrier\|rot`) **+** the branch's `arm=transformer` docs |
| 2 — 58 lines at the same insertion point | main's `model_pre_save`/`stage0` block (37 lines) **+** the branch's `read_firing`/`context_reachability` counters (21) |
| 3 — JSON record | `"init"` **+** `"parameter_match"` |

**No design decision was required.** The hunk-2 conflict looked semantic but is not: `grep -nE '^fn '` shows main's bench defines `stage0_tally_dump`/`stage0_sieve` while the branch's defines `read_firing`/`context_reachability` — **different functions, added independently at the same line.** Both sides are kept.

Plus **two `StackArch::Hybrid` label arms**, because the branch predates #1798 and the enum gained a variant in between.

## Verification

- `cargo check -p uor-r4-training --example mqar-bench --offline` — **clean** (2 pre-existing warnings)
- Release build, then the original recipe (`lr=0.001, layers=6, width=128, heads=4, mlp=matched, match_pattern=rrarra`): **step 600 → in-class 0.250**, against the original run's 0.2529/0.2510/0.2563 final. Confirming the seal.

## Notes

This is a **relocation, not a new measurement** — the arm's behaviour is expected to be unchanged, and the check above is what establishes that. `arm=transformer` also now accepts `StackArch::Hybrid` labels for the read-attribution control added in #1798.

References #820.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

