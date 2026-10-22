# ORDER: m6-reasoning-panel-freeze, antigravity-20

Lab: antigravity. Item: [m6-reasoning-panel-freeze](../../../../roadmap/items/m6-reasoning-panel-freeze.md). Base: `d0c16660f`.

## Goal

The item's done-when reads:

> A frozen, pre-registered panel with deterministic oracles for compositions, altered constraints and code tasks (D28: no external teacher). CHARTER §3.10 requires that panels are graded deterministically, with no neural comparator, control, teacher or grader.

## Cancel probe (run first)

Ran `git grep -n "panel_m6_reasoning\|generate_frozen_m6_panel" crates/ docs/` and inspected [m6-reasoning.json](../../../../harness/panels/m6-reasoning.json) at `d0c16660f`:
- `docs/harness/panels/m6-reasoning.json` is an unfrozen stub (`"frozen": false`, `"grader": ""`, `"bars": []`, `"noise": "unstated - a bar must not be narrower than it"`).
- No deterministic M6 panel generator, 3-family oracle (`Compositions`, `AlteredConstraints`, `CodeTasks`), overlap-exclusion check, or lawful count-table baseline evaluator exists in `crates/uor-r4-harness`.
- The cancel probe confirms the work is genuinely absent.

## Files in scope

- `crates/uor-r4-harness/src/panel*.rs`
- `crates/uor-r4-harness/tests/panel*.rs`
- `docs/harness/panels/**`
- `docs/reference/storage.md`
- `docs/labs/m6-reasoning-panel-freeze/**`
- `docs/roadmap/items/m6-reasoning-panel-freeze.md`
- `docs/roadmap/README.md`
- `docs/loop/tasks/m6-reasoning-panel-freeze/antigravity-20/**`

## Done when

- `host:` `CARGO_TARGET_DIR=/Users/casey.allard/uor-r4/local/harness-target ~/.cargo/bin/cargo test --locked -p uor-r4-harness --test panel_m6_reasoning` passes (`4/4` deterministic panel, oracle, overlap-exclusion, and count-table noise tests), `docs/harness/panels/m6-reasoning.json` is frozen (`"frozen": true`) with deterministic D28 grader, measured count-table noise band, and 4 bars (`overall-pass-240`, `compositions-80`, `altered-constraints-80`, `code-tasks-80`), `docs/reference/storage.md` and `docs/labs/m6-reasoning-panel-freeze/README.md` document the SHA-256 content seal and non-author `TEST FITNESS: FIT` review, and `uor-harness loop gates --full` is green.

## Stop and ask if

- Any file outside the declared scope must change.
- Any GPU execution is needed (this item is host-only and releases no GPU run).
- Any D23 (`softmax`/squashing), D25 (`matmul`), D28 (neural grader), or D32 (`discrete-float`) ratchet count increases.
