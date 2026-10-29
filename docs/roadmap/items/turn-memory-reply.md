---
id: turn-memory-reply
title: Carry the turn memory past a reply's first word
milestone: research
area: mechanism
status: ready
open_to: lab
needs_gpu: true
depends_on: [route-plus-memory]
notebook: docs/research/notebook/float-training.md
scope: [crates/uor-r4-float-train/src/flock_read*.rs, crates/uor-r4-float-train/src/lib.rs, crates/uor-r4-float-train/src/in_space_long.rs, crates/uor-r4-float-train/src/bin/train-in-space.rs, crates/uor-r4-float-train/src/geometric_stack_native.rs, docs/harness/prep/turn-memory-reply*, docs/loop/tasks/turn-memory-reply/**, docs/research/notebook/float-training.md, docs/roadmap/items/turn-memory-reply.md]
---

## Why it matters

The owner, 28 Oct 2026, in claude's chat: "ok yes, my vote is to pivot to your diagnosed recommended new roadmap". Its decisive run, [claude-161](../../loop/tasks/route-plus-memory/claude-161/REPORT.md), confirmed the diagnosis: routing works, and storage was missing. An exact memory keyed by the user's turn doubled reply openings, to `598.7 / 598.3` of 1,226.

But the memory moves only a reply's first row. After the first word, replies are still the trunk's templates ("Got it, your friend is  gets whations …", claude-161). Copies did not rise, `1,713 / 2,333` of 4,130.

## What is missing

- **A memory key for later reply rows.** Options named, none tested:
  - the turn's first `k` ids with the reply's own earlier tokens;
  - the turn's key and the position in the reply;
  - the slot-address key (entity prime × attribute prime, [pointer-key](../../mechanisms/pointer-key.md)), which was never built.
- **The ceiling for each key, by a probe on TRAIN against DEV:** for each key, TRAIN's commonest next token, and how many DEV reply rows it names. This is the `--opening-bound` probe extended past the opening. claude-161's own ceiling was the probe's `607`.
- **Which rows move.** claude-161 named only opening rows. A wider key needs its own rule.

## Done when

1. **The probe:** for each candidate key, the DEV reply rows TRAIN's table names. It splits them into openings, copies and the rest, against claude-161's openings-only memory. This is a count with no GPU (CHARTER §4.5).
2. **The read, extended** to the best key from the probe, with host and CUDA tests in the style of claude-161's, and no change to any file a run writes when off.
3. **A pre-registered run** (a sheet with a fresh FIT): claude-161's (m) recipe with the extended memory, an empty-table control, two seeds. The bar is on top-1, copies and the rest rows against claude-161's (m), with openings held.
4. One notebook row and a REPORT.

## How to start

Read:
- [claude-161's REPORT](../../loop/tasks/route-plus-memory/claude-161/REPORT.md);
- `crates/uor-r4-float-train/src/flock_read_memory.rs`;
- `opening_bound` and `opening_rows` in `in_space_long.rs`.

## Rules that apply

- **Geometry only, no squashing** (CHARTER §3.1–§3.2): the move stays `n + w ⊙ (c − n)` toward a stored point. No softmax over candidates.
- **A memory gain needs a no-memory control** (L-010).
- **Integer-exact keys and tables,** so the result can be served (`turn-memory-integer`).

## Sources

- The owner's words, 28 Oct 2026, in claude's chat (quoted above).
- [claude-161](../../loop/tasks/route-plus-memory/claude-161/REPORT.md), [`docs/mechanisms/pointer-key.md`](../../mechanisms/pointer-key.md) and the [pipeline map](../../model/pipeline-map.md).
