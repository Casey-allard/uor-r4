# ORDER: infra-loop-harness, deepseek-64 (refresh four stale roadmap item sections)

Lab: deepseek. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Base: `a2e74123a`. Umbrella scope: `docs/roadmap/items/*.md`. No item status, bar, hold or plan change.

## Goal

Four item `What is missing` sections are stale after deliveries that are already on main: **m1-position-mechanism** still says no mechanism is assigned, **m4-lawful-likelihood-bpb** says a lawful likelihood is still unbuilt, **m7-studio-self-host-page-assets** still lists the CDN page and the unchecked page-load claim as missing, and **object-store-consumer-m3** says nothing consumes the store. Replace those lines with short, dated, evidenced status sentences (delivered commits and PRs, the FIT/registration state where it exists, and what remains). This is a roadmap-text refresh under the umbrella item's `docs/roadmap/items/*.md` scope; it changes no item status, bar, hold or mechanism.

## Cancel probe (run first)

At `a2e74123a`: the four sections read as described above (for example object-store-consumer-m3 says "nothing consumes it"), while the named deliveries are on main (`3e3967c7`, `4797ca883`, `0d5d6791d`, PRs #3046/#3073/#3080/#3106/#3108). Task stands.

## Files in scope

- `docs/roadmap/items/m1-position-mechanism.md`
- `docs/roadmap/items/m4-lawful-likelihood-bpb.md`
- `docs/roadmap/items/m7-studio-self-host-page-assets.md`
- `docs/roadmap/items/object-store-consumer-m3.md`
- this task folder

## Done when

- Each of the four `What is missing` sections states what is delivered (with the commit/PR evidence), what the current FIT/registration state is (only where one exists), and what remains open; no item frontmatter, status, bar, hold or scope line changes.
- Every cited commit/PR exists and is merged; the m7 list numbering is contiguous.
- `xtask roadmap check` and render/sync, `loop report --check`, `loop pr-check --local` and `loop gates --quick` pass; a counted non-deepseek exact-head review records.

## Stop and ask if

- Any item status, bar, hold, frontmatter or file outside the list would change.