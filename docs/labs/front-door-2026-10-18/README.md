# Front door PR-C: README hero, figure badges, templates and good-first-issues — 2026-10-18

Course-correction Phase 2 (front door), one PR. Base `50d63c258` (#2677); branch
`kimi/pr-c-front`. Documentation, assets, GitHub templates and one `xtask` figure source:
no model compute, no evidence change, $0.

## Change

- `README.md`: the status block is the front-door audit's ~110-word hero (§1.2), with the pod ID,
  waiver comment ID, UTC timestamp and admission counts left to `STATUS.md`, where they already
  live. Adds a "What works today, and what does not" table, a contents line after the badges, and
  `docs/figures/serving-pipeline.svg` embedded under the Serving (M4) boundary.
- `xtask/src/figures.rs`: a `HISTORICAL · INACTIVE` status. `geometry-stack.svg` (all 12 rows,
  H1 "the historical float form") and `compliant-path.svg` (its 3 pills) use it; `operators.svg`
  is now "the three geometric operators of the earlier float trunk (inactive)"; `milestones.svg`
  M1 reads "Dose2 stopped at Q1; admission collapse". Regenerated with
  `cargo run -q --locked -p xtask -- figures --write`.
- `assets/images/hero_banner.jpg` → `historical-concept-art-2077.jpg` (`git mv`), and the
  directory caption now names it as invented-2077 concept art. No live page embeds it.
- `.github/ISSUE_TEMPLATE/config.yml` (NOW.md, OUTSIDE.md, HARD-RULES.md, #2028) and a light
  `question.yml`; `.github/pull_request_template.md` gains the outside-contributor checklist.
- Five `good first issue`s seeded: #2679 (PROJECT_MAP `ops/` + heading spacing), #2680 (ROADMAP
  D32 → D37), #2681 (ROADMAP M2/M3/M4 vocabulary), #2682 (core README "Active native model"),
  #2683 (geometry.md status wall).

## Notes and deviations

- The audit says every node of `compliant-path.svg` carries `BUILT & TESTED`; only three pills
  exist. All three are the dormant float-source cards and all changed; the other cards carry no
  pill.
- The audit's good-first examples included correcting the merged #2654 body and annotating
  #2655. Both need maintainer custody knowledge rather than a newcomer's first PR, so the seed
  uses the core README contradiction (contributor audit FIX-10) and the `docs/geometry.md` status
  duplication (front-door §2.7) instead.
- Outside this PR's item list and not applied here: the README rules-table D35/D36 rows (audit
  §6.6) and the `docs/README.md` / `docs/PROJECT_MAP.md` splits (§6.7). The ROADMAP milestone
  table and D32 paragraph remain stale on purpose: they are #2680 and #2681.

## Verification

- `python3 scripts/check_claim_wording.py` and `cargo run -q --locked -p uor-r4-claim-wording -- .`
  pass.
- `cargo fmt --check -p xtask` passes; `cargo run -q --locked -p xtask -- figures` reports all 8
  generated figures current.
- `grep -rn hero_banner` leaves only the dated audit/review records and the rename note.
