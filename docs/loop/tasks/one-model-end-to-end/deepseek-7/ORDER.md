# ORDER: one-model-end-to-end, deepseek-7

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Base: `4b0fb8028091e85c5e6c0a34ab0c632b7ccad498`.

## Goal

Restore the repository-wide harness after Claude's merged capacity sheet failed
`prep.sheet_shape`. Add only the required pre-registration labels and summaries,
using facts already present in the sheet and REPORT. Do not change an arm,
measurement, bar, stop rule, verdict or model claim.

## Scope

- `docs/harness/prep/one-model-end-to-end-capacity-2026-11-02.md`
- `docs/loop/tasks/one-model-end-to-end/deepseek-7/**`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/one-model-end-to-end.md`

## Required design

- Add the recorded `TEST FITNESS: FIT` and `PRE-RUN: PASS` lines from the
  capacity REPORT's issue review, with a shape-only amendment note.
- Add discoverable sections named `Base artifact`, `Data split`, `Bar`,
  `Steps`, and `Minimum test power`.
- Derive every added statement from the sheet's existing arms, data, readings,
  decision table, instrument checks, cost and review record.
- Preserve all existing text and say explicitly that no criterion or result
  changed.

## Done when

- `prep-check` parses the sheet completely.
- `cargo test -p uor-r4-harness` passes the real-sheet rules.
- REPORT and notebook record the shape-only repair.
- A fresh exact-head review approves the PR.
