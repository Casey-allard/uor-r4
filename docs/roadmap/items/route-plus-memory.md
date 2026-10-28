---
id: route-plus-memory
title: Route the user's turn to an exact memory built from TRAIN, and read it at the readout
milestone: research
area: memory
status: ready
open_to: lab
needs_gpu: true
depends_on: []
notebook: docs/research/notebook/float-training.md
scope: [crates/uor-r4-float-train/src/flock_read*.rs, crates/uor-r4-float-train/src/lib.rs, crates/uor-r4-float-train/src/in_space_long.rs, crates/uor-r4-float-train/src/bin/train-in-space.rs, crates/uor-r4-float-train/src/geometric_stack_native.rs, docs/harness/prep/route-plus-memory*, docs/loop/tasks/route-plus-memory/**, docs/research/notebook/float-training.md, docs/roadmap/items/route-plus-memory.md]
---

## Why it matters

The owner, 28 Oct 2026, in claude's chat: "ok yes, my vote is to pivot to your diagnosed recommended new roadmap". This item is that roadmap's decisive run.

The charter's mission is a model that predicts text "with geometry … and exact memory" (CHARTER §1). The trunk line (claude-152 to claude-155) tested geometry alone. Its reads route the prompt into the trunk:
- copies rose (claude-152);
- the bag is used (claude-155: dropping it costs `189 / 236` openings, [float-training notebook](../../research/notebook/float-training.md), row of 28 Oct).

But openings never passed claude-152's `302.0 / 228.0`. A reply's first token is an association ("Note these down" → "Noted."), and the trunk has `64,000` values to store associations in ([claude-155 REPORT](../../loop/tasks/in-space-trunk-to-text/claude-155/REPORT.md), "The pivot"). Exact memory works on its own: with the true address, `80/80` against `0/80` with no memory (PLAN, the exact-memory premise). It is not connected to the trunk.

## What is missing

- **No read takes a key from the user's turn into an exact memory.**
- **The evidence that such a memory carries openings:** claude-145's probe looks up TRAIN's commonest opening by the prompt's first 8 tokens (the turn anchor's slots, `--opening-bound`). It names `607` of DEV's 1,226 openings. TRAIN holds `212,968` openings (claude-154) over `26,450` distinct prompt starts (claude-155). How many DEV keys TRAIN never saw is not recorded: the probe prints it as `dev_unseen`, and the unit reports it.
- **The routing piece that already works:** claude-154's turn anchor. Its first 8 slots equal that key at all 1,226 DEV and 212,968 TRAIN openings.
- **Where a read must land:** where the readout reads. A point added inside the stream was ignored (claude-154). The Hamming readout's move toward a copied token's table point is the form the decoder sees ([pipeline map](../../model/pipeline-map.md), stage 5).
- **Cost:** a lookup is a hash and a load, integer-exact and multiplier-free. It is far cheaper per token than the trunk.

## Done when

1. **A memory read,** float for training with an exact integer form:
   - the key is the turn anchor's first `k` token ids, exact, or a learned code of the bag as a second arm;
   - the table is built from TRAIN only: per key, its commonest next token, or a few with counts;
   - at a reply row, the stored token's table point moves the readout point through a learned weight that starts at zero, as the Hamming readout does.

   Host tests pin the key and the table build. CUDA tests pin that zero weight is no read and that the move equals the stored point's pull. With no new flag, every file a run writes is unchanged.
2. **A pre-registered run** (one sheet, a fresh FIT and pre-run review, CHARTER §4.5):
   - (m) claude-152's (a) recipe plus the memory read;
   - (m0) the same with an empty table, the no-memory control (L-010);
   - (g) claude-155's bag, which holds the item `in-space-trunk-to-text`'s bar, as a second reference;
   - two seeds each;
   - openings read as the mean of the last three checkpoints;
   - (m) barred against (a), (m0) and (g);
   - the per-slot or per-key removal reading reported, as claude-154's I-5 did.
3. **The REPORT** leads with cost per token: training rate and the memory's bytes and operations per token. One notebook row, and the result on main.
