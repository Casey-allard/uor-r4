# Geometric stack (layered quaternion-recurrence language model)

**Status: PARKED.** The version that was trained leaned on arithmetic the project later banned (softmax reads, sigmoid and SiLU gates, dense matrix products), and it never met the first milestone's bars. Its quaternion "spinner" idea was kept. A rules-compliant rebuild exists only as a small, untrained extract. See [what would reopen it](#what-would-reopen-it).

Labels used below: **proof** means derived from definitions or code, **measurement** means a recorded run, **hypothesis** means an untested idea. See [`docs/formal_vocabulary.md`](../formal_vocabulary.md).

## In plain words

This was the project's main trained language model from late September to mid-October 2026, between 7 and 214 million parameters. It reads text through a stack of layers. Most layers carry a running memory made of four numbers, a "spinner" called a quaternion. At every word the spinner is turned a little and the old memory fades. A few layers look back over earlier words, and every layer ends in a small feed-forward block. It was trained on GPUs, then rounded to whole numbers so a laptop could run it. It produced chat-style replies, though only 43 of 232 reply checks passed and it was served through the integer engine on pod CPUs. It used arithmetic the project later banned, and it never reached its quality bars.

## What it replaces in a transformer

The whole network. The recurrence layers do the job that attention does in a transformer, carrying context forward. The read layers and feed-forward blocks, though, are the transformer's own parts. The project's own audit in October found that the served model was a transformer-shaped dense stack with a quaternion-mixed residual. The geometry was a thin layer inside it (see the figure).

![Three panels. Left: one bar for the 30.3 million multiply-adds per token, split by part, with the quaternion rotation as a hair-thin sliver. Middle: one recurrence lane, a state arrow rotated at each word and shrunk by a decay. Right: a layer diagram with each block marked allowed or banned.](../figures/mechanisms/geometric-stack.svg)

*Left, where one token's arithmetic went in the 29-million-parameter stack. Middle, what one recurrence lane does. Right, which parts of the trained form the later rules allow.*

## Its chapter in the project's history

- **26 to 30 August (planning a decoder).** Before building a stack, the project checked whether ordinary attention could live inside the geometry. A rotation-frame version of softmax attention matched a donor model's logits to within 1.05e-5, but it carried no meaning of its own. Trace distillation added nothing (#1002, #1011).
- **30 August.** A 7.16-million-parameter softmax language model in R4/Spin frames showed learned attention matters: loss per token 2.1274 with it, 4.8048 without. A single frozen continuation to 150 million tokens ended at 1.5728, short of its 1.50 bar (#1014, #1017).
- **25 September.** A 1.68-million-parameter recurrent learner with quaternion transport lost narrowly to a matched ordinary one (2.110 against 2.085) (#1389).
- **27 to 29 September.** A capacity-matched layered stack reached parity with a transformer control, and the owner made multiplier-free serving the one contract for every lab (D11, #1437, #1443). The stack became the main line.
- **6 to 10 October.** The first chat stack was trained, exported and served end to end (#1893). Matching the training data mattered more than adding parameters (#1913). Memory and reply targets were missed.
- **11 October.** An arithmetic audit showed the quaternion rotations were 0.05 percent of the arithmetic (#2186).
- **12 to 13 October.** The bans on softmax, matrix multiplication and squashing functions arrived (D23, D25, D29, D30). Thirty-three kernels carrying banned content were moved out of every compiled module, and a 24-kernel lawful extract was kept. The discrete path (`crates/uor-r4-discrete`) then became the live model.

## How it works

**Layer pattern (definition).** Layers repeat in a pattern such as `rrarra`. Each `r` is a recurrence layer and each `a` a read layer. Every layer is followed by a feed-forward block, inside a residual stream with RMS normalisation.

**Recurrence (definition).** Lanes of four channels hold a quaternion state $h_t$. Per token,

$$h_t = \lambda_t\,(u_t \otimes h_{t-1}) + \sqrt{1-\lambda_t^{2}}\;c_t ,$$

where $\otimes$ is the Hamilton product, $u_t$ is a unit quaternion computed from the input (it starts at the identity), $\lambda_t$ is a gated decay with timescales spread from 2 to about 1,000 tokens, and $c_t$ is a width-4 causal convolution of the input. Multiplying by a unit quaternion is a rotation of 4-space, so $u_t$ turns the memory without changing its length.

**Reads and feed-forward.** Read layers attend over earlier positions with a dot-product, Euclidean or Lorentz hyperbolic-distance score. Feed-forward blocks are SwiGLU with dense projections. Optional parts were a copy-pointer head, a product-key memory that replaced a feed-forward block or the last read, and flock or starling rank reads.

**Training and serving.** Trained in float on GPU at 7M, 19.9M, 29M and 214M parameters, then exported to 4-bit grouped weights and served by the integer engine described in [lut-serving](./lut-serving.md).

**Where the arithmetic went (measurement, an operation count of the deployed 29M stack, width 576, 10 layers).** Of 30,270,720 multiply-adds per token, the feed-forward blocks took 39.6 percent, recurrence projections 31.0 percent, reads 16.6 percent (40.5 percent at 4,096 tokens of context) and the output head 7.4 percent. The quaternion rotations took 16,128 multiplies, or 0.05 percent. The prime addresses, zeta phases and exact store took none, because they were not in the trained base ([geometry audit](../../archive/integration/geometry-audit-2026-10-11/README.md)).

**Rule check.**

| Rule | Verdict for the trained form |
|---|---|
| No softmax or squashing anywhere ([D23](../integration/DECISIONS.md#d23---no-softmax-anywhere-in-the-active-project)) | Broken. Softmax reads, a sigmoid decay gate, a softplus-shaped decay rate, SiLU in the feed-forward block and a GELU output gate |
| No matrix multiplication ([D30](../integration/DECISIONS.md)) | Broken. Dense projections and feed-forward blocks are matrix products, and the old kernels used tensor cores |
| Integer serving ([D11](../integration/DECISIONS.md#d11--native-multiplier-free-transformerless-serving-for-every-lab-d10-superseded)) | Served bit-for-bit against its quantised reference, but every weight is read for every token (about 30.2 million parameters at 29M), which the sparsity rule calls non-compliant |
| Offline float, with an export gap ([D37](../integration/DECISIONS.md#d37--floating-point-is-legal-in-offline-training-only-d32s-learning-path-float-ban-is-revised)) | Float training would be legal again only in a separately scanned module with lawful parts. Past export gaps: 0.00883 bits per byte on identical positions for the 19.9M stack, and +0.0257 nats with GPTQ on the 7M stack |

## What has been tried

"Development" means a panel used while fitting. A "control" is a matched comparison model; transformer controls were allowed when these runs were made and are not allowed now (D28).

| Date | What was run | Result | Verdict | Record |
|---|---|---|---|---|
| 26 Aug | One geometric mixer layer inside SmolLM2-135M, then distilled with a memory adapter | Decodable, cycle-free output (32 tokens, 4 of 5 prompts responsive). The distilled version scored a 4.4416% advantage against a 5% bar on 9 held-out positions | positive, then negative | [spike](../../archive/geometric_decoder_spike_950.md), [qualification](../../archive/geometric_mixer_qualification_951.md) |
| 30 Aug | Trace distillation into a geometric recurrent suffix | Frozen suffix 2.660721 against geometric 2.660705; nothing added on 9 positions | negative | [record](../../archive/r4_softmax_trace_state_student_1011.md) |
| 30 Aug | 7.155M R4/Spin softmax model, then one continuation to 150M tokens | 2.1274 with attention, 4.8048 without; sealed 1.5728 against a 1.50 ceiling. Attention kept, campaign closed | negative | [record](../../archive/r4_softmax_quality_capacity_continuation_1017.md) |
| Aug to Sep | Prompt-capacity rungs, and multi-query recall | Paired-H4 addresses won 282 of 512 against 308 needed. A copied recall cell scored 11,900 of 12,000, so earlier binding failures came from the recipe | negative | [capacity](../../archive/r4_paired_h4_prompt_capacity_973.md), [recall](../../archive/r4_zoology_release_reproduction_1050.md) |
| 25 Sep | 1.68M recurrent learner, quaternion against matched ordinary transport, one seed | 2.110368 against 2.085241; prose stayed poor | inconclusive | [record](../../archive/integration/joint-recurrent-result-2026-09-25.md) |
| 27 Sep | Capacity-matched `rrarra` stack against a transformer control, 7,324 updates, one seed | 1.998113 against 2.011149 and equal speed. The stack led early; the lead closed by about 2,750 updates. Parity, not advantage | positive (parity) | [record](../../archive/integration/geometric-stack-cycle4-2026-09-27.md) |
| 28 Sep | Quaternion against identity transport, 1,000 updates, two seeds | Gap 0.0711 and 0.0774 nats against a 0.02 rule. Development only; the core still held banned parts and the identity arm had 332,928 fewer parameters | positive | [record](../../archive/integration/transport-attribution-d1-2026-09-28.md) |
| 28 Sep | Export of the 7.15M stack to 4-bit, and other rounding recipes | +0.0362 (nearest) and +0.0257 (GPTQ) nats against a 0.02 gate. Snapping rotations to icosians cost +0.026 | negative | [record](../../archive/integration/s1-stack-serving-measurements-2026-09-28.md) |
| 28 Sep | First dialogue-trained stack | Loss beat its bar (2.520916 against 2.773887), but 2 of 38 replies answered the request and 0 of 10 memory recalls worked | positive on loss | [record](../../archive/integration/s2-dialogue-stack-baseline-2026-09-28.md) |
| 29 Sep | E8 lattice coding of SmolLM2-360M weights | Kill rule triggered at 3 bits (+4.96 nats). Negative for this uncompensated version only | negative | [record](../../archive/integration/b3-e8-smollm2-result-2026-09-29.md) |
| 6 Oct | Geometric read stack against a matched transformer, 3 seeds, multi-query recall | Geometric 0.0073, 0.9995, 1.0000 against control 0.2529, 0.2510, 0.2563. One failed seed, so the pre-registered rule returned inconclusive | inconclusive | [report](../../archive/labs/d20-run-2026-10-06/d20-run-report.md) |
| 8 Oct | 19.9M chat stack trained, exported, served end to end | Served 0.9550 bits per byte on 64 windows (the previous served artifact scored 4.3885 on the same corpus). The float figure 0.8770 is a 512-window number and is not comparable; on identical positions the rounding cost is 0.00883. Earlier null results had used models below uniform | positive | [record](../../archive/history/github-archive-2026-10-15/markdown/1893-a-chat-trained-geometric-stack-served-multiplier-free-0-9550.md) |
| 8 Oct | Data against size | In-domain data was worth 0.159 bits per byte, extra size 0.084. A 214M stack (1.0966) lost to a 7.16M chat-only one (0.9607) | positive | [record](../../archive/history/github-archive-2026-10-15/markdown/1913-size-at-10-the-214m-chat-stack-1-0966-loses-to-a-7-16m-chat.md) |
| 9 to 10 Oct | 29M chat stack on the memory panel, and recall-share doses | 10 of 40 against at least 34. In the doses, memory rose from 10 to 20 of 40 at a 10% recall share, but replies fell from 43 to 22 of 232 as the share rose to 55% | negative | [v5](../../archive/labs/v5-declared-acceptance-2026-10-09/README.md), [doses](../../archive/labs/mixture-dose-2026-10-10/README.md) |
| 10 Oct | Product-key memory in 4 of 10 layers, 2 seeds | 19 of 40 on both against a 27 of 40 bar | negative | [record](../../archive/labs/addressed-memory-2026-10-10/README.md) |
| 11 Oct | Product-key memory at the last read layer (a correction note says it replaced the feed-forward block there), 3 seeds; then a lean codebook | Wrong-value answers 47 to 34 of 120 with the headline flat at 57 of 120. Cost: 20 times the training time. Lean version lost 0.095 bits per byte against 0.02 allowed | positive, then negative | [fat](../../archive/labs/addressed-read-2026-10-11/README.md), [lean](../../archive/labs/addressed-read-lean-2026-10-11/README.md) |
| 11 Oct | Arithmetic ledger of the deployed 29M stack | Geometry was 0.05% of the arithmetic | diagnostic | [record](../../archive/integration/geometry-audit-2026-10-11/README.md) |

**Reading these honestly.** The stack reached parity with a transformer control at small scale, and it was served bit-for-bit through an integer engine. The quaternion transport kept its place in the core on a development measurement. But the pieces that did the work were dense ones, the recurrence was never shown to help in a model built from allowed parts, and no run met the memory or quality bars. A loss win was not a conversation.

## What has never been tested

- A stack built only from allowed parts (Hamilton-gate feed-forward, power-of-two recurrence, rank or flock reads), trained at the scale where this one reached 0.877 bits per byte. The 24-kernel extract is qualified at small finite checks only.
- Product-key memory layers with fixed H4 and E8 codebooks (cycle 5; held, never run).
- Trained-in icosian or E8 read addresses with their length carried.
- The Lorentz read at chat scale. The chat base trained with the flat Euclidean read.
- A larger chat data set than the 82-million-token split (none was reachable).
- A fair energy measurement. The only figures come from a truncated capture and a table-emulation test that cost 4.3 times more.
- A step-matched control for the 22 of 40 reading that the half-length memory run produced.

## What would reopen it

1. **A new pre-registered sheet that rebuilds the stack from allowed parts only**, in its own scanned module, with the owner's release (D37). It needs a `TEST FITNESS: FIT` review first.
2. **Report the export gap** on the same evaluator as the float score, and aim at the served bar (at most 0.90 bits per byte on the chat held-out stream).
3. **At least 2 seeds or 200 fresh rows** (D22), compared only with earlier native rungs and count-table references (D28).
4. **Replace the dense hotspots** (feed-forward 39.6%, recurrence projections 31.0%) with addressed or exact-geometry reads, and show that tokens per second move on the product path.
5. **Optional:** the quaternion adapter in the post-dose plan ([optimization path](../labs/optimization-path-2026-10-18/README.md)), conditional and measured against the ceiling.

## Also known as

rrarra / rrarrarr / rrarrarrar stack; cycle-4 stack (`geometric_s1`); chat-29m-B; the 19.9M chat stack; 214M pretrain `geo-214m-e448de86`; R4/Spin softmax reference (#1014, #1017); joint recurrent learner (the 1.68M precursor); `StackArch::Geometric`; `geometric_stack_native` (the lawful extract).

## Code

- [`geometric_stack.rs`](../../crates/uor-r4-training/src/geometric_stack.rs): the historical float stack (36,976 lines). Inactive source.
- [`geometric_stack_native.rs`](../../crates/uor-r4-training/src/geometric_stack_native.rs) and [`geometric_stack_native_cuda.rs`](../../crates/uor-r4-training/src/geometric_stack_native_cuda.rs): the qualified extract. Only the allowed projection, recurrence and shared writer run (24 prebuilt kernels).
- [`cuda_stack.cu`](../../crates/uor-r4-training/src/cuda_stack.cu): the historical kernel source. Thirty-three banned kernels are excluded from every compiled module.
- [`stack_memory.rs`](../../crates/uor-r4-training/src/stack_memory.rs): the product-key memory. [`stack_export.rs`](../../crates/uor-r4-training/src/stack_export.rs): the exporter, compiled out.
- [`joint_model.rs`](../../crates/uor-r4-training/src/joint_model.rs): the September precursor. [`chat-grade.rs`](../../crates/uor-r4-training/src/bin/chat-grade.rs): the chat and memory scorer.
- [`geometric-stack-native.rs`](../../crates/uor-r4-training/examples/geometric-stack-native.rs): the current example entry.
- [`session.rs`](../../crates/uor-r4-integer/src/stack/session.rs): the integer engine that served these stacks.
- Evidence: [served stacks](../../archive/labs/d11-serving-2026-10-10/README.md), [module split](../../archive/labs/d29-module-split-2026-10-13/README.md), [cycle 4](../../archive/integration/geometric-stack-cycle4-2026-09-27.md).

## Related mechanisms

[lut-serving](./lut-serving.md): the engine that served these stacks bit-for-bit. [spin-weights](./spin-weights.md): icosian-pair weights, the allowed form of the stack's quaternion maps. [recurrence-gates](./recurrence-gates.md): the allowed form of the gated recurrence. [hamilton-gate](./hamilton-gate.md): the rotation gate that replaces the SwiGLU block. [flock-read](./flock-read.md) and [route-read](./route-read.md): softmax-free reads for the attention layers. [exact-memory](./exact-memory.md) and [e8-orbit-memory](./e8-orbit-memory.md): stores meant to replace the dense feed-forward block and read. [read-binding](./read-binding.md) and [pointer-key](./pointer-key.md): the supervised read and copy head added here. [spiralcore-fbs](./spiralcore-fbs.md): the projection the extract runs. [rank-margin](./rank-margin.md): the allowed loss. [native-geometric](./native-geometric.md): the parallel from-scratch engine. [r4g1-graph](./r4g1-graph.md): the earlier approach this stack replaced. [discrete-learning](./discrete-learning.md): the live model that took over.
