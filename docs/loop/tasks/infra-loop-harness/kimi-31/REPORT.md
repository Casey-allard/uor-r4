# REPORT: infra-loop-harness, kimi-31

Lab: kimi. Order: [ORDER.md](ORDER.md). Setup row: 2 (item metadata). `References #2904`.

## What did not work, first

- The first metadata edit failed `xtask roadmap check`: the M2 and M6 notebook files did not exist. Creating them exposed a bootstrap problem: the scope guard rejected a first notebook page before its binding was allowed.
- The first guard repair handled only the ORDER item's new binding. The real PR then failed for M2/M6 because those pages belong to other roadmap items. I added the exact cross-item regression and generalized the guard to collect first bindings across all changed item files. Existing out-of-scope and row-rewrite refusals remain green.
- One `pr-check` invocation used the main-built harness and repeated the old scope failure. Building this PR's harness and invoking that binary passed scope; this was a verification-binary mistake, not a third rule change.
- No status, bar, hold, dependency, research result or PLAN content changed. M2/M6/M1-position acceptance remains unfinished as recorded in #2028 comment 6374744852; the D11 premise remains open. #3087's owner fixture decision is separate and unchanged.

## What changed

- Bound M2 grounded reply and M6 reasoning/coding to their new notebook scaffolds; bound M1 position and M1 close to `m1-discrete-attention.md`, and M4 export/likelihood to `m4-serving.md`. Added `PLAN.md` to the loop-harness scope for later separately reviewed PM premise records.
- Added empty M2/M6 notebook scaffolds only. No research result rows or capability claims.
- Updated the shared scope guard to accept a new page introduced with its first binding, including bindings in other changed item files. Added same-item and cross-item regressions.
- Size: tests 1 file +60/-0; code 1 file +18/-0; docs 11 files +126/-1 (including this REPORT). Total 204 additions/1 deletion against the ORDER's about-140-line estimate: **1.46x**.

## Measurements

- `cargo test -p uor-r4-harness --test loop_prcheck scope_allows_a_new_notebook`: same-item RED then GREEN; cross-item RED then GREEN. Full scope suite: **47 passed, 0 failed**.
- `xtask roadmap check`: **91 items OK** (ready 33, held 26, decision 4, done 28). `roadmap render`: no diff. `roadmap sync` dry-run: **nothing to change** (0 labels/issues/updates/closes/checklists).
- Host-only, no GPU, no model evaluation and no capability measurement.

## Checks run at the head

- `uor-harness loop gates --quick`: 3/3 GREEN at `a5305edde`.
- `uor-harness loop gates --full`: 11/11 GREEN at `50ee24b90` before the cross-item repair and again at `3c2123fed` after it and the merge of current main.
- `cargo fmt -p uor-r4-harness -- --check`: PASS. `git diff --check`: PASS.
- PR-built `uor-harness loop pr-check --local`: scope PASS at `3ee4f4bf4`; REPORT was then missing. `loop report --check` and final PR check at `3c2123fed`: verdict PASS, every applicable rule green.

## Skills used

`uor-research-architect`; Superpowers `using-superpowers`, `test-driven-development`, `systematic-debugging`, and `verification-before-completion`.

## Still open

- Final full gates, this REPORT's final policy checks, and an other-vendor exact-head review are required before merge.
- Actual M2/M6/M1-position acceptance and the held M1 closing bar remain separate work. The new notebooks are empty scaffolds, not results.

## Plain summary

The roadmap now says where several research records belong, and the scope checker allows a notebook page and its first binding to be introduced together, including in a cross-item metadata PR. This makes future evidence records possible without changing any milestone result or verdict.
