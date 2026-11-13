# REPORT: one-model-end-to-end, claude-346 (the E8 address memory: built, tested, and its first training run)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [ORDER.md](ORDER.md). Sheet: [the address-memory sheet](../../../../harness/prep/one-model-end-to-end-address-memory-2026-11-13.md). PR #3913.

The address memory is a learned table inside the trunk, read by the state's own E8 cell: 32 rows a layer and token, added to the residual. This unit built it in the float trainer with its four address arms, and on 13 Nov trained it inside today's model at two seeds and on real text. The replies word for word are in [transcripts.md](transcripts.md).

Every figure below is in `local/claude/claude-346/` in the main checkout (`read-*.log`, `kinds-*.log`, the fetched stages under `run/`), kit files kept out of the repository by the project's rule on run output. The runs' source is `f941a33fd1a4eea751705d1cd6021af35452a4b7`.

## What did not work, first

1. **The sheet's verdict is "not called".** The bar was a mean gain of 30 exact replies of 1,108 over the same recipe without the part. Seed 2 gained 30 (218 against 188) and seed 3 gained 29 (254 against 225): the mean is 29.5. The bar is not moved after the reading.
2. **The control gives the same gain.** The `token` arm keeps the table and takes the state out of the address (a fixed hash of the position's own input token). At seed 2 it scored 219, one above the E8 arm's 218. So this run does not show that the gain comes from the state's address. The sheet reads the control only after an "adds" verdict; it is given here because it is the plainest fact in the run. My prediction, that the token arm would land within 20 of the no-address run, was wrong: it landed 31 above.
3. **Replies that never end doubled under the E8 address.** Replies with no end mark inside 48 tokens: 99 without the part, 190 and 200 with it, 56 with the token address. One-word repeats: 47, then 90 and 107, then 4.
4. **No arm answers a never-told question.** Of 79 questions whose answer the conversation never gave, every arm answered 0 with the wanted "not told" reply (the no-address run said it once, the token arm four times, both E8 runs never). The gain is almost all on general replies (119 to 142 and 177 of 520); told values moved 69 to 76 and 77 of 509.
5. **On real text the part changed nothing.** OASST2, 4,096 steps: next-token hits on held-out text 31,718 of 184,542 without the part and 31,673 with it (17.18% and 17.16%). I predicted a gain of 1 to 6 points. Neither model writes a usable reply (339 and 343 of 386 never end).
6. **It is slower.** 3,355 s against 2,143 s for the same steps on the model's own corpus (1.57 times), and 1,252 s against 784 s on real text.
7. **Two tries were lost.** Try 1's pre stage ended "the e8 smoke did not train": the trainer ran out of GPU memory, because an index computed on the GPU kept its history and the rows' gradient then held every step's forward pass. Try 2 found no machine. Tries 3 and 4 ran with the fix.
8. **The laptop cannot run the address tests at this head.** Done-when item 1 says the decode tests run on the laptop. The crate's test binary does not link here (`ld: library 'cuda' not found`), so they ran on the pod only (below).
9. **Two readings the sheet names were not taken:** the commonest-token share and the 64-step smoke's hit rate, the two reference points for the real-text reading.
10. **The seed-1 pair the sheet unlocks on "not called" was not run in this unit.**
11. **The fresh review of the tests found five gaps in what they would catch** (no defect in the code they test was found by the two reviews of that code). They are not repaired here, because a repair can only be shown on a pod and the part is off unless its flag is given; they are listed under Still open.

## What changed

- `crates/uor-r4-float-train/src/e8_address.rs` (new): the E8 decode (nearest root, runner-up, distance to the wall), the four address arms (`e8`, `sign`, `collapse`, `token`), the rows' read and the gradient through the address near a wall, the telemetry, and the tests.
- `geometric_stack_native.rs`: the hook after each layer's Hamilton block. `in_space_long.rs` and `bin/train-in-space.rs`: the `--address-memory` flag, the rows' optimizer at `lr/32`, the rounding to the served grid, save and load, the census. `flock_read.rs`: the running means keep no history of an address memory.
- The fix for the lost try: index tensors are detached before they select rows, with two tests that fail without it.
- The sheet, with five fresh non-author reads (three FIT WITH CHANGES with every finding taken, then two FIT).
- A run without the flag writes the same `config.json` and reads no new file. The exporter still refuses a checkpoint that holds the part, so nothing is served.

## Measurements

Panel m, 1,108 prompts; float scoring pass on each job's last checkpoint, greedy, turn memory on, context 192, 48 reply tokens.

| Job | Exact replies | No end in 48 | One-word repeats | Train loss | Dev loss | Nearest is true | Seconds |
|---|---|---|---|---|---|---|---|
| no address, seed 2 | 188 | 99 | 47 | 0.5447 | 2.563 | 87.66% | 2,143 |
| E8 address, seed 2 | 218 | 190 | 90 | 0.4700 | 2.469 | 88.21% | 3,355 |
| E8 address, seed 3 | 254 | 200 | 107 | 0.4112 | 2.377 | 88.59% | 3,354 |
| token address, seed 2 | 219 | 56 | 4 | 0.4684 | 2.3305 | 88.24% | 2,923 |

The seed 3 run without the part is claude-358's (225). The pin check holds: this unit's no-address run at seed 2 gives 188, claude-358's figure.

Exact replies by kind:

| Job | Never told (79) | General (520) | Told value (509) |
|---|---|---|---|
| no address, seed 2 | 0 | 119 | 69 |
| E8 address, seed 2 | 0 | 142 | 76 |
| E8 address, seed 3 | 0 | 177 | 77 |
| token address, seed 2 | 0 | 141 | 78 |

Against the sheet's bars: D2 = +30, D3 = +29, M = +29.5, bar M at least +30: **not called**. Control C = E8 minus token at seed 2 = −1.

The rows were used. Cells in use per block in layer 1, of 240: 229.2 on average (least 203) at seed 2 and 219.25 (least 171) at seed 3; share of reads whose cell moved since the last checkpoint 0.387 and 0.423; share inside the band 0.487 and 0.646. The token arm used 179.7 cells. On real text: 238.8 cells, moved 0.694 (0.912 in layer 2), in band 0.49.

Real text (OASST2, 4,096 steps):

| Job | Next-token hits of 184,542 | Train loss | Seconds | Replies with no end, of 386 |
|---|---|---|---|---|
| no address | 31,718 (17.18%) | 0.99287 | 784 | 339 |
| E8 address | 31,673 (17.16%) | 0.99302 | 1,252 | 343 |

By my own reading of the first 20 replies of each, about 4 are word-ordered without the part (prompts 2, 3, 6, 17) and 3 with it (2, 6, 17); several echo the prompt. That reading is one reader's and is not a score.

Predictions written before the run: M between −10 and +50, held. E8 train loss at seed 2 below the no-address run's by 0.1 or more, missed (0.075). Token arm within 20 of the no-address run, missed (+31). Real-text hits up by 1 to 6 points, missed. Neither real-text job writes replies, held.

Cost: `vast-60444803` $2.97 (down 07:53:52Z) and `vast-60452774` $3.29 (down 08:43:27Z), $6.26 in all, inside the caps.

## Checks run at the head

- Pod, 10 Nov: the crate's library tests with CUDA, 483 passed, 0 failed, 18 ignored of 501.
- Pod, 13 Nov, the pre stage of tries 3 and 4 at the runs' source: the address tests, 54 passed, 0 failed (500 filtered out), in each. The job script ends the stage if no test ran or one failed, and no training step runs before it.
- Pod, 13 Nov, the fix's negative gate: with the fix set aside the two tests fail (exit 101); with it in place, 54 passed.
- Laptop, at the head: the float crate type-checks with `--features packet-parity` under the stub compiler for every target (exit 0, 0 errors). The test binary does not link on the laptop (no CUDA library), so no laptop test is claimed.
- `uor-harness prep-check` on the sheet.

## Skills used

None.

## Still open

1. The seed-1 pair (`e8` and `b`), which the sheet unlocks on "not called". It is queued behind the key read (claude-361), which acts on the losses this run did not touch: never-told questions and told values.
2. Whether the gain belongs to the table's size or to an address: the `collapse` and `sign` arms were built and not trained.
3. The run-on replies under the E8 address. The token arm has the same gain with fewer of them than no address at all (56 against 99), which is the cheaper table to carry forward if a table is carried at all.
4. The served form: the integer runtime's section for the rows and the exporter.
5. Real text: by this run, the next real-text job goes without the part; the limit there is size and context.
6. Test gaps from the review of 13 Nov, to close before the part is trained again:
   - The wall gradient's hand-worked value is checked on one position with one block, where the block's, the position's and the batch's RMS are the same number. A test with two blocks of different sizes and two positions would pin that the scale is the position's.
   - The wall gradient is pinned for the `e8` arm across a wall between two integer roots only: not for the `sign` arm, and not across a half-root wall.
   - For trained rows the trunk test asserts only that the output changed, so the layer a table belongs to and the place of the read after the Hamilton block are not pinned.
   - The telemetry's in-band share and mean row size are checked at zero only.
   - `the_rows_gradient_carries_no_history_of_the_forward_pass` pins the property that caused the lost try (the rows' gradient has no history) and fails when the read goes back to a flat indexed read. It does not separately pin the `detach` on the index, which the read by block makes a second guard.
7. `AddressMemory::load` checks each table's shape and not the count of tables against the trunk's layers (a short checkpoint fails at the first read, loudly); under `--train-as-served` the rows are held inside the 16-bit range on every forward, which the design text does not say; a band given to the `collapse` or `token` arm is stored though those arms have no wall.

## Plain summary

The E8 address memory is built and tested, and it trains. Inside today's model it gave 30 and 29 more exact replies of 1,108 at two seeds, half a reply under the bar set before the run, so the sheet's verdict is "not called". A table of the same size addressed by the input token alone gave the same gain with far fewer run-on replies, so the gain is not shown to come from the state's address. No arm learned to say "not told", told values barely moved, and on real text the part made no difference. It costs 1.57 times the training time.
