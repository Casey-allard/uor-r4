# REPORT: infra-loop-harness, kimi-12

Lab: kimi. Order: [ORDER.md](ORDER.md).

## What did not work, first

- The first attempt at this work (as a single PR under item `m7-parity-library-cli-browser`) failed `uor-harness loop pr-check --local` on `scope`: the rule reads the item's scope at the merge base, so a PR cannot widen the scope it itself relies on. That is the README's two-step rule working as designed; this PR is step 1 (the scope entries), and the registration content follows as kimi-13 under the m7 item.
- No other attempt or check failed. Docs-only; no GPU, no code, no model compute.

## What changed

- `docs/roadmap/items/m7-parity-library-cli-browser.md`: scope gains `docs/harness/panels/m7-parity.json` and `docs/roadmap/items/m7-parity-library-cli-browser.md` (frontmatter line only), so kimi-13 can register the frozen `m7-parity-prompts-v1` bar panel and point the item's Done-when at it (#2867 comment 6352825431; PM approval 6354929619).
- `docs/roadmap/items/m4-lawful-likelihood-bpb.md`: scope gains `docs/harness/prep/m4-lawful-likelihood-bpb*` (frontmatter line only), so deepseek can write the pre-registration sheet where every prep sheet lives (card 6354581554; PM approval 6354929619). No GPU is touched; the sheet still needs a non-author FIT and a typed owner release before any run.

Size by class: 2 docs files, one frontmatter line each; the ORDER and REPORT under the task folder. No code, no tests (nothing executable changed).

## Measurements

Nothing was measured; there is no runnable behaviour to measure. Verification is the check suite below.

## Checks run at the head

- Cancel probe at base `6ab26955b`: `grep -n "panels/m7-parity" docs/roadmap/items/m7-parity-library-cli-browser.md` and `grep -n "harness/prep" docs/roadmap/items/m4-lawful-likelihood-bpb.md` — both absent before the change, present after.
- `uor-harness loop report --check`: result recorded in the PR body.
- `uor-harness loop pr-check --local`: result recorded in the PR body; both edited files match the umbrella's `docs/roadmap/items/*.md`, so `scope` is expected to pass.

## Skills used

None.

## Still open

- kimi-13 (the actual `m7-parity-prompts-v1` registration: `docs/harness/panels/m7-parity.json`, the m7 item's Done-when, `docs/products/web-client.md`) is held until this PR merges, because `pr-check` reads scope at the merge base.
- DeepSeek's pre-registration sheet for #2861 is unblocked by this PR's merge, not before.

## Plain summary

Every task in this project may only touch the files its roadmap item lists, and the check reads that list as of the PR's starting point. Two teammates need permission lines added to their items: one so the project's registry can record the official browser/CLI/library sameness test, one so a planning document can go in the standard folder. This change adds exactly those two permission lines, each already requested by the teammate and approved by the project manager (this lab), and nothing else.
