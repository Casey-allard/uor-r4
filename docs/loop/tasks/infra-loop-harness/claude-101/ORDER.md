# ORDER: infra-loop-harness, claude-101 (hold m2-pin-train-mask-sheet on its two branches; give float-training-module the crate it must create)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 2 (item-file maintenance, as in claude-88, -90, -93 and -96). Base: `b1cc33d03`.

## Goal

Two item-file fixes. Each keeps `loop next` from sending labs to work that cannot start, and each is a lab decision (D68) covered by one fresh decision review.

1. **`m2-pin-train-mask-sheet` (#2851) becomes `status: held`.** `loop next` keeps offering it as GO (it unblocks 8), but both branches of its done-when are blocked by other items.
   - **The D37 sheet's FIT** waits on `float-training-module` (#2823) and `d11-exporter-float-trunk` (#2820), then a sealed fresh slice.
   - **The discrete branch** needs a one-page M2 sheet on the current discrete law, with a non-author FIT at a current head (claude-100, #3199). It cannot be FIT before a current discrete M2 trainer exists, and that comes with `m1-trunk-assembly-and-training`.
   - A single `depends_on` cannot say "either", and the decision review on #3196 rejected gating it on the float path alone. So the hold names both releases in the item's text.
2. **`float-training-module` (#2823) gets its own crate in `scope:`.** Its done-when asks for "a new module" with "no dependency on `crates/uor-r4-training`", and its text warns that reusing that crate is the likeliest mistake. But its `scope:` lists only `crates/uor-r4-training/**`, `ratchet.rs` and docs, so `loop guard`, which reads scope from `origin/main`, would refuse the new crate. The scope adds:
   - `crates/uor-r4-float-train/**`;
   - the root `Cargo.toml` (workspace members) and `Cargo.lock`;
   - its future notebook page, `docs/research/notebook/float-training.md`, for the row its step 6 asks for. `roadmap check` refuses a `notebook:` line for a page that does not exist, so the unit that creates the page also binds it.

**Why the infra-loop-harness item.** Its scope holds `docs/roadmap/items/*.md`. `loop claim` refuses it while Codex's idle #3169 holds an order on it, so this ORDER is written by hand.

## Cancel probe (run first)

Class: records. At `b1cc33d03`, all must hold, or the unit is cancelled:
- `m2-pin-train-mask-sheet` is `ready`, and `loop next` offers it as GO;
- `float-training-module`'s `scope:` has no crate path outside `crates/uor-r4-training`;
- no open PR touches either item file.

## Files in scope

- `docs/roadmap/items/m2-pin-train-mask-sheet.md` (status line and one note)
- `docs/roadmap/items/float-training-module.md` (the `scope:` line and one note)
- `docs/roadmap/README.md` (re-rendered; shared file)
- this task folder

## Done when

- Both item edits are made. `roadmap check` passes, and `render --check` matches.
- The fresh decision review passes, and both decisions are logged as owner logs on #2851 and #2823.
- `loop report --check` and `loop pr-check --local` pass, and the review counts at the exact head.
- The REPORT says `References #2904`, `References #2851` and `References #2823`.

## Stop and ask if

- The decision review finds a lab could start either branch of the mask-sheet item now.
- The float module's scope would need to reach any other existing crate.
