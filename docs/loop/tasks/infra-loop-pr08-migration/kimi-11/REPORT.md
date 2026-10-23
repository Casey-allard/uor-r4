# REPORT: kimi-11, infra-loop-pr08-migration

Lab: kimi. Order: [ORDER.md](ORDER.md).

## What did not work, first

- A re-review was requested at `84b10704e` before the repair commits were all in; the
  counted CHANGES records (claude and deepseek at `e05f9feeb`) landed against that state.
  The repair order kimi-11 was written against those records instead of arguing them.
- The first post-repair fmt pass left `fmt-harness` RED: the `mailto:` scheme guard in
  `looprender.rs` was not rustfmt-clean. Fixed in `7a640ce42`; the gate is GREEN since.
- The first `--break all` attempt at the merged head hit its own wall-clock timeout mid-run
  and had to be re-run; the re-run below is the one recorded here. (Harness quirk, not a
  gate failure: the run simply takes longer than the first allowance.)
- `loop close --verify` still prints its PASS lines and then hangs idle with no child
  processes (seen twice this session; killed via timeout each time). The verification
  itself completes; noted on #2904. No harness change in this order.

## What changed

Six repairs against the counted CHANGES at `e05f9feeb`, plus the cheap lower-priority
fixes the same reviews named:

1. **STATE renderer truthfulness** (`looprender.rs`): when a bar has no `current` reading,
   the renderer now carries the bar's registry `text` and its qualifiers instead of
   printing a bare "none" that read as "bar met". STATE.md keeps the M1 SUSPENDED/R14
   note, the M2 `direction: open`, the M3/M5–M8 vacuous markers, and the 437-of-256 line
   is qualified with its run and split against the confirmed `m2-fresh256` bar.
2. **Rendered-lock both-way tests** (`tests/loop_prcheck.rs`, +39): views absent at base
   and added on a non-render branch PASS; one view present at base and edited FAILS.
3. **`migrate --check` negative tests** (`tests/loop_migrate.rs`, +111): seven tests —
   pointer missing, L-id/file-stem mismatch, L-001 missing, DECISIONS columns, read-path
   budget, FILEMAP row naming a gone path — each fails the verifier when its fault is
   planted and passes clean.
4. **PLAN.md** (authored, PM): every Premises row now carries a runnable backticked probe;
   the Route names M5 and M6, which HANDOFF's `m8-release-packaging` depends on.
5. **Catalog rows** (`archive/catalog.jsonl` + regenerated `archive/catalog/*.md`): the
   two new rows point `path` at their archive copies, carry integer `prs`, and use
   repo-style spacing; re-rendered with `xtask docs-catalog`.
6. **`docs/harness/gates.json`**: restored 2-space formatting so the diff is the
   substantive `loop-migrate` change only (~61/23 instead of 291/253 whitespace churn).

Lower-priority fixes from the same reviews: a unit test that rerooting never rewrites a
`mailto:` link; `migrated_links` names the view it skips; the loop-migrate gate's planted
fault (a broken link in PLAN.md) matches the loop page.

Size: 10 files, +592/-318 against `e05f9feeb` — roughly 190 lines of tests, ~80 of
renderer code, ~290 of reindented gates.json (substantive part ~84), the rest docs and
catalog. The ORDER carried no numeric estimate; the repair stayed inside the item scope
it declared.

## Measurements

Nothing measured; this order changes harness code, tests and docs only.

## Checks run at the head

At `383a8d1fe` (repair commits plus a clean merge of origin/main carrying #3019, #3021,
#3022 — the timeline window that had been blocking the docs CI job):

- `cargo test -p uor-r4-harness --release --test loop_migrate --test loop_prcheck` —
  13/13 and 29/29 pass. The new negative tests fail with their fault planted and pass
  fixed (verified during kimi-11 development).
- `uor-harness loop gates --full` — 11/11 GREEN (fmt-claim-wording, fmt-harness,
  claim-wording, loop-gates-watch, loop-drive, loop-next-sources, loop-next-wait,
  loop-next-owner-direction, git-hooks, loop-migrate, loop-review-changes-blocks).
- `uor-harness loop gates --break all` — 11/11 CAUGHT (each gate fails when its planted
  fault is applied and recovers after).
- `uor-harness loop render --check` — PASS at the merged head (the four rendered views
  committed here are byte-current).
- `uor-harness loop report --check`, `loop pr-check --local`, `loop migrate --check` —
  PASS at the head before push (pr-check's `scope` row passes since scopes landed).
- Gate receipts re-bound at the final head with `uor-harness loop report --receipts`.

## Skills used

None.

## Still open

- The owner's M1-headline card and six bar cards on #2028 remain the owner's to answer;
  the m1 `"suspended": true` flag in `docs/harness/milestones.json` records the R14
  ruling only.
- The `loop close --verify` idle hang after its PASS lines (noted on #2904).

## Plain summary

This task is the repair round for the big "notes migration" pull request. Two reviewers
read the earlier version and listed specific problems: a status page could make an unmet
bar look met, some new checks had no tests proving they actually catch faults, a planning
document's probe commands were not runnable, two archive-catalog rows were malformed, and
one data file had been reformatted so its real change was buried in whitespace churn.
Each problem is fixed, and each new check now has a test that proves it fails when the
fault it guards against is present and passes when the tree is clean. All eleven quality
gates pass normally and all eleven are caught when deliberately broken, and the rendered
status pages are regenerated and committed.
