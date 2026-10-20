# Starling read, rung 1: recall at chance when trained from scratch — October 11

Lab: claude. Milestone: [M4 #2032](https://github.com/UOR-Foundation/uor-r4/issues/2032). Line: starling read · count 0/3 → 1/3.
Pre-registration: [#2032 comment 6105228141](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6105228141), amended at [6105276314](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6105276314). `TEST FITNESS: FIT` at [6105293249](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6105293249). Owner direction: the [starling design](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6104705984).

**Result: decision row 5. Both starling arms are at chance.** This closes only the tested configuration: `starling:4:7` and `starling:8:7`, trained from scratch with this address map and gate. It does not close the mechanism (D22 §1). The headline is unchanged: softmax at runtime yes, served 0.886838 BPB.

## The mechanism (`read_route=starling:M:K`)

A softmax-free read in which each query touches only about K geometric neighbours:
1. **Placement.** Each head maps the token to two learned unit 4-vectors, a query address and a key address (separate maps, so the key shift can match). Each address is snapped to the nearest of the 120 vertices of the 600-cell, the unit icosians.
2. **Candidates.** A query's candidates are the last M keys stored in its own cell and its **12 adjacent cells**. That is a table lookup, with no all-pairs score.
3. **Selection.** It keeps the K nearest by S3 geodesic.
4. **Weights.** `R[h][m][r] · sigmoid(β·⟨a,b⟩ + γ)`: a plain learned rank table times a gate. No softmax, no normalisation.
5. **No length wall.** There is no age table, so the model runs past its trained context.

**Tests:**
- The 600-cell adjacency is 12-regular and symmetric.
- Selection equals a brute-force reference; a negative gate fails it when the neighbour cells are removed.
- Gradients match finite differences.
- A length-32 run works at context 12.
- Save/load round-trips.

## The rung

mqar-bench, synthetic multi-query associative recall: context 512, 4 distance buckets × 8 pairs, 1,800 steps, `pattern=rarr key_shift=true`, width 128, heads 4, 3 seeds, M1 laptop CPU, \$0.

| arm | in-class recall per seed | mean | by distance (seed 1: d16 / d64 / d200 / d400) | positions scored per query (mean / at the last position) |
|---|---|---:|---|---:|
| L, L2 softmax read | 0.669, 0.373, 0.809 | **0.617** | 0.666 / 0.645 / 0.676 / 0.689 | 1,427 / 2,048 |
| S7, `starling:4:7` | 0.013, 0.008, 0.006 | **0.009** | 0.029 / 0.012 / 0.004 / 0.008 | 207 / 208 |
| S7w, `starling:8:7` | 0.009, 0.010, 0.009 | **0.009** | 0.020 / 0.010 / 0.000 / 0.006 | 411 / 416 |

Chance is 1/224 = 0.45 %; the pre-registered "at chance" line is ≤ 2 %. Held-out-class accuracy for both S arms is 0.000–0.001. Per-seed figures are in [`result.json`](result.json).

## Diagnosis

- Recall is flat at chance in **every** distance bucket, and M = 8 is no better than M = 4. So it is not ring eviction: the address map never learns to put the right key in the query's neighbourhood.
- **The reason is structural.** Selection is hard, so a key outside the selected K gets no gradient. Nothing pulls the right key toward its query; the read only learns from neighbours it already happens to pick, which at initialisation are unrelated tokens.
- **The design's cost saving is real:** 7× fewer positions scored than the L2 read here (207 against 1,427), with a hard cap of 13·M per head at any context. What is missing is a training signal.

## Next (row 5's pre-registered action)

An address-training change, training only:
1. Begin with soft addresses over all past tokens (address dot product), so every key gets a gradient.
2. Anneal to the hard 7-neighbour 600-cell form, the same pattern as SoftSort's τ schedule.

The served model keeps the hard, softmax-free read. That rung gets its own pre-registration and test-fitness review.

## Code in this PR

- `read_route=starling:M:K` (library and `mqar-bench`), opt-in. The default path is unchanged.
- No served form: export refuses it.
