# ORDER: one-model-end-to-end, claude-381 (V2(a): three vector-native reads behind flags)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407).
Base: `origin/main` at `d5eb4a55656726b04ea79b06968f91905933c925`. Roadmap step V2(a) of the 14 Nov audit; V0 is claude-376 (#3970). V1 runs beside it as claude-380.

## Goal

Port three state-keyed look-back reads into the float trunk, each behind its own flag. With every flag off, a `p` (claude-358) step is bit-identical to today's. This is a float reference port for V2(b)'s selection test (D85: float stays reference code; the winner's integer form is V2(c)). It trains nothing.

## Cancel probe

A state-keyed read already wired into the `p` trunk. At base: none. The Hamming reads key on token identity; claude-144's Flock read sits after the trunk, credits only its kept neighbours, and never learned where to look. The historical Starling read normalised its Lorentz scores with a softmax (excluded by D29), so only its score geometry ports.

## The reads (one new file, `vector_reads.rs`, hooked after layer D in `trunk()`)

Common: query and key are the state turned lane by lane by learned unit quaternions (Hamilton products) and folded to a 16-coordinate address by a fixed sum. Every all-pairs score is built from elementwise differences, never a product of two states (§3.3). The choice is hard and detached. The payload is the shared table's point of the token after the chosen position, added through a weight that starts at 0.
1. `--starling-read lorentz|l2`: score `offset - d`, `d` the hyperbolic distance of the lifted addresses (or the flat L2 control); no read when the best score is below 0.
2. `--prime-route-read 1`: each address block names its nearest icosian (of 120), the cell's prime its phase on 8 zeta channels; least summed wrapped phase energy wins (`uor-r4-core` table form).
3. `--state-flock-read K`: the K nearest addresses in mean L1, their payloads averaged.

Credit (all three): a margin hinge from each row's best positive (an earlier position followed by the row's own next token) to **every** earlier negative.

## Scope change (decision record, D68)

`crates/uor-r4-float-train/src/vector_reads.rs` is added to the item's `scope:` in this ORDER's commit, as claude-346 added `e8_address.rs`. A new file keeps the reads out of `in_space_long.rs`, which claude-380 edits. Edits there: config, optimizer, credit, save/load. In `train-in-space.rs`: flag parsing only.

## Files in scope

`vector_reads.rs` (new), `geometric_stack_native.rs`, `lib.rs`, `in_space_long.rs`, `bin/train-in-space.rs`, `fbs_export.rs` (refuse a checkpoint with a vector read), this folder, the notebook row, the item file, a V2(b) prep sheet.

## Native-space five items (SKILL)

None is met: a float reference unit, so it is an inventory-class unit and runs no training. The schema, shared operator, replay oracle, in-state update and serialization belong to V2(c).

## Done when

1. Laptop: closed-form tests of each score, the cell and phase tables, the hinge's pair set, and flag parsing pass.
2. GPU (cuda-gated): forward and gradient checks of each read on tiny fixed inputs; a set-then-cleared read leaves the trunk bit-identical.
3. GPU: base and head binaries train `p` for 20 steps with flags off; checkpoints and losses match byte for byte.
4. GPU smoke per flag: no NaN, step time against flags off.

## Stop rules

NaN in a smoke is a repair here. The selection test is not run here.
