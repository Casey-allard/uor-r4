# ORDER: infra-loop-harness, kimi-31

Lab: kimi. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 2 (item metadata). Base: `3c0e0b294`. `References #2904`.

## Goal

Repair known item-metadata and notebook-bootstrap gaps so later evidence records can pass the scope guard. This PR changes no milestone status, bar, hold, research result or PLAN text.

## Cancel probe (run first)

At `3c0e0b294`, only three items declare `notebook:`. The M4 serving, M2 fresh-panel, M6 reasoning-panel, M1 position and M1 close items require notebook records but lack that binding; #3087's row was refused for this reason. `xtask roadmap check` then requires the M2/M6 pages to exist, while the scope guard refuses to create them before the binding lands. The loop-harness scope also omits `PLAN.md`. The task stands. Class: instrument/governance metadata.

## Files in scope

- `docs/roadmap/items/infra-loop-harness.md`
- `docs/roadmap/items/m2-fresh-heldout-panel.md`
- `docs/roadmap/items/m6-reasoning-panel-freeze.md`
- `docs/roadmap/items/m1-position-mechanism.md`
- `docs/roadmap/items/m1-close-saved-model.md`
- `docs/roadmap/items/m4-export-uorlut01-cli.md`
- `docs/roadmap/items/m4-lawful-likelihood-bpb.md`
- `docs/research/notebook/m2-grounded-reply.md` and `m6-reasoning-coding.md` (empty scaffolds only)
- `crates/uor-r4-harness/src/loopprcheck.rs` (shared registration file)
- `crates/uor-r4-harness/tests/loop_prcheck.rs`

## Done when

- The five named research items carry correct `notebook:` paths, with empty M2/M6 notebook scaffolds. The loop-harness scope includes `PLAN.md` for a later separately reviewed PM premise record.
- A regression test shows a new notebook page introduced with its binding is accepted; existing out-of-scope and row-rewrite refusals stay intact.
- No roadmap status, bar, hold, dependency, research result or PLAN text changes. `xtask roadmap check` and `roadmap render` agree, `roadmap sync` dry-run reports nothing to change, and REPORT, quick/full gates and local PR checks pass.

## Stop and ask if

A result row, status, bar, hold, dependency or PLAN content would change; another item needs a scope edit; or a file outside this list is needed.

Size estimate: about 140 lines.
