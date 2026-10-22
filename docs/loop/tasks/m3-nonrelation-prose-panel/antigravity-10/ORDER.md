# ORDER: m3-nonrelation-prose-panel, antigravity-10

Lab: antigravity. Item: [m3-nonrelation-prose-panel](../../../../roadmap/items/m3-nonrelation-prose-panel.md). Base: `72e5f3e92`.

## Goal

The item's done-when reads:

> A pre-registered non-relation prose panel with grading, plus the baseline false-assert and false-overwrite rates on world=v2. The pass bar for those rates is not yet registered; registering it is step 2 below.

## Cancel probe (run first)

Ran `uor-harness loop gates --quick` on base `72e5f3e92` (`fmt-claim-wording` GREEN 0.5s, `fmt-harness` GREEN 0.8s, `claim-wording` GREEN 5.4s; logs at `local/antigravity/gates/1792659908-61067`) and checked `#2856` and `docs/roadmap/items/m3-nonrelation-prose-panel.md`. No active claim or existing non-relation prose panel exists; proceeding with the design note, frozen plain-text panel, Rust grader with finite CPU tests, and one-page pre-registration sheet.

## Files in scope

- `crates/uor-r4-training/**`
- `data/**`
- `docs/labs/m3-nonrelation-prose-panel/**`

## Done when

- Design note posted on `#2856` defining the 5 panel categories (`small_talk`, `opinion`, `nonrelation_question`, `anaphoric`, `statement_near_miss`), the grading criterion (a prose turn is clean only when `CompiledAction::Unresolved`), the `false_assert` and `false_overwrite` rates, and a proposed pass-bar decision card to the PM on `#2028` (CHARTER §5.2).
- Plain-text panel frozen at `data/panels/nonrelation-prose-v1.txt` with one turn per line and SHA-256 hash recorded, including a held-out slice disjoint from `world=v2` (`milestone_world` / `milestone_world_v2`) training templates and vocabulary.
- Rust grader module `crates/uor-r4-training/src/nonrelation_prose_panel.rs` (wired into `crates/uor-r4-training/src/lib.rs`) grading `CompiledAction` and `MemoryEffect` over a seeded `StackStore` and `TurnCompiler` stub, with finite CPU unit tests verifying clean `Unresolved` turns, `false_assert` (`CompiledAction::Assert`), `false_overwrite` (`CompiledAction::Correct` that modifies a stored fact vs. unseeded `Correct`), false queries, SHA-256 verification, and `world=v2` template disjointness (`~/.cargo/bin/cargo test -p uor-r4-training`).
- One-page pre-registration sheet and lab record in `docs/labs/m3-nonrelation-prose-panel/` (with `References #2856` since the GPU `world=v2` baseline run and owner pass-bar decision remain gated).

## Stop and ask if

- Any file outside `[crates/uor-r4-training/**, data/**, docs/labs/m3-nonrelation-prose-panel/**]` must change.
- Any GPU run on the `world=v2` compiler is needed before a non-author FIT review and owner release.
- Fixing a pass bar is required (pass bars belong to the owner under CHARTER §5.2).
