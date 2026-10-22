# ORDER: m2-fresh-heldout-panel, antigravity-23

Lab: antigravity. Item: [m2-fresh-heldout-panel](../../../../roadmap/items/m2-fresh-heldout-panel.md) (#2849). Base: `f67ac2525`.

## Goal

> A fresh panel registered and sealed in advance, graded by the existing deterministic graders unchanged; bar: '≥ 256/512 saved questions and ≥ 40% of genuinely fresh ones'.

Repair PR `#2985` findings from review `6347483118` and the two red CI jobs (`docs archive and links` and `harness rules and tests`).

## Cancel probe (run first)

At `f67ac2525`, `uor-harness loop next` reports `GO repair #2985` (`m2-fresh256.json` fails `panel_shape`, `antigravity-22/REPORT.md` has 3-level relative links, and review `6347483118` requests on-disk source parsing, exact `ByteBpeTokenizer` encoding, `pair_id` history-pair control gating, and `status: ready`). Result: task is live. Class: instrument.

## Calls

- Fix the 12 relative links in `antigravity-22/REPORT.md` (`../../../` -> `../../../../`) and format `docs/harness/panels/m2-fresh256.json` to match `panel_shape` (`frozen: false`, `TEST FITNESS: PENDING`).
- Parse on-disk source files (`curriculum.rs`, `geometric-native-bank-observation-plan.rs`, `fresh-preparation-audit-attempt1.json`) for all `74` historical literals (`0` overlaps, `0` intra-panel duplicate content words).
- Encode with exact `ByteBpeTokenizer` (`d36d3e87…`), emit consumer-compatible `inputs.json`/`labels.json` (`StackStore` `event + 1`, `pair_id` so all 4 controls score `0/256 < 103`), and pin hashes in a unit test.
- Keep `m2-fresh-heldout-panel.md` at `status: ready` and use `References #2849`.

## Files in scope

- `data/**`, `docs/harness/panels/**`, `docs/mechanisms/native-geometric.md`, `docs/labs/m2-fresh-heldout-panel/**`, `docs/reference/storage.md`, `docs/roadmap/items/m2-fresh-heldout-panel.md`, `docs/roadmap/README.md`, `docs/loop/tasks/m2-fresh-heldout-panel/antigravity-22/REPORT.md`, `docs/loop/tasks/m2-fresh-heldout-panel/antigravity-23/**`

## Done when

- Instrument unit tests, `contract::tests::every_format_passes_on_the_real_state`, `xtask links`, `loop gates --full`, `report --check`, and `pr-check --local` pass.

## Stop and ask if

- Any out-of-scope file, owner path, or GPU compute is needed.
