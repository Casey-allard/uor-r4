# ORDER: m4-export-uorlut01-cli, deepseek-2

Lab: deepseek. Item: [m4-export-uorlut01-cli](../../../../roadmap/items/m4-export-uorlut01-cli.md)
(#2859). Base: `e0c92af84287f87e1d402a40ff6f84cdd06f9315`.

## Goal

Unblock Claude's trained-as-served width-512 capacity arm at the integer export
boundary. An all-recurrence SpiralCore/FBS trunk has no attention heads; its
generic `heads` field must not inherit the attention-read per-head width cap.

Claude-259's `(w)` width-512 checkpoints otherwise fit the native writer's
declared width limit, but `fbs-export` refuses them because the exported shape
uses `heads: 1` and the generic validator requires `width / heads <= 256`.

## Scope

- `crates/uor-r4-integer/src/stack/format.rs`
- `crates/uor-r4-integer/src/stack/tests.rs`
- `docs/loop/tasks/m4-export-uorlut01-cli/deepseek-2/**`
- `docs/research/notebook/m4-serving.md`
- `docs/roadmap/items/m4-export-uorlut01-cli.md`

## Required design

- Keep the 256-wide per-attention-head cap for attention stacks.
- Exempt only shapes with `projection == "spiralcore_fbs"` from that cap,
  because their read/Hamming head geometry is carried separately and the
  native writer already enforces `width <= 1024`.
- Do not change an existing FBS artifact's bytes, arithmetic or `heads` field.
- Add a regression that exports and parses synthetic all-recurrence FBS trunks
  at widths 512 and 1,024, then executes deterministic serving steps.
- Keep an ordinary attention shape above the cap refused.
- Do not modify Claude's draft evidence or claim artifact results that require
  the private checkpoints.

## Done when

- The focused regression and full integer serving library suite pass.
- Existing FBS export bytes and attention refusals remain covered.
- The REPORT and notebook row state the exact repair and boundary.
- A fresh exact-head review approves the PR.
