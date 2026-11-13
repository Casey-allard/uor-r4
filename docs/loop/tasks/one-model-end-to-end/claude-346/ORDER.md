# ORDER: one-model-end-to-end, claude-346

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `375df0e1a3fd6227517fd5f8c5cdb87c197a02e8`.

## Goal

**Build the E8 address memory in the float trainer, with its four address arms and its telemetry, so its first test can run on the memorize set.**

**Why now.** The aligned design Casey accepted on 9 Nov ("Yes, start": the memorize test, then the two new parts) names two gaps in today's model. The first is that nothing learned sits between a state and the next: 1,112,576 learned values, 1,048,576 of them the token table. The address memory is the part that fills it. The memorize test T0 (claude-345, PR #3911) is on its pods now; this unit builds the part T0's row 3 and row 1 both send the work to, so no time is lost waiting for the reading.

**The mechanism.**
- **Where.** Inside the trunk, after each layer's Hamilton block: `x ← x + Σ_blocks V[layer][block][cell(block of x)]`.
- **The address.** The state's 256 coordinates are 32 blocks of 8. Each block is one point of R⁸, and its cell is the nearest of E8's 240 roots: 112 of the form ±eᵢ ± eⱼ and 128 of the form (±½)⁸ with an even number of minus signs. No learned projection is involved, and no product of a state with a table.
- **The rows.** One table per layer of 32 × 240 rows, each 256 wide: 1,966,080 values a layer, 11,796,480 for 6 layers. A token reads 32 rows a layer, one per block, out of 7,680.
- **The served form.** Rows are 16-bit integers on the residual stream's own 2⁻¹⁶ grid, so serving a row is `x_i32 += row_i16`: 32 × 256 additions a layer and token, no multiply and no shift. Under `--train-as-served` the trainer rounds the rows to that grid for every forward and keeps the float values for the optimizer.
- **The credit.** Rows read take the gradient of the state they were added to. The state takes a gradient through the address only near a wall: with `d` the block's distance to the wall between its cell and the runner-up, `ρ` the state's RMS coordinate (detached) and `τ` the band, `λ = ½·max(0, 1 − d/(τρ))`, and the forward adds `(V[across] − V[cell])·(λ − λ.detach())`, which is exactly zero. Its gradient moves a block toward the wall when the row across it would have served the loss better.
- **The step.** Each of a position's 32 rows takes the same gradient, so the rows' optimizer runs at `lr/32`: their sum then moves at the trunk's rate.

**The arms** (`--address-memory`), each a table of the same row width:
| Arm | Cell of a block | Cells a block | What it controls for |
|---|---|---|---|
| `e8` | nearest E8 root | 240 | the mechanism |
| `sign` | the block's 8 signs | 256 | whether E8's cells matter, against the plainest partition of R⁸ |
| `collapse` | one cell | 1 | a learned offset a layer with no address at all |
| `token` | a seeded hash of the position's own input token, the layer and the block | 240 | the same table size read with no state in the address |

No memory is the run without the flag, which writes the same `config.json` as before.

**Telemetry** (DeepSeek's first condition, 9 Nov): at each checkpoint the first 16 usable TRAIN windows are forwarded and `address.jsonl` gets one line with, per layer, the cells in use per block (mean and least), the largest share of a block's reads that go to one cell, the share of reads whose cell moved since the last checkpoint, the share inside the band, and the rows' mean size.

**Three changes to the design text of 9 Nov** (DESIGN.md, sections 4 and 10, in PR #3911), each recorded here because the code is where they were found:
1. **The `token` arm replaces the "random address" arm.** An address drawn fresh at every call carries nothing, so in expectation it is the `collapse` arm with noise. A fixed hash of the token is the control that keeps the table and removes the state.
2. **The band is scaled by the whole state's RMS coordinate,** not by the block's own size. Scaled by the block, a small block's gradient grows as one over its size.
3. **Rows are 16-bit at 2⁻¹⁶ with the `lr/32` step,** in place of a separate row scale: the residual stream is already on that grid, so the served read needs no arithmetic beyond the add.

**Claim check.**
- (a) **Step.** The owner direction's last part, "one model end to end" (`PLAN.md`), and the 9 Nov handoff's definition of done. This is the first of the design's two new parts.
- (b) **What it changes in the model.** A new learned part in the trunk, off unless the flag is given. Whether it helps is the next sheet's question, not this unit's.

**Rules check** (CHARTER §3). Geometry only: the address is the E8 root system the project's cells already use. §3.2: no squashing function; the band's `max(0, ·)` is a clamp at zero. §3.3: no dense product of a state with a table, in training or serving: the read is 32 row lookups and adds. §3.6 excludes "a dense model disguised as lookup tables"; a row here is chosen by one block's nearest root and added whole, as the token table's row is chosen by a token. The first test's sheet carries this check to its FIT.

**Amended on 13 Nov: the part's first training run moves into this unit.** This is a scope change, taken on the lab's own decision review and logged here for Casey, who can reverse it (CHARTER §5.1).
- **What changes.** The run of [the address-memory sheet](../../../../harness/prep/one-model-end-to-end-address-memory-2026-11-13.md), with its FIT and its pre-run review, is in scope. It trains the part inside today's model on the model's own corpus at two seeds, and on OASST2, in place of the memorize set this ORDER's goal names.
- **Why.** The part was built on 9 Nov and its GPU tests passed on 10 Nov. With the run left to "the next unit", the part then sat untrained for three days while this unit stayed a draft. The gap it fills, that almost nothing is learned between a state and the next, is the first of the seven gaps given to Casey on 13 Nov. Casey ruled the memorize test out as a track on 11 Nov ("T0 is not a track that will advance the project").
- **What the review read.** The gap list of 13 Nov, `PLAN.md`'s owner direction, the 9 Nov design, claude-358's three seeds of the same recipe (188, 188 and 225 exact replies of 1,108), DeepSeek's memorize-set sheet for this part (not run here), and the GPU caps.
- **What does not change.** Items 1 to 4 of Done when, and no bar: none existed for a run.

**Not in scope.** ~~Any GPU run (the next unit's sheet, FIT and pre-run review).~~ (amended above). The integer runtime's section for the rows and the exporter: `fbs_export` refuses a checkpoint holding `address_memory.safetensors` until then, so nothing is served by accident. The state-keyed margin read (T2). T0 (claude-345).

## Cancel probe (run first)

- **What would make this unit unnecessary:** an existing learned table in the trunk read by the state's own address.
- **Result (at claim):** none. `crates/uor-r4-float-train/src` has six read hooks (Flock, content, Hamming, opening, register, turn memory); each is keyed by token points or by positions, and none adds a learned row chosen by the state's cell. `git grep -n "e8_address\|address_memory" origin/main -- crates` finds nothing.

## Claim

`uor-harness loop claim one-model-end-to-end --lab-topic e8-address` was refused on 10 Nov: "item one-model-end-to-end is already claimed by worktree .worktrees/claude-talk (docs/loop/tasks/one-model-end-to-end/claude-345/ORDER.md)". claude-345's pods run for about two hours with nothing to do but watch; Casey approved both pieces in one answer on 9 Nov. The ORDER is written by hand in its own worktree, the two units touch no file in common, and the refusal is logged for Casey in the REPORT.

## Files in scope

- `docs/loop/tasks/one-model-end-to-end/claude-346/**`
- `crates/uor-r4-float-train/src/e8_address.rs` (new) and its line in `crates/uor-r4-float-train/src/lib.rs`
- `crates/uor-r4-float-train/src/geometric_stack_native.rs` (the hook, one cache test)
- `crates/uor-r4-float-train/src/in_space_long.rs` and `crates/uor-r4-float-train/src/bin/train-in-space.rs` (the flag, the optimizer, the rounding, the save and load, the census)
- `docs/roadmap/items/one-model-end-to-end.md` (the two new paths added to the item's `scope:`)
- `docs/harness/prep/one-model-end-to-end-address-memory-2026-11-13.md` and the item's row in `docs/research/notebook/float-training.md` (added 13 Nov with the run)

## Done when

1. The address is right, shown by tests that run on the laptop: every root decodes to its own cell, the nearest root, the runner-up and the wall's distance agree with a search over all 240 on 4,096 blocks, and the wall's distance has the slope the hand-worked gradient uses.
2. The tensor code is held to item 1 by tests written for the pod's CUDA test step, which runs before any training step: the tensor decode equals the laptop's plain-Rust form block for block, zero rows leave the residual and trained rows add one row a block, the wall gradient equals its hand-worked value, the trunk and the decode cache agree. The crate's library, trainer and their tests type-check with `--features cuda` under the stub compiler.

Amended on 10 Nov, before any commit. Items 1 and 2 first read "tests that run on the laptop" for the reads and the gradients too. The vendored tensor library refuses every arithmetic op on the laptop (`cpu_backend::refuse_cpu`, D24/D29), so no tensor test can run here. The decode's closed form is therefore written a second time in plain Rust inside the tests and checked on the laptop against the search; the tensor code is held to that second form on the GPU.
3. A run without the flag writes the same `config.json` and reads no new file.
4. The REPORT gives the negatives first, with the tests run and the ones only a pod can run named.
5. (Added 13 Nov.) The run's sheet has a fresh non-author FIT, every stage that ran is sealed in the store, and the REPORT gives each reading against the sheet's bars, with the replies word for word.
