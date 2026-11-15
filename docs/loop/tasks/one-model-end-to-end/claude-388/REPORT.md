# REPORT: one-model-end-to-end, claude-388 (the hard-copy read, eval-only; D41's release recorded as D86)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [ORDER.md](ORDER.md). PR #3997. Base `origin/main` at `a8cf8e36d`, which holds claude-387 (#3991).

**Reading map letter: P.** The hard copy (Flock's payload copied from its first kept pick alone) meets V2(b)'s bar at one shared scale, ×16:

| State | Bucket 1 `R_ask` | Bucket 2 `R_ask` |
|---|---|---|
| Seed 1, 16,384 steps | 0.984 | 0.969 |
| Seed 2, 16,384 steps | 0.969 | 0.965 |

The lowest value, 0.965, is outside the borderline band (0.875 to 0.925).

On the rows whose first pick is right, the copied token decodes back to itself on all of them: 1,000 of 1,000 and 990 of 990. There the median `|p| / |x|` is 0.77 and 0.75. So the native operator needs a **fixed gain**, not a replacing write. The 0.75 is the smallest tested gain that passes at both seeds, an upper bound on what is needed: seed 1 already passes at 0.19 (×4), seed 2's threshold lies in (0.19, 0.75] on this grid, and every gain from 0.75 to 49 passes. The control, the mean payload, reproduces claude-387's numbers exactly: at best 0.367 / 0.330 and 0.363 / 0.328.

As the sheet's FIT said beforehand, this is not new evidence that the read *learns*. The first pick was already right on 97% of rows. What the run settles is the write's form: one copied pick, at three quarters of the stream's size or more, decodes to its own token every time.

## What did not work, first

1. **At the trained scale (×1), the hard copy alone misses the bar.** Seed 1 gives 0.631 / 0.643 and seed 2 gives 0.096 / 0.145. The trained weight stays small: `|p| / |x|` is 0.048 and 0.047 with one pick, against 0.026 for the mean of four. The copy needs about a 16-fold gain that no trained parameter supplies today.
2. **×4 passes at seed 1 only.** Seed 1 gives 0.984 / 0.969 at ×4, but seed 2 gives 0.836 / 0.844. The smallest scale that passes at both seeds is ×16.
3. **The 4,096-step states stay capped at their scorer's accuracy.** With the copy at ×256 and ×1,024, `R_ask` is 0.416 / 0.400 and 0.443 / 0.482. Those equal the share of rows whose first pick is right: 418 and 474 of 1,024. Every first-right row decodes right. These states sit outside the bar and show N1's limit, the scorer.
4. **No new training, and no native code.** This is a float reference diagnostic on saved float states (D85). The native Q31 read operator is the next unit.
5. **The donor pass is not run here.** Part 1 only records Casey's release (D86).

## What changed

- **D86, Casey's D41 answers recorded verbatim** (ORDER Part 1). The record is in four places:
  - a row in `docs/integration/DECISIONS.md`;
  - an entry in `docs/rules/changes.md` (§3.10; the charter's text is unchanged);
  - an owner-direction row and an updated M2 route sentence in `PLAN.md`;
  - notes in `m2-donor-corpus-generation` and `d41-driver-math-pilot`, both now `ready`.

  What D86 decides:
  - gpt-oss-20b on wick, in place of gpt-oss-120b on a pod;
  - up to four hard answer texts per prompt;
  - the driver-math gate is dropped, and the ~1M-token pilot stays first;
  - (b) embedding import is deferred, and (c) and (d) stay banned.
- **`vector_reads.rs`:**
  - `with_first_pick_payload(f)` is eval only and off by default. Inside it, Flock's payload is weighted by its first real kept pick alone (`count` = 1), and every other kind is unchanged.
  - The default forward runs the same ops (`real_f.clone()`).
  - Two tests are new. A host test checks that the setting is off by default and restored, including after an error. A cuda test, `the_hard_copy_adds_the_first_pick_alone`, checks three things: Flock's added row is `w ⊙` the first chosen position's next-token row; a fixture row exists where that differs from the mean; every other kind is bit-identical.
  - The default Flock forward stays pinned against the mean reference by the existing `the_forward_matches_the_plain_reference_for_every_kind`.
- **`in_space_long.rs`:** `PayloadProbeSettings.first_only`. Every probe pass with the read runs under the copy when it is set; the cut pass has no read.
- **`recall-payload-probe`:** `--first-only` (parsed and tested).

## Measurements

The eight probes ran on wick, job c388, at `90b588fd2`. Each probe reads 1,024 held-out query rows, 512 per far bucket.

- **"FR"** = the rows whose first pick is right: 1,000 at seed 1 and 990 at seed 2 at 16,384 steps.
- **"self"** = how many of those decode to the first pick's token.
- **"ratio"** = the median `|p| / |x|` on those rows.

**Hard copy, 16,384 steps:**

| scale | s1 `R_ask` | s1 self | s1 ratio | s2 `R_ask` | s2 self | s2 ratio |
|---|---|---|---|---|---|---|
| 1 | 0.631, 0.643 | 652 | 0.048 | 0.096, 0.145 | 123 | 0.047 |
| 4 | 0.984, 0.969 | 1,000 | 0.192 | 0.836, 0.844 | 860 | 0.187 |
| **16** | **0.984, 0.969** | **1,000** | 0.770 | **0.969, 0.965** | **990** | 0.747 |
| 64 | 0.984, 0.969 | 1,000 | 3.079 | 0.969, 0.965 | 990 | 2.986 |
| 256 | 0.984, 0.969 | 1,000 | 12.31 | 0.969, 0.965 | 990 | 11.95 |
| 1,024 | 0.984, 0.969 | 1,000 | 49.26 | 0.969, 0.965 | 990 | 47.78 |

**Control: the mean payload** (claude-387 reproduced), 16,384 steps:

| scale | s1 `R_ask` | s1 self | s1 ratio | s2 `R_ask` | s2 self | s2 ratio |
|---|---|---|---|---|---|---|
| 1 | 0.014, 0.023 | 18 | 0.026 | 0.000, 0.010 | 5 | 0.026 |
| 16 | 0.367, 0.330 | 349 | 0.423 | 0.258, 0.238 | 242 | 0.413 |
| 64 | 0.363, 0.328 | 346 | 1.692 | 0.363, 0.328 | 339 | 1.653 |
| 1,024 | 0.309, 0.232 | 270 | 27.08 | 0.285, 0.250 | 261 | 26.45 |

**The read cut** (`R^{-v}`): 0.000 / 0.004 at seed 1, 0.002 / 0.002 at seed 2.

**Hard copy, 4,096 steps** (outside the bar). FR is 418 at seed 1 and 474 at seed 2.

| scale | s1 `R_ask` | s1 self | s2 `R_ask` | s2 self |
|---|---|---|---|---|
| 64 | 0.365, 0.371 | 376 | 0.322, 0.350 | 344 |
| 256 | 0.416, 0.400 | 418 | 0.443, 0.482 | 474 |

**Results:** `local/claude/claude-388/results/c388/` holds `copy-*.json`, `mean-*.json`, `SHA256SUMS` (checked on the laptop against the container's) and the job log. The HF store has `results/claude/one-model-end-to-end-hard-copy-2026-11-15/c388-results.tar.gz`, sha256 `ed68bbd4b89c43629d3715f676006d43a8ef17fff11cabdd63e12201f1687314`. Per file:

| File | sha256 |
|---|---|
| `copy-ext-s1` | `ccf05fc5…ce3d331b` |
| `copy-ext-s2` | `c1d790f1…5a72532f` |
| `mean-ext-s1` | `8e8df5ae…b7a9f51e` |
| `mean-ext-s2` | `e4b6b19f…0cc46227` |

## Checks run at the head

- **Laptop (`aarch64-apple-darwin`), at `90b588fd2`:**
  - `cargo test -p uor-r4-float-train --lib vector_reads`: 9 passed.
  - `cargo test -p uor-r4-float-train --bin recall-payload-probe`: 1 passed.
  - `cargo clippy -p uor-r4-float-train --lib --bin recall-payload-probe`: no finding in the changed lines. With `--tests` added, the `packet-parity` test fails to compile on an unresolved `uor_r4_cpu_train`. That is a file this unit does not touch, and the failure is already there on `main`.
  - `rustfmt --edition 2021 --check` on the binary: clean. `vector_reads.rs` is not formatter-clean on `main`, so it was not reformatted.
  - `uor-harness check`: 0 errors (402 warnings, all branch hygiene that predates this unit).
- **wick (`sm_89`), job c388, the tree synced at `90b588fd2`:**
  - `cargo +1.97.1 build --release --locked -p uor-r4-float-train --features cuda --bin recall-payload-probe`: binary sha256 `7f8b6d642f57c62d8dd86a34853483276273c7d0b9310c48720dcfbd7b709510`.
  - `cargo test --release --locked … --features cuda --lib vector_read -- --test-threads=1`: 26 passed, among them `the_hard_copy_adds_the_first_pick_alone` and `the_payload_probe_reads_the_forward_and_leaves_it_bit_for_bit`.
  - The binary's test: 1 passed.
- **The sheet's FIT:** a fresh-context non-author review returned `TEST FITNESS: FIT` before the run. Its findings were folded into the reading map at `8c2f0b962`, before the job started.

## Skills used

- `native-space-cycle`: read in the ORDER. None of the five items is met; this is a float reference diagnostic, an inventory-class unit.
- The loop's ORDER/REPORT form, with a one-page sheet and its FIT.
- The wick kit (`local/claude/wick/wick.sh`: sync, job, wait, get), with a completion watcher armed.
- `uor-store` for the results.

## Still open

1. **The native Q31 read operator** (the architects' step 2, D85). It is one typed operator called by the native training step and by `IntegerStackSession`:
   - an integer argmax over the carry-keyed scores, taking the first pick;
   - the copied token's point written with a fixed gain sized from this run: 0.75 of the stream passes at both seeds, and the true threshold may be lower (a finer scale grid between ×4 and ×16 would find it);
   - a bit-identical replay of picks and tokens.

   Its first check is this task's `R_ask` reproduced natively.
2. **The trained gain.** The float read's weight did not grow to the size the copy needs: ×1 gives 0.63 and 0.10. In the native operator, the gain is either fixed or trained under a target size. That choice is the operator unit's.
3. **The scorer's training,** for states where the first pick is wrong (the 4,096-step states cap at 0.41 and 0.46).
4. **`with_first_pick_payload` is eval-only by its doc comment alone.** It has no runtime guard against use while a training step collects gradients. Nothing calls it there today. A `debug_assert!` on the collecting state would make the rule checkable; this float read is not trained further (P), so the guard belongs to the native operator's own eval switch, if it has one.
5. **The D41 pilot** (`d41-driver-math-pilot`): gpt-oss-20b on wick, with its own sheet and FIT. It can run beside the operator unit, since it shares no file with it.

## Plain summary

Five architects agreed that the model's look-back read found the right fact but blurred it with three others. This unit tested the fix without retraining: copy only the best match. On both saved models it now recalls the asked fact 96.5% to 98.4% of the time, against 33% to 37% before and a 90% bar. That holds once the copy is made three quarters the size of the model's running state or larger; a smaller copy may also do, but the run did not test sizes between a fifth and three quarters. The next step builds that copy into the native integer model, used the same way in training and serving.

Casey's decisions on distillation are also recorded:
- a one-time donor corpus from gpt-oss-20b on his own GPU, up to four answers per prompt, after a ~1M-token pilot;
- copying a model's weights or probabilities stays off the table.
