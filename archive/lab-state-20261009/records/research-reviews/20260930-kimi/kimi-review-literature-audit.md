# Primary-source audit of the Kimi external synthesis

Reviewed 2026-09-30. Scope: literature, numerical execution, hardware measurements and feasible training implications. Input: the owner's `Pasted text.txt` attachment. This is independent source verification, not a model run or validation of the attachment's repository-specific artifact claims. No training, paid provider call, repository build or GitHub mutation was performed. Author conclusions and this reviewer's inferences are separated below.

**Finding:** the synthesis identifies useful research directions, but its claim that external work has solved *chat with an entirely multiplier-free, floating-point-free, transformer-free serving path* is not supported. It combines different models, numerical contracts, evaluation types and hardware measurements. Preserve the useful mechanisms; correct the scope before using this review as a development mandate.

## Claim-by-claim findings

### BitNet b1.58 2B4T: score correct, execution conclusion incorrect

Microsoft's [official model card](https://huggingface.co/microsoft/bitnet-b1.58-2B-4T) reports MT-bench **5.85**, four trillion pretraining tokens, followed by SFT and DPO. It explicitly describes a **Transformer-based** architecture with RoPE, normalization and BitLinear. The comparison is against similarly sized instruction-tuned open models, not contemporary frontier reasoning equivalence. The 0.4GB figure is **non-embedding** memory, not complete process memory.

The [technical report, 2025-04-16](https://arxiv.org/html/2504.12285v1) reports CPU decode measurements on an Intel i7-13800H Surface Laptop Studio 2, eight threads, generating 128 tokens. BitNet uses bitnet.cpp while comparators use llama.cpp. This is a useful deployment result; it is neither an M1 measurement nor an instruction audit of the complete runtime.

Direct official-code verification is stronger than inferring an operation set from “LUT.” At inspected BitNet commit `0b341e582afbf9e1011f24744b554c96a3477eb5`:

- [TL1 preset, lines 120–125](https://github.com/microsoft/BitNet/blob/0b341e582afbf9e1011f24744b554c96a3477eb5/preset_kernels/bitnet_b1_58-3B/bitnet-lut-kernels-tl1.h#L120) performs floating-point activation scaling with `vmulq_n_f32`.
- [TL2 preset, lines 116–118](https://github.com/microsoft/BitNet/blob/0b341e582afbf9e1011f24744b554c96a3477eb5/preset_kernels/bitnet_b1_58-3B/bitnet-lut-kernels-tl2.h#L116) uses `_mm256_mul_ps` before integer conversion.
- [The shared header](https://github.com/microsoft/BitNet/blob/0b341e582afbf9e1011f24744b554c96a3477eb5/include/ggml-bitnet.h#L8) defines `bitnet_float_type` as a floating-point type; [the LUT dispatcher](https://github.com/microsoft/BitNet/blob/0b341e582afbf9e1011f24744b554c96a3477eb5/src/ggml-bitnet-lut.cpp#L108) passes floating scales/biases.

These are source observations, not disassembly of a particular benchmark binary. They nevertheless directly refute blanket claims that TL1/TL2 contain no floating-point multiplication. Integer lookup/accumulation in the central weight kernel must be distinguished from scaling, attention, normalization, embeddings, logits and sampling. The exact 2B4T backend/configuration must also be named rather than assigning all BitNet results to TL1/TL2.

### T-MAC: throughput real, machine and whole-path scope matter

The [T-MAC paper](https://arxiv.org/html/2407.00088v2), first submitted July 2024 and published at EuroSys 2025, reports **30 tokens/s on one M2 Ultra core and 71 on eight** for BitNet-b1.58-3B. Model-level testing integrates T-MAC into llama.cpp and repeatedly generates 64 tokens for 20 iterations. BitNet is executed using two-bit storage. This is model-level decoding throughput, not merely a microkernel rate.

It is an **M2 Ultra** result, not an M1 laptop promise. Its platform table lists 819.2GB/s maximum memory bandwidth. T-MAC supports mixed precision and describes floating-point table values, quantization scales, and conversion of accumulated results to float16. Its reduction of low-bit matrix products to lookup does not remove every operation in the transformer. [Official implementation](https://github.com/microsoft/T-MAC) also describes mixed-precision kernels. The transferable result is efficient packed-weight lookup with carefully tuned cache/register layout and measured end-to-end costs.

### MatMul-free LM: promising recurrence, not a demonstrated chat system

[Zhu et al., June 2024; updated July 2025](https://arxiv.org/html/2406.02528v7) compare the **2.7B** model trained on **100B tokens** with Transformer++: six-task zero-shot average **49.9 versus 50.7**. This is not instruction/chat evaluation. Dense ternary transforms coexist with elementwise products and nonlinear normalization; “MatMul-free” does not mean “no multiplication instruction.”

The **59.4 tokens/s, 70.8mJ/token Loihi 2** figure concerns **370M**, with W8 normalization parameters and A16 activations, not that 2.7B model. Crucially, Appendix C.4.1 derives complete-model latency from **single-block** hardware measurements; system figures incorporate an approximately 20% inter-chip adjustment. Tables distinguish these derived figures from the edge-GPU measurements. Do not present this as a measured complete 2.7B chat deployment or laptop result. The hardware section appeared in later revisions; [arXiv records v7 on 2025-07-25](https://arxiv.org/abs/2406.02528).

The earlier [v3, section 3.3](https://arxiv.org/html/2406.02528v3) reports non-convergence for its **specific 370M quantized-Q/K transformer experiment**. It establishes a negative result for that recipe, not impossibility for every quantized attention mechanism.

### “QK cannot be ternary” and “integer softmax is the unique missing trick”: unsupported

These are distinct questions: how Q/K are represented, how their similarity is computed, whether the resulting scores are quantized, and how scores are normalized. As a mathematical observation, ternary vectors of width d produce integer dot products in [-d,d]; the score is not automatically ternary. A fixed-point softmax does not repair information erased by a coarse query/key representation.

[ShiftAddViT, NeurIPS 2023](https://arxiv.org/abs/2306.06446) does provide a counterexample to a blanket inability to replace attention products: binary Q/K codes and additive attention kernels work in its vision tasks. Its final MLP design includes a mixture of multiplication and shift experts. It does **not** establish language-model success or a wholly multiplication-free model.

[I-BERT, ICML 2021](https://proceedings.mlr.press/v139/kim21d.html) already implements end-to-end integer inference including approximate softmax and normalization, evaluated on RoBERTa/GLUE. It retains integer multiplication and is not a chat model, but disproves novelty of integer-only softmax as a general external mechanism. “Exact” must name the integer specification: finite lookup/rounding can be exact relative to that specification without equaling the real exponential or arbitrary floating-point parent.

### Spectra/TriLM: scale claim correct, architecture and precision still mixed

[Spectra](https://arxiv.org/html/2407.12327v3) studies 99M–3.9B models trained on **300B tokens**. Its largest TriLM is competitive with its 3.9B FloatLM comparison across the reported benchmarks. TriLM is explicitly a **LLaMA-style autoregressive transformer**, with RMSNorm, SwiGLU, RoPE and multihead attention. Linear matrices use ternary states plus a floating scale; embeddings and the language-model head remain half precision. This supports useful ternary weight capacity. It does not establish a float-free, transformer-free serving graph or a chat-quality threshold at 3.9B.

### Native QAT is evidence-backed; universal PTQ failure is not

The useful distinction is **training/adaptation with the deployed discretization in the forward graph** versus exporting coarse codes that the learner never experienced. Merely doing floating-point autodiff and exporting integer weights is not automatically equivalent to QAT. Whether UOR's exact selected path meets that condition requires its own source/gradient/artifact audit.

[ParetoQ](https://arxiv.org/html/2502.02631v2), first submitted February 2025, directly studies full-precision pretraining followed by QAT. Its budget experiment peaks near **90% full-precision pretraining / 10% QAT** across its tested settings. Thus “must train ternary from scratch” is too strong. This does not promise an arbitrary pretrained model can be converted cheaply or that the ratio transfers unchanged to UOR.

[PT²-LLM](https://arxiv.org/html/2510.03267v1), submitted October 2025 and already historical at this review date, supplies an actual training-free ternarization counterexample. It uses calibration, asymmetric grids, reordering and compensation. Quality still drops substantially: its LLaMA-2-7B seven-task average is **43.33 versus FP16 61.85**, with WikiText2 PPL **11.56 versus 5.47**; LLaMA-3-8B is worse. This refutes universal impossibility, not the practical risk of aggressive PTQ. Its grid scales/offsets and execution representation also need independent serving-cost qualification.

The cited [HF Llama3-8B-1.58 model card](https://huggingface.co/HF1BitLLM/Llama3-8B-1.58-100B-tokens) describes **fine-tuning from Llama-3-8B-Instruct for a total of 100B tokens**, including a lambda schedule. It is not a training-free PTQ experiment. Its reported residual quality gap is useful evidence about that conversion/adaptation campaign, not a universal PTQ law.

## Memory: useful external evidence, but exact identity is not what all these systems test

| Primary source | Actual retrieval mechanism | What transfers / what does not |
|---|---|---|
| [kNN-LM, 2019/ICLR 2020](https://arxiv.org/abs/1911.00172) | Nearest neighbors in a pretrained LM's embedding space, interpolated with neural probabilities. | Supports external datastore value; learned similarity retrieval is not prime identity lookup. Index construction, storage and search remain costs. |
| [Memorizing Transformers, 2022](https://arxiv.org/abs/2203.08913) | Approximate kNN over nondifferentiable recent key/value activations; tested up to 262K memory tokens. | Supports inference-time memory, but neither collision-free addressing nor replacing its transformer backbone. |
| [Memory Layers at Scale, 2024/2025](https://arxiv.org/html/2412.09764v2) | Learned product keys, top-k similarity and weighted values; searches two sets of approximately sqrt(N) half-keys. | Supports sparse parameter access and capacity/compute separation. It is not a fixed exact hash lookup; optimizer state and distributed memory traffic are material. |

[Engram, submitted 2026-01-12](https://arxiv.org/html/2601.07372v1), is real, not a future citation. Its **MMLU +3.4** compares Engram-27B with an iso-parameter/iso-FLOPs MoE model; experiments use 262B training tokens. It augments a **transformer** with hashed normalized n-grams and contextual gates. Token normalization is surjective; multiple heads mitigate **hash collisions**; hashes are multiplicative-XOR and table sizes prime. Fusion uses learned projections, RMSNorm, sigmoid and convolution, including FP8 matrix multiplication. Thus deterministic table indexing is not exact preservation of occurrence/version/token identity or a prime-derived semantic metric.

Engram's **O(1)** refers to a fixed collection of indexed n-gram lookups, not arbitrary-history retrieval or full-model cost. Its reported less-than-3% overhead for offloading a 100B-parameter table was measured with **H800, 512 sequences, lengths 100–1024**, overlapping transfer with dense compute. That does not establish inexpensive cold SSD lookup or low single-user laptop latency. Its worthwhile lesson is predictable sparse reads plus learned contextual integration, to be retested under UOR's own information and cost controls.

## Consequences for the next UOR decisions — reviewer inference

1. **Keep three acceptance axes independent:** model capability, discrete numerical fidelity, and complete-path physical cost. None of the cited papers authorizes promoting one from evidence about another. Instruction quality, reasoning, teacher fidelity and opcode compliance need separate evidence.
2. **Do not jump to a 1–3B/100B-token campaign from these citations.** They demonstrate sufficient resources for particular results, not necessary/sufficient thresholds. The BitNet chat result uses 4T tokens plus post-training. “100B tokens” is not a discovered minimum for chat or a guarantee that a modified recurrent model will work.
3. **Use an explicit resource estimate before proposing scale.** A rough dense-training planning model, 6ND operations, gives 6e20–1.8e21 operations for 1–3B parameters and 100B tokens. At a hypothetical sustained 1e12 operations/s this is about 19–57 years; at 1e13, about 1.9–5.7 years. These are arithmetic scenarios, not measured M1 runtimes; architecture, sparsity, kernels, utilization and counting conventions change them. They expose why a measured small curriculum/teacher-adaptation ladder is the actionable next step.
4. **Training memory differs radically from packed serving memory.** A typical full-FP32 Adam configuration with weights, gradients and two moments alone uses roughly 16 bytes/parameter: 16–48GB for 1–3B, before activations, temporary tensors and other states. Optimizer changes can reduce this. A 1.58-bit inference label does not grant 1.58-bit training.
5. **Test slots, multiple reads and geometry one change at a time.** Slots can preserve independent facts; more heads can improve access; neither automatically identifies the relevant slot, learns updates or guarantees language generation. Compare learned routing with direct-index oracle, ordinary content similarity and matched-capacity ordinary parameter sharing. Include index-build/update, selected bytes, misses, collision handling and cold-cache costs.
6. **An integer softmax is useful infrastructure, not a selector.** Freeze a known-correct numerical reference, then test whether the learned query selects the right memory under distractors, paraphrase, role reversal and conflicting updates. Preserve exact factual identity alongside approximate features. Do not modify both representation and evaluator to obtain a pass.
7. **Treat quaternion sharing and fixed phases as hypotheses with controls.** The requested audit did not verify the attachment's QLSTM, GHRR or GATr claims; neither “exactly three legitimate niches” nor “never quality gains” follows from the sources audited here. RoPE's success would not by itself establish zeta frequencies as superior. Keep shared-parameter real controls and equal-cost alternative phase schedules.
8. **Do not confuse storage address compression with information creation.** Lossless indexing can remove redundant representations and sparse access can reduce work; neither can reconstruct distinctions discarded by a state encoding without retained information. Learned geometry needs a measurable information/routing benefit at matched resources, not only algebraic elegance.

The literature supports a bounded programme combining native low-bit learning/adaptation, explicit external memory, and economical numerical kernels. It does not support retiring an entire geometric family from one control result, declaring the serving problem universally solved, or presuming a billion-parameter architecture is the next feasible local experiment.

## Coverage and limitations

Primary papers, author/model cards and official source repositories only were used as evidence. Search-engine relative dates were ignored in favor of arXiv/conference/model records. The June-2024 and July-2025 versions of MatMul-free LM were distinguished explicitly. Source reads were scoped; no claim is made to have executed or exhaustively audited the external repositories. Repository-specific assertions about 4,096 distribution hashes, an accepted source/artifact, current gradient paths and the exact current UOR read geometry require a separate live source/artifact audit. This document supplies no model-quality receipt for UOR.
