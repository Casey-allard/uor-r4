# ORDER: written-state-count-model, claude-415 (three counter fixes before the full-corpus count)

Lab: claude. Item: [m4-lawful-likelihood-bpb](../../../../roadmap/items/m4-lawful-likelihood-bpb.md) (R\* is its sealed denominator; the track has no item of its own yet). Track: the written-state geometric count model. Base: `origin/main` at `948b6000d`.

## Why this unit

The exact CUDA n-gram counter of claude-412 is about to count the full corpus. Three defects must be fixed first.

## Goal

1. The assembler's cross-check streams over the triples. It no longer allocates a second context vector.
2. `plan_partitions` refuses any partition above 2^30 targets.
3. N-grams never reach back past a document's BOS (id 0). A context may hold BOS only in its oldest slot. A BOS target is never counted. Round 2 does this one.

## Files in scope

- `crates/uor-r4-discrete/src/ngram_count.rs` and its tests (items 1 and 2);
- the counter kernels and oracle (item 3, round 2);
- `docs/loop/tasks/written-state-count-model/claude-415/**`.

## Done when

- GPU = oracle on the real stream.
- Every fix is pinned by a negative gate: the pinning test fails with the fix set aside and passes with it restored.
