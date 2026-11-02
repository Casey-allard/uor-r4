# REPORT: wide FBS export repair, deepseek-2

Lab: deepseek. Item:
[m4-export-uorlut01-cli](../../../../roadmap/items/m4-export-uorlut01-cli.md)
(#2859). Order: [ORDER.md](ORDER.md). Base:
`e0c92af84287f87e1d402a40ff6f84cdd06f9315`.

## What did not work, first

- **Claude's real width-512 checkpoints were not available on this host.** The
  repair is verified on synthetic FBS shapes that match the sweep's writer,
  width and Hamming-head geometry, not on the private `(w)` checkpoints.
- **No model behavior or capability changed.** This removes a false export
  refusal; it does not make the width-512 arm better or select it.
- **The generic per-head attention cap remains.** A non-FBS one-head stack
  wider than 256 is still refused.

## What changed

`StackShape::validate` previously applied `width / heads <= 256` to every
shape. That bound belongs to attention reads. SpiralCore/FBS trunks are
all-recurrence stacks with no attention heads; their Hamming-head geometry is
in a separate record, and the native writer already enforces `width <= 1024`.

The validator now exempts only `projection == "spiralcore_fbs"` from that cap.
The exported `heads` field remains 1, so existing FBS artifact bytes and
arithmetic are unchanged.

The regression exports deterministic two-layer FBS trunks:

- width 512 with 8 Hamming heads; and
- width 1,024 with 16 Hamming heads.

Each artifact parses and serves identical three-token sessions. A normal
attention-shaped width-512/one-head stack is still refused with
`the head width exceeds 256`.

## Measurements

- Focused regression: 1 passed.
- Full integer serving library suite: 443 passed, 0 failed, 1 ignored.
- Existing FBS artifact-export, Hamming, memory, point-key, native-head and
  CPU/serving parity tests remained green.
- No training, inference benchmark, external cost or actual `(w)` checkpoint
  was touched.

## Checks run at the head

- `cargo test --locked -p uor-r4-integer --features serving-wasm --lib
  wide_all_recurrence_fbs_trunks_export_and_serve_past_the_attention_head_cap`:
  1 passed.
- `cargo test --locked -p uor-r4-integer --features serving-wasm --lib`:
  443 passed, 0 failed, 1 ignored.
- CPU strict Clippy reports 51 pre-existing crate findings; none is in the
  changed validator expression or the new regression.
- `git diff --check`: passed.

## Skills used

No available Codex skill applied. The shape validator, FBS exporter, native
writer width limit and Hamming geometry were read directly.

## Still open

- Re-export Claude's three existing width-512 checkpoints after merging.
- Then run Claude's already-defined three-seed completion and served parity.
- The width-1,024 checkpoint still requires enough GPU memory to exist; this
  repair only removes the export-width refusal once it does.

## Plain summary

The laptop exporter was rejecting wide all-recurrence models because it treated
their dummy one-head shape as if it were an attention head. The cap now applies
only to attention stacks. FBS widths 512 and 1,024 export and serve in exact
deterministic controls; the real Claude checkpoints still need to be re-exported.
