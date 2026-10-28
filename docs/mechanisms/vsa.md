# Vector-symbolic binding (hypervector codes)

**Status: PARKED.** Trained into the older prose learner, the idea lowered held-out bits per byte by 0.014 to 0.023 on 2 seeds and 2 slices. But that run used a CPU learner (the record says it belonged on the laptop; since 12 October the owner rule D24 bars CPU model compute), and the variants that tried to add geometry failed. The idea is not on the current discrete path. It stays open: what would reopen it is listed below ([native test](../../archive/labs/vsa-native-test-2026-10-09/README.md)).

Labels used below: **proof** means derived from definitions or code, **measurement** means a recorded run, **hypothesis** means an untested idea (see [the charter](../../CHARTER.md), section 4.6). Unfamiliar project terms (K16, A8, admission, NOT_RUN) are in the [glossary](../start/glossary.md).

## In plain words

Give every word a long string of 4,096 bits. To glue two things together, such as a role and a word, flip the bits of one wherever the other has a one (XOR). Applying the same glue again undoes it. To merge many strings into one summary, take a majority vote at each position. To tell whether two strings are alike, count how many positions differ. In the older prose model this gave the next-word score a cheap cue: "was this word recently in the context?" It is a lossy sketch. Exact copies still need an exact lookup.

## What it replaces in a transformer

Part of the key-value memory: the cue that says whether something stored earlier matches the current query. A transformer compares a query against every stored key with dot products. A hypervector sketch compares bit strings by counting differences, using only XOR and bit counts.

![Three words drawn as 64-cell bit strips, each XORed with a role strip and folded by majority vote into one summary strip. Unbinding with one role lights up the matching word, with similarity bars that fall as more items are bundled. Beside it, three code modes drawn as 4,096 dots: far apart for a random hash, 120 clumps for root codes, loosened clumps for the root-plus-residual code, each with its held-out change in bits per byte.](../figures/mechanisms/vsa.svg)

*Bind, bundle and probe on bit strips, and three ways to choose each word's code. Only the random-hash codes helped.*

## How it works

A hypervector has $D=4096$ bits.

- **Bind (Definition).** Bitwise XOR. It is its own inverse: $(A\oplus B)\oplus B=A$. That follows from the definition of XOR (proof).
- **Order (Definition).** A cyclic bit rotation marks position.
- **Bundle (Definition).** A bitwise majority vote over a context window of up to 64 terms, with a fixed tie-breaker.
- **Similarity (Definition).** Hamming distance $d_H$ from a bit count, turned into a bipolar correlation $1-2d_H/D$ in Q1.15 fixed point.

**Cross-talk (standard estimate, proof for random codes).** Unbinding a bundle of $k$ random bipolar terms returns the stored value plus noise with standard deviation about $\sqrt{(k-1)/D}$. So 64 terms in 4,096 bits leave a signal-to-noise ratio near $\sqrt{4096/63}\approx 8$. This is the textbook figure for random codes, not a measurement on this model.

**How the older prose scorer used it.** One additive feature per candidate word:
$$\text{score}\mathrel{+}=\big(\text{vsa\_scale\_q15}\cdot \mathrm{corr}(\text{context vector},\ \mathrm{code}(\text{candidate}))\big)\gg 16 .$$
The context vector bundles an associative transition context and up to four "orthogonal VSA attention" heads. The scale is trained. The codes are not.

**Three ways to choose a word's code.**

| Mode | Code | Distinct codes |
|---|---|---|
| 0, fixed hash | A hash of the seed and the word id. Distances are about $D/2$, so only identity carries signal | 4,096 |
| 1, root | A hash of the word's icosian root (a point from a 120-element rotation group) | at most 120 |
| 2, learned | The root code XOR a hash of a 5-dimensional readout, rebuilt every 1,000 steps | 4,096 |

**Rule check.** The bit operations contain no softmax and no matrix multiplication. The Q15 scale multiply is a runtime product that has not been shown to be table-only, so it is **not audited** against the zero-multiplier serving rule (D11). The host learner that trained the scale was not audited against D23 here. The 9 October run used a CPU learner (the record says it was meant to run on the laptop, owner rule #2095). D24, dated 12 October, now bars CPU model compute. Binary 4,096-bit codes are not part of the current discrete path (D32). D32 welcomes XOR and Hamming comparison on ternary and E8 labels as one component of a model, not as a Hamming-only model.

## What has been tried

| Date | What was run | Result | Verdict | Record |
|---|---|---|---|---|
| Jun–Jul 2026 | First hypervector algebra beside the prime router (UOR-Foundation/uor-r4#9) | Folded into the graph-compiler plan; no model quality recorded | inconclusive | [#9 issue record](../../archive/history/github-archive-2026-10-15/markdown/0009-feat-implement-proof-carrying-semantic-addresses-vsa-algebra.md) |
| 7 Aug | Router geometry switch, spectral against VSA, 500 probes (#434) | VSA scored 0.0000 MRR against 0.7179. A later correction (#493) found the cause: the scorer compared a 1,024-dimension hypervector with a 512-dimension content vector, so every score was exactly 0. Re-grounding both sides gave non-zero scores but still chance ranking, because that router's hypervector is a hash of the text, not a semantic encoder | diagnostic stop | [record, with its corrections](../../archive/geometry_ablation_434.md) |
| 8 Aug | Re-measure of three retrieval negatives (#500) | Three negatives were one dead-cosine artifact (MRR 0.7179 to 0.8542 with a content query). No record shows the VSA arm rerun after the fix | diagnostic stop | [record](../../archive/retrieval_reassessment_500.md) |
| 19 Sep | VSA codebook tied to learned roots for shortlist routing | Recall 8.4–8.8% to 10.7–11.2%; shortlist bits per byte down 0.045 to 0.053, on two disjoint slices | positive | [handoff](../../archive/integration/handoff-2026-09-19.md) |
| 19–28 Sep | Falsification ablation of the September prose model | VSA attention inert (flips 0.7% of decisions; about 0.0013 bits per byte, within noise). Decision D2 rules this a wiring defect, not a retirement | inconclusive | [toolbox](../../archive/integration/geometric-toolbox-2026-09-28.md) |
| 8 Oct | Swap codes in a frozen artifact, then set the scale to 0 (#820) | Change of only −0.0010 to −0.0020 bits per byte. Valid null for "swap codes into a frozen model": nothing had adapted to them | inconclusive | [review](../../archive/labs/vsa-review-2026-10-09/README.md) |
| 9 Oct | Retrain with fixed hash codes against VSA off; 2 seeds, 2 held-out slices, 6,795,016 tokens per run, CPU learner | **Keep.** Held-out bits per byte improved in all four seed-and-slice pairs, by 0.0144 to 0.0226 (for example 1.8366 against 1.8574). Each in-model ablation interval excludes zero | positive | [native test](../../archive/labs/vsa-native-test-2026-10-09/README.md) |
| 9 Oct | Same run with root codes (mode 1) | No gain: +0.0003 and +0.0001 (seed 1), −0.0053 and −0.0054 (seed 2) | negative | same record |
| 9 Oct | Mode 2, root code plus readout residual | Worse than fixed codes in all four cells, by +0.013 to +0.028. Its own VSA term slightly harmful. Moving-codes hypothesis untested | negative | same record |
| 9 Oct | Check of a draft octonion signed-multiplication table for signed binding | Draft table breaks alternativity: $x(xy)=-2e_4+2e_7$ but $(xx)y=-2e_4$ for $x=e_1+e_2$, $y=e_4$. Rejected | negative | [review](../../archive/labs/octonion-substrate-review-2026-10-09/README.md) |

**Reading these honestly.** The one clear gain is real but narrow. The two seeds differ by about 0.015 bits per byte, the same size as the effect, and the cross-arm comparison is a point estimate because the logs did not keep per-position losses. The same model's exact-memory table (the engram) is worth about 0.19 to 0.20 bits per byte, over ten times what VSA adds, so VSA may be duplicating evidence the copy and route heads already supply. The failures close configurations: a wiring and scoring fault, a frozen-artifact swap, codes with only 120 values, and a codebook rebuilt while the heads trained. D22 applies: these close configurations, not the mechanism.

## What has never been tested

- Mode 2 with its codebook frozen after a warm-up. This is the one follow-up the 9 October result justifies.
- A scale sweep on a frozen artifact, and VSA with the exact copy head switched off.
- Role-times-filler binding for relation reads on held-out phrasing, against an equal-bit ordinary codebook.
- Octonion-signed binding against plain XOR at equal cost, using the canonical Fano table.
- Any hypervector binding on the discrete path (GPU, integer writer, ternary or E8 Hamming comparison).
- More than two seeds, more than 6.8 million training tokens, or chat data in place of TinyStories.
- Spectral against VSA retrieval after the dimension and hash-placeholder faults are fixed.
- Whether a 64-term bundle in 4,096 bits carries order and bindings or only membership (hypothesis).

## What would reopen it

1. **A lawful retraining of the fixed-code term**, either as an offline float learner under D37 with a fresh `TEST FITNESS: FIT` review and a D11 integer export gap, or as a Hamming binding on the discrete writer, with owner release where a hold applies.
2. **The freeze-after-warm-up test for mode 2**, to settle the moving-codes hypothesis.
3. **A redundancy test with the exact copy head switched off**, plus a scale sweep (0, 250, 500, 1000, 2000, 4000), so a null cannot be blamed on a tiny scale or on duplicated evidence.
4. **A role-times-filler test on held-out phrasing** against an equal-bit ordinary codebook, reporting the share of decisions that flip, with exact word and occurrence identity kept apart from the sketch.
5. **An equal-cost test of octonion-signed against plain XOR** on order-sensitive relations, using the canonical Fano table and not the rejected draft.

## Also known as

VSA; hyperdimensional computing (HDC); binary spatter code; `Hypervector4096`; HRR-style associative transition context; the `vsa_scale_q15` term and `set-vsa-code-mode`; octonion-signed binding (a design only).

## Code

- [`native_geometric/vsa/hypervector.rs`](../../crates/uor-r4-core/src/native_geometric/vsa/hypervector.rs): XOR bind, rotation, majority bundle, bit-count Hamming, excess overlap bits above $D/2$, 8-bit sector routing, Q1.15 score (`Hypervector2048`, `Hypervector4096`, `Hypervector8192`). Siblings: `accumulator.rs`, `codebook.rs`, `context_engine.rs`, `attention.rs`, `hierarchical.rs`.
- [`native_geometric/vsa/accumulator.rs`](../../crates/uor-r4-core/src/native_geometric/vsa/accumulator.rs): signed `i16` bipolar integer accumulator (`BipolarAccumulator`, `BipolarAccumulator2048`, `BipolarAccumulator4096`, `BipolarAccumulator8192`) with weighted bundling, exact $O(1)$ algebraic subtraction, dyadic shift decay, and top-$k$ seed sparsification; paired with `RollingBipolarContext` and `encode_prime_stride_ngram` in [`context_engine.rs`](../../crates/uor-r4-core/src/native_geometric/vsa/context_engine.rs).
- [`learner/vsa_codes.rs`](../../crates/uor-r4-core/src/native_geometric/learner/vsa_codes.rs): the three code modes. [`jepa_trainer.rs`](../../crates/uor-r4-core/src/native_geometric/learner/jepa_trainer.rs) holds the trained scale.
- [`bin/ablate-prose.rs`](../../crates/uor-r4-core/src/bin/ablate-prose.rs): the held-out scorer with VSA ablations. [`set-vsa-code-mode.rs`](../../crates/uor-r4-core/src/bin/set-vsa-code-mode.rs): the load-time code swap.
- [`semantic/vsa.rs`](../../crates/uor-r4-core/src/semantic/vsa.rs): the router geometry and facet stores.
- [`tests/vsa_conformance.rs`](../../crates/uor-r4-core/tests/vsa_conformance.rs): algebra conformance tests.
- [`spiralcore_operator.rs`](../../crates/uor-r4-core/src/spiralcore_operator.rs): the canonical Fano table, the control for any octonion-signed binding.

## Related mechanisms

[ternary-xor](./ternary-xor.md): the same XOR and Hamming arithmetic; a discrete VSA would live there. [exact-memory](./exact-memory.md): the intended partner, where sketches propose and exact references decide. [spin-weights](./spin-weights.md): mode 1 took codes from icosian roots. [read-binding](./read-binding.md): another route to a read that binds the right value. [flock-read](./flock-read.md): its Hamming distance on coordinates is the discrete cousin. [hopf-observation](./hopf-observation.md): another lossy summary kept beside exact identity. [spiralcore-fbs](./spiralcore-fbs.md): supplies the Fano table.
