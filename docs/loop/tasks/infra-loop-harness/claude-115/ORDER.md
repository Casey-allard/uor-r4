# ORDER: infra-loop-harness, claude-115 (new item: a GPU panel scorer and tok/s path for D11 artifacts)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 2 (item-file maintenance: a new gap item, as CHARTER §5.3 asks). Base: `ac937d6fb`.

## Goal

PLAN.md step 6 (`m1-export-gap-and-toks`, #2839, held for step 4's model) and the M1 close (step 8) read the model "through the D11 export … on the same DEV panel, with tokens/s reported" (CHARTER §4.8). CHARTER §3.6 makes "panel scoring … of an exported artifact" GPU research compute. On `main`, three tools that step 6 needs do not exist:
- **No exporter CLI.** `export_d11(&Snapshot)` (`crates/uor-r4-discrete/src/d11_export.rs:227`) has no caller. The step-6 sheet's `discrete-stack export-d11` and `export_d11_to_path` are not on `main`, and the "#2998" it cites is another PR.
- **No panel scorer for an exported artifact,** on the GPU or anywhere else. `crates/uor-r4-integer` has no `.cu` file and no `build.rs`. `IntegerDiscreteModel::evaluate_prefix` (`crates/uor-r4-integer/src/discrete_d11.rs:1165`) serves one prefix at a time.
- **No serving tok/s harness** for `lut-discrete` files that splits prompt-reading from reply-writing.

This unit adds one roadmap item for them (CHARTER §5.3: "a newly found gap is a new item file in a PR"), with a D68 decision card. It builds nothing.

**Why the infra-loop-harness item.** Its scope holds `docs/roadmap/items/*.md`. Codex's idle #3169 still holds the order, and its files do not overlap, so this ORDER is hand-written under the run file's umbrella exception.

## Cancel probe (run first)

Class: instrument. Run at `ac937d6fb`; all must hold:
- **The three tools are absent on `main`.** `git grep` finds no `export-d11` subcommand and no `export_d11_to_path`, and `uor-r4-integer` has no `.cu` file. **Holds.**
- **No item covers them.** `m4-export-uorlut01-cli` is an M4 product CLI that depends on `m1-close-saved-model`, so it cannot serve M1 step 6. **Holds.**
- **The new item takes no lock.** Its scope holds no crate path; the instrument lives under `docs/labs/`. **Holds.**

## Files in scope

- `docs/roadmap/items/m1-d11-artifact-scorer.md` (new)
- `docs/roadmap/issues.json` (one line: the new item's issue, #3248, created by hand so that the PM's weekly `roadmap sync` updates it instead of creating a duplicate; the dry run would also touch 33 other issues, which is the PM's to apply)
- `docs/roadmap/README.md` (the generated render only)
- This task folder

## Done when

- **The new item** is `ready`, scoped with no crate path, and has a done-when for each of the three tools.
- **The D68 card** has a fresh decision review that approves it, and that review has read the step-6 sheet, its task folders and #2839.
- **The checks pass:** `xtask roadmap check`, `loop report --check` and `loop pr-check --local`.
- **A fresh exact-head review** approves, and the PR is queued and verified.
- **A note on #2839** tells antigravity that the tools come through this item.
- **The REPORT** says `References #2904` and `References #2839`.

## Stop and ask if

Stop and decide, never stop and wait.

- A lab is already building any of the three tools. Then this unit narrows to the rest, or stops.
