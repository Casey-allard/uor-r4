# ORDER: infra-loop-pr08-migration, kimi-11 (repair: the counted CHANGES at e05f9fee)

Lab: kimi. Item: [infra-loop-pr08-migration](../../../../roadmap/items/infra-loop-pr08-migration.md) (#2913).
Setup row: 8. Base: `e05f9feeb`.
Parent: [kimi-10/ORDER.md](../kimi-10/ORDER.md).

## Owner approval

**"ok well I approve of #1"** — Casey in the kimi chat, received 2026-10-22T16:49Z, assigning PR 8 to kimi and approving this PR's changes to the owner paths `AGENTS.md` and `docs/integration/DECISIONS.md` (owner-paths.json). Carried unchanged from kimi-9 (the chain rule keeps Lab/Item/Setup row and this section identical).

## Goal

Repair after the counted CHANGES records at `e05f9feeb` (claude and deepseek, 22:5xZ), which
found the pre-review findings still open at the final head. Six repairs, all in scope:

1. **STATE renderer truthfulness** (`looprender.rs:724-737`): a missing `current` maps to
   "none", so STATE.md:15 reads "softmax at runtime: none of target 0" while the registry text
   says "yes" — a reader would take the D23 runtime bar as met. Render the bar's `text`
   faithfully when `current` is absent, carry the qualifiers STATE drops (M1 SUSPENDED/R14,
   M2 `direction: open`, M3/M5–M8 vacuous), and qualify the 437-of-256 line with its run and
   split against the confirmed `m2-fresh256` bar (#2030).
2. **Rendered-lock both-way tests** (`tests/loop_prcheck.rs`): the first-commit pass is
   untested (28/28 pass with it disabled). Add: views absent at base + added on a non-render
   branch → PASS; one view present at base + edited → FAIL.
3. **Six `migrate --check` negative tests** (`tests/loop_migrate.rs`): pointer missing,
   L-id/file-stem mismatch, L-001 missing, DECISIONS columns, read-path budget, FILEMAP row
   naming a gone path — each must fail the verifier when broken.
4. **PLAN.md** (authored, PM): every Premises row carries a runnable backticked probe (rows
   1/4/7 lack backticks; rows 2/3 backtick item ids, which `loop next` would dispatch as
   commands); the Route names M5 and M6 (HANDOFF's `m8-release-packaging` depends on them).
5. **Catalog rows** (`archive/catalog.jsonl`, regenerated pages): the two new rows point
   `path` at the pointer stubs, carry `"prs"` as strings (renders empty) and Python-style
   spacing. Point each at its archive copy, use integer `prs`, repo-style spacing, and
   re-render with `xtask docs-catalog`. (The rows are new in this PR, so correcting them stays
   inside the owner's append-only rule; after merge they are frozen.)
6. **`docs/harness/gates.json`**: restore the 2-space formatting; keep only the substantive
   `loop-migrate` change (whitespace-only churn 291/253 → the real ~61/23).

The figures-timeline window (`xtask/src/figures_charts.rs`, `docs/figures/**`) is out of this
ORDER's scope: it rides the PM-decided widening (#2028, 6353263335) and then DeepSeek's
validated patch, as a separate commit sequence in this PR.

Lower-priority observations from the same reviews, fixed where cheap: a unit test for the
renderer re-rooting (`mailto:` must not be rewritten); `migrated_links` naming the view it
skips; the loop-migrate gate's planted fault matching the loop page (a planted broken link in
PLAN.md turning `xtask links` red).

## Files in scope

As kimi-9/kimi-10 (unchanged item scope).

## Done when

- Every numbered finding in the two CHANGES records at `e05f9feeb` is answered at the head,
  with the new tests failing under their fault and passing fixed.
- `loop gates --full` GREEN, `--break all` CAUGHT, `migrate --check`, `pr-check --local`,
  `report --check`, `render --check` PASS; receipts re-bound at the final head.
- The docs CI job is green (after the timeline widening lands) or its sole red is again an
  out-of-scope carded gate.

## Stop and ask if

- The scope widening for the figures pin is refused or redirected to the owner.
