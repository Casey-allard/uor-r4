# The pipeline map

This page follows one token from the training data to the last word of a reply served on an Apple M1 laptop. For each stage it gives seven things, cost first:
- its cost per token, measured where a profile exists;
- the mechanism;
- the exact domain it reads and the domain it writes, and whether the next stage reads that domain;
- its size now and the size the goal needs;
- its hardware;
- its status;
- what it depends on.

It answers the owner's direction of 27 October 2026 (recorded verbatim in the item [`pipeline-map`](../roadmap/items/pipeline-map.md)). The goal is a local model of about GPT-4's ability, trained on less than a datacenter and served on a laptop like this one.

**This revision (claude-159, 28 Oct)** adds:
- claude-149 to claude-158, as listed under [what changed since the first map](#what-changed-since-the-first-map);
- the owner's 28 Oct criterion: "I don't care what we serve at long as it is cheap". Every stage now leads with its cost per token, and the gaps are ranked by what blocks a cheap, capable model first;
- a section on [the archived prime and angular router](#the-archived-prime-and-angular-router).

The page ends with:
- the scale arithmetic for the goal;
- the gaps, ranked;
- an order of work.

**Labels.**
- **Measurement:** a recorded run; its source is named.
- **Proof:** follows from the code or from arithmetic on stated numbers.
- **Hypothesis:** untested.
- **External:** a published figure from outside the project, with the publication named under [Sources outside the project](#sources-outside-the-project).

Code line numbers are at `5bc6f6522`. [Float-training notebook](../research/notebook/float-training.md) rows are named by unit and date, because the notebook adds new rows at the top.

**The hardware this page targets** (measurement, `sysctl` on this laptop, 27 Oct; the same machine as [claude-126's timing](../loop/tasks/d11-exporter-float-trunk/claude-126/REPORT.md), line 73, and [claude-158's profile](../loop/tasks/fbs-cost-profile/claude-158/REPORT.md)):
- Apple M1, with 4 performance and 4 efficiency cores;
- 16 GiB of memory (17,179,869,184 bytes);
- macOS 27.

The M1's other units are an 8-core GPU and a 16-core Neural Engine (external, Apple). Training runs on one RTX 5090 per pod.

## What changed since the first map

| Unit | Result | Where it lands |
|---|---|---|
| claude-149 | The map's units became roadmap items. | Order of work |
| [claude-150](../loop/tasks/one-decoder/claude-150/REPORT.md) | One decoder: the writer held at seed 1 (`8,430` top-1, nearest `6,762`) and never learned at seed 2 (`493`, nearest `6,909`). **Decision: serve whole-table nearest.** | Stage 7 |
| [claude-151](../loop/tasks/in-space-trunk-to-text/claude-151/REPORT.md) | The Hamming read's type spin, built from SpiralCore's quaternion group: negative (openings spin/twin `208/298`, `291/200`, `234/302`; seed 2 passed the openings bar, and no checkpoint met the done-when). | Stage 5 |
| [claude-152](../loop/tasks/in-space-trunk-to-text/claude-152/REPORT.md) | The cell objective (only the nearest wrong token pushed): top-1 `9,328 / 10,573`, the best final checkpoints then. Copies `1,517 / 2,933` of 4,130. Openings `306 / 245`. | Stages 4, 5, 9 |
| [claude-153](../loop/tasks/in-space-trunk-to-text/claude-153/REPORT.md) | Width 512 with 8 heads: the best fit, `10,152 / 11,140`. It failed the prompt-use bar (copies `2,394 / 3,265`), at about half the training speed (`17,033` against `38,578` with no read). | Stages 4, 9 |
| [claude-154](../loop/tasks/in-space-trunk-to-text/claude-154/REPORT.md) | The opening read: negative (openings `206 / 156` against `302 / 228`); removing it changes `25 / 19` of 14,287 tokens. | Stage 5 |
| [claude-155](../loop/tasks/in-space-trunk-to-text/claude-155/REPORT.md) | The plain bag of the user's turn **meets `in-space-trunk-to-text`'s done-when** (openings `217 / 275`, `R16` `0.166 / 0.155`). The learned register collapses training (`T` `684 / 1,246`). **Diagnosis:** routing works, storage is missing. | Stages 5, 6; the gaps |
| claude-156 | The owner's pivot became three items: this one, `fbs-cost-profile` and `route-plus-memory`. | Order of work |
| [claude-157](../loop/tasks/fbs-cost-profile/claude-157/REPORT.md) | One training step by stage, on an RTX 5090 (all shares are of the step). **(a), width 256 with 4 heads:** backward 62%, whole-table search 13%, the two Hamming reads 16%, the trunk's forward 5%. **Width 512 with no read:** backward 54%, search 34%, the trunk's forward 9%. | Stages 3, 4, 5, 7, 9 |
| [claude-158](../loop/tasks/fbs-cost-profile/claude-158/REPORT.md) | One served token by stage, on the M1: a step of 3.73 ms, 3.77 ms a decoded token (265 tok/s). The six layers are 98.6% of the step, the writer 0.6%, and a whole-table nearest would be 21 times the writer. | Stages 4, 7, 11 |

## Two model paths

The repository trains two different models, and they do not meet:

| | Float in-space trunk ("FBS trunk") | Discrete path |
|---|---|---|
| Code | [`crates/uor-r4-float-train`](../../crates/uor-r4-float-train) | [`crates/uor-r4-discrete`](../../crates/uor-r4-discrete/README.md) |
| What it carries | The replies: DEV text at every checkpoint ([float-training notebook](../research/notebook/float-training.md), row of 27 Oct "Exploration") | The M1 bar ([PLAN](../../PLAN.md), "The path to M1") |
| Table | 4,096 × 256 float values | 4,096 tokens × 4 blocks of 8 integers, each block one of 16 E8 roots ([the model, end to end](architecture.md), stage 2) |
| Export | `uor-r4.lut-stack/4`, through `fbs-export` | `uor-r4.lut-discrete/1` |
| Best DEV top-1 of 14,287 | **11,140** whole-table nearest, float, width 512 with 8 Hamming heads, cell objective, seed 2 ([claude-153](../loop/tasks/in-space-trunk-to-text/claude-153/REPORT.md)) | **455** writer, integer, Stage T `FrozenZero` ([m4-serving](../research/notebook/m4-serving.md) line 17) |

They share:
- the tokenizer and the data;
- the D11 runtime crate, [`crates/uor-r4-integer`](../../crates/uor-r4-integer).

They share no table, no trunk and no artifact format. This page follows the FBS trunk, because it is the one that writes replies. Where the discrete path differs, the stage says so.

## The chain at a glance

| # | Stage | Cost per served token (M1) | Cost in a training step (RTX 5090) | Reads → writes | Does the next stage read it in that domain? |
|---|---|---|---|---|---|
| 1 | Data | none | 2 bytes per token (proof) | text → token ids (u16) in windows of 192 | yes |
| 2 | Tokenizer | not measured | not measured | UTF-8 bytes → ids in [0, 4,096) | yes |
| 3 | Table | one row, inside the embedding's 83 ns | inside the forward | an id → a point in ℝ²⁵⁶ | yes, as trunk input. The readout compares against the same points |
| 4 | Trunk | **3.68 ms, 98.6%** | **forward 4.8 ms, 5%** (its backward is inside the backward's 62%) | one table point per position → a state in ℝ²⁵⁶ | yes, but the state carries about 6 tokens back |
| 5 | Look-back | none: the integer runtime has no read | **15.9 ms, 16%** (two Hamming reads, forward) | sign codes in {0,1}⁶⁴ per head → a moved readout point | yes for copies: the readout-move form landed (claude-146), and copies rose (claude-152) |
| 6 | Long memory | none: no stage reads it | none | an exact address → stored bytes | **no**: no stage forms the address, and no stage reads the bytes |
| 7 | Readout | writer **23 µs, 0.6%**; whole-table nearest **486 µs** | whole-table search **13.0 ms, 13%** | a query point in ℝ²⁵⁶ → a token id | yes. The decision is to serve nearest (claude-150), at 21 times the writer's cost |
| 8 | Decode loop | **36 µs outside the step, 1% of the plain token** | none | an id → the next id, greedy | ids only: the FBS artifact has no text path |
| 9 | Training | none | **96.5 ms a step, 31,718 context tok/s** at (a) | windows → float weights | yes |
| 10 | Integer export | once per model | none | a float snapshot → a `UORLUT01` integer artifact | discrete: exact. FBS: the export gap fails |
| 11 | Serving on M1 | **3.77 ms a token (the step 3.73), 265 tok/s** | none | the artifact and prompt ids → reply ids | — |

Serving shares are of a step on claude-126's width-256 export ([claude-158](../loop/tasks/fbs-cost-profile/claude-158/REPORT.md)); training shares are of claude-152's (a) step ([claude-157](../loop/tasks/fbs-cost-profile/claude-157/REPORT.md)). The two platforms differ in hardware, number type and what a step holds, so shares are compared only within each column.

## The stages

### 1. Data

| Field | Now |
|---|---|
| Cost per token | Training reads 2 bytes per token (proof, u16). Serving: none. |
| Mechanism | Memory dialogues `mw-balp`, generated by this project, cut into 192-token windows: 68,218 usable training windows ([float-training](../research/notebook/float-training.md), row of 27 Oct "Exploration"). It is pinned by sha256 at load (`PINNED_TRAIN_*`, claude-152). A 13-times-larger mixed corpus hurt: the model learned to repeat the user's numbers and lost the language ([claude-152](../loop/tasks/in-space-trunk-to-text/claude-152/REPORT.md)). `distill-d1`, a one-time donor corpus, is registered and not generated. It needs the owner's release at dispatch ([data and tokenizer](data-and-tokenizer.md) lines 56, 79). Merged. |
| Domain | Text in, token ids out. The ids fit in u16 (proof: 4,096 < 65,536). The trainer reads them directly. |
| Size | **Now:** TRAIN has 7,512,797 tokens (2,425,925 scored targets); DEV has 42,856 tokens (14,287 scored targets). Measurement, [data and tokenizer](data-and-tokenizer.md) line 50. **Goal:** Phi-3-mini, at 3.8B parameters, used 3.3T tokens for 69% on MMLU. Chinchilla's rule is about 20 tokens per parameter (both external). 3.3T is about 440,000 times the corpus now (proof). |
| Hardware | Generated offline; read by the GPU trainer; never on the M1. |
| Status | Corpus counts measured. No corpus beyond 7.5M tokens fits the test's dialogues. |
| Depends on | Upstream: the tokenizer, which fixes the ids. Downstream: training. |

### 2. Tokenizer

| Field | Now |
|---|---|
| Cost per token | Not measured; it runs outside `fbs-toks`, which takes ids. |
| Mechanism | Byte-level BPE, 4,096 pieces, identity `d36d3e87`, pinned for every corpus and artifact ([data and tokenizer](data-and-tokenizer.md) lines 14, 34, 85). A `train-bpe` command builds 259 to 65,536 pieces (line 36); no model has used it. Merged. |
| Domain | UTF-8 bytes to ids in [0, 4,096), and ids back to bytes. The table indexes by id: yes. |
| Size | **Now:** 4,096 pieces with no multi-digit piece, so 84 is the pieces 8 and 4 (line 38). **Goal:** GPT-4's encoding, `cl100k_base`, has 100,256 tokens (external), 24 times this. The readout's writer refuses more than 8,192 identities (stage 7). |
| Hardware | CPU for both training and serving; on the M1, a performance core. |
| Status | Built and pinned (measurement of identity). |
| Depends on | Upstream: the data. Downstream: the table, and the decode loop for text out. |

### 3. Table

| Field | Now |
|---|---|
| Cost per token | **Serving:** one row in, inside the embedding stage's 83 ns ([claude-158](../loop/tasks/fbs-cost-profile/claude-158/REPORT.md)). **Training:** the input row is inside the forward. The table as the objective's rival set is in stage 7. |
| Mechanism | One shared float table, `embedding.weight`, 4,096 × 256. It is the input embedding and also the readout's points ([float-training](../research/notebook/float-training.md), row of 27 Oct "Exploration": "one shared vector table for input and output"). Built by [`build_hamilton_model`](../../crates/uor-r4-float-train/src/hamilton_sheet.rs) (line 1935), with `SHEET_VOCAB` = 4,096 and `SHEET_WIDTH` = 256 (lines 61, 67). The discrete path uses 4 blocks of 8 integers per token, each block one of 16 E8 roots ([architecture](architecture.md), stage 2). Merged. |
| Domain | An id in; a point in ℝ²⁵⁶ out (discrete: ℤ³², four root blocks). The trunk reads it: yes. **The readout compares the trunk's output against these same points.** Their gradient is split: the output side gets 0.0030 against 1.24 for the input side under the clamp, so the decoding geometry gets 1/410 of the table's gradient. Measurement, [float-training](../research/notebook/float-training.md), row of 27 Oct "fold-against-clamp". |
| Size | **Now:** 1,048,576 values, 4 MiB as f32 (proof), 94% of the model's 1,112,576 learnable values ([claude-155](../loop/tasks/in-space-trunk-to-text/claude-155/REPORT.md), "The pivot"). **Goal:** at 100,256 tokens × width 1,024, the table alone would hold 102,662,144 values (proof; the shape is an illustration, not a design). |
| Hardware | Trained on the GPU. Served as part of the integer artifact on the M1 CPU. |
| Status | Built. The gradient split is measured. |
| Depends on | Upstream: the tokenizer. Downstream: the trunk, the look-back's codes, and the readout. |

### 4. Trunk

| Field | Now |
|---|---|
| Cost per token | **Serving, measured** ([claude-158](../loop/tasks/fbs-cost-profile/claude-158/REPORT.md)): 3.68 ms of a 3.73 ms step on the M1, 98.6%. That is 613 µs a layer: the recurrence block (three route gathers around the recurrence core) 352–364 µs, and the feed-forward block (three gathers, the gated product and the requantization) 253–259 µs. That is about 57 ns per learnable value, scalar integer code with no NEON. Which part of each block dominates is not split. **Training, measured** ([claude-157](../loop/tasks/fbs-cost-profile/claude-157/REPORT.md)): the forward is 4.8 ms of a 96.5 ms step at width 256 (5%), and 6.9 of 79.0 ms at width 512 (9%). Each recurrence is 0.55–0.84 ms of GPU time, and its host time is 0.99 of that: host-bound by the sheet's rule. The layer right after the stream's read is the exception, at 1.69 ms of host time. The trunk's backward is inside the backward (62% / 54%), not split. |
| Mechanism | The FBS sheet, all merged. [`hamilton_sheet.rs`](../../crates/uor-r4-float-train/src/hamilton_sheet.rs) lines 61–72 and 1935–1953, [`geometric_stack_native.rs`](../../crates/uor-r4-float-train/src/geometric_stack_native.rs) lines 101, 344, 479 and 687, [`spiralcore_fbs.rs`](../../crates/uor-r4-float-train/src/spiralcore_fbs.rs) lines 26–28 and 257: <ul><li>six recurrence layers (pattern `rrrrrr`);</li><li>width 256, as 64 quaternion lanes;</li><li>4 heads;</li><li>a 384-wide inner block;</li><li>a context of 192;</li><li>the Hamilton gate, with geometric recurrence gates and quaternion rotation;</li><li>a convolution of width 4;</li><li>six maps per layer, `rec.in`, `rec.gate`, `rec.out` and the three `mlp` maps (`shapes()`, line 687), each a SpiralCore/FBS projection of shape `[out/8, stages, 8]` in place of a dense matrix (`QUATERNION_MAPS`, line 479; line 344).</li></ul> On the discrete path, the reads Route, Flock and Relation each produce one integer vector of width 32 ([architecture](architecture.md), stage 4). |
| Domain | One table point per position in; a state in ℝ²⁵⁶ after the final norm out. The readout reads it as a query point in the table's space: yes. **What the state carries:** replacing the token k places back with BOS moves the opening point by 5.97 at k = 0, 0.37 at k = 6 and 0.04 at k = 12. 57–66% of lane positions in layers 1–5 sit at the shortest memory rate after training, against 33% at step 0. Both measurements: [float-training](../research/notebook/float-training.md), row of 27 Oct "Exploration". |
| Size | **Now:** 64,000 learnable values in the six layers and the final norm ([claude-155](../loop/tasks/in-space-trunk-to-text/claude-155/REPORT.md), "The pivot"). Each layer has six FBS maps holding 8,448 values, with 4 stages for inputs of 256 and 384, plus 2,176 norm, convolution, gate and decay values (proof by arithmetic on `shapes()`, line 687). With the table, the model holds 1,112,576 values, 94% of them the table. The state does not grow with context (proof: a recurrence). Width 512 is the best fit measured (Status, below). **Goal:** Phi-3-mini has 3.8B parameters (external), about 3,400 times this whole model. |
| Hardware | Training: GPU only. The model refuses to run unless on CUDA in F32 with SpiralCore/FBS, with "no host or dense fallback" (`geometric_stack_native.rs` line 938). Serving: the integer runtime on the M1 CPU, scalar code. Native NEON table kernels exist only for an older frozen stack ([desktop client](../products/desktop-client.md) line 49). The M1 GPU and Neural Engine are unused. |
| Status | **Fit is measured:** whole-table nearest top-1 is `9,328 / 10,573` at width 256 with the cell objective ([claude-152](../loop/tasks/in-space-trunk-to-text/claude-152/REPORT.md)), and `10,152 / 11,140` at width 512 ([claude-153](../loop/tasks/in-space-trunk-to-text/claude-153/REPORT.md)). Both are above the bigram's 6,300 and the M1 count table's 7,002. **Storage is measured short:** the trunk holds 64,000 values beside a 1,048,576-value table. Routing the prompt in works (stage 5), but the associations that pick a reply's first token have nowhere to be stored. Openings never beat claude-152's `302.0 / 228.0` by the sheet's margin (bar `347.3`), where a lookup of the prompt's first 8 tokens names `607` of 1,226 ([claude-155](../loop/tasks/in-space-trunk-to-text/claude-155/REPORT.md), "The pivot"). |
| Depends on | Upstream: the table. Downstream: the look-back and the readout. |

### 5. Look-back

| Field | Now |
|---|---|
| Cost per token | **Training, measured** ([claude-157](../loop/tasks/fbs-cost-profile/claude-157/REPORT.md)): the two Hamming reads, in the stream after layer 2 and on the readout, cost 7.9 ms each in the forward of a 96.5 ms step (16% together). They are GPU-bound (host time 0.86 of GPU time), and the same cost at both places. Their backward is inside the backward. Rates with the bag `28,560 / 21,430` and with the learned register `12,573 / 12,641` context tok/s, against (a)'s `32,049` with the Hamming read alone; these ran on different hosts, so the gap is a bound ([claude-155](../loop/tasks/in-space-trunk-to-text/claude-155/REPORT.md)). **Serving:** none, because the integer runtime has no read. **By arithmetic** (from the read's shapes): 4 heads × 64 bits is 32 bytes of code per position per read. With reads at two points, a token costs 8N popcounts and reads 64N bytes at context N. At N = 32,768 that is 2 MiB per token. A dense model's cache, at Llama-2-7B's shape (32 layers, width 4,096, 16-bit keys and values; external), is 512 KiB **per position**, or 16 GiB per token at the same N: 8,192 times as many bytes (proof). |
| Mechanism | **Merged (claude-146, #3332): the Hamming read.** <ul><li>each earlier position's key is the sign code of its token's table point after a learned per-lane rotation, 64 bits per head, with 4 heads (8 at width 512);</li><li>the match is popcount of XOR, a hard top 16, weighted by max(0, m − d)/m;</li><li>copy heads move the readout point n toward the copied token's point c: n + w(c − n).</li></ul> **Variants since:** <ul><li>type spin, through SpiralCore's quaternion group: negative, with seed 2 passing the openings bar only ([claude-151](../loop/tasks/in-space-trunk-to-text/claude-151/REPORT.md));</li><li>the opening read, the mean of the turn's first 8 points after layer 2: negative, and ignored, since its removal changes `25 / 19` tokens ([claude-154](../loop/tasks/in-space-trunk-to-text/claude-154/REPORT.md));</li><li>**the plain bag of the user's turn** (one slot, uniform over the span, by a doubling scan): it meets `in-space-trunk-to-text`'s done-when. Dropping it costs `1,514 / 1,648` tokens and `189 / 236` openings ([claude-155](../loop/tasks/in-space-trunk-to-text/claude-155/REPORT.md));</li><li>the learned prompt register: collapses training (`T` `684 / 1,246`), with untested causes: keys bent through the live stream, and slot starts that coincide in groups (claude-155).</li></ul> Earlier, negative against their bars: the state-keyed Flock read ([claude-144](../loop/tasks/in-space-trunk-to-text/claude-144/REPORT.md)) and the content read ([claude-145](../loop/tasks/in-space-trunk-to-text/claude-145/REPORT.md)). |
| Domain | Sign codes in {0,1}⁶⁴ per head; out, a moved readout point in ℝ²⁵⁶. **The readout reads it: yes.** With the cell objective, the read names `1,517 / 2,933` of 4,130 copied rows ([claude-152](../loop/tasks/in-space-trunk-to-text/claude-152/REPORT.md)). At width 512 the same trunk without the read copies about a quarter of them ([claude-153](../loop/tasks/in-space-trunk-to-text/claude-153/REPORT.md)). Points added inside the stream are ignored: the opening read (claude-154), and the two earlier copy forms, whose payload "reads to the trunk as its own input" and "barely moves where the prediction lands" ([claude-146](../loop/tasks/in-space-trunk-to-text/claude-146/REPORT.md) lines 29–30). |
| Size | **Now:** a context of 192 (`SHEET_CONTEXT`, `hamilton_sheet.rs` line 66). The serving session cannot slide its window (`fbs-toks.rs` line 9). The discrete path's window holds at most 243 = 3⁵ positions ([architecture](architecture.md), stage 3). **Goal:** GPT-4 shipped with 8,192 and 32,768 tokens of context (external), 43 to 171 times this. |
| Hardware | Training: GPU. Serving: none. The integer runtime has no Hamming read and no bag. |
| Status | Copies measured positive; openings measured flat. The bag carries the turn into the trunk, and the trunk cannot store what to do with it (stage 4). |
| Depends on | Upstream: the table (its codes come from table points) and the trunk. Downstream: the readout, which it moves. |

### 6. Long memory

| Field | Now |
|---|---|
| Cost per token | Per read: 384 bytes and 8 fingerprint checks for a small range (measurement, [object store](../mechanisms/object-store.md) line 46). How many reads a token needs is undefined while no stage issues them. **By arithmetic:** a lookup keyed by the turn's first `k` ids is a hash and a load, far below the trunk's 3.68 ms. This is the form `route-plus-memory` tests. |
| Mechanism | The object store, merged ([object store](../mechanisms/object-store.md)). A record is a scale byte, a 12-byte fingerprint and a 32-byte leaf (line 23). A range read authenticates only the branches it touches (line 31). The E8 orbit memory is PARKED, at about 59 MB per layer ([e8-orbit-memory](../mechanisms/e8-orbit-memory.md) line 34). The archived prime routes (gcd admission over n-let keys) are in [the archived router](#the-archived-prime-and-angular-router). |
| Domain | An exact address in; stored bytes out. **No stage reads its output, and no stage forms its input.** <ul><li>With the true address supplied by the oracle, the answer was right 80 of 80 times, against 20 of 80 for the deterministic sieve and 0 of 80 with no memory ([PLAN](../../PLAN.md), the exact-memory premise).</li><li>Free-running with an exact hint recovered 0 of 40 ([exact memory](../mechanisms/exact-memory.md) line 11).</li><li>**The evidence for keying it by the turn:** TRAIN's commonest opening per first-8-token key names `607` of DEV's 1,226 openings. TRAIN has `212,968` openings over `26,450` distinct prompt starts ([claude-154](../loop/tasks/in-space-trunk-to-text/claude-154/REPORT.md), [claude-155](../loop/tasks/in-space-trunk-to-text/claude-155/REPORT.md)). claude-154's turn anchor already forms that key exactly at every opening.</li></ul> |
| Size | **Now:** a 23-byte range of a 32,768-byte object reads 8 nodes, 384 bytes (measurement, [object store](../mechanisms/object-store.md) line 46). A record stores 32 bytes in 45, about 1.4 times the payload (proof). **Goal:** hold facts in place of parameters, at whatever size a laptop disk allows (hypothesis). |
| Hardware | CPU, memory and disk, on both the GPU host and the M1. No GPU work. |
| Status | The store is measured. Its connection to prediction is the next decisive run (`route-plus-memory`). |
| Depends on | Upstream: an address, which the turn anchor can form (claude-154). Downstream: the readout. The look-back's lesson applies: a stored token must land as its table point where the readout reads. |

### 7. Readout

| Field | Now |
|---|---|
| Cost per token | **Serving, measured on the M1** ([claude-158](../loop/tasks/fbs-cost-profile/claude-158/REPORT.md)): the writer 23 µs (0.6% of a step); a whole-table nearest 486 µs, 21 times the writer and 13% of a step. **Training, measured** ([claude-157](../loop/tasks/fbs-cost-profile/claude-157/REPORT.md)): the cell objective's whole-table search is 13.0 ms of a 96.5 ms step at width 256, and 26.6 of 79.0 ms at width 512. It is GPU-bound (host time 0.04–0.06 of GPU time), and it doubles with the width. Its gradient is inside the backward. **By arithmetic:** whole-table nearest is 4,096 × 256 = 1,048,576 absolute differences; the writer is 8 cells × 16 tokens × 256 = 32,768, plus ranking the cells. At 100,256 tokens and width 1,024, whole-table nearest would be 102,662,144 per token, and a sublinear index would be needed (hypothesis). |
| Mechanism | Two decoders, both merged, both scoring by negative mean L1 distance to table points: <ul><li>**The writer** ([`native_vector_writer.rs`](../../crates/uor-r4-float-train/src/native_vector_writer.rs) lines 13–39). It has 256 cells, keeps 1 to 16 of them (8 here), and picks the nearest token inside them. It refuses more than 8,192 identities, a width above 1,024, or more than 16 kept cells. `fbs-toks` decodes with it.</li><li>**Whole-table nearest** (`nearest_scores`, [`in_space_long.rs`](../../crates/uor-r4-float-train/src/in_space_long.rs) line 204). The float runs evaluate with it, and the cell objective trains against it. Its integer form now exists beside the served step (`stack_whole_table_nearest`, claude-158).</li></ul> The discrete path uses an integer writer: it opens K = 8 of 256 cells, and admission caps every stage above it ([architecture](architecture.md), stage 5). |
| Domain | A query in ℝ²⁵⁶ in, a token id out. The decode loop reads it: yes. **One decoder** ([claude-150](../loop/tasks/one-decoder/claude-150/REPORT.md)). The `voronoi` objective, whose margin can be met, made the writer hold at seed 1: `8,430` writer against `6,762` nearest at step 32,768, peaking at `8,581`. At seed 2 it never learned: `493` against `6,909`, with the target's cell admitted on only `461`–`714` of 14,287 rows. A decoder that works at one seed of two cannot be the one served, so **the decision is to serve whole-table nearest**, the decoder the cell objective trains for. Before that, the two decoders disagreed on one checkpoint at 345 against 7,813 ([float-training](../research/notebook/float-training.md), row of 27 Oct "Exploration"). On the discrete path, a table keyed by the read code predicts 3,414 rows against the writer's 149 ([m1-discrete-attention](../research/notebook/m1-discrete-attention.md) line 97). |
| Size | **Now:** 4,096 candidates. **Goal:** about 100,000 (stage 2), beyond the writer's limit of 8,192. |
| Hardware | Training: GPU. Serving: M1 CPU, scalar. |
| Status | Decided: nearest. That decision costs 21 times the writer on the M1 today, and grows with the vocabulary. **What might make the cheap readout correct (hypothesis):** the writer's admission, so the target's cell is kept at every seed as it is at seed 1. Why admission failed at seed 2 is not known (claude-150). |
| Depends on | Upstream: the table, the trunk and the look-back. Downstream: the decode loop. |

### 8. Decode loop

| Field | Now |
|---|---|
| Cost per token | **Measured on the M1** ([claude-158](../loop/tasks/fbs-cost-profile/claude-158/REPORT.md)): 36 µs outside the step, 1% of the plain per-token time. That holds the loop and `step_native`'s copies of the kept cells and the query. |
| Mechanism | [`fbs-toks`](../../crates/uor-r4-integer/src/bin/fbs-toks.rs) (lines 9–11): a 128-token prefill, then 64 greedy decode steps, each feeding back "the token the writer chose". The step is `stack_step_fbs` ([`stack/session.rs`](../../crates/uor-r4-integer/src/stack/session.rs) line 2953). Its stage-timed twin, `measure_fbs_step_stages`, is exact against `step_native` (claude-158). A text loop, with the tokenizer and a turn-end token, exists only for the older bundle ([`generation.rs`](../../crates/uor-r4-integer/src/generation.rs), lines 196–202). `uor-chat --stack` takes raw `--prompt-ids` ([`uor-chat.rs`](../../crates/uor-r4-integer/src/bin/uor-chat.rs) line 48). Merged. |
| Domain | Id in, next id out. **Text out is not wired for the FBS artifact.** Sampling is greedy. No lawful sampling rule exists, since softmax is banned (CHARTER §3.2). |
| Size | **Now:** a reply fits in what is left of the 192-token window; the session cannot slide it (`fbs-toks.rs` line 9). **Goal:** prompt and reply inside a context of 8,192 to 32,768 tokens, as GPT-4 shipped with (external, stage 5). |
| Hardware | Training: none; this is a serving stage. Serving: M1 CPU. |
| Status | Ids are measured. Replies are fluent but templated: they do not follow the question, and only rarely bring back the user's number ("Hi is 5246.", [claude-152](../loop/tasks/in-space-trunk-to-text/claude-152/REPORT.md)). |
| Depends on | Upstream: the readout and the tokenizer. Downstream: the user. |

### 9. Training

| Field | Now |
|---|---|
| Cost per token | **Measured, one training step by stage** ([claude-157](../loop/tasks/fbs-cost-profile/claude-157/REPORT.md)). **(a), width 256 with 4 Hamming heads:** 96.5 ms a step, 31,718 context tok/s. The backward is 62%, the whole-table search 13%, the reads 16%, the trunk's forward 5% and the optimizer 3%. **Width 512 with no read:** 79.0 ms, 38,809 tok/s. The backward is 54%, the search 34% and the trunk's forward 9%. The backward's host time is 0.98–0.99 of its GPU time: it waits on launches or on a sync inside it, not yet told apart. **Single-run rates measured since the first map:** `41,018–41,295` alone (claude-150, no read), `32,049` at (a) (claude-152), `38,578` at width 512 with no read and `17,033` with 8 heads (claude-153). |
| Mechanism | All merged. A float, offline trainer, allowed by D37: [`in_space_long.rs`](../../crates/uor-r4-float-train/src/in_space_long.rs) and `train-in-space`. The objective is a hinge, not a likelihood: <ul><li>`hinge` at margin 1 (line 181);</li><li>`nearest_loss` (line 260): the nearest objective averages the hinges against the whole table's `SHEET_RIVALS` = 8 nearest wrong tokens, and **the cell objective** keeps only the nearest one's (claude-152);</li><li>`voronoi_loss` (line 220);</li><li>the writer's `MarginHinge` ([`hamilton_sheet.rs`](../../crates/uor-r4-float-train/src/hamilton_sheet.rs) line 70).</li></ul> `train-in-space --profile K` times each stage with CUDA events (claude-157). The discrete path learns by trying all 16 legal states of one table entry on the GPU against an integer rank-margin hinge ([architecture](architecture.md), stages 6–7). |
| Domain | Windows in; float weights out. The exporter reads them: yes. |
| Size | **Now:** up to 32,768 steps × 16 windows, 7.7 epochs ([float-training](../research/notebook/float-training.md), row of 27 Oct "Exploration"). **Goal:** Llama 3.1 405B trained on 15.6T tokens with 3.8 × 10²⁵ FLOPs (external). The aim is GPT-4 ability for far less. |
| Hardware | One RTX 5090 per pod on Vast, at $0.56/h ([claude-140](../loop/tasks/d11-exporter-float-trunk/claude-140/REPORT.md) line 27). The project's cap is 4 pods and $8/h ([AGENTS.md](../../AGENTS.md), "Compute"). |
| Status | Measured. **Nondeterministic:** five runs of one recipe gave seed-1 top-1 values of 745, 1,472, 1,780, 2,914 and 3,845 ([claude-140](../loop/tasks/d11-exporter-float-trunk/claude-140/REPORT.md) line 9). The suspects are float `atomicAdd` sums in the FBS backward (line 7). |
| Depends on | Upstream: the data and every model stage. Downstream: the export. |

### 10. Integer export

| Field | Now |
|---|---|
| Cost per token | Once per model, not per token. |
| Mechanism | Both merged: <ul><li>**FBS:** [`fbs_export.rs`](../../crates/uor-r4-float-train/src/fbs_export.rs) snapshots the float model. [`fbs-export`](../../crates/uor-r4-integer/src/bin/fbs-export.rs) writes `uor-r4.lut-stack/4` `UORLUT01` (lines 1–2).</li><li>**Discrete:** `uor-r4.lut-discrete/1`, 542,260 bytes per checkpoint ([m4-serving](../research/notebook/m4-serving.md) line 17).</li></ul> |
| Domain | Float weights in, integer codes out. The runtime reads the artifact: yes. **Whether the artifact computes what was trained:** <ul><li>**Discrete:** yes. The gap is exactly 0 on all four Stage T checkpoints, 14,287 of 14,287 rows (line 17).</li><li>**FBS:** no. The D11 forward's own gap from float lies outside its tolerance on all six receipts, by 17 to 287 rows; the export itself moves each by at most 8 rows (line 14). The 25 Oct sheet's Build A gap was 1,914 rows, a FAIL ([float-training](../research/notebook/float-training.md), row of 25 Oct).</li></ul> |
| Size | **Now:** discrete, 542,260 bytes. FBS: claude-126's width-256 export is about 2 MB (the store's listing). **Goal:** an artifact that fits beside its store in a 16 GiB laptop. For scale, 7B values at 4 bits would take 3.5 GB (proof by arithmetic; an illustration, not a design). |
| Hardware | Training: none; export runs once, on the host CPU, after training. A GPU evaluator of the artifact is qualified bit-exact ([m4-serving](../research/notebook/m4-serving.md) line 23). Serving: the artifact is read by the M1 CPU (stage 11). |
| Status | Measured: exact on the discrete path, failing on the FBS path. |
| Depends on | Upstream: training. Comparing two training runs needs a deterministic backward or repeats with a stated spread (stage 9). Downstream: serving. |

### 11. Serving on M1

| Field | Now |
|---|---|
| Cost per token | **Measured, one served token by stage** ([claude-158](../loop/tasks/fbs-cost-profile/claude-158/REPORT.md); claude-126's width-256 FBS export, AC power). 3.73 ms a step and 3.77 ms a decoded token (265 tok/s decode): <ul><li>the six layers 3.68 ms (98.6%);</li><li>the writer 23 µs (0.6%);</li><li>the final norm 6 µs;</li><li>the embedding and bookkeeping under 0.1 µs.</li></ul> 36 µs a token falls outside the step. **Other measured rates:** discrete `FrozenZero` decodes at 12,630 and 12,523 tok/s, at top-1 455 of 14,287, and `Relation` at 1,197 and 1,118 ([m4-serving](../research/notebook/m4-serving.md) line 17). The older 29M stack ran 68 to 80 tok/s on 8 threads, teacher-forced ([serving and export](serving-and-export.md) line 82). **Energy:** table-emulated products cost 0.00413 J/token against 0.00096 for float, about 4.3 times as much, on a dense model (line 83). |
| Mechanism | The D11 engine: integer only, with no multiply instruction. Each product reads a 16-entry table built by repeated addition ([serving and export](serving-and-export.md) line 35). The native build's audit finds 44 symbol ranges free of float, multiply and divide ([m4-serving](../research/notebook/m4-serving.md) line 17). The rule is D11 ([DECISIONS](../integration/DECISIONS.md) line 72, CHARTER §3.6). Merged. |
| Domain | The artifact and prompt ids in; reply ids out. |
| Size | **Now:** the discrete artifact is 542,260 bytes. **Goal:** fit a capable model in a 16 GiB laptop alongside its store. |
| Hardware | Training: none; this is a serving stage. Serving: M1 performance cores only, scalar code. Not used: <ul><li>**NEON:** used only by an older frozen stack ([desktop client](../products/desktop-client.md) line 49);</li><li>**GPU:** has no lawful kernel yet. Popcount and L1 nearest need no multiply, so a lawful kernel is possible (hypothesis);</li><li>**Neural Engine:** Core ML exposes it as a tensor engine for convolution and matrix products, so no lawful use under §3.3 is known (hypothesis).</li></ul> |
| Status | Profiled for the FBS step. The cost is the trunk's scalar kernels. |
| Depends on | Upstream: the export, the decode loop and the text path (missing for FBS). |

## The archived prime and angular router

The owner, 28 Oct: "it is worth looking into the archived hopf golden ratio complex - riemannian bridge we used for the original prime router or angular router". This section reads it from the files the archive holds.

**What it was, in three generations:**
1. **The ancestor**, a Python "R4 Prime Router" ([`server.py`](../../projects/research-archives/code/gemini-dev/prime-router/server.py), [README](../../projects/research-archives/code/gemini-dev/prime-router/README.md); [evidence note](../../archive/prime_router_geometric_context_evidence.md)).
   - Word vectors (GloVe blended with geometric coordinates) lie on a hypersphere, and a query is the sum of its words.
   - A brain state evolves as H ← normalize(γ·H + (1−γ)·S_query) (README).
   - Retrieval scores shared prime factors ×100, then cosine, then a window slice norm.
   - The Hopf coordinates (χ, δ, α) and the top sentence went into the system prompt of a local Ollama model (`gemma4:e2b`).
   - The "Riemannian Hopf canvas" is its 3-D stereographic view of the zeta rings.
   - Its next word is a greedy argmax over `VOCAB_MATRIX @ S`, which is a dense product. Above temperature 0, one path samples from a normalized exponential, which is banned.
2. **ADR-0003**, fixed-zeta prime routes ([ADR](../reference/adr/0003-fixed-zeta-prime-route-attention.md); the pointer at `archive/adr/0003-…` names a moved path).
   - **Zeta phases:** θ_j(p) = wrap(γ_j·log p) per prime atom, with a compiled phase-delta table at runtime (§1).
   - **Semiprime transition experts:** e_t = p_(t−1)·p_t, with gcd(e_t, e_(t+1)) = p_t (§2).
   - **An n-let context key:** the product of the last k primes.
   - **An S³ compute state** with its Hopf observation h ∈ S², the fibre phase kept as torsion (§3).
   - **The choice rule:** least integer energy over a few index rows.
   - **§5, the direction-preserving golden radial bridge.** This is the "golden ratio complex–Riemannian bridge" the owner remembers. With E : ℂ² → ℝ⁴ the ordinary identification, R_φ(z) = φ·E(z), and φ = (1 + √5)/2. It preserves direction exactly: angle, prime direction and Hopf sector are unchanged, and only the radial shell moves. In ℤ[φ], a shell step is the integer Fibonacci step (a, b)·φ → (b, a + b): one add, or one subtraction for φ⁻¹. The map is from ℂ², not ℂ⁴.
3. **The prime-route pointer** (1–2 Oct, `70d8a223c` #1587, `f9c926925` #1589).
   - Each token has a registered prime, and a position's key is the product of its last W tokens' distinct primes.
   - A source is admitted when it shares a factor with the query (gcd > 1).
   - Admitted sources are scored by ln gcd plus recency, then by a normalized exponential, which is now banned. The ranked variant adds a learned score.

**What was measured** (each a measurement, source named):
- **The ancestor:** angular routing and decodability only, never generation, and not reproduced under today's gates. Verdict: inconclusive ([hopf-observation](../mechanisms/hopf-observation.md), "What has been tried").
- **Hopf sector routing:**
  - #303: 7 of 512 sectors occupied;
  - #306, after a fixed signed projection: 43 of 512;
  - #430: as a retrieval filter, MRR 0.0045 against 0.0743, no retrieval value.

  **Sector routing is PARKED; the kept-fibre map is READY** (D72, [hopf-observation](../mechanisms/hopf-observation.md)).
- **ADR-0003 as a whole:** `RETAIN_STORAGE_RECALL_ONLY` ([qualification](../../archive/prime_route_attention_qualification_958.md)). The product probe never ran: there was no prompt-to-address compiler, no address-to-token decoder and no generation loop. The learned four-coordinate mixer before it "selected support but did not transport adequate semantic value": a 4.44% real-against-permuted held-out advantage against a 5% gate ([#951](../../archive/geometric_mixer_qualification_951.md), recorded on #958).
- **The prime-ranked pointer** (10 Oct, [pointer-key](../mechanisms/pointer-key.md)): the hit rate rose from 0.377 to 0.545. Memory answers stayed at 19, 21 and 23 of 40, against a bar of 25. The intended key, entity prime × attribute prime, was never built.
- **The choice rule:** on PR #455's context-class test (not the router), among the latent arms hard selection got 22.81% top-1, top-3 24.54% and averaging over all 25.91%. Hard selection did not win. The test's own fixed baseline, at 26.42%, was above all three.
- **The golden bridge (§5):** built and tested as algebra (the ℤ[φ] steps are `IMPLEMENTED_SUBSTRATE` and pass their source-free fixtures, `PASS_SOURCE_FREE`, ADR-0003 and the qualification record). It has no language result, and no rule ties a zeta phase or n-let to a shell ("bound only as structural transport", ADR-0003).
- **The exact-memory sieve:** 20 of 80 against 80 of 80 with the true address, and 0 of 80 with no memory (stage 6).

**Cost per token today, by arithmetic:**

| Piece | Operations per token | Integer and multiplier-free? |
|---|---|---|
| Zeta phase step θ_j | 2J table reads and adds (J up to 512) | yes, with a compiled table |
| Semiprime or n-let key update | 1 integer multiply and 1 divide, or a hash of the last k ids | as a product, no (a multiply); as a hash of ids, yes |
| gcd admission over n-let keys | 1 binary gcd per candidate: up to about 244 shift-and-subtract steps for an 8-prime key of about 122 bits | yes (subtractions and shifts) |
| Hopf observation h(z1, z2) | 8 products and about 5 adds per 4-vector | no (products) |
| Golden radial shell step in ℤ[φ] | 1 integer add (1 subtraction for φ⁻¹) | yes |
| Ancestor cosine retrieval | grows with the items stored; float | no |

**Which piece fits which stage, and in which domain** (all hypotheses unless marked):
- **Long memory (stage 6): the prime and n-let key is the closest fit.** A key from the turn's first k ids, with exact admission, is the form `route-plus-memory` runs: a table built from TRAIN, read at the readout. The archive's measured lesson is that admission alone raised the hit rate but not the answers (the prime-ranked pointer). Why is a hypothesis: the pointer page's own reading is that the found token did not reach the prediction. The plan puts the stored token where the readout reads, which tests it. The slot-address key (entity prime × attribute prime) remains untested.
- **Look-back (stage 5): the choice rule.** #455's ordering (averaging over all, then top-3, then hard selection) matches the current evidence. The uniform bag is the read that works (claude-155), and the learned hard-picking register collapsed.
- **Table and trunk (stages 3 and 4): zeta phases and the golden shell.**
  - θ_j(p) is a fixed angular code per token, computed once per vocabulary. It could seed the table's directions or the read's keys at no per-token cost.
  - The golden bridge changes only radius. So it carries a level or scale, not a route: for example, a memory tier or a confidence shell, at one integer add.
  - Neither has been tested in this trunk.
- **Readout (stage 7): nothing archived serves it.** That was ADR-0003's own missing piece, an address-to-token decoder (recorded as `NOT_RUN` in the qualification record).
- **The Hopf oracle:** as a sector router it is measured negative (#430). The kept fibre is ready as extra integer state, untested for language.

**Against the owner's memory:**
- **"Golden ratio … complex – Riemannian bridge":** yes, ADR-0003 §5. It is from ℂ², not ℂ⁴, and it is a radial similarity. It was built and tested as algebra, with no language result.
- **"Angular router":** yes, the zeta phases and the ancestor's angular routing.
- **"Hopf as a quick oracle":** yes as designed. Measured as a router, it is negative.
- **"Hard coded actually did win":** read here as hard selection, not on the record: on #455, among the latent arms, averaging won. Read as a fixed rule instead of a learned one, #455's fixed baseline (26.42%) did beat the learned arms.

## The scale arithmetic for the goal

GPT-4's own size and training data are not published (GPT-4 Technical Report, external). The comparisons below therefore use published open models of known size as reference points.

| Quantity | Now | Reference (external) | Ratio (proof) |
|---|---|---|---|
| Training tokens | 7,512,797 (stage 1) | Phi-3-mini: 3.3T. Llama 3.1 405B: 15.6T | about 440,000× to Phi-3-mini |
| Trainable values | 1,112,576 (stage 4) | Phi-3-mini: 3.8B | about 3,400× |
| Context | 192 (stage 5) | GPT-4: 8,192 and 32,768 | 43× and 171× |
| Vocabulary | 4,096 (stage 2) | `cl100k_base`: 100,256 | 24× |
| Training throughput | 31,718–41,295 context tok/s per run, one run per RTX 5090 (stage 9) | — | 1T tokens in about 70 to 91 days on 4 GPUs, assuming perfect scaling (below) |
| Decode on this laptop | 265 tok/s at 1.1M values, 98.6% in the trunk (stage 11) | A dense 7B model at 4 bits: weights of 3.5 GB against about 68 GB/s of M1 memory bandwidth, a ceiling near 19.5 tok/s | see below |

**Training time** (proof by arithmetic from the measured single-run rates, valid only for this 1.1M-value model):
- At 4 GPUs, one run each, the rate would be 127,000 to 165,000 tok/s. That assumes one model trains across them with perfect scaling, a hypothesis: no multi-GPU run of one model is recorded.
- At that rate, 1T tokens take about 70 to 91 days and $3,800 to $4,900 at $0.56/h a GPU. 15.6T take about 3.0 to 3.9 years.
- Cutting the step's own costs (the backward and the search, stage 9) shortens all of these.

**Serving speed per value read** (measurement and proof):
- A token reads about 64,000 trunk values and about 33,000 writer values (stages 4 and 7) in 3.73 ms, about 2.6 × 10⁷ values a second.
- A dense 4-bit model limited only by bandwidth reads about 1.4 × 10¹¹ values per second on this chip (68.3 GB/s at half a byte each).
- So per value read, the integer runtime is about 5,000 times slower than that bandwidth limit.

**The profile now says where the time goes:** 98.6% in the six layers' scalar kernels, at about 57 ns per learnable value (claude-158). The bandwidth is not what binds.

**Where the geometry claims to change the arithmetic**, and what is measured:

| Claim | What it would change | Measured | Hoped for |
|---|---|---|---|
| Facts live in exact memory, not in parameters | Fewer parameters for the same knowledge. Language models store about 2 bits of knowledge per parameter (Allen-Zhu and Li, external). | With the true address supplied, 80/80, against 0/80 with no memory. Free-running, 0/40. A lookup by the turn's first 8 tokens names 607 of 1,226 openings, against the trunk's best of about 300 (stage 6). | That a turn-keyed memory read at the readout carries the openings the trunk cannot store (`route-plus-memory`). |
| Long context by popcount | Look-back reads 64 bytes per position, not 512 KiB: 8,192 times fewer (proof, stage 5). | The Hamming read copies (`1,517 / 2,933` of 4,130), at 16% of a training step. The integer runtime has no read yet (stage 5). | That an integer read serves it at the arithmetic's cost. |
| A recurrence with constant state | Memory per token does not grow with context (proof, stage 4). | The state carries about 6 tokens back (stage 4). | That retention or a look-back extends it. |
| Integer serving with no multiplier | Runs on any CPU, with an exact artifact. | 265 tok/s for 1.1M values, 98.6% in scalar trunk kernels; the discrete export is exact. Energy is 4.3 times worse on an older dense model (stage 11). | That vector kernels, exact against the scalar step, serve a capable model at a useful speed. |
| Training below datacenter scale | 1T tokens on 4 GPUs in about 70 to 91 days, about $3,800 to $4,900, if one model scales across them (above). | Only for 1.1M values. Nondeterministic (stage 9). | That ability arrives at a small enough size for this budget. No scaling slope has been measured beyond width 512's gain (stage 4). |

## The gaps, ranked by what blocks a cheap, capable model first

1. **Storage: the trunk cannot hold the associations that pick a reply.** Routing the turn in works: dropping the bag costs `189 / 236` openings. But openings never beat `302 / 228` by the sheet's margin (bar `347.3`), while a lookup of the turn's first 8 tokens names 607. The trunk has 64,000 values beside a 1,048,576-value table (stages 4, 5 and 6). Exact memory is measured to work with the true address and is connected to nothing. **This blocks capability first, and its fix is cheap per token** (stage 6, by arithmetic).
2. **The served decoder: correct and expensive, or cheap and unreliable.**
   - Whole-table nearest is the decision (claude-150). It costs 21 times the writer on the M1 (13% of a token) and 13–34% of a training step, and it grows with the vocabulary (stage 7).
   - The writer costs 0.6% of a token, but its admission failed at one seed of two.
   - Making the writer's admission hold is what makes the cheap readout the correct one.
3. **Serving speed: the trunk's scalar kernels are 98.6% of a token** (stage 11), at about 57 ns per value. Vector (NEON) kernels, checked bit-exact against `step_native` with claude-158's twin, are the change. First, split each layer to see whether the gathers or the core dominate.
4. **Training cost: the backward is 54–62% of a step and host-bound, and the whole-table search is 13–34%** (stage 9). Splitting the backward with the driver's profiler names the next change. A rival search over a subset, such as the writer's kept cells, removes most of the search's cost; its effect on prediction must be run.
5. **The FBS artifact does not compute what was trained, and training does not repeat.** The FBS export gap fails on all six receipts, and five identical runs spread from 745 to 3,845 (stages 9 and 10). The discrete export is exact, but that model scores 455.
6. **The serving session cannot slide its 192-token window, and the integer runtime has no look-back** (stages 5 and 8). Long context, the geometry's main claim, cannot be served yet.
7. **The FBS artifact has no text path.** It takes ids and gives ids (stage 8).
8. **The two model paths do not join.** The M1 bar rests on the discrete path, the replies on the FBS trunk, and they use different tables, trunks and formats (Two model paths).
9. **Scale caps, with one data point.** Width 512 fits best (`10,152 / 11,140`), at about half the training speed with 8 heads. The writer allows at most 8,192 identities and width 1,024. The corpus is 7.5M tokens, and no run has measured how ability grows beyond that (stages 1, 4 and 7).
10. **Greedy decoding only, and the cost of multiplier-free serving.** Neither has a decision: a lawful sampling rule does not exist, and the 4.3-fold energy cost was measured on an older model (stages 8 and 11).

## Recommended order of work

Each unit is one loop session. "Pass" and "fail" say what each outcome unlocks. The first map's U1 (the Hamming read, claude-146), U2 (one decoder, claude-150) and U9's profile (claude-157, claude-158) are done.

| Unit | What it does | A pass unlocks | A fail unlocks |
|---|---|---|---|
| **R1 (first, item `route-plus-memory`)** | **Route the turn to an exact memory.** The turn anchor's first k ids key a table built from TRAIN, and the stored token's point moves the readout. Barred against (a), a no-memory control (L-010) and claude-155's bag, at two seeds. | The first evidence that storage, not the trunk, carries what the trunk cannot. The memory line then grows: the slot-address key, M3's writes. | That key closes. Try the bag's learned code as the key, the item's second arm. |
| R2 | **The writer's admission** (the `one-decoder` line): make the target's cell be kept at every seed, then read the writer against nearest on the same checkpoints. | The served readout costs 0.6% of a token instead of 13%, and scales past 4,096 tokens without a full search. | Serve nearest, and build a sublinear index before a larger vocabulary. |
| R3 | **Split each served layer, then NEON kernels** exact against `step_native`, timed with `fbs-toks --stages`. | A per-value serving rate near the hardware's, which can be extrapolated. | The kernels' layout is redesigned before a larger model is served. |
| R4 | **Split the backward** (driver's profiler), and run **a cheaper rival search** against (a). | Training's cost per token falls, so every later run costs less. | Keep the search, and report its share. |
| R5 | An integer Hamming read and bag in `uor-r4-integer`, and a sliding session with a ring of codes longer than 192. | The look-back and long context are served on the laptop. | The look-back stays float-only; find the part that does not export. |
| R6 | A deterministic FBS backward (a fixed-order reduction in place of float `atomicAdd`), then the export gap re-measured with repeats. | Runs can be compared, and the export gap gets a verdict. | Report every comparison with a stated spread from repeats. |
| R7 | A text path for the FBS artifact: the tokenizer, the dialogue protocol and the turn-end token, in `uor-chat --stack`. | A typed question gets a typed reply on this laptop. | The protocol bundle needs porting first. |
| R8 | A scale probe: width 1,024 (the writer's limit), and a corpus beyond 7.5M tokens that fits the test's dialogues. `distill-d1` needs the owner's release. | A measured slope of ability against size: the first number for the goal arithmetic. | The architecture's scaling becomes the open question before anything larger is trained. |

R1 comes first because it is the one step aimed at the storage gap, and its per-token cost is far below the trunk's. R2 and R3 make the served model cheap: the readout and the kernels. R4 makes every later run cheaper. R2 to R4 can run beside R1, since they touch other files.

## Proposal for the owner's queue

This is only a proposal. No §3 constraint is relaxed here, and this page waits on no answer.

The multiplier-free rule (D11, CHARTER §3.6) has one measured cost: 4.3 times the energy, on an older dense model (stage 11). claude-158's profile puts 98.6% of a served FBS token in the trunk's scalar kernels. It does not split the share spent in table-emulated products inside them. If the owner wants that share as a number, R3's per-layer split can report it.

## Sources outside the project

- Apple, M1 announcement (November 2020): 8-core GPU at 2.6 TFLOPS; 16-core Neural Engine at 11 trillion operations per second. LPDDR4X-4266 on a 128-bit bus gives 4,266 MT/s × 16 bytes ≈ 68.3 GB/s.
- OpenAI, *GPT-4 Technical Report* (2023): no parameter count or training data size is disclosed. The March 2023 announcement offered 8,192 and 32,768 tokens of context.
- OpenAI, `tiktoken`: the `cl100k_base` encoding used by GPT-4 has 100,256 tokens.
- Touvron et al., *Llama 2: Open Foundation and Fine-Tuned Chat Models* (2023): the 7B model has 32 layers and width 4,096, with one key and one value per head.
- Llama Team, *The Llama 3 Herd of Models* (2024): the 405B model trained on 15.6T tokens with 3.8 × 10²⁵ FLOPs.
- Abdin et al., *Phi-3 Technical Report* (2024): Phi-3-mini has 3.8B parameters, trained on 3.3T tokens, and scores 69% on MMLU.
- Hoffmann et al., *Training Compute-Optimal Large Language Models* (2022): about 20 training tokens per parameter.
- Allen-Zhu and Li, *Physics of Language Models: Part 3.3, Knowledge Capacity Scaling Laws* (2024): about 2 bits of knowledge per parameter.
