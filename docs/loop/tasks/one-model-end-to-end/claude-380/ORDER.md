# ORDER: one-model-end-to-end, claude-380 (V1: train as served, the state carried across windows)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407).
Base: `origin/main` at `d5eb4a55656726b04ea79b06968f91905933c925`. Follows V0, claude-376 (PR #3970). Roadmap: `local/claude/one-vector-roadmap-2026-11-14.md` §4, step V1. Kit, not committed: `local/claude/claude-380/`.

## Goal

**Carry the recurrence's state from one training window into the next, with backprop truncated at K windows, behind a flag whose off setting is the trainer as it is. Then train it, two seeds, against the same recipe with the carry off, and read recall of a value told before the current window.**

**Why now.** Casey, 14 Nov 2026 (verbatim): "I want to focus on training directly into vector space, using vector space for the whole mechanism end to end and serving from vector space". The audit's gap: training cuts back-to-back windows and starts each from a zero state, while the integer session reads the whole conversation (claude-288's carry). Training never matches serving past one window.

## Cancel probe (run first)

- **Would make this unnecessary:** a trainer that already carries state across windows, or a TRAIN where no conversation spans two windows (then a carry trains nothing).
- **Result (at base):**
  - Nothing carries in training. `LayerCarry` and `CarryMode::{Capture, Step}` (claude-278) are decode-only and detach every tensor; `PastWindowRule::Carry` is a scoring rule.
  - **The `p` recipe's TRAIN spans no window.** `corpus-noline` holds 60,000 conversations; claude-358's p-s1 log reads "usable TRAIN windows 59936"; its manifest's longest episode is 246 tokens, inside W = 256. A carry arm on it would train what the off arm trains: a vacuous instrument (§4.2).
  - So the runs use claude-275's **long TRAIN** (`results/claude/one-model-end-to-end-long-context-2026-11-04/inputs/train-long`: 16,000 conversations of 1,703 to 2,048 tokens, 60,444 recall queries) and its held-out **p-long** panel (609 queries, distances to 1,994, 586 at 192 or more). Both are in the store with sums. Decision review in the REPORT.

## The change

- `--carry-windows K` (default 0, off; serialized only when on). On, each step's batch of B windows is B/K streams of K consecutive windows of one conversation, cut back to back with no overlap. The scan starts each window from the state the stream's previous window ended in (zero at a conversation's first window), through the scan's own kernel (one zero-transition position, as `quaternion_scan_step` does). The convolution reads the previous window's last three inputs. The gradient runs through the K windows of a step, then the carry is detached.
- Refused with the reads and controls it does not carry (named in the code), and with micro-batches.
- The Hamming reads and the key ring are not carried: each window's reads see that window only.

## Files in scope

- `crates/uor-r4-float-train/src/in_space_long.rs`, `geometric_stack_native.rs` (the recurrence's carry hook only), `bin/train-in-space.rs` (flag parsing only)
- `docs/harness/prep/one-model-end-to-end-carry-*`, `docs/loop/tasks/one-model-end-to-end/claude-380/**`, `docs/research/notebook/float-training.md`

## Done when

1. Finite tests that can fail, each with a negative gate: the stream cut is back to back and resets at a conversation; carry off writes the config and runs the step it did (a 64-step GPU smoke matches the base binary's losses bit for bit); carry on equals the whole-window forward at the next window's positions (GPU); the gradient reaches K − 1 windows back and stops at the step's first window (GPU).
2. A prep sheet that passes `uor-harness prep-check`, with a fresh non-author FIT, before any training.
3. Four runs (carry on and off, seeds 1 and 2), each read on p-long under `--past-window carry`: queries at distance 256 or more whose reply states the answer, memory on and off. Plus V0's panel: P14 + P23 exact replies with the turn memory dropped (`raw-id-replies --turn-memory 0`) and served tok/s on the laptop.
4. REPORT (negatives first), one notebook row, own fresh exact-head review, `loop pr-check`, `loop queue`, `loop close --verify`.

## Stop list

- No change to the scan kernel, the export or the integer session. No tuning against p-long or P14/P23.
- Outside the files above: stop and re-scope.
