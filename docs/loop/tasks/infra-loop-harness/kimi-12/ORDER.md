# ORDER: infra-loop-harness, kimi-12 (PM scope entries: m7 parity panel + item, m4 prep sheet)

Lab: kimi. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 2. Base: `6ab26955b`.

## Goal

Two PM scope entries, both requested on #2028 and approved there by the PM (6354929619):

1. `m7-parity-library-cli-browser` (#2867) gains `docs/harness/panels/m7-parity.json` and `docs/roadmap/items/m7-parity-library-cli-browser.md` in its scope, so the PM registration PR (kimi-13) can add the frozen `m7-parity-prompts-v1` bar panel and point the item's Done-when at it. `pr-check`'s scope rule reads the item at the merge base, so the scope entry must land first (the docs/loop/README.md:103 two-step).
2. `m4-lawful-likelihood-bpb` (#2861) gains `docs/harness/prep/m4-lawful-likelihood-bpb*` in its scope, so deepseek can write the pre-registration sheet where every prep sheet lives (deepseek's card 6354581554). No GPU is touched by this; the sheet still needs a non-author FIT and a typed owner release before any run.

## Cancel probe (run first)

`grep -n "panels/m7-parity" docs/roadmap/items/m7-parity-library-cli-browser.md; grep -n "harness/prep" docs/roadmap/items/m4-lawful-likelihood-bpb.md` at the base: both absent (no matches). The scope entries are needed; the task stands.

## Files in scope

- `docs/roadmap/items/m7-parity-library-cli-browser.md` (frontmatter scope line only)
- `docs/roadmap/items/m4-lawful-likelihood-bpb.md` (frontmatter scope line only)
- `docs/loop/tasks/infra-loop-harness/kimi-12/**` (this ORDER and the REPORT)

Anything else is a stop-and-ask.

## Done when

- Both items carry the new scope entries and no other line changes.
- `uor-harness loop report --check` and `uor-harness loop pr-check --local` pass, scope included (both edited files match the umbrella's `docs/roadmap/items/*.md`).

## Stop and ask if

- Any reviewer asks for a wider glob than the named file (say why not, or escalate).
- The item owners withdraw the underlying requests on #2867 / #2028.
