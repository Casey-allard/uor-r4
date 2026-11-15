# REPORT: one-model-end-to-end, claude-381 (V2(a): three vector-native reads behind flags)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [ORDER.md](ORDER.md). PR #3978. Roadmap step V2(a) of the 14 Nov audit; V0 is claude-376 (#3970), V1 runs beside it as claude-380.

The order asked for three state-keyed look-back reads in the float trunk, each behind its own flag, with a flags-off step bit-identical to today's `p` (claude-358). This unit trains nothing: it is a float reference port (D85) and an inventory-class unit under the native-space skill. There was no bar, so the notebook word is **inconclusive**: a qualification, with the selection test (V2(b)) pre-registered for the next unit.

**What is now true, at the code head `6a3d80ff54d8a6494205aae567cc78a4b748c646` (wick try-5; the smokes ran at `a62fe168f`, whose read forward is the same):**
- **Flags off is the same training.** Base `d5eb4a556` and head train 20 steps of `p` on wick's RTX 4090 with identical eval lines (3 of 3) and all 7 checkpoint files byte-identical.
- **Each read is checked on the device.** 11 of 11 cuda tests matching `vector_read` pass: forward against a plain-f64 reference for every kind, the credit's gradient against central differences, the Starling gate over an offset sweep (no read at 0, some rows at others), and the trunk itself: a read set at a middle and the last depth is in `StackModel::hidden` (weight 0 changes no value, 0.5 changes the points), and once cleared the trunk's points are bit-identical to before.
- **Each read trains for 256 steps with no NaN,** at 1.23–1.37 times the flags-off step time on a shared card.

## What did not work, first

1. **The Starling reads never opened.** At the initial offset 2.5, both `--starling-read lorentz` and `--starling-read l2` read 0 of 522,240 rows at steps 128 and 256 (`[vread]` lines): the best score `offset - distance` never reached above 0, so no payload was added. The addresses are folded sums of 16 lane groups of a 256-wide state, so their distances sit well above 2.5, and the credit's threshold terms move the offset by about `lr` a step, about 0.26 in 256 steps. This is an instrument setting, not a mechanism result: the offset's start must come from the address scale (for example the median earlier-position distance of the first batch), a geometric fix V2(b) needs before its Starling arms mean anything. Not repaired here; it changes no flags-off step.
2. **The credit at weight 1 costs the trunk early.** Dev top-1 at step 256, of 14,287: flags off 88; Lorentz 5, L2 4, prime-route 4, Flock 10. The Starling arms read nothing, so the cost is the credit's gradient through the addresses into the stream, not the payload. Last train losses 1.0032 (off) against 1.0045–1.0048. One seed and 256 steps: a smoke, not a result. V2(b)'s sheet keeps credit 1; its FIT should weigh a smaller weight or a stop-gradient on the stream under the address.
3. **The Lorentz score is the hyperboloid chord, not the arccosh distance.** The vendored candle refuses the CUDA `log` kernel (D29: `ulog_f32`), so `arcosh(1 + e) = ln(1 + e + sqrt(e (e + 2)))` cannot run on the device; all three cuda tests failed on it at `27d3dd0c9`. The score now uses the Minkowski chord between the lifts, `sqrt(2 (cosh d - 1)) = sqrt(2 e)`, which orders candidates exactly as `d` does (a laptop test pins the chord against arccosh and its order). Every hard choice is the same; the hinge's margins are measured in chord units. A deviation from the ORDER's "hyperbolic distance".
4. **Two more device-only faults, each caught by the cuda tests before any smoke:** Flock's chosen position came back flat `(rows)` where the credit's counts expected `(batch, time)` (fixed at `14d8dfc70`); and the gradient test restored a variable from a tensor sharing its own storage, which candle refuses. The gradient check then missed by 4.3e-4 on one Lorentz entry (difference −0.00811, autodiff −0.00768), the size of f32 rounding in a loss differenced at h = 1e-3; it now differences the f64 reference credit, which the forward test ties to the device's at 1e-5 (`a62fe168f`). Four wick tries to green; a fifth (`6a3d80ff5`) added the review's tests.
5. **The laptop's stub-nvcc check could not cover this crate.** The vendored `candle-kernels` build reads each PTX's `.target` line and export list, which a stub cannot fake honestly; the CUDA compile ran on wick instead, at every head.
6. **The step times are on a shared card.** claude-380's training ran on the same 4090 throughout (one, then two `train-in-space` processes, 2.75 GB each, GPU 65–100% busy). Flags off measured 0.2033 s a step against 0.1742 s alone (claude-370), so every time below is inflated by the neighbour and is a ratio, not a rate.
7. **Not done:** no read was trained past 256 steps, none was scored on replies, and none has an integer form (V2(c)). `--recall` refuses a vector read, so V2(b) builds that path first.

## What changed

- **`crates/uor-r4-float-train/src/vector_reads.rs` (new, 1,424 lines).** `VectorReadKind` (`StarlingLorentz`, `StarlingL2`, `PrimeRoute`, `Flock { neighbours }`), `VectorReadArm`, `VectorRead::{new, apply, save, load}`, thread-local `set_vector_read`/`vector_read`, `collecting`, `take_credit`, `take_stats`, and `reference`, every closed form again in plain f64.
  - **Address:** the state turned lane by lane by learned unit quaternions (`hamilton_gate`), folded by a fixed sum to 16 coordinates, so every all-pairs tensor is `(batch, time, time, 16)`.
  - **Scores, from elementwise differences only (§3.3):** Starling `offset - chord` (Lorentz) or `offset - |q - k|` (L2), no read when the best is at or below 0; prime-route names each 4-coordinate block's nearest of the 120 icosians, cell `i` takes the `i`-th prime, phases `wrap(gamma_c ln p_i)` on the 8 zeta channels `[0, 1, 2, 3, 5, 8, 13, 21]` of `uor-r4-core`, least summed `|wrap|` energy wins, ties to the latest, gradient through `-L1`; Flock the K nearest by mean L1 (`flock_read::choose_nearest`), payloads averaged.
  - **Payload:** the shared table's point of the token after the chosen position, times a straight-through factor of the chosen distance, through a weight that starts at 0.
  - **Credit:** a margin hinge from each row's best positive (an earlier position followed by the row's own next token) to **every** earlier negative, plus Starling's threshold terms, averaged over the terms and times the credit weight. Collected only inside `collecting`, so an eval forward keeps no credit graph.
- **`geometric_stack_native.rs`:** `trunk()` applies the read set on the thread after layer `depth`; `carried_refusal` names it.
- **`in_space_long.rs`:** `LongConfig::vector_read` (left out of `config.json` when absent); the read starts in `run_long` with its own `FlockAdamW`, steps at the trainer's rate, saves `vector_read.safetensors` with each checkpoint and prints a `[vread]` count line; `step_gradient` adds the credit inside the training forward; refused with `--init-from`, `--recall` and `--train-as-served`; cleared by the arm guard; loaded with a checkpoint.
- **`bin/train-in-space.rs` (flag parsing only):** `--starling-read lorentz|l2`, `--prime-route-read 1`, `--state-flock-read K`, `--vector-read-depth D` (the last layer when left out), `--vector-read-margin X` (1), `--vector-read-credit X` (1); one read at a time; training with `--out` only. Test `the_vector_read_flags_name_one_read`.
- **`fbs_export.rs`:** no code change; the export's allowlist already refuses the file. Test `a_checkpoint_with_a_vector_read_is_refused`.
- **Scope change (D68),** in the ORDER's commit: `vector_reads.rs` added to the item's `scope:`.
- **The V2(b) sheet:** [one-model-end-to-end-vector-read-selection-2026-11-14.md](../../../../harness/prep/one-model-end-to-end-vector-read-selection-2026-11-14.md), `TEST FITNESS: PENDING`.

**Size:** 1,778 lines added and 5 removed over 8 files, 1,424 of them the new module. The ORDER gave no size estimate, so there is no ratio to report.

## Measurements

wick, RTX 4090 (`sm_89`), shared with claude-380. Binary `a62fe168f` `ff501b6718b6942fae41dd8595c8834325d669f383851edeb29461ff05497c77`, base `d5eb4a556` `2fa2a89d5d811c7a2bfee1101ec123b4162106daed4861376103e0a5bc0a655a`. The `p` recipe (claude-358), seed 2.

**Flags off, 20 steps, eval every 10 (`off-identity.txt`):** eval lines identical, 3 of 3; at step 20 `model.safetensors`, `hamming_read.safetensors`, `turn_memory.safetensors`, `turn_memory_reply.safetensors`, `hamming_place.json`, `config.json` and `.checkpoint-ready` identical, none on one side only.

**Smokes, 256 steps, eval every 128, read after layer 6 of 6, margin 1, credit 1:**

| Arm | s a step | × off | card peak MiB | last train loss | dev top-1 (256) | dev loss (256) | rows read (256) | chose a positive (256) |
|---|---|---|---|---|---|---|---|---|
| off | 0.2033 | 1.00 | 6,287 | 1.00320 | 88 | 2.77775 | – | – |
| Starling Lorentz | 0.2507 | 1.23 | 9,300 | 1.00471 | 5 | 2.79355 | 0 / 522,240 | 0 |
| Starling L2 | 0.2663 | 1.31 | 9,300 | 1.00473 | 4 | 2.73790 | 0 / 522,240 | 0 |
| prime-route | 0.2693 | 1.32 | 9,364 | 1.00484 | 4 | 2.76185 | 522,240 / 522,240 | 137,488 |
| Flock 4 | 0.2787 | 1.37 | 9,236 | 1.00455 | 10 | 2.77398 | 522,240 / 522,240 | 188,657 |

- **Card peak** is the whole card, the neighbour's 2.75–5.5 GB with it; the read arms add about 3 GB of `(batch, time, time, 16)` tensors at batch 16 and context 256.
- **Positive rows:** 402,974 of the 522,240 rows at step 256 have an earlier position followed by their own next token. Flock's first kept neighbour is such a position on 188,657 (36%, up from 153,978 at 128), prime-route's choice on 137,488 (26%, up from 127,428). Descriptive: no control was run.
- **Credit terms** a checkpoint interval: 44.5M (Starling, with the threshold terms), 37.9M (the others).

## Checks run at the head

- **Laptop:** `cargo test -p uor-r4-float-train --lib vector_read` 6 passed; `--lib in_space_long` 199 passed; `--lib fbs_export` 19 passed; `--bin train-in-space` 15 passed. Clippy clean on every changed line (two pre-existing warnings in `in_space_long.rs` untouched). `rustfmt --edition 2021` on the new file only.
- **wick (`job-381.sh build-u`):** at `a62fe168f`, `cargo +1.97.1 test --release --locked --features cuda --lib vector_reads` 8 passed; at `6a3d80ff5` (binary `529811ad…cec3`), `--lib vector_read` 11 passed and `--bin train-in-space` 15 passed; the flags-off identity (`off`) was run again at `6a3d80ff5`: eval lines 3 of 3 and 7 of 7 checkpoint files identical.
- **Receipts:** every log of try-4 and try-5 (no checkpoints) is in `local/claude/claude-381/sealed/`, 287 files, `SHA256SUMS` sha256 `e2c4215adf19fb8f1d79523ac73ab5c2a864febf2ece962bc8de57c575e2e78a`.
- **Review (fresh context, three slices):** the forward path and the plumbing came back APPROVE with minor findings; the tests came back CHANGES on one item, a set-then-cleared read was not tested on the trunk itself. Repaired at `6a3d80ff5` with that test and a Starling gate sweep; the minor items are under Still open.
- **Banned ops:** `grep -nEi "softmax|sigmoid|tanh|gelu|softplus|\.exp\(|matmul|einsum|log_sum_exp"` on `vector_reads.rs`: no line. The device path has no `log` either; host-side `ln` builds the phase table and the f64 reference only.
- **Negative gates on the device:** each of the three fixes above was preceded by its failing wick run (tries 1–3), the failing output kept in `/workspace/out/c381/try-N/build-u/run/gpu-tests.out` until cleanup.

## Skills used

`native-space-cycle` (read before the work: five items, none met, so an inventory-class unit, recorded in the ORDER); the loop's ORDER/REPORT form; wick kit `local/claude/wick/wick.sh` with a completion watcher on every chain.

## Still open

- **V2(b):** its sheet needs a non-author FIT, the `--recall` path with a vector read, an eval-only cut reading, and a Starling offset started from the address scale (item 1). Credit weight and a stream stop-gradient are FIT questions (item 2).
- **V2(c):** the winner's integer form, its native state schema, shared operator and bit-identical replay oracle (D85).
- **A shared-card time:** the step-time ratios want one quiet-card repeat before V2(b) sizes its cost.
- **From the review (all minor, none repaired here):**
  - **Prime-route memory.** The phase gap is `(batch, time, time, 32)`, four blocks times eight channels, twice the 16-wide address tensors. A detached 120 × 120 cell-energy table indexed by the cell pair would give the energy at `(batch, time, time)`. Do this before V2(b) raises the context.
  - **Lorentz precision.** In f32, `e = lift_q lift_k - (|q|^2 + |k|^2 - |q - k|^2)/2 - 1` cancels when the addresses' norms are large, so close candidates can swap order against the f64 reference. Relatedly, `lift_q lift_k` is an outer product of one scalar per row, with no sum over coordinates: it is not a matmul, but it is the one product between positions in the read.
  - **Straight-through factor.** Its gradient (the payload's dependence on the chosen distance) is not pinned by any test. The gradient test differences only the credit, and the factor's value is 1 in the forward pass.
  - **Ties.** Equal scores are never exercised. Flock breaks ties to the earliest position (an argmin), the other reads to the latest. The module doc now says so.
  - **Signed zero.** A zero-weight read turns a `-0.0` stream entry into `+0.0`: equal values, different bits. This holds only while a read is set; with the flags off, nothing changes.
  - **Credit weight under micro-batching.** It is per window, where the cross-entropy's share is per scored row, and a micro-batch with no scored row adds none. With one forward per step, as in every run here, the weight is 1.

## Plain summary

The three new reads are in the trainer behind their own flags, and with the flags off training is byte-for-byte the same as before. On the GPU each read's forward pass, its credit's gradient and its zero-weight behaviour are checked against a plain reference, and each trains 256 steps without NaN at about 1.2 to 1.4 times the step time. The two Starling reads never actually read at their starting setting, and the credit as set costs the model early accuracy; both are settings for the next unit's selection test to fix before it compares the reads.
