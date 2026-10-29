---
id: turn-memory-integer
title: Serve the turn memory from the integer artifact on the laptop
milestone: research
area: serving
status: ready
open_to: lab
needs_gpu: true
depends_on: [route-plus-memory]
notebook: docs/research/notebook/m4-serving.md
scope: [crates/uor-r4-integer/src/stack/memory.rs, crates/uor-r4-integer/src/stack/mod.rs, crates/uor-r4-integer/src/stack/session.rs, crates/uor-r4-integer/src/stack/format.rs, crates/uor-r4-integer/src/stack/tests.rs, crates/uor-r4-integer/src/bin/fbs-export.rs, crates/uor-r4-integer/src/bin/fbs-toks.rs, crates/uor-r4-float-train/src/fbs_export.rs, crates/uor-r4-float-train/src/flock_read_memory.rs, crates/uor-r4-float-train/src/in_space_long.rs, crates/uor-r4-float-train/src/bin/train-in-space.rs, docs/harness/prep/turn-memory-integer*, docs/loop/tasks/turn-memory-integer/**, docs/research/notebook/m4-serving.md, docs/roadmap/items/turn-memory-integer.md]
---

## Why it matters

The owner, 28 Oct 2026, in claude's chat: "determinism is still probably the goal but like you said we can do the training in float and maybe just export the integer exact…". This is the [pipeline map](../../model/pipeline-map.md)'s R5, for the memory.

[claude-161](../../loop/tasks/route-plus-memory/claude-161/REPORT.md)'s turn memory doubled reply openings, to `598.7 / 598.3` of 1,226, at no measured training cost. Its key and table are already exact integer ids. Only the move `n + w ⊙ (c − n)` is float, and the integer runtime has no memory.

## What is missing

- **The artifact** carries the trunk and the writer's table, not the memory's table or weight. The exporter refuses a model with a memory ("a memory, read selection or read router is not served", `fbs_export.rs`).
- **No per-row float reading of the memory.** claude-161 reports opening counts, not each opening row's decoded id. Parity needs the ids, so the float side writes them (the three float-train files in this item's scope).
- **The integer session** has no memory. It needs:
  - the key from the turn anchor's first 8 ids;
  - the lookup;
  - the move on the integer query at a reply's first decode step, with the weight's products through D11's tables.
- **No parity:** the integer reply openings have not been compared with float's on the same checkpoint.

## Done when

1. **Export and session.** The exporter writes the table (keys and values as integer ids) and the weight (at the exporter's fixed point, rounded as it rounds). The session applies the move at a reply's first step only. `step` and its zero-multiply audit are unchanged, and the memory is off for an artifact without one.
2. **Parity,** on DEV, against the float forward of claude-161's sealed (m) checkpoint at step 16,384 (`caseyallard/uor-r4-store:results/claude/route-plus-memory-2026-10-28/`).
   - The share of opening rows whose integer and float decoded ids agree, and the integer openings count.
   - The float reference runs on the GPU (CHARTER §3.5). The integer side runs on the laptop.
   - The bar is pre-registered in a sheet with a fresh FIT.
3. **Cost on the M1:** `fbs-toks --stages` with the memory, timing its lookup and move beside the writer and the trunk, plus the artifact's added bytes.
4. One notebook row and a REPORT.

## How to start

Read:
- [claude-161's REPORT](../../loop/tasks/route-plus-memory/claude-161/REPORT.md);
- `crates/uor-r4-float-train/src/flock_read_memory.rs` (the float read, its `save`, and its key);
- `fbs_export.rs` and `fbs-export`;
- `stack_step_fbs` in `crates/uor-r4-integer/src/stack/session.rs`.

`integer-hamming-read` and `fbs-serving-kernels` also edit `session.rs`, and the float-side items edit `in_space_long.rs` and `train-in-space.rs`. This item runs when none of those is claimed.

## Rules that apply

- **Serving stays integer and multiplier-free** (D11, CHARTER §3.6): the move's products go through the tables, and the lookup is a hash and a load.
- **Geometry only, no squashing** (§3.1–§3.2).

## Sources

- The owner's words, 28 Oct 2026, in claude's chat (quoted above).
- [claude-161](../../loop/tasks/route-plus-memory/claude-161/REPORT.md) and the [pipeline map](../../model/pipeline-map.md).
