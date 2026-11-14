<!-- freshness: sources=none -->
# Transformer parts and their geometric replacements

**In one paragraph.** A transformer is built from standard parts: a tokenizer, embedding table,
position signal, attention, feed-forward layers, normalization, output layer, loss and optimizer.
UOR-R4 replaces those parts with geometric or exact mechanisms that avoid softmax and dense matrix
multiplication. **M1 is complete (1 of 8)**: the saved discrete model passed all six bars at two
seeds. Useful conversation is still open, and the separate FBS complete-reply line shows why.

## The replacements at their current scope

| Transformer part | Geometric or exact replacement in UOR-R4 | Current state |
|---|---|---|
| Tokenizer | [Byte-level BPE](../mechanisms/tokenizer.md), 4,096 pieces, identity `d36d3e87` | ACTIVE; pinned to corpora, panels and artifacts |
| Embedding table | [Prime-zeta codebook](../mechanisms/prime-zeta-codebook.md) and [address cells](../mechanisms/address-cells.md): prime/zeta addresses on a 16×16 board | ACTIVE; construction and placement results exist, independent language effects remain open |
| Position | SpiralCore transport spin on the FBS trunk; `PositionHolonomyV1` with the discrete quaternion recurrence in the M1 composition | Trained and served in their respective models; independent order effects remain open |
| Attention / look-back | [Route](../mechanisms/route-read.md), [Flock](../mechanisms/flock-read.md) and [Relation](../mechanisms/relation-read.md) reads: tree walk, neighbour consensus and copy links | ACTIVE; independent effects are not all isolated |
| Feed-forward | [Hamilton rotation gate](../mechanisms/hamilton-gate.md) is in M1 and FBS; [SpiralCore/FBS routes](../mechanisms/spiralcore-fbs.md) are on the FBS trunk only | Hamilton trained in M1 and FBS; FBS routes trained and served; dated float projection form READY |
| Normalization | RMS norm with integer square root on the FBS trunk; `OrbitRootSnapNormV1` on the discrete M1 trunk | Each served in its own model; not the same normalization mechanism |
| Output head | [Admission](../mechanisms/admission.md): open a few address cells, then rank inside them | ACTIVE; M1's post-rotation `OuterL1V1` readout is measured |
| Loss | [Rank-margin hinge](../mechanisms/rank-margin.md): right token must beat its strongest rival by a fixed gap | ACTIVE; no likelihood is formed |
| Optimizer | [Discrete learning](../mechanisms/discrete-learning.md): try every legal rotation and keep only strict improvements, including BVG-K | ACTIVE; produced the M1 model |
| Matrix-multiply runtime | [D11 integer serving](../mechanisms/lut-serving.md): integer adds, shifts, tables, multiply and divide, but no float or dense matmul | ACTIVE; the M1 pair and a separate FBS export serve |
| Exact memory | Exact addressed object store and exact turn memory | Oracle and familiar-DEV gains exist; new-phrasing key formation remains the blocker |

A replacement being ACTIVE means it is on a current model path, not that it has proved an
independent advantage. READY means built and tested but not trained in; PARKED means tested forms
failed or broke a rule while the mechanism stays open. See the [mechanism board](../mechanisms/README.md).

![Transformer parts and their geometric replacements, with current status and the M1 composition distinguished from the FBS trunk](../figures/start/transformer-vs-geometry.svg)

*The replacement map at 31 October 2026: M1 is complete, while FBS and durable-reply work continue.*

## What M1 established

The saved discrete 19-table model has exact 14,287-row DEV export/reference agreement and passes B4. It scores:

- DEV **7,002 / 7,001** of 14,287 against unigram 1,226 and T2 6,300;
- Fresh V2 **7,215 / 7,228** of 14,479 against unigram 1,279 and T2 6,582;
- p-f **7,850 / 7,872** of 15,953 against unigram 1,409 and T2 7,375, beyond paired band 81;
- Apple M1 decode **10,876 / 10,566 tok/s** under the bar-f protocol.

p-f has no prompt value-masked templates in TRAIN at K = 1,024, so the model reads new prompt
phrasings into known reply kinds. But 62.5% of p-f's reference replies are verbatim TRAIN replies.
That is not new reply kinds or useful conversation
([M1 close](../labs/m1-close-saved-model/README.md), [Fresh V2](../labs/m1-fresh-panel-transfer/README.md), [bar-f report](../loop/tasks/m1-heldout-phrasing-transfer/claude-211/REPORT.md)).

## The remaining gaps

**Reply opening on new phrasing.** On the separate FBS path, claude-217's teacher-forced p-s openings are 17.4%,
13.7% and 18.6% at three seeds, against about 80% on later tokens. Exact turn memory keys on exact
first token ids and misses reworded prompts. A cycle step ends loops but does not repair the opening.

**A joined model and M4.** The discrete M1 artifact and the FBS trunk do not share a table, trunk
or served session. M4 needs the accepted model's full quality, CLI, session and audit path; `PRLL`
is built and registered, but GPU held-out scoring remains open.

**M5 and M7.** Full-path laptop cost and accepted-model browser/library/CLI parity are unmeasured.
Native M1 serving does not establish browser parity or energy.

**M2 and M3.** Next-token ranking is not complete correct replies or durable memory. The true-address
80/80 oracle is exact-read evidence, not learned address formation.

## Historical K16

K16-A scored 65 and 463 of 14,287, below unigram 1,226 and T2 6,300. K16-R collapsed to 19 or 20
cells. These are dated configuration negatives, not the current best and not a reason to repeat the
same test without new causal evidence ([K16 outcome](../labs/k16-outcome-call-2026-10-19/README.md)).

## Go deeper

- [Where we are](./where-we-are.md) and [where we're going](./where-were-going.md).
- [The model, end to end](../model/architecture.md) and [pipeline map](../model/pipeline-map.md).
- [Mechanism board](../mechanisms/README.md): every replacement, its evidence and reopen conditions.
- [Open questions](../research/open-questions.md): reply opening, joined serving, cost and distribution.
