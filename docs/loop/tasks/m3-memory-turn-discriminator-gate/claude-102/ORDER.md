# ORDER: m3-memory-turn-discriminator-gate, claude-102

Lab: claude. Item: [m3-memory-turn-discriminator-gate](../../../../roadmap/items/m3-memory-turn-discriminator-gate.md) (#2855). Base: `023910c63`.

## Goal

The item's done-when reads:

> The last clause of the M3 bar in `docs/integration/project-track.md:116-120`: non-memory conversational prose does not silently overwrite stored facts. It is measured on the pre-registered panel.

Its note (claude-96) adds: "This gate's work is a compiler that also compiles unseen phrasings and still keeps the panel clean". The lawful sieve compiler's own abstention is never a pass of this gate.

claude-94's baseline, for the sieve compiler on world=v2:
- the panel scored false_assert 0/50, false_overwrite 0/50 and clean 50/50;
- R1 (training phrasings filled with development values) was 462/462;
- R2 (unseen development phrasings) was 0/220.

This unit adds an explicit discriminator to the sieve compiler: a compositional marker rule with an exact-explanation margin gate. It then measures on the GPU whether R2 rises while the panel stays at 0/0/50.

## Cancel probe (run first)

**Result at `023910c63`:** no lawful generalizing compiler with a gate exists.
- The sieve compiler compiles only exact training frames (R2 0/220), so its clean panel is not this gate's pass.
- Antigravity's #3079 margin gate sits in `relation_compiler::SavedCompiler`, whose heads are `SparseSoftmax` (CHARTER §3.2) and whose module is test-only.
- **Class:** mechanism work, not an instrument repair.

## The mechanism: discourse markers composed with exact frames

Everything below is derived from the world=v2 **training** phrasings only. Development phrasings and the panel are not read while building.
- **Markers are found exactly.** A word sequence `m` is a marker when some training slot frame `f` of a relation equals `m` followed by another training slot frame `g` of the same relation, with the same suffix. The marker's act is update if `f` is an update frame, and neutral otherwise. In the prime picture, `sig(f) = sig(m) · sig(g)`: `m` is the cofactor that sieving one frame by another leaves.
- **Compile.** A turn compiles when it is a sequence of at most two markers, then a training slot frame aligned exactly as the sieve aligns it today, then a 1 to 4 word value. An update marker on an assert base gives `Correct`. Markers never attach to query frames.
- **The margin gate is exact explanation plus agreement.**
  - Every non-value word must be explained by the markers or the base frame. Any unexplained word leaves the turn `Unresolved`.
  - Every surviving reading must name one expert, one act and one value span, as the sieve requires today.
- **No float, score, softmax or matrix product.** The kernel gains a bounded marker-strip step before today's sieve. The decision stays on the GPU, and tokenizing and the marker table are host data preparation.

## Measured, in one launch

- **The frozen panel:** the three registered counts.
- **R1:** training phrasings with development values, which must stay at least 95% exact.
- **R2:** development phrasings, the generalization this gate is for.
- **Finite CUDA controls,** including marker cases.
- **The registered bar:** panel 0/0/50 together with R1 ≥ 95%. A pass also needs R2 above claude-94's 0/220, so a clean panel by refusal is not a pass. R2's level is reported, with no further bar.

## Files in scope

- `crates/uor-r4-training/**`
- `docs/labs/m3-memory-turn-discriminator-gate/**` (the prep sheet goes here)
- this task folder

## Done when

1. The marker table and the gated compile are in the sieve compiler and kernel, with finite host tests and finite CUDA controls, and the frame table and marker table digests are pinned by tests.
2. A prep sheet with its own fresh FIT and pre-run review is posted on #2855.
3. The GPU run records the panel counts, R1, R2 and tokens/s, with the outputs in the HF store.
4. If the bar passes, the item is set to done with `Closes #2855`. If it misses, the configuration is recorded as negative (D22) and the REPORT says `References #2855`.

## Stop and decide (never stop and wait)

- If no marker is found in the training phrasings, the mechanism is vacuous: record it and stop this configuration.
- A failed finite CUDA control is an instrument fault, to be repaired.
- If the guard refuses a notebook row, the row goes in the REPORT (this item names no `notebook:` page).
