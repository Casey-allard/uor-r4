# The model, end to end

UOR-R4 turns text into a short list of whole numbers per word, looks back at a few earlier words by walking a small tree of addresses, and picks the stored word nearest to what it found. It never turns scores into probabilities and never multiplies large grids of numbers. This page follows one piece of text through that whole chain and marks each stage as built, trained, or missing. The honest summary is short. Most stages exist and pass small fixed tests. None of them has learned language yet: the best trained model so far is worse than a plain table of word counts ([K16 outcome](../labs/k16-outcome-call-2026-10-19/README.md)). The project is pre-alpha, and 0 of its 8 milestones are complete (`STATUS.md`).

![A left-to-right pipeline of nine boxes: text, tokenizer, token table, ordered context, reads (Route, Flock, Relation), shared writer with admission, rank-margin loss, discrete learning on the GPU, and export to a D11 integer artifact that is then served. Each box is coloured built, trained or missing, and four dashed empty boxes mark the slots with no mechanism: position, normalization, feed-forward on the discrete path, and an exporter for a trained model.](../figures/model/pipeline.svg)

*The current pipeline. Solid boxes exist in `crates/uor-r4-discrete`. Dashed boxes are slots a transformer fills that this model has not filled.*

## How to read the status labels

- **Built** means the source exists and passes its finite construction controls. These are small, fixed GPU tests that show the code does what it says. A control is not a language result.
- **Trained in** means the stage took part in at least one learning run with a recorded result. It does not mean the run helped.
- **Missing** means no code exists, or no mechanism has been assigned.

Labels follow [`docs/formal_vocabulary.md`](../formal_vocabulary.md). **Proof** means derived from definitions or code. **Measurement** means a recorded run. **Hypothesis** means an untested idea. `NOT_RUN` is never a pass.

## One piece of text, stage by stage

The current model lives in [`crates/uor-r4-discrete`](../../crates/uor-r4-discrete/README.md). The code-level map is [`docs/architecture.md`](../architecture.md).

**1. Tokenizer (built; fixed before training).** Text is cut into numbered pieces by a byte-level BPE tokenizer with 4,096 pieces, identity `d36d3e87`. Every corpus, test panel and trained artifact is pinned to it ([tokenizer](../mechanisms/tokenizer.md)). A 16,384-piece trainer exists as tooling only, and no model has used it.

**2. Token table (built; trained in).** Each of the $V=4096$ tokens owns $B=4$ blocks of 8 whole numbers, so its vector has width 32. Each block is always one of 16 *roots* from a single orbit of the E8 lattice; each root is a signed rearrangement of $(2,2,0,0,0,0,0,0)$. The model does not use all 240 E8 roots. The same table encodes a token as input, carries it as memory, and ranks it as output; that is the shared vector writer. Exact token identity is kept separately from this geometric address.

There are three ways to fill the table at the start, and they matter a great deal:

| Start | What it gives | Source |
|---|---|---|
| Seeded default | At most 16 distinct vectors for 4,096 tokens (**measurement**) | [pilot diagnosis](../../archive/labs/a2-pilot-diagnosis-2026-10-14/README.md) |
| PrimeZeta injective | 4,096 of 4,096 tokens get unique vectors (**measurement**) | [construction](../../archive/labs/prime-zeta-construction-2026-10-14/README.md) |
| Address-structured cells (A8) | 1,351 of 3,847 copy-successor targets reachable at zero training, against 127 for the injective PrimeZeta start (**measurement**) | [pre-flight](../labs/address-cells-preflight-2026-10-19/README.md) |

An injective code is possible in principle, because $16^4=65{,}536\ge 4{,}096$ (**proof**, from the block and root counts). See [prime-zeta-codebook](../mechanisms/prime-zeta-codebook.md) and [address-cells](../mechanisms/address-cells.md).

**3. Ordered context (built; not learned).** The most recent tokens form a window of at most 243 positions ($3^5$). Recorded runs used 192. A ternary tree built by SpiralCore, the project's exact-arithmetic geometry library, puts those positions at its leaves in order ([spiralcore-fbs](../mechanisms/spiralcore-fbs.md)). Which position a read lands on is decided by address, not by a score over all positions.

**4. Reads (built; trained in, but no read's own value is isolated).** A read turns the window into one integer vector $q$ of width 32. There are three, each a hard selection with no softmax:

- [Route](../mechanisms/route-read.md) walks the tree, one nearest port per level, and returns one earlier position's vector rotated along the path.
- [Flock](../mechanisms/flock-read.md) keeps a few nearest earlier positions, rotates each into the current frame, sums them and snaps to a root. It was trained only inside the Relation runs.
- [Relation](../mechanisms/relation-read.md) is Flock plus a rerank along link types such as "the word after the earlier copy". It has trained twice, and both runs stopped before a verdict.

Route is the base read of every trained artifact, yet it received only 20 learning proposals in 16,404 updates ([attention state](../labs/attention-state-2026-10-18/README.md)). Nobody has switched a read off to see what it adds.

**5. Shared writer and admission (built; trained in).** The writer gives each token a *cell* from the root numbers of its first and last block: $c(t)=(\rho_{\text{first}}(t),\rho_{\text{last}}(t))$, so there are $16\times16=256$ cells. It opens the $K$ occupied cells nearest to $q$ by L1 distance (8 in the lineage of record, at most 16), then picks the nearest token inside them. Ties go to the lowest id. No probabilities are formed. This shortlist is [admission](../mechanisms/admission.md).

**Proof.** A target outside the opened cells can never be chosen, whatever the read did. So admission caps everything upstream. Rung 0's top-1 bar was 1,941 rows, but only 1,920 targets were admitted at the end, so the bar was unreachable ([Stage A sweep](../../archive/labs/rung0-stage-a-sweep-2026-10-15/README.md)).

**6. Loss (built; trained in).** The score is an integer [rank-margin hinge](../mechanisms/rank-margin.md): the right token must beat its strongest rival by a fixed gap, and the loss is the shortfall. It has no likelihood, so it yields no bits per byte.

**7. Discrete learning (built; trained in).** One table entry is tried in all 16 legal states on the GPU, and the best is kept only if it strictly lowers the loss. See [training](./training.md).

**8. Export (missing for any trained model).** The learned table is already integers, so there is no float to round. But no exporter turns a trained snapshot into a D11 artifact, and no export gap has been measured. The existing table engine's schemas /1 and /2 serve softmax through exponent tables, and all three schemas stream dense 4-bit maps with SiLU and GELU tables, so a new table-native schema is likely needed (audit reading, `lut-serving` dossier; **hypothesis**).

**9. Serving (engine built; never used for this model).** The D11 engine runs integer artifacts with no multiply instruction. Its audits pass: 44 native symbol ranges ([project track](../integration/project-track.md), M4) and 7 of 7 WebAssembly serving functions ([WASM runtime record](../labs/wasm-serving-runtime-2026-10-18/README.md)) are free of multiplies. It has served only older models, which missed the milestone bar (the 19.9M-parameter stack reached 0.933 bits per byte against 0.90, "NOT MET by 0.033"). Tokens per second for the discrete model has never been measured ([lut-serving](../mechanisms/lut-serving.md)).

## The missing slots

A transformer has parts this model does not. The board lists them as gaps, and no record fills them:

| Slot | What a transformer does | What exists here | Status |
|---|---|---|---|
| Position | Adds a signal for word order | No mechanism assigned. The tree keeps positions in order, and quaternion transport along a sequence is an idea; its only result came from a softmax, sigmoid and dense core, with unmatched parameter counts ([transport record](../../archive/integration/transport-attribution-d1-2026-09-28.md)) | **Missing.** Transport as order is a **hypothesis** |
| Normalization | Rescales activations between layers | Scale-matched E8 orbit-root projection (`OrbitRootSnapNormV1`, [`normalization_mechanism.rs`](../../crates/uor-r4-discrete/src/normalization_mechanism.rs)): snaps an $n$-term 8D block sum to the nearest scaled orbit root $n\rho_r$ under $(L_1, \text{Hamming}, r)$, holding $\|x_b\|_1=4$, $\|x_b\|_2^2=8$, $\|x_b\|_\infty=2$ (4 bits/block). Finite CPU controls pass; GPU trunk run pre-registered in [`m1-normalization-mechanism-DRAFT-2026-10-22.md`](../harness/prep/m1-normalization-mechanism-DRAFT-2026-10-22.md) ([lab record](../labs/m1-normalization-mechanism/README.md)) | **Built (controls); trunk run pre-registered** |
| Feed-forward block | A per-token processing layer | The Hamilton gate and SpiralCore/FBS projection are READY, never trained. E8 memory is PARKED. None is on the discrete path | **Missing on this path** |
| Exporter for a trained trunk | Not a transformer part; the step to a shippable file | The recurrence mode is refused by export, the Hamilton gate "still needs the icosian snap", and `stack_export.rs` is compiled out | **Missing** |

Two further things are also not built: using an address *as* the read (memory by exact name), and a writer cheaper than a scan of the whole vocabulary. The writer's own count is about 5×10⁶ vocabulary-cell comparisons per row (chief architect's review, `docs/labs/course-correction-2026-10-18/audits/d37-reactivation-architect.md` §3.5), which breaks the per-token sparsity rule D5-b at larger sizes. A bounded $K$ limits the shortlist, not the work to build it.

## Where it stands

| Stage | Built | Trained in | Evidence |
|---|---|---|---|
| Tokenizer | yes | fixed | `d36d3e87` pinned everywhere |
| Token table | yes | yes | seeded start has 16 distinct vectors; A8 start admits 1,351 of 3,847 |
| Ordered context | yes | no | 3^5 cap, 192 used |
| Route / Flock / Relation | yes | yes, none isolated | 25 CUDA controls pass (Route and Flock); Relation 9 of 9; verdicts `NOT_RUN` |
| Writer and admission | yes | yes | K16 runs admit 2,237 and 3,167 of 14,287 (objective A) |
| Rank-margin loss | yes | yes | one variant collapsed to 19 or 20 occupied cells |
| Discrete learning | yes | yes | no run beat its control |
| Export | no | no | no exporter |
| Serving | yes (engine) | no | zero-multiply audits green |

**The measured result.** On the 14,287-row development panel, a count of the most common next token scores 1,226 top-1 hits and a context-free successor table scores 6,300 ([host diagnostics](../../archive/labs/host-diagnostics-2026-10-17/README.md)). The best trained discrete model scored 65 and 463 on its two seeds, with admission of 2,237 and 3,167, and both fail the admission gate (**measurement**, [K16 outcome](../labs/k16-outcome-call-2026-10-19/README.md)). The cause is not established. A census showed that training had broken much of the early aliasing, yet top-1 stayed low, so ranking capacity, the read and the schedule all remain open, as does the address key (the architect review, section 3.2, shows it takes only five values).

**Proof (address key).** The address distance uses only the first and last block, and after snapping each endpoint distance takes one of three values, so the sum takes at most five values: 0, 4, 8, 12 or 16. That key must order up to 256 cells, so which cells sit just inside $K$ is largely decided by the arbitrary cell number (chief architect §3.2, read from `discrete.cu`).

## Go deeper

- Training: [how the model learns](./training.md).
- Mechanism pages: [discrete-learning](../mechanisms/discrete-learning.md), [admission](../mechanisms/admission.md), [the mechanism board](../mechanisms/README.md).
- Code: [`discrete.cu`](../../crates/uor-r4-discrete/src/discrete.cu) holds the reads, the writer and the learning kernels. [`metadata.rs`](../../crates/uor-r4-discrete/src/metadata.rs) builds the context plans. [`device.rs`](../../crates/uor-r4-discrete/src/device.rs) is the GPU boundary.
- Audit: the [renormalization audit record](../labs/repo-renormalization-2026-10-19/README.md), which holds the stitching map and the code inventory.
- Terms: [glossary](../start/glossary.md).
